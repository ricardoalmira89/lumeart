//! El pincel: ver dibujar al agente píxel a píxel, como en el studio web de
//! pxcli. Cada cambio que llega del agente o de la CLI con sus líneas se
//! repite sobre el lienzo de antes (`lume_domain::replay`) y se va pintando a
//! la velocidad elegida; si se queda atrás, acelera para ponerse al día.
//!
//! Solo cambia lo que se ve: el documento ya tiene el cambio entero. Cuando
//! la cola se vacía, se vuelve a mostrar el lienzo de verdad.

use std::collections::VecDeque;
use std::sync::Arc;

use lume_app::{DrawSpeed, ViewModel};
use lume_domain::replay::{DrawStep, StepPaint, draw_steps};
use lume_domain::{Canvas, Origin};

/// Qué hay que subir a la textura tras un fotograma del pincel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dirty {
    /// Todo lo que se ve.
    Full,
    /// Este rectángulo: `x`, `y`, ancho, alto.
    Region { x: u32, y: u32, w: u32, h: u32 },
}

/// Lo que pasó en un fotograma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tick {
    /// Hay que subir esto (si sigue pintando).
    pub dirty: Option<Dirty>,
    /// Acaba de terminar: hay que volver a mostrar el lienzo de verdad.
    pub finished: bool,
}

/// Un paso en la cola, con lo que lleva pintado.
#[derive(Debug, Clone)]
struct Job {
    revision: u64,
    line: usize,
    step: DrawStep,
    done: usize,
}

/// Por dónde va el pincel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Revisión del cambio que está pintando.
    pub revision: u64,
    /// Línea de ese cambio.
    pub line: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Brush {
    /// Lo que se ve mientras pinta.
    display: Option<Canvas>,
    queue: VecDeque<Job>,
    /// Celdas que faltan por pintar.
    pending: usize,
    /// Fracción de celda que sobró del fotograma anterior.
    carry: f32,
    /// Celdas por segundo para acabar lo que hay en cola dentro del plazo de
    /// la velocidad (si la velocidad normal no llega).
    boost: f32,
    /// Última celda pintada (donde se dibuja el pincel).
    last: Option<(u32, u32)>,
    /// Última revisión vista.
    seen: Option<u64>,
    /// Revisión del cambio más nuevo visto: uno que ya se vio (al volver de
    /// ver una versión, por ejemplo) no se vuelve a pintar.
    newest: u64,
    /// El lienzo de verdad al que lleva la cola.
    target: Option<Arc<Canvas>>,
}

impl Brush {
    /// Lo que hay que mostrar mientras pinta (`None`: el lienzo de verdad).
    pub fn display(&self) -> Option<&Canvas> {
        self.display.as_ref()
    }

    /// ¿Está pintando?
    pub fn is_painting(&self) -> bool {
        self.display.is_some()
    }

    /// Celda donde está el pincel.
    pub fn position(&self) -> Option<(u32, u32)> {
        self.display.as_ref().and(self.last)
    }

    /// Paso que está pintando.
    pub fn progress(&self) -> Option<Progress> {
        self.queue.front().map(|j| Progress {
            revision: j.revision,
            line: j.line,
        })
    }

    /// ¿Le quedan pasos de esta revisión por empezar o terminar?
    pub fn is_pending(&self, revision: u64) -> bool {
        self.queue.iter().any(|j| j.revision == revision)
    }

    /// Mira el `ViewModel` del fotograma: si trae un cambio nuevo del agente
    /// o de la CLI con sus líneas, lo pone en la cola; si trae otra cosa que
    /// cambia el lienzo, deja de pintar y muestra la verdad.
    pub fn observe(&mut self, vm: &ViewModel, speed: DrawSpeed) {
        let first = self.seen.is_none();
        if self.seen == Some(vm.revision) {
            return;
        }
        self.seen = Some(vm.revision);
        let newest = self.newest;
        if let Some(change) = &vm.last_change {
            self.newest = self.newest.max(change.revision);
        }
        if first {
            return;
        }
        let change = vm
            .last_change
            .as_ref()
            .filter(|c| c.revision == vm.revision && c.revision > newest);
        let animate = change.filter(|c| {
            c.origin != Origin::User && !c.lines.is_empty() && speed != DrawSpeed::Instant
        });
        let Some(change) = animate else {
            // Una revisión que no cambia el lienzo (versión, nombre…) no
            // corta lo que se está pintando.
            let same = self
                .target
                .as_ref()
                .is_some_and(|t| Arc::ptr_eq(t, &vm.canvas) || **t == *vm.canvas);
            if !same {
                self.stop();
            }
            return;
        };
        let mut before = (*vm.canvas).clone();
        if change.patch.revert(&mut before).is_err() {
            self.stop();
            return;
        }
        let steps = draw_steps(&before, &change.lines);
        if self.display.is_none() {
            self.display = Some(before);
        }
        for (line, step) in steps.into_iter().enumerate() {
            self.pending += step.cell_count();
            self.queue.push_back(Job {
                revision: change.revision,
                line,
                step,
                done: 0,
            });
        }
        self.target = Some(vm.canvas.clone());
        if let Some((_, lag)) = speed.rate() {
            self.boost = self.boost.max(self.pending as f32 / lag);
        }
    }

