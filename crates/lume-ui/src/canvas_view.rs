//! El lienzo del studio: una textura del tamaño del documento escalada con
//! vecino más cercano, actualizada solo en las celdas del último patch, con
//! zoom, desplazamiento, rejilla, reglas y la celda bajo el cursor.

use eframe::egui::{
    self, Color32, ColorImage, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Vec2, pos2, vec2,
};
use lume_app::{DrawSpeed, ViewModel};
use lume_domain::{Canvas, Patch, PatchStep, Rgba};

use crate::brush::{Brush, Dirty};
use crate::theme;
use crate::tools::{Button, PointerEvent};

/// Zooms posibles: píxeles de pantalla por píxel del lienzo.
pub const ZOOM_STEPS: [f32; 16] = [
    1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 17.0, 20.0, 24.0, 32.0, 40.0, 48.0, 64.0,
];

/// Lo que se ve detrás del dibujo donde es transparente.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CanvasBackground {
    /// Cuadros de transparencia (el de pixel art).
    Checker,
    /// Nada: el fondo de la ventana.
    None,
    White,
    Black,
    /// Un color elegido.
    Color(Color32),
    /// El del propio dibujo (vector: la capa «Lienzo» de `dibujo.js`; el de
    /// vector de fábrica).
    #[default]
    Drawing,
}

impl CanvasBackground {
    pub fn label(self) -> &'static str {
        match self {
            CanvasBackground::Checker => "Cuadros",
            CanvasBackground::None => "Ninguno",
            CanvasBackground::White => "Blanco",
            CanvasBackground::Black => "Negro",
            CanvasBackground::Color(_) => "Color",
            CanvasBackground::Drawing => "Del dibujo",
        }
    }

    /// Color liso del fondo, si lo es.
    pub fn solid(self) -> Option<Color32> {
        match self {
            CanvasBackground::White => Some(Color32::WHITE),
            CanvasBackground::Black => Some(Color32::BLACK),
            CanvasBackground::Color(c) => Some(c),
            CanvasBackground::Checker | CanvasBackground::None | CanvasBackground::Drawing => None,
        }
    }
}

/// «Fondo: Cuadros ▾» en la barra de opciones; con «Color», además, una
/// muestra que abre el selector.
pub fn background_selector(
    ui: &mut egui::Ui,
    bg: &mut CanvasBackground,
    salt: &str,
    drawing: bool,
) {
    // En una barra de derecha a izquierda: primero (a la derecha) la muestra.
    if let CanvasBackground::Color(c) = bg {
        let mut color = *c;
        egui::color_picker::color_edit_button_srgba(
            ui,
            &mut color,
            egui::color_picker::Alpha::Opaque,
        )
        .on_hover_text("Color del fondo");
        *bg = CanvasBackground::Color(color);
    }
    let combo = egui::ComboBox::from_id_salt(salt)
        .width(118.0)
        .selected_text(
            egui::RichText::new(format!("Fondo: {}", bg.label()))
                .size(12.0)
                .color(theme::TEXT_2),
        )
        .show_ui(ui, |ui| {
            if drawing {
                ui.selectable_value(bg, CanvasBackground::Drawing, "Del dibujo");
            }
            for option in [
                CanvasBackground::Checker,
                CanvasBackground::None,
                CanvasBackground::White,
                CanvasBackground::Black,
            ] {
                ui.selectable_value(bg, option, option.label());
            }
            let custom = matches!(bg, CanvasBackground::Color(_));
            if ui.selectable_label(custom, "Color…").clicked() && !custom {
                *bg = CanvasBackground::Color(Color32::from_rgb(0x3A, 0x6E, 0x8F));
            }
        });
    let _ = combo
        .response
        .on_hover_text("Lo que se ve detrás del dibujo donde es transparente")
        .accessible_name("Fondo del lienzo");
}

/// Desde qué zoom se dibuja la rejilla.
pub const GRID_MIN_ZOOM: f32 = 6.0;

/// Transformación entre la pantalla y los píxeles del lienzo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Esquina superior izquierda del píxel (0, 0) en pantalla.
    pub origin: Pos2,
    /// Lado de un píxel del lienzo en pantalla (el zoom: 17 = 1700 %).
    pub zoom: f32,
}

impl Viewport {
    /// Píxel del lienzo bajo un punto de la pantalla (puede estar fuera).
    pub fn pixel_at(&self, pos: Pos2) -> (i64, i64) {
        let local = (pos - self.origin) / self.zoom;
        (local.x.floor() as i64, local.y.floor() as i64)
    }

    /// Píxel bajo un punto, si está dentro de un lienzo de `w` × `h`.
    pub fn pixel_in(&self, pos: Pos2, w: u32, h: u32) -> Option<(u32, u32)> {
        let (x, y) = self.pixel_at(pos);
        (x >= 0 && y >= 0 && x < i64::from(w) && y < i64::from(h)).then_some((x as u32, y as u32))
    }

    /// Rectángulo en pantalla del píxel (`x`, `y`).
    pub fn pixel_rect(&self, x: i64, y: i64) -> Rect {
        Rect::from_min_size(
            self.origin + vec2(x as f32, y as f32) * self.zoom,
            Vec2::splat(self.zoom),
        )
    }

