//! Repetir un cambio paso a paso, como un pincel: qué celdas pinta cada
//! comando y en qué orden (un rectángulo fila a fila, una línea punto a
//! punto). La ventana lo usa para ver dibujar al agente como en el studio web
//! de pxcli, en vez de que aparezca todo de golpe.

use crate::batch::normalize_batch_line;
use crate::canvas::Canvas;
use crate::color::Rgba;
use crate::history::{History, Origin};
use crate::patch::{Patch, PatchStep};
use crate::protocol::handle_line;

/// Lo que pinta un paso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepPaint {
    /// Celdas en el orden en que se pintan, con su color final.
    Cells(Vec<(u32, u32, Rgba)>),
    /// Cambia el lienzo entero de golpe (`clear`, `new` o un cambio de
    /// tamaño): cómo queda tras el paso.
    Frame(Canvas),
}

/// Un comando de un cambio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawStep {
    /// La línea tal como llegó.
    pub line: String,
    pub paint: StepPaint,
}

impl DrawStep {
    /// Celdas que pinta (0 en un paso de golpe o que no cambia nada).
    pub fn cell_count(&self) -> usize {
        match &self.paint {
            StepPaint::Cells(cells) => cells.len(),
            StepPaint::Frame(_) => 0,
        }
    }
}

/// Los pasos de `lines` aplicados uno a uno sobre `before`. Una línea que no
/// vale o que no cambia nada da un paso sin celdas (sigue contando para
/// saber por qué línea va el dibujo).
pub fn draw_steps(before: &Canvas, lines: &[String]) -> Vec<DrawStep> {
    let mut work = before.clone();
    let keep = |path: &str| path.to_string();
    lines
        .iter()
        .map(|line| {
            let paint = match normalize_batch_line(line, &keep) {
                Ok(request) => step(&mut work, &request),
                Err(_) => StepPaint::Cells(Vec::new()),
            };
            DrawStep {
                line: line.clone(),
                paint,
            }
        })
        .collect()
}

/// Aplica una línea a `work` y dice qué pintó.
fn step(work: &mut Canvas, request: &str) -> StepPaint {
    // Un historial por línea: su única entrada es el patch de este comando,
    // con las celdas en el orden en que se pintaron.
    let mut history = History::new();
    let _ = handle_line(request, work, &mut history, Origin::Agent { run: 0 });
    // `new` empieza un historial nuevo: no deja entrada.
    if matches!(request.split_whitespace().next(), Some("clear" | "new")) {
        return StepPaint::Frame(work.clone());
    }
    match history.entries().last() {
        None => StepPaint::Cells(Vec::new()),
        Some(entry) if has_resize(&entry.patch) => StepPaint::Frame(work.clone()),
        Some(entry) => StepPaint::Cells(entry.patch.cells().map(|c| (c.x, c.y, c.after)).collect()),
    }
}

fn has_resize(patch: &Patch) -> bool {
    patch
        .steps()
        .iter()
        .any(|s| matches!(s, PatchStep::Resize(_)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &[&str]) -> Vec<String> {
        text.iter().map(|s| s.to_string()).collect()
    }

    fn cells(step: &DrawStep) -> Vec<(u32, u32)> {
        match &step.paint {
            StepPaint::Cells(c) => c.iter().map(|(x, y, _)| (*x, *y)).collect(),
            StepPaint::Frame(_) => panic!("{} es de golpe", step.line),
        }
    }

    #[test]
    fn rectangles_go_row_by_row_and_lines_point_by_point() {
        let before = Canvas::new(8, 8).unwrap();
        let steps = draw_steps(
            &before,
            &lines(&[
                "fill_rect 1 1 3 2 #ff0000",
                "line 5 5 2 2 #00ff00",
                "set_pixel 7 0 #0000ff",
            ]),
        );
        assert_eq!(steps.len(), 3);
        assert_eq!(
            cells(&steps[0]),
            [(1, 1), (2, 1), (3, 1), (1, 2), (2, 2), (3, 2)]
        );
        // Desde el primer punto de la línea hasta el último.
        assert_eq!(cells(&steps[1]), [(5, 5), (4, 4), (3, 3), (2, 2)]);
        let StepPaint::Cells(blue) = &steps[2].paint else {
            panic!()
        };
        assert_eq!(blue, &[(7, 0, Rgba::rgb(0, 0, 255))]);
    }

    /// Repetir los pasos sobre el lienzo de antes da el mismo dibujo que el
    /// cambio de verdad.
    #[test]
    fn replaying_the_steps_gives_the_same_canvas() {
        let mut doc = crate::Document::new("prueba", 16, 16).unwrap();
        let _ = doc.handle_line("fill_rect 0 0 16 16 #202020", Origin::Cli);
        let before = doc.canvas().clone();
        let script = lines(&[
            "fill_rect 2 2 5 5 #ff0000",
            "lumeart line 0 15 15 0 #00ff00",
            "set_pixel 3 3 #0000ff",
            "esto no es un comando",
            "get_pixel 3 3",
            "set_pixel 0 0 #202020",
            "fill_rect 2 2 5 5 #ff0000",
        ]);
        let mut truth = before.clone();
        let mut history = History::new();
        for line in &script {
            let request = normalize_batch_line(line, &|p| p.to_string()).unwrap_or_default();
            let _ = handle_line(&request, &mut truth, &mut history, Origin::Cli);
        }

        let steps = draw_steps(&before, &script);
        assert_eq!(steps.len(), script.len());
        assert_eq!(steps[3].cell_count(), 0, "una línea que no vale");
        assert_eq!(steps[4].cell_count(), 0, "una consulta");
        assert_eq!(steps[5].cell_count(), 0, "pintar el color que ya hay");
        assert_eq!(
            steps[6].cell_count(),
            1,
            "solo vuelve a pintar la celda azul"
        );
        let mut shown = before.clone();
        for step in &steps {
            match &step.paint {
                StepPaint::Cells(cells) => {
                    for (x, y, c) in cells {
                        shown.put(*x, *y, *c);
                    }
                }
                StepPaint::Frame(canvas) => shown = canvas.clone(),
            }
        }
        assert_eq!(shown, truth);
    }

    #[test]
    fn clear_and_resizes_are_whole_frames() {
        let before = Canvas::new(4, 4).unwrap();
        let steps = draw_steps(
            &before,
            &lines(&["clear #112233", "new 6 2", "set_pixel 5 1 #ffffff"]),
        );
        let StepPaint::Frame(cleared) = &steps[0].paint else {
            panic!("clear")
        };
        assert_eq!(cleared.get(3, 3).unwrap(), Rgba::rgb(0x11, 0x22, 0x33));
        let StepPaint::Frame(resized) = &steps[1].paint else {
            panic!("new")
        };
        assert_eq!((resized.width(), resized.height()), (6, 2));
        assert_eq!(cells(&steps[2]), [(5, 1)]);
    }
}
