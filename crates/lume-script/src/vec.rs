//! `dibujo.js` con QuickJS: el porte de `internal/vec/vec.go` de pxcli.
//!
//! Se ejecuta `lib.js` (la librería de dibujo de pxcli, sin cambios) y
//! después `dibujo.js`, cada uno con 4 s como mucho; luego `__render` da el
//! SVG del studio y el limpio. La librería sabe de qué línea viene cada
//! forma por `__chain()` y `__line()`, que leen la pila de llamadas. Sin
//! archivos ni red. Los fallos vuelven como avisos con su línea.
//!
//! VectorCraft (`trace`, `freeform`, `revolve`…) llegará con el calco: por
//! ahora esas funciones dicen que hace falta, como pxcli sin él.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use lume_app::ports::{VecEngine, VecHelper};
use lume_domain::vec::{FILE_NAME, Level, VecResult};

use crate::calls::{CallMap, Pos};
use rquickjs::context::EvalOptions;
use rquickjs::{Array, Coerced, Context, Ctx, Exception, FromJs, Function, Runtime, Value};

/// La librería de dibujo (`internal/vec/lib.js` de pxcli).
pub const LIB: &str = include_str!("vec_lib.js");

/// Tiempo para cargar la librería y para ejecutar `dibujo.js`.
pub const RUN_TIMEOUT: Duration = Duration::from_secs(4);
/// Tiempo para convertir lo dibujado en SVG (pxcli no lo limitaba; aquí,
/// por si acaso, con holgura).
const RENDER_TIMEOUT: Duration = Duration::from_secs(60);

const TIMEOUT_MESSAGE: &str = "se ha superado el tiempo límite (¿un bucle infinito?)";

/// Líneas de `dibujo.js` que recuerda `__chain()` (la de la forma y las de
/// las funciones que la rodean).
const MAX_CHAIN: usize = 5;

/// Llamadas a VectorCraft (y al calco) que puede hacer un dibujo.
const MAX_HELPER_CALLS: u32 = 60;

/// El motor de `dibujo.js` de LumeArt, con sus ayudas opcionales.
#[derive(Clone, Default)]
pub struct QuickJsVec {
    /// El calco de la referencia (`trace()`); sin él, `trace()` pide
    /// VectorCraft, como pxcli.
    pub trace: Option<VecHelper>,
    /// VectorCraft, si está instalado (`freeform`, `revolve`, sólidos…).
    pub vectorcraft: Option<VecHelper>,
}

impl VecEngine for QuickJsVec {
    fn render(&self, source: &str) -> VecResult {
        run(source, false, self)
    }

    fn render_scene(&self, source: &str) -> VecResult {
        run(source, true, self)
    }
}

/// Un marco de una pila de QuickJS: «at nombre (archivo:línea:columna)».
#[derive(Debug, Clone, PartialEq, Eq)]
struct Frame {
    name: String,
    file: String,
    pos: Option<Pos>,
}

/// Los marcos de una pila, de dentro afuera.
fn frames(stack: &str) -> Vec<Frame> {
    stack
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("at ")?;
            let (name, place) = match rest.strip_suffix(')').and_then(|r| r.split_once(" (")) {
                Some((name, place)) => (name, place),
                None => ("", rest),
            };
            let mut parts = place.rsplitn(3, ':');
            let col = parts.next()?;
            let (line, file) = match (parts.next(), parts.next()) {
                (Some(line), Some(file)) => (line, file),
                _ => {
                    return Some(Frame {
                        name: name.to_string(),
                        file: place.to_string(),
                        pos: None,
                    });
                }
            };
            Some(Frame {
                name: name.to_string(),
                file: file.to_string(),
                pos: line
                    .parse()
                    .ok()
                    .zip(col.parse().ok())
                    .map(|(line, col)| Pos { line, col }),
            })
        })
        .collect()
}

/// Las líneas de `dibujo.js` de una pila, de dentro afuera, como las daría
/// goja: un marco que llama a otro (todos menos el primero de la pila) está
/// en la línea del `(` de esa llamada.
fn script_lines(stack: &str, calls: &CallMap) -> Vec<u32> {
    let frames = frames(stack);
    frames
        .iter()
        .enumerate()
        .filter(|(_, f)| f.file == FILE_NAME)
        .filter_map(|(i, f)| {
            let pos = f.pos?;
            Some(match i.checked_sub(1) {
                Some(inner) => calls.paren_line(pos, Some(frames[inner].name.as_str())),
                None => pos.line,
            })
        })
        .collect()
}