    /// Rectángulo en pantalla de un lienzo de `w` × `h`.
    pub fn canvas_rect(&self, w: u32, h: u32) -> Rect {
        Rect::from_min_size(self.origin, vec2(w as f32, h as f32) * self.zoom)
    }

    /// Zoom en porcentaje, como lo muestra la interfaz («1700%»).
    pub fn percent(&self) -> u32 {
        (self.zoom * 100.0).round() as u32
    }
}

/// El zoom `steps` pasos por encima (o por debajo, si es negativo) en
/// [`ZOOM_STEPS`]. Si `zoom` no es uno de los pasos, el primer paso cuenta
/// desde el más cercano en esa dirección.
pub fn next_zoom(zoom: f32, steps: i32) -> f32 {
    let n = ZOOM_STEPS.len() as i32;
    let index = if steps > 0 {
        let first_above = ZOOM_STEPS
            .iter()
            .position(|z| *z > zoom + 0.01)
            .map_or(n, |i| i as i32);
        first_above + steps - 1
    } else if steps < 0 {
        let last_below = ZOOM_STEPS
            .iter()
            .rposition(|z| *z < zoom - 0.01)
            .map_or(-1, |i| i as i32);
        last_below + steps + 1
    } else {
        return zoom;
    };
    ZOOM_STEPS[index.clamp(0, n - 1) as usize]
}

/// El mayor zoom de [`ZOOM_STEPS`] con el que el lienzo cabe en `area` con
/// un margen.
pub fn fit_zoom(area: Vec2, w: u32, h: u32) -> f32 {
    let margin = 80.0;
    let fit = ((area.x - margin) / w as f32).min((area.y - margin) / h as f32);
    ZOOM_STEPS
        .iter()
        .rev()
        .copied()
        .find(|z| *z <= fit)
        .unwrap_or(ZOOM_STEPS[0])
}

/// Qué hay que subir a la textura con un `ViewModel` nuevo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureUpdate {
    /// Nada (misma revisión).
    None,
    /// Todo el lienzo (primera vez, cambio de tamaño, revisiones perdidas).
    Full,
    /// Solo este rectángulo de píxeles: `x`, `y`, ancho, alto.
    Region { x: u32, y: u32, w: u32, h: u32 },
}

/// Rectángulo que cubre las celdas de un patch (`None` si no hay celdas o si
/// cambia el tamaño).
pub fn patch_bounds(patch: &Patch) -> Option<(u32, u32, u32, u32)> {
    if patch
        .steps()
        .iter()
        .any(|s| matches!(s, PatchStep::Resize(_)))
    {
        return None;
    }
    let mut cells = patch.cells();
    let first = cells.next()?;
    let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.x, first.y);
    for c in cells {
        x0 = x0.min(c.x);
        y0 = y0.min(c.y);
        x1 = x1.max(c.x);
        y1 = y1.max(c.y);
    }
    Some((x0, y0, x1 - x0 + 1, y1 - y0 + 1))
}

/// Decide qué subir: si llega la revisión siguiente con un patch sin cambio
/// de tamaño, solo su rectángulo; si no, todo.
pub fn plan_update(uploaded: Option<(u64, u32, u32)>, vm: &ViewModel) -> TextureUpdate {
    let size = (vm.canvas.width(), vm.canvas.height());
    let Some((revision, w, h)) = uploaded else {
        return TextureUpdate::Full;
    };
    if revision == vm.revision && (w, h) == size {
        return TextureUpdate::None;
    }
    if (w, h) != size || vm.revision != revision + 1 {
        return TextureUpdate::Full;
    }
    match vm
        .last_change
        .as_ref()
        .filter(|c| c.revision == vm.revision)
    {
        Some(change) => match patch_bounds(&change.patch) {
            Some((x, y, w, h)) => TextureUpdate::Region { x, y, w, h },
            // Sin celdas (comando sin cambios visibles) no hay nada que subir;
            // con cambio de tamaño, todo.
            None if change.patch.cell_count() == 0 && change.patch.steps().is_empty() => {
                TextureUpdate::None
            }
            None => TextureUpdate::Full,
        },
        None => TextureUpdate::Full,
    }
}

/// Sube `canvas` (entero o una región) a la textura del lienzo.
fn upload(
    ctx: &egui::Context,
    texture: &mut Option<TextureHandle>,
    canvas: &Canvas,
    update: TextureUpdate,
) {
    match update {
        TextureUpdate::None => {}
        TextureUpdate::Full => {
            let image = region_image(canvas, 0, 0, canvas.width(), canvas.height());
            match texture {
                Some(texture) => texture.set(image, TextureOptions::NEAREST),
                None => {
                    *texture = Some(ctx.load_texture("lienzo", image, TextureOptions::NEAREST));
                }
            }
        }
        TextureUpdate::Region { x, y, w, h } => {
            let image = region_image(canvas, x, y, w, h);
            if let Some(texture) = texture {
                texture.set_partial([x as usize, y as usize], image, TextureOptions::NEAREST);
            }
        }
    }
}

