//! El calco de la referencia (`trace.go` de pxcli): la imagen de referencia
//! convertida en trazos vectoriales sobre el lienzo del dibujo (encajada y
//! centrada, como la muestra `vec ref`). En color, regiones planas con las
//! formas exactas de la referencia; en «lines», solo la tinta oscura. Los
//! colores se pueden quitar (un cielo con ruido que el agente pinta él) o
//! ajustar a la paleta del dibujo.
//!
//! Aquí está lo que no depende de cómo se calcula el calco: las opciones,
//! el retoque de los trazos por su color y la lista de colores.

use super::rig::fnum;

/// Las opciones de `trace()` (ver `lib.js`) y de `vec calco`.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceOptions {
    /// Tamaño del lienzo.
    pub width: u32,
    pub height: u32,
    /// Imagen de referencia (vacía: la `referencia-*` más nueva).
    pub reference: String,
    /// Solo la tinta oscura (`lines`) o regiones de color (`color`).
    pub lines: bool,
    /// Colores del calco (de 2 a 64).
    pub colors: u32,
    /// Fidelidad de los trazos (de 0 a 100).
    pub paths: f64,
    /// Esquinas (de 0 a 100: más, más esquinas).
    pub corners: f64,
    /// Manchas más pequeñas que esto (en píxeles) se ignoran.
    pub noise: f64,
    /// En «lines», lo más oscuro que esto (de 0 a 255) es tinta.
    pub threshold: u32,
    /// Colores que se quitan.
    pub drop: Vec<String>,
    /// Distancia hasta la que un color se quita (40 si no se da).
    pub tolerance: f64,
    /// Si no está vacía, cada color pasa al más cercano de la paleta.
    pub palette: Vec<String>,
    /// Color de la tinta en «lines».
    pub ink: String,
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
            reference: String::new(),
            lines: false,
            colors: 16,
            paths: 60.0,
            corners: 50.0,
            noise: 6.0,
            threshold: 110,
            drop: Vec::new(),
            tolerance: 40.0,
            palette: Vec::new(),
            ink: "#1b1b1b".into(),
        }
    }
}

/// Un color del calco y cuántos trazos lo usan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceColor {
    pub hex: String,
    pub paths: usize,
}

/// `#rrggbb` en componentes de 0 a 255 (`hexRGB` de pxcli).
pub fn hex_rgb(s: &str) -> [f64; 3] {
    let hex: String = s.trim_start_matches('#').chars().take(6).collect();
    let v = u32::from_str_radix(&hex, 16).unwrap_or(0);
    [
        f64::from((v >> 16) & 255),
        f64::from((v >> 8) & 255),
        f64::from(v & 255),
    ]
}

/// Componentes de 0 a 255 en `#rrggbb`.
pub fn rgb_hex(c: [f64; 3]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        c[0] as i64 & 255,
        c[1] as i64 & 255,
        c[2] as i64 & 255
    )
}

/// Distancia RGB ponderada, bastante cercana a la percepción para ajustar
/// colores (`colorDist` de pxcli).
pub fn color_dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let rm = (a[0] + b[0]) / 2.0;
    let (dr, dg, db) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    ((2.0 + rm / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rm) / 256.0) * db * db).sqrt()
}

/// Luminancia de 0 a 1.
pub fn luminance(c: [f64; 3]) -> f64 {
    (0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]) / 255.0
}

/// Los colores `#rrggbb` de una lista (los demás se ignoran).
fn parse_hexes(list: &[String]) -> Vec<[f64; 3]> {
    list.iter()
        .filter(|s| s.len() >= 7 && s.starts_with('#'))
        .map(|s| hex_rgb(&s[..7]))
        .collect()
}

/// Cada `<path …/>` de un fragmento, con dónde empieza y acaba.
fn path_elements(fragment: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = fragment[from..].find("<path") {
        let start = from + i;
        let after = &fragment[start + 5..];
        // `<path` seguido de más letras (`<paths`) no es un trazo (`\b`).
        if after.chars().next().is_some_and(is_word) {
            from = start + 5;
            continue;
        }
        match after.find("/>") {
            // Un `>` antes del `/>` es otro elemento: no es un trazo vacío.
            Some(end) if !after[..end].contains('>') => {
                out.push((start, start + 5 + end + 2));
                from = start + 5 + end + 2;
            }
            _ => from = start + 5,
        }
    }
    out
}

/// El primer `fill="#rrggbb"` de un elemento: dónde empieza y el color.
fn fill_hex(element: &str) -> Option<(usize, &str)> {
    let mut from = 0;
    while let Some(i) = element[from..].find("fill=\"#") {
        let at = from + i;
        let hex = element.get(at + 6..at + 13)?;
        if hex[1..].chars().all(|c| c.is_ascii_hexdigit()) && element[at + 13..].starts_with('"') {
            return Some((at, hex));
        }
        from = at + 6;
    }
    None
}

