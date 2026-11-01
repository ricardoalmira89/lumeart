//! Modo animación (`docs/diseno/capturas/04-animacion-editor.png`): panel de
//! acciones con la base y las comprobaciones, escenario con papel cebolla,
//! suelo y pivote, transporte, línea de tiempo con una pista por pieza
//! (rombos donde cambia) y la duración de cada fotograma, que se cambia con
//! un comando reversible.
//!
//! Es un hijo de `shell` (comparte su cabecera, su panel del agente y sus
//! estilos). El `AnimResult` lo calcula `StudioApp` en segundo plano y lo
//! deja aquí con [`AnimMode::set_result`].

use std::collections::HashMap;
use std::sync::Arc;

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Panel, Pos2, Rect,
    RichText, Sense, Shape, Stroke, StrokeKind, TextureHandle, TextureOptions, UiBuilder, pos2,
    vec2,
};
use lume_app::ViewModel;
use lume_domain::anim::animation::Animation;
use lume_domain::anim::export::{ExportFormat, sheet_file_name};
use lume_domain::anim::sheet::sheet;
use lume_domain::anim::{Action, AnimResult, Image, Level, Track, tracks};

use super::{AppShell, bar_frame, option_button, panel_label, vsep};
use crate::anim_player::Player;
use crate::intent::Intent;
use crate::theme;

/// Ancho del panel de acciones.
pub const ACTIONS_WIDTH: f32 = 248.0;
/// Alto del transporte y de la línea de tiempo.
pub const TRANSPORT_HEIGHT: f32 = 48.0;
pub const TIMELINE_HEIGHT: f32 = 226.0;
/// Columna de nombres y columna de cada fotograma de la línea de tiempo.
const NAME_COLUMN: f32 = 132.0;
const FRAME_COLUMN: f32 = 100.0;
const THUMBS_ROW: f32 = 92.0;
const TRACK_ROW: f32 = 25.0;
const DURATION_ROW: f32 = 28.0;

/// Colores de los puntos de las pistas.
const TRACK_COLORS: [Color32; 6] = [
    Color32::from_rgb(0xE8, 0x5D, 0x5D),
    Color32::from_rgb(0xE8, 0xE4, 0xD8),
    Color32::from_rgb(0xC0, 0x8A, 0x5A),
    Color32::from_rgb(0x7A, 0xB8, 0xFF),
    Color32::from_rgb(0xB6, 0x8C, 0xFF),
    Color32::from_rgb(0x5B, 0xD6, 0x9A),
];

/// En qué punto está una variante en pantalla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantCardState {
    Running,
    Ready,
    Failed(String),
}

/// Una variante en pantalla.
#[derive(Debug, Clone)]
pub struct VariantCard {
    pub letter: char,
    pub mix: bool,
    pub state: VariantCardState,
    /// Lo que dijo el agente.
    pub summary: String,
    /// Su `anim.js` (si está lista) y cómo se dibuja.
    pub source: Option<String>,
    pub result: Option<Arc<AnimResult>>,
    /// Cambia cada vez que llega un `result` (para las texturas).
    pub id: u64,
}

/// El diálogo «Exportar animación».
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimExport {
    pub format: ExportFormat,
    pub scale: u32,
    pub file_name: String,
    pub open_folder: bool,
    /// El usuario cambió el nombre (ya no sigue a la escala).
    edited: bool,
}

/// Estado del modo animación.
#[derive(Clone, Default)]
pub struct AnimMode {
    /// La animación dibujada (la del documento ahora o la anterior mientras
    /// se recalcula).
    pub result: Option<Arc<AnimResult>>,
    /// Acción elegida.
    pub action: usize,
    pub player: Player,
    pub onion: bool,
    pub ground: bool,
    pub pivot: bool,
    /// No hay motor de animación en esta ventana.
    pub no_engine: bool,
    /// Fotograma cuya duración se está escribiendo, con el texto (y si
    /// acaba de empezar, para seleccionarlo entero).
    editing: Option<(usize, String, bool)>,
    last_time: Option<f64>,
    generation: u64,
    textures: HashMap<(u64, usize, usize), TextureHandle>,
    /// Variantes pedidas y si se están viendo.
    pub variants: Vec<VariantCard>,
    pub showing_variants: bool,
    mix_text: String,
    /// El diálogo de exportar, si está abierto.
    pub export: Option<AnimExport>,
}

impl std::fmt::Debug for AnimMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimMode")
            .field("action", &self.action)
            .field("player", &self.player)
            .finish_non_exhaustive()
    }
}

/// Clave de textura de la base (fuera de las acciones).
const BASE: usize = usize::MAX;

impl AnimMode {
    /// Pone una animación recién dibujada (conserva la acción y el
    /// fotograma si siguen existiendo).
    pub fn set_result(&mut self, result: AnimResult) {
        self.generation += 1;
        self.textures.clear();
        if self.action >= result.actions.len() {
            self.action = 0;
            self.player = Player::default();
        }
        let looping = result.actions.get(self.action).map(|a| a.spec.looping);
        let frames = result
            .actions
            .get(self.action)
            .map_or(0, |a| a.images.len());
        if self.player.frame >= frames {
            self.player.seek(0);
        }
        if self.result.is_none()
            && let Some(looping) = looping
        {
            self.player.looping = looping;
        }
        self.result = Some(Arc::new(result));
    }

    /// Abre el diálogo de exportar (con la escala de la última vez).
    pub fn open_export(&mut self) {
        let Some(result) = &self.result else {
            return;
        };
        let scale = self.export.as_ref().map_or(1, |e| e.scale);
        self.export = Some(AnimExport {
            format: ExportFormat::Sheet,
            scale,
            file_name: sheet_file_name(result, scale),
            open_folder: true,
            edited: false,
        });
    }

    /// Nombre de la acción que se ve.
    pub fn action_name(&self) -> Option<String> {
        self.current().map(|a| a.spec.name.clone())
    }

    /// Sin animación.
    pub fn clear(&mut self) {
        self.result = None;
        self.textures.clear();
        self.player = Player::default();
        self.action = 0;
    }

    /// La acción elegida.
    pub fn current(&self) -> Option<&Action> {
        self.result.as_ref()?.actions.get(self.action)
    }

    /// Elige una acción (y vuelve a su primer fotograma, con su bucle).
    pub fn select(&mut self, index: usize) {
        if self.action == index {
            return;
        }
        self.action = index;
        let looping = self
            .result
            .as_ref()
            .and_then(|r| r.actions.get(index))
            .is_none_or(|a| a.spec.looping);
        let pingpong = self.player.pingpong;
        self.player = Player::new(looping);
        self.player.pingpong = pingpong;
        self.editing = None;
    }

    /// Duraciones de la acción elegida.
    pub fn durations(&self, animation: Option<&Animation>) -> Vec<u32> {
        let Some(action) = self.current() else {
            return Vec::new();
        };
        animation
            .map(|a| a.durations_for(&action.spec))
            .unwrap_or_else(|| Animation::default().durations_for(&action.spec))
    }

