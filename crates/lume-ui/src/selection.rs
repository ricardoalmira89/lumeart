//! Selección rectangular (M) y mover (V), como máquina de estados sin egui.
//! Mover es un único `Command` (borrar el origen y pintar en el destino), así
//! que se deshace en un paso.

use lume_domain::{Canvas, Command, Rgba};

use crate::tools::PointerEvent;

/// Rectángulo seleccionado, en píxeles del lienzo (siempre dentro de él).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selection {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Selection {
    /// Rectángulo de `a` a `b` (ambos incluidos) recortado al lienzo; `None`
    /// si queda fuera.
    pub fn between(a: (i64, i64), b: (i64, i64), canvas: &Canvas) -> Option<Selection> {
        let (x0, x1) = (
            a.0.min(b.0).max(0),
            a.0.max(b.0).min(i64::from(canvas.width()) - 1),
        );
        let (y0, y1) = (
            a.1.min(b.1).max(0),
            a.1.max(b.1).min(i64::from(canvas.height()) - 1),
        );
        (x0 <= x1 && y0 <= y1).then(|| Selection {
            x: x0 as u32,
            y: y0 as u32,
            w: (x1 - x0 + 1) as u32,
            h: (y1 - y0 + 1) as u32,
        })
    }

    /// Todo el lienzo.
    pub fn all(canvas: &Canvas) -> Selection {
        Selection {
            x: 0,
            y: 0,
            w: canvas.width(),
            h: canvas.height(),
        }
    }

    pub fn contains(&self, x: i64, y: i64) -> bool {
        x >= i64::from(self.x)
            && y >= i64::from(self.y)
            && x < i64::from(self.x + self.w)
            && y < i64::from(self.y + self.h)
    }

    /// La selección desplazada (`dx`, `dy`) y recortada al lienzo.
    pub fn offset(&self, dx: i64, dy: i64, canvas: &Canvas) -> Option<Selection> {
        let (x, y) = (i64::from(self.x) + dx, i64::from(self.y) + dy);
        Selection::between(
            (x, y),
            (x + i64::from(self.w) - 1, y + i64::from(self.h) - 1),
            canvas,
        )
    }
}

/// Píxeles visibles (opacidad > 0) de una selección: (`x`, `y`, color).
pub fn selected_pixels(canvas: &Canvas, sel: &Selection) -> Vec<(i64, i64, Rgba)> {
    let mut out = Vec::new();
    for y in sel.y..sel.y + sel.h {
        for x in sel.x..sel.x + sel.w {
            let c = canvas.pixels()[(y * canvas.width() + x) as usize];
            if c.a > 0 {
                out.push((i64::from(x), i64::from(y), c));
            }
        }
    }
    out
}

/// Comandos compactos para pintar píxeles de varios colores: tramos seguidos
/// del mismo color en una fila → `fill_rect` de alto 1; sueltos →
/// `set_pixel`. Recorta al lienzo.
pub fn paint_commands(pixels: &[(i64, i64, Rgba)], canvas: &Canvas) -> Vec<Command> {
    let (w, h) = (i64::from(canvas.width()), i64::from(canvas.height()));
    let mut cells: Vec<(i64, i64, Rgba)> = pixels
        .iter()
        .copied()
        .filter(|(x, y, _)| *x >= 0 && *y >= 0 && *x < w && *y < h)
        .collect();
    cells.sort_by_key(|(x, y, _)| (*y, *x));
    let mut out = Vec::new();
    let mut i = 0;
    while i < cells.len() {
        let (x0, y, color) = cells[i];
        let mut x1 = x0;
        while i + 1 < cells.len() && cells[i + 1] == (x1 + 1, y, color) {
            x1 += 1;
            i += 1;
        }
        out.push(if x1 == x0 {
            Command::SetPixel { x: x0, y, color }
        } else {
            Command::FillRect {
                x: x0,
                y,
                w: x1 - x0 + 1,
                h: 1,
                color,
            }
        });
        i += 1;
    }
    out
}

