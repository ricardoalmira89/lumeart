//! `anim.js` con QuickJS: el porte de `internal/anim/engine.go` de pxcli.
//!
//! Se ejecuta `prelude.js` y después `anim.js` (3 s como mucho); luego se
//! dibuja el reposo y cada fotograma de cada acción (700 ms por fotograma).
//! Cada fotograma empieza en limpio, con `rand()` sembrado por acción y
//! fotograma, y se vuelve a dibujar al revés para avisar si el código guarda
//! estado entre fotogramas. Sin archivos ni red: la base la carga quien
//! llama. Los fallos vuelven como avisos con su línea.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use lume_app::ports::{AnimEngine, BaseLoader};
use lume_domain::Rgba;
use lume_domain::anim::{
    Action, ActionSpec, AnimResult, Image, Level, MAX_ACTIONS, MAX_CELL, MAX_FRAMES, PartCall,
    same_image,
};
use lume_domain::canvas::bresenham;
use lume_domain::error::go_quote;
use rquickjs::context::EvalOptions;
use rquickjs::function::Rest;
use rquickjs::{Coerced, Context, Ctx, Exception, FromJs, Function, Object, Runtime, Value};

/// Funciones de apoyo de todo `anim.js` (las de pxcli).
pub const PRELUDE: &str = include_str!("prelude.js");

/// Tiempo para cargar el script.
pub const LOAD_TIMEOUT: Duration = Duration::from_secs(3);
/// Tiempo para dibujar un fotograma.
pub const FRAME_TIMEOUT: Duration = Duration::from_millis(700);
/// Píxeles que puede pintar un fotograma.
pub const MAX_OPS_PER_FRAME: usize = 4_000_000;

const TIMEOUT_MESSAGE: &str = "se ha superado el tiempo límite (¿un bucle infinito?)";

/// El motor de animación de LumeArt.
#[derive(Debug, Clone, Copy, Default)]
pub struct QuickJsAnim;

impl AnimEngine for QuickJsAnim {
    fn render(&self, source: &str, load_base: BaseLoader) -> AnimResult {
        render(source, load_base)
    }
}

#[derive(Debug, Clone, Copy)]
struct Xform {
    dx: i64,
    dy: i64,
    flip: bool,
    /// Espejo: x' = axis2 - x.
    axis2: i64,
}

impl Xform {
    fn shift(dx: i64, dy: i64) -> Xform {
        Xform {
            dx,
            dy,
            flip: false,
            axis2: 0,
        }
    }
}

struct State<'js> {
    res: AnimResult,
    load_base: BaseLoader,
    base_name: String,
    parts: HashMap<String, Function<'js>>,
    rest: Option<Function<'js>>,
    /// La función de cada acción, en el orden de `res.actions`.
    action_fns: Vec<Function<'js>>,
    // Del fotograma en curso.
    surf: Image,
    stack: Vec<Xform>,
    oob: usize,
    ops: usize,
    seed: u32,
    colors: HashMap<String, Rgba>,
    /// Las piezas que dibuja el fotograma en curso.
    calls: Vec<PartCall>,
}

type Shared<'js> = Rc<RefCell<State<'js>>>;