/// Imagen RGBA de una región del lienzo.
pub fn region_image(canvas: &Canvas, x: u32, y: u32, w: u32, h: u32) -> ColorImage {
    let mut bytes = Vec::with_capacity(w as usize * h as usize * 4);
    for py in y..y + h {
        for px in x..x + w {
            let c = canvas.pixels()[(py * canvas.width() + px) as usize];
            bytes.extend_from_slice(&[c.r, c.g, c.b, c.a]);
        }
    }
    ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &bytes)
}

/// Estado del lienzo en pantalla.
#[derive(Clone)]
pub struct CanvasView {
    pub zoom: f32,
    /// Desplazamiento del centro del lienzo respecto al centro del área.
    pub pan: Vec2,
    /// ¿Ajustar el zoom al área en el próximo fotograma?
    pub fit_pending: bool,
    /// Píxel bajo el cursor.
    pub hover: Option<(u32, u32)>,
    /// Transformación del último fotograma.
    pub viewport: Option<Viewport>,
    texture: Option<TextureHandle>,
    /// Revisión y tamaño de lo que hay en la textura.
    uploaded: Option<(u64, u32, u32)>,
    /// Última actualización hecha (para tests y diagnóstico).
    pub last_update: TextureUpdate,
    /// Píxel del último evento de un trazo en curso.
    drawing: Option<(i64, i64)>,
    /// Ver dibujar al agente píxel a píxel.
    pub brush: Brush,
    /// A qué velocidad.
    pub speed: DrawSpeed,
    /// La textura tiene lo que va pintando el pincel.
    brush_uploaded: bool,
    /// Textura de 2×2 para los cuadros de transparencia (se repite).
    checker: Option<TextureHandle>,
    /// Detrás del dibujo.
    pub background: CanvasBackground,
}

impl std::fmt::Debug for CanvasView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanvasView")
            .field("zoom", &self.zoom)
            .field("pan", &self.pan)
            .field("hover", &self.hover)
            .field("uploaded", &self.uploaded)
            .field("last_update", &self.last_update)
            .finish_non_exhaustive()
    }
}

impl Default for CanvasView {
    fn default() -> Self {
        Self {
            zoom: 17.0,
            pan: Vec2::ZERO,
            fit_pending: true,
            hover: None,
            viewport: None,
            texture: None,
            uploaded: None,
            last_update: TextureUpdate::None,
            drawing: None,
            brush: Brush::default(),
            speed: DrawSpeed::Normal,
            brush_uploaded: false,
            checker: None,
            background: CanvasBackground::Checker,
        }
    }
}

impl CanvasView {
    /// Cambia el zoom manteniendo fijo el punto de la pantalla `anchor`.
    pub fn zoom_to(&mut self, zoom: f32, anchor: Pos2, area: Rect) {
        let old = self.zoom;
        if (zoom - old).abs() < f32::EPSILON {
            return;
        }
        // El centro del lienzo está en area.center() + pan; el punto bajo
        // `anchor` debe seguir en `anchor`.
        let center = area.center() + self.pan;
        let offset = anchor - center;
        let new_center = anchor - offset * (zoom / old);
        self.pan = new_center - area.center();
        self.zoom = zoom;
    }

    /// Transformación para un lienzo de `w` × `h` en `area`.
    pub fn viewport_for(&self, area: Rect, w: u32, h: u32) -> Viewport {
        let size = vec2(w as f32, h as f32) * self.zoom;
        let origin = (area.center() + self.pan - size / 2.0).round();
        Viewport {
            origin,
            zoom: self.zoom,
        }
    }

    /// Sube a la textura lo necesario para mostrar `vm` (o, mientras el
    /// agente dibuja, lo que lleva pintado el pincel).
    pub fn sync_texture(&mut self, ctx: &egui::Context, vm: &ViewModel) {
        self.brush.observe(vm, self.speed);
        let tick = self.brush.tick(ctx.input(|i| i.stable_dt), self.speed);
        if let Some(shown) = self.brush.display() {
            ctx.request_repaint();
            let update = match (self.brush_uploaded, tick.dirty, &self.texture) {
                (false, _, _) | (_, Some(Dirty::Full), _) | (_, _, None) => TextureUpdate::Full,
                (true, Some(Dirty::Region { x, y, w, h }), Some(_)) => {
                    TextureUpdate::Region { x, y, w, h }
                }
                (true, None, Some(_)) => TextureUpdate::None,
            };
            upload(ctx, &mut self.texture, shown, update);
            self.brush_uploaded = true;
            // Al terminar se vuelve a subir el lienzo de verdad entero.
            self.uploaded = None;
            return;
        }
        self.brush_uploaded = false;
        let canvas = &vm.canvas;
        let update = match &self.texture {
            None => TextureUpdate::Full,
            Some(_) => plan_update(self.uploaded, vm),
        };
        upload(ctx, &mut self.texture, canvas, update);
        if update != TextureUpdate::None {
            self.last_update = update;
        }
        self.uploaded = Some((vm.revision, canvas.width(), canvas.height()));
    }

    /// Textura del lienzo (para las vistas previas), si ya se subió.
    pub fn texture_id(&self) -> Option<egui::TextureId> {
        self.texture.as_ref().map(|t| t.id())
    }

    /// ¿Hay un trazo en curso (botón pulsado sobre el lienzo)?
    pub fn is_drawing(&self) -> bool {
        self.drawing.is_some()
    }

