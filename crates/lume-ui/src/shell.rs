//! `AppShell`: el armazón del studio, igual en los tres modos (captura
//! `docs/diseno/capturas/01-studio-pixel.png`, valores de `Main.dc.html`).
//!
//! ```text
//! ┌──────────────────────────── cabecera 40 ─────────────────────────────┐
//! │herr.│ opciones 44                                │ agente 392        │
//! │ 52  │ lienzo                                     │                   │
//! │     │ panel inferior 132                         │                   │
//! └──────────────────────────── estado 26 ───────────────────────────────┘
//! ```
//!
//! `show` solo lee el `ViewModel` y devuelve `Intent`s: no tiene acceso a la
//! sesión, así que no puede modificar el documento.

use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Frame, Layout, Margin, Panel, Rect, RichText,
    Sense, Stroke, StrokeKind, UiBuilder, pos2, vec2,
};
use lume_app::ViewModel;
use lume_domain::Rgba;

#[path = "anim_mode.rs"]
mod anim_mode;
pub use anim_mode::{AnimExport, AnimMode, VariantCard, VariantCardState};
#[path = "vec_anim.rs"]
mod vec_anim;
#[path = "vec_mode.rs"]
mod vec_mode;
pub use vec_anim::{RigImages, VecAnim, frame_count as vec_anim_frames};
pub use vec_mode::{
    ReplayFrame, SliceImage, VecMode, VecTool, VecView, ZoneState, finish_corrections,
};

use crate::agent_panel::PanelTab;
use crate::canvas_view::CanvasView;
use crate::icons::{self, Icon};
use crate::intent::{Intent, Mode};
use crate::selection::{EditMachine, Selection, selected_pixels};
use crate::theme;
use crate::tools::{Button, DrawTool, ToolMachine, ToolOptions, ToolResult};

/// Herramientas del modo pixel art, en el orden de la barra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tool {
    #[default]
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
}

impl Tool {
    pub const DRAW: [Tool; 7] = [
        Tool::Pencil,
        Tool::Eraser,
        Tool::Fill,
        Tool::Line,
        Tool::Rect,
        Tool::Ellipse,
        Tool::Eyedropper,
    ];
    pub const EDIT: [Tool; 3] = [Tool::Select, Tool::Move, Tool::Hand];

    /// La herramienta de dibujo que corresponde (las de edición no dibujan).
    pub fn draw_tool(self) -> Option<DrawTool> {
        Some(match self {
            Tool::Pencil => DrawTool::Pencil,
            Tool::Eraser => DrawTool::Eraser,
            Tool::Fill => DrawTool::Fill,
            Tool::Line => DrawTool::Line,
            Tool::Rect => DrawTool::Rect,
            Tool::Ellipse => DrawTool::Ellipse,
            Tool::Eyedropper => DrawTool::Eyedropper,
            Tool::Select | Tool::Move | Tool::Hand => return None,
        })
    }

    /// Tecla del atajo.
    pub fn egui_key(self) -> egui::Key {
        match self {
            Tool::Pencil => egui::Key::B,
            Tool::Eraser => egui::Key::E,
            Tool::Fill => egui::Key::G,
            Tool::Line => egui::Key::L,
            Tool::Rect => egui::Key::U,
            Tool::Ellipse => egui::Key::O,
            Tool::Eyedropper => egui::Key::I,
            Tool::Select => egui::Key::M,
            Tool::Move => egui::Key::V,
            Tool::Hand => egui::Key::H,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Tool::Pencil => "Lápiz",
            Tool::Eraser => "Borrador",
            Tool::Fill => "Relleno",
            Tool::Line => "Línea",
            Tool::Rect => "Rectángulo",
            Tool::Ellipse => "Elipse",
            Tool::Eyedropper => "Cuentagotas",
            Tool::Select => "Selección",
            Tool::Move => "Mover",
            Tool::Hand => "Mano",
        }
    }

    /// Atajo de teclado.
    pub fn key(self) -> char {
        match self {
            Tool::Pencil => 'B',
            Tool::Eraser => 'E',
            Tool::Fill => 'G',
            Tool::Line => 'L',
            Tool::Rect => 'U',
            Tool::Ellipse => 'O',
            Tool::Eyedropper => 'I',
            Tool::Select => 'M',
            Tool::Move => 'V',
            Tool::Hand => 'H',
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Tool::Pencil => Icon::Pencil,
            Tool::Eraser => Icon::Eraser,
            Tool::Fill => Icon::Fill,
            Tool::Line => Icon::Line,
            Tool::Rect => Icon::Rect,
            Tool::Ellipse => Icon::Ellipse,
            Tool::Eyedropper => Icon::Eyedropper,
            Tool::Select => Icon::Select,
            Tool::Move => Icon::Move,
            Tool::Hand => Icon::Hand,
        }
    }
}

/// Rectángulos de cada zona en el último fotograma (para tests y para
/// colocar capas encima).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zones {
    pub header: Rect,
    pub toolbar: Rect,
    pub options: Rect,
    pub canvas: Rect,
    pub bottom: Rect,
    pub agent: Rect,
    pub status: Rect,
}

impl Default for Zones {
    fn default() -> Self {
        Self {
            header: Rect::NOTHING,
            toolbar: Rect::NOTHING,
            options: Rect::NOTHING,
            canvas: Rect::NOTHING,
            bottom: Rect::NOTHING,
            agent: Rect::NOTHING,
            status: Rect::NOTHING,
        }
    }
}

/// Trazo enviado a la sesión: revisión en la que se envió, celdas y color.
type PendingStroke = (u64, Vec<(u32, u32)>, Rgba);

/// Estado propio de la interfaz (no del documento).
#[derive(Debug, Clone)]
pub struct AppShell {
    pub mode: Mode,
    pub tool: Tool,
    pub grid: bool,
    pub primary: Rgba,
    pub secondary: Rgba,
    /// Dirección de la API local («127.0.0.1:7777»), si la hay.
    pub server: Option<String>,
    /// Estado del agente para la pastilla de la cabecera («Vuelta 3 de 8»).
    pub agent_status: Option<String>,
    pub zones: Zones,
    /// Lienzo en pantalla: zoom, desplazamiento, textura y cursor.
    pub canvas: CanvasView,
    /// Opciones de las herramientas (tamaño, píxel perfecto, simetría…).
    pub options: ToolOptions,
    /// Estado de la herramienta de dibujo activa.
    pub machine: ToolMachine,
    /// Trazo enviado que aún no ha vuelto en el `ViewModel` (revisión en la
    /// que se envió, celdas y color), para que no parpadee.
    pending: Option<PendingStroke>,
    /// Selección rectangular actual.
    pub selection: Option<Selection>,
    /// Nombre del archivo para la cabecera (si no, el del documento).
    pub file_name: Option<String>,
    /// Encargos que llegaron por MCP (cabecera y menú Agente).
    pub jobs: Vec<crate::jobs::JobItem>,
    /// El encargo que se está viendo (`None`: el dibujo propio).
    pub viewing_job: Option<lume_app::ArtId>,
    /// Hay cambios sin guardar (marca «•» en la cabecera).
    pub dirty: bool,
    /// Último aviso (guardado, exportado, error…), en la barra de estado.
    pub notice: Option<String>,
    /// Escala elegida en el diálogo «Exportar PNG», si está abierto.
    pub export_scale: Option<u32>,
    /// Resaltar en el lienzo el último comando de la CLI o del agente
    /// (Ajustes: «Mostrar cada comando en el lienzo»).
    pub show_commands: bool,
    /// Estado de Selección y Mover.
    pub edit: EditMachine,
    /// Panel del agente (conversación y redacción).
    pub agent: crate::agent_panel::AgentPanel,
    /// Lo que el panel del agente necesita saber (ajustes, si dibuja).
    pub agent_view: crate::agent_panel::AgentView,
    /// Versión que se está viendo en el lienzo en lugar del dibujo actual.
    pub preview: Option<VersionPreview>,
    /// Estado del modo animación.
    pub anim: AnimMode,
    /// Estado del modo vector.
    pub vec: VecMode,
    /// Ancho del grupo central de la cabecera en el fotograma anterior (para
    /// centrarlo).
    header_center_width: f32,
    /// Pestaña Código: lo que va dibujando el agente.
    pub code_log: crate::panel_tabs::CodeLog,
    /// Pestaña Historial: miniatura de cada paso.
    pub history_thumbs: crate::panel_tabs::HistoryThumbs,
    /// Pestañas de documentos abiertos (las pone `StudioApp`).
    pub tabs: Vec<TabView>,
    pub active_tab: usize,
    /// Nombre que se está escribiendo para la pestaña activa (F2).
    pub renaming: Option<String>,
    /// Animación del dibujo vectorial (su esqueleto).
    pub vanim: VecAnim,
    /// Acaba de empezar el renombrado: se selecciona todo el nombre.
    rename_fresh: bool,
}