/// FNV-1a de 32 bits.
fn fnv32a(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in bytes {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// mulberry32: el generador sembrado de pxcli (se repite igual por fotograma).
fn next_rand(seed: &mut u32) -> f64 {
    *seed = seed.wrapping_add(0x6D2B_79F5);
    let mut t = *seed;
    t = (t ^ (t >> 15)).wrapping_mul(t | 1);
    t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
    f64::from(t ^ (t >> 14)) / 4_294_967_296.0
}

/// Línea de `anim.js` en una pila de QuickJS («at f (anim.js:3:5)»).
fn stack_line(stack: &str) -> u32 {
    let Some(i) = stack.find("anim.js:") else {
        return 0;
    };
    stack[i + "anim.js:".len()..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

/// Un error de JavaScript ya recogido: su línea y su mensaje.
struct JsError {
    line: u32,
    message: String,
    /// Se cortó por tiempo.
    timeout: bool,
}

/// quickjs-ng comprueba el tiempo en los saltos de los bucles sin guardar
/// antes dónde va, así que la pila de un corte por tiempo señala la última
/// llamada de la función (o su comienzo), no el bucle. Se busca el primer
/// bucle (`while`, `for`, `do`) desde esa línea para señalarlo, como pxcli.
fn loop_line(source: &str, line: u32) -> u32 {
    let start = line.max(1) as usize - 1;
    source
        .lines()
        .enumerate()
        .skip(start)
        .take(200)
        .find(|(_, text)| {
            text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .any(|word| matches!(word, "while" | "for" | "do"))
        })
        .map(|(i, _)| i as u32 + 1)
        .unwrap_or(line)
}

/// Recoge la excepción pendiente (como `errLine` y `cleanErr` de pxcli).
fn take_error(ctx: &Ctx<'_>, interrupted: &Cell<bool>) -> JsError {
    let value = ctx.catch();
    if interrupted.replace(false) {
        let line = value
            .as_exception()
            .and_then(|e| e.stack())
            .map(|s| stack_line(&s))
            .unwrap_or(0);
        return JsError {
            line,
            message: TIMEOUT_MESSAGE.into(),
            timeout: true,
        };
    }
    let (line, mut text) = match value.as_exception() {
        Some(ex) => {
            let line = ex.stack().map(|s| stack_line(&s)).unwrap_or(0);
            let name: Option<String> = ex
                .get::<_, Option<Coerced<String>>>("name")
                .ok()
                .flatten()
                .map(|c| c.0);
            let message = ex.message().unwrap_or_default();
            let text = match name {
                Some(name) if !name.is_empty() => format!("{name}: {message}"),
                _ => message,
            };
            (line, text)
        }
        None => (
            0,
            Coerced::<String>::from_js(ctx, value)
                .map(|c| c.0)
                .unwrap_or_else(|_| "error desconocido".into()),
        ),
    };
    if let Some(i) = text.find('\n') {
        text.truncate(i);
    }
    for prefix in ["TypeError: ", "GoError: ", "Error: "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            text = rest.to_string();
        }
    }
    JsError {
        line,
        message: text.replacen("anim.js: ", "", 1).trim().to_string(),
        timeout: false,
    }
}

fn type_error(ctx: &Ctx<'_>, message: &str) -> rquickjs::Error {
    Exception::throw_type(ctx, message)
}

fn is_missing(v: &Value<'_>) -> bool {
    v.is_undefined() || v.is_null()
}

/// Argumento `i` como número entero (redondeado), o un `TypeError`.
fn num<'js>(
    ctx: &Ctx<'js>,
    args: &[Value<'js>],
    i: usize,
    func: &str,
    what: &str,
) -> rquickjs::Result<i64> {
    let v = args
        .get(i)
        .cloned()
        .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
    if is_missing(&v) {
        return Err(type_error(ctx, &format!("{func}: falta {what}")));
    }
    let f = Coerced::<f64>::from_js(ctx, v)?.0;
    if !f.is_finite() {
        return Err(type_error(ctx, &format!("{func}: {what} no es un número")));
    }
    Ok(f.round() as i64)
}

/// Texto de un valor (`String(v)` de JavaScript).
fn js_string<'js>(ctx: &Ctx<'js>, v: Value<'js>) -> rquickjs::Result<String> {
    Ok(Coerced::<String>::from_js(ctx, v)?.0)
}

fn arg<'js>(ctx: &Ctx<'js>, args: &[Value<'js>], i: usize) -> Value<'js> {
    args.get(i)
        .cloned()
        .unwrap_or_else(|| Value::new_undefined(ctx.clone()))
}

/// Argumento `i` como color (con caché por texto).
fn color<'js>(
    ctx: &Ctx<'js>,
    state: &Shared<'js>,
    v: Value<'js>,
    func: &str,
) -> rquickjs::Result<Rgba> {
    if is_missing(&v) {
        return Err(type_error(ctx, &format!("{func}: falta el color")));
    }
    let text = js_string(ctx, v)?.trim().to_string();
    if let Some(c) = state.borrow().colors.get(&text) {
        return Ok(*c);
    }
    match Rgba::parse(&text) {
        Ok(c) => {
            state.borrow_mut().colors.insert(text, c);
            Ok(c)
        }
        Err(_) => Err(type_error(
            ctx,
            &format!("{func}: color no válido {}", quote(&text)),
        )),
    }
}

/// `%q` de Go.
fn quote(text: &str) -> String {
    go_quote(text)
}

fn callback<'js>(
    ctx: &Ctx<'js>,
    args: &[Value<'js>],
    i: usize,
    func: &str,
) -> rquickjs::Result<Function<'js>> {
    args.get(i)
        .and_then(|v| v.as_function().cloned())
        .ok_or_else(|| type_error(ctx, &format!("{func}: falta la función a dibujar")))
}

impl State<'_> {
    /// Pinta un píxel a través de los movimientos y espejos en curso.
    fn plot(&mut self, mut x: i64, mut y: i64, c: Rgba) -> Result<(), &'static str> {
        self.ops += 1;
        if self.ops > MAX_OPS_PER_FRAME {
            return Err("demasiados píxeles en un frame");
        }
        for t in self.stack.iter().rev() {
            if t.flip {
                x = t.axis2 - x;
            }
            x += t.dx;
            y += t.dy;
        }
        let (w, h) = (i64::from(self.res.cell.0), i64::from(self.res.cell.1));
        if x < 0 || y < 0 || x >= w || y >= h {
            self.oob += 1;
            return Ok(());
        }
        self.surf.set(x, y, c);
        Ok(())
    }
}

