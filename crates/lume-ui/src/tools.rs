//! Herramientas de dibujo como máquina de estados (patrón State): reciben
//! eventos de puntero en coordenadas de píxel y devuelven una vista previa
//! mientras se arrastra y, al soltar, un único `Command` (un paso de
//! deshacer). No saben nada de egui: se prueban con eventos simulados.

use std::collections::HashSet;

use lume_domain::canvas::bresenham;
use lume_domain::{Canvas, Command, Rgba};

/// Botón del puntero: el principal pinta con el color principal y el
/// secundario con el secundario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    Primary,
    Secondary,
}

/// Evento de puntero sobre el lienzo, en píxeles (puede caer fuera).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerEvent {
    Down { x: i64, y: i64, button: Button },
    Move { x: i64, y: i64 },
    Up { x: i64, y: i64 },
}

/// Herramientas que dibujan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawTool {
    Pencil,
    Eraser,
    Fill,
    Line,
    Rect,
    Ellipse,
    Eyedropper,
}

impl DrawTool {
    pub fn label(self) -> &'static str {
        match self {
            DrawTool::Pencil => "Lápiz",
            DrawTool::Eraser => "Borrador",
            DrawTool::Fill => "Relleno",
            DrawTool::Line => "Línea",
            DrawTool::Rect => "Rectángulo",
            DrawTool::Ellipse => "Elipse",
            DrawTool::Eyedropper => "Cuentagotas",
        }
    }
}

/// Opciones de las herramientas (barra de opciones).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolOptions {
    /// Lado del pincel en píxeles (1–16).
    pub size: u32,
    /// Quita las esquinas en L de los trazos del lápiz.
    pub pixel_perfect: bool,
    /// Repite el trazo reflejado en horizontal (eje vertical central).
    pub mirror_x: bool,
    /// Repite el trazo reflejado en vertical (eje horizontal central).
    pub mirror_y: bool,
    /// Rectángulo y elipse rellenos (si no, solo el contorno).
    pub filled: bool,
}

impl Default for ToolOptions {
    fn default() -> Self {
        Self {
            size: 1,
            pixel_perfect: true,
            mirror_x: false,
            mirror_y: false,
            filled: false,
        }
    }
}

/// Lo que devuelve una herramienta al recibir un evento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolResult {
    Nothing,
    /// El trazo terminó: aplicar este comando con esta etiqueta de historial.
    Commit {
        command: Command,
        label: String,
    },
    /// El cuentagotas eligió un color para este botón.
    PickColor {
        color: Rgba,
        button: Button,
    },
}

/// Estado en curso de una herramienta.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum State {
    #[default]
    Idle,
    /// Lápiz o borrador: puntos recorridos.
    Stroke {
        points: Vec<(i64, i64)>,
        color: Rgba,
    },
    /// Línea, rectángulo o elipse: desde dónde y hasta dónde.
    Drag {
        start: (i64, i64),
        end: (i64, i64),
        color: Rgba,
    },
}

/// Una herramienta con su estado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolMachine {
    pub tool: DrawTool,
    state: State,
}

impl ToolMachine {
    pub fn new(tool: DrawTool) -> Self {
        Self {
            tool,
            state: State::Idle,
        }
    }

    /// ¿Hay un trazo o arrastre en curso?
    pub fn is_active(&self) -> bool {
        self.state != State::Idle
    }

    /// Cancela lo que esté en curso (Esc).
    pub fn cancel(&mut self) {
        self.state = State::Idle;
    }