    /// Avanza la reproducción con el reloj de la ventana y pide repintar a
    /// tiempo.
    fn tick(&mut self, ctx: &egui::Context, durations: &[u32]) {
        let now = ctx.input(|i| i.time);
        let dt = self.last_time.map_or(0.0, |t| (now - t) * 1000.0);
        self.last_time = Some(now);
        self.player.advance(dt.max(0.0), durations);
        if let Some(ms) = self.player.remaining_ms(durations) {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(ms / 1000.0));
        }
    }

    fn texture(
        &mut self,
        ctx: &egui::Context,
        key: usize,
        frame: usize,
        img: &Image,
    ) -> TextureHandle {
        let generation = self.generation;
        self.textures
            .entry((generation, key, frame))
            .or_insert_with(|| {
                let bytes: Vec<u8> = img
                    .pixels()
                    .iter()
                    .flat_map(|c| [c.r, c.g, c.b, c.a])
                    .collect();
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [img.width() as usize, img.height() as usize],
                    &bytes,
                );
                ctx.load_texture(
                    format!("anim-{generation}-{key}-{frame}"),
                    image,
                    TextureOptions::NEAREST,
                )
            })
            .clone()
    }
}

/// «0,65 s en bucle».
fn total_text(durations: &[u32], looping: bool) -> String {
    let ms: u32 = durations.iter().sum();
    let secs = format!("{:.2}", f64::from(ms) / 1000.0).replace('.', ",");
    format!("{secs} s {}", if looping { "en bucle" } else { "una vez" })
}