fn plot<'js>(ctx: &Ctx<'js>, state: &Shared<'js>, x: i64, y: i64, c: Rgba) -> rquickjs::Result<()> {
    state
        .borrow_mut()
        .plot(x, y, c)
        .map_err(|m| Exception::throw_message(ctx, m))
}

/// Un número entero de un valor de `config` o de las opciones de `action`.
fn int_of(v: &Value<'_>) -> Option<i64> {
    v.as_number().map(|n| n.round() as i64)
}

/// `[a, b]` de dos números.
fn pair(v: &Value<'_>) -> Option<(i64, i64)> {
    let array = v.as_array()?;
    if array.len() != 2 {
        return None;
    }
    let a: Value = array.get(0).ok()?;
    let b: Value = array.get(1).ok()?;
    Some((int_of(&a)?, int_of(&b)?))
}

fn get<'js>(obj: &Option<Object<'js>>, key: &str) -> Option<Value<'js>> {
    obj.as_ref()?
        .get::<_, Value>(key)
        .ok()
        .filter(|v| !v.is_undefined())
}

/// Línea de la llamada en curso desde `anim.js` (para `action(...)`).
fn caller_line(ctx: &Ctx<'_>) -> u32 {
    Exception::from_message(ctx.clone(), "")
        .ok()
        .and_then(|e| e.stack())
        .map(|s| stack_line(&s))
        .unwrap_or(0)
}

type NativeResult<'js> = rquickjs::Result<Value<'js>>;

