//! Lo que necesita el modo vector del studio, sin E/S: la zona que se rodea
//! con el lazo y el encargo de retocarla (`vecSelectionPrompt` de pxcli),
//! las capas del dibujo con sus formas, ocultar capas y el dibujo hasta la
//! forma N (para «Repetir trazado»).
//!
//! Trabaja sobre el SVG que genera `lib.js`: el limpio (cada capa es un
//! `<g>` de primer nivel con `data-name`) y el del studio (cada forma con
//! `data-i`, `data-l` y, dentro de funciones, `data-c`).

use super::rig::fnum;

/// Una forma que se ve dentro de una zona.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeHit {
    /// Línea de `dibujo.js` de la forma.
    pub line: u32,
    /// Las llamadas que llevan a ella, de dentro afuera (la primera es
    /// `line`), si está dentro de funciones.
    pub chain: Vec<u32>,
    /// Etiqueta SVG (`path`, `circle`…).
    pub tag: String,
    /// La capa (`layer()`) en que está, si está en una.
    pub layer: String,
    /// Parte de la zona que cubre (de 0 a 1).
    pub share: f64,
    /// Parte de la zona donde es la forma de encima.
    pub top: f64,
    /// Cubre casi todo el lienzo (fondo, luces, viñeta, grano).
    pub big: bool,
}

/// La zona rodeada con el lazo y lo que hay debajo.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Zone {
    /// El contorno, en unidades del dibujo.
    pub points: Vec<[f64; 2]>,
    /// Lo que se ve dentro, de más a menos superficie.
    pub hits: Vec<ShapeHit>,
}

/// La caja de unos puntos: `[x0, y0, x1, y1]`.
pub fn bounds(points: &[[f64; 2]]) -> [f64; 4] {
    let mut b = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for p in points {
        b[0] = b[0].min(p[0]);
        b[1] = b[1].min(p[1]);
        b[2] = b[2].max(p[0]);
        b[3] = b[3].max(p[1]);
    }
    b
}

/// ¿Está `(x, y)` dentro del polígono? (par-impar, como la página de pxcli).
pub fn in_polygon(x: f64, y: f64, points: &[[f64; 2]]) -> bool {
    let mut inside = false;
    let n = points.len();
    let mut j = n.wrapping_sub(1);
    for i in 0..n {
        let ([xi, yi], [xj, yj]) = (points[i], points[j]);
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (x, y) = (a[0] + t * dx, a[1] + t * dy);
    ((p[0] - x).powi(2) + (p[1] - y).powi(2)).sqrt()
}

/// El trazo del lazo con menos puntos (Ramer–Douglas–Peucker, `eps` en
/// unidades).
pub fn simplify(points: &[[f64; 2]], eps: f64) -> Vec<[f64; 2]> {
    if points.len() < 4 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0, points.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let mut index = None;
        let mut dmax = eps;
        for i in a + 1..b {
            let d = segment_distance(points[i], points[a], points[b]);
            if d > dmax {
                dmax = d;
                index = Some(i);
            }
        }
        if let Some(i) = index {
            keep[i] = true;
            stack.push((a, i));
            stack.push((i, b));
        }
    }
    points
        .iter()
        .zip(keep)
        .filter(|(_, k)| *k)
        .map(|(p, _)| *p)
        .collect()
}

impl Zone {
    /// Las formas propias de la zona (sin las que cubren casi todo).
    pub fn own(&self) -> Vec<&ShapeHit> {
        self.hits.iter().filter(|h| !h.big).collect()
    }

    /// La capa con más superficie dentro de la zona.
    pub fn main_layer(&self) -> Option<&str> {
        let mut by_layer: Vec<(&str, f64)> = Vec::new();
        for h in self.own().into_iter().filter(|h| !h.layer.is_empty()) {
            match by_layer.iter_mut().find(|(l, _)| *l == h.layer) {
                Some((_, s)) => *s += h.share,
                None => by_layer.push((&h.layer, h.share)),
            }
        }
        by_layer
            .into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(l, _)| l)
    }

    /// «9 formas de la capa «mar»», «1 forma», «zona vacía: solo fondo».
    pub fn summary(&self) -> String {
        let own = self.own();
        let n = own.len();
        if n == 0 {
            return "zona vacía: solo fondo".into();
        }
        let shapes = if n == 1 {
            "1 forma".to_string()
        } else {
            format!("{n} formas")
        };
        let all_in_one = self
            .main_layer()
            .filter(|layer| own.iter().all(|h| h.layer == *layer));
        match all_in_one {
            Some(layer) => format!("{shapes} de la capa «{layer}»"),
            None => shapes,
        }
    }

    /// Las líneas de `dibujo.js` de las formas propias, sin repetir y en
    /// orden.
    pub fn lines(&self) -> Vec<u32> {
        let mut lines: Vec<u32> = self.own().iter().map(|h| h.line).collect();
        lines.sort_unstable();
        lines.dedup();
        lines
    }
}

