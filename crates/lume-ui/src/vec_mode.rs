//! Modo vector (`docs/diseno/capturas/03-studio-vector.png`): el dibujo que
//! sale de `dibujo.js`, la herramienta de selección (clic: la forma y su
//! línea), el **lazo de retoque** (rodea una zona, dice qué formas hay
//! debajo y la suma, con lo que hay que cambiar, a la cola de correcciones;
//! «Ejecutar correcciones» las pide todas al agente en una pasada),
//! «Repetir trazado» y el panel inferior con el código y las capas.
//!
//! Es un hijo de `shell` (comparte su cabecera, su panel del agente y sus
//! estilos). `StudioApp` ejecuta `dibujo.js`, rasteriza y busca las formas
//! en segundo plano y deja aquí el resultado ([`VecMode::set_view`]).

use std::sync::Arc;

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Frame, Layout, Margin, Panel, Pos2, Rect, RichText,
    Sense, Shape, Stroke, StrokeKind, TextureHandle, TextureOptions, UiBuilder, Vec2, pos2, vec2,
};
use lume_app::ViewModel;
use lume_app::ports::RasterImage;
use lume_domain::vec::VecResult;
use lume_domain::vec::studio::{LayerInfo, ShapeHit, Zone, coordinates};

use super::{AppShell, bar_frame, digits, fixed_label, icon_button, panel_label, vsep};
use crate::canvas_view::CanvasBackground;
use crate::icons::Icon;
use crate::intent::Intent;
use crate::theme;

/// Los trozos del dibujo ya pintados (cada capa sola y lo de entre capas),
/// para ocultar capas al instante. `layer` es el índice de la capa.
pub struct SliceImage {
    pub layer: Option<usize>,
    pub image: RasterImage,
}

/// Texturas de los trozos, con la capa de cada uno.
type SliceTextures = Vec<(Option<usize>, TextureHandle)>;

/// Zoom más grande y más pequeño respecto al ajustado.
const ZOOM_RANGE: (f32, f32) = (0.1, 16.0);

/// Alto de una línea del panel de código.
const CODE_ROW: f32 = 20.0;
/// Columna de los números de línea.
const GUTTER: f32 = 52.0;
/// Ancho del panel de capas.
pub const LAYERS_WIDTH: f32 = 236.0;
/// Alto de una fila de capas.
const LAYER_ROW: f32 = 30.0;
/// Margen alrededor del dibujo en el lienzo.
const CANVAS_MARGIN: f32 = 40.0;

/// Herramientas del modo vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VecTool {
    /// Clic en una forma: su línea en `dibujo.js` y su capa.
    #[default]
    Select,
    /// Rodear una zona para retocarla.
    Lasso,
    /// Arrastrar: un rectángulo (Mayús: cuadrado).
    Rect,
    /// Arrastrar: una elipse (Mayús: círculo).
    Ellipse,
    /// Arrastrar: una línea (Mayús: a 45°).
    Line,
    /// A mano alzada: un trazo de pincel.
    Brush,
    /// Clic y escribir: un texto.
    Text,
}

impl VecTool {
    pub const ALL: [VecTool; 7] = [
        VecTool::Select,
        VecTool::Lasso,
        VecTool::Rect,
        VecTool::Ellipse,
        VecTool::Line,
        VecTool::Brush,
        VecTool::Text,
    ];

    pub fn label(self) -> &'static str {
        match self {
            VecTool::Select => "Seleccionar",
            VecTool::Lasso => "Lazo",
            VecTool::Rect => "Rectángulo",
            VecTool::Ellipse => "Elipse",
            VecTool::Line => "Línea",
            VecTool::Brush => "Pincel",
            VecTool::Text => "Texto",
        }
    }

    pub fn key(self) -> char {
        match self {
            VecTool::Select => 'V',
            VecTool::Lasso => 'L',
            VecTool::Rect => 'R',
            VecTool::Ellipse => 'E',
            VecTool::Line => 'I',
            VecTool::Brush => 'B',
            VecTool::Text => 'T',
        }
    }

    fn egui_key(self) -> egui::Key {
        match self {
            VecTool::Select => egui::Key::V,
            VecTool::Lasso => egui::Key::L,
            VecTool::Rect => egui::Key::R,
            VecTool::Ellipse => egui::Key::E,
            VecTool::Line => egui::Key::I,
            VecTool::Brush => egui::Key::B,
            VecTool::Text => egui::Key::T,
        }
    }

    fn icon(self) -> Icon {
        match self {
            VecTool::Select => Icon::Pointer,
            VecTool::Lasso => Icon::Lasso,
            VecTool::Rect => Icon::Rect,
            VecTool::Ellipse => Icon::Ellipse,
            VecTool::Line => Icon::Line,
            VecTool::Brush => Icon::Pencil,
            VecTool::Text => Icon::Text,
        }
    }

    fn hint(self) -> &'static str {
        match self {
            VecTool::Select => "Haz clic en una forma para ver su línea en dibujo.js.",
            VecTool::Lasso => "Rodea una zona y di qué cambiar. Se suma a la cola de correcciones.",
            VecTool::Rect | VecTool::Ellipse | VecTool::Line => {
                "Arrastra para dibujar (Mayús: proporción fija)."
            }
            VecTool::Brush => "Arrastra para pintar un trazo.",
            VecTool::Text => "Haz clic donde va el texto y escríbelo.",
        }
    }

    /// La forma que dibuja (las herramientas de arrastrar).
    fn shape(self) -> Option<crate::vec_draw::Shape> {
        use crate::vec_draw::Shape;
        match self {
            VecTool::Rect => Some(Shape::Rect),
            VecTool::Ellipse => Some(Shape::Ellipse),
            VecTool::Line => Some(Shape::Line),
            VecTool::Brush => Some(Shape::Brush),
            _ => None,
        }
    }

    /// ¿Dibuja algo nuevo? (Las demás miran o señalan.)
    pub fn draws(self) -> bool {
        self.shape().is_some() || self == VecTool::Text
    }
}

/// El dibujo calculado: lo que dio `dibujo.js` y su imagen.
#[derive(Debug, Clone)]
pub struct VecView {
    /// El código que se ejecutó.
    pub source: String,
    pub result: VecResult,
    /// Las capas del dibujo, en orden de dibujo.
    pub layers: Vec<LayerInfo>,
    /// La imagen del dibujo entero (las capas ocultas se quitan con los
    /// trozos de `VecMode::slices`), o por qué no se pudo pintar.
    pub image: Result<RasterImage, String>,
    /// Cambia con cada imagen nueva (para la textura).
    pub key: u64,
}

/// Un paso de «Repetir trazado».
#[derive(Debug, Clone)]
pub struct ReplayFrame {
    pub key: u64,
    pub image: Arc<RasterImage>,
    /// Formas que se ven y cuántas hay.
    pub shown: usize,
    pub total: usize,
}

/// Una zona rodeada con el lazo: la que se está escribiendo o una
/// corrección de la cola.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneState {
    /// Su número en la cola (1, 2, 3…; la que se escribe lleva el siguiente).
    pub number: u32,
    pub zone: Zone,
    /// Lo que se pide para la zona.
    pub text: String,
    /// Ya se pidió al agente (está en curso).
    pub sent: bool,
}

/// Estado del modo vector.
#[derive(Default)]
pub struct VecMode {
    /// El dibujo en pantalla (el del documento o el anterior mientras se
    /// recalcula).
    pub view: Option<Arc<VecView>>,
    /// No hay motor de `dibujo.js` en esta ventana.
    pub no_engine: bool,
    pub tool: VecTool,
    /// Rejilla de coordenadas sobre el dibujo.
    pub grid: bool,
    /// Lazo que se está trazando (en unidades del dibujo).
    pub lasso: Vec<[f64; 2]>,
    /// Lazo terminado que espera sus formas (lo recoge `StudioApp`).
    pub pending_zone: Option<Vec<[f64; 2]>>,
    /// Clic de selección que espera su forma.
    pub click: Option<[f64; 2]>,
    /// La zona rodeada que aún se está escribiendo.
    pub zone: Option<ZoneState>,
    /// Cola de correcciones: zonas con su texto que esperan a «Ejecutar
    /// correcciones» (o que el agente está haciendo, con `sent`).
    pub corrections: Vec<ZoneState>,
    /// Corrección de la cola bajo el ratón (se resalta en el dibujo).
    pub hovered_correction: Option<usize>,
    /// La forma elegida con un clic.
    pub selected: Option<ShapeHit>,
    /// Capas ocultas (por su índice en `VecView::layers`).
    pub hidden: Vec<usize>,
    /// Líneas que acaba de cambiar el agente (de 1 en adelante).
    pub agent_lines: Option<(usize, usize)>,
    /// Línea a la que llevar el panel de código.
    pub scroll_to: Option<usize>,
    /// «Repetir trazado» en marcha.
    pub replay: Option<ReplayFrame>,
    /// Punto del dibujo bajo el ratón.
    pub cursor: Option<[f64; 2]>,
    /// Píxeles de pantalla por unidad del dibujo.
    pub zoom: f32,
    /// Dónde está el dibujo en pantalla (en el último fotograma).
    pub image_rect: Option<Rect>,
    texture: Option<(u64, TextureHandle)>,
    /// Llevar el foco al campo de la zona en el próximo fotograma.
    focus_zone: bool,
    /// Zoom elegido por el usuario (`None`: el dibujo se ajusta al área).
    pub view_zoom: Option<f32>,
    /// Desplazamiento del dibujo respecto al centro del área.
    pub pan: Vec2,
    /// Detrás del dibujo.
    pub background: CanvasBackground,
    /// Trozos del dibujo de la clave `.0` (para ocultar capas al instante).
    pub slices: Option<(u64, Arc<Vec<SliceImage>>)>,
    /// El diálogo «Exportar» abierto: formato y ancho del PNG.
    pub export: Option<(crate::intent::VecFormat, u32)>,
    slice_textures: Option<(u64, SliceTextures)>,
    /// Tamaño y fondo del dibujo en blanco que se va a crear.
    pub new_size: crate::size_picker::SizeChoice,
    pub new_transparent: bool,
    /// Colores y grosor de las herramientas de dibujo.
    pub style: crate::vec_draw::DrawStyle,
    /// La forma que se está arrastrando (puntos en unidades del dibujo).
    pub draft: Vec<[f64; 2]>,
    /// El texto que se está escribiendo y dónde.
    pub typing: Option<([f64; 2], String)>,
}