/// Lo que se ve de una pestaña de documento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabView {
    /// Id fijo de la pestaña (el `tab` del MCP).
    pub id: u64,
    pub title: String,
    pub dirty: bool,
    pub mode: Mode,
    /// Su agente está dibujando (aunque no se vea).
    pub busy: bool,
    /// Enseña Inicio.
    pub home: bool,
    /// Es un encargo (`create_art`), con su número.
    pub job: Option<u64>,
}

/// Alto de la tira de pestañas de documentos.
pub const TAB_STRIP_HEIGHT: f32 = 34.0;

impl Default for AppShell {
    fn default() -> Self {
        Self {
            mode: Mode::Pixel,
            tool: Tool::Pencil,
            grid: true,
            primary: Rgba::rgb(0x8B, 0x3F, 0xE0),
            secondary: Rgba::rgb(0x1B, 0x10, 0x26),
            server: None,
            agent_status: None,
            zones: Zones::default(),
            canvas: CanvasView::default(),
            options: ToolOptions::default(),
            machine: ToolMachine::new(DrawTool::Pencil),
            pending: None,
            selection: None,
            show_commands: false,
            agent: Default::default(),
            agent_view: Default::default(),
            preview: None,
            anim: AnimMode::default(),
            vec: VecMode::default(),
            file_name: None,
            jobs: Vec::new(),
            viewing_job: None,
            dirty: false,
            notice: None,
            export_scale: None,
            edit: EditMachine::default(),
            header_center_width: 360.0,
            code_log: Default::default(),
            history_thumbs: Default::default(),
            tabs: Vec::new(),
            active_tab: 0,
            renaming: None,
            rename_fresh: false,
            vanim: VecAnim::default(),
        }
    }
}

pub(crate) fn color32(c: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

fn bar_frame(fill: Color32) -> Frame {
    Frame::NONE.fill(fill)
}

/// Etiqueta de panel: 11/600 en mayúsculas, `text_3`.
fn panel_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .font(theme::semibold(11.0))
            .color(theme::TEXT_3)
            .extra_letter_spacing(0.66),
    );
}

/// Botón de icono de `size` × `size` (herramientas, deshacer…).
fn icon_button(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    size: f32,
    active: bool,
    enabled: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let response = response.on_hover_text(label);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, label));
    let response = if enabled {
        response
    } else {
        response.on_disabled_hover_text(label)
    };
    let painter = ui.painter_at(rect);
    let hovered = enabled && response.hovered();
    let (bg, fg) = if active {
        (theme::ACCENT_SOFT, theme::ACCENT)
    } else if hovered {
        (theme::HOVER, theme::TEXT)
    } else if enabled {
        (Color32::TRANSPARENT, theme::TEXT_3)
    } else {
        (Color32::TRANSPARENT, theme::TEXT_4)
    };
    painter.rect_filled(rect, CornerRadius::same(theme::RADIUS_BUTTON), bg);
    icons::paint(&painter, icon, rect.center(), fg);
    if enabled { response } else { response.clone() }
}

/// Botón de texto de la barra de opciones (alto 26).
fn option_button(ui: &mut egui::Ui, text: &str, on: bool) -> egui::Response {
    let (fill, stroke, color) = if on {
        (theme::HOVER, Stroke::NONE, theme::TEXT)
    } else {
        (Color32::TRANSPARENT, Stroke::NONE, theme::TEXT_2)
    };
    ui.add(
        egui::Button::new(RichText::new(text).size(12.0).color(color))
            .fill(fill)
            .stroke(stroke)
            .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
            .min_size(vec2(0.0, 26.0)),
    )
}

/// «Tamaño» con un campo de 1 a 16 px.
fn size_field(ui: &mut egui::Ui, size: &mut u32) {
    ui.label(RichText::new("Tamaño").size(12.0).color(theme::TEXT_2));
    ui.add(
        egui::DragValue::new(size)
            .range(1..=16)
            .suffix(" px")
            .speed(0.1),
    );
}

/// Botón con borde que se enciende y se apaga.
fn toggle(ui: &mut egui::Ui, text: &str, on: &mut bool) {
    let (fill, color) = if *on {
        (theme::FIELD, theme::TEXT)
    } else {
        (Color32::TRANSPARENT, theme::TEXT_2)
    };
    let response = ui.add(
        egui::Button::new(RichText::new(text).size(12.0).color(color))
            .fill(fill)
            .stroke(Stroke::new(1.0, theme::LINE_STRONG))
            .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
            .min_size(vec2(0.0, 26.0))
            .selected(*on),
    );
    if response.clicked() {
        *on = !*on;
    }
}

fn vsep(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, 18.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::LINE_SOFT);
}

impl AppShell {
    /// Dibuja el armazón entero en `ui` y devuelve lo que pide el usuario.
    pub fn show(&mut self, ui: &mut egui::Ui, vm: &ViewModel) -> Vec<Intent> {
        let mut out = Vec::new();
        self.shortcuts(ui, vm, &mut out);
        if self.mode == Mode::Animation {
            // Un dibujo vectorial se anima con su esqueleto; el pixel art,
            // con anim.js.
            if vm.vector.is_some() {
                self.show_vec_animation(ui, vm, &mut out);
            } else {
                self.show_animation(ui, vm, &mut out);
            }
            return out;
        }
        if self.mode == Mode::Vector {
            self.show_vector(ui, vm, &mut out);
            return out;
        }
        self.code_log.observe(vm);
        // Antes que los paneles: la pestaña Código ve ya por dónde va.
        self.canvas.brush.observe(vm, self.canvas.speed);
        // Viendo una versión, el lienzo no es el del último paso.
        if self.preview.is_none() {
            self.history_thumbs.observe(vm);
        }

        let header = Panel::top("cabecera")
            .exact_size(theme::HEADER_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin {
                left: 14,
                right: 12,
                top: 0,
                bottom: 0,
            }))
            .show(ui, |ui| self.header(ui, vm, &mut out));
        self.zones.header = header.response.rect;