    /// Deja de pintar: se ve el lienzo de verdad.
    pub fn stop(&mut self) {
        self.display = None;
        self.queue.clear();
        self.pending = 0;
        self.carry = 0.0;
        self.boost = 0.0;
        self.last = None;
        self.target = None;
    }

    /// Pinta lo que toque en `dt` segundos.
    pub fn tick(&mut self, dt: f32, speed: DrawSpeed) -> Tick {
        let Some(display) = self.display.as_mut() else {
            return Tick::default();
        };
        let mut budget = match speed.rate() {
            // Si va por detrás, acelera para acabar la cola en el plazo.
            Some((rate, _)) => self.carry + rate.max(self.boost) * dt.clamp(0.0, 0.1),
            None => f32::INFINITY,
        };
        let mut dirty: Option<Dirty> = None;
        let mut grow = |x: u32, y: u32| {
            dirty = Some(match dirty {
                Some(Dirty::Full) => Dirty::Full,
                Some(Dirty::Region { x: rx, y: ry, w, h }) => {
                    let (x0, y0) = (rx.min(x), ry.min(y));
                    let (x1, y1) = ((rx + w).max(x + 1), (ry + h).max(y + 1));
                    Dirty::Region {
                        x: x0,
                        y: y0,
                        w: x1 - x0,
                        h: y1 - y0,
                    }
                }
                None => Dirty::Region { x, y, w: 1, h: 1 },
            });
        };
        let mut full = false;
        while let Some(job) = self.queue.front_mut() {
            match &job.step.paint {
                StepPaint::Frame(canvas) => {
                    if budget < 1.0 {
                        break;
                    }
                    *display = canvas.clone();
                    budget -= 1.0;
                    full = true;
                    self.queue.pop_front();
                }
                StepPaint::Cells(cells) => {
                    while job.done < cells.len() && budget >= 1.0 {
                        let (x, y, color) = cells[job.done];
                        let _ = display.set(i64::from(x), i64::from(y), color);
                        grow(x, y);
                        self.last = Some((x, y));
                        job.done += 1;
                        self.pending = self.pending.saturating_sub(1);
                        budget -= 1.0;
                    }
                    if job.done < cells.len() {
                        break;
                    }
                    self.queue.pop_front();
                }
            }
        }
        if self.queue.is_empty() {
            self.stop();
            return Tick {
                dirty: None,
                finished: true,
            };
        }
        self.carry = if budget.is_finite() { budget } else { 0.0 };
        Tick {
            dirty: if full { Some(Dirty::Full) } else { dirty },
            finished: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_app::Change;
    use lume_domain::{Document, Patch, Rgba};

    /// `ViewModel` tras un batch de `origin` (o sin líneas, si `lines` está
    /// vacío).
    fn after(doc: &mut Document, revision: u64, origin: Origin, lines: &[&str]) -> ViewModel {
        let before = doc.canvas().clone();
        for line in lines {
            let _ = doc.handle_line(line, origin);
        }
        let change = Change {
            revision,
            origin,
            label: "batch".into(),
            patch: Arc::new(Patch::between(&before, doc.canvas())),
            lines: Arc::new(lines.iter().map(|s| s.to_string()).collect()),
        };
        ViewModel::of(doc, revision, Some(change))
    }

    #[test]
    fn paints_cell_by_cell_at_the_chosen_speed() {
        let mut doc = Document::new("b", 8, 8).unwrap();
        let mut brush = Brush::default();
        brush.observe(&ViewModel::of(&doc, 0, None), DrawSpeed::Normal);
        let vm = after(
            &mut doc,
            1,
            Origin::Agent { run: 1 },
            &["fill_rect 0 0 4 2 #ff0000", "set_pixel 7 7 #00ff00"],
        );
        brush.observe(&vm, DrawSpeed::Normal);
        assert!(brush.is_painting());
        assert_eq!(
            brush
                .display()
                .unwrap()
                .pixels()
                .iter()
                .filter(|p| p.a > 0)
                .count(),
            0
        );
        assert_eq!(
            brush.progress(),
            Some(Progress {
                revision: 1,
                line: 0
            })
        );

        // 160 píxeles por segundo: en 1/40 s, 4 celdas (la primera fila).
        let tick = brush.tick(0.025, DrawSpeed::Normal);
        assert_eq!(
            tick.dirty,
            Some(Dirty::Region {
                x: 0,
                y: 0,
                w: 4,
                h: 1
            })
        );
        assert_eq!(brush.position(), Some((3, 0)));
        let shown = brush.display().unwrap();
        assert_eq!(shown.get(3, 0).unwrap(), Rgba::rgb(255, 0, 0));
        assert_eq!(shown.get(0, 1).unwrap(), Rgba::TRANSPARENT);

        // El resto: la segunda fila y luego la línea siguiente.
        brush.tick(0.025, DrawSpeed::Normal);
        assert_eq!(
            brush.progress(),
            Some(Progress {
                revision: 1,
                line: 1
            })
        );
        let tick = brush.tick(0.025, DrawSpeed::Normal);
        assert!(tick.finished);
        assert!(!brush.is_painting());
        assert!(!brush.is_pending(1));
    }

    #[test]
    fn falls_behind_and_catches_up() {
        let mut doc = Document::new("b", 64, 64).unwrap();
        let mut brush = Brush::default();
        brush.observe(&ViewModel::of(&doc, 0, None), DrawSpeed::Slow);
        let vm = after(&mut doc, 1, Origin::Cli, &["fill_rect 0 0 64 64 #ff0000"]);
        brush.observe(&vm, DrawSpeed::Slow);
        // 4096 celdas a 40 por segundo serían 100 s; se pone al día en 20.
        let mut seconds = 0.0;
        while brush.is_painting() {
            brush.tick(0.1, DrawSpeed::Slow);
            seconds += 0.1;
            assert!(seconds < 25.0, "no se pone al día");
        }
        assert!(seconds > 15.0, "{seconds}");
    }

    #[test]
    fn own_strokes_and_instant_speed_are_not_animated() {
        let mut doc = Document::new("b", 8, 8).unwrap();
        let mut brush = Brush::default();
        brush.observe(&ViewModel::of(&doc, 0, None), DrawSpeed::Normal);
        let vm = after(&mut doc, 1, Origin::User, &["set_pixel 1 1 #ff0000"]);
        brush.observe(&vm, DrawSpeed::Normal);
        assert!(!brush.is_painting(), "lo que dibujas tú");
        let vm = after(&mut doc, 2, Origin::Cli, &["set_pixel 2 2 #ff0000"]);
        brush.observe(&vm, DrawSpeed::Instant);
        assert!(!brush.is_painting(), "al instante");
        let vm = after(&mut doc, 3, Origin::Cli, &[]);
        brush.observe(&vm, DrawSpeed::Normal);
        assert!(!brush.is_painting(), "sin líneas");
    }

    #[test]
    fn a_window_change_stops_it_but_an_untouched_canvas_does_not() {
        let mut doc = Document::new("b", 8, 8).unwrap();
        let mut brush = Brush::default();
        brush.observe(&ViewModel::of(&doc, 0, None), DrawSpeed::Normal);
        let vm = after(&mut doc, 1, Origin::Cli, &["fill_rect 0 0 8 8 #ff0000"]);
        brush.observe(&vm, DrawSpeed::Normal);
        brush.tick(0.01, DrawSpeed::Normal);
        // Revisión nueva con el mismo lienzo (se guardó una versión).
        brush.observe(
            &ViewModel::of(&doc, 2, vm.last_change.clone()),
            DrawSpeed::Normal,
        );
        assert!(brush.is_painting());
        // Deshacer en la ventana: se ve la verdad enseguida.
        let _ = doc.handle_line("undo", Origin::User);
        brush.observe(&ViewModel::of(&doc, 3, None), DrawSpeed::Normal);
        assert!(!brush.is_painting());
    }

    /// Volver de ver una versión trae otra vez la revisión del último
    /// cambio: no se vuelve a pintar.
    #[test]
    fn a_change_already_seen_is_not_painted_again() {
        let mut doc = Document::new("b", 8, 8).unwrap();
        let mut brush = Brush::default();
        brush.observe(&ViewModel::of(&doc, 0, None), DrawSpeed::Normal);
        let vm = after(&mut doc, 1, Origin::Cli, &["set_pixel 1 1 #ff0000"]);
        brush.observe(&vm, DrawSpeed::Instant);
        // Viendo una versión (otra revisión, sin cambio) y de vuelta.
        brush.observe(&ViewModel::of(&doc, 1 << 62, None), DrawSpeed::Normal);
        brush.observe(&vm, DrawSpeed::Normal);
        assert!(!brush.is_painting());
    }

    #[test]
    fn two_changes_in_a_row_queue_up() {
        let mut doc = Document::new("b", 8, 8).unwrap();
        let mut brush = Brush::default();
        brush.observe(&ViewModel::of(&doc, 0, None), DrawSpeed::Normal);
        let vm = after(&mut doc, 1, Origin::Cli, &["fill_rect 0 0 8 1 #ff0000"]);
        brush.observe(&vm, DrawSpeed::Normal);
        let vm2 = after(&mut doc, 2, Origin::Cli, &["fill_rect 0 1 8 1 #00ff00"]);
        brush.observe(&vm2, DrawSpeed::Normal);
        assert!(brush.is_pending(1) && brush.is_pending(2));
        // Primero el uno entero, luego el dos.
        while brush.progress().map(|p| p.revision) == Some(1) {
            brush.tick(0.016, DrawSpeed::Normal);
        }
        let shown = brush.display().unwrap();
        assert_eq!(shown.get(7, 0).unwrap(), Rgba::rgb(255, 0, 0));
        assert_eq!(shown.get(7, 1).unwrap(), Rgba::TRANSPARENT);
        while brush.is_painting() {
            brush.tick(0.016, DrawSpeed::Normal);
        }
    }
}