fn install<'js>(ctx: &Ctx<'js>, state: &Shared<'js>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    // Las funciones de Math como en goja (el motor de pxcli).
    crate::gomath::install(ctx)?;
    macro_rules! native {
        ($name:literal, |$c:ident, $s:ident, $a:ident| $body:block) => {{
            let st = state.clone();
            let f = Function::new(
                ctx.clone(),
                move |$c: Ctx<'js>, args: Rest<Value<'js>>| -> NativeResult<'js> {
                    let $s = &st;
                    let $a: &[Value<'js>] = &args.0;
                    $body
                },
            )?;
            globals.set($name, f)?;
        }};
    }
    let undefined = |c: &Ctx<'js>| Value::new_undefined(c.clone());

    native!("set_pixel", |c, s, a| {
        let x = num(&c, a, 0, "set_pixel", "x")?;
        let y = num(&c, a, 1, "set_pixel", "y")?;
        let col = color(&c, s, arg(&c, a, 2), "set_pixel")?;
        plot(&c, s, x, y, col)?;
        Ok(undefined(&c))
    });
    native!("fill_rect", |c, s, a| {
        let x = num(&c, a, 0, "fill_rect", "x")?;
        let y = num(&c, a, 1, "fill_rect", "y")?;
        let w = num(&c, a, 2, "fill_rect", "ancho")?;
        let h = num(&c, a, 3, "fill_rect", "alto")?;
        let col = color(&c, s, arg(&c, a, 4), "fill_rect")?;
        for yy in y..y.saturating_add(h) {
            for xx in x..x.saturating_add(w) {
                plot(&c, s, xx, yy, col)?;
            }
        }
        Ok(undefined(&c))
    });
    native!("line", |c, s, a| {
        let col = color(&c, s, arg(&c, a, 4), "line")?;
        let x1 = num(&c, a, 0, "line", "x1")?;
        let y1 = num(&c, a, 1, "line", "y1")?;
        let x2 = num(&c, a, 2, "line", "x2")?;
        let y2 = num(&c, a, 3, "line", "y2")?;
        for (x, y) in bresenham(x1, y1, x2, y2) {
            plot(&c, s, x, y, col)?;
        }
        Ok(undefined(&c))
    });
    native!("ellipse", |c, s, a| {
        let cx = num(&c, a, 0, "ellipse", "cx")?;
        let cy = num(&c, a, 1, "ellipse", "cy")?;
        let rx = num(&c, a, 2, "ellipse", "rx")?;
        let ry = num(&c, a, 3, "ellipse", "ry")?;
        let col = color(&c, s, arg(&c, a, 4), "ellipse")?;
        if rx < 0 || ry < 0 {
            return Ok(undefined(&c));
        }
        let (fx, fy) = (rx as f64 + 0.5, ry as f64 + 0.5);
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                let (dxf, dyf) = (dx as f64, dy as f64);
                if (dxf * dxf) / (fx * fx) + (dyf * dyf) / (fy * fy) <= 1.0 {
                    plot(&c, s, cx + dx, cy + dy, col)?;
                }
            }
        }
        Ok(undefined(&c))
    });
    native!("clear", |c, s, a| {
        let col = if a.is_empty() {
            Rgba::TRANSPARENT
        } else {
            color(&c, s, arg(&c, a, 0), "clear")?
        };
        s.borrow_mut().surf.fill(col);
        Ok(undefined(&c))
    });
    native!("outline", |c, s, a| {
        let col = color(&c, s, arg(&c, a, 0), "outline")?;
        let mut st = s.borrow_mut();
        let surf = &st.surf;
        let opaque = |x: i64, y: i64| surf.get(x, y).a > 0;
        let mut points = Vec::new();
        for y in 0..i64::from(surf.height()) {
            for x in 0..i64::from(surf.width()) {
                if !opaque(x, y)
                    && (opaque(x + 1, y)
                        || opaque(x - 1, y)
                        || opaque(x, y + 1)
                        || opaque(x, y - 1))
                {
                    points.push((x, y));
                }
            }
        }
        for (x, y) in points {
            st.surf.set(x, y, col);
        }
        Ok(undefined(&c))
    });
    native!("base", |c, s, _a| {
        let Some(base) = s.borrow().res.base.clone() else {
            return Err(type_error(&c, "base(): no hay dibujo base"));
        };
        for y in 0..i64::from(base.height()) {
            for x in 0..i64::from(base.width()) {
                let p = base.get(x, y);
                if p.a > 0 {
                    plot(&c, s, x, y, p)?;
                }
            }
        }
        Ok(undefined(&c))
    });
    native!("move", |c, s, a| {
        let dx = num(&c, a, 0, "move", "dx")?;
        let dy = num(&c, a, 1, "move", "dy")?;
        let cb = callback(&c, a, 2, "move")?;
        s.borrow_mut().stack.push(Xform::shift(dx, dy));
        let result = cb.call::<_, Value>(());
        s.borrow_mut().stack.pop();
        result?;
        Ok(undefined(&c))
    });
    native!("flip_x", |c, s, a| {
        let cb = callback(&c, a, 0, "flip_x")?;
        let mut axis2 = {
            let st = s.borrow();
            match &st.res.base {
                Some(base) => i64::from(base.width()) - 1,
                None => i64::from(st.res.cell.0) - 1,
            }
        };
        let v = arg(&c, a, 1);
        if !v.is_undefined() {
            axis2 = (2.0 * Coerced::<f64>::from_js(&c, v)?.0).round() as i64;
        }
        s.borrow_mut().stack.push(Xform {
            dx: 0,
            dy: 0,
            flip: true,
            axis2,
        });
        let result = cb.call::<_, Value>(());
        s.borrow_mut().stack.pop();
        result?;
        Ok(undefined(&c))
    });
    native!("rand", |c, s, _a| {
        let r = next_rand(&mut s.borrow_mut().seed);
        Ok(Value::new_float(c.clone(), r))
    });
    native!("config", |c, s, a| {
        let cfg = arg(&c, a, 0).into_object();
        if let Some(name) =
            get(&cfg, "name").and_then(|v| v.as_string().and_then(|s| s.to_string().ok()))
        {
            s.borrow_mut().res.name = name;
        }
        if let Some(name) =
            get(&cfg, "base").and_then(|v| v.as_string().and_then(|s| s.to_string().ok()))
            && !name.is_empty()
        {
            let loader = s.borrow().load_base.clone();
            s.borrow_mut().base_name = name.clone();
            match loader(&name) {
                Ok(img) => s.borrow_mut().res.base = Some(img),
                Err(e) => {
                    return Err(type_error(
                        &c,
                        &format!("config: no se puede abrir la base {}: {e}", quote(&name)),
                    ));
                }
            }
        }
        {
            let mut st = s.borrow_mut();
            if let Some(size) = st.res.base.as_ref().map(Image::size) {
                st.res.cell = size;
            }
        }
        if let Some((w, h)) = get(&cfg, "cell").and_then(|v| pair(&v)) {
            let max = i64::from(MAX_CELL);
            if w < 1 || h < 1 || w > max || h > max {
                return Err(type_error(
                    &c,
                    &format!("config: cell debe estar entre 1 y {MAX_CELL}"),
                ));
            }
            s.borrow_mut().res.cell = (w as u32, h as u32);
        }
        if let Some(origin) = get(&cfg, "origin").and_then(|v| pair(&v)) {
            s.borrow_mut().res.origin = origin;
        }
        if let Some(list) = get(&cfg, "colors").and_then(|v| v.as_array().cloned()) {
            for item in list.iter::<Value>() {
                let item = item?;
                let text = item
                    .as_string()
                    .and_then(|s| s.to_string().ok())
                    .unwrap_or_default();
                match Rgba::parse(&text) {
                    Ok(col) => s.borrow_mut().res.colors.push(col),
                    Err(_) => {
                        return Err(type_error(
                            &c,
                            &format!("config: color no válido {}", quote(&text)),
                        ));
                    }
                }
            }
        }
        Ok(undefined(&c))
    });
    native!("part", |c, s, a| {
        let name = js_string(&c, arg(&c, a, 0))?;
        if s.borrow().parts.contains_key(&name) {
            return Err(type_error(
                &c,
                &format!("part: la pieza {} está repetida", quote(&name)),
            ));
        }
        let cb = callback(&c, a, 1, "part")?;
        let mut st = s.borrow_mut();
        st.res.parts.push(name.clone());
        st.parts.insert(name, cb);
        Ok(undefined(&c))
    });
    native!("draw", |c, s, a| {
        let name = js_string(&c, arg(&c, a, 0))?;
        let Some(part) = s.borrow().parts.get(&name).cloned() else {
            return Err(type_error(
                &c,
                &format!("draw: no existe la pieza {}", quote(&name)),
            ));
        };
        let mut params = arg(&c, a, 1);
        if is_missing(&params) {
            params = Object::new(c.clone())?.into_value();
        }
        // Firma de la llamada: parámetros y movimientos en curso.
        let json = c
            .json_stringify(params.clone())
            .ok()
            .flatten()
            .and_then(|j| j.to_string().ok())
            .unwrap_or_default();
        {
            let mut st = s.borrow_mut();
            let moves: Vec<String> = st
                .stack
                .iter()
                .map(|t| format!("{},{},{},{}", t.dx, t.dy, t.flip, t.axis2))
                .collect();
            let key = format!("{json}|{}", moves.join(";"));
            st.calls.push(PartCall { part: name, key });
        }
        part.call::<_, Value>((params,))?;
        Ok(undefined(&c))
    });
    native!("rest", |c, s, a| {
        let cb = callback(&c, a, 0, "rest")?;
        s.borrow_mut().rest = Some(cb);
        Ok(undefined(&c))
    });
    native!("action", |c, s, a| {
        let first = arg(&c, a, 0);
        let name = if first.is_undefined() {
            String::new()
        } else {
            js_string(&c, first)?.trim().to_string()
        };
        if name.is_empty() {
            return Err(type_error(&c, "action: falta el nombre"));
        }
        {
            let st = s.borrow();
            if st.res.actions.iter().any(|x| x.spec.name == name) {
                return Err(type_error(
                    &c,
                    &format!("action: la acción {} está repetida", quote(&name)),
                ));
            }
            if st.res.actions.len() >= MAX_ACTIONS {
                return Err(type_error(
                    &c,
                    &format!("action: como mucho {MAX_ACTIONS} acciones"),
                ));
            }
        }
        let mut spec = ActionSpec {
            name: name.clone(),
            frames: 8,
            fps: 8,
            looping: true,
        };
        let mut frames = 8i64;
        let mut fps = 8i64;
        let opts = arg(&c, a, 1).into_object();
        if let Some(n) = get(&opts, "frames").and_then(|v| int_of(&v)) {
            frames = n;
        }
        if let Some(n) = get(&opts, "fps").and_then(|v| int_of(&v)) {
            fps = n;
        }
        if let Some(b) = get(&opts, "loop").and_then(|v| v.as_bool()) {
            spec.looping = b;
        }
        if frames < 1 || frames > i64::from(MAX_FRAMES) {
            return Err(type_error(
                &c,
                &format!(
                    "action {}: frames debe estar entre 1 y {MAX_FRAMES}",
                    quote(&name)
                ),
            ));
        }
        if !(1..=60).contains(&fps) {
            return Err(type_error(
                &c,
                &format!("action {}: fps debe estar entre 1 y 60", quote(&name)),
            ));
        }
        spec.frames = frames as u32;
        spec.fps = fps as u32;
        let line = caller_line(&c);
        let cb = callback(&c, a, 2, "action")?;
        let mut st = s.borrow_mut();
        st.res.actions.push(Action {
            spec,
            line,
            images: Vec::new(),
            out_of_bounds: Vec::new(),
            part_calls: Vec::new(),
        });
        st.action_fns.push(cb);
        Ok(undefined(&c))
    });

    // Azar repetible y sin reloj.
    let rand: Function = globals.get("rand")?;
    let math: Object = globals.get("Math")?;
    math.set("random", rand)?;
    globals.set("Date", Value::new_undefined(ctx.clone()))?;
    Ok(())
}