        let status = Panel::bottom("estado")
            .exact_size(theme::STATUS_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::symmetric(14, 0)))
            .show(ui, |ui| self.status_bar(ui, vm));
        self.zones.status = status.response.rect;

        let agent = Panel::right("agente")
            .exact_size(theme::AGENT_WIDTH)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::same(16)))
            .show(ui, |ui| self.agent_panel(ui, &mut out, Some(vm)));
        self.zones.agent = agent.response.rect;

        let toolbar = Panel::left("herramientas")
            .exact_size(theme::TOOLBAR_WIDTH)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::symmetric(8, 10)))
            .show(ui, |ui| self.toolbar(ui));
        self.zones.toolbar = toolbar.response.rect;

        self.tab_panel(ui, &mut out);
        let options = Panel::top("opciones")
            .exact_size(theme::OPTIONS_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR_ALT).inner_margin(Margin::symmetric(14, 0)))
            .show(ui, |ui| self.options_bar(ui, vm, &mut out));
        self.zones.options = options.response.rect;

        let bottom = Panel::bottom("panel-inferior")
            .exact_size(theme::BOTTOM_PIXEL_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR))
            .show(ui, |ui| self.bottom_dock(ui, vm));
        self.zones.bottom = bottom.response.rect;

        if self.export_scale.is_some() {
            self.export_dialog(ui.ctx(), vm, &mut out);
        }

        let canvas = egui::CentralPanel::default()
            .frame(Frame::NONE.fill(theme::BG))
            .show(ui, |ui| self.canvas_area(ui, vm, &mut out));
        self.zones.canvas = canvas.response.rect;

        out
    }

    /// Abre el diálogo «Exportar PNG».
    pub fn open_export_dialog(&mut self) {
        self.export_scale = Some(self.export_scale.unwrap_or(1));
    }

    /// Diálogo «Exportar PNG»: escala (1×, 2×, 4×, 8×, 16× u otra hasta 64) y
    /// tamaño resultante.
    fn export_dialog(&mut self, ctx: &egui::Context, vm: &ViewModel, out: &mut Vec<Intent>) {
        let Some(mut scale) = self.export_scale else {
            return;
        };
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::unique("lumeart-exportar")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.label(
                RichText::new("Exportar PNG")
                    .font(theme::semibold(16.0))
                    .color(theme::TEXT),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let label = ui.label(RichText::new("Escala").size(13.0).color(theme::TEXT_2));
                for s in [1u32, 2, 4, 8, 16] {
                    if ui.selectable_label(scale == s, format!("{s}×")).clicked() {
                        scale = s;
                    }
                }
                ui.add(egui::DragValue::new(&mut scale).range(1..=64).suffix("×"))
                    .labelled_by(label.id);
            });
            let (w, h) = (vm.canvas.width(), vm.canvas.height());
            ui.label(
                RichText::new(format!("{w} × {h} → {} × {} px", w * scale, h * scale))
                    .font(theme::mono(12.0))
                    .color(theme::TEXT_3),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cancelar").clicked() {
                    close = true;
                }
                let export = egui::Button::new(RichText::new("Exportar…").color(theme::ACCENT_INK))
                    .fill(theme::ACCENT);
                if ui.add(export).clicked() {
                    out.push(Intent::ExportPng { scale });
                    close = true;
                }
            });
        });
        self.export_scale = if close || modal.should_close() {
            None
        } else {
            Some(scale)
        };
    }

    fn header(&mut self, ui: &mut egui::Ui, vm: &ViewModel, out: &mut Vec<Intent>) {
        let full = ui.max_rect();
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            app_icon(ui);
            ui.label(
                RichText::new("LumeArt")
                    .font(theme::semibold(13.0))
                    .color(theme::TEXT),
            );
            ui.add_space(8.0);
            ui.spacing_mut().item_spacing.x = 2.0;
            self.menus(ui, vm, out);
        });

        // Grupo central: selector de modo, archivo y tamaño.
        let center =
            Rect::from_center_size(full.center(), vec2(self.header_center_width, full.height()));
        let group = ui.scope_builder(
            UiBuilder::new()
                .max_rect(center)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                self.mode_selector(ui, out);
                ui.add_space(8.0);
                let name = self
                    .file_name
                    .clone()
                    .unwrap_or_else(|| format!("{}.lume", vm.name));
                ui.label(RichText::new(name).color(theme::TEXT).size(13.0));
                if self.dirty {
                    ui.label(RichText::new("•").color(theme::WARN).size(13.0))
                        .on_hover_text("Cambios sin guardar");
                }
                ui.label(RichText::new("·").color(Color32::from_rgb(0x4A, 0x51, 0x5C)));
                // En modo vector, el tamaño del dibujo (en unidades).
                let (w, h) = match (self.mode, self.vec.size()) {
                    (Mode::Vector, Some(size)) => size,
                    (Mode::Animation, Some(size)) if vm.vector.is_some() => size,
                    _ => (i64::from(vm.canvas.width()), i64::from(vm.canvas.height())),
                };
                ui.label(
                    RichText::new(format!("{w} × {h}"))
                        .font(theme::mono(12.0))
                        .color(theme::TEXT_2),
                );
            },
        );
        self.header_center_width = group.response.rect.width().max(1.0);

        // Derecha: pastilla de estado del agente y «Exportar».
        let right = Rect::from_min_max(pos2(full.right() - 360.0, full.top()), full.right_bottom());
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(right)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let export = ui.add(
                    egui::Button::new(
                        RichText::new("      Exportar")
                            .size(12.0)
                            .color(theme::TEXT),
                    )
                    .fill(theme::FIELD)
                    .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                    .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
                    .min_size(vec2(0.0, 28.0)),
                );
                icons::paint(
                    &ui.painter_at(export.rect),
                    Icon::Export,
                    export.rect.left_center() + vec2(18.0, 0.0),
                    theme::TEXT,
                );
                if export.clicked() {
                    out.push(Intent::Export);
                }
                if let Some(status) = &self.agent_status {
                    status_pill(ui, status);
                }
                if !self.jobs.is_empty() {
                    let running = self
                        .jobs
                        .iter()
                        .filter(|j| j.state == lume_app::ArtState::Running)
                        .count();
                    let text = if running > 0 {
                        format!("Encargos · {running} en marcha")
                    } else {
                        format!("Encargos · {}", self.jobs.len())
                    };
                    let (jobs, viewing) = (&self.jobs, self.viewing_job);
                    ui.menu_button(RichText::new(text).size(12.0).color(theme::TEXT_2), |ui| {
                        crate::jobs::menu_entries(ui, jobs, viewing, out);
                    });
                }
            },
        );
    }

    fn menus(&mut self, ui: &mut egui::Ui, vm: &ViewModel, out: &mut Vec<Intent>) {
        // Menús sin marco: solo texto, con fondo al pasar el ratón.
        let w = &mut ui.style_mut().visuals.widgets;
        w.inactive.weak_bg_fill = Color32::TRANSPARENT;
        w.inactive.bg_stroke = Stroke::NONE;
        w.hovered.weak_bg_fill = Color32::from_rgb(0x18, 0x1C, 0x22);
        w.hovered.bg_stroke = Stroke::NONE;
        w.open.weak_bg_fill = Color32::from_rgb(0x18, 0x1C, 0x22);
        w.open.bg_stroke = Stroke::NONE;
        ui.spacing_mut().button_padding = vec2(10.0, 4.0);
        let menu = |text: &str| RichText::new(text).size(13.0).color(theme::TEXT_2);
        ui.menu_button(menu("Archivo"), |ui| {
            let items = [
                ("Nuevo…", "Ctrl+N", Intent::GoHome),
                ("Abrir…", "Ctrl+O", Intent::OpenFile),
                ("Guardar", "Ctrl+S", Intent::Save),
                ("Guardar como…", "Ctrl+Mayús+S", Intent::SaveAs),
                ("Exportar PNG…", "Ctrl+E", Intent::Export),
                ("Importar carpeta de pxcli…", "", Intent::ImportPxcli),
            ];
            for (text, shortcut, intent) in items {
                let button = egui::Button::new(text).shortcut_text(shortcut);
                if ui.add(button).clicked() {
                    out.push(intent);
                    ui.close();
                }
            }
        });
        ui.menu_button(menu("Editar"), |ui| {
            if ui
                .add_enabled(vm.can_undo, egui::Button::new("Deshacer"))
                .clicked()
            {
                out.push(Intent::Undo);
                ui.close();
            }
            if ui
                .add_enabled(vm.can_redo, egui::Button::new("Rehacer"))
                .clicked()
            {
                out.push(Intent::Redo);
                ui.close();
            }
            ui.separator();
            if ui
                .add(egui::Button::new("Ajustes…").shortcut_text("Ctrl+,"))
                .clicked()
            {
                out.push(Intent::OpenSettings);
                ui.close();
            }
        });
        ui.menu_button(menu("Ver"), |ui| {
            ui.checkbox(&mut self.grid, "Cuadrícula");
            // Es un ajuste: se guarda (lo atiende el controlador).
            let mut show = self.show_commands;
            if ui
                .checkbox(&mut show, "Mostrar cada comando en el lienzo")
                .changed()
            {
                self.show_commands = show;
                out.push(Intent::ShowCommands(show));
            }
        });
        let (jobs, viewing) = (&self.jobs, self.viewing_job);
        ui.menu_button(menu("Agente"), |ui| {
            ui.label(RichText::new("Encargos").size(11.0).color(theme::TEXT_3));
            crate::jobs::menu_entries(ui, jobs, viewing, out);
        });
    }

    fn mode_selector(&mut self, ui: &mut egui::Ui, out: &mut Vec<Intent>) {
        Frame::NONE
            .fill(theme::BG)
            .stroke(Stroke::new(1.0, theme::LINE_SOFT))
            .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
            .inner_margin(Margin::same(2))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for mode in Mode::ALL {
                    let selected = self.mode == mode;
                    let color = if selected { theme::TEXT } else { theme::TEXT_3 };
                    let button =
                        egui::Button::new(RichText::new(mode.label()).size(12.0).color(color))
                            .fill(if selected {
                                theme::HOVER
                            } else {
                                Color32::TRANSPARENT
                            })
                            .stroke(Stroke::NONE)
                            .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
                            .min_size(vec2(0.0, 24.0));
                    let response = ui.add_enabled(mode.available(), button);
                    if response.clicked() && !selected {
                        self.mode = mode;
                        out.push(Intent::SwitchMode(mode));
                    }
                }
            });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for tool in Tool::DRAW {
                self.tool_button(ui, tool);
            }
            ui.add_space(6.0);
            let (rect, _) = ui.allocate_exact_size(vec2(24.0, 1.0), Sense::hover());
            ui.painter().rect_filled(rect, 0.0, theme::LINE_SOFT);
            ui.add_space(6.0);
            for tool in Tool::EDIT {
                self.tool_button(ui, tool);
            }
        });
        // Colores principal y secundario, abajo: un clic abre el selector.
        let max = ui.max_rect();
        let area = Rect::from_min_size(
            pos2(max.center().x - 18.0, max.bottom() - 40.0),
            vec2(36.0, 40.0),
        );
        let secondary =
            Rect::from_min_size(area.right_bottom() - vec2(22.0, 22.0), vec2(22.0, 22.0));
        let primary = Rect::from_min_size(area.min, vec2(22.0, 22.0));
        // El principal tapa una esquina del secundario: se registra después
        // para que el clic en esa esquina sea suyo.
        let secondary_click = swatch(ui, secondary, "Color secundario", self.secondary);
        let primary_click = swatch(ui, primary, "Color principal", self.primary);
        color_popup(
            &primary_click,
            &mut self.primary,
            Some(egui::RectAlign::RIGHT_END),
        );
        color_popup(
            &secondary_click,
            &mut self.secondary,
            Some(egui::RectAlign::RIGHT_END),
        );
        let painter = ui.painter();
        painter.rect_filled(secondary, CornerRadius::same(5), color32(self.secondary));
        painter.rect_stroke(
            secondary,
            CornerRadius::same(5),
            Stroke::new(1.0, Color32::from_rgb(0x3A, 0x41, 0x4D)),
            StrokeKind::Outside,
        );
        painter.rect_stroke(
            primary.expand(2.0),
            CornerRadius::same(7),
            Stroke::new(1.0, theme::TEXT),
            StrokeKind::Inside,
        );
        painter.rect_stroke(
            primary,
            CornerRadius::same(5),
            Stroke::new(2.0, theme::BAR),
            StrokeKind::Outside,
        );
        painter.rect_filled(primary, CornerRadius::same(5), color32(self.primary));
    }

    fn tool_button(&mut self, ui: &mut egui::Ui, tool: Tool) {
        let label = format!("{} ({})", tool.label(), tool.key());
        let active = self.tool == tool;
        if icon_button(ui, tool.icon(), &label, theme::TOOL_BUTTON, active, true).clicked() {
            self.tool = tool;
        }
    }

    /// Opciones de la herramienta activa.
    fn tool_options(&mut self, ui: &mut egui::Ui) {
        let hint = |ui: &mut egui::Ui, text: &str| {
            ui.label(RichText::new(text).size(12.0).color(theme::TEXT_3));
        };
        let o = &mut self.options;
        match self.tool {
            Tool::Pencil => {
                size_field(ui, &mut o.size);
                toggle(ui, "Píxel perfecto", &mut o.pixel_perfect);
                toggle(ui, "Simetría X", &mut o.mirror_x);
                toggle(ui, "Simetría Y", &mut o.mirror_y);
            }
            Tool::Eraser | Tool::Line => {
                size_field(ui, &mut o.size);
                toggle(ui, "Simetría X", &mut o.mirror_x);
                toggle(ui, "Simetría Y", &mut o.mirror_y);
            }
            Tool::Rect | Tool::Ellipse => {
                size_field(ui, &mut o.size);
                toggle(ui, "Con relleno", &mut o.filled);
                toggle(ui, "Simetría X", &mut o.mirror_x);
                toggle(ui, "Simetría Y", &mut o.mirror_y);
            }
            Tool::Fill => hint(ui, "Rellena la zona contigua del mismo color (4 vecinos)"),
            Tool::Eyedropper => hint(ui, "Clic: color principal · clic derecho: secundario"),
            Tool::Select => hint(ui, "Arrastra para seleccionar · Esc cancela"),
            Tool::Move => hint(ui, "Arrastra la selección para moverla"),
            Tool::Hand => hint(
                ui,
                "Arrastra para mover la vista (también espacio o botón central)",
            ),
        }
    }

    /// La tira de pestañas de documentos, encima de la barra de opciones
    /// (como en Photoshop): modo, nombre, punto si hay cambios, «×» para
    /// cerrar (o botón central) y «+» para abrir o crear otro.
    pub(crate) fn tab_panel(&mut self, ui: &mut egui::Ui, out: &mut Vec<Intent>) {
        if self.tabs.is_empty() {
            return;
        }
        Panel::top("pestanas-documentos")
            .exact_size(TAB_STRIP_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BG))
            .show(ui, |ui| self.tab_strip(ui, out));
    }

    pub(crate) fn tab_strip(&mut self, ui: &mut egui::Ui, out: &mut Vec<Intent>) {
        let full = ui.max_rect();
        ui.painter().hline(
            full.x_range(),
            full.bottom() - 0.5,
            Stroke::new(1.0, theme::LINE),
        );
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (i, tab) in self.tabs.iter().enumerate() {
                let active = i == self.active_tab;
                let title = crate::agent_panel::ellipsize(&tab.title, 28);
                let font = egui::FontId::proportional(12.5);
                let text_w = ui
                    .painter()
                    .layout_no_wrap(title.clone(), font.clone(), theme::TEXT)
                    .size()
                    .x;
                let size = vec2((text_w + 72.0).max(120.0), TAB_STRIP_HEIGHT);
                let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                response.widget_info(|| {
                    egui::WidgetInfo::selected(egui::Role::Tab, true, active, &tab.title)
                });
                let close_rect = Rect::from_center_size(
                    pos2(rect.right() - 16.0, rect.center().y),
                    vec2(18.0, 18.0),
                );
                let close = ui
                    .interact(close_rect, response.id.with("cerrar"), Sense::click())
                    .on_hover_text("Cerrar");
                close.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::Button,
                        true,
                        format!("Cerrar {}", tab.title),
                    )
                });
                let painter = ui.painter();
                let fill = if active {
                    theme::BAR_ALT
                } else if response.hovered() {
                    theme::HOVER
                } else {
                    theme::BG
                };
                painter.rect_filled(rect, 0.0, fill);
                if active {
                    painter.hline(
                        rect.x_range(),
                        rect.top() + 1.0,
                        Stroke::new(2.0, theme::ACCENT),
                    );
                }
                painter.vline(
                    rect.right() - 0.5,
                    rect.y_range(),
                    Stroke::new(1.0, theme::LINE),
                );
                let icon_at = pos2(rect.left() + 18.0, rect.center().y);
                if tab.busy {
                    // Su agente dibuja: un arco que gira.
                    let t = ui.input(|i| i.time) as f32;
                    let points: Vec<egui::Pos2> = (0..=16)
                        .map(|k| {
                            let a = t * 5.0 + k as f32 / 16.0 * 4.4;
                            icon_at + vec2(a.cos(), a.sin()) * 6.0
                        })
                        .collect();
                    painter.add(egui::Shape::line(points, Stroke::new(1.8, theme::ACCENT)));
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(33));
                } else if tab.home {
                    painter.circle_stroke(icon_at, 5.0, Stroke::new(1.5, theme::TEXT_3));
                } else {
                    let icon = match tab.mode {
                        Mode::Pixel => Icon::Pencil,
                        Mode::Vector => Icon::Curve,
                        Mode::Animation => Icon::Play,
                    };
                    crate::icons::paint(
                        painter,
                        icon,
                        icon_at,
                        if active { theme::ACCENT } else { theme::TEXT_3 },
                    );
                }
                painter.text(
                    pos2(rect.left() + 34.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &title,
                    font,
                    if active { theme::TEXT } else { theme::TEXT_2 },
                );
                let show_x = response.hovered() || close.hovered() || active;
                if tab.dirty && !close.hovered() {
                    painter.circle_filled(close_rect.center(), 3.5, theme::WARN);
                } else if show_x {
                    if close.hovered() {
                        painter.rect_filled(close_rect, CornerRadius::same(4), theme::HOVER);
                    }
                    painter.text(
                        close_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "×",
                        egui::FontId::proportional(14.0),
                        theme::TEXT_3,
                    );
                }
                // Renombrar en la propia pestaña (F2 o doble clic).
                if active && response.double_clicked() {
                    self.renaming = Some(tab.title.trim_end_matches(".lume").to_string());
                    self.rename_fresh = true;
                }
                if let (true, Some(name)) = (active, self.renaming.as_mut()) {
                    let field = Rect::from_min_max(
                        pos2(rect.left() + 30.0, rect.top() + 5.0),
                        pos2(rect.right() - 30.0, rect.bottom() - 5.0),
                    );
                    let edit = ui.put(
                        field,
                        egui::TextEdit::singleline(name)
                            .font(egui::FontId::proportional(12.5))
                            .margin(vec2(6.0, 3.0)),
                    );
                    edit.request_focus();
                    if self.rename_fresh {
                        // Todo seleccionado: lo que se escribe lo sustituye.
                        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), edit.id) {
                            let all = egui::text::CCursorRange::two(
                                egui::text::CCursor::new(0),
                                egui::text::CCursor::new(name.chars().count()),
                            );
                            state.cursor.set_char_range(Some(all));
                            state.store(ui.ctx(), edit.id);
                            self.rename_fresh = false;
                        }
                    }
                    let (enter, escape) = ui.input(|i| {
                        (
                            i.key_pressed(egui::Key::Enter),
                            i.key_pressed(egui::Key::Escape),
                        )
                    });
                    if enter {
                        out.push(Intent::RenameDocument(name.clone()));
                        self.renaming = None;
                    } else if escape || edit.lost_focus() {
                        self.renaming = None;
                    }
                }
                if close.clicked() || response.middle_clicked() {
                    out.push(Intent::CloseTab(i));
                } else if response.clicked() && !active {
                    out.push(Intent::SelectTab(i));
                }
            }
            // «+»: Inicio, para crear o abrir otro.
            let (rect, response) =
                ui.allocate_exact_size(vec2(34.0, TAB_STRIP_HEIGHT), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Button, true, "Nuevo o abrir")
            });
            let response =
                response.on_hover_text("Inicio en una pestaña nueva: crear o abrir otro (Ctrl+N)");
            if response.hovered() {
                ui.painter()
                    .rect_filled(rect.shrink(4.0), CornerRadius::same(6), theme::HOVER);
            }
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "+",
                egui::FontId::proportional(17.0),
                theme::TEXT_2,
            );
            if response.clicked() {
                out.push(Intent::GoHome);
            }
        });
    }

    /// Velocidad a la que se ve dibujar al agente (Píxel a píxel, Normal,
    /// Rápido, Al instante). Se guarda en los ajustes.
    fn speed_selector(&mut self, ui: &mut egui::Ui, out: &mut Vec<Intent>) {
        let mut speed = self.canvas.speed;
        let combo = egui::ComboBox::from_id_salt("velocidad-dibujo")
            .width(108.0)
            .selected_text(RichText::new(speed.label()).size(12.0).color(theme::TEXT_2))
            .show_ui(ui, |ui| {
                for s in lume_app::DrawSpeed::ALL {
                    ui.selectable_value(&mut speed, s, s.label());
                }
            });
        let _ = combo
            .response
            .on_hover_text("Velocidad a la que se ve dibujar al agente")
            .accessible_name("Velocidad de dibujo");
        if speed != self.canvas.speed {
            self.canvas.speed = speed;
            out.push(Intent::DrawSpeed(speed));
        }
    }

    fn options_bar(&mut self, ui: &mut egui::Ui, vm: &ViewModel, out: &mut Vec<Intent>) {
        let full = ui.max_rect();
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            ui.label(
                RichText::new(self.tool.label())
                    .size(12.0)
                    .color(theme::TEXT),
            );
            vsep(ui);
            self.tool_options(ui);
        });
        let right = Rect::from_min_max(pos2(full.right() - 540.0, full.top()), full.right_bottom());
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(right)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                let zoom = ui.add(
                    egui::Button::new(
                        RichText::new(format!("{}%", zoom_percent(self.canvas.zoom)))
                            .font(theme::mono(12.0))
                            .color(theme::TEXT),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                    .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
                    .min_size(vec2(0.0, 26.0)),
                );
                if zoom.on_hover_text("Ajustar al área (0)").clicked() {
                    self.canvas.fit_pending = true;
                }
                crate::canvas_view::background_selector(
                    ui,
                    &mut self.canvas.background,
                    "fondo-pixel",
                    false,
                );
                if option_button(ui, "Cuadrícula", self.grid).clicked() {
                    self.grid = !self.grid;
                }
                self.speed_selector(ui, out);
                vsep(ui);
                ui.spacing_mut().item_spacing.x = 4.0;
                if icon_button(ui, Icon::Redo, "Rehacer", 28.0, false, vm.can_redo).clicked()
                    && vm.can_redo
                {
                    out.push(Intent::Redo);
                }
                if icon_button(ui, Icon::Undo, "Deshacer", 28.0, false, vm.can_undo).clicked()
                    && vm.can_undo
                {
                    out.push(Intent::Undo);
                }
            },
        );
    }

    fn canvas_area(&mut self, ui: &mut egui::Ui, vm: &ViewModel, out: &mut Vec<Intent>) {
        // Viendo una versión: solo mirar (y moverse), con la banda encima.
        if let Some(preview) = self.preview.clone() {
            let area = ui.max_rect();
            let _ = self.canvas.show(ui, vm, self.grid, true, false);
            self.version_banner(ui, area, &preview, out);
            return;
        }
        let draw_tool = self.tool.draw_tool();
        if let Some(tool) = draw_tool
            && self.machine.tool != tool
        {
            self.machine = ToolMachine::new(tool);
        }
        let area = ui.max_rect();
        let hand = self.tool == Tool::Hand;
        // Una selección de otro tamaño de lienzo (tras `new` o `resize`) ya no
        // vale.
        if let Some(sel) = self.selection
            && (sel.x + sel.w > vm.canvas.width() || sel.y + sel.h > vm.canvas.height())
        {
            self.selection = None;
        }
        let edit_tool = matches!(self.tool, Tool::Select | Tool::Move);
        let events = self
            .canvas
            .show(ui, vm, self.grid, hand, draw_tool.is_some() || edit_tool);
        if edit_tool {
            for event in events {
                if self.tool == Tool::Select {
                    self.edit.select(event, &vm.canvas, &mut self.selection);
                } else if let Some(result) =
                    self.edit.drag_move(event, &vm.canvas, &mut self.selection)
                {
                    out.push(Intent::Apply {
                        command: result.command,
                        label: result.label,
                    });
                }
            }
            self.paint_edit(ui, area, vm);
            return;
        }
        for event in events {
            let preview = self.machine.preview(&vm.canvas, &self.options);
            match self.machine.handle(
                event,
                &vm.canvas,
                self.primary,
                self.secondary,
                &self.options,
            ) {
                ToolResult::Commit { command, label } => {
                    if let Some((cells, color)) = preview {
                        self.pending = Some((vm.revision, cells, color));
                    }
                    out.push(Intent::Apply { command, label });
                }
                ToolResult::PickColor { color, button } => match button {
                    Button::Primary => self.primary = color,
                    Button::Secondary => self.secondary = color,
                },
                ToolResult::Nothing => {}
            }
        }
        if let Some((cells, color)) = self.machine.preview(&vm.canvas, &self.options) {
            self.canvas.paint_preview(ui, area, &cells, color);
        } else if let Some((revision, cells, color)) = &self.pending {
            if *revision == vm.revision {
                self.canvas.paint_preview(ui, area, cells, *color);
            } else {
                self.pending = None;
            }
        }
        if let Some(sel) = &self.selection {
            self.canvas.paint_selection(ui, area, sel);
        }
        if self.show_commands {
            self.canvas.paint_last_change(ui, area, vm);
        }
    }

    /// Banda «Versión N de M» encima del lienzo: anterior, siguiente,
    /// «Volver a esta» y «Ver la actual».
    fn version_banner(
        &self,
        ui: &mut egui::Ui,
        area: Rect,
        preview: &VersionPreview,
        out: &mut Vec<Intent>,
    ) {
        let rect = Rect::from_center_size(
            pos2(area.center().x, area.top() + 30.0),
            vec2(area.width().min(620.0), 40.0),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                Frame::NONE
                    .fill(theme::BAR)
                    .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                    .corner_radius(CornerRadius::same(theme::RADIUS_CARD_SMALL))
                    .inner_margin(Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let previous = ui
                            .add_enabled(preview.number > 1, egui::Button::new("Anterior"))
                            .accessible_name("Versión anterior");
                        if previous.clicked() {
                            out.push(Intent::ShowVersion(preview.number - 1));
                        }
                        ui.label(
                            RichText::new(format!(
                                "Versión {} de {}",
                                preview.number, preview.total
                            ))
                            .font(theme::semibold(13.0))
                            .color(theme::TEXT),
                        );
                        let next = ui
                            .add_enabled(
                                preview.number < preview.total,
                                egui::Button::new("Siguiente"),
                            )
                            .accessible_name("Versión siguiente");
                        if next.clicked() {
                            out.push(Intent::ShowVersion(preview.number + 1));
                        }
                        ui.label(
                            RichText::new(&preview.message)
                                .size(12.5)
                                .color(theme::TEXT_2),
                        );
                        let restore = egui::Button::new(
                            RichText::new("Volver a esta")
                                .font(theme::semibold(12.5))
                                .color(theme::ACCENT_INK),
                        )
                        .fill(theme::ACCENT);
                        if ui.add(restore).clicked() {
                            out.push(Intent::RestoreVersion(preview.number));
                        }
                        if ui.button("Ver la actual").clicked() {
                            out.push(Intent::ClosePreview);
                        }
                    });
            },
        );
    }

    /// Vista previa de Selección y Mover.
    fn paint_edit(&self, ui: &egui::Ui, area: Rect, vm: &ViewModel) {
        let (w, h) = (vm.canvas.width(), vm.canvas.height());
        if let Some((sel, dx, dy)) = self.edit.moving() {
            let moved: Vec<_> = selected_pixels(&vm.canvas, &sel)
                .into_iter()
                .map(|(x, y, c)| (x + dx, y + dy, c))
                .collect();
            self.canvas.paint_cleared(ui, area, &sel);
            self.canvas.paint_pixels(ui, area, &moved, w, h);
            if self.selection.is_some()
                && let Some(target) = sel.offset(dx, dy, &vm.canvas)
            {
                self.canvas.paint_selection(ui, area, &target);
            }
        } else if let Some(sel) = self.edit.selecting(&vm.canvas) {
            self.canvas.paint_selection(ui, area, &sel);
        } else if let Some(sel) = &self.selection {
            self.canvas.paint_selection(ui, area, sel);
        }
    }

    /// Atajos: herramientas (B, E, G, L, U, O, I, M, V, H), Esc, deshacer
    /// (Ctrl+Z) y rehacer (Ctrl+Mayús+Z o Ctrl+Y). Solo si no se escribe en un
    /// campo, y una vez por fotograma.
    fn shortcuts(&mut self, ui: &mut egui::Ui, vm: &ViewModel, out: &mut Vec<Intent>) {
        if ui.ctx().current_pass_index() != 0 || ui.memory(|m| m.focused().is_some()) {
            return;
        }
        // F2: renombrar la pestaña activa.
        if ui.input(|i| i.key_pressed(egui::Key::F2))
            && let Some(tab) = self.tabs.get(self.active_tab)
        {
            self.renaming = Some(tab.title.trim_end_matches(".lume").to_string());
            self.rename_fresh = true;
        }
        let (file, tool, escape, undo, redo) = ui.input(|i| {
            let file = if pressed(i, egui::Key::S, CMD | SHIFT) {
                Some(Intent::SaveAs)
            } else if pressed(i, egui::Key::S, CMD) {
                Some(Intent::Save)
            } else if pressed(i, egui::Key::O, CMD) {
                Some(Intent::OpenFile)
            } else if pressed(i, egui::Key::N, CMD) {
                Some(Intent::GoHome)
            } else if pressed(i, egui::Key::E, CMD) {
                Some(Intent::Export)
            } else if pressed(i, egui::Key::Comma, CMD) {
                Some(Intent::OpenSettings)
            } else {
                None
            };
            let tool = Tool::DRAW
                .into_iter()
                .chain(Tool::EDIT)
                .find(|t| pressed(i, t.egui_key(), PLAIN));
            let undo = pressed(i, egui::Key::Z, CMD);
            let redo = pressed(i, egui::Key::Z, CMD | SHIFT) || pressed(i, egui::Key::Y, CMD);
            (file, tool, pressed(i, egui::Key::Escape, PLAIN), undo, redo)
        });
        if let Some(intent) = file {
            out.push(intent);
        }
        if self.mode == Mode::Vector {
            // V, L, G y Esc son del modo vector.
            ui.input(|i| self.vec_shortcuts(i));
        } else if let Some(tool) = tool {
            self.tool = tool;
        }
        if escape && self.mode != Mode::Vector {
            // Primero lo que esté en curso; si no hay nada, la selección.
            if self.edit.is_active() || self.machine.is_active() || self.canvas.is_drawing() {
                self.edit.cancel();
                self.machine.cancel();
                self.canvas.cancel_drawing();
            } else {
                self.selection = None;
            }
        }
        if undo && vm.can_undo {
            out.push(Intent::Undo);
        }
        if redo && vm.can_redo {
            out.push(Intent::Redo);
        }
    }

    fn bottom_dock(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
        let full = ui.max_rect();
        ui.painter()
            .hline(full.x_range(), full.top(), Stroke::new(1.0, theme::LINE));
        // Paleta (resto) · Vista previa (236). El historial está en su
        // pestaña del panel del agente.
        let preview = Rect::from_min_max(pos2(full.right() - 236.0, full.top()), full.max);
        let palette = Rect::from_min_max(full.min, pos2(preview.left(), full.bottom()));
        ui.painter().vline(
            preview.left(),
            preview.y_range(),
            Stroke::new(1.0, theme::LINE),
        );
        let inner =
            |r: Rect| Rect::from_min_max(r.min + vec2(16.0, 12.0), r.max - vec2(16.0, 12.0));
        ui.scope_builder(UiBuilder::new().max_rect(inner(palette)), |ui| {
            self.palette_section(ui, vm)
        });
        ui.scope_builder(UiBuilder::new().max_rect(inner(preview)), |ui| {
            self.preview_section(ui, vm)
        });
    }

    /// Paleta del dibujo: casillas de 28 × 28 (clic: principal; clic
    /// derecho: secundario), «+» para elegir otro color y el color principal
    /// con su nombre.
    fn palette_section(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
        ui.spacing_mut().item_spacing.y = 10.0;
        ui.horizontal(|ui| {
            panel_label(ui, "Paleta del dibujo");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let n = vm.palette.len();
                let text = if n == 1 {
                    "1 color".to_string()
                } else {
                    format!("{n} colores")
                };
                ui.label(RichText::new(text).size(12.0).color(theme::TEXT_3));
            });
        });
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for entry in vm.palette.colors() {
                let (rect, response) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
                let hex = hex_label(entry.color);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, true, hex.clone())
                });
                let response = match &entry.name {
                    Some(name) => response.on_hover_text(format!("{hex} · {name}")),
                    None => response.on_hover_text(&hex),
                };
                let painter = ui.painter();
                if entry.color == self.primary {
                    painter.rect_stroke(
                        rect.expand(3.0),
                        CornerRadius::same(8),
                        Stroke::new(1.0, theme::TEXT),
                        StrokeKind::Inside,
                    );
                    painter.rect_stroke(
                        rect,
                        CornerRadius::same(6),
                        Stroke::new(2.0, theme::BAR),
                        StrokeKind::Outside,
                    );
                }
                painter.rect_filled(rect, CornerRadius::same(6), color32(entry.color));
                if response.clicked() {
                    self.primary = entry.color;
                }
                if response.secondary_clicked() {
                    self.secondary = entry.color;
                }
            }
            // «+»: elegir otro color principal.
            let (rect, response) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Button, true, "Añadir color")
            });
            let painter = ui.painter();
            for (a, b) in [
                (rect.left_top(), rect.right_top()),
                (rect.right_top(), rect.right_bottom()),
                (rect.right_bottom(), rect.left_bottom()),
                (rect.left_bottom(), rect.left_top()),
            ] {
                painter.add(egui::Shape::dashed_line(
                    &[a, b],
                    Stroke::new(1.0, Color32::from_rgb(0x3A, 0x41, 0x4D)),
                    3.0,
                    2.0,
                ));
            }
            painter.hline(
                (rect.center().x - 5.0)..=(rect.center().x + 5.0),
                rect.center().y,
                Stroke::new(1.6, theme::TEXT_3),
            );
            painter.vline(
                rect.center().x,
                (rect.center().y - 5.0)..=(rect.center().y + 5.0),
                Stroke::new(1.6, theme::TEXT_3),
            );
            color_popup(&response, &mut self.primary, None);
        });
        let name = vm
            .palette
            .colors()
            .iter()
            .find(|c| c.color == self.primary)
            .and_then(|c| c.name.clone());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.label(
                RichText::new(hex_label(self.primary))
                    .font(theme::mono(12.0))
                    .color(theme::TEXT_2),
            );
            if let Some(name) = name {
                ui.label(
                    RichText::new(format!("· {name}"))
                        .font(theme::mono(12.0))
                        .color(theme::TEXT_4),
                );
            }
        });
    }

    /// Vista previa a 1×, a 2× y a 2× sobre fondo claro.
    fn preview_section(&self, ui: &mut egui::Ui, vm: &ViewModel) {
        ui.spacing_mut().item_spacing.y = 10.0;
        panel_label(ui, "Vista previa");
        let (w, h) = (vm.canvas.width() as f32, vm.canvas.height() as f32);
        // Como mucho 32 px de alto a 1× y 64 a 2× (los lienzos grandes se reducen).
        let fit = |k: f32, max: f32| {
            let s = (max / w.max(h)).min(k);
            vec2(w * s, h * s)
        };
        let Some(texture) = self.canvas.texture_id() else {
            return;
        };
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            for (size, light, label) in [
                (fit(1.0, 32.0), false, "1×"),
                (fit(2.0, 64.0), false, "2×"),
                (fit(2.0, 64.0), true, "fondo claro"),
            ] {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;
                    let box_size = if light {
                        vec2(64.0, 64.0)
                    } else {
                        size.max(vec2(1.0, 64.0 - 0.0)).min(vec2(64.0, 64.0))
                    };
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(box_size.x.max(size.x), 64.0), Sense::hover());
                    let image_rect = Rect::from_center_size(
                        pos2(
                            rect.center().x,
                            rect.bottom()
                                - size.y / 2.0
                                - if light { (64.0 - size.y) / 2.0 } else { 0.0 },
                        ),
                        size,
                    );
                    if light {
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(6), theme::PREVIEW_LIGHT);
                    }
                    ui.painter().image(texture, image_rect, uv, Color32::WHITE);
                    ui.label(RichText::new(label).size(11.0).color(theme::TEXT_3));
                });
            }
        });
    }

    /// El panel del agente. Con `vm` (pixel art) lleva las pestañas Código
    /// e Historial.
    fn agent_panel(&mut self, ui: &mut egui::Ui, out: &mut Vec<Intent>, vm: Option<&ViewModel>) {
        let full = ui.max_rect();
        ui.painter().vline(
            full.left() - 16.0,
            (full.top() - 16.0)..=(full.bottom() + 16.0),
            Stroke::new(1.0, theme::LINE),
        );
        let view = self.agent_view.clone();
        let Some(vm) = vm else {
            self.agent.show(ui, &view, out, None);
            return;
        };
        // El panel se saca un momento para que las pestañas puedan leer el
        // resto del armazón.
        let mut agent = std::mem::take(&mut self.agent);
        let mut content = |ui: &mut egui::Ui, tab: PanelTab, out: &mut Vec<Intent>| match tab {
            PanelTab::Code => {
                crate::panel_tabs::code_tab(ui, &mut self.code_log, &self.canvas.brush)
            }
            PanelTab::History => {
                crate::panel_tabs::history_tab(ui, vm, &mut self.history_thumbs, out)
            }
            PanelTab::Agent => {}
        };
        agent.show(ui, &view, out, Some(&mut content));
        self.agent = agent;
    }

    fn status_bar(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
        let full = ui.max_rect();
        ui.painter().hline(
            (full.left() - 14.0)..=(full.right() + 14.0),
            full.top(),
            Stroke::new(1.0, theme::LINE),
        );
        let font = FontId::monospace(11.5);
        let text = |s: String| RichText::new(s).font(font.clone()).color(theme::TEXT_3);
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 18.0;
            // Las coordenadas, el color bajo el cursor y el zoom tienen sitio
            // fijo: al pasar el ratón por el lienzo nada se desplaza.
            let (w, h) = (vm.canvas.width(), vm.canvas.height());
            let nines = "9".repeat(digits(u64::from(w.max(h).saturating_sub(1))));
            let coords = match self.canvas.hover {
                Some((x, y)) => format!("x {x} · y {y}"),
                None => "x – · y –".into(),
            };
            fixed_label(
                ui,
                text(coords),
                &font,
                &[&format!("x {nines} · y {nines}"), "x – · y –"],
            );
            // Muestra y código del color. Va en una fila del alto del texto:
            // `ui.horizontal` pide más alto que la barra, y la barra crecía
            // (y con ella se movía todo) al entrar el ratón en el lienzo.
            let hex_size = text_size(ui, &font, &["#FFFFFFFF"]);
            let slot = vec2(9.0 + 6.0 + hex_size.x, hex_size.y);
            ui.allocate_ui_with_layout(slot, Layout::left_to_right(Align::Center), |ui| {
                ui.set_min_width(slot.x);
                ui.spacing_mut().item_spacing.x = 6.0;
                let (r, _) = ui.allocate_exact_size(vec2(9.0, 9.0), Sense::hover());
                if let Some(color) = self.canvas.hover_color(vm) {
                    ui.painter()
                        .rect_filled(r, CornerRadius::same(2), color32(color));
                    ui.label(text(hex_label(color)));
                }
            });
            ui.label(text(format!("{w} × {h} px")));
            fixed_label(
                ui,
                text(format!("{}%", zoom_percent(self.canvas.zoom))),
                &font,
                &["9999%"],
            );
            if let Some(notice) = &self.notice {
                ui.label(RichText::new(notice).size(11.5).color(theme::OK));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let provider = match self.agent_view.settings.provider {
                    lume_app::ProviderKind::ClaudeCode => "Claude Code",
                    lume_app::ProviderKind::Anthropic => "Anthropic",
                    lume_app::ProviderKind::OpenAi => "OpenAI",
                    lume_app::ProviderKind::Local => "Local",
                };
                ui.label(text(format!(
                    "{provider} · {}",
                    crate::agent_panel::tokens_label(self.agent.tokens)
                )));
                if let Some(server) = &self.server {
                    ui.label(text(format!("servidor local {server}")));
                }
            });
        });
    }
}

