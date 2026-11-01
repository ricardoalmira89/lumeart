//! Modo vector: lo que no depende del motor de JavaScript ni del
//! rasterizador. `dibujo.js` llama a una librería de dibujo (`lib.js`) y el
//! resultado es un documento SVG (`internal/vec/vec.go` de pxcli).

pub mod drawing;
pub mod raster;
pub mod reference;
pub mod rig;
pub mod scene;
pub mod studio;
pub mod trace;

pub use crate::anim::{Issue, Level};

/// Nombre del script en los errores y en los números de línea.
pub const FILE_NAME: &str = "dibujo.js";

/// Lo que dio una ejecución de `dibujo.js`. Si el script falla a medias, el
/// SVG tiene todo lo dibujado hasta el error.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VecResult {
    /// Tamaño del dibujo, en unidades (el `viewBox`).
    pub width: i64,
    pub height: i64,
    /// SVG para el studio: cada forma con `data-i`, `data-l` y `data-h`.
    pub svg: String,
    /// SVG limpio para guardar y compartir.
    pub clean: String,
    /// Formas dibujadas.
    pub count: i64,
    pub issues: Vec<Issue>,
    /// Huesos, piezas y acciones, si `dibujo.js` declara un esqueleto.
    pub rig: Option<rig::Rig>,
    /// Las capas partidas en piezas (solo al exportar una escena).
    pub scene: Option<scene::Scene>,
}

impl VecResult {
    /// Solo los errores.
    pub fn errors(&self) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|i| i.level == Level::Error)
            .collect()
    }

    /// Añade un problema.
    pub fn issue(&mut self, level: Level, line: u32, msg: impl Into<String>) {
        self.issues.push(Issue {
            level,
            line,
            place: String::new(),
            msg: msg.into(),
        });
    }
}

/// Las líneas con que la CLI cuenta los problemas (`printVecIssues` de
/// pxcli): `ERROR (línea 3): …` o `aviso: …`.
pub fn issue_lines(res: &VecResult) -> Vec<String> {
    res.issues
        .iter()
        .map(|is| {
            let label = if is.level == Level::Error {
                "ERROR"
            } else {
                "aviso"
            };
            let place = if is.line > 0 {
                format!(" (línea {})", is.line)
            } else {
                String::new()
            };
            format!("{label}{place}: {}", is.msg)
        })
        .collect()
}

/// El error con que termina la CLI si `dibujo.js` tiene errores.
pub fn errors_message(count: usize) -> String {
    format!("{FILE_NAME} tiene {count} error(es): lo dibujado hasta el error sí se ha generado")
}

/// El `dibujo.js` con que empieza un dibujo nuevo (`Template` de pxcli).
pub fn template(width: u32, height: u32, transparent: bool) -> String {
    let (bg, note) = if transparent {
        ("null", "null = fondo transparente (objeto suelto)")
    } else {
        (
            "\"#f2ede4\"",
            "pon el color o el degradado que pida la escena",
        )
    };
    format!(
        "// dibujo.js: cada forma se dibuja encima de las anteriores.\n\
         // Lienzo de {width}x{height}: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.\n\
         canvas({{ width: {width}, height: {height}, background: {bg} }}) // {note}\n"
    )
}

/// `--size ANCHOxALTO` de `vec init` (de 16 a 4096 por lado).
pub fn parse_size(raw: &str) -> Result<(u32, u32), String> {
    let lower = raw.trim().to_lowercase();
    let parts: Vec<&str> = lower.split('x').collect();
    if parts.len() != 2 {
        return Err(format!(
            "tamaño no válido {} (usa ANCHOxALTO)",
            crate::error::go_quote(raw)
        ));
    }
    match (go_atoi(parts[0]), go_atoi(parts[1])) {
        (Some(w), Some(h)) if (16..=4096).contains(&w) && (16..=4096).contains(&h) => {
            Ok((w as u32, h as u32))
        }
        _ => Err(format!(
            "tamaño no válido {} (entre 16 y 4096)",
            crate::error::go_quote(raw)
        )),
    }
}

