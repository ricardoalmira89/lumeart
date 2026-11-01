//! Herramientas de dibujo del modo vector: rectángulo, elipse, línea, pincel
//! y texto. Lo que se dibuja con el ratón se convierte en una línea de
//! `dibujo.js` (al final del archivo), así que se puede deshacer, el agente
//! la ve y se puede retocar a mano o con el lazo.

use eframe::egui::Color32;

/// Colores y grosor con los que se dibuja.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawStyle {
    pub fill: Color32,
    /// Con relleno (rectángulo, elipse, texto).
    pub fill_on: bool,
    pub stroke: Color32,
    /// Con borde (rectángulo y elipse; la línea y el pincel siempre lo usan).
    pub stroke_on: bool,
    /// Grosor del borde, de la línea o del pincel (en unidades del dibujo).
    pub width: f32,
    /// Tamaño del texto.
    pub text_size: f32,
}

impl Default for DrawStyle {
    fn default() -> Self {
        Self {
            fill: Color32::from_rgb(0xE9, 0xA2, 0x3B),
            fill_on: true,
            stroke: Color32::from_rgb(0x1F, 0x24, 0x30),
            stroke_on: false,
            width: 6.0,
            text_size: 48.0,
        }
    }
}

/// Qué se está dibujando.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Rect,
    Ellipse,
    Line,
    Brush,
}

/// `#rrggbb` (o `#rrggbbaa` si es semitransparente).
pub fn hex(c: Color32) -> String {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    if a == 255 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

/// Un número corto: entero si lo es, si no con un decimal.
pub fn num(v: f64) -> String {
    let r = (v * 10.0).round() / 10.0;
    if r.fract() == 0.0 {
        format!("{}", r as i64)
    } else {
        format!("{r:.1}")
    }
}

/// El estilo de una forma cerrada: relleno y, si lo hay, borde.
fn closed_style(s: &DrawStyle) -> String {
    let fill = if s.fill_on {
        format!("fill: \"{}\"", hex(s.fill))
    } else {
        "fill: \"none\"".into()
    };
    if s.stroke_on || !s.fill_on {
        format!(
            "{{ {fill}, stroke: \"{}\", width: {} }}",
            hex(s.stroke),
            num(f64::from(s.width))
        )
    } else {
        format!("{{ {fill} }}")
    }
}

/// Con Mayús: cuadrado/círculo o línea a 45°.
fn constrain(shape: Shape, a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    match shape {
        Shape::Rect | Shape::Ellipse => {
            let side = dx.abs().max(dy.abs());
            [a[0] + side * dx.signum(), a[1] + side * dy.signum()]
        }
        Shape::Line => {
            let angle = dy.atan2(dx);
            let step = std::f64::consts::FRAC_PI_4;
            let snapped = (angle / step).round() * step;
            let len = dx.hypot(dy);
            [a[0] + len * snapped.cos(), a[1] + len * snapped.sin()]
        }
        Shape::Brush => b,
    }
}

/// La punta final de un arrastre (con Mayús, ajustada).
pub fn end_point(shape: Shape, a: [f64; 2], b: [f64; 2], shift: bool) -> [f64; 2] {
    if shift { constrain(shape, a, b) } else { b }
}

/// La línea de `dibujo.js` de una forma arrastrada de `a` a `b` (o por
/// `points` con el pincel). `None` si es demasiado pequeña.
pub fn shape_code(shape: Shape, points: &[[f64; 2]], style: &DrawStyle) -> Option<String> {
    let (&a, &b) = (points.first()?, points.last()?);
    let (x0, y0) = (a[0].min(b[0]), a[1].min(b[1]));
    let (w, h) = ((b[0] - a[0]).abs(), (b[1] - a[1]).abs());
    match shape {
        Shape::Rect => (w >= 1.0 && h >= 1.0).then(|| {
            format!(
                "rect({}, {}, {}, {}, {})",
                num(x0),
                num(y0),
                num(w),
                num(h),
                closed_style(style)
            )
        }),
        Shape::Ellipse => (w >= 1.0 && h >= 1.0).then(|| {
            let (cx, cy) = (x0 + w / 2.0, y0 + h / 2.0);
            if (w - h).abs() < 0.05 {
                format!(
                    "circle({}, {}, {}, {})",
                    num(cx),
                    num(cy),
                    num(w / 2.0),
                    closed_style(style)
                )
            } else {
                format!(
                    "ellipse({}, {}, {}, {}, {})",
                    num(cx),
                    num(cy),
                    num(w / 2.0),
                    num(h / 2.0),
                    closed_style(style)
                )
            }
        }),
        Shape::Line => (w.hypot(h) >= 1.0).then(|| {
            format!(
                "line({}, {}, {}, {}, {{ stroke: \"{}\", width: {} }})",
                num(a[0]),
                num(a[1]),
                num(b[0]),
                num(b[1]),
                hex(style.stroke),
                num(f64::from(style.width))
            )
        }),
        Shape::Brush => {
            if points.len() < 2 {
                return None;
            }
            let list: Vec<String> = points
                .iter()
                .map(|p| format!("[{}, {}]", num(p[0]), num(p[1])))
                .collect();
            Some(format!(
                "brush([{}], {{ fill: \"{}\", width: {} }})",
                list.join(", "),
                hex(style.stroke),
                num(f64::from(style.width))
            ))
        }
    }
}

/// La línea de un texto en `(x, y)`.
pub fn text_code(at: [f64; 2], text: &str, style: &DrawStyle) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let quoted = serde_json::to_string(text).ok()?;
    Some(format!(
        "text({}, {}, {quoted}, {{ fill: \"{}\", size: {} }})",
        num(at[0]),
        num(at[1]),
        hex(style.fill),
        num(f64::from(style.text_size))
    ))
}

/// `source` con `code` al final (en su propia línea) y el número de esa
/// línea (de 1 en adelante).
pub fn append(source: &str, code: &str) -> (String, usize) {
    let mut out = source.trim_end().to_string();
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(code);
    out.push('\n');
    let line = out.trim_end().lines().count();
    (out, line)
}