/// Una versión del documento que se está viendo en el lienzo.
#[derive(Debug, Clone, PartialEq)]
pub struct VersionPreview {
    pub number: u32,
    /// Cuántas versiones hay.
    pub total: u32,
    /// Su mensaje («Vuelta 3 de 8»).
    pub message: String,
    pub canvas: std::sync::Arc<lume_domain::Canvas>,
    /// Su dibujo vectorial, si el documento tenía (se ve en modo vector).
    pub vector: Option<std::sync::Arc<lume_domain::vec::drawing::VectorDrawing>>,
}

/// Texto de una entrada del historial: las etiquetas de las herramientas ya
/// dicen cuántos píxeles («Lápiz · 3 píxeles»); las demás, cuántos comandos
/// («batch · 22 comandos»).
pub fn history_text(item: &lume_app::HistoryItem) -> String {
    if item.label.contains(" · ") {
        item.label.clone()
    } else if item.commands == 1 {
        format!("{} · 1 comando", item.label)
    } else {
        format!("{} · {} comandos", item.label, item.commands)
    }
}

/// Zoom en porcentaje («1700»).
fn zoom_percent(zoom: f32) -> u32 {
    (zoom * 100.0).round() as u32
}

/// Color como en la barra de estado: `#RRGGBB`, y `AA` si no es opaco.
fn hex_label(c: Rgba) -> String {
    if c.a == 255 {
        format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", c.r, c.g, c.b, c.a)
    }
}