/// Como mucho `n` puntos del contorno, para el encargo.
fn outline_points(points: &[[f64; 2]], n: usize) -> String {
    let step = points.len().div_ceil(n).max(1);
    let parts: Vec<String> = points
        .iter()
        .step_by(step)
        .map(|p| format!("[{:.0}, {:.0}]", p[0], p[1]))
        .collect();
    format!("[{}]", parts.join(", "))
}

fn chain_text(chain: &[u32]) -> String {
    if chain.len() < 2 {
        return String::new();
    }
    let parts: Vec<String> = chain[1..].iter().map(u32::to_string).collect();
    format!(" (llamada desde la línea {})", parts.join(" › "))
}

/// El encargo de retocar la zona (va detrás de la petición del usuario).
pub fn zone_prompt(zone: &Zone, width: i64, height: i64) -> String {
    let mut b = String::from("\n## Retoque de una zona\n");
    b.push_str(
        "El usuario ha rodeado con el lazo una zona del dibujo y la petición se refiere a esa zona.\n",
    );
    b.push_str(&zone_facts(zone, width, height));
    b.push_str(RETOUCH_RULES);
    b
}

/// El encargo de varias correcciones en una pasada: cada una con lo que
/// pidió el usuario y su zona, y las reglas una sola vez.
pub fn zones_prompt(items: &[(&str, &Zone)], width: i64, height: i64) -> String {
    let mut b = format!(
        "Haz estas {} correcciones del dibujo en esta misma pasada. El usuario ha rodeado con el lazo una zona para cada una: cada corrección se refiere solo a su zona.\n",
        items.len()
    );
    for (i, (text, zone)) in items.iter().enumerate() {
        b.push_str(&format!("\n## Corrección {}: {}\n", i + 1, text.trim()));
        b.push_str(&zone_facts(zone, width, height));
    }
    b.push('\n');
    b.push_str(RETOUCH_RULES);
    b.push_str(
        "- Haz todas las correcciones antes de revisar y, al terminar, di en una frase qué has cambiado en cada una (por su número).\n",
    );
    b
}

/// Dónde está la zona y qué formas hay en ella.
fn zone_facts(zone: &Zone, width: i64, height: i64) -> String {
    let [x0, y0, x1, y1] = bounds(&zone.points);
    let mut b = String::new();
    b.push_str(&format!(
        "- La zona va de x {x0:.0} a {x1:.0} y de y {y0:.0} a {y1:.0} (lienzo de {width}x{height}). Su contorno: {}\n",
        outline_points(&zone.points, 24)
    ));
    let mut shapes: Vec<&ShapeHit> = zone.own();
    shapes.sort_by(|a, b| b.share.total_cmp(&a.share));
    if shapes.is_empty() {
        b.push_str("- Dentro de la zona no hay formas propias: solo fondo. Es un sitio vacío.\n");
    } else {
        b.push_str(
            "- Formas de dibujo.js que se ven dentro de la zona (de más a menos superficie):\n",
        );
        for (i, hit) in shapes.iter().enumerate() {
            if i == 25 {
                b.push_str(&format!("  - y {} más pequeñas\n", shapes.len() - 25));
                break;
            }
            b.push_str(&format!(
                "  - línea {}{}: {}, cubre el {:.0} % de la zona y está encima en el {:.0} %\n",
                hit.line,
                chain_text(&hit.chain),
                hit.tag,
                hit.share * 100.0,
                hit.top * 100.0
            ));
        }
    }
    let big: Vec<String> = zone
        .hits
        .iter()
        .filter(|h| h.big)
        .map(|h| h.line.to_string())
        .collect();
    if !big.is_empty() {
        b.push_str(&format!(
            "- También pasan por la zona formas que cubren casi todo el lienzo (fondo, luces, viñeta, grano): líneas {}. No las cambies salvo que la petición lo pida.\n",
            big.join(", ")
        ));
    }
    b
}

