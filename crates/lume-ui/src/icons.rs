//! Iconos de trazo (1.6 px, extremos redondeados) dibujados con el `Painter`,
//! en una caja de 18 × 18 centrada en `c`.

use eframe::egui::{Color32, Painter, Pos2, Shape, Stroke, pos2, vec2};

/// Qué icono.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Pencil,
    Eraser,
    Fill,
    Line,
    Rect,
    Ellipse,
    Eyedropper,
    Select,
    Move,
    Hand,
    Undo,
    Redo,
    Export,
    Grid,
    Lasso,
    Play,
    Eye,
    /// Flecha del ratón (seleccionar una forma).
    Pointer,
    /// Carpeta (abrir).
    Folder,
    /// Engranaje (ajustes).
    Settings,
    /// Curva con sus nodos (dibujo vectorial).
    Curve,
    /// Pausa (dos barras).
    Pause,
    /// Una T (texto).
    Text,
}

const W: f32 = 1.6;

fn line(p: &Painter, c: Pos2, color: Color32, pts: &[(f32, f32)]) {
    let pts: Vec<Pos2> = pts.iter().map(|(x, y)| c + vec2(*x, *y)).collect();
    p.add(Shape::line(pts, Stroke::new(W, color)));
}

fn closed(p: &Painter, c: Pos2, color: Color32, pts: &[(f32, f32)]) {
    let pts: Vec<Pos2> = pts.iter().map(|(x, y)| c + vec2(*x, *y)).collect();
    p.add(Shape::closed_line(pts, Stroke::new(W, color)));
}

/// Arco de `from` a `to` grados (0 = derecha, sentido horario en pantalla).
fn arc(p: &Painter, c: Pos2, color: Color32, r: f32, from: f32, to: f32) {
    let steps = 16;
    let pts: Vec<Pos2> = (0..=steps)
        .map(|i| {
            let a = (from + (to - from) * i as f32 / steps as f32).to_radians();
            c + vec2(a.cos() * r, a.sin() * r)
        })
        .collect();
    p.add(Shape::line(pts, Stroke::new(W, color)));
}