/// Comando que mueve el contenido de `sel` (`dx`, `dy`): borra el origen y
/// pinta sus píxeles visibles en el destino (lo que sale del lienzo se
/// pierde). `None` si no hay nada que mover o el desplazamiento es nulo.
pub fn move_command(
    canvas: &Canvas,
    sel: &Selection,
    dx: i64,
    dy: i64,
) -> Option<(Command, usize)> {
    if dx == 0 && dy == 0 {
        return None;
    }
    let pixels = selected_pixels(canvas, sel);
    if pixels.is_empty() {
        return None;
    }
    let moved: Vec<(i64, i64, Rgba)> = pixels
        .iter()
        .map(|(x, y, c)| (x + dx, y + dy, *c))
        .collect();
    let mut commands = vec![Command::FillRect {
        x: i64::from(sel.x),
        y: i64::from(sel.y),
        w: i64::from(sel.w),
        h: i64::from(sel.h),
        color: Rgba::TRANSPARENT,
    }];
    commands.extend(paint_commands(&moved, canvas));
    Some((Command::Batch(commands), pixels.len()))
}

/// Estado de la selección y de mover.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum State {
    #[default]
    Idle,
    Selecting {
        start: (i64, i64),
        end: (i64, i64),
    },
    Moving {
        sel: Selection,
        start: (i64, i64),
        now: (i64, i64),
    },
}

/// Máquina de las herramientas Selección y Mover.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EditMachine {
    state: State,
}

/// Lo que devuelve mover al soltar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveResult {
    pub command: Command,
    pub label: String,
}

impl EditMachine {
    pub fn is_active(&self) -> bool {
        self.state != State::Idle
    }

    /// Cancela el arrastre en curso (Esc).
    pub fn cancel(&mut self) {
        self.state = State::Idle;
    }

    /// Selección (M): arrastrar crea una selección; un clic sin arrastrar la
    /// quita.
    pub fn select(
        &mut self,
        event: PointerEvent,
        canvas: &Canvas,
        selection: &mut Option<Selection>,
    ) {
        match event {
            PointerEvent::Down { x, y, .. } => {
                self.state = State::Selecting {
                    start: (x, y),
                    end: (x, y),
                };
            }
            PointerEvent::Move { x, y } => {
                if let State::Selecting { end, .. } = &mut self.state {
                    *end = (x, y);
                }
            }
            PointerEvent::Up { x, y } => {
                if let State::Selecting { start, .. } = self.state {
                    *selection = if start == (x, y) {
                        None
                    } else {
                        Selection::between(start, (x, y), canvas)
                    };
                }
                self.state = State::Idle;
            }
        }
    }

    /// Mover (V): arrastra la selección (o todo el lienzo si no hay). Al
    /// soltar devuelve un único comando y deja la selección en su sitio nuevo.
    pub fn drag_move(
        &mut self,
        event: PointerEvent,
        canvas: &Canvas,
        selection: &mut Option<Selection>,
    ) -> Option<MoveResult> {
        match event {
            PointerEvent::Down { x, y, .. } => {
                let sel = selection.unwrap_or_else(|| Selection::all(canvas));
                self.state = State::Moving {
                    sel,
                    start: (x, y),
                    now: (x, y),
                };
                None
            }
            PointerEvent::Move { x, y } => {
                if let State::Moving { now, .. } = &mut self.state {
                    *now = (x, y);
                }
                None
            }
            PointerEvent::Up { x, y } => {
                let State::Moving { sel, start, .. } = self.state else {
                    return None;
                };
                self.state = State::Idle;
                let (dx, dy) = (x - start.0, y - start.1);
                let (command, count) = move_command(canvas, &sel, dx, dy)?;
                if selection.is_some() {
                    *selection = sel.offset(dx, dy, canvas);
                }
                Some(MoveResult {
                    command,
                    label: format!(
                        "Mover · {}",
                        if count == 1 {
                            "1 píxel".to_string()
                        } else {
                            format!("{count} píxeles")
                        }
                    ),
                })
            }
        }
    }

    /// Rectángulo que se está seleccionando (vista previa).
    pub fn selecting(&self, canvas: &Canvas) -> Option<Selection> {
        match self.state {
            State::Selecting { start, end } => Selection::between(start, end, canvas),
            _ => None,
        }
    }