/// Cómo retocar una zona (lo mismo para una o para varias).
const RETOUCH_RULES: &str = "Cómo retocar:
- Si la petición cambia algo que ya está en la zona, edita el código de esas formas (y el de sus sombras, luces y reflejos) y nada más.
- Si pide añadir algo, dibújalo dentro de la zona y colócalo en el sitio correcto del orden de dibujo: lo que queda detrás va antes en el código y lo que queda delante, después. Tiene que parecer parte del dibujo: la misma luz (dirección, color y dureza), la misma paleta, el mismo estilo y grosor de línea, sombra de contacto si apoya en algo y proporciones coherentes con lo que hay alrededor.
- No toques nada fuera de la zona, salvo lo imprescindible para que el cambio encaje (la sombra que proyecta, un reflejo). Cambia solo esas líneas: no reescribas el resto del archivo.
- Revisa la zona ampliada y, antes de terminar, el dibujo entero (un cambio local puede descuadrar el conjunto: luz, escala, colores). Como mucho tres revisiones.
- Al terminar, di en una o dos frases qué has cambiado.
";

// ---------------------------------------------------------------------------
// Capas y formas en el SVG de lib.js
// ---------------------------------------------------------------------------

/// Una etiqueta del SVG: dónde empieza y acaba, su nombre y su tipo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Open,
    Close,
    SelfClosing,
}

/// Las etiquetas del SVG en orden (sin comentarios ni declaraciones).
fn tags(svg: &str) -> Vec<(usize, usize, &str, Kind)> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(off) = svg[i..].find('<') {
        let start = i + off;
        let Some(len) = svg[start..].find('>') else {
            break;
        };
        let end = start + len + 1;
        let inner = &svg[start + 1..end - 1];
        i = end;
        if inner.starts_with('!') || inner.starts_with('?') {
            continue;
        }
        let (kind, body) = if let Some(rest) = inner.strip_prefix('/') {
            (Kind::Close, rest)
        } else if let Some(rest) = inner.strip_suffix('/') {
            (Kind::SelfClosing, rest)
        } else {
            (Kind::Open, inner)
        };
        let name_end = body
            .find(|c: char| c.is_whitespace() || c == '/')
            .unwrap_or(body.len());
        out.push((start, end, &body[..name_end], kind));
    }
    out
}

/// El valor de un atributo en el texto de una etiqueta.
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let i = tag.find(&key)? + key.len();
    let end = tag[i..].find('"')?;
    Some(&tag[i..i + end])
}

fn unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Una capa del dibujo: un `<g>` de primer nivel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerInfo {
    pub name: String,
    /// Formas dentro (elementos que no son grupos).
    pub shapes: usize,
    /// Dónde está en el SVG.
    pub range: (usize, usize),
}

/// Las capas del SVG limpio, en orden de dibujo.
pub fn layers(svg: &str) -> Vec<LayerInfo> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut current: Option<LayerInfo> = None;
    let mut in_defs = 0usize;
    for (start, end, name, kind) in tags(svg) {
        match kind {
            Kind::Open => {
                depth += 1;
                if depth == 2 && name == "g" {
                    let tag = &svg[start..end];
                    let label = attribute(tag, "data-name")
                        .or_else(|| attribute(tag, "id"))
                        .map(unescape)
                        .unwrap_or_else(|| format!("capa {}", out.len() + 1));
                    current = Some(LayerInfo {
                        name: label,
                        shapes: 0,
                        range: (start, end),
                    });
                } else if name == "defs" {
                    in_defs += 1;
                } else if depth > 2 && name != "g" && in_defs == 0 {
                    // Una forma con contenido (un texto).
                    if let Some(layer) = current.as_mut() {
                        layer.shapes += 1;
                    }
                }
            }
            Kind::SelfClosing => {
                if depth >= 2
                    && name != "g"
                    && in_defs == 0
                    && let Some(layer) = current.as_mut()
                {
                    layer.shapes += 1;
                }
            }
            Kind::Close => {
                if name == "defs" {
                    in_defs = in_defs.saturating_sub(1);
                }
                if depth == 2
                    && name == "g"
                    && let Some(mut layer) = current.take()
                {
                    layer.range.1 = end;
                    out.push(layer);
                }
                depth = depth.saturating_sub(1);
            }
        }
    }
    out
}

/// El SVG sin las capas de `hidden` (por su índice en [`layers`]).
pub fn hide_layers(svg: &str, hidden: &[usize]) -> String {
    if hidden.is_empty() {
        return svg.to_string();
    }
    let mut out = String::with_capacity(svg.len());
    let mut last = 0;
    for (i, layer) in layers(svg).into_iter().enumerate() {
        if hidden.contains(&i) {
            out.push_str(&svg[last..layer.range.0]);
            last = layer.range.1;
        }
    }
    out.push_str(&svg[last..]);
    out
}

