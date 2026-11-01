//! Animación de un dibujo vectorial (modo Animación sobre un documento con
//! `dibujo.js`), como en Spine, Rive o After Effects: el esqueleto de
//! `dibujo.js` (`bone`, `part`, `action`) se reproduce en tiempo real.
//!
//! - Cada pieza se pinta una vez como imagen (`StudioApp` lo hace en segundo
//!   plano) y la ventana la mueve y la gira con la matriz de su hueso en cada
//!   fotograma: se reproduce suave y se puede arrastrar el cabezal.
//! - Lo que no está en ninguna pieza es el fondo fijo.
//! - A la izquierda, las acciones; abajo, la línea de tiempo con una pista
//!   por hueso y sus fotogramas clave; en el centro, el dibujo posado.
//! - El agente crea el esqueleto y las acciones editando `dibujo.js`.
//!
//! Es un hijo de `shell`, como el modo vector.

use std::sync::Arc;

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Frame, Layout, Margin, Mesh, Panel, Rect, RichText,
    Sense, Shape, Stroke, StrokeKind, TextureHandle, TextureOptions, UiBuilder, pos2, vec2,
};
use lume_app::ViewModel;
use lume_app::ports::RasterImage;
use lume_domain::vec::rig::{Rig, RigAction};

use super::{AppShell, bar_frame, icon_button, panel_label, vsep};
use crate::icons::Icon;
use crate::intent::Intent;
use crate::theme;

/// Ancho de la lista de acciones.
const ACTIONS_WIDTH: f32 = 230.0;
/// Alto de la línea de tiempo.
const TIMELINE_HEIGHT: f32 = 190.0;
/// Columna de los nombres de los huesos en la línea de tiempo.
const TRACK_LABEL: f32 = 120.0;
/// Alto de una pista.
const TRACK_ROW: f32 = 22.0;

/// Las piezas del esqueleto ya pintadas y el fondo fijo, para una versión
/// de `dibujo.js` (`key`).
pub struct RigImages {
    pub key: u64,
    /// Una por pieza, en el orden de `Rig::parts` (recortada a su caja).
    pub parts: Vec<Option<RasterImage>>,
    /// Lo que no está en ninguna pieza (del tamaño del dibujo).
    pub backdrop: Option<RasterImage>,
}

/// Estado del modo de animación vectorial.
#[derive(Default)]
pub struct VecAnim {
    /// Acción elegida (`None`: el reposo).
    pub action: Option<String>,
    /// Segundo de la acción que se ve.
    pub time: f64,
    pub playing: bool,
    /// Enseñar los huesos (articulaciones y uniones) encima.
    pub bones: bool,
    /// El usuario ya eligió algo en la lista (si no, se reproduce la primera
    /// acción al abrir).
    pub chosen: bool,
    /// Las piezas pintadas (las pone `StudioApp`).
    pub images: Option<Arc<RigImages>>,
    /// El diálogo «Exportar» abierto: GIF (o hoja), fps y ancho.
    pub export: Option<(bool, u32, u32)>,
    textures: Option<(u64, Vec<Option<TextureHandle>>, Option<TextureHandle>)>,
    /// Hora del último fotograma (para avanzar al reproducir).
    last: Option<f64>,
}

impl Clone for VecAnim {
    fn clone(&self) -> Self {
        Self {
            action: self.action.clone(),
            time: self.time,
            playing: self.playing,
            bones: self.bones,
            chosen: self.chosen,
            images: self.images.clone(),
            export: self.export,
            textures: self.textures.clone(),
            last: self.last,
        }
    }
}

impl std::fmt::Debug for VecAnim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecAnim")
            .field("action", &self.action)
            .field("time", &self.time)
            .field("playing", &self.playing)
            .finish_non_exhaustive()
    }
}

