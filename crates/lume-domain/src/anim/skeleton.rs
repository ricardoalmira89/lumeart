//! El `anim.js` de partida (`internal/anim/skeleton.go`): la receta de la
//! base como una pieza, un reposo que la reproduce y una función por acción.

use std::fmt::Write;

use crate::error::go_quote;

use super::ActionSpec;
use super::hex;
use super::recipe::{Cmd, Op};

/// Qué `anim.js` generar a partir de un dibujo base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkeletonOptions {
    pub name: String,
    /// Nombre del PNG de la base, relativo a `anim.js`.
    pub base: String,
    pub cell: (u32, u32),
    /// Dónde cae el (0, 0) de la base dentro de cada celda.
    pub origin: (i64, i64),
    pub recipe: Vec<Cmd>,
    /// La receta se rehízo con los píxeles, no con los comandos originales.
    pub from_pixels: bool,
    pub actions: Vec<ActionSpec>,
}

/// Coloca una base de tamaño `base` en una celda: centrada en horizontal y
/// apoyada abajo, o centrada del todo (`anchor` = `center`).
pub fn origin(base: (u32, u32), cell: (u32, u32), anchor: &str) -> (i64, i64) {
    let x = (i64::from(cell.0) - i64::from(base.0)) / 2;
    let y = if anchor == "center" {
        (i64::from(cell.1) - i64::from(base.1)) / 2
    } else {
        i64::from(cell.1) - i64::from(base.1)
    };
    (x, y)
}

/// Escribe el `anim.js` inicial.
pub fn skeleton(o: &SkeletonOptions) -> String {
    let mut b = String::new();
    let _ = writeln!(
        b,
        "// Animación {} · celdas de {}x{} · la base empieza en ({}, {}) dentro de cada celda.",
        go_quote(&o.name),
        o.cell.0,
        o.cell.1,
        o.origin.0,
        o.origin.1
    );
    let _ = writeln!(
        b,
        "config({{ name: {}, base: {}, cell: [{}, {}], origin: [{}, {}], colors: [] }});\n",
        go_quote(&o.name),
        go_quote(&o.base),
        o.cell.0,
        o.cell.1,
        o.origin.0,
        o.origin.1
    );
    if o.from_pixels {
        b.push_str("// Receta del dibujo base, reconstruida a partir de sus píxeles (rectángulos por color).\n");
    } else {
        b.push_str(
            "// Receta del dibujo base: los mismos comandos de pxcli que lo dibujaron, en orden.\n",
        );
    }
    b.push_str("// Repártela en piezas con nombre para poder moverlas. El orden de dibujo hace de capas:\n");
    b.push_str("// lo que se dibuja después queda encima.\n");
    b.push_str("part(\"base\", () => {\n");
    for c in &o.recipe {
        b.push_str("  ");
        b.push_str(&cmd_js(c));
        b.push('\n');
    }
    b.push_str("});\n\n");
    b.push_str("// La pose de reposo tiene que ser idéntica al dibujo base (lo comprueba `pxcli anim check`).\n");
    b.push_str("rest(() => {\n  draw(\"base\");\n});\n\n");
    b.push_str("// Una función por acción: dibuja el frame f desde cero. t va de 0 a 1.\n");
    for a in &o.actions {
        let _ = write!(
            b,
            "action({}, {{ frames: {}, fps: {}, loop: {} }}, (f, {{ t }}) => {{\n  draw(\"base\");\n}});\n\n",
            go_quote(&a.name),
            a.frames,
            a.fps,
            a.looping
        );
    }
    b
}

/// Un comando de la receta en JavaScript.
pub fn cmd_js(c: &Cmd) -> String {
    let col = go_quote(&hex(c.color));
    let a = &c.args;
    match c.op {
        Op::SetPixel => format!("set_pixel({}, {}, {col});", a[0], a[1]),
        Op::FillRect => format!("fill_rect({}, {}, {}, {}, {col});", a[0], a[1], a[2], a[3]),
        Op::Line => format!("line({}, {}, {}, {}, {col});", a[0], a[1], a[2], a[3]),
        Op::Clear => format!("clear({col});"),
    }
}