    /// Termina el trazo en curso sin más eventos (Esc).
    pub fn cancel_drawing(&mut self) {
        self.drawing = None;
    }

    /// Dibuja el lienzo, atiende zoom, desplazamiento y cursor, y devuelve los
    /// eventos de puntero para la herramienta (si `draw` está activo).
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        vm: &ViewModel,
        grid: bool,
        hand: bool,
        draw: bool,
    ) -> Vec<PointerEvent> {
        let mut events = Vec::new();
        let area = ui.max_rect();
        let (w, h) = (vm.canvas.width(), vm.canvas.height());
        if self.fit_pending && area.width() > 1.0 {
            self.zoom = fit_zoom(area.size(), w, h);
            self.pan = Vec2::ZERO;
            self.fit_pending = false;
        }
        self.sync_texture(ui.ctx(), vm);
        let response = ui.allocate_rect(area, Sense::click_and_drag());

        // egui puede hacer dos pasadas por fotograma con la misma entrada: el
        // zoom y el desplazamiento solo se aplican en la primera.
        let first_pass = ui.ctx().current_pass_index() == 0;
        // Desplazamiento: mano, espacio o botón central.
        let space = ui.input(|i| i.key_down(egui::Key::Space));
        let panning = response.dragged_by(egui::PointerButton::Middle)
            || (response.dragged_by(egui::PointerButton::Primary) && (hand || space));
        if panning && first_pass {
            self.pan += response.drag_delta();
        }
        // Zoom: rueda (centrado en el cursor), + / − y 0 para ajustar.
        if let Some(pointer) = response.hover_pos().filter(|_| first_pass) {
            // Como mucho un paso de zoom por fotograma, según el sentido de
            // la rueda (en X11 cada clic de rueda llega como dos eventos).
            let wheel: f32 = ui.input(|i| {
                i.events
                    .iter()
                    .map(|e| match e {
                        egui::Event::MouseWheel { unit, delta, .. } => match unit {
                            egui::MouseWheelUnit::Point => delta.y / 50.0,
                            _ => delta.y,
                        },
                        _ => 0.0,
                    })
                    .sum()
            });
            if wheel.abs() > 0.1 {
                let zoom = next_zoom(self.zoom, if wheel > 0.0 { 1 } else { -1 });
                self.zoom_to(zoom, pointer, area);
            }
        }
        if first_pass && (response.hovered() || ui.memory(|m| m.focused().is_none())) {
            let (plus, minus, zero) = ui.input(|i| {
                (
                    i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals),
                    i.key_pressed(egui::Key::Minus),
                    i.key_pressed(egui::Key::Num0),
                )
            });
            if plus {
                self.zoom_to(next_zoom(self.zoom, 1), area.center(), area);
            }
            if minus {
                self.zoom_to(next_zoom(self.zoom, -1), area.center(), area);
            }
            if zero {
                self.fit_pending = true;
            }
        }

        let viewport = self.viewport_for(area, w, h);
        self.viewport = Some(viewport);

        // Eventos para la herramienta: los eventos crudos en orden (pulsar,
        // mover y soltar pueden llegar en el mismo fotograma si el ratón va
        // rápido), para no perder ningún punto ni confundir el inicio.
        if first_pass && draw && !hand && !space {
            let raw = ui.input(|i| i.events.clone());
            let layer = ui.layer_id();
            for event in raw {
                match event {
                    egui::Event::PointerMoved(pos) => {
                        if let Some(last) = self.drawing {
                            let p = viewport.pixel_at(pos);
                            if p != last {
                                events.push(PointerEvent::Move { x: p.0, y: p.1 });
                                self.drawing = Some(p);
                            }
                        }
                    }
                    egui::Event::PointerButton {
                        pos,
                        button,
                        pressed: true,
                        ..
                    } if self.drawing.is_none()
                        && area.contains(pos)
                        && ui.ctx().layer_id_at(pos) == Some(layer) =>
                    {
                        let button = match button {
                            egui::PointerButton::Primary => Button::Primary,
                            egui::PointerButton::Secondary => Button::Secondary,
                            _ => continue,
                        };
                        let (x, y) = viewport.pixel_at(pos);
                        events.push(PointerEvent::Down { x, y, button });
                        self.drawing = Some((x, y));
                    }
                    egui::Event::PointerButton {
                        pos,
                        pressed: false,
                        ..
                    } => {
                        if let Some(last) = self.drawing.take() {
                            let p = viewport.pixel_at(pos);
                            if p != last {
                                events.push(PointerEvent::Move { x: p.0, y: p.1 });
                            }
                            events.push(PointerEvent::Up { x: p.0, y: p.1 });
                        }
                    }
                    egui::Event::PointerGone => {
                        // El puntero salió de la ventana: el trazo sigue hasta
                        // que se suelte el botón.
                    }
                    _ => {}
                }
            }
        } else if !draw {
            self.drawing = None;
        }
        self.hover = response
            .hover_pos()
            .and_then(|p| viewport.pixel_in(p, w, h));

        let painter = ui.painter_at(area);
        paint_dots(&painter, area);
        let canvas_rect = viewport.canvas_rect(w, h);
        // Marco con sombra.
        painter.rect_filled(
            canvas_rect.translate(vec2(0.0, 24.0)).expand(8.0),
            CornerRadius::same(16),
            Color32::from_black_alpha(60),
        );
        painter.rect_filled(
            canvas_rect.expand(1.0),
            0.0,
            Color32::from_rgb(0x07, 0x08, 0x0A),
        );
        painter.rect_stroke(
            canvas_rect.expand(1.0),
            0.0,
            Stroke::new(1.0, theme::LINE_STRONG),
            StrokeKind::Outside,
        );
        let checker = self
            .checker
            .get_or_insert_with(|| checker_texture(ui.ctx()))
            .id();
        match self.background {
            CanvasBackground::Checker => paint_checker(&painter, &viewport, w, h, checker),
            CanvasBackground::None => {}
            other => {
                if let Some(color) = other.solid() {
                    painter.rect_filled(canvas_rect, 0.0, color);
                }
            }
        }
        if let Some(texture) = &self.texture {
            painter.image(
                texture.id(),
                canvas_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        if grid && self.zoom >= GRID_MIN_ZOOM {
            paint_grid(&painter, &viewport, w, h);
        }
        paint_rulers(&painter, &viewport, w, h, area);
        // El pincel: la última celda que pintó el agente (como en pxcli).
        if let Some((x, y)) = self.brush.position().filter(|_| self.zoom >= 4.0) {
            let cell = viewport.pixel_rect(i64::from(x), i64::from(y));
            painter.rect_stroke(
                cell,
                0.0,
                Stroke::new(1.0, Color32::from_black_alpha(180)),
                StrokeKind::Outside,
            );
            painter.rect_stroke(
                cell.shrink(1.0),
                0.0,
                Stroke::new(2.0, Color32::WHITE),
                StrokeKind::Inside,
            );
        }
        if let Some((x, y)) = self.hover {
            let cell = viewport.pixel_rect(i64::from(x), i64::from(y));
            painter.rect_stroke(
                cell,
                0.0,
                Stroke::new(2.0, Color32::WHITE),
                StrokeKind::Inside,
            );
        }
        events
    }

    /// Resalta el último cambio de la CLI o del agente: borde interior acento
    /// en sus celdas y una etiqueta con el comando («agente · set_pixel 21
    /// 24»). Los cambios hechos en la ventana no se resaltan.
    pub fn paint_last_change(&self, ui: &mut egui::Ui, area: Rect, vm: &ViewModel) {
        let (Some(vp), Some(change)) = (self.viewport, vm.last_change.as_ref()) else {
            return;
        };
        if change.revision != vm.revision || change.origin == lume_domain::Origin::User {
            return;
        }
        let painter = ui.painter_at(area);
        let stroke = Stroke::new(1.0, theme::ACCENT);
        let cells: Vec<_> = change.patch.cells().map(|c| (c.x, c.y)).collect();
        let Some(&(lx, ly)) = cells.last() else {
            return;
        };
        if cells.len() <= 4096 {
            for &(x, y) in &cells {
                painter.rect_stroke(
                    vp.pixel_rect(i64::from(x), i64::from(y)),
                    0.0,
                    stroke,
                    StrokeKind::Inside,
                );
            }
        } else if let Some((x, y, w, h)) = patch_bounds(&change.patch) {
            let rect = Rect::from_min_size(
                vp.origin + vec2(x as f32, y as f32) * vp.zoom,
                vec2(w as f32, h as f32) * vp.zoom,
            );
            painter.rect_stroke(rect, 0.0, stroke, StrokeKind::Inside);
        }
        let text = format!(
            "{} · {}",
            author_label(change.origin),
            short_command(&change.label)
        );
        let galley =
            painter.layout_no_wrap(text.clone(), FontId::monospace(11.0), theme::ACCENT_INK);
        let cell = vp.pixel_rect(i64::from(lx), i64::from(ly));
        let pos = pos2(cell.right() + 4.0, cell.top() - 2.0);
        let rect = Rect::from_min_size(pos, galley.size() + vec2(16.0, 8.0));
        painter.rect_filled(
            rect.translate(vec2(0.0, 6.0)),
            CornerRadius::same(6),
            Color32::from_black_alpha(90),
        );
        painter.rect_filled(rect, CornerRadius::same(6), theme::ACCENT);
        // Una etiqueta de verdad (también para lectores de pantalla y tests).
        ui.put(
            rect,
            egui::Label::new(
                egui::RichText::new(text)
                    .font(FontId::monospace(11.0))
                    .color(theme::ACCENT_INK),
            )
            .selectable(false),
        );
    }

    /// Contorno discontinuo de una selección (blanco sobre negro).
    pub fn paint_selection(&self, ui: &egui::Ui, area: Rect, sel: &crate::selection::Selection) {
        let Some(vp) = self.viewport else {
            return;
        };
        let rect = Rect::from_min_size(
            vp.origin + vec2(sel.x as f32, sel.y as f32) * vp.zoom,
            vec2(sel.w as f32, sel.h as f32) * vp.zoom,
        );
        let painter = ui.painter_at(area);
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, Color32::BLACK),
            StrokeKind::Middle,
        );
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
            rect.left_top(),
        ];
        for pair in corners.windows(2) {
            painter.add(egui::Shape::dashed_line(
                &[pair[0], pair[1]],
                Stroke::new(1.0, Color32::WHITE),
                4.0,
                4.0,
            ));
        }
    }

    /// Píxeles de varios colores encima del lienzo (lo que sale del lienzo no
    /// se pinta).
    pub fn paint_pixels(
        &self,
        ui: &egui::Ui,
        area: Rect,
        pixels: &[(i64, i64, Rgba)],
        w: u32,
        h: u32,
    ) {
        let Some(vp) = self.viewport else {
            return;
        };
        let painter = ui.painter_at(area.intersect(vp.canvas_rect(w, h)));
        for &(x, y, c) in pixels {
            painter.rect_filled(
                vp.pixel_rect(x, y),
                0.0,
                Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a),
            );
        }
    }

    /// Tapa una zona con el fondo de cuadros (el origen de lo que se mueve).
    pub fn paint_cleared(&self, ui: &egui::Ui, area: Rect, sel: &crate::selection::Selection) {
        let Some(vp) = self.viewport else {
            return;
        };
        let painter = ui.painter_at(area);
        for y in sel.y..sel.y + sel.h {
            for x in sel.x..sel.x + sel.w {
                let color = if ((x / 4) + (y / 4)).is_multiple_of(2) {
                    theme::CHECKER_A
                } else {
                    theme::CHECKER_B
                };
                painter.rect_filled(vp.pixel_rect(i64::from(x), i64::from(y)), 0.0, color);
            }
        }
    }

    /// Pinta la vista previa de un trazo (celdas y color) encima del lienzo.
    pub fn paint_preview(&self, ui: &egui::Ui, area: Rect, cells: &[(u32, u32)], color: Rgba) {
        let Some(vp) = self.viewport else {
            return;
        };
        let painter = ui.painter_at(area);
        let fill = if color.a == 0 {
            // Borrador: se ve el fondo de cuadros.
            theme::CHECKER_B
        } else {
            Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
        };
        for &(x, y) in cells {
            painter.rect_filled(vp.pixel_rect(i64::from(x), i64::from(y)), 0.0, fill);
        }
    }

    /// Color del píxel bajo el cursor.
    pub fn hover_color(&self, vm: &ViewModel) -> Option<Rgba> {
        let (x, y) = self.hover?;
        vm.canvas.get(i64::from(x), i64::from(y)).ok()
    }
}