/// La pila de la llamada en curso.
fn current_stack(ctx: &Ctx<'_>) -> String {
    Exception::from_message(ctx.clone(), "")
        .ok()
        .and_then(|e| e.stack())
        .unwrap_or_default()
}

/// `__chain()` de pxcli: las líneas de `dibujo.js` que llevan a la llamada
/// en curso, de dentro afuera, sin repetir seguidas y como mucho cinco.
fn chain(lines: Vec<u32>) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();
    for line in lines {
        if out.last() != Some(&line) {
            out.push(line);
        }
    }
    out.truncate(MAX_CHAIN);
    out
}

/// Como en el motor de animación: quickjs-ng señala la última llamada de la
/// función cortada por tiempo, no el bucle; se busca el primer bucle desde
/// esa línea.
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

/// Un error de JavaScript ya recogido.
struct JsError {
    line: u32,
    message: String,
}

/// `cleanErr` de pxcli: solo el mensaje, sin la pila ni el tipo de error, y
/// «X no existe» para lo que no está definido.
pub fn clean_message(text: &str) -> String {
    let mut s = text.lines().next().unwrap_or_default().to_string();
    for prefix in [
        "TypeError: ",
        "GoError: ",
        "Error: ",
        "ReferenceError: ",
        "SyntaxError: ",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
        }
    }
    let s = s
        .replacen(&format!("{FILE_NAME}: "), "", 1)
        .trim()
        .to_string();
    // Una variable declarada dos veces: el texto de goja.
    if let Some(name) = s
        .strip_prefix("redeclaration of '")
        .and_then(|rest| rest.strip_suffix('\''))
    {
        return format!("Identifier '{name}' has already been declared");
    }
    if s.contains("is not defined") {
        let name = s.strip_suffix("is not defined").unwrap_or(&s).trim();
        // QuickJS lo escribe entre comillas ('x' is not defined); goja no.
        let name = name
            .strip_prefix('\'')
            .and_then(|n| n.strip_suffix('\''))
            .unwrap_or(name);
        return format!("{name} no existe (¿una errata o una función que no está en la librería?)");
    }
    s
}

/// Recoge la excepción pendiente (`errLine` y `cleanErr` de pxcli).
fn take_error(ctx: &Ctx<'_>, interrupted: &Cell<bool>, source: &str, calls: &CallMap) -> JsError {
    let value = ctx.catch();
    let stack_line = |v: &Value<'_>| {
        v.as_exception()
            .and_then(|e| e.stack())
            .and_then(|s| script_lines(&s, calls).first().copied())
            .unwrap_or(0)
    };
    if interrupted.replace(false) {
        let line = stack_line(&value);
        return JsError {
            line: if line > 0 { loop_line(source, line) } else { 0 },
            message: TIMEOUT_MESSAGE.into(),
        };
    }
    let line = stack_line(&value);
    let text = match value.as_exception() {
        Some(ex) => {
            let name: Option<String> = ex
                .get::<_, Option<Coerced<String>>>("name")
                .ok()
                .flatten()
                .map(|c| c.0);
            let message = ex.message().unwrap_or_default();
            match name {
                Some(name) if !name.is_empty() => format!("{name}: {message}"),
                _ => message,
            }
        }
        None => Coerced::<String>::from_js(ctx, value)
            .map(|c| c.0)
            .unwrap_or_else(|_| "error desconocido".into()),
    };
    JsError {
        line,
        message: clean_message(&text),
    }
}

/// Ejecuta `dibujo.js` y devuelve el SVG y los problemas.
pub fn render(source: &str) -> VecResult {
    run(source, false, &QuickJsVec::default())
}

/// Como [`render`], y además la escena partida en capas y piezas si no hay
/// errores (`RenderScene` de pxcli; cuesta otra pasada por el dibujo).
pub fn render_scene(source: &str) -> VecResult {
    run(source, true, &QuickJsVec::default())
}