/// Dibuja `tex` dentro de `rect` (escalado entero, centrado) sobre cuadros.
fn checkered_image(
    painter: &egui::Painter,
    rect: Rect,
    tex: Option<&TextureHandle>,
    size: (u32, u32),
) -> Rect {
    let (w, h) = (size.0.max(1) as f32, size.1.max(1) as f32);
    let k = (rect.width() / w).min(rect.height() / h);
    let k = if k >= 1.0 { k.floor() } else { k };
    let image = Rect::from_center_size(rect.center(), vec2(w * k, h * k));
    let cell = (k * 2.0).max(4.0);
    let mut y = image.top();
    let mut row = 0;
    while y < image.bottom() {
        let mut x = image.left();
        let mut col = 0;
        while x < image.right() {
            let c = if (row + col) % 2 == 0 {
                theme::CHECKER_A
            } else {
                theme::CHECKER_B
            };
            let r = Rect::from_min_max(
                pos2(x, y),
                pos2(
                    (x + cell).min(image.right()),
                    (y + cell).min(image.bottom()),
                ),
            );
            painter.rect_filled(r, 0.0, c);
            x += cell;
            col += 1;
        }
        y += cell;
        row += 1;
    }
    if let Some(tex) = tex {
        painter.image(
            tex.id(),
            image,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    image
}

/// Rombo de la línea de tiempo.
fn diamond(painter: &egui::Painter, center: Pos2, r: f32, color: Color32) {
    painter.add(Shape::convex_polygon(
        vec![
            center + vec2(0.0, -r),
            center + vec2(r, 0.0),
            center + vec2(0.0, r),
            center + vec2(-r, 0.0),
        ],
        color,
        Stroke::NONE,
    ));
}

/// Botón del transporte dibujado a mano (sin glifos que falten).
fn transport_button(ui: &mut egui::Ui, label: &str, kind: u8, primary: bool) -> egui::Response {
    let size = if primary { 36.0 } else { 28.0 };
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let response = response.on_hover_text(label);
    let painter = ui.painter_at(rect);
    let c = rect.center();
    let fg = if primary {
        painter.circle_filled(c, size / 2.0, theme::ACCENT);
        theme::ACCENT_INK
    } else {
        if response.hovered() {
            painter.rect_filled(rect, CornerRadius::same(theme::RADIUS_BUTTON), theme::HOVER);
        }
        theme::TEXT_2
    };
    let tri = |dir: f32, dx: f32| {
        Shape::convex_polygon(
            vec![
                c + vec2(dx - 4.0 * dir, -5.0),
                c + vec2(dx + 5.0 * dir, 0.0),
                c + vec2(dx - 4.0 * dir, 5.0),
            ],
            fg,
            Stroke::NONE,
        )
    };
    let bar = |dx: f32| Rect::from_center_size(c + vec2(dx, 0.0), vec2(2.0, 10.0));
    match kind {
        // Primero, anterior, siguiente, último.
        0 => {
            painter.rect_filled(bar(-5.0), 0.0, fg);
            painter.add(tri(-1.0, 1.0));
        }
        1 => {
            painter.add(tri(-1.0, 0.0));
        }
        3 => {
            painter.add(tri(1.0, 0.0));
        }
        4 => {
            painter.add(tri(1.0, -1.0));
            painter.rect_filled(bar(5.0), 0.0, fg);
        }
        // Reproducir y pausa.
        5 => {
            painter.rect_filled(bar(-3.0), 0.0, fg);
            painter.rect_filled(bar(3.0), 0.0, fg);
        }
        _ => {
            painter.add(tri(1.0, 1.0));
        }
    }
    response
}

impl AppShell {
    /// El armazón en modo animación.
    pub(crate) fn show_animation(
        &mut self,
        ui: &mut egui::Ui,
        vm: &ViewModel,
        out: &mut Vec<Intent>,
    ) {
        let animation = vm.animation.as_deref();
        let durations = self.anim.durations(animation);
        self.anim.tick(ui.ctx(), &durations);

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

        let status = Panel::bottom("estado")
            .exact_size(theme::STATUS_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::symmetric(14, 0)))
            .show(ui, |ui| self.anim_status(ui, &durations));
        self.zones.status = status.response.rect;

        let agent = Panel::right("agente")
            .exact_size(theme::AGENT_WIDTH)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::same(16)))
            .show(ui, |ui| self.agent_panel(ui, out, None));
        self.zones.agent = agent.response.rect;

        let actions = Panel::left("acciones")
            .exact_size(ACTIONS_WIDTH)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::symmetric(14, 14)))
            .show(ui, |ui| self.actions_panel(ui, vm, animation, out));
        self.zones.toolbar = actions.response.rect;

        if self.anim.export.is_some() {
            self.anim_export_dialog(ui.ctx(), out);
        }

        if self.anim.showing_variants && !self.anim.variants.is_empty() {
            self.variants_view(ui, out);
            return;
        }

        self.tab_panel(ui, out);
        let options = Panel::top("opciones")
            .exact_size(theme::OPTIONS_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR_ALT).inner_margin(Margin::symmetric(14, 0)))
            .show(ui, |ui| self.anim_options(ui));
        self.zones.options = options.response.rect;

        if self.anim.current().is_some() {
            let bottom = Panel::bottom("linea-de-tiempo")
                .exact_size(TRANSPORT_HEIGHT + TIMELINE_HEIGHT)
                .resizable(false)
                .frame(bar_frame(theme::BAR))
                .show(ui, |ui| {
                    self.transport(ui, &durations);
                    self.timeline(ui, animation, &durations, out);
                });
            self.zones.bottom = bottom.response.rect;
        } else {
            self.zones.bottom = Rect::NOTHING;
        }

        let stage = egui::CentralPanel::default()
            .frame(Frame::NONE.fill(theme::BG))
            .show(ui, |ui| self.stage(ui, vm, &durations, out));
        self.zones.canvas = stage.response.rect;
    }

    fn anim_status(&mut self, ui: &mut egui::Ui, durations: &[u32]) {
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 18.0;
            let font = theme::mono(11.5);
            let text = |s: String| RichText::new(s).font(font.clone()).color(theme::TEXT_3);
            if let (Some(action), Some(res)) = (self.anim.current(), self.anim.result.as_ref()) {
                let f = self.anim.player.frame;
                let n = action.images.len();
                // Sitio fijo: al reproducir, lo de detrás no baila.
                let nines = "9".repeat(super::digits(n as u64));
                super::fixed_label(
                    ui,
                    text(format!(
                        "fotograma {}/{n} · {} ms",
                        f + 1,
                        durations.get(f).copied().unwrap_or(0)
                    )),
                    &font,
                    &[&format!("fotograma {nines}/{nines} · 9999 ms")],
                );
                ui.label(text(format!("celda {} × {}", res.cell.0, res.cell.1)));
                let frames: usize = res.actions.iter().map(|a| a.images.len()).sum();
                ui.label(text(format!(
                    "{} acciones · {frames} fotogramas",
                    res.actions.len()
                )));
            }
            if let Some(notice) = &self.notice {
                ui.label(RichText::new(notice).size(11.5).color(theme::TEXT_3));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(text("anim.js".into()));
            });
        });
    }

    fn actions_panel(
        &mut self,
        ui: &mut egui::Ui,
        vm: &ViewModel,
        animation: Option<&Animation>,
        out: &mut Vec<Intent>,
    ) {
        let ctx = ui.ctx().clone();
        let full = ui.max_rect();
        panel_label(ui, "Dibujo base");
        ui.add_space(8.0);
        let base = animation
            .and_then(|a| a.base.clone())
            .unwrap_or_else(|| Image::from_canvas(&vm.canvas));
        let base_tex = self.anim.texture(&ctx, BASE, 0, &base);
        let parts = self.anim.result.as_ref().map_or(0, |r| r.parts.len());
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
            checkered_image(ui.painter(), rect, Some(&base_tex), base.size());
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(
                        self.file_name
                            .clone()
                            .unwrap_or_else(|| format!("{}.lume", vm.name)),
                    )
                    .size(13.0)
                    .color(theme::TEXT),
                );
                ui.label(
                    RichText::new(match parts {
                        1 => "1 pieza".to_string(),
                        n => format!("{n} piezas"),
                    })
                    .size(12.0)
                    .color(theme::TEXT_3),
                );
            });
        });
        ui.add_space(14.0);
        let Some(result) = self.anim.result.clone() else {
            return;
        };
        ui.horizontal(|ui| {
            panel_label(ui, "Acciones");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(result.actions.len().to_string())
                        .size(11.0)
                        .color(theme::TEXT_3),
                );
            });
        });
        ui.add_space(6.0);
        let checks_height = 170.0;
        let list_height = (full.bottom() - ui.cursor().top() - checks_height).max(80.0);
        egui::ScrollArea::vertical()
            .id_salt("acciones")
            .max_height(list_height)
            .show(ui, |ui| {
                for (i, action) in result.actions.iter().enumerate() {
                    let selected = i == self.anim.action;
                    let (rect, response) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 66.0), Sense::click());
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::Role::Button,
                            true,
                            selected,
                            &action.spec.name,
                        )
                    });
                    let painter = ui.painter_at(rect);
                    let (fill, stroke) = if selected {
                        (theme::ACCENT_SOFT, Stroke::new(1.0, theme::ACCENT))
                    } else if response.hovered() {
                        (theme::HOVER, Stroke::NONE)
                    } else {
                        (Color32::TRANSPARENT, Stroke::NONE)
                    };
                    painter.rect(
                        rect,
                        CornerRadius::same(theme::RADIUS_CARD_SMALL),
                        fill,
                        stroke,
                        StrokeKind::Inside,
                    );
                    if let Some(img) = action.images.first() {
                        let tex = self.anim.texture(&ctx, i, 0, img);
                        let thumb =
                            Rect::from_min_size(rect.min + vec2(8.0, 11.0), vec2(44.0, 44.0));
                        checkered_image(&painter, thumb, Some(&tex), img.size());
                    }
                    painter.text(
                        rect.min + vec2(62.0, 14.0),
                        Align2::LEFT_TOP,
                        &action.spec.name,
                        FontId::proportional(13.0),
                        theme::TEXT,
                    );
                    painter.text(
                        rect.min + vec2(62.0, 34.0),
                        Align2::LEFT_TOP,
                        format!(
                            "{} fotogramas · {} fps",
                            action.spec.frames, action.spec.fps
                        ),
                        FontId::proportional(11.5),
                        theme::TEXT_3,
                    );
                    let chip = if action.spec.looping {
                        "bucle"
                    } else {
                        "una vez"
                    };
                    let chip_rect = Rect::from_min_size(
                        pos2(rect.right() - 62.0, rect.top() + 12.0),
                        vec2(54.0, 18.0),
                    );
                    painter.rect_filled(chip_rect, CornerRadius::same(9), theme::FIELD);
                    painter.text(
                        chip_rect.center(),
                        Align2::CENTER_CENTER,
                        chip,
                        FontId::proportional(11.0),
                        theme::TEXT_2,
                    );
                    if response.clicked() {
                        self.anim.select(i);
                    }
                    ui.add_space(4.0);
                }
                let new_action = ui.add(
                    egui::Button::new(
                        RichText::new("Nueva acción")
                            .size(12.5)
                            .color(theme::TEXT_2),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                    .corner_radius(CornerRadius::same(theme::RADIUS_CARD_SMALL))
                    .min_size(vec2(ui.available_width(), 34.0)),
                );
                if new_action.on_hover_text("Pídesela al agente").clicked() {
                    out.push(Intent::AgentDraft(
                        "Añade a anim.js una acción nueva: ".into(),
                    ));
                }
            });

        // Comprobaciones al pie.
        let checks = Rect::from_min_max(pos2(full.left(), full.bottom() - checks_height), full.max);
        ui.scope_builder(UiBuilder::new().max_rect(checks), |ui| {
            ui.painter().hline(
                checks.x_range(),
                checks.top() - 6.0,
                Stroke::new(1.0, theme::LINE),
            );
            panel_label(ui, "Comprobaciones");
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .id_salt("comprobaciones")
                .show(ui, |ui| {
                    let mut passed = Vec::new();
                    let has = |text: &str| result.issues.iter().any(|i| i.msg.contains(text));
                    if result.base.is_some() && result.rest.is_some() && !has("la pose de reposo") {
                        passed.push("El reposo coincide con la base");
                    }
                    if result.base.is_some() && !has("colores que no están en la base") {
                        passed.push("Solo colores de la paleta");
                    }
                    for text in passed {
                        ui.horizontal(|ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                            let c = rect.center();
                            ui.painter().line_segment(
                                [c + vec2(-4.0, 0.0), c + vec2(-1.0, 3.0)],
                                Stroke::new(1.6, theme::OK),
                            );
                            ui.painter().line_segment(
                                [c + vec2(-1.0, 3.0), c + vec2(4.0, -3.0)],
                                Stroke::new(1.6, theme::OK),
                            );
                            ui.label(RichText::new(text).size(12.0).color(theme::TEXT_BODY));
                        });
                    }
                    for issue in &result.issues {
                        let (color, fill) = match issue.level {
                            Level::Error => (theme::DANGER, theme::DANGER_BG),
                            Level::Warning => (
                                theme::WARN,
                                Color32::from_rgba_unmultiplied(0xF5, 0xB8, 0x41, 16),
                            ),
                        };
                        Frame::NONE
                            .fill(fill)
                            .stroke(Stroke::new(1.0, color.gamma_multiply(0.5)))
                            .corner_radius(CornerRadius::same(theme::RADIUS_CARD_SMALL))
                            .inner_margin(Margin::same(10))
                            .show(ui, |ui| {
                                let place = match (issue.line, issue.place.as_str()) {
                                    (0, "") => String::new(),
                                    (0, p) => format!("{p}: "),
                                    (l, "") => format!("línea {l}: "),
                                    (l, p) => format!("{p} (línea {l}): "),
                                };
                                ui.label(
                                    RichText::new(format!("{place}{}", issue.msg))
                                        .size(12.0)
                                        .color(color),
                                );
                                if ui
                                    .button(RichText::new("Pedir arreglo al agente").size(12.0))
                                    .clicked()
                                {
                                    out.push(Intent::AgentDraft(format!(
                                        "Arregla esto de anim.js: {place}{}",
                                        issue.msg
                                    )));
                                }
                            });
                        ui.add_space(6.0);
                    }
                });
        });
    }

    fn anim_options(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            let onion = option_button(ui, "      Papel cebolla", self.anim.onion);
            let c = onion.rect.left_center() + vec2(12.0, 0.0);
            ui.painter().rect_filled(
                Rect::from_center_size(c, vec2(7.0, 7.0)),
                2.0,
                theme::ONION_PREV.to_opaque(),
            );
            ui.painter().rect_filled(
                Rect::from_center_size(c + vec2(9.0, 0.0), vec2(7.0, 7.0)),
                2.0,
                theme::ONION_NEXT.to_opaque(),
            );
            if onion.clicked() {
                self.anim.onion = !self.anim.onion;
            }
            if option_button(ui, "Suelo", self.anim.ground).clicked() {
                self.anim.ground = !self.anim.ground;
            }
            if option_button(ui, "Pivote", self.anim.pivot).clicked() {
                self.anim.pivot = !self.anim.pivot;
            }
            vsep(ui);
        });
    }

    fn transport(&mut self, ui: &mut egui::Ui, durations: &[u32]) {
        let frames = durations.len();
        let rect = ui
            .allocate_exact_size(vec2(ui.available_width(), TRANSPORT_HEIGHT), Sense::hover())
            .0;
        ui.painter().hline(
            rect.x_range(),
            rect.bottom() - 0.5,
            Stroke::new(1.0, theme::LINE),
        );
        let fps = self.anim.current().map_or(0, |a| a.spec.fps);
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(rect.shrink2(vec2(16.0, 0.0)))
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let player = &mut self.anim.player;
                if transport_button(ui, "Primer fotograma", 0, false).clicked() {
                    player.first();
                }
                if transport_button(ui, "Fotograma anterior", 1, false).clicked() {
                    player.prev(frames);
                }
                let (label, kind) = if player.playing {
                    ("Pausa", 5)
                } else {
                    ("Reproducir", 2)
                };
                if transport_button(ui, label, kind, true).clicked() {
                    player.toggle(frames);
                    self.anim.last_time = None;
                }
                let player = &mut self.anim.player;
                if transport_button(ui, "Fotograma siguiente", 3, false).clicked() {
                    player.next(frames);
                }
                if transport_button(ui, "Último fotograma", 4, false).clicked() {
                    player.last(frames);
                }
                vsep(ui);
                if option_button(ui, "Bucle", player.looping).clicked() {
                    player.looping = !player.looping;
                }
                if option_button(ui, "Ida y vuelta", player.pingpong).clicked() {
                    player.pingpong = !player.pingpong;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let speed = match player.speed {
                        s if s > 1.5 => "2×",
                        s if s < 0.75 => "½×",
                        _ => "1×",
                    };
                    if option_button(ui, speed, false)
                        .on_hover_text("Velocidad de reproducción")
                        .clicked()
                    {
                        player.speed = match speed {
                            "1×" => 2.0,
                            "2×" => 0.5,
                            _ => 1.0,
                        };
                    }
                    ui.label(
                        RichText::new(format!("{fps} fps"))
                            .font(theme::mono(12.0))
                            .color(theme::TEXT),
                    );
                    ui.label(RichText::new("Velocidad").size(12.0).color(theme::TEXT_3));
                });
            },
        );
    }

    fn timeline(
        &mut self,
        ui: &mut egui::Ui,
        animation: Option<&Animation>,
        durations: &[u32],
        out: &mut Vec<Intent>,
    ) {
        let ctx = ui.ctx().clone();
        let (Some(result), Some(action)) = (self.anim.result.clone(), self.anim.current().cloned())
        else {
            return;
        };
        let tracks: Vec<Track> = tracks(&result, &action);
        let frames = action.images.len();
        let current = self.anim.player.frame;
        let default = lume_domain::anim::animation::default_duration(action.spec.fps);
        let height = THUMBS_ROW + TRACK_ROW * tracks.len() as f32 + DURATION_ROW + 8.0;
        let width = NAME_COLUMN + FRAME_COLUMN * frames as f32 + 40.0;
        egui::ScrollArea::both()
            .id_salt("linea-de-tiempo")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let (area, _) = ui.allocate_exact_size(
                    vec2(width.max(ui.available_width()), height),
                    Sense::hover(),
                );
                let painter = ui.painter_at(area);
                let col = |f: usize| {
                    Rect::from_min_size(
                        pos2(
                            area.left() + NAME_COLUMN + FRAME_COLUMN * f as f32,
                            area.top(),
                        ),
                        vec2(FRAME_COLUMN, height),
                    )
                };
                // Columna actual.
                let now = col(current);
                painter.rect_filled(now, 0.0, theme::ACCENT_SOFT);
                painter.hline(
                    now.x_range(),
                    now.top() + 1.0,
                    Stroke::new(2.0, theme::ACCENT),
                );
                // Nombre y duración total.
                ui.put(
                    Rect::from_min_size(
                        area.min + vec2(14.0, THUMBS_ROW - 36.0),
                        vec2(NAME_COLUMN - 20.0, 18.0),
                    ),
                    egui::Label::new(
                        RichText::new(&action.spec.name)
                            .font(theme::semibold(13.0))
                            .color(theme::TEXT),
                    )
                    .truncate(),
                );
                ui.put(
                    Rect::from_min_size(
                        area.min + vec2(14.0, THUMBS_ROW - 18.0),
                        vec2(NAME_COLUMN - 20.0, 16.0),
                    ),
                    egui::Label::new(
                        RichText::new(total_text(durations, action.spec.looping))
                            .size(11.5)
                            .color(theme::TEXT_3),
                    ),
                );
                // Miniaturas y números.
                for (f, img) in action.images.iter().enumerate() {
                    let c = col(f);
                    let thumb = Rect::from_min_size(
                        pos2(c.center().x - 32.0, c.top() + 8.0),
                        vec2(64.0, 64.0),
                    );
                    let tex = self.anim.texture(&ctx, self.anim.action, f, img);
                    checkered_image(&painter, thumb, Some(&tex), img.size());
                    let number = Rect::from_min_size(
                        pos2(c.left(), thumb.bottom() + 2.0),
                        vec2(FRAME_COLUMN, 16.0),
                    );
                    let response = ui.interact(
                        Rect::from_min_max(c.min, number.max),
                        ui.scope_id().with(("fotograma", f)),
                        Sense::click(),
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::Role::Button,
                            true,
                            f == current,
                            format!("Fotograma {}", f + 1),
                        )
                    });
                    if response.clicked() {
                        self.anim.player.playing = false;
                        self.anim.player.seek(f);
                    }
                    painter.text(
                        number.center(),
                        Align2::CENTER_CENTER,
                        (f + 1).to_string(),
                        FontId::monospace(11.5),
                        if f == current {
                            theme::ACCENT
                        } else {
                            theme::TEXT_3
                        },
                    );
                }
                // Una pista por pieza: rombos donde cambia.
                for (i, track) in tracks.iter().enumerate() {
                    let y = area.top() + THUMBS_ROW + TRACK_ROW * i as f32;
                    let row = Rect::from_min_size(pos2(area.left(), y), vec2(width, TRACK_ROW));
                    painter.hline(row.x_range(), row.top(), Stroke::new(1.0, theme::LINE_SOFT));
                    let dot = TRACK_COLORS[i % TRACK_COLORS.len()];
                    painter.circle_filled(pos2(row.left() + 18.0, row.center().y), 3.0, dot);
                    painter.text(
                        pos2(row.left() + 28.0, row.center().y),
                        Align2::LEFT_CENTER,
                        &track.part,
                        FontId::proportional(12.0),
                        theme::TEXT_BODY,
                    );
                    let first = track.drawn.iter().position(|d| *d).unwrap_or(0);
                    let last = track.drawn.iter().rposition(|d| *d).unwrap_or(0);
                    painter.hline(
                        col(first).center().x..=col(last).center().x,
                        row.center().y,
                        Stroke::new(1.0, theme::LINE_STRONG),
                    );
                    for f in 0..frames {
                        if !track.keys[f] {
                            continue;
                        }
                        let center = pos2(col(f).center().x, row.center().y);
                        let color = if f == current {
                            theme::ACCENT
                        } else {
                            theme::TEXT
                        };
                        diamond(&painter, center, 5.0, color);
                        let hit = Rect::from_center_size(center, vec2(14.0, 14.0));
                        let response =
                            ui.interact(hit, ui.scope_id().with(("rombo", i, f)), Sense::hover());
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::Role::Label,
                                true,
                                format!("{}: cambia en el fotograma {}", track.part, f + 1),
                            )
                        });
                    }
                }
                // Duración de cada fotograma (editable).
                let y = area.top() + THUMBS_ROW + TRACK_ROW * tracks.len() as f32;
                let row = Rect::from_min_size(pos2(area.left(), y), vec2(width, DURATION_ROW));
                painter.hline(row.x_range(), row.top(), Stroke::new(1.0, theme::LINE_SOFT));
                painter.text(
                    pos2(row.left() + 14.0, row.center().y),
                    Align2::LEFT_CENTER,
                    "Duración",
                    FontId::proportional(12.0),
                    theme::TEXT_3,
                );
                for (f, ms) in durations.iter().enumerate() {
                    let cell = Rect::from_center_size(
                        pos2(col(f).center().x, row.center().y),
                        vec2(76.0, 22.0),
                    );
                    let label = format!("Duración del fotograma {}", f + 1);
                    match &mut self.anim.editing {
                        Some((editing, text, fresh)) if *editing == f => {
                            let edit = ui.put(
                                cell,
                                egui::TextEdit::singleline(text)
                                    .font(theme::mono(12.0))
                                    .align(Align2::CENTER_CENTER),
                            );
                            edit.widget_info(|| {
                                egui::WidgetInfo::labeled(egui::Role::TextInput, true, &label)
                            });
                            if *fresh {
                                // Al empezar, todo seleccionado: lo que se escribe lo sustituye.
                                edit.request_focus();
                                if let Some(mut state) =
                                    egui::TextEdit::load_state(ui.ctx(), edit.id)
                                {
                                    let all = egui::text::CCursorRange::two(
                                        egui::text::CCursor::new(0),
                                        egui::text::CCursor::new(text.chars().count()),
                                    );
                                    state.cursor.set_char_range(Some(all));
                                    state.store(ui.ctx(), edit.id);
                                }
                                *fresh = false;
                            }
                            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
                            if escape {
                                self.anim.editing = None;
                            } else if edit.lost_focus() || enter {
                                let typed =
                                    text.trim().trim_end_matches("ms").trim().parse::<u32>();
                                self.anim.editing = None;
                                match typed {
                                    Ok(value) if value != *ms => {
                                        out.push(Intent::SetFrameDuration {
                                            action: action.spec.clone(),
                                            frame: f as u32,
                                            ms: value,
                                        })
                                    }
                                    Ok(_) => {}
                                    Err(_) => {
                                        self.notice = Some(
                                            "La duración tiene que ser un número de milisegundos"
                                                .into(),
                                        )
                                    }
                                }
                            }
                        }
                        _ => {
                            let response = ui.interact(
                                cell,
                                ui.scope_id().with(("duracion", f)),
                                Sense::click(),
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(egui::Role::Button, true, &label)
                            });
                            let highlighted = *ms != default;
                            if response.hovered() {
                                painter.rect_filled(
                                    cell,
                                    CornerRadius::same(theme::RADIUS_FIELD),
                                    theme::HOVER,
                                );
                            }
                            painter.text(
                                cell.center(),
                                Align2::CENTER_CENTER,
                                format!("{ms} ms"),
                                if highlighted {
                                    theme::mono(12.0)
                                } else {
                                    FontId::monospace(11.5)
                                },
                                if highlighted {
                                    theme::TEXT
                                } else {
                                    theme::TEXT_3
                                },
                            );
                            if response.on_hover_text("Clic para cambiarla").clicked() {
                                self.anim.editing = Some((f, ms.to_string(), true));
                            }
                        }
                    }
                }
            });
        let _ = animation;
    }

    fn stage(
        &mut self,
        ui: &mut egui::Ui,
        vm: &ViewModel,
        durations: &[u32],
        out: &mut Vec<Intent>,
    ) {
        let ctx = ui.ctx().clone();
        let full = ui.max_rect();
        if vm.animation.is_none() {
            ui.scope_builder(UiBuilder::new().max_rect(Rect::from_center_size(full.center(), vec2(420.0, 180.0))), |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Este dibujo aún no tiene animación").font(theme::semibold(16.0)).color(theme::TEXT));
                    ui.add_space(6.0);
                    ui.label(RichText::new("LumeArt la prepara a partir del dibujo: su receta queda como una pieza y el reposo lo reproduce. Después, pídele al agente las acciones o edita anim.js.").size(12.5).color(theme::TEXT_3));
                    ui.add_space(12.0);
                    let create = egui::Button::new(RichText::new("Crear animación").color(theme::ACCENT_INK)).fill(theme::ACCENT).min_size(vec2(0.0, 32.0));
                    if ui.add(create).clicked() {
                        out.push(Intent::CreateAnimation);
                    }
                });
            });
            return;
        }
        if self.anim.no_engine {
            ui.put(
                full,
                egui::Label::new(
                    RichText::new("El motor de animación no está disponible en esta ventana.")
                        .color(theme::TEXT_3),
                ),
            );
            return;
        }
        let (Some(result), Some(action)) = (self.anim.result.clone(), self.anim.current().cloned())
        else {
            let text = if self.anim.result.is_some() {
                "anim.js no define ninguna acción todavía."
            } else {
                "Dibujando la animación…"
            };
            ui.put(
                full,
                egui::Label::new(RichText::new(text).color(theme::TEXT_3)),
            );
            return;
        };
        let f = self
            .anim
            .player
            .frame
            .min(action.images.len().saturating_sub(1));
        let frames = action.images.len();
        // Etiqueta del fotograma.
        let label = format!(
            "{} · fotograma {} de {} · {} ms",
            action.spec.name,
            f + 1,
            frames,
            durations.get(f).copied().unwrap_or(0)
        );
        let tag = Rect::from_min_size(
            full.min + vec2(16.0, 14.0),
            vec2(8.0 * label.chars().count() as f32 * 0.82 + 24.0, 28.0),
        );
        let painter = ui.painter_at(full);
        painter.rect(
            tag,
            CornerRadius::same(theme::RADIUS_BUTTON),
            theme::CARD,
            Stroke::new(1.0, theme::LINE_STRONG),
            StrokeKind::Inside,
        );
        painter.text(
            tag.left_center() + vec2(12.0, 0.0),
            Align2::LEFT_CENTER,
            &label,
            FontId::proportional(12.0),
            theme::TEXT,
        );
        // El fotograma, ampliado sobre cuadros, con papel cebolla.
        let area = Rect::from_min_max(full.min + vec2(40.0, 56.0), full.max - vec2(40.0, 24.0));
        let tex = self
            .anim
            .texture(&ctx, self.anim.action, f, &action.images[f]);
        let image = checkered_image(&painter, area, None, result.cell);
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        if self.anim.onion && frames > 1 {
            let prev = (f + frames - 1) % frames;
            let next = (f + 1) % frames;
            let prev_tex = self
                .anim
                .texture(&ctx, self.anim.action, prev, &action.images[prev]);
            let next_tex = self
                .anim
                .texture(&ctx, self.anim.action, next, &action.images[next]);
            painter.image(prev_tex.id(), image, uv, theme::ONION_PREV);
            painter.image(next_tex.id(), image, uv, theme::ONION_NEXT);
        }
        painter.image(tex.id(), image, uv, Color32::WHITE);
        let k = image.width() / result.cell.0.max(1) as f32;
        let base_size = result.base.as_ref().map(Image::size);
        // Suelo: la parte de abajo de la base (o de la celda).
        let ground_y = base_size.map_or(result.cell.1 as f32, |(_, h)| {
            (result.origin.1 as f32 + h as f32).min(result.cell.1 as f32)
        });
        let ground = image.top() + ground_y * k;
        if self.anim.ground {
            let mut x = image.left();
            while x < image.right() {
                painter.line_segment(
                    [pos2(x, ground), pos2((x + 6.0).min(image.right()), ground)],
                    Stroke::new(1.0, theme::ACCENT),
                );
                x += 10.0;
            }
            painter.text(
                pos2(image.left() + 4.0, ground - 4.0),
                Align2::LEFT_BOTTOM,
                "suelo",
                FontId::proportional(11.0),
                theme::ACCENT,
            );
        }
        if self.anim.pivot {
            let center_x = base_size.map_or(result.cell.0 as f32 / 2.0, |(w, _)| {
                result.origin.0 as f32 + w as f32 / 2.0
            });
            let p = pos2(image.left() + center_x * k, ground);
            painter.circle_stroke(p, 6.0, Stroke::new(1.5, theme::INFO));
            painter.line_segment(
                [p + vec2(-10.0, 0.0), p + vec2(10.0, 0.0)],
                Stroke::new(1.0, theme::INFO),
            );
            painter.line_segment(
                [p + vec2(0.0, -10.0), p + vec2(0.0, 10.0)],
                Stroke::new(1.0, theme::INFO),
            );
        }
        // «En el juego»: el fotograma a 2× sobre un fondo de escena.
        let scene_size = vec2(
            (result.cell.0 as f32 * 2.0).max(120.0) + 120.0,
            result.cell.1 as f32 * 2.0 + 44.0,
        );
        let scene = Rect::from_min_size(full.max - scene_size - vec2(16.0, 16.0), scene_size);
        painter.rect(
            scene,
            CornerRadius::same(theme::RADIUS_CARD),
            theme::CARD,
            Stroke::new(1.0, theme::LINE_STRONG),
            StrokeKind::Inside,
        );
        painter.text(
            scene.min + vec2(12.0, 10.0),
            Align2::LEFT_TOP,
            "En el juego",
            theme::semibold(12.0),
            theme::TEXT,
        );
        painter.text(
            pos2(scene.right() - 12.0, scene.top() + 10.0),
            Align2::RIGHT_TOP,
            "2×",
            FontId::proportional(11.5),
            theme::TEXT_3,
        );
        let sky = Rect::from_min_max(scene.min + vec2(12.0, 32.0), scene.max - vec2(12.0, 12.0));
        painter.rect_filled(
            sky,
            CornerRadius::same(6),
            Color32::from_rgb(0x2A, 0x34, 0x52),
        );
        let grass = Rect::from_min_max(pos2(sky.left(), sky.bottom() - 10.0), sky.max);
        painter.rect_filled(
            grass,
            CornerRadius::same(6),
            Color32::from_rgb(0x3E, 0x55, 0x3A),
        );
        let sprite = Rect::from_min_size(
            pos2(
                sky.center().x - result.cell.0 as f32,
                grass.top() + 2.0 - ground_y * 2.0,
            ),
            vec2(result.cell.0 as f32 * 2.0, result.cell.1 as f32 * 2.0),
        );
        painter.image(tex.id(), sprite, uv, Color32::WHITE);
    }
}