/// «agente», «cli» o «tú».
fn author_label(origin: lume_domain::Origin) -> &'static str {
    match origin {
        lume_domain::Origin::User => "tú",
        lume_domain::Origin::Agent { .. } => "agente",
        lume_domain::Origin::Cli => "cli",
    }
}

/// Un comando del protocolo sin el color del final, como en la etiqueta del
/// diseño: `set_pixel 21 24 #5b2aa6` → `set_pixel 21 24`.
pub fn short_command(label: &str) -> String {
    let fields: Vec<&str> = label.split_whitespace().collect();
    let drawing = matches!(
        fields.first(),
        Some(&("set_pixel" | "fill_rect" | "line" | "clear"))
    );
    match fields.split_last() {
        Some((last, rest)) if drawing && !rest.is_empty() && Rgba::parse(last).is_ok() => {
            rest.join(" ")
        }
        _ => fields.join(" "),
    }
}

/// Fondo de puntos: 1 px `#1A1E25` cada 16 px.
fn paint_dots(painter: &egui::Painter, area: Rect) {
    let mut y = area.top() + 8.0;
    while y < area.bottom() {
        let mut x = area.left() + 8.0;
        while x < area.right() {
            painter.rect_filled(
                Rect::from_min_size(pos2(x, y), vec2(1.0, 1.0)),
                0.0,
                theme::DOT,
            );
            x += 16.0;
        }
        y += 16.0;
    }
}