fn run(source: &str, with_scene: bool, helpers: &QuickJsVec) -> VecResult {
    let mut res = VecResult {
        width: 1024,
        height: 1024,
        ..VecResult::default()
    };
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            res.issue(Level::Error, 0, format!("no se pudo crear el motor: {e}"));
            return res;
        }
    };
    runtime.set_memory_limit(512 * 1024 * 1024);
    runtime.set_max_stack_size(1024 * 1024);
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
            res.issue(Level::Error, 0, format!("no se pudo crear el motor: {e}"));
            return res;
        }
    };
    context.with(|ctx| {
        let calls = Rc::new(CallMap::new(source));
        let runner = Runner {
            ctx: ctx.clone(),
            source,
            calls: calls.clone(),
            deadline: deadline.clone(),
            interrupted,
        };
        if let Err(e) = install(&ctx, &calls, helpers, &deadline) {
            res.issue(
                Level::Error,
                0,
                format!("no se pudo preparar el motor: {e}"),
            );
            return;
        }
        runner.run(&mut res, with_scene);
    });
    res
}

/// Las funciones que `lib.js` espera del motor.
fn install(
    ctx: &Ctx<'_>,
    calls: &Rc<CallMap>,
    helpers: &QuickJsVec,
    deadline: &Rc<Cell<Option<Instant>>>,
) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    // Pilas largas: `__chain` busca las líneas de dibujo.js entre las de la
    // librería (goja miraba 64 marcos).
    let error: rquickjs::Object = globals.get("Error")?;
    error.set("stackTraceLimit", 64)?;
    // Las funciones de Math como en goja, para que los números salgan igual.
    crate::gomath::install(ctx)?;
    let map = calls.clone();
    globals.set(
        "__chain",
        Function::new(ctx.clone(), move |ctx: Ctx<'_>| {
            chain(script_lines(&current_stack(&ctx), &map))
        })?,
    )?;
    let map = calls.clone();
    globals.set(
        "__line",
        Function::new(ctx.clone(), move |ctx: Ctx<'_>| {
            script_lines(&current_stack(&ctx), &map)
                .first()
                .copied()
                .unwrap_or(0)
        })?,
    )?;
    globals.set("__hasVC", helpers.vectorcraft.is_some())?;
    globals.set("__hasTrace", helpers.trace.is_some())?;
    let used: Rc<Cell<u32>> = Rc::default();
    for (name, helper) in [
        ("__trace", helpers.trace.clone()),
        ("__vc", helpers.vectorcraft.clone()),
    ] {
        let (used, deadline) = (used.clone(), deadline.clone());
        globals.set(
            name,
            Function::new(
                ctx.clone(),
                move |ctx: Ctx<'_>, job: Coerced<String>| -> rquickjs::Result<String> {
                    let Some(helper) = &helper else {
                        return Err(Exception::throw_message(
                            &ctx,
                            "hace falta VectorCraft (vectorcraft-cli) instalado",
                        ));
                    };
                    used.set(used.get() + 1);
                    if used.get() > MAX_HELPER_CALLS {
                        return Err(Exception::throw_message(
                            &ctx,
                            &format!(
                                "demasiadas llamadas a VectorCraft en un dibujo (máximo {MAX_HELPER_CALLS})"
                            ),
                        ));
                    }
                    // El tiempo de la ayuda no cuenta para el límite del dibujo.
                    let start = Instant::now();
                    let result = helper(&job.0);
                    if let Some(d) = deadline.get() {
                        deadline.set(Some(d + start.elapsed()));
                    }
                    result.map_err(|e| Exception::throw_message(&ctx, &e))
                },
            )?,
        )?;
    }
    Ok(())
}

struct Runner<'a, 'js> {
    ctx: Ctx<'js>,
    source: &'a str,
    calls: Rc<CallMap>,
    deadline: Rc<Cell<Option<Instant>>>,
    interrupted: Rc<Cell<bool>>,
}

