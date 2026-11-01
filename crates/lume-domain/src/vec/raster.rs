//! Cómo se convierte un dibujo vectorial en imagen, como
//! `internal/vec/raster.go` de pxcli: el encuadre (recorte y tamaño de
//! salida), la rejilla con coordenadas y el SVG compuesto (dibujo, rejilla y
//! superposición) que pinta el rasterizador.
//!
//! pxcli componía una página HTML con un `<svg>` por capa y la fotografiaba
//! con Chrome; aquí las mismas capas van como `<svg>` anidados dentro de uno
//! solo, con los mismos atributos.

use crate::DomainError;

/// Ancho de salida si no se pide otro.
pub const DEFAULT_WIDTH: u32 = 1024;
/// Lado máximo de la imagen de salida.
pub const MAX_SIZE: u32 = 4096;

/// Qué parte del dibujo se ve y a qué tamaño.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// `x, y, ancho, alto` visibles, en unidades del dibujo.
    pub view: [f64; 4],
    /// Tamaño de la imagen en píxeles.
    pub width: u32,
    pub height: u32,
}

/// El encuadre de un dibujo de `width` × `height` unidades: el recorte (si
/// tiene ancho y alto) o todo, a `out_width` píxeles de ancho (1024 si no se
/// da, como mucho 4096) y el alto en proporción.
pub fn frame(
    width: f64,
    height: f64,
    crop: Option<[f64; 4]>,
    out_width: Option<u32>,
) -> Result<Frame, DomainError> {
    let view = match crop {
        Some(c) if c[2] > 0.0 && c[3] > 0.0 => c,
        _ => [0.0, 0.0, width, height],
    };
    let out_w = match out_width {
        Some(w) if w > 0 => w.min(MAX_SIZE),
        _ => DEFAULT_WIDTH,
    };
    // `int(math.Round(...))` de Go: NaN o infinito no dan un tamaño válido.
    let h = (f64::from(out_w) * view[3] / view[2]).round();
    let out_h = if h.is_finite() { h as i64 } else { 0 };
    if out_h < 1 || out_h > i64::from(MAX_SIZE) {
        return Err(DomainError::invalid_args(format!(
            "tamaño de salida no válido: {out_w}x{out_h}"
        )));
    }
    Ok(Frame {
        view,
        width: out_w,
        height: out_h as u32,
    })
}

impl Frame {
    /// Atributos de cada capa: su `viewBox`, el tamaño de salida y el
    /// encaje centrado (`box` en pxcli).
    pub fn box_attrs(&self) -> String {
        let [vx, vy, vw, vh] = self.view;
        format!(
            r#"viewBox="{} {} {} {}" width="{}" height="{}" preserveAspectRatio="xMidYMid meet""#,
            num(vx),
            num(vy),
            num(vw),
            num(vh),
            self.width,
            self.height
        )
    }
}

const SVG_NS: &str = r#"xmlns="http://www.w3.org/2000/svg""#;

/// Cambia la etiqueta `<svg …>` del principio de `svg` (si empieza por ella)
/// por `<svg xmlns=… {attrs}>`, como `reSVGOpen` de pxcli.
pub fn replace_root(svg: &str, attrs: &str) -> String {
    if !svg.starts_with("<svg") {
        return svg.to_string();
    }
    match svg.find('>') {
        Some(end) => format!("<svg {SVG_NS} {attrs}>{}", &svg[end + 1..]),
        None => svg.to_string(),
    }
}

/// El SVG que se pinta: el dibujo encuadrado, la rejilla (cada `grid`
/// unidades) y la superposición (SVG en unidades del dibujo), cada uno en su
/// `<svg>` con el mismo encuadre, dentro de uno del tamaño de salida. El
/// fondo no va aquí: lo pone el rasterizador debajo de todo.
pub fn compose(svg: &str, frame: &Frame, grid: Option<f64>, overlay: Option<&str>) -> String {
    let attrs = frame.box_attrs();
    let mut out = format!(
        r#"<svg {SVG_NS} xmlns:xlink="http://www.w3.org/1999/xlink" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
        w = frame.width,
        h = frame.height
    );
    out.push_str(&replace_root(svg, &attrs));
    if let Some(step) = grid.filter(|s| *s > 0.0) {
        let [vx, vy, vw, vh] = frame.view;
        out.push_str(&format!("<svg {SVG_NS} {attrs}>"));
        out.push_str(&grid_lines(vx, vy, vw, vh, step));
        out.push_str("</svg>");
    }
    if let Some(overlay) = overlay.filter(|o| !o.is_empty()) {
        out.push_str(&format!("<svg {SVG_NS} {attrs}>{overlay}</svg>"));
    }
    out.push_str("</svg>");
    out
}