/// Dibuja `icon` centrado en `c`.
pub fn paint(p: &Painter, icon: Icon, c: Pos2, color: Color32) {
    let s = Stroke::new(W, color);
    match icon {
        Icon::Pencil => {
            closed(
                p,
                c,
                color,
                &[
                    (-6.0, 6.0),
                    (-6.0, 3.5),
                    (3.5, -6.0),
                    (6.0, -3.5),
                    (-3.5, 6.0),
                ],
            );
            line(p, c, color, &[(1.5, -4.0), (4.0, -1.5)]);
        }
        Icon::Eraser => {
            closed(
                p,
                c,
                color,
                &[
                    (-6.5, 2.5),
                    (0.5, -4.5),
                    (5.5, 0.5),
                    (-1.5, 7.5),
                    (-3.5, 7.5),
                ],
            );
            line(p, c, color, &[(-3.0, -1.0), (2.0, 4.0)]);
            line(p, c, color, &[(-1.5, 7.5), (7.0, 7.5)]);
        }
        Icon::Fill => {
            closed(
                p,
                c,
                color,
                &[(-6.0, 0.0), (0.0, -6.0), (6.0, 0.0), (0.0, 6.0)],
            );
            line(p, c, color, &[(-6.0, 0.0), (6.0, 0.0)]);
            p.circle_stroke(c + vec2(7.0, 5.0), 1.6, s);
        }
        Icon::Line => {
            line(p, c, color, &[(-5.5, 5.5), (5.5, -5.5)]);
            p.circle_filled(c + vec2(-5.5, 5.5), 1.8, color);
            p.circle_filled(c + vec2(5.5, -5.5), 1.8, color);
        }
        Icon::Rect => {
            closed(
                p,
                c,
                color,
                &[(-6.5, -5.0), (6.5, -5.0), (6.5, 5.0), (-6.5, 5.0)],
            );
        }
        Icon::Ellipse => {
            let pts: Vec<Pos2> = (0..32)
                .map(|i| {
                    let a = (i as f32 / 32.0) * std::f32::consts::TAU;
                    c + vec2(a.cos() * 7.0, a.sin() * 5.5)
                })
                .collect();
            p.add(Shape::closed_line(pts, s));
        }
        Icon::Eyedropper => {
            line(p, c, color, &[(-6.0, 6.0), (-6.0, 4.0), (2.0, -4.0)]);
            line(p, c, color, &[(-6.0, 6.0), (-4.0, 6.0), (4.0, -2.0)]);
            line(p, c, color, &[(0.5, -5.5), (5.5, -0.5)]);
            p.circle_stroke(c + vec2(4.5, -4.5), 2.2, s);
        }
        Icon::Select => {
            let dash = |a: Pos2, b: Pos2| {
                p.add(Shape::dashed_line(&[a, b], s, 2.5, 2.0));
            };
            let (l, t, r, b) = (c.x - 6.5, c.y - 6.0, c.x + 6.5, c.y + 6.0);
            dash(pos2(l, t), pos2(r, t));
            dash(pos2(r, t), pos2(r, b));
            dash(pos2(r, b), pos2(l, b));
            dash(pos2(l, b), pos2(l, t));
        }
        Icon::Move => {
            line(p, c, color, &[(0.0, -7.0), (0.0, 7.0)]);
            line(p, c, color, &[(-7.0, 0.0), (7.0, 0.0)]);
            line(p, c, color, &[(-2.0, -5.0), (0.0, -7.0), (2.0, -5.0)]);
            line(p, c, color, &[(-2.0, 5.0), (0.0, 7.0), (2.0, 5.0)]);
            line(p, c, color, &[(-5.0, -2.0), (-7.0, 0.0), (-5.0, 2.0)]);
            line(p, c, color, &[(5.0, -2.0), (7.0, 0.0), (5.0, 2.0)]);
        }
        Icon::Hand => {
            for x in [-3.5f32, -0.5, 2.5] {
                line(p, c, color, &[(x, 1.0), (x, -5.5)]);
            }
            line(p, c, color, &[(5.0, 1.0), (5.0, -3.0)]);
            line(
                p,
                c,
                color,
                &[
                    (-3.5, 1.0),
                    (-6.0, -1.0),
                    (-6.5, 1.5),
                    (-3.0, 6.5),
                    (3.5, 6.5),
                    (5.0, 3.0),
                    (5.0, 1.0),
                ],
            );
        }
        Icon::Undo => {
            arc(p, c + vec2(1.0, 1.0), color, 5.0, 200.0, 450.0);
            line(p, c, color, &[(-6.5, -4.0), (-4.0, -0.5), (-0.5, -3.0)]);
        }
        Icon::Redo => {
            arc(p, c + vec2(-1.0, 1.0), color, 5.0, -270.0, -20.0);
            line(p, c, color, &[(6.5, -4.0), (4.0, -0.5), (0.5, -3.0)]);
        }
        Icon::Export => {
            line(p, c, color, &[(0.0, -6.5), (0.0, 3.0)]);
            line(p, c, color, &[(-3.5, -0.5), (0.0, 3.0), (3.5, -0.5)]);
            line(
                p,
                c,
                color,
                &[(-6.5, 3.5), (-6.5, 6.5), (6.5, 6.5), (6.5, 3.5)],
            );
        }
        Icon::Grid => {
            for k in [-2.5f32, 2.5] {
                line(p, c, color, &[(k, -6.5), (k, 6.5)]);
                line(p, c, color, &[(-6.5, k), (6.5, k)]);
            }
        }
        Icon::Pointer => {
            closed(
                p,
                c,
                color,
                &[
                    (-4.5, -7.0),
                    (-4.5, 5.0),
                    (-1.5, 2.0),
                    (1.0, 7.0),
                    (3.0, 6.0),
                    (0.5, 1.0),
                    (4.5, 1.0),
                ],
            );
        }
        Icon::Text => {
            line(p, c, color, &[(-5.5, -6.0), (5.5, -6.0)]);
            line(p, c, color, &[(0.0, -6.0), (0.0, 6.5)]);
            line(p, c, color, &[(-2.5, 6.5), (2.5, 6.5)]);
        }
        Icon::Lasso => {
            // Un lazo: un óvalo con su cola.
            let pts: Vec<Pos2> = (0..=24)
                .map(|i| {
                    let a = i as f32 / 24.0 * std::f32::consts::TAU;
                    c + vec2(6.5 * a.cos(), -1.5 + 4.5 * a.sin())
                })
                .collect();
            p.add(Shape::line(pts, s));
            line(p, c, color, &[(-3.0, 2.5), (-4.0, 5.5), (-2.0, 7.0)]);
        }
        Icon::Eye => {
            let top: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let t = i as f32 / 12.0;
                    c + vec2(-6.5 + 13.0 * t, -4.0 * (std::f32::consts::PI * t).sin())
                })
                .collect();
            let bottom: Vec<Pos2> = top.iter().map(|q| pos2(q.x, 2.0 * c.y - q.y)).collect();
            p.add(Shape::line(top, s));
            p.add(Shape::line(bottom, s));
            p.circle_stroke(c, 1.8, s);
        }
        Icon::Play => {
            p.add(Shape::convex_polygon(
                vec![
                    c + vec2(-3.5, -5.0),
                    c + vec2(5.0, 0.0),
                    c + vec2(-3.5, 5.0),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Pause => {
            for dx in [-3.0, 3.0] {
                p.rect_filled(
                    eframe::egui::Rect::from_center_size(c + vec2(dx, 0.0), vec2(3.0, 10.0)),
                    1.0,
                    color,
                );
            }
        }
        Icon::Folder => {
            closed(
                p,
                c,
                color,
                &[
                    (-7.0, -5.0),
                    (-2.5, -5.0),
                    (-1.0, -3.0),
                    (7.0, -3.0),
                    (7.0, 5.5),
                    (-7.0, 5.5),
                ],
            );
        }
        Icon::Settings => {
            p.circle_stroke(c, 3.6, s);
            for i in 0..8 {
                let a = i as f32 / 8.0 * std::f32::consts::TAU;
                let (x, y) = (a.cos(), a.sin());
                p.line_segment([c + vec2(5.2 * x, 5.2 * y), c + vec2(7.0 * x, 7.0 * y)], s);
            }
        }
        Icon::Curve => {
            let pts: Vec<Pos2> = (0..=16)
                .map(|i| {
                    let t = i as f32 / 16.0;
                    let a = pos2(-6.0, 5.0);
                    let b = pos2(-1.0, -9.0);
                    let d = pos2(6.0, -4.0);
                    let u = 1.0 - t;
                    c + (a.to_vec2() * u * u + b.to_vec2() * 2.0 * u * t + d.to_vec2() * t * t)
                })
                .collect();
            p.add(Shape::line(pts, s));
            for q in [vec2(-6.0, 5.0), vec2(6.0, -4.0)] {
                p.rect_filled(
                    eframe::egui::Rect::from_center_size(c + q, vec2(3.4, 3.4)),
                    0.0,
                    color,
                );
            }
        }
    }
}