impl VecAnim {
    /// La acción elegida dentro del esqueleto (si sigue existiendo).
    fn current<'a>(&self, rig: &'a Rig) -> Option<&'a RigAction> {
        let name = self.action.as_deref()?;
        rig.actions.iter().find(|a| a.name == name)
    }

    /// Texturas de las piezas y del fondo (se crean al llegar las imágenes).
    fn textures(
        &mut self,
        ctx: &egui::Context,
    ) -> Option<(&[Option<TextureHandle>], Option<&TextureHandle>)> {
        let images = self.images.clone()?;
        if self
            .textures
            .as_ref()
            .is_none_or(|(k, _, _)| *k != images.key)
        {
            let load = |name: String, img: &RasterImage| {
                let color = egui::ColorImage::from_rgba_unmultiplied(
                    [img.width as usize, img.height as usize],
                    &img.rgba,
                );
                ctx.load_texture(name, color, TextureOptions::LINEAR)
            };
            let parts = images
                .parts
                .iter()
                .enumerate()
                .map(|(i, img)| {
                    img.as_ref()
                        .map(|img| load(format!("pieza-{}-{i}", images.key), img))
                })
                .collect();
            let backdrop = images
                .backdrop
                .as_ref()
                .map(|img| load(format!("fondo-fijo-{}", images.key), img));
            self.textures = Some((images.key, parts, backdrop));
        }
        self.textures
            .as_ref()
            .map(|(_, parts, backdrop)| (parts.as_slice(), backdrop.as_ref()))
    }
}

/// Fotogramas de una acción a `fps`: en bucle, el último no repite el
/// primero; una vez, se incluye el final.
pub fn frame_count(action: &RigAction, fps: u32) -> usize {
    let n = (action.duration * f64::from(fps)).round().max(1.0) as usize;
    if action.looping { n } else { n + 1 }
}