/// Una casilla de color de la barra de herramientas: se pulsa para abrir el
/// selector y se anuncia con su nombre.
fn swatch(ui: &mut egui::Ui, rect: Rect, name: &str, color: Rgba) -> egui::Response {
    let response = ui.interact(
        rect,
        ui.make_persistent_id(("casilla-color", name)),
        Sense::click(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name));
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(format!("{name} · {}", hex_label(color)))
}

/// El selector de color que abre `response` (una casilla o el «+»). Se queda
/// abierto mientras se elige y se cierra con un clic fuera o con Esc.
fn color_popup(response: &egui::Response, color: &mut Rgba, align: Option<egui::RectAlign>) {
    let mut popup =
        egui::Popup::menu(response).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
    if let Some(align) = align {
        popup = popup.align(align);
    }
    popup.show(|ui| {
        let mut picked = color32(*color);
        if egui::color_picker::color_picker_color32(
            ui,
            &mut picked,
            egui::color_picker::Alpha::OnlyBlend,
        ) {
            let [r, g, b, a] = picked.to_srgba_unmultiplied();
            *color = Rgba::new(r, g, b, a);
        }
    });
}

/// Una etiqueta de la barra de estado con el ancho del más largo de
/// `widest`: así lo que va detrás no se mueve cuando cambia el texto.
pub(crate) fn fixed_label(ui: &mut egui::Ui, text: RichText, font: &FontId, widest: &[&str]) {
    let size = text_size(ui, font, widest);
    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
        ui.set_min_width(size.x);
        ui.label(text);
    });
}