/// Ejecuta `anim.js` y dibuja el reposo y cada fotograma de cada acción.
pub fn render(source: &str, load_base: BaseLoader) -> AnimResult {
    let mut fallback = AnimResult::default();
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            fallback.issue(
                Level::Error,
                0,
                "",
                format!("no se pudo crear el motor: {e}"),
            );
            return fallback;
        }
    };
    runtime.set_memory_limit(256 * 1024 * 1024);
    runtime.set_max_stack_size(512 * 1024);
    let deadline: Rc<Cell<Option<Instant>>> = Rc::default();
    let interrupted: Rc<Cell<bool>> = Rc::default();
    {
        let (deadline, interrupted) = (deadline.clone(), interrupted.clone());
        runtime.set_interrupt_handler(Some(Box::new(move || {
            let late = deadline.get().is_some_and(|d| Instant::now() > d);
            if late {
                interrupted.set(true);
            }
            late
        })));
    }
    let context = match Context::full(&runtime) {
        Ok(ctx) => ctx,
        Err(e) => {
            fallback.issue(
                Level::Error,
                0,
                "",
                format!("no se pudo crear el motor: {e}"),
            );
            return fallback;
        }
    };
    context.with(|ctx| {
        let state: Shared = Rc::new(RefCell::new(State {
            res: AnimResult::default(),
            load_base,
            base_name: String::new(),
            parts: HashMap::new(),
            rest: None,
            action_fns: Vec::new(),
            surf: Image::new(0, 0),
            stack: Vec::new(),
            oob: 0,
            ops: 0,
            seed: 0,
            colors: HashMap::new(),
            calls: Vec::new(),
        }));
        let runner = Runner {
            ctx: ctx.clone(),
            source: source.to_string(),
            state: state.clone(),
            deadline,
            interrupted,
        };
        if let Err(e) = install(&ctx, &state) {
            let mut st = state.borrow_mut();
            st.res.issue(
                Level::Error,
                0,
                "",
                format!("no se pudo preparar el motor: {e}"),
            );
            return std::mem::take(&mut st.res);
        }
        runner.run_all(source);
        // Se sueltan las funciones de JavaScript antes de cerrar el motor.
        let mut st = state.borrow_mut();
        st.parts.clear();
        st.rest = None;
        st.action_fns.clear();
        std::mem::take(&mut st.res)
    })
}

