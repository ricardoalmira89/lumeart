//! `Command`: cada cambio del documento es un comando que, al aplicarse,
//! devuelve su `Patch`. Lápiz, agente, CLI y receta usan los mismos comandos.

use crate::canvas::Canvas;
use crate::color::Rgba;
use crate::error::DomainError;
use crate::patch::Patch;

/// Un cambio del lienzo. Las coordenadas son `i64` (el `int` de pxcli) para
/// que las negativas den `out_of_bounds` con el mismo mensaje.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
    /// `set_pixel x y color`
    SetPixel { x: i64, y: i64, color: Rgba },
    /// `fill_rect x y w h color`
    FillRect {
        x: i64,
        y: i64,
        w: i64,
        h: i64,
        color: Rgba,
    },
    /// `line x1 y1 x2 y2 color` (Bresenham, extremos incluidos)
    Line {
        from: (i64, i64),
        to: (i64, i64),
        color: Rgba,
    },
    /// `clear [color]`; sin color, transparente.
    Clear { color: Option<Rgba> },
    /// Cambia el tamaño conservando el contenido (arriba a la izquierda).
    Resize { w: i64, h: i64 },
    /// Varios comandos como un solo cambio. Es atómico: si uno falla, se
    /// deshace lo aplicado y se devuelve su error. (El `batch` del protocolo,
    /// que aplica los válidos y avisa de los demás, va con transacciones del
    /// historial.)
    Batch(Vec<Command>),
}