impl<'js> Runner<'_, 'js> {
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
        result.map_err(|_| take_error(&self.ctx, &self.interrupted, self.source, &self.calls))
    }

    fn eval(&self, name: &str, code: &str) -> Result<(), JsError> {
        let mut options = EvalOptions::default();
        options.global = true;
        options.strict = false;
        options.filename = Some(name.to_string());
        self.timed(RUN_TIMEOUT, || {
            self.ctx
                .eval_with_options::<Value, _>(code, options)
                .map(|_| ())
        })
    }

    /// Una función global de la librería, si existe.
    fn function(&self, name: &str) -> Option<Function<'js>> {
        self.ctx.globals().get::<_, Function>(name).ok()
    }

    /// `ToInteger` de una global numérica (0 si no es un número).
    fn integer(&self, name: &str) -> i64 {
        let n: f64 = self
            .ctx
            .globals()
            .get::<_, Coerced<f64>>(name)
            .map(|c| c.0)
            .unwrap_or(0.0);
        if n.is_finite() { n.trunc() as i64 } else { 0 }
    }

    fn run(&self, res: &mut VecResult, with_scene: bool) {
        if let Err(e) = self.eval("lib.js", LIB) {
            res.issue(Level::Error, 0, format!("librería: {}", e.message));
            return;
        }
        if let Err(e) = self.eval(FILE_NAME, self.source) {
            res.issue(Level::Error, e.line, e.message);
        }
        // Para nombrar las formas exportadas por su código.
        let lines = Array::new(self.ctx.clone()).and_then(|array| {
            for (i, line) in self.source.split('\n').enumerate() {
                array.set(i, line)?;
            }
            self.ctx.globals().set("__src", array)
        });
        if let Err(e) = lines {
            res.issue(Level::Error, 0, format!("librería: {e}"));
            return;
        }
        let Some(render) = self.function("__render") else {
            res.issue(Level::Error, 0, "librería: falta __render");
            return;
        };
        match self.timed(RENDER_TIMEOUT, || {
            render.call::<_, Coerced<String>>((true,))
        }) {
            Ok(svg) => res.svg = svg.0,
            Err(e) => res.issue(Level::Error, e.line, e.message),
        }
        if let Ok(clean) = self.timed(RENDER_TIMEOUT, || {
            render.call::<_, Coerced<String>>((false,))
        }) {
            res.clean = clean.0;
        }
        if let Some(info) = self.function("__rigInfo") {
            match self.timed(RENDER_TIMEOUT, || info.call::<_, Coerced<String>>(())) {
                Ok(json) => res.rig = crate::vec_json::parse_rig(&json.0),
                Err(e) => res.issue(Level::Error, 0, format!("esqueleto: {}", e.message)),
            }
        }
        if with_scene
            && res.errors().is_empty()
            && let Some(info) = self.function("__sceneInfo")
        {
            match self.timed(RENDER_TIMEOUT, || info.call::<_, Coerced<String>>(())) {
                Ok(json) => res.scene = crate::vec_json::parse_scene(&json.0),
                Err(e) => res.issue(Level::Error, 0, format!("escena: {}", e.message)),
            }
        }
        res.width = self.integer("__W");
        res.height = self.integer("__H");
        res.count = self.integer("__count");
        if res.count == 0 && res.errors().is_empty() {
            res.issue(
                Level::Warning,
                0,
                "el dibujo está vacío: todavía no hay ninguna forma",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_match_pxcli() {
        let stack = "    at __chain (native)\n    at __where (lib.js:16:12)\n    at circle (lib.js:200:3)\n    at flor (dibujo.js:4:3)\n    at flor (dibujo.js:4:3)\n    at <eval> (dibujo.js:9:1)\n";
        assert_eq!(
            frames(stack)[3],
            Frame {
                name: "flor".into(),
                file: "dibujo.js".into(),
                pos: Some(Pos { line: 4, col: 3 })
            }
        );
        let none = CallMap::default();
        assert_eq!(script_lines(stack, &none), [4, 4, 9]);
        assert_eq!(chain(script_lines(stack, &none)), [4, 9]);
        assert_eq!(chain(Vec::new()), Vec::<u32>::new());
        // Un error de sintaxis: solo el archivo y la posición.
        assert_eq!(script_lines("    at dibujo.js:7:3\n", &none), [7]);
        assert_eq!(
            clean_message("ReferenceError: 'circel' is not defined\n    at <eval>"),
            "circel no existe (¿una errata o una función que no está en la librería?)"
        );
        assert_eq!(
            clean_message("SyntaxError: redeclaration of 'ink'"),
            "Identifier 'ink' has already been declared"
        );
        assert_eq!(
            clean_message("Error: circle: r tiene que ser un número (es undefined)"),
            "circle: r tiene que ser un número (es undefined)"
        );
    }
}