struct Runner<'js> {
    ctx: Ctx<'js>,
    /// `anim.js` (para señalar el bucle de un corte por tiempo).
    source: String,
    state: Shared<'js>,
    deadline: Rc<Cell<Option<Instant>>>,
    interrupted: Rc<Cell<bool>>,
}

impl<'js> Runner<'js> {
    fn issue(&self, level: Level, line: u32, place: impl Into<String>, msg: impl Into<String>) {
        self.state.borrow_mut().res.issue(level, line, place, msg);
    }

    /// Ejecuta `f` con un tiempo máximo.
    fn timed<T>(
        &self,
        limit: Duration,
        f: impl FnOnce() -> rquickjs::Result<T>,
    ) -> Result<T, JsError> {
        self.interrupted.set(false);
        self.deadline.set(Some(Instant::now() + limit));
        let result = f();
        self.deadline.set(None);
        result.map_err(|_| {
            let mut error = take_error(&self.ctx, &self.interrupted);
            if error.timeout && error.line > 0 {
                error.line = loop_line(&self.source, error.line);
            }
            error
        })
    }

    fn eval(&self, name: &str, source: &str) -> Result<(), JsError> {
        let mut options = EvalOptions::default();
        options.global = true;
        options.strict = false;
        options.filename = Some(name.to_string());
        self.timed(LOAD_TIMEOUT, || {
            self.ctx
                .eval_with_options::<Value, _>(source, options)
                .map(|_| ())
        })
    }