/// Lo que ocupa el más largo de `samples` escrito con `font`.
fn text_size(ui: &egui::Ui, font: &FontId, samples: &[&str]) -> egui::Vec2 {
    samples
        .iter()
        .map(|s| {
            ui.painter()
                .layout_no_wrap((*s).to_string(), font.clone(), Color32::WHITE)
                .size()
        })
        .fold(egui::Vec2::ZERO, |a, b| a.max(b))
}

/// Cifras de `n` (para reservar sitio a las coordenadas).
pub(crate) fn digits(n: u64) -> usize {
    n.max(1).ilog10() as usize + 1
}

/// Clave de la textura del icono en la memoria de egui.
fn icon_id() -> egui::Id {
    egui::Id::unique("lumeart-icono-app")
}

/// Icono de la aplicación (20 px) en la cabecera.
pub(crate) fn app_icon(ui: &mut egui::Ui) {
    let texture = ui
        .ctx()
        .memory_mut(|m| m.data.get_temp::<egui::TextureHandle>(icon_id()));
    let texture = texture.unwrap_or_else(|| {
        let icon = crate::window_icon();
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [icon.width as usize, icon.height as usize],
            &icon.rgba,
        );
        let handle = ui
            .ctx()
            .load_texture("icono-app", image, egui::TextureOptions::LINEAR);
        ui.ctx()
            .memory_mut(|m| m.data.insert_temp(icon_id(), handle.clone()));
        handle
    });
    ui.add(egui::Image::new(&texture).fit_to_exact_size(vec2(20.0, 20.0)));
}