impl std::fmt::Debug for VecMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecMode")
            .field("tool", &self.tool)
            .field("zone", &self.zone)
            .field("selected", &self.selected)
            .finish_non_exhaustive()
    }
}

impl Clone for VecMode {
    fn clone(&self) -> Self {
        Self {
            view: self.view.clone(),
            no_engine: self.no_engine,
            tool: self.tool,
            grid: self.grid,
            lasso: self.lasso.clone(),
            pending_zone: self.pending_zone.clone(),
            click: self.click,
            zone: self.zone.clone(),
            corrections: self.corrections.clone(),
            hovered_correction: self.hovered_correction,
            selected: self.selected.clone(),
            hidden: self.hidden.clone(),
            agent_lines: self.agent_lines,
            scroll_to: self.scroll_to,
            replay: self.replay.clone(),
            cursor: self.cursor,
            zoom: self.zoom,
            image_rect: self.image_rect,
            texture: self.texture.clone(),
            focus_zone: self.focus_zone,
            view_zoom: self.view_zoom,
            pan: self.pan,
            background: self.background,
            slices: self.slices.clone(),
            export: self.export,
            slice_textures: self.slice_textures.clone(),
            new_size: self.new_size.clone(),
            new_transparent: self.new_transparent,
            style: self.style,
            draft: self.draft.clone(),
            typing: self.typing.clone(),
        }
    }
}

impl VecMode {
    /// Pone un dibujo recién calculado. Si cambian las capas, se olvidan
    /// las ocultas que ya no existen.
    pub fn set_view(&mut self, view: VecView) {
        let layers = view.layers.len();
        self.hidden.retain(|&i| i < layers);
        self.view = Some(Arc::new(view));
    }

    /// Sin dibujo (el documento no tiene `dibujo.js`).
    pub fn clear(&mut self) {
        self.view = None;
        self.texture = None;
        self.slices = None;
        self.slice_textures = None;
        self.zone = None;
        self.corrections.clear();
        self.hovered_correction = None;
        self.selected = None;
        self.lasso.clear();
        self.pending_zone = None;
        self.agent_lines = None;
        self.replay = None;
        self.image_rect = None;
    }

    /// Pone la zona que salió del lazo (con sus formas) y lleva el código a
    /// su primera línea.
    pub fn set_zone(&mut self, zone: Zone) {
        if let Some(&line) = zone.lines().first() {
            self.scroll_to = Some(line as usize);
        }
        self.zone = Some(ZoneState {
            number: self.corrections.len() as u32 + 1,
            zone,
            text: String::new(),
            sent: false,
        });
        self.focus_zone = true;
    }

    /// Pasa la zona que se escribe a la cola de correcciones (si dice qué
    /// cambiar). No se pide nada al agente hasta «Ejecutar correcciones».
    pub fn queue_zone(&mut self) -> bool {
        let Some(mut zone) = self.zone.take_if(|z| !z.text.trim().is_empty()) else {
            return false;
        };
        zone.text = zone.text.trim().to_string();
        zone.sent = false;
        self.corrections.push(zone);
        self.renumber();
        true
    }

    /// Quita de la cola la corrección `i` si aún no se ha ejecutado.
    pub fn remove_correction(&mut self, i: usize) {
        if self.corrections.get(i).is_some_and(|c| !c.sent) {
            self.corrections.remove(i);
            self.hovered_correction = None;
            self.renumber();
        }
    }

    /// Correcciones de la cola que esperan.
    pub fn pending_corrections(&self) -> usize {
        self.corrections.iter().filter(|c| !c.sent).count()
    }

    /// Terminó la pasada del agente: fuera las correcciones hechas; si
    /// falló, vuelven a esperar.
    pub fn corrections_finished(&mut self, failed: bool) {
        finish_corrections(&mut self.corrections, failed);
        self.hovered_correction = None;
        self.renumber();
    }

    /// Numera la cola 1, 2, 3… y la zona que se escribe con el siguiente.
    fn renumber(&mut self) {
        for (i, c) in self.corrections.iter_mut().enumerate() {
            c.number = i as u32 + 1;
        }
        let next = self.corrections.len() as u32 + 1;
        if let Some(zone) = &mut self.zone {
            zone.number = next;
        }
    }

    /// Pone la forma elegida con un clic y lleva el código a su línea.
    pub fn set_selected(&mut self, hit: Option<ShapeHit>) {
        if let Some(hit) = &hit {
            self.scroll_to = Some(hit.line as usize);
        }
        self.selected = hit;
    }

    /// El punto de la pantalla de un punto del dibujo (si se ve).
    pub fn screen_point(&self, p: [f64; 2]) -> Option<Pos2> {
        let r = self.image_rect?;
        Some(pos2(
            r.left() + p[0] as f32 * self.zoom,
            r.top() + p[1] as f32 * self.zoom,
        ))
    }

    /// Tamaño del dibujo, si hay uno.
    pub fn size(&self) -> Option<(i64, i64)> {
        self.view
            .as_ref()
            .map(|v| (v.result.width, v.result.height))
    }

    /// La capa que se resalta: la de la zona o la de la forma elegida.
    fn highlighted_layer(&self) -> Option<&str> {
        match (&self.zone, &self.selected) {
            (Some(zone), _) => zone.zone.main_layer(),
            (None, Some(hit)) if !hit.layer.is_empty() => Some(hit.layer.as_str()),
            _ => None,
        }
    }

    /// La textura de lo que se ve (el paso de «Repetir trazado» o el
    /// dibujo).
    fn texture(&mut self, ctx: &egui::Context) -> Option<(TextureHandle, [usize; 2])> {
        let (key, image): (u64, &RasterImage) = match (&self.replay, &self.view) {
            (Some(frame), _) => (frame.key, &frame.image),
            (None, Some(view)) => (view.key, view.image.as_ref().ok()?),
            (None, None) => return None,
        };
        let size = [image.width as usize, image.height as usize];
        match &self.texture {
            Some((k, tex)) if *k == key => Some((tex.clone(), size)),
            _ => {
                let color = egui::ColorImage::from_rgba_unmultiplied(size, &image.rgba);
                let tex = ctx.load_texture(format!("vector-{key}"), color, TextureOptions::LINEAR);
                self.texture = Some((key, tex.clone()));
                Some((tex, size))
            }
        }
    }
}

impl VecMode {
    /// Las texturas de los trozos (si ya están pintados para el dibujo que
    /// se ve), en orden de pintado.
    fn slice_textures(&mut self, ctx: &egui::Context) -> Option<&[(Option<usize>, TextureHandle)]> {
        let key = self.view.as_ref()?.key;
        let (slice_key, slices) = self.slices.as_ref()?;
        if *slice_key != key {
            return None;
        }
        if self.slice_textures.as_ref().is_none_or(|(k, _)| *k != key) {
            let textures = slices
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let size = [s.image.width as usize, s.image.height as usize];
                    let color = egui::ColorImage::from_rgba_unmultiplied(size, &s.image.rgba);
                    let tex = ctx.load_texture(
                        format!("vector-{key}-trozo-{i}"),
                        color,
                        TextureOptions::LINEAR,
                    );
                    (s.layer, tex)
                })
                .collect();
            self.slice_textures = Some((key, textures));
        }
        self.slice_textures.as_ref().map(|(_, t)| t.as_slice())
    }
}

/// Paso redondo de la rejilla: unas diez divisiones en el lado mayor.
pub fn grid_step(size: f64) -> f64 {
    [10.0, 20.0, 25.0, 50.0, 100.0, 200.0, 250.0, 500.0, 1000.0]
        .into_iter()
        .find(|step| size / step <= 12.0)
        .unwrap_or(1000.0)
}

/// Terminó la pasada del agente sobre una cola: fuera las correcciones
/// hechas (si falló, vuelven a esperar) y se numeran de nuevo.
pub fn finish_corrections(list: &mut Vec<ZoneState>, failed: bool) {
    if failed {
        for c in list.iter_mut() {
            c.sent = false;
        }
    } else {
        list.retain(|c| !c.sent);
    }
    for (i, c) in list.iter_mut().enumerate() {
        c.number = i as u32 + 1;
    }
}

/// Lo que pasó con una fila de la cola de correcciones.
struct RowResponse {
    hovered: bool,
    remove: bool,
}