    /// Atiende un evento de puntero.
    pub fn handle(
        &mut self,
        event: PointerEvent,
        canvas: &Canvas,
        primary: Rgba,
        secondary: Rgba,
        options: &ToolOptions,
    ) -> ToolResult {
        let pick = |button: Button| match button {
            Button::Primary => primary,
            Button::Secondary => secondary,
        };
        match (self.tool, event) {
            (DrawTool::Eyedropper, PointerEvent::Down { x, y, button }) => match canvas.get(x, y) {
                Ok(color) => ToolResult::PickColor { color, button },
                Err(_) => ToolResult::Nothing,
            },
            (DrawTool::Fill, PointerEvent::Down { x, y, button }) => {
                match flood_fill(canvas, x, y, pick(button)) {
                    Some((command, count)) => ToolResult::Commit {
                        command,
                        label: format!("Relleno · {}", pixels(count)),
                    },
                    None => ToolResult::Nothing,
                }
            }
            (DrawTool::Pencil | DrawTool::Eraser, PointerEvent::Down { x, y, button }) => {
                let color = if self.tool == DrawTool::Eraser {
                    Rgba::TRANSPARENT
                } else {
                    pick(button)
                };
                self.state = State::Stroke {
                    points: vec![(x, y)],
                    color,
                };
                ToolResult::Nothing
            }
            (
                DrawTool::Line | DrawTool::Rect | DrawTool::Ellipse,
                PointerEvent::Down { x, y, button },
            ) => {
                self.state = State::Drag {
                    start: (x, y),
                    end: (x, y),
                    color: pick(button),
                };
                ToolResult::Nothing
            }
            (_, PointerEvent::Move { x, y }) => {
                match &mut self.state {
                    State::Stroke { points, .. } => {
                        let last = *points.last().expect("un trazo tiene al menos un punto");
                        if last != (x, y) {
                            points.extend(bresenham(last.0, last.1, x, y).into_iter().skip(1));
                            if options.pixel_perfect && self.tool == DrawTool::Pencil {
                                *points = pixel_perfect(points);
                            }
                        }
                    }
                    State::Drag { end, .. } => *end = (x, y),
                    State::Idle => {}
                }
                ToolResult::Nothing
            }
            (_, PointerEvent::Up { x, y }) => {
                // El último movimiento cuenta.
                let _ = self.handle(
                    PointerEvent::Move { x, y },
                    canvas,
                    primary,
                    secondary,
                    options,
                );
                let cells = self.cells(canvas, options);
                let color = self.color();
                self.state = State::Idle;
                let Some(color) = color else {
                    return ToolResult::Nothing;
                };
                let count = cells.len();
                match cells_to_command(&cells, color) {
                    Some(command) => ToolResult::Commit {
                        command,
                        label: format!("{} · {}", self.tool.label(), pixels(count)),
                    },
                    None => ToolResult::Nothing,
                }
            }
        }
    }

    fn color(&self) -> Option<Rgba> {
        match &self.state {
            State::Stroke { color, .. } | State::Drag { color, .. } => Some(*color),
            State::Idle => None,
        }
    }

    /// Celdas (dentro del lienzo, sin repetir) que pintaría el trazo actual.
    pub fn cells(&self, canvas: &Canvas, options: &ToolOptions) -> Vec<(u32, u32)> {
        let raw: Vec<(i64, i64)> = match &self.state {
            State::Idle => return Vec::new(),
            State::Stroke { points, .. } => points.clone(),
            State::Drag { start, end, .. } => match self.tool {
                DrawTool::Line => bresenham(start.0, start.1, end.0, end.1),
                DrawTool::Rect => rect_cells(*start, *end, options.filled),
                DrawTool::Ellipse => ellipse_cells(*start, *end, options.filled),
                _ => Vec::new(),
            },
        };
        let brush = match self.tool {
            DrawTool::Rect | DrawTool::Ellipse if options.filled => 1,
            _ => options.size.clamp(1, 16),
        };
        expand(&raw, brush, canvas, options)
    }

    /// Vista previa: celdas y color del trazo en curso.
    pub fn preview(
        &self,
        canvas: &Canvas,
        options: &ToolOptions,
    ) -> Option<(Vec<(u32, u32)>, Rgba)> {
        let color = self.color()?;
        Some((self.cells(canvas, options), color))
    }
}

fn pixels(n: usize) -> String {
    if n == 1 {
        "1 píxel".into()
    } else {
        format!("{n} píxeles")
    }
}

/// Quita las esquinas en L: si un punto tiene un vecino ortogonal antes y
/// otro después, y esos dos son diagonales entre sí, sobra.
pub fn pixel_perfect(points: &[(i64, i64)]) -> Vec<(i64, i64)> {
    let mut out: Vec<(i64, i64)> = Vec::with_capacity(points.len());
    for &p in points {
        if out.last() == Some(&p) {
            continue;
        }
        out.push(p);
        let n = out.len();
        if n >= 3 {
            let (a, b, c) = (out[n - 3], out[n - 2], out[n - 1]);
            let ortho = |p: (i64, i64), q: (i64, i64)| (p.0 - q.0).abs() + (p.1 - q.1).abs() == 1;
            let diag = (a.0 - c.0).abs() == 1 && (a.1 - c.1).abs() == 1;
            if ortho(a, b) && ortho(b, c) && diag {
                out.remove(n - 2);
            }
        }
    }
    out
}