/// El icono de la app a `side` píxeles (la pantalla de inicio).
pub(crate) fn app_icon_sized(ui: &mut egui::Ui, side: f32) {
    let id = icon_id().with("grande");
    let texture = ui
        .ctx()
        .memory_mut(|m| m.data.get_temp::<egui::TextureHandle>(id));
    let texture = texture.unwrap_or_else(|| {
        let png = include_bytes!("../../../assets/brand/icon-256.png");
        let (w, h, rgba) = crate::decode_png(png).expect("el icono de la marca es un PNG");
        let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
        let handle = ui
            .ctx()
            .load_texture("icono-app-grande", image, egui::TextureOptions::LINEAR);
        ui.ctx()
            .memory_mut(|m| m.data.insert_temp(id, handle.clone()));
        handle
    });
    ui.add(egui::Image::new(&texture).fit_to_exact_size(vec2(side, side)));
}

/// Pastilla acento con punto («Vuelta 3 de 8»): alto 26, punto de 7 px.
fn status_pill(ui: &mut egui::Ui, text: &str) {
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), FontId::proportional(12.0), theme::ACCENT);
    let size = vec2(8.0 + 7.0 + 8.0 + galley.size().x + 10.0, 26.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    // Para los lectores de pantalla (y los tests): el texto de la pastilla.
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, text));
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(13), theme::ACCENT_SOFT);
    painter.circle_filled(
        rect.left_center() + vec2(8.0 + 3.5, 0.0),
        3.5,
        theme::ACCENT,
    );
    let text_pos = pos2(
        rect.left() + 8.0 + 7.0 + 8.0,
        rect.center().y - galley.size().y / 2.0,
    );
    painter.galley(text_pos, galley, theme::ACCENT);
}

/// Sin modificadores, con Ctrl (⌘ en macOS) y con Mayúsculas.
const PLAIN: u8 = 0;
const CMD: u8 = 1;
const SHIFT: u8 = 2;

/// ¿Se pulsó `key` en este fotograma con exactamente esos modificadores?
///
/// Mira los modificadores de cada evento y no el estado al final del
/// fotograma: si Ctrl y la tecla se pulsan y se sueltan en el mismo fotograma
/// (un atajo muy rápido o enviado por otro programa), al acabar el fotograma
/// Ctrl ya no está pulsado y el atajo se perdería.
fn pressed(input: &egui::InputState, key: egui::Key, mods: u8) -> bool {
    input.events.iter().any(|e| match e {
        egui::Event::Key {
            key: k,
            pressed: true,
            repeat: false,
            modifiers,
            ..
        } => {
            *k == key
                && modifiers.command == (mods & CMD != 0)
                && modifiers.shift == (mods & SHIFT != 0)
                && !modifiers.alt
        }
        _ => false,
    })
}