    /// Lo que se está moviendo: la selección de origen y el desplazamiento.
    pub fn moving(&self) -> Option<(Selection, i64, i64)> {
        match self.state {
            State::Moving { sel, start, now } => Some((sel, now.0 - start.0, now.1 - start.1)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Button;
    use lume_domain::{Document, Origin};

    const RED: Rgba = Rgba::rgb(255, 0, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    fn down(x: i64, y: i64) -> PointerEvent {
        PointerEvent::Down {
            x,
            y,
            button: Button::Primary,
        }
    }

    #[test]
    fn selecting_a_rectangle() {
        let canvas = Canvas::new(8, 6).unwrap();
        let mut m = EditMachine::default();
        let mut sel = None;
        m.select(down(5, 4), &canvas, &mut sel);
        m.select(PointerEvent::Move { x: 2, y: 1 }, &canvas, &mut sel);
        assert_eq!(
            m.selecting(&canvas),
            Selection::between((5, 4), (2, 1), &canvas)
        );
        m.select(PointerEvent::Up { x: 2, y: 1 }, &canvas, &mut sel);
        assert_eq!(
            sel,
            Some(Selection {
                x: 2,
                y: 1,
                w: 4,
                h: 4
            })
        );
        assert!(!m.is_active());
        // Se recorta al lienzo.
        m.select(down(-3, -3), &canvas, &mut sel);
        m.select(PointerEvent::Up { x: 1, y: 20 }, &canvas, &mut sel);
        assert_eq!(
            sel,
            Some(Selection {
                x: 0,
                y: 0,
                w: 2,
                h: 6
            })
        );
        // Un clic sin arrastrar quita la selección.
        m.select(down(3, 3), &canvas, &mut sel);
        m.select(PointerEvent::Up { x: 3, y: 3 }, &canvas, &mut sel);
        assert_eq!(sel, None);
        // Del todo fuera: nada.
        assert_eq!(Selection::between((20, 20), (30, 30), &canvas), None);
    }

    #[test]
    fn moving_a_selection_is_one_reversible_command() {
        let mut doc = Document::new("t", 8, 8).unwrap();
        doc.apply(
            Command::FillRect {
                x: 1,
                y: 1,
                w: 2,
                h: 2,
                color: RED,
            },
            Origin::User,
        )
        .unwrap();
        doc.apply(
            Command::SetPixel {
                x: 6,
                y: 6,
                color: BLUE,
            },
            Origin::User,
        )
        .unwrap();
        let before = doc.canvas().clone();
        let mut sel = Some(Selection {
            x: 0,
            y: 0,
            w: 4,
            h: 4,
        });
        let mut m = EditMachine::default();
        m.drag_move(down(1, 1), doc.canvas(), &mut sel);
        m.drag_move(PointerEvent::Move { x: 3, y: 2 }, doc.canvas(), &mut sel);
        assert_eq!(
            m.moving(),
            Some((
                Selection {
                    x: 0,
                    y: 0,
                    w: 4,
                    h: 4
                },
                2,
                1
            ))
        );
        let result = m
            .drag_move(PointerEvent::Up { x: 4, y: 3 }, doc.canvas(), &mut sel)
            .unwrap();
        assert_eq!(result.label, "Mover · 4 píxeles");
        assert!(matches!(result.command, Command::Batch(_)));

        doc.begin(Origin::User, result.label).unwrap();
        doc.apply(result.command, Origin::User).unwrap();
        doc.commit();
        // El rojo está ahora en (4..6, 3..5) y el origen quedó vacío.
        for (x, y) in [(4, 3), (5, 3), (4, 4), (5, 4)] {
            assert_eq!(doc.canvas().get(x, y).unwrap(), RED, "({x},{y})");
        }
        for (x, y) in [(1, 1), (2, 2)] {
            assert_eq!(doc.canvas().get(x, y).unwrap(), Rgba::TRANSPARENT);
        }
        assert_eq!(
            doc.canvas().get(6, 6).unwrap(),
            BLUE,
            "lo de fuera de la selección no cambia"
        );
        assert_eq!(
            sel,
            Some(Selection {
                x: 3,
                y: 2,
                w: 4,
                h: 4
            }),
            "la selección se mueve"
        );
        // Un solo paso de deshacer.
        doc.undo().unwrap();
        assert_eq!(doc.canvas(), &before);
    }

    #[test]
    fn moving_without_selection_moves_everything_and_clips() {
        let mut canvas = Canvas::new(4, 4).unwrap();
        canvas.set(0, 0, RED).unwrap();
        canvas.set(3, 3, BLUE).unwrap();
        let mut sel = None;
        let mut m = EditMachine::default();
        m.drag_move(down(2, 2), &canvas, &mut sel);
        let result = m
            .drag_move(PointerEvent::Up { x: 3, y: 2 }, &canvas, &mut sel)
            .unwrap();
        let mut after = canvas.clone();
        result.command.apply(&mut after).unwrap();
        assert_eq!(after.get(1, 0).unwrap(), RED);
        assert_eq!(after.get(0, 0).unwrap(), Rgba::TRANSPARENT);
        // El azul sale por la derecha y se pierde.
        assert!(after.pixels().iter().all(|p| *p != BLUE));
        assert_eq!(sel, None, "sin selección sigue sin selección");
        // Sin desplazamiento o sin nada que mover: ningún comando.
        let mut m = EditMachine::default();
        m.drag_move(down(1, 1), &canvas, &mut sel);
        assert!(
            m.drag_move(PointerEvent::Up { x: 1, y: 1 }, &canvas, &mut sel)
                .is_none()
        );
        let empty = Canvas::new(4, 4).unwrap();
        m.drag_move(down(1, 1), &empty, &mut sel);
        assert!(
            m.drag_move(PointerEvent::Up { x: 2, y: 2 }, &empty, &mut sel)
                .is_none()
        );
    }

    #[test]
    fn overlapping_move_keeps_the_pixels() {
        // Mover 1 a la derecha una fila de 3: el destino se pisa con el origen.
        let mut canvas = Canvas::new(6, 1).unwrap();
        for (x, c) in [(1, RED), (2, BLUE), (3, RED)] {
            canvas.set(x, 0, c).unwrap();
        }
        let sel = Selection {
            x: 1,
            y: 0,
            w: 3,
            h: 1,
        };
        let (cmd, count) = move_command(&canvas, &sel, 1, 0).unwrap();
        assert_eq!(count, 3);
        cmd.apply(&mut canvas).unwrap();
        let row: Vec<Rgba> = (0..6).map(|x| canvas.get(x, 0).unwrap()).collect();
        assert_eq!(
            row,
            vec![
                Rgba::TRANSPARENT,
                Rgba::TRANSPARENT,
                RED,
                BLUE,
                RED,
                Rgba::TRANSPARENT
            ]
        );
    }

    #[test]
    fn escape_cancels_the_drag() {
        let canvas = Canvas::new(4, 4).unwrap();
        let mut sel = Some(Selection {
            x: 0,
            y: 0,
            w: 2,
            h: 2,
        });
        let mut m = EditMachine::default();
        m.select(down(0, 0), &canvas, &mut sel);
        m.cancel();
        m.select(PointerEvent::Up { x: 3, y: 3 }, &canvas, &mut sel);
        assert_eq!(
            sel,
            Some(Selection {
                x: 0,
                y: 0,
                w: 2,
                h: 2
            }),
            "la selección anterior sigue"
        );
        m.drag_move(down(0, 0), &canvas, &mut sel);
        m.cancel();
        assert!(
            m.drag_move(PointerEvent::Up { x: 2, y: 2 }, &canvas, &mut sel)
                .is_none()
        );
    }

    #[test]
    fn paint_commands_group_runs_of_the_same_color() {
        let canvas = Canvas::new(8, 2).unwrap();
        let cmds = paint_commands(
            &[
                (0, 0, RED),
                (1, 0, RED),
                (2, 0, BLUE),
                (5, 1, RED),
                (9, 1, RED),
            ],
            &canvas,
        );
        assert_eq!(
            cmds,
            vec![
                Command::FillRect {
                    x: 0,
                    y: 0,
                    w: 2,
                    h: 1,
                    color: RED
                },
                Command::SetPixel {
                    x: 2,
                    y: 0,
                    color: BLUE
                },
                Command::SetPixel {
                    x: 5,
                    y: 1,
                    color: RED
                },
            ]
        );
    }
}