/// Celdas de un rectángulo de `a` a `b` (contorno o relleno).
pub fn rect_cells(a: (i64, i64), b: (i64, i64), filled: bool) -> Vec<(i64, i64)> {
    let (x0, x1) = (a.0.min(b.0), a.0.max(b.0));
    let (y0, y1) = (a.1.min(b.1), a.1.max(b.1));
    let mut out = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            if filled || x == x0 || x == x1 || y == y0 || y == y1 {
                out.push((x, y));
            }
        }
    }
    out
}

/// Celdas de la elipse inscrita en el rectángulo de `a` a `b` (algoritmo de
/// Alois Zingl, «plotEllipseRect»), contorno o relleno.
pub fn ellipse_cells(a: (i64, i64), b: (i64, i64), filled: bool) -> Vec<(i64, i64)> {
    let (mut x0, mut y0, mut x1, mut y1) = (a.0, a.1, b.0, b.1);
    let mut outline: Vec<(i64, i64)> = Vec::new();
    let mut aa = (x1 - x0).abs();
    let bb = (y1 - y0).abs();
    let mut b1 = bb & 1;
    let mut dx = 4 * (1 - aa) * bb * bb;
    let mut dy = 4 * (b1 + 1) * aa * aa;
    let mut err = dx + dy + b1 * aa * aa;
    if x0 > x1 {
        x0 = x1;
        x1 += aa;
    }
    if y0 > y1 {
        y0 = y1;
    }
    y0 += (bb + 1) / 2;
    y1 = y0 - b1;
    aa = 8 * aa * aa;
    b1 = 8 * bb * bb;
    loop {
        outline.push((x1, y0));
        outline.push((x0, y0));
        outline.push((x0, y1));
        outline.push((x1, y1));
        let e2 = 2 * err;
        if e2 <= dy {
            y0 += 1;
            y1 -= 1;
            dy += aa;
            err += dy;
        }
        if e2 >= dx || 2 * err > dy {
            x0 += 1;
            x1 -= 1;
            dx += b1;
            err += dx;
        }
        if x0 > x1 {
            break;
        }
    }
    while y0 - y1 <= bb {
        outline.push((x0 - 1, y0));
        outline.push((x1 + 1, y0));
        y0 += 1;
        outline.push((x0 - 1, y1));
        outline.push((x1 + 1, y1));
        y1 -= 1;
    }
    let mut seen = HashSet::new();
    outline.retain(|p| seen.insert(*p));
    if !filled {
        return outline;
    }
    // Relleno: cada fila del mínimo al máximo x del contorno.
    let mut rows: std::collections::BTreeMap<i64, (i64, i64)> = std::collections::BTreeMap::new();
    for (x, y) in &outline {
        let e = rows.entry(*y).or_insert((*x, *x));
        e.0 = e.0.min(*x);
        e.1 = e.1.max(*x);
    }
    rows.into_iter()
        .flat_map(|(y, (x0, x1))| (x0..=x1).map(move |x| (x, y)))
        .collect()
}

/// Aplica el tamaño del pincel y la simetría, recorta al lienzo y quita
/// repetidos (conservando el orden).
fn expand(
    points: &[(i64, i64)],
    size: u32,
    canvas: &Canvas,
    options: &ToolOptions,
) -> Vec<(u32, u32)> {
    let (w, h) = (i64::from(canvas.width()), i64::from(canvas.height()));
    let size = i64::from(size);
    let offset = (size - 1) / 2;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let mut push = |x: i64, y: i64| {
        if x >= 0 && y >= 0 && x < w && y < h && seen.insert((x, y)) {
            out.push((x as u32, y as u32));
        }
    };
    for &(px, py) in points {
        for dy in 0..size {
            for dx in 0..size {
                let (x, y) = (px - offset + dx, py - offset + dy);
                push(x, y);
                if options.mirror_x {
                    push(w - 1 - x, y);
                }
                if options.mirror_y {
                    push(x, h - 1 - y);
                }
                if options.mirror_x && options.mirror_y {
                    push(w - 1 - x, h - 1 - y);
                }
            }
        }
    }
    out
}