/// `strconv.Atoi` de Go: signo opcional y dígitos, nada más.
fn go_atoi(s: &str) -> Option<i64> {
    let digits = s.strip_prefix(['+', '-']).unwrap_or(s);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// `--crop x,y,ancho,alto` de `vec render`.
pub fn parse_crop(raw: &str) -> Result<[f64; 4], String> {
    let parts: Vec<&str> = raw.split(',').collect();
    if parts.len() != 4 {
        return Err("--crop tiene que ser x,y,ancho,alto".into());
    }
    let mut crop = [0.0; 4];
    for (i, p) in parts.iter().enumerate() {
        crop[i] = go_parse_float(p.trim())
            .ok_or_else(|| format!("--crop: {} no es un número", crate::error::go_quote(p)))?;
    }
    if crop[2] <= 0.0 || crop[3] <= 0.0 {
        return Err("--crop: el ancho y el alto tienen que ser mayores que 0".into());
    }
    Ok(crop)
}

/// `strconv.ParseFloat` de Go para lo que se escribe en la terminal:
/// decimales, exponentes, `inf` y `nan`.
fn go_parse_float(s: &str) -> Option<f64> {
    let lower = s.to_ascii_lowercase();
    let body = lower.trim_start_matches(['+', '-']);
    if matches!(body, "inf" | "infinity" | "nan") {
        return s.parse().ok();
    }
    if body.is_empty()
        || !body
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'+' | b'-' | b'_'))
    {
        return None;
    }
    s.replace('_', "").parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_template_is_pxclis() {
        assert_eq!(
            template(1024, 768, false),
            "// dibujo.js: cada forma se dibuja encima de las anteriores.\n// Lienzo de 1024x768: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.\ncanvas({ width: 1024, height: 768, background: \"#f2ede4\" }) // pon el color o el degradado que pida la escena\n"
        );
        assert!(template(64, 64, true).contains(
            "canvas({ width: 64, height: 64, background: null }) // null = fondo transparente (objeto suelto)"
        ));
    }

    #[test]
    fn sizes_and_crops_like_pxcli() {
        assert_eq!(parse_size("1024x768"), Ok((1024, 768)));
        assert_eq!(parse_size(" 64X32 "), Ok((64, 32)));
        assert_eq!(
            parse_size("10x10"),
            Err("tamaño no válido \"10x10\" (entre 16 y 4096)".into())
        );
        assert_eq!(
            parse_size("grande"),
            Err("tamaño no válido \"grande\" (usa ANCHOxALTO)".into())
        );
        assert_eq!(parse_crop("10, 20,30.5,40"), Ok([10.0, 20.0, 30.5, 40.0]));
        assert_eq!(
            parse_crop("1,2,3"),
            Err("--crop tiene que ser x,y,ancho,alto".into())
        );
        assert_eq!(
            parse_crop("1,2,a,4"),
            Err("--crop: \"a\" no es un número".into())
        );
        assert_eq!(
            parse_crop("1,2,0,4"),
            Err("--crop: el ancho y el alto tienen que ser mayores que 0".into())
        );
    }

    #[test]
    fn issues_read_like_pxcli() {
        let mut res = VecResult::default();
        res.issue(Level::Error, 3, "circle: r tiene que ser un número");
        res.issue(Level::Warning, 0, "el dibujo está vacío");
        assert_eq!(
            issue_lines(&res),
            [
                "ERROR (línea 3): circle: r tiene que ser un número",
                "aviso: el dibujo está vacío"
            ]
        );
        assert_eq!(res.errors().len(), 1);
        assert_eq!(
            errors_message(2),
            "dibujo.js tiene 2 error(es): lo dibujado hasta el error sí se ha generado"
        );
    }
}