    fn run_all(&self, source: &str) {
        if let Err(e) = self.eval("prelude.js", PRELUDE) {
            self.issue(Level::Error, 0, "", format!("prelude: {}", e.message));
            return;
        }
        if let Err(e) = self.eval("anim.js", source) {
            self.issue(Level::Error, e.line, "", e.message);
            return;
        }
        let (needs_base, loader) = {
            let st = self.state.borrow();
            (
                st.res.base.is_none() && st.base_name.is_empty(),
                st.load_base.clone(),
            )
        };
        if needs_base && let Ok(img) = loader("base.png") {
            // Sin base con nombre: base.png junto a anim.js, si existe.
            self.state.borrow_mut().res.base = Some(img);
        }
        if self.state.borrow().res.actions.is_empty() {
            self.issue(
                Level::Error,
                0,
                "",
                "anim.js no define ninguna acción: usa action(nombre, { frames, fps }, (f, { t }) => { ... })",
            );
        }

        let rest = self.state.borrow().rest.clone();
        if let Some(rest) = rest {
            let (img, _) = self.frame(&rest, "reposo", 0, None);
            self.state.borrow_mut().res.rest = Some(img);
        }
        let count = self.state.borrow().res.actions.len();
        for i in 0..count {
            let (func, spec, line) = {
                let st = self.state.borrow();
                let a = &st.res.actions[i];
                (st.action_fns[i].clone(), a.spec.clone(), a.line)
            };
            let mut images = Vec::with_capacity(spec.frames as usize);
            let mut oob = Vec::with_capacity(spec.frames as usize);
            let mut calls = Vec::with_capacity(spec.frames as usize);
            for f in 0..spec.frames {
                let (img, n) = self.frame(&func, &spec.name, f, Some((&spec, line)));
                images.push(img);
                oob.push(n);
                calls.push(std::mem::take(&mut self.state.borrow_mut().calls));
            }
            // Dibujar los fotogramas al revés tiene que dar lo mismo; si no,
            // el código guarda estado entre fotogramas.
            if spec.frames > 1 {
                for f in (0..spec.frames).rev() {
                    let (img, _) = self
                        .draw(&func, &spec.name, f, Some(&spec))
                        .unwrap_or_else(|(img, _)| (img, 0));
                    if !same_image(&img, &images[f as usize]) {
                        self.issue(
                            Level::Warning,
                            line,
                            spec.name.clone(),
                            format!(
                                "el frame {f} cambia según el orden en que se dibujan los frames: no guardes estado entre frames"
                            ),
                        );
                        break;
                    }
                }
            }
            let mut st = self.state.borrow_mut();
            st.res.actions[i].images = images;
            st.res.actions[i].out_of_bounds = oob;
            st.res.actions[i].part_calls = calls;
        }
    }