/// Quita y cambia de color los trazos por su relleno (`postTrace`).
pub fn post_trace(fragment: &str, job: &TraceOptions) -> String {
    let tol = if job.tolerance <= 0.0 {
        40.0
    } else {
        job.tolerance
    };
    let drop = parse_hexes(&job.drop);
    let palette = parse_hexes(&job.palette);
    let ink = if job.ink.is_empty() {
        "#1b1b1b"
    } else {
        &job.ink
    };
    let mut out = String::with_capacity(fragment.len());
    let mut last = 0;
    for (start, end) in path_elements(fragment) {
        out.push_str(&fragment[last..start]);
        last = end;
        let el = &fragment[start..end];
        let Some((at, hex)) = fill_hex(el) else {
            out.push_str(el);
            continue;
        };
        let c = hex_rgb(hex);
        if drop.iter().any(|d| color_dist(c, *d) <= tol) {
            continue;
        }
        let fill = if job.lines {
            if luminance(c) > 0.5 {
                continue;
            }
            ink.to_string()
        } else if !palette.is_empty() {
            let mut best = palette[0];
            let mut best_dist = f64::INFINITY;
            for p in &palette {
                let d = color_dist(c, *p);
                if d < best_dist {
                    best = *p;
                    best_dist = d;
                }
            }
            rgb_hex(best)
        } else {
            hex.to_string()
        };
        out.push_str(&el[..at]);
        out.push_str(&format!("fill=\"{fill}\""));
        out.push_str(&el[at + 14..]);
    }
    out.push_str(&fragment[last..]);
    out
}

/// Los colores de un calco, del más usado al menos (`TraceColors`).
pub fn trace_colors(fragment: &str) -> Vec<TraceColor> {
    let mut count: std::collections::BTreeMap<String, usize> = Default::default();
    for (start, end) in path_elements(fragment) {
        if let Some((_, hex)) = fill_hex(&fragment[start..end]) {
            *count.entry(hex.to_lowercase()).or_default() += 1;
        }
    }
    let mut out: Vec<TraceColor> = count
        .into_iter()
        .map(|(hex, paths)| TraceColor { hex, paths })
        .collect();
    out.sort_by(|a, b| b.paths.cmp(&a.paths).then(a.hex.cmp(&b.hex)));
    out
}

/// El calco solo, como un SVG completo (para la vista previa).
pub fn trace_svg(fragment: &str, width: u32, height: u32) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">{fragment}</svg>"#
    )
}

/// El texto de las opciones (para el error de un trabajo mal formado y
/// para las cachés).
pub fn describe(job: &TraceOptions) -> String {
    format!(
        "{}x{} {} colores={} detalle={} esquinas={} ruido={} umbral={}",
        job.width,
        job.height,
        if job.lines { "lines" } else { "color" },
        job.colors,
        fnum(job.paths),
        fnum(job.corners),
        fnum(job.noise),
        job.threshold
    )
}

/// Lo de dentro del `<svg>` raíz de un SVG de VectorCraft, con los `id`
/// prefijados para que no choquen con los del dibujo (`svgFragment`).
pub fn svg_fragment(svg: &str, prefix: &str) -> String {
    let Some(start) = svg.find("<svg") else {
        return String::new();
    };
    let (Some(open), Some(end)) = (svg[start..].find('>'), svg.rfind("</svg>")) else {
        return String::new();
    };
    if start + open + 1 > end {
        return String::new();
    }
    let mut inner = svg[start + open + 1..end].to_string();
    // Sin <title>…</title>.
    while let Some(t) = inner.find("<title>") {
        match inner[t..].find("</title>") {
            Some(e) => inner.replace_range(t..t + e + "</title>".len(), ""),
            None => break,
        }
    }
    let inner = inner.replace("xlink:href=", "href=");
    let inner = prefix_attr(&inner, "id=\"", prefix, true);
    let inner = prefix_attr(&inner, "url(#", prefix, false);
    let inner = prefix_attr(&inner, "href=\"#", prefix, true);
    let inner = remove_data_names(&inner);
    // Sin espacios entre etiquetas.
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '>' {
            out.push('>');
            let rest = &inner[i + 1..];
            let trimmed = rest.trim_start_matches(is_space);
            if trimmed.starts_with('<') && trimmed.len() < rest.len() {
                for _ in 0..rest.len() - trimmed.len() {
                    chars.next();
                }
            }
        } else {
            out.push(c);
        }
    }
    out.trim().to_string()
}