/// Una fila de la cola: su número, su texto, qué hay en la zona y la ×
/// para quitarla (solo si aún no se ha ejecutado).
fn correction_row(ui: &mut egui::Ui, c: &ZoneState) -> RowResponse {
    let mut remove = false;
    let inner = ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (badge, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
        let color = if c.sent {
            theme::ACCENT.gamma_multiply(0.6)
        } else {
            theme::ACCENT
        };
        ui.painter().circle_filled(badge.center(), 10.0, color);
        ui.painter().text(
            badge.center(),
            Align2::CENTER_CENTER,
            c.number.to_string(),
            theme::semibold(11.0),
            theme::ACCENT_INK,
        );
        let text_width = (ui.available_width() - 28.0).max(60.0);
        ui.vertical(|ui| {
            ui.set_width(text_width);
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.add(
                egui::Label::new(RichText::new(&c.text).size(12.5).color(theme::TEXT_BODY)).wrap(),
            );
            let summary = c.zone.summary();
            let sub = if c.sent {
                format!("en curso · {summary}")
            } else {
                summary
            };
            ui.add(egui::Label::new(RichText::new(sub).size(11.0).color(theme::TEXT_4)).wrap());
        });
        if !c.sent {
            let label = format!("Quitar la corrección {}", c.number);
            let (rect, response) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
            response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &label));
            let response = response.on_hover_text(&label);
            if response.hovered() {
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(4), theme::HOVER);
            }
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "×",
                egui::FontId::proportional(15.0),
                if response.hovered() {
                    theme::TEXT
                } else {
                    theme::TEXT_3
                },
            );
            remove = response.clicked();
        }
    });
    RowResponse {
        hovered: ui.rect_contains_pointer(inner.response.rect),
        remove,
    }
}

/// «líneas 3, 4, 9» (como mucho seis).
fn lines_text(lines: &[u32]) -> String {
    match lines {
        [] => "sin líneas propias".into(),
        [one] => format!("línea {one}"),
        _ => {
            let mut parts: Vec<String> = lines.iter().take(6).map(u32::to_string).collect();
            if lines.len() > 6 {
                parts.push("…".into());
            }
            format!("líneas {}", parts.join(", "))
        }
    }
}

/// Pinta el interior de un polígono (aunque sea cóncavo) por franjas de
/// `step` píxeles: lo bastante fino para el relleno del lazo.
fn fill_polygon(painter: &egui::Painter, pts: &[Pos2], color: Color32) {
    if pts.len() < 3 {
        return;
    }
    let step = 2.0;
    let (top, bottom) = pts
        .iter()
        .fold((f32::MAX, f32::MIN), |(t, b), p| (t.min(p.y), b.max(p.y)));
    let mut y = top + step / 2.0;
    let mut xs: Vec<f32> = Vec::new();
    while y < bottom {
        xs.clear();
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            if (a.y <= y) != (b.y <= y) {
                xs.push(a.x + (y - a.y) / (b.y - a.y) * (b.x - a.x));
            }
        }
        xs.sort_by(f32::total_cmp);
        for pair in xs.chunks_exact(2) {
            painter.rect_filled(
                Rect::from_min_max(pos2(pair[0], y - step / 2.0), pos2(pair[1], y + step / 2.0)),
                0.0,
                color,
            );
        }
        y += step;
    }
}

/// Rejilla de coordenadas: líneas finas verdes sobre el dibujo y, fuera de
/// él (sobre fondo oscuro), reglas arriba y a la izquierda con las marcas y
/// los números en el verde de la marca.
fn grid_overlay(painter: &egui::Painter, full: Rect, image: Rect, w: f32, h: f32, k: f32) {
    let step = grid_step(f64::from(w.max(h))) as f32;
    let line = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0x3D, 0xBE, 0x6B, 70));
    let clip = painter.with_clip_rect(image.intersect(full));
    let mut u = step;
    while u < w {
        clip.vline(image.left() + u * k, image.y_range(), line);
        u += step;
    }
    let mut u = step;
    while u < h {
        clip.hline(image.x_range(), image.top() + u * k, line);
        u += step;
    }
    // Reglas: siempre a la vista, aunque el dibujo se salga del área.
    const RULER: f32 = 22.0;
    let top = (image.top() - RULER).max(full.top());
    let left = (image.left() - RULER - 14.0).max(full.left());
    let font = egui::FontId::monospace(10.5);
    let tick = Stroke::new(1.0, theme::ACCENT);
    let strip_top = Rect::from_min_max(
        pos2(image.left().max(full.left()), top),
        pos2(image.right().min(full.right()), top + RULER),
    );
    let strip_left = Rect::from_min_max(
        pos2(left, image.top().max(full.top())),
        pos2(left + RULER + 14.0, image.bottom().min(full.bottom())),
    );
    painter.rect_filled(strip_top, 0.0, theme::BG);
    painter.rect_filled(strip_left, 0.0, theme::BG);
    let mut u = 0.0;
    while u <= w + 0.5 {
        let x = image.left() + u * k;
        if strip_top.x_range().contains(x) {
            painter.vline(x, (top + RULER - 7.0)..=(top + RULER), tick);
            painter.text(
                pos2(x + 3.0, top + 3.0),
                Align2::LEFT_TOP,
                format!("{u:.0}"),
                font.clone(),
                theme::ACCENT,
            );
        }
        u += step;
    }
    let mut u = 0.0;
    while u <= h + 0.5 {
        let y = image.top() + u * k;
        if strip_left.y_range().contains(y) {
            painter.hline((left + RULER + 7.0)..=(left + RULER + 14.0), y, tick);
            painter.text(
                pos2(left + RULER + 4.0, y + 2.0),
                Align2::RIGHT_TOP,
                format!("{u:.0}"),
                font.clone(),
                theme::ACCENT,
            );
        }
        u += step;
    }
}

/// Cuadros bajo un dibujo con fondo transparente.
fn checkers(painter: &egui::Painter, rect: Rect) {
    painter.rect_filled(rect, 0.0, theme::CHECKER_A);
    let cell = 12.0;
    let mut y = rect.top();
    let mut row = 0;
    while y < rect.bottom() {
        let mut x = rect.left() + if row % 2 == 0 { 0.0 } else { cell };
        while x < rect.right() {
            let r = Rect::from_min_max(
                pos2(x, y),
                pos2((x + cell).min(rect.right()), (y + cell).min(rect.bottom())),
            );
            painter.rect_filled(r, 0.0, theme::CHECKER_B);
            x += cell * 2.0;
        }
        y += cell;
        row += 1;
    }
}

/// Una línea de `dibujo.js` tal como se pinta (los tabuladores, a espacios).
fn code_line(line: &str) -> String {
    line.replace('\t', "    ")
}