impl Command {
    /// Nombre del comando en el protocolo (`batch` para un grupo).
    pub fn name(&self) -> &'static str {
        match self {
            Command::SetPixel { .. } => "set_pixel",
            Command::FillRect { .. } => "fill_rect",
            Command::Line { .. } => "line",
            Command::Clear { .. } => "clear",
            Command::Resize { .. } => "resize",
            Command::Batch(_) => "batch",
        }
    }

    /// Número de comandos simples que contiene (1 salvo en `Batch`).
    pub fn count(&self) -> usize {
        match self {
            Command::Batch(commands) => commands.iter().map(Command::count).sum(),
            _ => 1,
        }
    }

    /// Aplica el comando. Si falla, el lienzo queda como estaba.
    pub fn apply(&self, canvas: &mut Canvas) -> Result<Patch, DomainError> {
        match self {
            Command::SetPixel { x, y, color } => Ok(Patch::from_cells(canvas.set(*x, *y, *color)?)),
            Command::FillRect { x, y, w, h, color } => {
                Ok(Patch::from_cells(canvas.fill_rect(*x, *y, *w, *h, *color)?))
            }
            Command::Line { from, to, color } => Ok(Patch::from_cells(
                canvas.line(from.0, from.1, to.0, to.1, *color)?,
            )),
            Command::Clear { color } => Ok(Patch::from_cells(
                canvas.clear(color.unwrap_or(Rgba::TRANSPARENT)),
            )),
            Command::Resize { w, h } => Ok(canvas
                .resize(*w, *h)?
                .map(Patch::from_resize)
                .unwrap_or_default()),
            Command::Batch(commands) => {
                let mut patch = Patch::new();
                for command in commands {
                    match command.apply(canvas) {
                        Ok(p) => patch.extend(p),
                        Err(err) => {
                            patch
                                .revert(canvas)
                                .expect("un patch recién aplicado siempre se puede revertir");
                            return Err(err);
                        }
                    }
                }
                patch.compact();
                Ok(patch)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::CellChange;
    use crate::error::ErrorCode;
    use crate::patch::PatchStep;
    use proptest::prelude::*;

    const RED: Rgba = Rgba::rgb(255, 0, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    fn canvas(w: i64, h: i64) -> Canvas {
        Canvas::new(w, h).unwrap()
    }

    #[test]
    fn set_pixel() {
        let mut c = canvas(4, 4);
        let p = Command::SetPixel {
            x: 1,
            y: 2,
            color: RED,
        }
        .apply(&mut c)
        .unwrap();
        assert_eq!(c.get(1, 2).unwrap(), RED);
        assert_eq!(
            p.cells().copied().collect::<Vec<_>>(),
            vec![CellChange {
                x: 1,
                y: 2,
                before: Rgba::TRANSPARENT,
                after: RED
            }]
        );
    }

    #[test]
    fn fill_rect() {
        let mut c = canvas(4, 4);
        let cmd = Command::FillRect {
            x: 1,
            y: 1,
            w: 2,
            h: 3,
            color: BLUE,
        };
        let p = cmd.apply(&mut c).unwrap();
        assert_eq!(p.cell_count(), 6);
        let mut expected = canvas(4, 4);
        expected.fill_rect(1, 1, 2, 3, BLUE).unwrap();
        assert_eq!(c, expected);
    }

    #[test]
    fn line() {
        let mut c = canvas(4, 4);
        let p = Command::Line {
            from: (0, 0),
            to: (3, 3),
            color: RED,
        }
        .apply(&mut c)
        .unwrap();
        assert_eq!(p.cell_count(), 4);
        for i in 0..4 {
            assert_eq!(c.get(i, i).unwrap(), RED);
        }
    }

    #[test]
    fn clear_with_and_without_color() {
        let mut c = canvas(3, 3);
        let p = Command::Clear { color: Some(RED) }.apply(&mut c).unwrap();
        assert_eq!(p.cell_count(), 9);
        assert!(c.pixels().iter().all(|p| *p == RED));
        let p = Command::Clear { color: None }.apply(&mut c).unwrap();
        assert_eq!(p.cell_count(), 9);
        assert_eq!(c, canvas(3, 3));
    }

    #[test]
    fn resize() {
        let mut c = canvas(4, 4);
        c.set(3, 3, RED).unwrap();
        let p = Command::Resize { w: 2, h: 6 }.apply(&mut c).unwrap();
        assert_eq!((c.width(), c.height()), (2, 6));
        assert!(matches!(p.steps(), [PatchStep::Resize(_)]));
        p.revert(&mut c).unwrap();
        assert_eq!((c.width(), c.height()), (4, 4));
        assert_eq!(c.get(3, 3).unwrap(), RED);
    }

    #[test]
    fn batch_groups_and_compacts() {
        let mut c = canvas(4, 4);
        let cmd = Command::Batch(vec![
            Command::SetPixel {
                x: 0,
                y: 0,
                color: RED,
            },
            Command::SetPixel {
                x: 0,
                y: 0,
                color: BLUE,
            },
            Command::SetPixel {
                x: 1,
                y: 0,
                color: RED,
            },
            Command::SetPixel {
                x: 1,
                y: 0,
                color: Rgba::TRANSPARENT,
            },
        ]);
        assert_eq!(cmd.count(), 4);
        let p = cmd.apply(&mut c).unwrap();
        assert_eq!(
            p.cells().copied().collect::<Vec<_>>(),
            vec![CellChange {
                x: 0,
                y: 0,
                before: Rgba::TRANSPARENT,
                after: BLUE
            }]
        );
    }

    #[test]
    fn batch_is_atomic() {
        let mut c = canvas(4, 4);
        let cmd = Command::Batch(vec![
            Command::FillRect {
                x: 0,
                y: 0,
                w: 2,
                h: 2,
                color: RED,
            },
            Command::Resize { w: 8, h: 8 },
            Command::SetPixel {
                x: 20,
                y: 0,
                color: BLUE,
            },
        ]);
        let err = cmd.apply(&mut c).unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfBounds);
        assert_eq!(err.message, "pixel (20,0) outside canvas");
        assert_eq!(c, canvas(4, 4));
    }

    #[test]
    fn failing_command_leaves_canvas_untouched() {
        let mut c = canvas(4, 4);
        c.set(0, 0, RED).unwrap();
        let before = c.clone();
        for cmd in [
            Command::SetPixel {
                x: -1,
                y: 0,
                color: BLUE,
            },
            Command::FillRect {
                x: 3,
                y: 3,
                w: 2,
                h: 1,
                color: BLUE,
            },
            Command::FillRect {
                x: 0,
                y: 0,
                w: 0,
                h: 1,
                color: BLUE,
            },
            Command::Line {
                from: (0, 0),
                to: (4, 0),
                color: BLUE,
            },
            Command::Resize { w: 0, h: 4 },
        ] {
            assert!(cmd.apply(&mut c).is_err(), "{cmd:?}");
            assert_eq!(c, before, "{cmd:?}");
        }
    }

    #[test]
    fn no_op_commands_produce_empty_patches() {
        let mut c = canvas(4, 4);
        c.fill_rect(0, 0, 4, 4, RED).unwrap();
        for cmd in [
            Command::SetPixel {
                x: 1,
                y: 1,
                color: RED,
            },
            Command::FillRect {
                x: 0,
                y: 0,
                w: 2,
                h: 2,
                color: RED,
            },
            Command::Line {
                from: (0, 3),
                to: (3, 0),
                color: RED,
            },
            Command::Clear { color: Some(RED) },
            Command::Resize { w: 4, h: 4 },
            Command::Batch(vec![]),
            Command::Batch(vec![
                Command::SetPixel {
                    x: 0,
                    y: 0,
                    color: BLUE,
                },
                Command::SetPixel {
                    x: 0,
                    y: 0,
                    color: RED,
                },
            ]),
        ] {
            let p = cmd.apply(&mut c).unwrap();
            assert!(p.is_empty(), "{cmd:?} → {p:?}");
        }
    }

    // --- Propiedades ---

    fn color() -> impl Strategy<Value = Rgba> {
        prop_oneof![
            Just(Rgba::TRANSPARENT),
            Just(RED),
            Just(BLUE),
            any::<(u8, u8, u8, u8)>().prop_map(|(r, g, b, a)| Rgba::new(r, g, b, a)),
        ]
    }

    /// Comandos simples, a veces fuera del lienzo para probar también los errores.
    fn simple_command() -> impl Strategy<Value = Command> {
        let coord = -2i64..14;
        prop_oneof![
            (coord.clone(), coord.clone(), color()).prop_map(|(x, y, color)| Command::SetPixel {
                x,
                y,
                color
            }),
            (coord.clone(), coord.clone(), -1i64..10, -1i64..10, color())
                .prop_map(|(x, y, w, h, color)| Command::FillRect { x, y, w, h, color }),
            (coord.clone(), coord.clone(), coord.clone(), coord, color()).prop_map(
                |(x1, y1, x2, y2, color)| Command::Line {
                    from: (x1, y1),
                    to: (x2, y2),
                    color
                }
            ),
            proptest::option::of(color()).prop_map(|color| Command::Clear { color }),
            (0i64..14, 0i64..14).prop_map(|(w, h)| Command::Resize { w, h }),
        ]
    }

    fn command() -> impl Strategy<Value = Command> {
        prop_oneof![
            3 => simple_command(),
            1 => proptest::collection::vec(simple_command(), 0..12).prop_map(Command::Batch),
        ]
    }

    /// Lienzo de partida con algo dibujado.
    fn start_canvas() -> impl Strategy<Value = Canvas> {
        (
            1i64..12,
            1i64..12,
            proptest::collection::vec(simple_command(), 0..6),
        )
            .prop_map(|(w, h, commands)| {
                let mut c = Canvas::new(w, h).unwrap();
                for cmd in commands {
                    let _ = cmd.apply(&mut c);
                }
                c
            })
    }

    proptest! {
        #[test]
        fn apply_then_revert_is_identity(start in start_canvas(), cmd in command()) {
            let mut c = start.clone();
            match cmd.apply(&mut c) {
                Ok(patch) => {
                    let after = c.clone();
                    patch.revert(&mut c).unwrap();
                    prop_assert_eq!(&c, &start);
                    patch.reapply(&mut c).unwrap();
                    prop_assert_eq!(&c, &after);
                }
                Err(_) => prop_assert_eq!(&c, &start),
            }
        }

        #[test]
        fn sequence_reverts_in_reverse_order(
            start in start_canvas(),
            cmds in proptest::collection::vec(command(), 1..8),
        ) {
            let mut c = start.clone();
            let patches: Vec<Patch> = cmds.iter().filter_map(|cmd| cmd.apply(&mut c).ok()).collect();
            for patch in patches.iter().rev() {
                patch.revert(&mut c).unwrap();
            }
            prop_assert_eq!(&c, &start);
        }

        #[test]
        fn patch_holds_only_changed_cells(start in start_canvas(), cmd in command()) {
            let mut c = start.clone();
            if let Ok(patch) = cmd.apply(&mut c) {
                for change in patch.cells() {
                    prop_assert_ne!(change.before, change.after);
                }
            }
        }
    }
}