/// Convierte celdas en un comando compacto: filas seguidas como `fill_rect`
/// de alto 1 y celdas sueltas como `set_pixel`. Un solo comando o un `Batch`.
fn cells_to_command(cells: &[(u32, u32)], color: Rgba) -> Option<Command> {
    if cells.is_empty() {
        return None;
    }
    let mut sorted: Vec<(u32, u32)> = cells.to_vec();
    sorted.sort_by_key(|(x, y)| (*y, *x));
    let mut commands = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let (x0, y) = sorted[i];
        let mut x1 = x0;
        while i + 1 < sorted.len() && sorted[i + 1] == (x1 + 1, y) {
            x1 += 1;
            i += 1;
        }
        commands.push(if x1 == x0 {
            Command::SetPixel {
                x: i64::from(x0),
                y: i64::from(y),
                color,
            }
        } else {
            Command::FillRect {
                x: i64::from(x0),
                y: i64::from(y),
                w: i64::from(x1 - x0 + 1),
                h: 1,
                color,
            }
        });
        i += 1;
    }
    Some(if commands.len() == 1 {
        commands.pop().expect("hay uno")
    } else {
        Command::Batch(commands)
    })
}

/// Relleno por inundación (4 vecinos) desde (`x`, `y`) con `color`: la región
/// conexa del mismo color exacto. `None` si el punto está fuera o ya es de ese
/// color. Devuelve el comando y cuántas celdas cambia.
pub fn flood_fill(canvas: &Canvas, x: i64, y: i64, color: Rgba) -> Option<(Command, usize)> {
    let target = canvas.get(x, y).ok()?;
    if target == color {
        return None;
    }
    let (w, h) = (canvas.width() as usize, canvas.height() as usize);
    let pixels = canvas.pixels();
    let mut visited = vec![false; w * h];
    let mut stack = vec![(x as usize, y as usize)];
    let mut cells = Vec::new();
    while let Some((cx, cy)) = stack.pop() {
        let i = cy * w + cx;
        if visited[i] || pixels[i] != target {
            continue;
        }
        visited[i] = true;
        cells.push((cx as u32, cy as u32));
        if cx > 0 {
            stack.push((cx - 1, cy));
        }
        if cx + 1 < w {
            stack.push((cx + 1, cy));
        }
        if cy > 0 {
            stack.push((cx, cy - 1));
        }
        if cy + 1 < h {
            stack.push((cx, cy + 1));
        }
    }
    let count = cells.len();
    cells_to_command(&cells, color).map(|c| (c, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_domain::{Document, Origin};

    const RED: Rgba = Rgba::rgb(255, 0, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    /// Pasa una secuencia de eventos y devuelve el último resultado.
    fn run(
        tool: DrawTool,
        canvas: &Canvas,
        options: ToolOptions,
        events: &[PointerEvent],
    ) -> ToolResult {
        let mut m = ToolMachine::new(tool);
        let mut last = ToolResult::Nothing;
        for e in events {
            last = m.handle(*e, canvas, RED, BLUE, &options);
        }
        last
    }

    fn down(x: i64, y: i64) -> PointerEvent {
        PointerEvent::Down {
            x,
            y,
            button: Button::Primary,
        }
    }
    fn mv(x: i64, y: i64) -> PointerEvent {
        PointerEvent::Move { x, y }
    }
    fn up(x: i64, y: i64) -> PointerEvent {
        PointerEvent::Up { x, y }
    }

    /// Celdas que pinta un comando sobre un lienzo vacío.
    fn painted(command: &Command, w: i64, h: i64) -> Vec<(i64, i64, Rgba)> {
        let mut c = Canvas::new(w, h).unwrap();
        command.apply(&mut c).unwrap();
        let mut out = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let p = c.get(x, y).unwrap();
                if p != Rgba::TRANSPARENT {
                    out.push((x, y, p));
                }
            }
        }
        out
    }

    fn commit(result: ToolResult) -> (Command, String) {
        match result {
            ToolResult::Commit { command, label } => (command, label),
            other => panic!("se esperaba un comando: {other:?}"),
        }
    }

    #[test]
    fn pencil_stroke_fills_gaps_and_commits_once() {
        let canvas = Canvas::new(8, 8).unwrap();
        let (cmd, label) = commit(run(
            DrawTool::Pencil,
            &canvas,
            ToolOptions::default(),
            &[down(1, 1), mv(4, 1), up(4, 1)],
        ));
        assert_eq!(
            cmd,
            Command::FillRect {
                x: 1,
                y: 1,
                w: 4,
                h: 1,
                color: RED
            }
        );
        assert_eq!(label, "Lápiz · 4 píxeles");
        // Movimiento en diagonal: sin huecos.
        let (cmd, _) = commit(run(
            DrawTool::Pencil,
            &canvas,
            ToolOptions::default(),
            &[down(0, 0), mv(3, 3), up(3, 3)],
        ));
        let cells: Vec<_> = painted(&cmd, 8, 8)
            .into_iter()
            .map(|(x, y, _)| (x, y))
            .collect();
        assert_eq!(cells, vec![(0, 0), (1, 1), (2, 2), (3, 3)]);
        // Un clic sin mover pinta un píxel.
        let (cmd, label) = commit(run(
            DrawTool::Pencil,
            &canvas,
            ToolOptions::default(),
            &[down(5, 6), up(5, 6)],
        ));
        assert_eq!(
            cmd,
            Command::SetPixel {
                x: 5,
                y: 6,
                color: RED
            }
        );
        assert_eq!(label, "Lápiz · 1 píxel");
    }

    #[test]
    fn secondary_button_and_eraser_colors() {
        let canvas = Canvas::new(4, 4).unwrap();
        let mut m = ToolMachine::new(DrawTool::Pencil);
        let o = ToolOptions::default();
        m.handle(
            PointerEvent::Down {
                x: 0,
                y: 0,
                button: Button::Secondary,
            },
            &canvas,
            RED,
            BLUE,
            &o,
        );
        let (cmd, _) = commit(m.handle(up(0, 0), &canvas, RED, BLUE, &o));
        assert_eq!(
            cmd,
            Command::SetPixel {
                x: 0,
                y: 0,
                color: BLUE
            }
        );
        let (cmd, label) = commit(run(
            DrawTool::Eraser,
            &canvas,
            o,
            &[down(1, 1), mv(2, 1), up(2, 1)],
        ));
        assert_eq!(
            cmd,
            Command::FillRect {
                x: 1,
                y: 1,
                w: 2,
                h: 1,
                color: Rgba::TRANSPARENT
            }
        );
        assert!(label.starts_with("Borrador"));
    }

    #[test]
    fn pixel_perfect_removes_l_corners() {
        assert_eq!(
            pixel_perfect(&[(0, 0), (1, 0), (1, 1)]),
            vec![(0, 0), (1, 1)]
        );
        assert_eq!(
            pixel_perfect(&[(0, 0), (1, 0), (2, 0)]),
            vec![(0, 0), (1, 0), (2, 0)]
        );
        // Un trazo en escalera con el ratón: sin esquinas dobles.
        let canvas = Canvas::new(8, 8).unwrap();
        let events = [down(0, 0), mv(1, 0), mv(1, 1), mv(2, 1), mv(2, 2), up(2, 2)];
        let with = commit(run(
            DrawTool::Pencil,
            &canvas,
            ToolOptions::default(),
            &events,
        ))
        .0;
        let without = commit(run(
            DrawTool::Pencil,
            &canvas,
            ToolOptions {
                pixel_perfect: false,
                ..Default::default()
            },
            &events,
        ))
        .0;
        assert_eq!(painted(&with, 8, 8).len(), 3);
        assert_eq!(painted(&without, 8, 8).len(), 5);
    }

    #[test]
    fn brush_size_symmetry_and_clipping() {
        let canvas = Canvas::new(8, 6).unwrap();
        let o = ToolOptions {
            size: 2,
            ..Default::default()
        };
        let (cmd, _) = commit(run(DrawTool::Pencil, &canvas, o, &[down(3, 3), up(3, 3)]));
        assert_eq!(painted(&cmd, 8, 6).len(), 4);
        // Simetría X y Y: cuatro copias.
        let o = ToolOptions {
            mirror_x: true,
            mirror_y: true,
            ..Default::default()
        };
        let (cmd, _) = commit(run(DrawTool::Pencil, &canvas, o, &[down(1, 1), up(1, 1)]));
        let cells: Vec<_> = painted(&cmd, 8, 6)
            .into_iter()
            .map(|(x, y, _)| (x, y))
            .collect();
        assert_eq!(cells, vec![(1, 1), (6, 1), (1, 4), (6, 4)]);
        // Un trazo que sale del lienzo se recorta (el Batch no puede fallar).
        let (cmd, _) = commit(run(
            DrawTool::Pencil,
            &canvas,
            ToolOptions::default(),
            &[down(-3, 2), mv(10, 2), up(10, 2)],
        ));
        assert_eq!(
            cmd,
            Command::FillRect {
                x: 0,
                y: 2,
                w: 8,
                h: 1,
                color: RED
            }
        );
        // Todo fuera: nada.
        assert_eq!(
            run(
                DrawTool::Pencil,
                &canvas,
                ToolOptions::default(),
                &[down(-5, -5), up(-6, -6)]
            ),
            ToolResult::Nothing
        );
    }

    #[test]
    fn line_rect_and_ellipse_preview_then_commit() {
        let canvas = Canvas::new(10, 10).unwrap();
        let o = ToolOptions::default();
        let mut m = ToolMachine::new(DrawTool::Line);
        m.handle(down(0, 0), &canvas, RED, BLUE, &o);
        m.handle(mv(5, 2), &canvas, RED, BLUE, &o);
        let (preview, color) = m.preview(&canvas, &o).unwrap();
        assert_eq!(color, RED);
        let expected: Vec<(u32, u32)> = bresenham(0, 0, 5, 2)
            .into_iter()
            .map(|(x, y)| (x as u32, y as u32))
            .collect();
        assert_eq!(preview, expected);
        let (cmd, _) = commit(m.handle(up(5, 2), &canvas, RED, BLUE, &o));
        let mut a = Canvas::new(10, 10).unwrap();
        cmd.apply(&mut a).unwrap();
        let mut b = Canvas::new(10, 10).unwrap();
        b.line(0, 0, 5, 2, RED).unwrap();
        assert_eq!(a, b);
        assert!(!m.is_active());

        let (cmd, _) = commit(run(
            DrawTool::Rect,
            &canvas,
            o,
            &[down(6, 5), mv(2, 1), up(2, 1)],
        ));
        assert_eq!(painted(&cmd, 10, 10).len(), 16, "contorno de 5×5");
        let filled = ToolOptions { filled: true, ..o };
        let (cmd, _) = commit(run(
            DrawTool::Rect,
            &canvas,
            filled,
            &[down(2, 1), up(6, 5)],
        ));
        assert_eq!(painted(&cmd, 10, 10).len(), 25);

        let (cmd, _) = commit(run(DrawTool::Ellipse, &canvas, o, &[down(0, 0), up(8, 6)]));
        let outline: HashSet<(i64, i64)> = painted(&cmd, 10, 10)
            .into_iter()
            .map(|(x, y, _)| (x, y))
            .collect();
        // Toca los cuatro lados del rectángulo y es simétrica.
        assert!(outline.contains(&(4, 0)) && outline.contains(&(4, 6)));
        assert!(outline.contains(&(0, 3)) && outline.contains(&(8, 3)));
        for &(x, y) in &outline {
            assert!(
                outline.contains(&(8 - x, y)) && outline.contains(&(x, 6 - y)),
                "({x},{y})"
            );
        }
        assert!(!outline.contains(&(4, 3)), "el contorno no tiene centro");
        let (cmd, _) = commit(run(
            DrawTool::Ellipse,
            &canvas,
            filled,
            &[down(0, 0), up(8, 6)],
        ));
        let full: HashSet<(i64, i64)> = painted(&cmd, 10, 10)
            .into_iter()
            .map(|(x, y, _)| (x, y))
            .collect();
        assert!(full.contains(&(4, 3)) && full.is_superset(&outline));
    }

    #[test]
    fn flood_fill_stays_in_its_region_and_inside_the_canvas() {
        // Pared en diagonal de (3,0) a (0,3): el triángulo de arriba a la
        // izquierda solo toca el resto en diagonal, que 4 vecinos no cruza.
        let mut canvas = Canvas::new(6, 4).unwrap();
        for i in 0..4 {
            canvas.set(3 - i, i, BLUE).unwrap();
        }
        let (cmd, count) = flood_fill(&canvas, 0, 0, RED).unwrap();
        assert_eq!(count, 6);
        let mut after = canvas.clone();
        cmd.apply(&mut after).unwrap();
        for i in 0..4 {
            assert_eq!(after.get(3 - i, i).unwrap(), BLUE, "la pared no cambia");
        }
        for (x, y) in [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (0, 2)] {
            assert_eq!(after.get(x, y).unwrap(), RED, "({x},{y})");
        }
        for (x, y) in [(2, 2), (5, 3), (4, 0), (1, 3)] {
            assert_eq!(
                after.get(x, y).unwrap(),
                Rgba::TRANSPARENT,
                "no cruza ({x},{y})"
            );
        }
        // Mismo color: nada que hacer. Fuera del lienzo: nada.
        assert!(flood_fill(&after, 0, 0, RED).is_none());
        assert!(flood_fill(&after, 9, 9, RED).is_none());
        // Un lienzo vacío se rellena entero, sin salirse.
        let (cmd, count) = flood_fill(&Canvas::new(5, 3).unwrap(), 2, 1, RED).unwrap();
        assert_eq!(count, 15);
        assert_eq!(
            cmd,
            Command::Batch(vec![
                Command::FillRect {
                    x: 0,
                    y: 0,
                    w: 5,
                    h: 1,
                    color: RED
                },
                Command::FillRect {
                    x: 0,
                    y: 1,
                    w: 5,
                    h: 1,
                    color: RED
                },
                Command::FillRect {
                    x: 0,
                    y: 2,
                    w: 5,
                    h: 1,
                    color: RED
                },
            ])
        );
        let (_, label) = commit(run(
            DrawTool::Fill,
            &Canvas::new(5, 3).unwrap(),
            ToolOptions::default(),
            // El relleno se aplica al pulsar.
            &[down(1, 1)],
        ));
        assert_eq!(label, "Relleno · 15 píxeles");
    }

    #[test]
    fn eyedropper_picks_colors() {
        let mut canvas = Canvas::new(4, 4).unwrap();
        canvas.set(2, 2, Rgba::rgb(9, 8, 7)).unwrap();
        let o = ToolOptions::default();
        let mut m = ToolMachine::new(DrawTool::Eyedropper);
        assert_eq!(
            m.handle(
                PointerEvent::Down {
                    x: 2,
                    y: 2,
                    button: Button::Secondary
                },
                &canvas,
                RED,
                BLUE,
                &o
            ),
            ToolResult::PickColor {
                color: Rgba::rgb(9, 8, 7),
                button: Button::Secondary
            }
        );
        assert_eq!(
            m.handle(down(-1, 0), &canvas, RED, BLUE, &o),
            ToolResult::Nothing
        );
    }

    #[test]
    fn a_whole_pencil_stroke_undoes_in_one_step() {
        let mut doc = Document::new("t", 16, 16).unwrap();
        doc.apply(
            Command::SetPixel {
                x: 15,
                y: 15,
                color: BLUE,
            },
            Origin::User,
        )
        .unwrap();
        let before = doc.canvas().clone();
        let events = [down(1, 1), mv(6, 3), mv(10, 9), mv(3, 12), up(2, 14)];
        let (cmd, label) = commit(run(
            DrawTool::Pencil,
            doc.canvas(),
            ToolOptions {
                size: 2,
                mirror_x: true,
                ..Default::default()
            },
            &events,
        ));
        doc.begin(Origin::User, label).unwrap();
        doc.apply(cmd, Origin::User).unwrap();
        doc.commit();
        assert_ne!(doc.canvas(), &before);
        assert_eq!(doc.history().entries().len(), 2);
        doc.undo().unwrap();
        assert_eq!(doc.canvas(), &before);
    }

    #[test]
    fn cancel_drops_the_stroke() {
        let canvas = Canvas::new(4, 4).unwrap();
        let o = ToolOptions::default();
        let mut m = ToolMachine::new(DrawTool::Rect);
        m.handle(down(0, 0), &canvas, RED, BLUE, &o);
        assert!(m.is_active());
        m.cancel();
        assert!(!m.is_active());
        assert_eq!(
            m.handle(up(3, 3), &canvas, RED, BLUE, &o),
            ToolResult::Nothing
        );
    }
}