/// Una letra, cifra o `_` (lo que separa `\b` en las expresiones de Go).
fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Los espacios de `\s` en Go: ASCII.
fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c')
}

/// Pone `prefix` delante de lo que sigue a cada `marker` (si `boundary`,
/// solo donde empieza una palabra, como `\b`).
fn prefix_attr(text: &str, marker: &str, prefix: &str, boundary: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut from = 0;
    while let Some(i) = text[from..].find(marker) {
        let at = from + i;
        let starts_word = !boundary || !text[..at].chars().next_back().is_some_and(is_word);
        out.push_str(&text[from..at + marker.len()]);
        if starts_word {
            out.push_str(prefix);
        }
        from = at + marker.len();
    }
    out.push_str(&text[from..]);
    out
}

/// Sin los atributos `data-name="…"` (con el espacio de delante).
fn remove_data_names(text: &str) -> String {
    const ATTR: &str = "data-name=\"";
    let mut out = String::with_capacity(text.len());
    let mut from = 0;
    while let Some(i) = text[from..].find(ATTR) {
        let at = from + i;
        let space = text[..at].chars().next_back().filter(|c| is_space(*c));
        let close = text[at + ATTR.len()..].find('"');
        match (space, close) {
            (Some(sp), Some(end)) => {
                out.push_str(&text[from..at - sp.len_utf8()]);
                from = at + ATTR.len() + end + 1;
            }
            _ => {
                out.push_str(&text[from..at + ATTR.len()]);
                from = at + ATTR.len();
            }
        }
    }
    out.push_str(&text[from..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAGMENT: &str = r##"<path d="M0 0" fill="#FF0000" transform="translate(1,2)"/><path d="M1 1" fill="#0000fe"/><path d="M2 2" fill="#fafafa"/><g><path d="M3 3" fill="#101010"/></g>"##;

    #[test]
    fn colours_are_dropped_and_snapped() {
        let job = TraceOptions {
            drop: vec!["#ffffff".into()],
            palette: vec!["#ee1100".into(), "#0000aa".into(), "#222222".into()],
            ..TraceOptions::default()
        };
        assert_eq!(
            post_trace(FRAGMENT, &job),
            r##"<path d="M0 0" fill="#ee1100" transform="translate(1,2)"/><path d="M1 1" fill="#0000aa"/><g><path d="M3 3" fill="#222222"/></g>"##
        );
        // Sin paleta, los colores quedan como estaban.
        let plain = post_trace(FRAGMENT, &TraceOptions::default());
        assert_eq!(plain, FRAGMENT);
        // En líneas: lo claro fuera y lo oscuro con la tinta.
        let lines = post_trace(
            FRAGMENT,
            &TraceOptions {
                lines: true,
                ink: "#123456".into(),
                ..TraceOptions::default()
            },
        );
        assert_eq!(lines.matches("fill=\"#123456\"").count(), 3, "{lines}");
        assert!(!lines.contains("M2 2"));
    }

    #[test]
    fn colours_are_counted_most_used_first() {
        let frag = r##"<path fill="#AA0000"/><path fill="#aa0000"/><path fill="#00bb00"/><path fill="#0000cc"/>"##;
        let colors = trace_colors(frag);
        assert_eq!(
            colors,
            [
                TraceColor {
                    hex: "#aa0000".into(),
                    paths: 2
                },
                TraceColor {
                    hex: "#0000cc".into(),
                    paths: 1
                },
                TraceColor {
                    hex: "#00bb00".into(),
                    paths: 1
                },
            ]
        );
    }

    #[test]
    fn vectorcraft_fragments_get_unique_ids() {
        let svg = r##"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg" width="10"><title>x</title>
  <defs><linearGradient id="g1"/></defs>
  <path data-name="forma" fill="url(#g1)" d="M0 0"/>
  <use xlink:href="#g1"/></svg>"##;
        assert_eq!(
            svg_fragment(svg, "vcab-"),
            r##"<defs><linearGradient id="vcab-g1"/></defs><path fill="url(#vcab-g1)" d="M0 0"/><use href="#vcab-g1"/>"##
        );
        assert_eq!(svg_fragment("nada", "p"), "");
    }

    #[test]
    fn colour_helpers_match_pxcli() {
        assert_eq!(hex_rgb("#102030"), [16.0, 32.0, 48.0]);
        assert_eq!(rgb_hex([16.0, 32.0, 48.0]), "#102030");
        assert_eq!(color_dist([0.0; 3], [0.0; 3]), 0.0);
        assert!((luminance([255.0; 3]) - 1.0).abs() < 1e-12);
    }
}