impl AppShell {
    /// Las variantes, sincronizadas: una tarjeta por variante con su
    /// fotograma, su tira, lo que dijo el agente y «Elegir».
    fn variants_view(&mut self, ui: &mut egui::Ui, out: &mut Vec<Intent>) {
        let ctx = ui.ctx().clone();
        let action_name = self.anim.action_name().unwrap_or_default();
        // El reloj común: las duraciones de la primera variante lista.
        let durations: Vec<u32> = self
            .anim
            .variants
            .iter()
            .find_map(|v| v.result.as_ref())
            .and_then(|r| {
                r.actions
                    .iter()
                    .find(|a| a.spec.name == action_name)
                    .or(r.actions.first())
            })
            .map(|a| Animation::default().durations_for(&a.spec))
            .unwrap_or_default();
        self.anim.tick(&ctx, &durations);
        let frame = self.anim.player.frame;

        self.tab_panel(ui, out);
        let options = Panel::top("opciones")
            .exact_size(theme::OPTIONS_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR_ALT).inner_margin(Margin::symmetric(14, 0)))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    if option_button(ui, "Volver al editor", false).clicked() {
                        out.push(Intent::ShowVariants(false));
                    }
                    vsep(ui);
                    ui.label(
                        RichText::new(format!("Variantes de «{action_name}»"))
                            .size(12.5)
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new("se reproducen sincronizadas")
                            .size(12.0)
                            .color(theme::TEXT_3),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if option_button(ui, "Suelo", self.anim.ground).clicked() {
                            self.anim.ground = !self.anim.ground;
                        }
                    });
                });
            });
        self.zones.options = options.response.rect;

        let bottom = Panel::bottom("mezclar")
            .exact_size(56.0)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::symmetric(16, 0)))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    let frames = durations.len();
                    let (label, kind) = if self.anim.player.playing {
                        ("Pausa", 5)
                    } else {
                        ("Reproducir", 2)
                    };
                    if transport_button(ui, label, kind, true).clicked() {
                        self.anim.player.toggle(frames);
                        self.anim.last_time = None;
                    }
                    // Sin variantes listas aún no hay fotogramas que contar.
                    let counter = if frames == 0 {
                        "fotograma –".to_string()
                    } else {
                        format!("fotograma {} / {frames}", frame + 1)
                    };
                    ui.label(
                        RichText::new(counter)
                            .font(theme::mono(12.0))
                            .color(theme::TEXT_2),
                    );
                    vsep(ui);
                    ui.label(RichText::new("Mezclar").size(12.0).color(theme::TEXT_3));
                    let ready = self
                        .anim
                        .variants
                        .iter()
                        .any(|v| v.state == VariantCardState::Ready);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let can_mix = ready && !self.anim.mix_text.trim().is_empty();
                        let mix = ui.add_enabled(
                            can_mix,
                            egui::Button::new(RichText::new("Crear mezcla").size(12.5))
                                .min_size(vec2(0.0, 32.0)),
                        );
                        if mix.clicked() {
                            out.push(Intent::MixVariants(self.anim.mix_text.trim().to_string()));
                            self.anim.mix_text.clear();
                        }
                        let field = ui.add(
                            egui::TextEdit::singleline(&mut self.anim.mix_text)
                                .margin(vec2(10.0, 7.0))
                                .hint_text("El rebote de A con los pasos de B")
                                .desired_width(ui.available_width()),
                        );
                        field.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Qué mezclar")
                        });
                    });
                });
            });
        self.zones.bottom = bottom.response.rect;

        let cards = egui::CentralPanel::default()
            .frame(Frame::NONE.fill(theme::BG).inner_margin(Margin::same(20)))
            .show(ui, |ui| {
                let full = ui.max_rect();
                let n = self.anim.variants.len().max(1);
                let gap = 16.0;
                let width = ((full.width() - gap * (n as f32 - 1.0)) / n as f32).max(160.0);
                let variants = self.anim.variants.clone();
                egui::ScrollArea::horizontal()
                    .id_salt("variantes")
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;
                            for (i, variant) in variants.iter().enumerate() {
                                let (rect, _) = ui.allocate_exact_size(
                                    vec2(width, full.height()),
                                    Sense::hover(),
                                );
                                self.variant_card(ui, rect, i, variant, &action_name, frame, out);
                            }
                        });
                    });
            });
        self.zones.canvas = cards.response.rect;
    }

    #[allow(clippy::too_many_arguments)]
    fn variant_card(
        &mut self,
        ui: &mut egui::Ui,
        rect: Rect,
        index: usize,
        variant: &VariantCard,
        action_name: &str,
        frame: usize,
        out: &mut Vec<Intent>,
    ) {
        let ctx = ui.ctx().clone();
        let painter = ui.painter_at(rect);
        painter.rect(
            rect,
            CornerRadius::same(theme::RADIUS_CARD),
            theme::CARD,
            Stroke::new(1.0, theme::LINE_STRONG),
            StrokeKind::Inside,
        );
        // Cabecera: letra, título y estado.
        let badge = Rect::from_min_size(rect.min + vec2(14.0, 14.0), vec2(20.0, 20.0));
        painter.rect_filled(
            badge,
            CornerRadius::same(5),
            if variant.mix {
                theme::INFO
            } else {
                theme::ACCENT
            },
        );
        painter.text(
            badge.center(),
            Align2::CENTER_CENTER,
            variant.letter.to_string(),
            theme::semibold(12.0),
            theme::ACCENT_INK,
        );
        let title = if variant.mix {
            format!("Mezcla {}", variant.letter)
        } else {
            format!("Variante {}", variant.letter)
        };
        // En una línea: con muchas tarjetas, recortado antes que partido.
        ui.put(
            Rect::from_min_max(
                badge.right_top() + vec2(8.0, 0.0),
                pos2(rect.right() - 64.0, badge.bottom()),
            ),
            egui::Label::new(
                RichText::new(title)
                    .font(theme::semibold(13.0))
                    .color(theme::TEXT),
            )
            .truncate(),
        );
        let (state, color) = match &variant.state {
            VariantCardState::Running => ("preparando…".to_string(), theme::TEXT_3),
            VariantCardState::Ready => ("nueva".to_string(), theme::TEXT_3),
            VariantCardState::Failed(_) => ("falló".to_string(), theme::DANGER),
        };
        painter.text(
            pos2(rect.right() - 14.0, badge.center().y),
            Align2::RIGHT_CENTER,
            state,
            FontId::proportional(11.5),
            color,
        );
        // Escenario: el fotograma común.
        let stage = Rect::from_min_size(
            rect.min + vec2(14.0, 46.0),
            vec2(
                rect.width() - 28.0,
                (rect.width() - 28.0).min(rect.height() * 0.42),
            ),
        );
        painter.rect_filled(
            stage,
            CornerRadius::same(theme::RADIUS_FIELD),
            theme::BAR_ALT,
        );
        let action = variant.result.as_ref().and_then(|r| {
            r.actions
                .iter()
                .find(|a| a.spec.name == action_name)
                .or(r.actions.first())
        });
        let mut y = stage.bottom() + 12.0;
        if let (Some(result), Some(action)) = (variant.result.as_ref(), action) {
            let action_index = result
                .actions
                .iter()
                .position(|a| std::ptr::eq(a, action))
                .unwrap_or(0);
            let f = frame.min(action.images.len().saturating_sub(1));
            let key = 10_000 + (variant.id as usize) * 64 + action_index;
            let tex = self.anim.texture(&ctx, key, f, &action.images[f]);
            let image = checkered_image(&painter, stage.shrink(8.0), Some(&tex), result.cell);
            if self.anim.ground {
                let ground_y = result.base.as_ref().map_or(result.cell.1 as f32, |b| {
                    (result.origin.1 as f32 + b.height() as f32).min(result.cell.1 as f32)
                });
                let gy = image.top() + ground_y * image.height() / result.cell.1.max(1) as f32;
                let mut x = image.left();
                while x < image.right() {
                    painter.line_segment(
                        [pos2(x, gy), pos2((x + 5.0).min(image.right()), gy)],
                        Stroke::new(1.0, theme::ACCENT),
                    );
                    x += 9.0;
                }
            }
            // Tira de fotogramas.
            let frames = action.images.len().max(1);
            let side = ((rect.width() - 28.0) / frames as f32).min(36.0);
            for (i, img) in action.images.iter().enumerate() {
                let tex = self.anim.texture(&ctx, key, i, img);
                let cell = Rect::from_min_size(
                    pos2(rect.left() + 14.0 + side * i as f32, y),
                    vec2(side - 3.0, side - 3.0),
                );
                checkered_image(&painter, cell, Some(&tex), img.size());
                if i == f {
                    painter.rect_stroke(
                        cell,
                        2.0,
                        Stroke::new(1.0, theme::ACCENT),
                        StrokeKind::Outside,
                    );
                }
            }
            y += side + 10.0;
        } else {
            let text = match &variant.state {
                VariantCardState::Failed(reason) => reason.clone(),
                _ => "El agente está preparando esta variante…".to_string(),
            };
            // Ajustado al ancho del escenario (un error puede ser largo).
            let galley = painter.layout(
                text,
                FontId::proportional(12.0),
                theme::TEXT_3,
                stage.width() - 24.0,
            );
            let at = stage.center() - galley.size() / 2.0;
            painter.galley(at, galley, theme::TEXT_3);
        }
        // Lo que dijo el agente, ajustado al ancho de la tarjeta.
        let summary = Rect::from_min_max(
            pos2(rect.left() + 14.0, y),
            pos2(rect.right() - 14.0, rect.bottom() - 60.0),
        );
        let layout = egui::Layout::top_down(egui::Align::Min);
        ui.scope_builder(UiBuilder::new().max_rect(summary).layout(layout), |ui| {
            egui::ScrollArea::vertical()
                .id_salt(("resumen", index))
                .show(ui, |ui| {
                    ui.set_width(summary.width());
                    ui.add(
                        egui::Label::new(
                            RichText::new(&variant.summary)
                                .size(12.5)
                                .color(theme::TEXT_BODY),
                        )
                        .wrap(),
                    );
                });
        });
        // Elegir.
        let button = Rect::from_min_max(
            pos2(rect.left() + 14.0, rect.bottom() - 48.0),
            pos2(rect.right() - 14.0, rect.bottom() - 14.0),
        );
        let ready = variant.state == VariantCardState::Ready;
        let choose = ui.put(
            button,
            egui::Button::new(
                RichText::new(format!("Elegir {}", variant.letter))
                    .font(theme::semibold(13.0))
                    .color(if ready {
                        theme::ACCENT_INK
                    } else {
                        theme::TEXT_3
                    }),
            )
            .fill(if ready { theme::ACCENT } else { theme::FIELD })
            .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON)),
        );
        if ready && choose.clicked() {
            out.push(Intent::ChooseVariant(index));
        }
    }

    /// «Exportar animación»: vista previa de la hoja, formato, escala,
    /// nombre y «Abrir la carpeta al terminar».
    fn anim_export_dialog(&mut self, ctx: &egui::Context, out: &mut Vec<Intent>) {
        let (Some(mut export), Some(result)) = (self.anim.export.clone(), self.anim.result.clone())
        else {
            self.anim.export = None;
            return;
        };
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::unique("lumeart-exportar-animacion")).show(ctx, |ui| {
            ui.set_width(920.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Exportar animación").font(theme::semibold(16.0)).color(theme::TEXT));
                ui.label(RichText::new(format!("{} · {} acciones", lume_domain::anim::project::safe_name(&result.name), result.actions.len())).size(12.5).color(theme::TEXT_3));
            });
            ui.separator();
            ui.horizontal_top(|ui| {
                // Vista previa de la hoja.
                ui.vertical(|ui| {
                    ui.set_width(520.0);
                    let preview = sheet(&result, 1);
                    let (w, h) = (preview.width() * export.scale, preview.height() * export.scale);
                    ui.horizontal(|ui| {
                        panel_label(ui, "Vista previa de la hoja");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new(format!("{w} × {h} px a {}×", export.scale)).font(theme::mono(11.5)).color(theme::TEXT_3));
                        });
                    });
                    let (rect, _) = ui.allocate_exact_size(vec2(520.0, 380.0), Sense::hover());
                    let tex = self.anim.texture(ctx, usize::MAX - 2, 0, &preview);
                    let image = checkered_image(ui.painter(), rect.shrink(16.0), Some(&tex), preview.size());
                    let (cw, ch) = (image.width() / preview.width().max(1) as f32 * result.cell.0 as f32, image.height() / preview.height().max(1) as f32 * result.cell.1 as f32);
                    for (row, a) in result.actions.iter().enumerate() {
                        for f in 0..a.images.len() {
                            let cell = Rect::from_min_size(image.min + vec2(cw * f as f32, ch * row as f32), vec2(cw, ch));
                            ui.painter().rect_stroke(cell, 0.0, Stroke::new(1.0, theme::ACCENT_RING), StrokeKind::Inside);
                        }
                    }
                    ui.label(RichText::new("Una fila por acción, una columna por fotograma. Las celdas que sobran quedan transparentes.").size(12.0).color(theme::TEXT_3));
                });
                ui.add_space(20.0);
                // Formato, escala y archivo.
                ui.vertical(|ui| {
                    ui.set_width(360.0);
                    ui.label(RichText::new("Formato").size(12.5).color(theme::TEXT_2));
                    for (format, title, desc) in [
                        (ExportFormat::Sheet, "Hoja de sprites PNG", "Todas las acciones en una imagen"),
                        (ExportFormat::Strips, "Una tira por acción", "Un PNG por acción"),
                        (ExportFormat::Gif, "GIF animado", "Para compartir; una acción por archivo"),
                        (ExportFormat::Godot, "Godot · SpriteFrames", "Recurso .tres listo para AnimatedSprite2D"),
                    ] {
                        let selected = export.format == format;
                        let response = ui.add(
                            egui::Button::selectable(selected, RichText::new(format!("{title}\n{desc}")).size(12.5))
                                .min_size(vec2(360.0, 46.0)),
                        );
                        let response = response.accessible_name(title);
                        if response.clicked() {
                            export.format = format;
                        }
                    }
                    ui.add_space(8.0);
                    ui.label(RichText::new("Escala").size(12.5).color(theme::TEXT_2));
                    ui.horizontal(|ui| {
                        for scale in [1u32, 2, 4, 8] {
                            let button = ui
                                .selectable_label(export.scale == scale, format!("{scale}×"))
                                .accessible_name(format!("Escala {scale}×"));
                            if button.clicked() {
                                export.scale = scale;
                                if !export.edited {
                                    export.file_name = sheet_file_name(&result, scale);
                                }
                            }
                        }
                    });
                    ui.add_space(8.0);
                    ui.label(RichText::new("Archivo").size(12.5).color(theme::TEXT_2));
                    let name = ui.add(egui::TextEdit::singleline(&mut export.file_name).margin(vec2(10.0, 7.0)).font(theme::mono(12.0)).desired_width(360.0));
                    let name = name.accessible_name("Nombre del archivo");
                    if name.changed() {
                        export.edited = true;
                    }
                    let stem = export.file_name.strip_suffix(".png").unwrap_or(&export.file_name).to_string();
                    ui.label(RichText::new(format!("También guarda {stem}.json con las acciones, fps y duraciones.")).size(12.0).color(theme::TEXT_3));
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.checkbox(&mut export.open_folder, "Abrir la carpeta al terminar");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let go = egui::Button::new(RichText::new("Exportar").color(theme::ACCENT_INK)).fill(theme::ACCENT);
                    if ui.add(go).clicked() {
                        out.push(Intent::ExportAnimation {
                            format: export.format,
                            scale: export.scale,
                            file_name: export.file_name.trim().to_string(),
                            open_folder: export.open_folder,
                        });
                        close = true;
                    }
                    if ui.button("Cancelar").clicked() {
                        close = true;
                    }
                });
            });
        });
        self.anim.export = if close || modal.should_close() {
            None
        } else {
            Some(export)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_duration_text() {
        assert_eq!(
            total_text(&[100, 100, 150, 100, 100, 100], true),
            "0,65 s en bucle"
        );
        assert_eq!(total_text(&[125; 4], false), "0,50 s una vez");
    }
}