/// Rejilla con coordenadas cada `step` unidades sobre la zona
/// (`vx`, `vy`, `vw`, `vh`): líneas rosas semitransparentes y su valor
/// escrito junto a cada una (`gridLines` de pxcli, mismo texto).
pub fn grid_lines(vx: f64, vy: f64, vw: f64, vh: f64, step: f64) -> String {
    let mut b = String::new();
    if step <= 0.0 || !step.is_finite() {
        return b;
    }
    let fs = vw.max(vh) / 70.0;
    let sw = vw.max(vh) / 900.0;
    let mut x = (vx / step).ceil() * step;
    while x <= vx + vw {
        line(&mut b, x, vy, x, vy + vh, sw);
        label(&mut b, x + fs * 0.2, vy + fs * 1.1, fs, x);
        x += step;
    }
    let mut y = (vy / step).ceil() * step;
    while y <= vy + vh {
        line(&mut b, vx, y, vx + vw, y, sw);
        label(&mut b, vx + fs * 0.2, y - fs * 0.25, fs, y);
        y += step;
    }
    b
}

fn line(b: &mut String, x1: f64, y1: f64, x2: f64, y2: f64, sw: f64) {
    b.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#ff2d7a" stroke-opacity=".45" stroke-width="{}"/>"##,
        num(x1),
        num(y1),
        num(x2),
        num(y2),
        num(sw)
    ));
}

fn label(b: &mut String, x: f64, y: f64, fs: f64, value: f64) {
    b.push_str(&format!(
        r##"<text x="{}" y="{}" font-size="{}" font-family="DejaVu Sans Mono, monospace" fill="#ff2d7a" stroke="#fff" stroke-width="{}" paint-order="stroke">{}</text>"##,
        num(x),
        num(y),
        num(fs),
        num(fs / 5.0),
        num(value)
    ));
}

/// Un número con dos decimales como mucho y sin ceros al final (`num` de
/// pxcli): `12`, `0.5`, `-3.25`; nunca `-0`.
pub fn num(v: f64) -> String {
    let s = format!("{v:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_like_pxcli() {
        assert_eq!(num(12.0), "12");
        assert_eq!(num(0.5), "0.5");
        assert_eq!(num(-3.25), "-3.25");
        assert_eq!(num(-0.001), "0");
        assert_eq!(num(1.005), "1"); // 1.00499999… en binario
        assert_eq!(num(100.0), "100");
        assert_eq!(num(14.628571428571428), "14.63");
    }

    #[test]
    fn the_frame_follows_the_crop_and_the_width() {
        let f = frame(768.0, 1024.0, None, None).unwrap();
        assert_eq!((f.width, f.height), (1024, 1365));
        assert_eq!(f.view, [0.0, 0.0, 768.0, 1024.0]);
        let f = frame(768.0, 1024.0, Some([100.0, 50.0, 200.0, 100.0]), Some(400)).unwrap();
        assert_eq!((f.width, f.height), (400, 200));
        assert_eq!(f.view, [100.0, 50.0, 200.0, 100.0]);
        // Un recorte sin ancho o alto es todo el dibujo.
        let f = frame(100.0, 50.0, Some([10.0, 10.0, 0.0, 5.0]), Some(0)).unwrap();
        assert_eq!((f.width, f.height, f.view[2]), (1024, 512, 100.0));
        // Ancho como mucho 4096; el alto, también.
        assert_eq!(frame(100.0, 100.0, None, Some(9000)).unwrap().width, 4096);
        assert_eq!(
            frame(10.0, 100.0, None, None).unwrap_err().message,
            "tamaño de salida no válido: 1024x10240"
        );
        assert_eq!(
            frame(1000.0, 1.0, None, Some(100)).unwrap_err().message,
            "tamaño de salida no válido: 100x0"
        );
    }

    #[test]
    fn layers_share_the_frame() {
        let f = frame(200.0, 100.0, Some([50.0, 25.0, 100.0, 50.0]), Some(400)).unwrap();
        let svg = compose(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100" width="200" height="100"><rect width="10" height="10"/></svg>"#,
            &f,
            Some(50.0),
            Some(r#"<circle r="3"/>"#),
        );
        let attrs = r#"viewBox="50 25 100 50" width="400" height="200" preserveAspectRatio="xMidYMid meet""#;
        assert!(svg.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="400" height="200" viewBox="0 0 400 200">"#), "{svg}");
        assert_eq!(svg.matches(attrs).count(), 3, "{svg}");
        assert!(svg.contains(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" {attrs}><rect width="10" height="10"/></svg>"#
        )));
        assert!(svg.ends_with(r#"<circle r="3"/></svg></svg>"#), "{svg}");
        // Sin rejilla ni superposición, solo el dibujo.
        let plain = compose("<svg><g/></svg>", &f, None, Some(""));
        assert_eq!(plain.matches("<svg").count(), 2);
        assert!(!plain.contains("<line"));
    }

    #[test]
    fn the_grid_marks_every_step_inside_the_zone() {
        let g = grid_lines(50.0, 25.0, 100.0, 50.0, 50.0);
        // x = 50, 100, 150; y = 50 (25 y 75 no son múltiplos de 50).
        assert_eq!(g.matches("<line").count(), 4);
        assert!(
            g.contains(r#"<line x1="100" y1="25" x2="100" y2="75""#),
            "{g}"
        );
        assert!(g.contains(">150</text>"), "{g}");
        assert!(
            g.contains(r#"<line x1="50" y1="50" x2="150" y2="50""#),
            "{g}"
        );
        assert_eq!(grid_lines(0.0, 0.0, 10.0, 10.0, 0.0), "");
    }
}