impl AppShell {
    /// El armazón en modo vector.
    pub(crate) fn show_vector(&mut self, ui: &mut egui::Ui, vm: &ViewModel, out: &mut Vec<Intent>) {
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
            .show(ui, |ui| self.vec_status(ui));
        self.zones.status = status.response.rect;

        let agent = Panel::right("agente")
            .exact_size(theme::AGENT_WIDTH)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::same(16)))
            .show(ui, |ui| self.agent_panel(ui, out, None));
        self.zones.agent = agent.response.rect;

        let toolbar = Panel::left("herramientas")
            .exact_size(theme::TOOLBAR_WIDTH)
            .resizable(false)
            .frame(bar_frame(theme::BAR).inner_margin(Margin::symmetric(8, 10)))
            .show(ui, |ui| self.vec_toolbar(ui));
        self.zones.toolbar = toolbar.response.rect;

        self.tab_panel(ui, out);
        let options = Panel::top("opciones")
            .exact_size(theme::OPTIONS_HEIGHT)
            .resizable(false)
            .frame(bar_frame(theme::BAR_ALT).inner_margin(Margin::symmetric(14, 0)))
            .show(ui, |ui| self.vec_options(ui, vm, out));
        self.zones.options = options.response.rect;

        if vm.vector.is_some() || self.preview.is_some() {
            // Se agranda o encoge arrastrando su borde de arriba.
            let bottom = Panel::bottom("codigo-y-capas")
                .default_size(theme::BOTTOM_VECTOR_HEIGHT)
                .size_range(120.0..=720.0)
                .resizable(true)
                .frame(bar_frame(theme::BAR))
                .show(ui, |ui| self.vec_bottom(ui));
            self.zones.bottom = bottom.response.rect;
        } else {
            self.zones.bottom = Rect::NOTHING;
        }

        let canvas = egui::CentralPanel::default()
            .frame(Frame::NONE.fill(theme::BG))
            .show(ui, |ui| self.vec_canvas(ui, vm, out));
        self.zones.canvas = canvas.response.rect;
        self.vec_export_dialog(ui.ctx(), out);
    }

    /// «Exportar» en vector: formato (PNG, SVG, PDF, Illustrator) y, para el
    /// PNG, el ancho.
    fn vec_export_dialog(&mut self, ctx: &egui::Context, out: &mut Vec<Intent>) {
        use crate::intent::VecFormat;
        let Some((mut format, mut width)) = self.vec.export else {
            return;
        };
        let size = self.vec.size();
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::unique("lumeart-exportar-vector")).show(ctx, |ui| {
            ui.set_width(380.0);
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.label(
                RichText::new("Exportar el dibujo")
                    .font(theme::semibold(16.0))
                    .color(theme::TEXT),
            );
            ui.horizontal_wrapped(|ui| {
                for f in VecFormat::ALL {
                    if ui.selectable_label(format == f, f.label()).clicked() {
                        format = f;
                    }
                }
            });
            if format == VecFormat::Png {
                ui.horizontal(|ui| {
                    let label = ui.label(RichText::new("Ancho").size(13.0).color(theme::TEXT_2));
                    for w in [512u32, 1024, 2048, 4096] {
                        if ui.selectable_label(width == w, w.to_string()).clicked() {
                            width = w;
                        }
                    }
                    ui.add(
                        egui::DragValue::new(&mut width)
                            .range(16..=8192)
                            .suffix(" px"),
                    )
                    .labelled_by(label.id);
                });
                if let Some((w, h)) = size {
                    let height = (width as f64 * h as f64 / w.max(1) as f64).round();
                    ui.label(
                        RichText::new(format!("{width} × {height} px"))
                            .font(theme::mono(12.0))
                            .color(theme::TEXT_3),
                    );
                }
            } else {
                ui.label(
                    RichText::new(
                        "Con sus capas y formas, para seguir editándolo en otro programa.",
                    )
                    .size(12.5)
                    .color(theme::TEXT_3),
                );
            }
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
                        out.push(Intent::ExportVector { format, width });
                        close = true;
                    }
                });
            });
        });
        self.vec.export = if close || modal.should_close() {
            None
        } else {
            Some((format, width))
        };
    }

    /// Relleno, borde y grosor de las herramientas de dibujo.
    fn style_controls(&mut self, ui: &mut egui::Ui) {
        let tool = self.vec.tool;
        let style = &mut self.vec.style;
        let small = |t: &str| RichText::new(t).size(12.0).color(theme::TEXT_2);
        ui.spacing_mut().item_spacing.x = 8.0;
        let closed = matches!(tool, VecTool::Rect | VecTool::Ellipse);
        if closed || tool == VecTool::Text {
            ui.checkbox(&mut style.fill_on, small("Relleno"));
            ui.add_enabled_ui(style.fill_on, |ui| {
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut style.fill,
                    egui::color_picker::Alpha::OnlyBlend,
                )
                .on_hover_text("Color de relleno");
            });
        }
        if closed {
            ui.checkbox(&mut style.stroke_on, small("Borde"));
        }
        if (!closed || style.stroke_on) && tool != VecTool::Text {
            if !closed {
                ui.label(small("Color"));
            }
            egui::color_picker::color_edit_button_srgba(
                ui,
                &mut style.stroke,
                egui::color_picker::Alpha::OnlyBlend,
            )
            .on_hover_text("Color del trazo");
            ui.label(small("Grosor"));
            ui.add(
                egui::DragValue::new(&mut style.width)
                    .range(0.5..=200.0)
                    .speed(0.25)
                    .max_decimals(1),
            );
        }
        if tool == VecTool::Text {
            ui.label(small("Tamaño"));
            ui.add(
                egui::DragValue::new(&mut style.text_size)
                    .range(4.0..=600.0)
                    .speed(0.5)
                    .max_decimals(0),
            );
        }
    }

    /// Atajos del modo vector: V y L cambian de herramienta, G la rejilla y
    /// Esc suelta el lazo, la zona o la forma elegida.
    pub(crate) fn vec_shortcuts(&mut self, i: &egui::InputState) {
        let plain = |key| i.key_pressed(key) && i.modifiers.is_none();
        if let Some(tool) = VecTool::ALL.into_iter().find(|t| plain(t.egui_key())) {
            self.vec.tool = tool;
        }
        if plain(egui::Key::G) {
            self.vec.grid = !self.vec.grid;
        }
        if plain(egui::Key::Num0) {
            self.vec.view_zoom = None;
            self.vec.pan = Vec2::ZERO;
        }
        if plain(egui::Key::Escape) {
            if self.vec.typing.is_some() || !self.vec.draft.is_empty() {
                self.vec.typing = None;
                self.vec.draft.clear();
            } else if !self.vec.lasso.is_empty() {
                self.vec.lasso.clear();
            } else if self.vec.zone.is_some() {
                self.vec.zone = None;
            } else {
                self.vec.selected = None;
            }
        }
    }

    fn vec_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for tool in VecTool::ALL {
                if tool == VecTool::Rect {
                    ui.add_space(6.0);
                    let (rect, _) = ui.allocate_exact_size(vec2(24.0, 1.0), Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, theme::LINE_SOFT);
                    ui.add_space(6.0);
                }
                let label = format!("{} ({})", tool.label(), tool.key());
                let active = self.vec.tool == tool;
                if icon_button(ui, tool.icon(), &label, theme::TOOL_BUTTON, active, true).clicked()
                {
                    self.vec.tool = tool;
                }
            }
            ui.add_space(6.0);
            let (rect, _) = ui.allocate_exact_size(vec2(24.0, 1.0), Sense::hover());
            ui.painter().rect_filled(rect, 0.0, theme::LINE_SOFT);
            ui.add_space(6.0);
            let grid = self.vec.grid;
            if icon_button(
                ui,
                Icon::Grid,
                "Rejilla de coordenadas (G)",
                theme::TOOL_BUTTON,
                grid,
                true,
            )
            .clicked()
            {
                self.vec.grid = !grid;
            }
        });
    }

    fn vec_options(&mut self, ui: &mut egui::Ui, vm: &ViewModel, out: &mut Vec<Intent>) {
        let full = ui.max_rect();
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            ui.label(
                RichText::new(self.vec.tool.label())
                    .size(12.0)
                    .color(theme::TEXT),
            );
            vsep(ui);
            if self.vec.tool.draws() {
                self.style_controls(ui);
            } else {
                ui.label(
                    RichText::new(self.vec.tool.hint())
                        .size(12.0)
                        .color(theme::TEXT_3),
                );
            }
        });
        let right = Rect::from_min_max(pos2(full.right() - 620.0, full.top()), full.right_bottom());
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(right)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let small = |text: &str| {
                    egui::Button::new(RichText::new(text).size(14.0).color(theme::TEXT_2))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                        .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
                        .min_size(vec2(26.0, 26.0))
                };
                if ui
                    .add(small("+"))
                    .on_hover_text("Acercar (rueda del ratón)")
                    .clicked()
                {
                    self.vec.view_zoom = Some(self.vec.zoom * 1.25);
                }
                let fit = ui
                    .add(
                        egui::Button::new(
                            RichText::new(format!("{:.0}%", self.vec.zoom * 100.0))
                                .font(theme::mono(12.0))
                                .color(theme::TEXT),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                        .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
                        .min_size(vec2(56.0, 26.0)),
                    )
                    .on_hover_text(
                        "Ajustar al área (0). Rueda: zoom · botón central o espacio: mover",
                    );
                if fit.clicked() {
                    self.vec.view_zoom = None;
                    self.vec.pan = Vec2::ZERO;
                }
                if ui
                    .add(small("−"))
                    .on_hover_text("Alejar (rueda del ratón)")
                    .clicked()
                {
                    self.vec.view_zoom = Some(self.vec.zoom / 1.25);
                }
                ui.spacing_mut().item_spacing.x = 14.0;
                crate::canvas_view::background_selector(
                    ui,
                    &mut self.vec.background,
                    "fondo-vector",
                    true,
                );
                vsep(ui);
                let can_replay = self.vec.view.is_some() && self.vec.replay.is_none();
                let replay = ui.add_enabled(
                    can_replay,
                    egui::Button::new(
                        RichText::new("      Repetir trazado")
                            .size(12.0)
                            .color(theme::TEXT),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                    .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
                    .min_size(vec2(0.0, 26.0)),
                );
                let replay = replay.accessible_name("Repetir trazado");
                crate::icons::paint(
                    &ui.painter_at(replay.rect),
                    Icon::Play,
                    replay.rect.left_center() + vec2(16.0, 0.0),
                    theme::TEXT_2,
                );
                if replay
                    .on_hover_text("Vuelve a dibujarlo desde cero, forma a forma")
                    .clicked()
                {
                    out.push(Intent::ReplayVector);
                }
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

    fn vec_status(&mut self, ui: &mut egui::Ui) {
        let full = ui.max_rect();
        ui.painter().hline(
            (full.left() - 14.0)..=(full.right() + 14.0),
            full.top(),
            Stroke::new(1.0, theme::LINE),
        );
        let font = theme::mono(11.5);
        let text = |s: String| RichText::new(s).font(font.clone()).color(theme::TEXT_3);
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 18.0;
            // Sitio fijo para las coordenadas: lo de detrás no se mueve.
            let side = self
                .vec
                .view
                .as_ref()
                .map(|v| v.result.width.max(v.result.height).unsigned_abs())
                .unwrap_or(1000);
            let nines = "9".repeat(digits(side));
            let coords = match self.vec.cursor {
                Some([x, y]) => coordinates(x, y),
                None => "x – · y –".into(),
            };
            fixed_label(
                ui,
                text(coords),
                &font,
                &[&format!("x -{nines} · y -{nines}"), "x – · y –"],
            );
            if let Some(view) = &self.vec.view {
                let r = &view.result;
                ui.label(text(format!("{} × {}", r.width, r.height)));
                ui.label(text(format!(
                    "{} {}",
                    r.count,
                    if r.count == 1 { "forma" } else { "formas" }
                )));
                let n = view.layers.len();
                ui.label(text(format!(
                    "{n} {}",
                    if n == 1 { "capa" } else { "capas" }
                )));
                ui.label(text(format!("{:.0}%", self.vec.zoom * 100.0)));
            }
            if let Some(notice) = &self.notice {
                ui.label(RichText::new(notice).size(11.5).color(theme::OK));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(text(lume_domain::vec::FILE_NAME.into()));
            });
        });
    }

    /// El lienzo: el dibujo ajustado al área, el lazo, la zona y su
    /// recuadro, y el estado (calculando, errores, repetición).
    fn vec_canvas(&mut self, ui: &mut egui::Ui, vm: &ViewModel, out: &mut Vec<Intent>) {
        let full = ui.max_rect();
        let painter = ui.painter_at(full);
        // Fondo de puntos.
        let mut y = full.top() + 10.0;
        while y < full.bottom() {
            let mut x = full.left() + 10.0;
            while x < full.right() {
                painter.circle_filled(pos2(x, y), 0.8, theme::DOT);
                x += 20.0;
            }
            y += 20.0;
        }
        if vm.vector.is_none() && self.preview.is_none() {
            self.vec_empty(ui, full, out);
            return;
        }
        if let Some(preview) = self.preview.clone()
            && preview.vector.is_none()
        {
            ui.put(
                full,
                egui::Label::new(
                    RichText::new("Esa versión no tiene dibujo vectorial.").color(theme::TEXT_3),
                ),
            );
            self.version_banner(ui, full, &preview, out);
            return;
        }
        if self.vec.no_engine {
            ui.put(
                full,
                egui::Label::new(
                    RichText::new("El motor de dibujo.js no está disponible en esta ventana.")
                        .color(theme::TEXT_3),
                ),
            );
            return;
        }
        let Some(view) = self.vec.view.clone() else {
            ui.put(
                full,
                egui::Label::new(RichText::new("Dibujando…").color(theme::TEXT_3)),
            );
            return;
        };
        let (w, h) = (
            view.result.width.max(1) as f32,
            view.result.height.max(1) as f32,
        );
        let area = full.shrink(CANVAS_MARGIN);
        let fit = (area.width() / w).min(area.height() / h).max(0.01);
        // Rueda: zoom alrededor del ratón; botón central o espacio + arrastrar:
        // mover el dibujo.
        let response = ui.interact(
            full,
            ui.scope_id().with("lienzo-vector"),
            Sense::click_and_drag(),
        );
        let space = ui.input(|i| i.key_down(egui::Key::Space));
        let panning = response.dragged_by(egui::PointerButton::Middle)
            || (space && response.dragged_by(egui::PointerButton::Primary));
        if panning {
            self.vec.pan += response.drag_delta();
        }
        let current = self.vec.view_zoom.unwrap_or(fit);
        if let Some(pointer) = response.hover_pos() {
            let wheel = ui.input(|i| i.smooth_scroll_delta.y + i.zoom_delta().ln() * 200.0);
            if wheel.abs() > 0.5 && ui.ctx().current_pass_index() == 0 {
                let next =
                    (current * (wheel / 200.0).exp()).clamp(fit * ZOOM_RANGE.0, fit * ZOOM_RANGE.1);
                // El punto bajo el ratón no se mueve.
                let center = area.center() + self.vec.pan;
                self.vec.pan += (pointer - center) * (1.0 - next / current);
                self.vec.view_zoom = Some(next);
            }
        }
        let k = self
            .vec
            .view_zoom
            .map_or(fit, |z| z.clamp(fit * ZOOM_RANGE.0, fit * ZOOM_RANGE.1));
        if self.vec.view_zoom.is_none() {
            self.vec.pan = Vec2::ZERO;
        }
        self.vec.zoom = k;
        let image = Rect::from_center_size(area.center() + self.vec.pan, vec2(w * k, h * k));
        self.vec.image_rect = Some(image);
        let to_units = |p: Pos2| -> [f64; 2] {
            [
                f64::from(((p.x - image.left()) / k).clamp(0.0, w)),
                f64::from(((p.y - image.top()) / k).clamp(0.0, h)),
            ]
        };
        let to_screen = |p: &[f64; 2]| {
            pos2(
                image.left() + p[0] as f32 * k,
                image.top() + p[1] as f32 * k,
            )
        };

        // El dibujo con su sombra.
        let shadow = egui::epaint::Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color: Color32::from_black_alpha(110),
        };
        painter.add(shadow.as_shape(image, CornerRadius::ZERO));
        match self.vec.background {
            CanvasBackground::Checker => checkers(&painter, image),
            CanvasBackground::None | CanvasBackground::Drawing => {}
            other => {
                if let Some(color) = other.solid() {
                    painter.rect_filled(image, 0.0, color);
                }
            }
        }
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        // Con capas ocultas, los trozos sin ellas (al instante); si aún no
        // están pintados, el dibujo entero.
        // Con otro fondo que el del dibujo, su capa «Lienzo» no se ve.
        let mut hidden = self.vec.hidden.clone();
        if self.vec.background != CanvasBackground::Drawing
            && let Some(i) = view.layers.iter().position(|l| l.name == "Lienzo")
            && !hidden.contains(&i)
        {
            hidden.push(i);
        }
        let sliced = if hidden.is_empty() || self.vec.replay.is_some() {
            None
        } else {
            self.vec.slice_textures(ui.ctx()).map(|slices| {
                slices
                    .iter()
                    .filter(|(layer, _)| layer.is_none_or(|l| !hidden.contains(&l)))
                    .map(|(_, tex)| tex.id())
                    .collect::<Vec<_>>()
            })
        };
        match (sliced, self.vec.texture(ui.ctx())) {
            (Some(ids), _) => {
                for id in ids {
                    painter.image(id, image, uv, Color32::WHITE);
                }
            }
            (None, Some((tex, _))) => {
                painter.image(tex.id(), image, uv, Color32::WHITE);
            }
            (None, None) => {
                if let Err(problem) = &view.image {
                    painter.text(
                        image.center(),
                        Align2::CENTER_CENTER,
                        format!("No se pudo pintar el dibujo: {problem}"),
                        egui::FontId::proportional(12.5),
                        theme::DANGER,
                    );
                }
            }
        }

        if self.vec.grid {
            grid_overlay(&painter, full, image, w, h, k);
        }

        // El ratón: coordenadas, lazo y clic.
        let previewing = self.preview.is_some();
        self.vec.cursor = response
            .hover_pos()
            .filter(|p| image.contains(*p))
            .map(to_units);
        // El lazo tiene su propio cursor: un punto y el icono del lazo.
        if self.vec.tool == VecTool::Lasso
            && !panning
            && !space
            && let Some(p) = response.hover_pos()
        {
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            let top = ui.ctx().layer_painter(egui::LayerId::new(
                egui::Order::Tooltip,
                egui::Id::unique("cursor-lazo"),
            ));
            top.circle_filled(p, 2.5, Color32::BLACK);
            top.circle_filled(p, 1.5, Color32::WHITE);
            let icon = p + vec2(13.0, 13.0);
            top.circle_filled(icon, 10.0, Color32::from_black_alpha(170));
            crate::icons::paint(&top, Icon::Lasso, icon, Color32::WHITE);
        }
        if !previewing && self.vec.replay.is_none() && !panning && !space {
            match self.vec.tool {
                VecTool::Lasso => {
                    if response.drag_started_by(egui::PointerButton::Primary)
                        && let Some(p) = response.interact_pointer_pos()
                    {
                        self.vec.lasso = vec![to_units(p)];
                    }
                    if response.dragged()
                        && let Some(p) = response.interact_pointer_pos()
                    {
                        let u = to_units(p);
                        let far = self.vec.lasso.last().is_none_or(|l| {
                            ((l[0] - u[0]).powi(2) + (l[1] - u[1]).powi(2)).sqrt() * f64::from(k)
                                >= 2.0
                        });
                        if far {
                            self.vec.lasso.push(u);
                        }
                    }
                    if response.drag_stopped() {
                        let points = std::mem::take(&mut self.vec.lasso);
                        if points.len() >= 3 {
                            // Un píxel de pantalla de tolerancia.
                            let eps = 1.0 / f64::from(k);
                            self.vec.pending_zone =
                                Some(lume_domain::vec::studio::simplify(&points, eps));
                            out.push(Intent::VectorZone);
                        }
                    }
                }
                VecTool::Select => {
                    if response.clicked()
                        && let Some(p) = response.interact_pointer_pos()
                        && image.contains(p)
                    {
                        self.vec.click = Some(to_units(p));
                        out.push(Intent::VectorSelect);
                    }
                }
                VecTool::Text => {
                    if response.clicked()
                        && let Some(p) = response.interact_pointer_pos()
                    {
                        // Otro clic con un texto a medias lo deja puesto.
                        if let Some((at, text)) = self.vec.typing.take()
                            && let Some(code) =
                                crate::vec_draw::text_code(at, &text, &self.vec.style)
                        {
                            out.push(Intent::AddVectorCode(code));
                        } else {
                            self.vec.typing = Some((to_units(p), String::new()));
                            self.vec.focus_zone = true;
                        }
                    }
                }
                tool => {
                    if let Some(shape) = tool.shape() {
                        let shift = ui.input(|i| i.modifiers.shift);
                        // Desde donde se pulsó (el arrastre se nota un poco
                        // después).
                        if response.drag_started_by(egui::PointerButton::Primary)
                            && let Some(p) = ui
                                .input(|i| i.pointer.press_origin())
                                .or(response.interact_pointer_pos())
                        {
                            self.vec.draft = vec![to_units(p)];
                        }
                        if response.dragged()
                            && !self.vec.draft.is_empty()
                            && let Some(p) = response.interact_pointer_pos()
                        {
                            let u = to_units(p);
                            if shape == crate::vec_draw::Shape::Brush {
                                // Un punto cada pocos píxeles de pantalla.
                                let far = self.vec.draft.last().is_none_or(|l| {
                                    ((l[0] - u[0]).powi(2) + (l[1] - u[1]).powi(2)).sqrt()
                                        * f64::from(k)
                                        >= 6.0
                                });
                                if far {
                                    self.vec.draft.push(u);
                                }
                            } else {
                                let end =
                                    crate::vec_draw::end_point(shape, self.vec.draft[0], u, shift);
                                self.vec.draft.truncate(1);
                                self.vec.draft.push(end);
                            }
                        }
                        if response.drag_stopped() {
                            let mut points = std::mem::take(&mut self.vec.draft);
                            if shape == crate::vec_draw::Shape::Brush && points.len() > 2 {
                                let eps = 1.5 / f64::from(k);
                                points = lume_domain::vec::studio::simplify(&points, eps);
                            }
                            if let Some(code) =
                                crate::vec_draw::shape_code(shape, &points, &self.vec.style)
                            {
                                out.push(Intent::AddVectorCode(code));
                            }
                        }
                    }
                }
            }
        }
        // La forma que se arrastra, tal como quedará.
        if self.vec.draft.len() >= 2
            && let Some(shape) = self.vec.tool.shape()
        {
            let pts: Vec<Pos2> = self.vec.draft.iter().map(to_screen).collect();
            let style = self.vec.style;
            let width = (style.width * k).max(1.0);
            let fill = if style.fill_on {
                style.fill
            } else {
                Color32::TRANSPARENT
            };
            let border = if style.stroke_on || !style.fill_on {
                Stroke::new(width, style.stroke)
            } else {
                Stroke::NONE
            };
            use crate::vec_draw::Shape as S;
            match shape {
                S::Rect => {
                    let r = Rect::from_two_pos(pts[0], pts[1]);
                    painter.rect(r, 0.0, fill, border, StrokeKind::Middle);
                }
                S::Ellipse => {
                    let r = Rect::from_two_pos(pts[0], pts[1]);
                    painter.add(egui::epaint::EllipseShape {
                        center: r.center(),
                        radius: r.size() / 2.0,
                        fill,
                        stroke: border,
                        angle: 0.0,
                    });
                }
                S::Line => {
                    painter.line_segment([pts[0], pts[1]], Stroke::new(width, style.stroke));
                }
                S::Brush => {
                    painter.add(Shape::line(pts, Stroke::new(width, style.stroke)));
                }
            }
        }
        // El texto que se escribe, en su sitio.
        if let Some((at, _)) = self.vec.typing {
            let p = to_screen(&at);
            let mut done = None;
            let mut cancel = false;
            egui::Area::new(egui::Id::unique("lumeart-texto"))
                .order(egui::Order::Foreground)
                .fixed_pos(p + vec2(0.0, -34.0))
                .show(ui.ctx(), |ui| {
                    let Some((_, text)) = self.vec.typing.as_mut() else {
                        return;
                    };
                    let field = ui.add(
                        egui::TextEdit::singleline(text)
                            .hint_text("Escribe y pulsa Enter")
                            .desired_width(220.0)
                            .margin(vec2(8.0, 6.0)),
                    );
                    field.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Texto nuevo")
                    });
                    if std::mem::take(&mut self.vec.focus_zone) {
                        field.request_focus();
                    }
                    if field.lost_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            done = Some(text.clone());
                        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            cancel = true;
                        }
                    }
                });
            painter.circle_filled(p, 3.0, theme::ACCENT);
            if cancel {
                self.vec.typing = None;
            } else if let Some(text) = done {
                self.vec.typing = None;
                if let Some(code) = crate::vec_draw::text_code(at, &text, &self.vec.style) {
                    out.push(Intent::AddVectorCode(code));
                }
            }
        }

        // El lazo en curso, el que espera sus formas o la zona.
        let accent_fill = Color32::from_rgba_unmultiplied(0x3D, 0xBE, 0x6B, 26);
        let dashed = |pts: &[Pos2]| {
            let mut closed = pts.to_vec();
            closed.push(pts[0]);
            painter.extend(Shape::dashed_line(
                &closed,
                Stroke::new(1.8, theme::ACCENT),
                6.0,
                4.0,
            ));
        };
        if self.vec.lasso.len() >= 2 {
            let pts: Vec<Pos2> = self.vec.lasso.iter().map(to_screen).collect();
            painter.add(Shape::dashed_line(
                &pts,
                Stroke::new(1.8, theme::ACCENT),
                6.0,
                4.0,
            ));
        } else if let Some(points) = &self.vec.pending_zone {
            let pts: Vec<Pos2> = points.iter().map(to_screen).collect();
            if pts.len() >= 3 {
                fill_polygon(&painter, &pts, accent_fill);
                dashed(&pts);
            }
        }
        // Las correcciones de la cola: su contorno y su número.
        let badge = |at: Pos2, number: u32, color: Color32| {
            painter.circle_filled(at, 10.0, color);
            painter.text(
                at,
                Align2::CENTER_CENTER,
                number.to_string(),
                theme::semibold(11.0),
                theme::ACCENT_INK,
            );
        };
        for (i, c) in self.vec.corrections.iter().enumerate() {
            let pts: Vec<Pos2> = c.zone.points.iter().map(to_screen).collect();
            if pts.len() < 3 {
                continue;
            }
            let hovered = self.vec.hovered_correction == Some(i);
            if hovered {
                fill_polygon(
                    &painter,
                    &pts,
                    Color32::from_rgba_unmultiplied(0x3D, 0xBE, 0x6B, 56),
                );
            }
            let mut closed = pts.clone();
            closed.push(pts[0]);
            let stroke = match (c.sent, hovered) {
                (_, true) => Stroke::new(2.2, theme::ACCENT),
                (false, false) => Stroke::new(1.6, theme::ACCENT),
                (true, false) => Stroke::new(1.0, theme::ACCENT.gamma_multiply(0.45)),
            };
            painter.extend(Shape::dashed_line(&closed, stroke, 5.0, 5.0));
            let color = if c.sent {
                theme::ACCENT.gamma_multiply(0.6)
            } else {
                theme::ACCENT
            };
            badge(pts[0], c.number, color);
        }
        // La zona que se está escribiendo, con su recuadro.
        let mut zone_bounds = None;
        if let Some(zone) = &self.vec.zone {
            let pts: Vec<Pos2> = zone.zone.points.iter().map(to_screen).collect();
            if pts.len() >= 3 {
                fill_polygon(&painter, &pts, accent_fill);
                dashed(&pts);
                badge(pts[0], zone.number, theme::ACCENT);
                zone_bounds = Some(Rect::from_points(&pts));
            }
        }
        // La cola, arriba a la derecha (el recuadro de la zona no la tapa).
        let mut free = full;
        if let Some(queue) = self.corrections_panel(ui, full, out) {
            free.max.x = queue.left() - 8.0;
        }
        if let Some(bounds) = zone_bounds {
            self.zone_box(ui, free, bounds);
        }

        // Etiquetas: versión, repetición y errores.
        if let Some(preview) = self.preview.clone() {
            self.version_banner(ui, full, &preview, out);
        } else {
            let tag = match (&self.vec.replay, vm.versions.last()) {
                (Some(frame), _) => Some(format!(
                    "Repitiendo el trazado · {} de {} formas",
                    frame.shown, frame.total
                )),
                (None, Some(version)) => {
                    Some(format!("Versión {} · {}", version.number, version.message))
                }
                (None, None) => None,
            };
            if let Some(tag) = tag {
                let galley =
                    painter.layout_no_wrap(tag, egui::FontId::proportional(12.0), theme::TEXT);
                let rect = Rect::from_min_size(
                    full.min + vec2(16.0, 14.0),
                    vec2(galley.size().x + 24.0, 28.0),
                );
                painter.rect(
                    rect,
                    CornerRadius::same(theme::RADIUS_BUTTON),
                    theme::CARD,
                    Stroke::new(1.0, theme::LINE_STRONG),
                    StrokeKind::Inside,
                );
                painter.galley(
                    pos2(rect.left() + 12.0, rect.center().y - galley.size().y / 2.0),
                    galley,
                    theme::TEXT,
                );
            }
        }
        let errors = view.result.errors();
        if let Some(first) = errors.first() {
            let more = if errors.len() > 1 {
                format!(" (y {} más)", errors.len() - 1)
            } else {
                String::new()
            };
            let text = format!("dibujo.js · línea {}: {}{more}", first.line, first.msg);
            let rect = Rect::from_min_max(
                pos2(full.left() + 16.0, full.bottom() - 40.0),
                pos2(full.right() - 16.0, full.bottom() - 12.0),
            );
            painter.rect(
                rect,
                CornerRadius::same(theme::RADIUS_FIELD),
                theme::DANGER_BG,
                Stroke::new(1.0, theme::DANGER_LINE),
                StrokeKind::Inside,
            );
            ui.put(
                rect.shrink2(vec2(10.0, 0.0)),
                egui::Label::new(RichText::new(text).size(12.0).color(theme::DANGER)).truncate(),
            );
        }
    }

    /// Sin dibujo vectorial: crear uno en blanco.
    fn vec_empty(&mut self, ui: &mut egui::Ui, full: Rect, out: &mut Vec<Intent>) {
        let card = Rect::from_center_size(full.center(), vec2(460.0, 330.0));
        ui.scope_builder(UiBuilder::new().max_rect(card), |ui| {
            Frame::NONE
                .fill(theme::CARD)
                .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                .corner_radius(CornerRadius::same(theme::RADIUS_CARD))
                .inner_margin(Margin::same(20))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;
                    ui.label(
                        RichText::new("Nuevo dibujo vectorial")
                            .font(theme::semibold(16.0))
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new(
                            "Elige el tamaño del dibujo.js en blanco. Después, dibuja con las herramientas o pídele al agente lo que quieras.",
                        )
                        .size(12.5)
                        .color(theme::TEXT_3),
                    );
                    let current = self.vec.new_size.size().ok();
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                        for &(w, h) in &crate::size_picker::VECTOR_PRESETS {
                            let label = format!("{w} × {h}");
                            if crate::size_picker::chip(ui, &label, current == Some((w, h)))
                                .clicked()
                            {
                                self.vec.new_size.set(w, h);
                            }
                        }
                    });
                    self.vec.new_size.ui(ui);
                    ui.checkbox(
                        &mut self.vec.new_transparent,
                        RichText::new("Fondo transparente (un objeto suelto)").size(12.5),
                    );
                    let size = self.vec.new_size.size();
                    if let Err(message) = &size {
                        ui.label(RichText::new(message).size(12.0).color(theme::WARN));
                    }
                    let create = egui::Button::new(
                        RichText::new("Crear dibujo vectorial")
                            .font(theme::semibold(13.0))
                            .color(theme::ACCENT_INK),
                    )
                    .fill(theme::ACCENT)
                    .min_size(vec2(0.0, 32.0));
                    if ui.add_enabled(size.is_ok(), create).clicked()
                        && let Ok((width, height)) = size
                    {
                        out.push(Intent::NewVector {
                            width,
                            height,
                            transparent: self.vec.new_transparent,
                        });
                    }
                });
        });
    }

    /// El recuadro de la zona que se está escribiendo: qué cambiar y
    /// «Añadir» (a la cola; no va al agente hasta «Ejecutar correcciones»).
    fn zone_box(&mut self, ui: &mut egui::Ui, full: Rect, bounds: Rect) {
        let Some(zone) = self.vec.zone.as_mut() else {
            return;
        };
        let size = vec2(326.0, 124.0);
        // A la derecha de la zona si cabe; si no, a la izquierda.
        let x = if bounds.right() + 16.0 + size.x <= full.right() - 8.0 {
            bounds.right() + 16.0
        } else {
            (bounds.left() - 16.0 - size.x).max(full.left() + 8.0)
        };
        let y = (bounds.center().y - size.y / 2.0).clamp(
            full.top() + 8.0,
            (full.bottom() - size.y - 8.0).max(full.top() + 8.0),
        );
        let mut remove = false;
        let mut add = false;
        egui::Area::new(egui::Id::unique("lumeart-zona"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos2(x, y))
            .show(ui.ctx(), |ui| {
                Frame::NONE
                    .fill(theme::CARD)
                    .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                    .corner_radius(CornerRadius::same(theme::RADIUS_CARD_SMALL))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(size.x - 24.0);
                        ui.spacing_mut().item_spacing.y = 10.0;
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label(
                                RichText::new(format!("Corrección {}", zone.number))
                                    .font(theme::semibold(12.0))
                                    .color(theme::TEXT),
                            );
                            ui.label(
                                RichText::new(format!(" · {}", zone.zone.summary()))
                                    .size(12.0)
                                    .color(theme::TEXT_3),
                            );
                        });
                        let field = ui.add(
                            egui::TextEdit::singleline(&mut zone.text)
                                .hint_text("Qué cambiar en esta zona")
                                .desired_width(f32::INFINITY)
                                .margin(vec2(10.0, 8.0)),
                        );
                        field.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::Role::TextInput,
                                true,
                                "Qué cambiar en la zona",
                            )
                        });
                        if std::mem::take(&mut self.vec.focus_zone) {
                            field.request_focus();
                        }
                        let enter =
                            field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            let can = !zone.text.trim().is_empty();
                            let queue = ui
                                .add_enabled(
                                    can,
                                    egui::Button::new(
                                        RichText::new("Añadir a la cola")
                                            .font(theme::semibold(12.5))
                                            .color(theme::ACCENT_INK),
                                    )
                                    .fill(theme::ACCENT)
                                    .min_size(vec2(0.0, 30.0)),
                                )
                                .on_hover_text(
                                    "Se ejecuta con las demás al pulsar «Ejecutar correcciones»",
                                );
                            if can && (queue.clicked() || enter) {
                                add = true;
                            }
                            let quit = egui::Button::new(
                                RichText::new("Quitar").size(12.5).color(theme::TEXT_2),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE)
                            .min_size(vec2(0.0, 30.0));
                            if ui.add(quit).clicked() {
                                remove = true;
                            }
                        });
                    });
            });
        if add {
            self.vec.queue_zone();
        } else if remove {
            self.vec.zone = None;
        }
    }

    /// La cola de correcciones, arriba a la derecha del lienzo: cada una con
    /// su número y su texto, una × para quitarla mientras no se haya
    /// ejecutado y «Ejecutar correcciones». Sale si hay alguna o si el lazo
    /// está elegido. Devuelve dónde quedó.
    fn corrections_panel(
        &mut self,
        ui: &mut egui::Ui,
        full: Rect,
        out: &mut Vec<Intent>,
    ) -> Option<Rect> {
        if self.vec.corrections.is_empty() && self.vec.tool != VecTool::Lasso {
            self.vec.hovered_correction = None;
            return None;
        }
        let width = 264.0;
        let running = self.agent_view.running;
        let pending = self.vec.pending_corrections();
        let mut hovered = None;
        let mut remove = None;
        let response = egui::Area::new(egui::Id::unique("lumeart-correcciones"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos2(full.right() - width - 12.0, full.top() + 12.0))
            .show(ui.ctx(), |ui| {
                Frame::NONE
                    .fill(theme::CARD)
                    .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                    .corner_radius(CornerRadius::same(theme::RADIUS_CARD_SMALL))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(width - 24.0);
                        ui.spacing_mut().item_spacing.y = 8.0;
                        ui.horizontal(|ui| {
                            panel_label(ui, "Correcciones");
                            if !self.vec.corrections.is_empty() {
                                ui.label(
                                    RichText::new(self.vec.corrections.len().to_string())
                                        .size(11.5)
                                        .color(theme::TEXT_3),
                                );
                            }
                        });
                        if self.vec.corrections.is_empty() {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(
                                        "Rodea una zona con el lazo y escribe qué cambiar. Se suma aquí y no va al agente hasta que pulses «Ejecutar correcciones».",
                                    )
                                    .size(12.0)
                                    .color(theme::TEXT_3),
                                )
                                .wrap(),
                            );
                            return;
                        }
                        egui::ScrollArea::vertical()
                            .max_height((full.height() - 140.0).clamp(120.0, 420.0))
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                for (i, c) in self.vec.corrections.iter().enumerate() {
                                    let row = correction_row(ui, c);
                                    if row.hovered {
                                        hovered = Some(i);
                                    }
                                    if row.remove {
                                        remove = Some(i);
                                    }
                                }
                            });
                        let label = if running && pending < self.vec.corrections.len() {
                            "Ejecutando…".to_string()
                        } else {
                            "Ejecutar correcciones".to_string()
                        };
                        let run = ui.add_enabled(
                            pending > 0 && !running,
                            egui::Button::new(
                                RichText::new(label)
                                    .font(theme::semibold(12.5))
                                    .color(theme::ACCENT_INK),
                            )
                            .fill(theme::ACCENT)
                            .min_size(vec2(ui.available_width(), 32.0)),
                        );
                        let run = if running {
                            run.on_disabled_hover_text("Espera a que el agente termine")
                        } else {
                            run
                        };
                        if run.clicked() {
                            out.push(Intent::RunCorrections);
                        }
                    });
            });
        self.vec.hovered_correction = hovered;
        if let Some(i) = remove {
            self.vec.remove_correction(i);
        }
        Some(response.response.rect)
    }

    /// Panel inferior: el código a la izquierda y las capas a la derecha.
    fn vec_bottom(&mut self, ui: &mut egui::Ui) {
        let full = ui.max_rect();
        ui.painter()
            .hline(full.x_range(), full.top(), Stroke::new(1.0, theme::LINE));
        let layers = Rect::from_min_max(pos2(full.right() - LAYERS_WIDTH, full.top()), full.max);
        let code = Rect::from_min_max(full.min, pos2(layers.left(), full.bottom()));
        ui.painter()
            .vline(layers.left(), full.y_range(), Stroke::new(1.0, theme::LINE));
        ui.scope_builder(UiBuilder::new().max_rect(code), |ui| self.code_panel(ui));
        ui.scope_builder(UiBuilder::new().max_rect(layers), |ui| {
            self.layers_panel(ui)
        });
    }

    /// `dibujo.js` con números de línea: las líneas del agente con barra
    /// acento, las de la zona y la de la forma elegida resaltadas.
    fn code_panel(&mut self, ui: &mut egui::Ui) {
        let full = ui.max_rect();
        let head = Rect::from_min_size(full.min, vec2(full.width(), 36.0));
        let hovered = self
            .vec
            .hovered_correction
            .and_then(|i| self.vec.corrections.get(i));
        let zone_lines: Vec<u32> = self
            .vec
            .zone
            .as_ref()
            .or(hovered)
            .map(|z| z.zone.lines())
            .unwrap_or_default();
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(head.shrink2(vec2(16.0, 0.0)))
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                panel_label(ui, "Código");
                ui.label(
                    RichText::new(lume_domain::vec::FILE_NAME)
                        .font(theme::mono(12.0))
                        .color(theme::TEXT),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (text, color) = if let Some((a, b)) = self.vec.agent_lines {
                        let text = if a == b {
                            format!("el agente edita la línea {a}")
                        } else {
                            format!("el agente edita las líneas {a}–{b}")
                        };
                        (Some(text), theme::ACCENT)
                    } else if let Some(zone) = self.vec.zone.as_ref().or(hovered) {
                        (
                            Some(format!(
                                "corrección {} · {}",
                                zone.number,
                                lines_text(&zone_lines)
                            )),
                            theme::TEXT_2,
                        )
                    } else if let Some(hit) = &self.vec.selected {
                        let layer = if hit.layer.is_empty() {
                            String::new()
                        } else {
                            format!(" · capa «{}»", hit.layer)
                        };
                        (
                            Some(format!("línea {} · {}{layer}", hit.line, hit.tag)),
                            theme::TEXT_2,
                        )
                    } else {
                        (None, theme::TEXT_3)
                    };
                    if let Some(text) = text {
                        ui.label(RichText::new(text).size(12.0).color(color));
                    }
                });
            },
        );
        ui.painter()
            .hline(full.x_range(), head.bottom(), Stroke::new(1.0, theme::LINE));
        let Some(view) = self.vec.view.clone() else {
            return;
        };
        let lines: Vec<&str> = view.source.split('\n').collect();
        let body = Rect::from_min_max(pos2(full.left(), head.bottom() + 1.0), full.max);
        let agent = self.vec.agent_lines;
        let selected = self.vec.selected.as_ref().map(|h| h.line);
        let scroll_to = self.vec.scroll_to.take();
        ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
            // `show_rows` cuenta el espacio entre filas del `ui` de fuera.
            ui.spacing_mut().item_spacing.y = 0.0;
            let mut area = egui::ScrollArea::vertical()
                .id_salt("codigo-dibujo")
                .auto_shrink(false);
            if let Some(line) = scroll_to {
                let offset = (line.saturating_sub(1) as f32 - 2.0).max(0.0) * CODE_ROW;
                area = area.vertical_scroll_offset(offset);
            }
            area.show_rows(ui, CODE_ROW, lines.len(), |ui, range| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for i in range {
                    let n = i + 1;
                    let (rect, _) = ui
                        .allocate_exact_size(vec2(ui.available_width(), CODE_ROW), Sense::hover());
                    let painter = ui.painter_at(rect);
                    let in_agent = agent.is_some_and(|(a, b)| (a..=b).contains(&n));
                    let in_zone = zone_lines.contains(&(n as u32));
                    let is_selected = selected == Some(n as u32);
                    let (bg, bar) = if in_agent {
                        (
                            Some(Color32::from_rgba_unmultiplied(0x3D, 0xBE, 0x6B, 22)),
                            Some(theme::ACCENT),
                        )
                    } else if is_selected {
                        (Some(theme::HOVER), Some(theme::INFO))
                    } else if in_zone {
                        (
                            Some(Color32::from_rgba_unmultiplied(0x3D, 0xBE, 0x6B, 12)),
                            Some(theme::ACCENT_RING),
                        )
                    } else {
                        (None, None)
                    };
                    if let Some(bg) = bg {
                        painter.rect_filled(rect, 0.0, bg);
                    }
                    if let Some(bar) = bar {
                        painter.rect_filled(
                            Rect::from_min_size(rect.min, vec2(2.0, rect.height())),
                            0.0,
                            bar,
                        );
                    }
                    painter.text(
                        pos2(rect.left() + GUTTER - 16.0, rect.center().y),
                        Align2::RIGHT_CENTER,
                        n.to_string(),
                        theme::mono(12.0),
                        theme::TEXT_4,
                    );
                    // Con colores de sintaxis.
                    let base = if in_agent || in_zone || is_selected {
                        theme::TEXT
                    } else {
                        theme::TEXT_BODY
                    };
                    let job =
                        crate::syntax::highlight(&code_line(lines[i]), theme::mono(12.0), base);
                    let galley = painter.layout_job(job);
                    painter.galley(
                        pos2(
                            rect.left() + GUTTER,
                            rect.center().y - galley.size().y / 2.0,
                        ),
                        galley,
                        base,
                    );
                }
            });
        });
    }

    /// Las capas, de la de encima a la del fondo: ojo, nombre y formas.
    fn layers_panel(&mut self, ui: &mut egui::Ui) {
        let full = ui.max_rect();
        let head = Rect::from_min_size(full.min, vec2(full.width(), 36.0));
        let count = self.vec.view.as_ref().map_or(0, |v| v.layers.len());
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(head.shrink2(vec2(16.0, 0.0)))
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                panel_label(ui, "Capas");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(count.to_string())
                            .font(theme::mono(11.5))
                            .color(theme::TEXT_3),
                    );
                });
            },
        );
        ui.painter()
            .hline(full.x_range(), head.bottom(), Stroke::new(1.0, theme::LINE));
        let Some(view) = self.vec.view.clone() else {
            return;
        };
        let highlighted = self.vec.highlighted_layer().map(str::to_string);
        let body = Rect::from_min_max(pos2(full.left(), head.bottom() + 1.0), full.max);
        let mut toggle = None;
        ui.scope_builder(
            UiBuilder::new().max_rect(body.shrink2(vec2(8.0, 6.0))),
            |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("capas-dibujo")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for (i, layer) in view.layers.iter().enumerate().rev() {
                            let hidden = self.vec.hidden.contains(&i);
                            let (rect, row) = ui.allocate_exact_size(
                                vec2(ui.available_width(), LAYER_ROW),
                                Sense::hover(),
                            );
                            let painter = ui.painter_at(rect);
                            if highlighted.as_deref() == Some(layer.name.as_str()) {
                                painter.rect_filled(
                                    rect,
                                    CornerRadius::same(theme::RADIUS_FIELD),
                                    Color32::from_rgba_unmultiplied(0x3D, 0xBE, 0x6B, 22),
                                );
                            } else if row.hovered() {
                                painter.rect_filled(
                                    rect,
                                    CornerRadius::same(theme::RADIUS_FIELD),
                                    theme::HOVER,
                                );
                            }
                            let eye = Rect::from_center_size(
                                rect.left_center() + vec2(14.0, 0.0),
                                vec2(24.0, 24.0),
                            );
                            let label = if hidden {
                                format!("Mostrar la capa «{}»", layer.name)
                            } else {
                                format!("Ocultar la capa «{}»", layer.name)
                            };
                            let eye_response = ui
                                .interact(eye, ui.scope_id().with(("ojo", i)), Sense::click())
                                .on_hover_text(&label);
                            eye_response.widget_info(|| {
                                egui::WidgetInfo::labeled(egui::Role::Button, true, &label)
                            });
                            if eye_response.clicked() {
                                toggle = Some(i);
                            }
                            let eye_color = if hidden { theme::TEXT_4 } else { theme::TEXT_3 };
                            crate::icons::paint(&painter, Icon::Eye, eye.center(), eye_color);
                            if hidden {
                                painter.line_segment(
                                    [
                                        eye.left_bottom() + vec2(5.0, -5.0),
                                        eye.right_top() + vec2(-5.0, 5.0),
                                    ],
                                    Stroke::new(1.6, eye_color),
                                );
                            }
                            painter.text(
                                pos2(rect.left() + 34.0, rect.center().y),
                                Align2::LEFT_CENTER,
                                &layer.name,
                                egui::FontId::proportional(12.5),
                                if hidden { theme::TEXT_4 } else { theme::TEXT },
                            );
                            painter.text(
                                pos2(rect.right() - 8.0, rect.center().y),
                                Align2::RIGHT_CENTER,
                                layer.shapes.to_string(),
                                theme::mono(11.5),
                                theme::TEXT_3,
                            );
                        }
                    });
            },
        );
        if let Some(i) = toggle {
            match self.vec.hidden.iter().position(|&h| h == i) {
                Some(at) => {
                    self.vec.hidden.remove(at);
                }
                None => self.vec.hidden.push(i),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_are_listed_briefly() {
        assert_eq!(lines_text(&[]), "sin líneas propias");
        assert_eq!(lines_text(&[7]), "línea 7");
        assert_eq!(lines_text(&[3, 4]), "líneas 3, 4");
        assert_eq!(
            lines_text(&[1, 2, 3, 4, 5, 6, 7]),
            "líneas 1, 2, 3, 4, 5, 6, …"
        );
    }

    #[test]
    fn tools_have_their_keys() {
        assert_eq!(VecTool::Lasso.key(), 'L');
        assert_eq!(VecTool::Select.label(), "Seleccionar");
    }
}