    /// `{ f, t, n, fps }` del fotograma.
    fn info(&self, f: u32, spec: Option<&ActionSpec>) -> rquickjs::Result<Object<'js>> {
        let (n, fps) = spec.map_or((1, 0), |a| (a.frames, a.fps));
        let t = match spec {
            // Un bucle no repite nunca su primer fotograma.
            Some(a) if a.looping => f64::from(f) / f64::from(n),
            // Una acción de una vez acaba justo en t = 1.
            Some(_) if n > 1 => f64::from(f) / f64::from(n - 1),
            _ => 0.0,
        };
        let info = Object::new(self.ctx.clone())?;
        info.set("f", f)?;
        info.set("t", t)?;
        info.set("n", n)?;
        info.set("fps", fps)?;
        Ok(info)
    }

    /// Dibuja un fotograma; si falla, apunta el error con su línea.
    fn frame(
        &self,
        func: &Function<'js>,
        name: &str,
        f: u32,
        action: Option<(&ActionSpec, u32)>,
    ) -> (Image, usize) {
        match self.draw(func, name, f, action.map(|(spec, _)| spec)) {
            Ok(done) => done,
            Err((img, error)) => {
                let mut line = error.line;
                if line == 0
                    && let Some((_, decl)) = action
                {
                    line = decl;
                }
                self.issue(Level::Error, line, format!("{name} f{f}"), error.message);
                (img, self.state.borrow().oob)
            }
        }
    }

    /// Dibuja el fotograma `f` desde cero.
    #[allow(clippy::result_large_err)]
    fn draw(
        &self,
        func: &Function<'js>,
        name: &str,
        f: u32,
        spec: Option<&ActionSpec>,
    ) -> Result<(Image, usize), (Image, JsError)> {
        {
            let mut st = self.state.borrow_mut();
            let (w, h) = st.res.cell;
            let origin = st.res.origin;
            st.surf = Image::new(w, h);
            st.stack = vec![Xform::shift(origin.0, origin.1)];
            st.oob = 0;
            st.ops = 0;
            st.calls.clear();
            st.seed = fnv32a(format!("{name}/{f}").as_bytes());
        }
        let result = self.timed(FRAME_TIMEOUT, || {
            let info = self.info(f, spec)?;
            func.call::<_, Value>((f, info)).map(|_| ())
        });
        let mut st = self.state.borrow_mut();
        let img = std::mem::replace(&mut st.surf, Image::new(0, 0));
        match result {
            Ok(()) => Ok((img, st.oob)),
            Err(e) => Err((img, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_match_pxcli() {
        // Valores conocidos de FNV-1a de 32 bits.
        assert_eq!(fnv32a(b""), 0x811c_9dc5);
        assert_eq!(fnv32a(b"a"), 0xe40c_292c);
        assert_eq!(
            stack_line("    at g (anim.js:3:3)\n    at <eval> (anim.js:5:1)\n"),
            3
        );
        assert_eq!(
            stack_line("    at act (prelude.js:1:29)\n    at <eval> (anim.js:12:1)\n"),
            12
        );
        assert_eq!(stack_line("sin línea"), 0);
        let src = "a();\nb();\n  while (true) {}\nc();\n";
        assert_eq!(loop_line(src, 1), 3);
        assert_eq!(loop_line(src, 3), 3);
        assert_eq!(loop_line(src, 4), 4, "sin bucle después, la misma");
        assert_eq!(
            loop_line("x = format(1);\n", 1),
            1,
            "«for» dentro de otra palabra no cuenta"
        );
        let mut seed = 0;
        let a = next_rand(&mut seed);
        assert!((0.0..1.0).contains(&a));
    }
}