/// Lado en pantalla de un cuadro de transparencia: medio píxel del lienzo
/// (cuatro cuadros por píxel, como el studio web de pxcli), al menos 4.
pub fn checker_cell(zoom: f32) -> f32 {
    (zoom / 2.0).round().max(4.0)
}

/// Textura de 2×2 cuadros que se repite.
fn checker_texture(ctx: &egui::Context) -> TextureHandle {
    let (a, b) = (theme::CHECKER_A, theme::CHECKER_B);
    let image = ColorImage::new([2, 2], vec![a, b, b, a]);
    ctx.load_texture(
        "cuadros",
        image,
        TextureOptions {
            wrap_mode: egui::TextureWrapMode::Repeat,
            ..TextureOptions::NEAREST
        },
    )
}

/// Cuadros de transparencia bajo el dibujo, con origen en la esquina del
/// lienzo.
fn paint_checker(painter: &egui::Painter, vp: &Viewport, w: u32, h: u32, texture: egui::TextureId) {
    let rect = vp.canvas_rect(w, h);
    let cell = checker_cell(vp.zoom);
    let uv = Rect::from_min_size(pos2(0.0, 0.0), rect.size() / (2.0 * cell));
    painter.image(texture, rect, uv, Color32::WHITE);
}

/// Línea de la rejilla entre píxeles: blanca al 7 % (premultiplicada).
pub const GRID_LINE: Color32 = Color32::from_rgba_premultiplied(18, 18, 18, 18);
/// Línea cada 8 píxeles: blanca al 14 %.
pub const GRID_LINE_8: Color32 = Color32::from_rgba_premultiplied(36, 36, 36, 36);