/// Un trozo del dibujo en orden de pintado: una capa sola (`layer`) o lo que
/// va entre capas fuera de ellas (`None`). Cada uno es un SVG completo con
/// las `defs` del dibujo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slice {
    pub layer: Option<usize>,
    pub svg: String,
}

/// El dibujo en trozos: pintados uno encima de otro dan el dibujo entero, y
/// sin los de las capas ocultas, el dibujo sin ellas. Así la ventana oculta
/// una capa al instante, sin volver a pintar nada.
pub fn slices(svg: &str) -> Vec<Slice> {
    // Elementos hijos del <svg> (de primer nivel), con dónde empiezan y
    // acaban y si son una capa (`<g>`) o las `defs`.
    let mut children: Vec<(usize, usize, &str)> = Vec::new();
    let mut depth = 0usize;
    let mut open_at = 0usize;
    let mut open_name = "";
    for (start, end, name, kind) in tags(svg) {
        match kind {
            Kind::Open => {
                depth += 1;
                if depth == 2 {
                    open_at = start;
                    open_name = name;
                }
            }
            Kind::Close => {
                if depth == 2 {
                    children.push((open_at, end, open_name));
                }
                depth = depth.saturating_sub(1);
            }
            Kind::SelfClosing => {
                if depth == 1 {
                    children.push((start, end, name));
                }
            }
        }
    }
    let (Some(first), Some(last)) = (children.first(), children.last()) else {
        return vec![Slice {
            layer: None,
            svg: svg.to_string(),
        }];
    };
    let head = &svg[..first.0];
    let tail = &svg[last.1..];
    let defs: String = children
        .iter()
        .filter(|c| c.2 == "defs")
        .map(|c| &svg[c.0..c.1])
        .collect();
    let wrap = |body: &str| format!("{head}{defs}{body}{tail}");
    let mut out = Vec::new();
    let mut loose = String::new();
    let mut layer = 0usize;
    for (start, end, name) in &children {
        match *name {
            "defs" => {}
            "g" => {
                if !loose.is_empty() {
                    out.push(Slice {
                        layer: None,
                        svg: wrap(&std::mem::take(&mut loose)),
                    });
                }
                out.push(Slice {
                    layer: Some(layer),
                    svg: wrap(&svg[*start..*end]),
                });
                layer += 1;
            }
            _ => loose.push_str(&svg[*start..*end]),
        }
    }
    if !loose.is_empty() {
        out.push(Slice {
            layer: None,
            svg: wrap(&loose),
        });
    }
    out
}

/// El SVG del studio sin las piezas del esqueleto (los grupos con
/// `data-part`): lo que no se mueve al animar.
pub fn without_parts(studio_svg: &str) -> String {
    let mut out = String::with_capacity(studio_svg.len());
    let mut last = 0;
    let mut skip_depth: Option<usize> = None;
    let mut depth = 0usize;
    for (start, end, name, kind) in tags(studio_svg) {
        match kind {
            Kind::Open => {
                depth += 1;
                if skip_depth.is_none()
                    && name == "g"
                    && studio_svg[start..end].contains(" data-part=\"")
                {
                    out.push_str(&studio_svg[last..start]);
                    skip_depth = Some(depth);
                }
            }
            Kind::Close => {
                if skip_depth == Some(depth) {
                    skip_depth = None;
                    last = end;
                }
                depth = depth.saturating_sub(1);
            }
            Kind::SelfClosing => {}
        }
    }
    out.push_str(&studio_svg[last..]);
    out
}

/// Lo de dentro de un `<svg …>…</svg>` (para meterlo en otro).
pub fn svg_inner(svg: &str) -> &str {
    let Some(open) = svg.find("<svg") else {
        return svg;
    };
    let Some(start) = svg[open..].find('>').map(|i| open + i + 1) else {
        return svg;
    };
    let end = svg.rfind("</svg>").unwrap_or(svg.len()).max(start);
    &svg[start..end]
}

/// Formas del SVG del studio (las que llevan `data-i`).
pub fn shape_count(studio_svg: &str) -> usize {
    studio_svg.matches(" data-i=\"").count()
}