/// Aplica una matriz `[a, b, c, d, e, f]` a un punto del reposo.
fn apply(m: &[f64; 6], x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

impl AppShell {
    /// El modo Animación de un documento con dibujo vectorial.
    pub(crate) fn show_vec_animation(
        &mut self,
        ui: &mut egui::Ui,
        vm: &ViewModel,
        out: &mut Vec<Intent>,
    ) {
        let header = Panel::top("cabecera")
            .exact_size(theme::HEADER_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin {
                left: 14,
                right: 12,
                top: 0,
                bottom: 0,
            }))
            .show(ui, |ui| self.header(ui, vm, out));
        self.zones.header = header.response.rect;

        let rig = self
            .vec
            .view
            .as_ref()
            .and_then(|v| v.result.rig.clone())
            .filter(|r| !r.parts.is_empty());

        let status = Panel::bottom("estado")
            .exact_size(theme::STATUS_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::symmetric(14, 0)))
            .show(ui, |ui| self.vanim_status(ui, rig.as_ref()));
        self.zones.status = status.response.rect;

        let agent = Panel::right("agente")
            .exact_size(theme::AGENT_WIDTH)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::same(16)))
            .show(ui, |ui| self.agent_panel(ui, out, None));
        self.zones.agent = agent.response.rect;

        let left = Panel::left("acciones-vector")
            .exact_size(ACTIONS_WIDTH)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::same(14)))
            .show(ui, |ui| self.vanim_actions(ui, rig.as_ref()));
        self.zones.toolbar = left.response.rect;

        self.tab_panel(ui, out);
        let options = Panel::top("opciones")
            .exact_size(theme::OPTIONS_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR_ALT).inner_margin(Margin::symmetric(14, 0)))
            .show(ui, |ui| self.vanim_options(ui, rig.as_ref()));
        self.zones.options = options.response.rect;

        if let Some(rig) = &rig {
            let bottom = Panel::bottom("linea-de-tiempo")
                .default_size(TIMELINE_HEIGHT)
                .size_range(110.0..=480.0)
                .resizable(true)
                .frame(bar_frame(theme::BAR))
                .show(ui, |ui| self.vanim_timeline(ui, rig));
            self.zones.bottom = bottom.response.rect;
        } else {
            self.zones.bottom = Rect::NOTHING;
        }

        let canvas = egui::CentralPanel::default()
            .frame(Frame::NONE.fill(theme::BG))
            .show(ui, |ui| self.vanim_canvas(ui, rig.as_ref(), out));
        self.zones.canvas = canvas.response.rect;
        self.vanim_export_dialog(ui.ctx(), rig.as_ref(), out);
    }

    /// «Exportar» la acción elegida: hoja de sprites (PNG, fotogramas en
    /// fila) o GIF, con sus fotogramas por segundo y su ancho.
    fn vanim_export_dialog(
        &mut self,
        ctx: &egui::Context,
        rig: Option<&Rig>,
        out: &mut Vec<Intent>,
    ) {
        let Some((mut gif, mut fps, mut width)) = self.vanim.export else {
            return;
        };
        let action = rig.and_then(|r| self.vanim.current(r).cloned());
        let mut close = false;
        let modal =
            egui::Modal::new(egui::Id::unique("lumeart-exportar-esqueleto")).show(ctx, |ui| {
                ui.set_width(400.0);
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.label(
                    RichText::new("Exportar la animación")
                        .font(theme::semibold(16.0))
                        .color(theme::TEXT),
                );
                let Some(action) = &action else {
                    ui.label(
                        RichText::new("Elige antes una acción a la izquierda.")
                            .size(13.0)
                            .color(theme::TEXT_3),
                    );
                    if ui.button("Cerrar").clicked() {
                        close = true;
                    }
                    return;
                };
                ui.horizontal(|ui| {
                    if ui.selectable_label(!gif, "Hoja de sprites (PNG)").clicked() {
                        gif = false;
                    }
                    if ui.selectable_label(gif, "GIF animado").clicked() {
                        gif = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Fotogramas por segundo")
                            .size(13.0)
                            .color(theme::TEXT_2),
                    );
                    for f in [8u32, 12, 24, 30] {
                        if ui.selectable_label(fps == f, f.to_string()).clicked() {
                            fps = f;
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Ancho de cada fotograma")
                            .size(13.0)
                            .color(theme::TEXT_2),
                    );
                    for w in [128u32, 256, 512] {
                        if ui.selectable_label(width == w, w.to_string()).clicked() {
                            width = w;
                        }
                    }
                    ui.add(
                        egui::DragValue::new(&mut width)
                            .range(16..=2048)
                            .suffix(" px"),
                    );
                });
                let frames = frame_count(action, fps);
                ui.label(
                    RichText::new(format!(
                        "{} · {frames} fotogramas · {:.2} s",
                        action.name, action.duration
                    ))
                    .font(theme::mono(12.0))
                    .color(theme::TEXT_3),
                );
                ui.horizontal(|ui| {
                    if ui.button("Cancelar").clicked() {
                        close = true;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let go = egui::Button::new(
                            RichText::new("Exportar…")
                                .font(theme::semibold(13.0))
                                .color(theme::ACCENT_INK),
                        )
                        .fill(theme::ACCENT);
                        if ui.add(go).clicked() {
                            out.push(Intent::ExportRigAction {
                                action: action.name.clone(),
                                gif,
                                fps,
                                width,
                            });
                            close = true;
                        }
                    });
                });
            });
        self.vanim.export = if close || modal.should_close() {
            None
        } else {
            Some((gif, fps, width))
        };
    }

    /// Avanza el tiempo si se está reproduciendo.
    fn vanim_tick(&mut self, ctx: &egui::Context, rig: &Rig) {
        let now = ctx.input(|i| i.time);
        let last = self.vanim.last.replace(now);
        if !self.vanim.playing {
            return;
        }
        let Some(action) = self.vanim.current(rig).cloned() else {
            self.vanim.playing = false;
            return;
        };
        let dt = last.map_or(0.0, |l| (now - l).clamp(0.0, 0.1));
        self.vanim.time += dt;
        if self.vanim.time >= action.duration {
            if action.looping {
                self.vanim.time %= action.duration.max(0.001);
            } else {
                self.vanim.time = action.duration;
                self.vanim.playing = false;
            }
        }
        ctx.request_repaint();
    }

    fn vanim_actions(&mut self, ui: &mut egui::Ui, rig: Option<&Rig>) {
        ui.spacing_mut().item_spacing.y = 8.0;
        panel_label(ui, "Acciones");
        let Some(rig) = rig else {
            ui.label(
                RichText::new("Sin esqueleto todavía.")
                    .size(12.5)
                    .color(theme::TEXT_3),
            );
            return;
        };
        let mut row = |ui: &mut egui::Ui, name: Option<&str>, detail: String| {
            let selected = self.vanim.action.as_deref() == name;
            let (rect, response) =
                ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
            let label = name.unwrap_or("Reposo");
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::Role::Button, true, selected, label)
            });
            let painter = ui.painter();
            let (fill, stroke) = if selected {
                (theme::ACCENT_SOFT, theme::ACCENT)
            } else if response.hovered() {
                (theme::HOVER, theme::LINE_STRONG)
            } else {
                (theme::CARD, theme::LINE_SOFT)
            };
            painter.rect_filled(rect, CornerRadius::same(theme::RADIUS_CARD_SMALL), fill);
            painter.rect_stroke(
                rect,
                CornerRadius::same(theme::RADIUS_CARD_SMALL),
                Stroke::new(1.0, stroke),
                StrokeKind::Inside,
            );
            painter.text(
                rect.left_top() + vec2(12.0, 9.0),
                Align2::LEFT_TOP,
                label,
                theme::semibold(13.0),
                theme::TEXT,
            );
            painter.text(
                rect.left_top() + vec2(12.0, 27.0),
                Align2::LEFT_TOP,
                detail,
                egui::FontId::proportional(11.5),
                theme::TEXT_3,
            );
            if response.clicked() {
                self.vanim.chosen = true;
                self.vanim.action = name.map(str::to_string);
                self.vanim.time = 0.0;
                self.vanim.playing = name.is_some();
            }
        };
        row(
            ui,
            None,
            format!("{} huesos · {} piezas", rig.bones.len(), rig.parts.len()),
        );
        for a in &rig.actions {
            let detail = format!(
                "{:.2} s · {} claves · {}",
                a.duration,
                a.keys.len(),
                if a.looping { "en bucle" } else { "una vez" }
            )
            .replace('.', ",");
            row(ui, Some(&a.name), detail);
        }
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "Para otra acción, pídesela al agente: «que la tapa se cierre de golpe».",
            )
            .size(11.5)
            .color(theme::TEXT_4),
        );
    }

    fn vanim_options(&mut self, ui: &mut egui::Ui, rig: Option<&Rig>) {
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            let action = rig.and_then(|r| self.vanim.current(r).cloned());
            let name = action.as_ref().map_or("Reposo", |a| a.name.as_str());
            ui.label(RichText::new(name).size(12.5).color(theme::TEXT));
            vsep(ui);
            let can_play = action.is_some();
            let icon = if self.vanim.playing {
                Icon::Pause
            } else {
                Icon::Play
            };
            let label = if self.vanim.playing {
                "Pausa (espacio)"
            } else {
                "Reproducir (espacio)"
            };
            if icon_button(ui, icon, label, 28.0, self.vanim.playing, can_play).clicked()
                && can_play
            {
                self.toggle_vanim_play(rig);
            }
            if let Some(a) = &action {
                ui.label(
                    RichText::new(format!("{:.2} s / {:.2} s", self.vanim.time, a.duration))
                        .font(theme::mono(12.0))
                        .color(theme::TEXT_2),
                );
                ui.label(
                    RichText::new(if a.looping { "En bucle" } else { "Una vez" })
                        .size(12.0)
                        .color(theme::TEXT_3),
                );
            }
            vsep(ui);
            if super::option_button(ui, "Huesos", self.vanim.bones).clicked() {
                self.vanim.bones = !self.vanim.bones;
            }
        });
    }

    /// Espacio o el botón: reproducir o parar (al final, vuelve a empezar).
    fn toggle_vanim_play(&mut self, rig: Option<&Rig>) {
        let Some(action) = rig.and_then(|r| self.vanim.current(r).cloned()) else {
            return;
        };
        if !self.vanim.playing && self.vanim.time >= action.duration {
            self.vanim.time = 0.0;
        }
        self.vanim.playing = !self.vanim.playing;
    }

    fn vanim_status(&mut self, ui: &mut egui::Ui, rig: Option<&Rig>) {
        let text = |s: String| {
            RichText::new(s)
                .font(theme::mono(11.5))
                .color(theme::TEXT_3)
        };
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 18.0;
            match rig {
                Some(r) => {
                    ui.label(text(format!("{} huesos", r.bones.len())));
                    ui.label(text(format!("{} piezas", r.parts.len())));
                    ui.label(text(format!("{} acciones", r.actions.len())));
                }
                None => {
                    ui.label(text("sin esqueleto".into()));
                }
            }
            if let Some(notice) = &self.notice {
                ui.label(RichText::new(notice).size(11.5).color(theme::OK));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(text(lume_domain::vec::FILE_NAME.into()));
            });
        });
    }

    /// La línea de tiempo: regla de segundos, una pista por hueso con sus
    /// claves (rombos) y el cabezal. Clic o arrastrar: ir a ese instante.
    fn vanim_timeline(&mut self, ui: &mut egui::Ui, rig: &Rig) {
        let full = ui.max_rect();
        ui.painter()
            .hline(full.x_range(), full.top(), Stroke::new(1.0, theme::LINE));
        let Some(action) = self.vanim.current(rig).cloned() else {
            ui.put(
                full,
                egui::Label::new(
                    RichText::new("Elige una acción para ver su línea de tiempo.")
                        .size(12.5)
                        .color(theme::TEXT_3),
                ),
            );
            return;
        };
        let inner = full.shrink2(vec2(14.0, 10.0));
        let ruler = Rect::from_min_max(
            pos2(inner.left() + TRACK_LABEL, inner.top()),
            pos2(inner.right(), inner.top() + 22.0),
        );
        let duration = action.duration.max(0.001);
        let x_of = |t: f64| ruler.left() + (t / duration) as f32 * ruler.width();
        let t_of =
            |x: f32| (((x - ruler.left()) / ruler.width()).clamp(0.0, 1.0) as f64) * duration;
        // Huesos que se mueven en la acción (los demás no tienen pista).
        let mut animated: Vec<&str> = Vec::new();
        for b in &rig.bones {
            if action.keys.iter().any(|k| k.pose.contains_key(&b.name)) {
                animated.push(&b.name);
            }
        }
        let tracks = Rect::from_min_max(
            pos2(inner.left(), ruler.bottom() + 4.0),
            pos2(inner.right(), inner.bottom()),
        );
        // Arrastrar en la regla o en las pistas mueve el cabezal.
        let scrub = ui.interact(
            Rect::from_min_max(
                pos2(ruler.left(), ruler.top()),
                pos2(ruler.right(), tracks.bottom()),
            ),
            ui.make_persistent_id("cabezal-vector"),
            Sense::click_and_drag(),
        );
        if (scrub.clicked() || scrub.dragged())
            && let Some(p) = scrub.interact_pointer_pos()
        {
            self.vanim.time = t_of(p.x);
            self.vanim.playing = false;
        }
        let painter = ui.painter_at(full);
        // Regla: marcas cada décima y números cada medio segundo (o cada
        // segundo si es larga).
        let step = if duration > 4.0 { 0.5 } else { 0.1 };
        let label_every = if duration > 4.0 { 1.0 } else { 0.5 };
        let mut t = 0.0;
        while t <= duration + 1e-6 {
            let x = x_of(t);
            let major = ((t / label_every).round() * label_every - t).abs() < 1e-6;
            painter.vline(
                x,
                (ruler.bottom() - if major { 9.0 } else { 5.0 })..=ruler.bottom(),
                Stroke::new(1.0, theme::LINE_STRONG),
            );
            if major {
                painter.text(
                    pos2(x + 3.0, ruler.top()),
                    Align2::LEFT_TOP,
                    format!("{t:.1} s").replace('.', ","),
                    theme::mono(10.5),
                    theme::TEXT_4,
                );
            }
            t += step;
        }
        // Pistas.
        for (row, bone) in animated.iter().enumerate() {
            let y = tracks.top() + row as f32 * TRACK_ROW;
            if y + TRACK_ROW > tracks.bottom() + 1.0 {
                break;
            }
            let band =
                Rect::from_min_max(pos2(inner.left(), y), pos2(inner.right(), y + TRACK_ROW));
            if row % 2 == 0 {
                painter.rect_filled(band, 0.0, theme::BAR_ALT);
            }
            painter.text(
                pos2(inner.left() + 4.0, band.center().y),
                Align2::LEFT_CENTER,
                *bone,
                theme::mono(11.5),
                theme::TEXT_2,
            );
            for key in action.keys.iter().filter(|k| k.pose.contains_key(*bone)) {
                let c = pos2(x_of(key.t * duration), band.center().y);
                let d = 5.0;
                painter.add(Shape::convex_polygon(
                    vec![
                        c + vec2(0.0, -d),
                        c + vec2(d, 0.0),
                        c + vec2(0.0, d),
                        c + vec2(-d, 0.0),
                    ],
                    theme::ACCENT,
                    Stroke::new(1.0, theme::BG),
                ));
            }
        }
        if animated.is_empty() {
            painter.text(
                pos2(ruler.left(), tracks.top() + 12.0),
                Align2::LEFT_CENTER,
                "Esta acción no mueve ningún hueso.",
                egui::FontId::proportional(12.0),
                theme::TEXT_3,
            );
        }
        // Cabezal.
        let x = x_of(self.vanim.time.min(duration));
        painter.vline(
            x,
            ruler.top()..=tracks.bottom(),
            Stroke::new(1.5, theme::ACCENT),
        );
        painter.add(Shape::convex_polygon(
            vec![
                pos2(x - 5.0, ruler.top()),
                pos2(x + 5.0, ruler.top()),
                pos2(x, ruler.top() + 7.0),
            ],
            theme::ACCENT,
            Stroke::NONE,
        ));
    }

    /// El dibujo posado en el instante elegido.
    fn vanim_canvas(&mut self, ui: &mut egui::Ui, rig: Option<&Rig>, out: &mut Vec<Intent>) {
        let full = ui.max_rect();
        let painter = ui.painter_at(full);
        let mut y = full.top() + 10.0;
        while y < full.bottom() {
            let mut x = full.left() + 10.0;
            while x < full.right() {
                painter.circle_filled(pos2(x, y), 0.8, theme::DOT);
                x += 20.0;
            }
            y += 20.0;
        }
        let Some(rig) = rig else {
            self.vanim_empty(ui, full, out);
            return;
        };
        // Al abrir un dibujo con acciones, se ve moverse la primera.
        if !self.vanim.chosen
            && self.vanim.action.is_none()
            && let Some(first) = rig.actions.first()
        {
            self.vanim.action = Some(first.name.clone());
            self.vanim.time = 0.0;
            self.vanim.playing = true;
        }
        self.vanim_tick(ui.ctx(), rig);
        if ui.input(|i| i.key_pressed(egui::Key::Space)) && ui.memory(|m| m.focused().is_none()) {
            self.toggle_vanim_play(Some(rig));
        }
        let (w, h) = (rig.width.max(1) as f32, rig.height.max(1) as f32);
        let area = full.shrink(40.0);
        let k = (area.width() / w).min(area.height() / h).max(0.01);
        let image = Rect::from_center_size(area.center(), vec2(w * k, h * k));
        let to_screen =
            |x: f64, y: f64| pos2(image.left() + x as f32 * k, image.top() + y as f32 * k);
        let shadow = egui::epaint::Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color: Color32::from_black_alpha(110),
        };
        painter.add(shadow.as_shape(image, CornerRadius::ZERO));
        let action = self.vanim.current(rig).cloned();
        let world = rig.pose_at(action.as_ref(), self.vanim.time);
        let fade = rig.alpha_at(action.as_ref(), self.vanim.time);
        let Some((parts, backdrop)) = self.vanim.textures(ui.ctx()) else {
            painter.text(
                image.center(),
                Align2::CENTER_CENTER,
                "Preparando las piezas…",
                egui::FontId::proportional(13.0),
                theme::TEXT_3,
            );
            return;
        };
        if let Some(backdrop) = backdrop {
            painter.image(
                backdrop.id(),
                image,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        // Cada pieza, en su orden, movida por su hueso (sin salirse del
        // lienzo, como al exportar).
        let painter = painter.with_clip_rect(image.intersect(full));
        for (part, texture) in rig.parts.iter().zip(parts) {
            let Some(texture) = texture else {
                continue;
            };
            let m = world
                .get(&part.bone)
                .copied()
                .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
            let opacity = fade.get(&part.bone).copied().unwrap_or(1.0) as f32;
            if opacity <= 0.0 {
                continue;
            }
            let tint = Color32::WHITE.gamma_multiply(opacity);
            let [bx, by, bw, bh] = part.bbox;
            let corners = [(bx, by), (bx + bw, by), (bx + bw, by + bh), (bx, by + bh)];
            let uvs = [
                pos2(0.0, 0.0),
                pos2(1.0, 0.0),
                pos2(1.0, 1.0),
                pos2(0.0, 1.0),
            ];
            let mut mesh = Mesh::with_texture(texture.id());
            for ((x, y), uv) in corners.into_iter().zip(uvs) {
                let (px, py) = apply(&m, x, y);
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: to_screen(px, py),
                    uv,
                    color: tint,
                });
            }
            mesh.indices.extend([0, 1, 2, 0, 2, 3]);
            painter.add(Shape::mesh(mesh));
        }
        // Huesos: cada articulación unida a la de su padre.
        if self.vanim.bones {
            for b in &rig.bones {
                let m = world
                    .get(&b.name)
                    .copied()
                    .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
                let (jx, jy) = apply(&m, b.at[0], b.at[1]);
                let joint = to_screen(jx, jy);
                if let Some(p) = rig.bones.iter().find(|p| p.name == b.parent) {
                    let pm = world
                        .get(&p.name)
                        .copied()
                        .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
                    let (px, py) = apply(&pm, p.at[0], p.at[1]);
                    painter.line_segment(
                        [to_screen(px, py), joint],
                        Stroke::new(2.0, theme::ACCENT.gamma_multiply(0.7)),
                    );
                }
                painter.circle_filled(joint, 5.0, theme::ACCENT);
                painter.circle_stroke(joint, 5.0, Stroke::new(1.5, theme::BG));
                painter.text(
                    joint + vec2(8.0, -8.0),
                    Align2::LEFT_BOTTOM,
                    &b.name,
                    theme::mono(10.5),
                    theme::ACCENT,
                );
            }
        }
    }

    /// Sin esqueleto: qué hace falta y un botón que deja el encargo escrito.
    fn vanim_empty(&mut self, ui: &mut egui::Ui, full: Rect, out: &mut Vec<Intent>) {
        let has_drawing = self.vec.view.is_some();
        ui.scope_builder(
            UiBuilder::new().max_rect(Rect::from_center_size(full.center(), vec2(520.0, 240.0))),
            |ui| {
                ui.vertical_centered(|ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;
                    ui.label(
                        RichText::new(if has_drawing {
                            "El dibujo aún no tiene esqueleto"
                        } else {
                            "Preparando el dibujo…"
                        })
                        .font(theme::semibold(16.0))
                        .color(theme::TEXT),
                    );
                    if !has_drawing {
                        return;
                    }
                    ui.label(
                        RichText::new(
                            "Para animarlo, las partes que se mueven van en piezas con su hueso y su pivote (la bisagra de una tapa, el hombro de un brazo). El agente lo prepara: pídele la animación que quieras.",
                        )
                        .size(13.0)
                        .color(theme::TEXT_2),
                    );
                    let ask = egui::Button::new(
                        RichText::new("Pedir al agente que lo prepare")
                            .font(theme::semibold(13.0))
                            .color(theme::ACCENT_INK),
                    )
                    .fill(theme::ACCENT)
                    .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
                    .min_size(vec2(0.0, 34.0));
                    if ui.add(ask).clicked() {
                        out.push(Intent::AgentDraft(
                            "Prepara el dibujo para animarlo y haz una acción: ".into(),
                        ));
                    }
                });
            },
        );
    }
}