/// Rejilla sobre el dibujo, como en el studio web de pxcli: una línea blanca
/// casi transparente entre píxeles y otra algo más marcada cada 8 (si el
/// lienzo es múltiplo de 8). No tapa los colores: solo los separa.
fn paint_grid(painter: &egui::Painter, vp: &Viewport, w: u32, h: u32) {
    let rect = vp.canvas_rect(w, h);
    let every8 = w > 8 && w.is_multiple_of(8) && h.is_multiple_of(8);
    let stroke = |i: u32| {
        let color = if every8 && i.is_multiple_of(8) {
            GRID_LINE_8
        } else {
            GRID_LINE
        };
        Stroke::new(1.0, color)
    };
    for x in 1..w {
        let sx = (vp.origin.x + x as f32 * vp.zoom).round() + 0.5;
        painter.vline(sx, rect.y_range(), stroke(x));
    }
    for y in 1..h {
        let sy = (vp.origin.y + y as f32 * vp.zoom).round() + 0.5;
        painter.hline(rect.x_range(), sy, stroke(y));
    }
}

/// Reglas cada 4 píxeles, arriba y a la izquierda (mono 10.5, `text_4`).
fn paint_rulers(painter: &egui::Painter, vp: &Viewport, w: u32, h: u32, area: Rect) {
    let rect = vp.canvas_rect(w, h);
    let font = FontId::monospace(10.5);
    let every = if vp.zoom >= 8.0 {
        4
    } else if vp.zoom >= 3.0 {
        8
    } else {
        32
    };
    let top = rect.top() - 22.0;
    let left = rect.left() - 26.0;
    for x in (0..w).step_by(every) {
        let sx = vp.origin.x + x as f32 * vp.zoom;
        if sx < area.left() || sx > area.right() {
            continue;
        }
        painter.vline(
            sx,
            (top + 4.0)..=(top + 18.0),
            Stroke::new(1.0, theme::LINE_STRONG),
        );
        painter.text(
            pos2(sx + 2.0, top + 4.0),
            egui::Align2::LEFT_TOP,
            x.to_string(),
            font.clone(),
            theme::TEXT_4,
        );
    }
    for y in (0..h).step_by(every) {
        let sy = vp.origin.y + y as f32 * vp.zoom;
        if sy < area.top() || sy > area.bottom() {
            continue;
        }
        painter.hline(
            (left + 4.0)..=(left + 22.0),
            sy,
            Stroke::new(1.0, theme::LINE_STRONG),
        );
        painter.text(
            pos2(left + 20.0, sy + 1.0),
            egui::Align2::RIGHT_TOP,
            y.to_string(),
            font.clone(),
            theme::TEXT_4,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_app::Change;
    use lume_domain::{Command, Document, Origin};
    use std::sync::Arc;

    #[test]
    fn screen_to_pixel_at_several_zooms() {
        for zoom in [1.0, 3.0, 6.0, 17.0, 40.0, 64.0] {
            for origin in [pos2(0.0, 0.0), pos2(123.0, 57.0), pos2(-40.0, 10.0)] {
                let vp = Viewport { origin, zoom };
                for (x, y) in [(0i64, 0i64), (1, 0), (5, 7), (31, 31), (-1, 3)] {
                    let rect = vp.pixel_rect(x, y);
                    // Centro, esquina superior izquierda y justo antes del borde derecho.
                    assert_eq!(
                        vp.pixel_at(rect.center()),
                        (x, y),
                        "zoom {zoom}, centro de {x},{y}"
                    );
                    assert_eq!(
                        vp.pixel_at(rect.min),
                        (x, y),
                        "zoom {zoom}, esquina de {x},{y}"
                    );
                    let inside = rect.max - vec2(0.01, 0.01);
                    assert_eq!(vp.pixel_at(inside), (x, y), "zoom {zoom}, borde de {x},{y}");
                    // Un poco más allá ya es el siguiente.
                    assert_eq!(vp.pixel_at(rect.max), (x + 1, y + 1));
                }
            }
        }
    }

    #[test]
    fn pixel_in_respects_the_canvas_size() {
        let vp = Viewport {
            origin: pos2(100.0, 100.0),
            zoom: 10.0,
        };
        assert_eq!(vp.pixel_in(pos2(100.0, 100.0), 4, 4), Some((0, 0)));
        assert_eq!(vp.pixel_in(pos2(139.9, 139.9), 4, 4), Some((3, 3)));
        assert_eq!(vp.pixel_in(pos2(140.0, 120.0), 4, 4), None);
        assert_eq!(vp.pixel_in(pos2(99.9, 120.0), 4, 4), None);
        assert_eq!(
            vp.canvas_rect(4, 2),
            Rect::from_min_size(pos2(100.0, 100.0), vec2(40.0, 20.0))
        );
        assert_eq!(
            Viewport {
                origin: Pos2::ZERO,
                zoom: 17.0
            }
            .percent(),
            1700
        );
    }

    #[test]
    fn zoom_keeps_the_pixel_under_the_cursor() {
        let area = Rect::from_min_size(pos2(52.0, 84.0), vec2(996.0, 658.0));
        for start in [8.0, 17.0] {
            let mut view = CanvasView {
                fit_pending: false,
                ..CanvasView::default()
            };
            view.zoom = start;
            // El cursor sobre el centro del píxel (10, 21).
            let anchor = view.viewport_for(area, 32, 32).pixel_rect(10, 21).center();
            for zoom in [17.0, 40.0, 64.0, 3.0, 2.0, 12.0] {
                view.zoom_to(zoom, anchor, area);
                let vp = view.viewport_for(area, 32, 32);
                assert_eq!(vp.pixel_at(anchor), (10, 21), "de {start} a {zoom}");
            }
        }
    }

    #[test]
    fn zoom_steps_and_fit() {
        assert_eq!(next_zoom(17.0, 1), 20.0);
        assert_eq!(next_zoom(17.0, -1), 14.0);
        assert_eq!(next_zoom(15.0, -1), 14.0);
        assert_eq!(next_zoom(15.0, 1), 17.0);
        assert_eq!(next_zoom(1.0, -1), 1.0);
        assert_eq!(next_zoom(64.0, 1), 64.0);
        // 32×32 en el área del diseño (996×658) cabe a 17 (1700 %).
        assert_eq!(fit_zoom(vec2(996.0, 658.0), 32, 32), 17.0);
        assert_eq!(fit_zoom(vec2(996.0, 658.0), 128, 128), 4.0);
        assert_eq!(fit_zoom(vec2(100.0, 100.0), 512, 512), 1.0);
    }

    fn vm_after(doc: &mut Document, revision: u64, command: Command) -> ViewModel {
        let patch = doc.apply(command.clone(), Origin::User).unwrap();
        let change = Change {
            revision,
            origin: Origin::User,
            label: command.to_string(),
            patch: Arc::new(patch),
            lines: Arc::new(Vec::new()),
        };
        ViewModel::of(doc, revision, Some(change))
    }

    #[test]
    fn only_the_patch_region_is_uploaded() {
        let mut doc = Document::new("t", 32, 32).unwrap();
        let first = ViewModel::of(&doc, 0, None);
        assert_eq!(plan_update(None, &first), TextureUpdate::Full);
        let uploaded = Some((0, 32, 32));
        assert_eq!(plan_update(uploaded, &first), TextureUpdate::None);

        let vm = vm_after(
            &mut doc,
            1,
            Command::SetPixel {
                x: 21,
                y: 24,
                color: Rgba::rgb(91, 42, 166),
            },
        );
        assert_eq!(
            plan_update(uploaded, &vm),
            TextureUpdate::Region {
                x: 21,
                y: 24,
                w: 1,
                h: 1
            }
        );

        let vm2 = vm_after(
            &mut doc,
            2,
            Command::Line {
                from: (3, 10),
                to: (8, 4),
                color: Rgba::rgb(1, 2, 3),
            },
        );
        assert_eq!(
            plan_update(Some((1, 32, 32)), &vm2),
            TextureUpdate::Region {
                x: 3,
                y: 4,
                w: 6,
                h: 7
            }
        );
        // Revisiones perdidas: todo.
        assert_eq!(plan_update(Some((0, 32, 32)), &vm2), TextureUpdate::Full);
        // Cambio de tamaño: todo.
        let vm3 = vm_after(&mut doc, 3, Command::Resize { w: 40, h: 32 });
        assert_eq!(plan_update(Some((2, 32, 32)), &vm3), TextureUpdate::Full);
    }

    #[test]
    fn region_image_copies_exactly_the_region() {
        let mut canvas = Canvas::new(4, 3).unwrap();
        canvas.set(2, 1, Rgba::new(10, 20, 30, 255)).unwrap();
        let img = region_image(&canvas, 1, 1, 2, 2);
        assert_eq!(img.size, [2, 2]);
        assert_eq!(img.pixels[1], Color32::from_rgb(10, 20, 30));
        assert_eq!(img.pixels[0], Color32::TRANSPARENT);
    }

    #[test]
    fn short_command_drops_the_color() {
        assert_eq!(
            short_command("set_pixel 21 24 #5b2aa6ff"),
            "set_pixel 21 24"
        );
        assert_eq!(short_command("fill_rect 0 0 2 2 red"), "fill_rect 0 0 2 2");
        assert_eq!(short_command("clear"), "clear");
        assert_eq!(short_command("clear #000"), "clear");
        assert_eq!(short_command("undo"), "undo");
        assert_eq!(short_command("batch"), "batch");
        assert_eq!(short_command("new 16 8"), "new 16 8");
    }

    #[test]
    fn sync_texture_uploads_regions() {
        let ctx = egui::Context::default();
        let mut doc = Document::new("t", 16, 16).unwrap();
        let mut view = CanvasView::default();
        view.sync_texture(&ctx, &ViewModel::of(&doc, 0, None));
        assert_eq!(view.last_update, TextureUpdate::Full);
        let vm = vm_after(
            &mut doc,
            1,
            Command::FillRect {
                x: 2,
                y: 3,
                w: 4,
                h: 2,
                color: Rgba::rgb(255, 0, 0),
            },
        );
        view.sync_texture(&ctx, &vm);
        assert_eq!(
            view.last_update,
            TextureUpdate::Region {
                x: 2,
                y: 3,
                w: 4,
                h: 2
            }
        );
    }
}