/// El SVG del studio con solo las formas anteriores a la `upto` (en orden
/// de dibujo): las demás, con `display="none"`.
pub fn shapes_until(studio_svg: &str, upto: usize) -> String {
    const KEY: &str = " data-i=\"";
    let mut out = String::with_capacity(studio_svg.len() + 64);
    let mut rest = studio_svg;
    while let Some(i) = rest.find(KEY) {
        let after = &rest[i + KEY.len()..];
        let Some(end) = after.find('"') else {
            break;
        };
        let index: usize = after[..end].parse().unwrap_or(0);
        out.push_str(&rest[..i + KEY.len() + end + 1]);
        if index >= upto {
            out.push_str(" display=\"none\"");
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// «x 214 · y 618» (para la barra de estado).
pub fn coordinates(x: f64, y: f64) -> String {
    format!("x {} · y {}", fnum(x.round()), fnum(y.round()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLEAN: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50" width="100" height="50"><defs><linearGradient id="g"><stop offset="0"/></linearGradient></defs><g id="Lienzo" data-name="Lienzo"><rect id="r" data-name="canvas línea 1" width="100" height="50"/></g><g id="mar" data-name="mar"><rect width="10" height="10"/><g id="ola"><circle r="2"/><path d="M0 0"/></g><text x="1">Hola &amp; adiós</text></g><g id="sol" data-name="sol &amp; luz"><circle r="5"/></g></svg>"##;

    #[test]
    fn layers_are_the_top_level_groups() {
        let l = layers(CLEAN);
        let names: Vec<(&str, usize)> = l.iter().map(|l| (l.name.as_str(), l.shapes)).collect();
        assert_eq!(names, [("Lienzo", 1), ("mar", 4), ("sol & luz", 1)]);
        let without = hide_layers(CLEAN, &[1]);
        assert!(!without.contains("ola") && without.contains("sol &amp; luz"));
        assert_eq!(layers(&without).len(), 2);
        assert_eq!(hide_layers(CLEAN, &[]), CLEAN);
    }

    #[test]
    fn the_replay_hides_later_shapes() {
        let studio = r#"<svg><rect data-i="0" data-l="2"/><g data-name="x"><circle data-i="1" data-l="3"/><path data-i="2" data-l="4"/></g></svg>"#;
        assert_eq!(shape_count(studio), 3);
        let two = shapes_until(studio, 2);
        assert_eq!(two.matches("display=\"none\"").count(), 1);
        assert!(
            two.contains(r#"<path data-i="2" display="none" data-l="4"/>"#),
            "{two}"
        );
        assert_eq!(shapes_until(studio, 3), studio);
    }

    #[test]
    fn the_lasso_is_simplified_and_tested() {
        let square = [
            [0.0, 0.0],
            [5.0, 0.1],
            [10.0, 0.0],
            [10.0, 10.0],
            [0.0, 10.0],
        ];
        assert_eq!(simplify(&square, 1.0).len(), 4);
        assert!(in_polygon(5.0, 5.0, &square));
        assert!(!in_polygon(15.0, 5.0, &square));
        assert_eq!(bounds(&square), [0.0, 0.0, 10.0, 10.0]);
    }

    fn hit(line: u32, layer: &str, share: f64, big: bool) -> ShapeHit {
        ShapeHit {
            line,
            chain: vec![line, 40],
            tag: "path".into(),
            layer: layer.into(),
            share,
            top: share / 2.0,
            big,
        }
    }

    #[test]
    fn the_zone_summary_and_prompt_name_the_shapes() {
        let zone = Zone {
            points: vec![[10.0, 20.0], [60.0, 20.0], [60.0, 50.0], [10.0, 50.0]],
            hits: vec![
                hit(37, "mar", 0.8, false),
                hit(38, "mar", 0.3, false),
                hit(2, "", 1.0, true),
            ],
        };
        assert_eq!(zone.summary(), "2 formas de la capa «mar»");
        assert_eq!(zone.lines(), [37, 38]);
        let prompt = zone_prompt(&zone, 1024, 768);
        assert!(
            prompt.contains("La zona va de x 10 a 60 y de y 20 a 50 (lienzo de 1024x768)"),
            "{prompt}"
        );
        assert!(prompt.contains("  - línea 37 (llamada desde la línea 40): path, cubre el 80 % de la zona y está encima en el 40 %\n"), "{prompt}");
        assert!(
            prompt.contains(
                "formas que cubren casi todo el lienzo (fondo, luces, viñeta, grano): líneas 2."
            ),
            "{prompt}"
        );
        let empty = Zone::default();
        assert_eq!(empty.summary(), "zona vacía: solo fondo");
        let mixed = Zone {
            points: vec![],
            hits: vec![hit(5, "cielo", 0.5, false), hit(9, "mar", 0.2, false)],
        };
        assert_eq!(mixed.summary(), "2 formas");
        assert_eq!(mixed.main_layer(), Some("cielo"));
    }
}
