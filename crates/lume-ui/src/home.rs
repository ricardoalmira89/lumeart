//! Pantalla de inicio (`docs/diseno/capturas/02-inicio.png`,
//! `Inicio.dc.html`): nuevo dibujo, tamaño del lienzo y recientes. Como el
//! armazón, solo lee datos y devuelve `Intent`s.

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Frame, Layout, Margin, Rect, RichText, Sense,
    Stroke, StrokeKind, TextureHandle, TextureOptions, UiBuilder, pos2, vec2,
};
use lume_app::RecentEntry;
use lume_domain::MAX_CANVAS_SIZE;

use crate::icons::Icon;
use crate::intent::{Intent, Mode};
use crate::theme;

/// Tamaños predefinidos (rejilla 4 × 2 del diseño).
pub const PRESETS: [(u32, u32); 8] = [
    (16, 16),
    (24, 24),
    (32, 32),
    (48, 48),
    (64, 64),
    (128, 128),
    (64, 32),
    (32, 64),
];

/// Paletas iniciales: (nombre en la interfaz, paleta fija o `None` = libre).
pub const PALETTES: [(&str, Option<&str>); 3] = [
    ("Libre", None),
    ("PICO-8", Some("pico-8")),
    ("Game Boy", Some("game-boy")),
];

/// Mensaje cuando el tamaño no vale.
pub const SIZE_ERROR: &str = "El ancho y el alto deben ser números enteros entre 1 y 512";

/// «hace 5 min», «hace 2 h», «ayer», «28 oct» (en UTC).
pub fn relative_time(now: u64, then: u64) -> String {
    let diff = now.saturating_sub(then);
    match diff {
        0..60 => "hace un momento".into(),
        60..3600 => format!("hace {} min", diff / 60),
        3600..86_400 => format!("hace {} h", diff / 3600),
        86_400..172_800 => "ayer".into(),
        _ => {
            let (_, month, day) = civil_from_days((then / 86_400) as i64);
            const MONTHS: [&str; 12] = [
                "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
            ];
            format!("{day} {}", MONTHS[(month - 1) as usize])
        }
    }
}

/// Fecha (año, mes, día) de un número de días desde 1970-01-01 (algoritmo de
/// Howard Hinnant).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Estado de la pantalla de inicio.
pub struct HomeScreen {
    /// «¿Qué quieres dibujar?»
    pub prompt: String,
    pub width: String,
    pub height: String,
    /// Índice en [`PALETTES`].
    pub palette: usize,
    /// Mensaje de error (por ejemplo, al abrir un reciente).
    pub error: Option<String>,
    /// Proveedor del agente elegido en Ajustes («Claude Code»…).
    pub provider: String,
    /// Encargos que llegaron por MCP (encima de los recientes).
    pub jobs: Vec<crate::jobs::JobItem>,
    thumbnails: HashMap<(PathBuf, u64), TextureHandle>,
    job_thumbnails: HashMap<(lume_app::ArtId, u64), TextureHandle>,
    /// El diálogo «Nuevo pixel art» (tamaño y paleta) está abierto.
    pub asking_size: bool,
    /// Proporción fija del pixel art (`None`: libre).
    pub pixel_ratio: Option<(u32, u32)>,
    /// El diálogo «Nuevo dibujo vectorial» está abierto, con su tamaño.
    pub asking_vector: bool,
    pub vector_size: crate::size_picker::SizeChoice,
    pub vector_transparent: bool,
}

impl Default for HomeScreen {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            width: "32".into(),
            height: "32".into(),
            palette: 0,
            error: None,
            provider: "Claude Code".into(),
            jobs: Vec::new(),
            thumbnails: HashMap::new(),
            job_thumbnails: HashMap::new(),
            asking_size: false,
            pixel_ratio: Some((1, 1)),
            asking_vector: false,
            vector_size: Default::default(),
            vector_transparent: false,
        }
    }
}

impl HomeScreen {
    /// Tamaño elegido, o el mensaje de error si no vale.
    pub fn size(&self) -> Result<(u32, u32), &'static str> {
        let parse = |s: &str| {
            s.trim()
                .parse::<u32>()
                .ok()
                .filter(|v| (1..=MAX_CANVAS_SIZE).contains(v))
        };
        match (parse(&self.width), parse(&self.height)) {
            (Some(w), Some(h)) => Ok((w, h)),
            _ => Err(SIZE_ERROR),
        }
    }

    /// Dibuja la pantalla. `now` en segundos desde 1970 (para «hace 5 min»).
    ///
    /// Como la bienvenida de un editor: arriba el logo y el nombre; a la
    /// izquierda «Empezar» (lienzos nuevos de cada modo, abrir, ajustes) y el
    /// encargo al agente; a la derecha los recientes en lista y los
    /// encargos; abajo, atajos y consejos.
    pub fn show(&mut self, ui: &mut egui::Ui, recents: &[RecentEntry], now: u64) -> Vec<Intent> {
        self.show_with_tabs(ui, recents, now, None)
    }

    /// Como [`HomeScreen::show`], con la tira de pestañas del studio bajo la
    /// cabecera (Inicio abierto en una pestaña).
    pub fn show_with_tabs(
        &mut self,
        ui: &mut egui::Ui,
        recents: &[RecentEntry],
        now: u64,
        tabs: Option<&mut dyn FnMut(&mut egui::Ui)>,
    ) -> Vec<Intent> {
        let mut out = Vec::new();
        let full = ui.max_rect();
        let header = Rect::from_min_size(full.min, vec2(full.width(), theme::HEADER_HEIGHT));
        let strip_h = if tabs.is_some() {
            crate::shell::TAB_STRIP_HEIGHT
        } else {
            0.0
        };
        let strip = Rect::from_min_size(
            pos2(full.left(), header.bottom()),
            vec2(full.width(), strip_h),
        );
        let status = Rect::from_min_max(
            pos2(full.left(), full.bottom() - theme::STATUS_HEIGHT),
            full.max,
        );
        let body = Rect::from_min_max(
            pos2(full.left(), strip.bottom()),
            pos2(full.right(), status.top()),
        );
        let painter = ui.painter();
        painter.rect_filled(full, 0.0, theme::BG);
        painter.rect_filled(header, 0.0, theme::BAR);
        painter.hline(
            header.x_range(),
            header.bottom() - 0.5,
            Stroke::new(1.0, theme::LINE),
        );
        painter.rect_filled(status, 0.0, theme::BAR);
        painter.hline(
            status.x_range(),
            status.top() + 0.5,
            Stroke::new(1.0, theme::LINE),
        );

        // Cabecera.
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(header.shrink2(vec2(14.0, 0.0)))
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                crate::shell::app_icon(ui);
                ui.label(
                    RichText::new("LumeArt")
                        .font(theme::semibold(13.0))
                        .color(theme::TEXT),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let settings = ui.add(
                        egui::Button::new(RichText::new("Ajustes").size(13.0).color(theme::TEXT_2))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE),
                    );
                    if settings.clicked() {
                        out.push(Intent::OpenSettings);
                    }
                });
            },
        );

        if let Some(tabs) = tabs {
            ui.scope_builder(
                UiBuilder::new()
                    .max_rect(strip)
                    .layout(Layout::left_to_right(Align::Center)),
                |ui| tabs(ui),
            );
        }

        // Cuerpo: una columna centrada de hasta 1180 px, con su desplazamiento.
        ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
            egui::ScrollArea::vertical()
                .id_salt("inicio")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let width = (body.width() - 96.0).clamp(640.0, 1180.0);
                    let margin = ((body.width() - width) / 2.0).max(24.0);
                    ui.horizontal(|ui| {
                        ui.add_space(margin);
                        ui.vertical(|ui| {
                            ui.set_width(width);
                            ui.add_space(40.0);
                            self.hero(ui);
                            ui.add_space(36.0);
                            let left_w = (width * 0.44).clamp(420.0, 520.0);
                            let gap = 56.0;
                            let right_w = width - left_w - gap;
                            ui.horizontal_top(|ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(left_w);
                                    self.start(ui, &mut out);
                                });
                                ui.add_space(gap);
                                ui.vertical(|ui| {
                                    ui.set_width(right_w);
                                    self.recents(ui, recents, now, &mut out);
                                    ui.add_space(28.0);
                                    tips(ui);
                                });
                            });
                            ui.add_space(40.0);
                        });
                    });
                });
        });

        self.new_pixel_dialog(ui.ctx(), &mut out);
        self.new_vector_dialog(ui.ctx(), &mut out);

        // Barra de estado.
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(status.shrink2(vec2(14.0, 0.0)))
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 18.0;
                let text = |s: &str| RichText::new(s).size(11.5).color(theme::TEXT_3);
                ui.label(text(&format!("Agente: {}", self.provider)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("lumeart {}", env!("CARGO_PKG_VERSION")))
                            .font(theme::mono(11.5))
                            .color(theme::TEXT_3),
                    );
                });
            },
        );
        out
    }

    /// El logo grande, el nombre y qué es.
    fn hero(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 18.0;
            crate::shell::app_icon_sized(ui, 64.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.add_space(4.0);
                ui.label(
                    RichText::new("LumeArt")
                        .font(theme::semibold(32.0))
                        .color(theme::TEXT),
                );
                ui.label(
                    RichText::new(
                        "Studio de arte para videojuegos: pixel art, vector y animación con un agente de IA",
                    )
                    .size(14.0)
                    .color(theme::TEXT_2),
                );
            });
        });
    }

    /// «Empezar»: lienzos nuevos de cada modo, abrir y ajustes.
    fn start(&mut self, ui: &mut egui::Ui, out: &mut Vec<Intent>) {
        section_title(ui, "Empezar");
        let size = self.size().unwrap_or((32, 32));
        let palette = PALETTES[self.palette].1.map(str::to_string);
        let new_document = |out: &mut Vec<Intent>, width: u32, height: u32| {
            out.push(Intent::NewDocument {
                width,
                height,
                palette: palette.clone(),
            });
        };
        if action(
            ui,
            Icon::Pencil,
            "Nuevo pixel art…",
            "eliges tamaño y paleta",
        ) {
            self.asking_size = true;
        }
        if action(
            ui,
            Icon::Curve,
            "Nuevo dibujo vectorial…",
            "eliges tamaño y proporción",
        ) {
            self.asking_vector = true;
        }
        if action(
            ui,
            Icon::Play,
            "Nueva animación",
            "un personaje y sus acciones",
        ) {
            new_document(out, size.0, size.1);
            out.push(Intent::SwitchMode(Mode::Animation));
        }
        if action(
            ui,
            Icon::Folder,
            "Abrir archivo…",
            "Ctrl+O · .lume, .png o un proyecto de pxcli",
        ) {
            out.push(Intent::OpenFile);
        }
        if action(
            ui,
            Icon::Settings,
            "Ajustes",
            "Ctrl+, · proveedores de IA y el bucle",
        ) {
            out.push(Intent::OpenSettings);
        }
    }

    /// «Nuevo pixel art»: tamaño (casillas o ancho y alto) y paleta.
    fn new_pixel_dialog(&mut self, ctx: &egui::Context, out: &mut Vec<Intent>) {
        if !self.asking_size {
            return;
        }
        let modal = egui::Modal::new(egui::Id::unique("lumeart-nuevo-pixel")).show(ctx, |ui| {
            ui.set_width(420.0);
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.label(
                RichText::new("Nuevo pixel art")
                    .font(theme::semibold(16.0))
                    .color(theme::TEXT),
            );
            let current = self.size().ok();
            ui.label(RichText::new("Tamaño").size(12.5).color(theme::TEXT_2));
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                for &(w, h) in &PRESETS {
                    if size_chip(ui, w, h, current == Some((w, h))).clicked() {
                        let choice = crate::size_picker::SizeChoice::new(w, h, MAX_CANVAS_SIZE);
                        self.width = choice.width;
                        self.height = choice.height;
                        self.pixel_ratio = choice.ratio;
                    }
                }
            });
            // Proporción, ancho y alto (con la proporción, uno sigue al otro).
            let mut choice = crate::size_picker::SizeChoice {
                width: std::mem::take(&mut self.width),
                height: std::mem::take(&mut self.height),
                ratio: self.pixel_ratio,
                max: MAX_CANVAS_SIZE,
            };
            choice.ui(ui);
            self.width = choice.width;
            self.height = choice.height;
            self.pixel_ratio = choice.ratio;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let label = ui.label(RichText::new("Paleta").size(12.5).color(theme::TEXT_2));
                let combo = egui::ComboBox::from_id_salt("paleta-inicial")
                    .width(96.0)
                    .selected_text(PALETTES[self.palette].0)
                    .show_ui(ui, |ui| {
                        for (i, (name, _)) in PALETTES.iter().enumerate() {
                            ui.selectable_value(&mut self.palette, i, *name);
                        }
                    });
                combo.response.labelled_by(label.id);
            });
            let size = self.size();
            if let Err(message) = size {
                ui.label(RichText::new(message).size(12.0).color(theme::WARN));
            }
            ui.horizontal(|ui| {
                if ui.button("Cancelar").clicked() {
                    self.asking_size = false;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let create = ui.add_enabled(
                        size.is_ok(),
                        egui::Button::new(
                            RichText::new("Crear lienzo en blanco")
                                .font(theme::semibold(13.0))
                                .color(theme::ACCENT_INK),
                        )
                        .fill(theme::ACCENT)
                        .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
                        .min_size(vec2(0.0, 32.0)),
                    );
                    if create.clicked()
                        && let Ok((width, height)) = size
                    {
                        out.push(Intent::NewDocument {
                            width,
                            height,
                            palette: PALETTES[self.palette].1.map(str::to_string),
                        });
                        out.push(Intent::SwitchMode(Mode::Pixel));
                        self.asking_size = false;
                    }
                });
            });
        });
        if modal.should_close() {
            self.asking_size = false;
        }
    }

    /// «Nuevo dibujo vectorial»: proporción, tamaño y fondo.
    fn new_vector_dialog(&mut self, ctx: &egui::Context, out: &mut Vec<Intent>) {
        if !self.asking_vector {
            return;
        }
        let modal = egui::Modal::new(egui::Id::unique("lumeart-nuevo-vector")).show(ctx, |ui| {
            ui.set_width(440.0);
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.label(
                RichText::new("Nuevo dibujo vectorial")
                    .font(theme::semibold(16.0))
                    .color(theme::TEXT),
            );
            let current = self.vector_size.size().ok();
            ui.label(RichText::new("Tamaño").size(12.5).color(theme::TEXT_2));
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                for &(w, h) in &crate::size_picker::VECTOR_PRESETS {
                    if size_chip(ui, w, h, current == Some((w, h))).clicked() {
                        self.vector_size.set(w, h);
                    }
                }
            });
            self.vector_size.ui(ui);
            ui.checkbox(
                &mut self.vector_transparent,
                RichText::new("Fondo transparente (un objeto suelto)").size(12.5),
            );
            let size = self.vector_size.size();
            if let Err(message) = &size {
                ui.label(RichText::new(message).size(12.0).color(theme::WARN));
            }
            ui.horizontal(|ui| {
                if ui.button("Cancelar").clicked() {
                    self.asking_vector = false;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let create = ui.add_enabled(
                        size.is_ok(),
                        egui::Button::new(
                            RichText::new("Crear dibujo en blanco")
                                .font(theme::semibold(13.0))
                                .color(theme::ACCENT_INK),
                        )
                        .fill(theme::ACCENT)
                        .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
                        .min_size(vec2(0.0, 32.0)),
                    );
                    if create.clicked()
                        && let Ok((width, height)) = size
                    {
                        out.push(Intent::NewDocument {
                            width: 32,
                            height: 32,
                            palette: None,
                        });
                        out.push(Intent::SwitchMode(Mode::Vector));
                        out.push(Intent::NewVector {
                            width,
                            height,
                            transparent: self.vector_transparent,
                        });
                        self.asking_vector = false;
                    }
                });
            });
        });
        if modal.should_close() {
            self.asking_vector = false;
        }
    }

    fn recents(
        &mut self,
        ui: &mut egui::Ui,
        recents: &[RecentEntry],
        now: u64,
        out: &mut Vec<Intent>,
    ) {
        if !self.jobs.is_empty() {
            section_title(ui, "Encargos");
            let jobs = self.jobs.clone();
            for job in &jobs {
                let texture = self.job_thumbnail(ui.ctx(), job);
                if recent_row(ui, &job.name, "", &job.state_text, &texture, &job.canvas).clicked() {
                    out.push(Intent::OpenJob(job.id));
                }
            }
            ui.add_space(24.0);
        }
        ui.horizontal(|ui| {
            section_title(ui, "Recientes");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let open = ui.add(
                    egui::Button::new(
                        RichText::new("Abrir archivo")
                            .size(12.5)
                            .color(theme::TEXT_2),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                    .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
                    .min_size(vec2(0.0, 28.0)),
                );
                if open.clicked() {
                    out.push(Intent::OpenFile);
                }
            });
        });
        if let Some(error) = &self.error {
            ui.label(RichText::new(error).size(12.0).color(theme::DANGER));
        }
        if recents.is_empty() {
            ui.add_space(6.0);
            ui.label(
                RichText::new("Aún no hay dibujos recientes.")
                    .size(13.0)
                    .color(theme::TEXT_3),
            );
            return;
        }
        for entry in recents.iter().take(12) {
            let texture = self.thumbnail(ui.ctx(), entry);
            let folder = entry
                .path
                .parent()
                .map(|p| {
                    let home = std::env::var("HOME").unwrap_or_default();
                    let text = p.display().to_string();
                    match text.strip_prefix(&home) {
                        Some(rest) if !home.is_empty() => format!("~{rest}"),
                        _ => text,
                    }
                })
                .unwrap_or_default();
            let meta = format!(
                "{} × {} · {}",
                entry.thumbnail.width(),
                entry.thumbnail.height(),
                relative_time(now, entry.opened_at)
            );
            if recent_row(ui, &entry.name, &folder, &meta, &texture, &entry.thumbnail).clicked() {
                out.push(Intent::OpenRecent(entry.path.clone()));
            }
        }
    }

    /// Miniatura del lienzo de un encargo (se rehace cuando cambia).
    fn job_thumbnail(&mut self, ctx: &egui::Context, job: &crate::jobs::JobItem) -> TextureHandle {
        let key = (job.id, job.revision);
        if !self.job_thumbnails.contains_key(&key) {
            self.job_thumbnails.retain(|(id, _), _| *id != job.id);
        }
        self.job_thumbnails
            .entry(key)
            .or_insert_with(|| {
                let c = &job.canvas;
                let image = crate::canvas_view::region_image(c, 0, 0, c.width(), c.height());
                ctx.load_texture(
                    format!("encargo-{}", job.id),
                    image,
                    TextureOptions::NEAREST,
                )
            })
            .clone()
    }

    fn thumbnail(&mut self, ctx: &egui::Context, entry: &RecentEntry) -> TextureHandle {
        let key = (entry.path.clone(), entry.opened_at);
        self.thumbnails
            .entry(key)
            .or_insert_with(|| {
                let c = &entry.thumbnail;
                let image = crate::canvas_view::region_image(c, 0, 0, c.width(), c.height());
                ctx.load_texture(
                    format!("reciente-{}", entry.path.display()),
                    image,
                    TextureOptions::NEAREST,
                )
            })
            .clone()
    }
}

/// Título de sección («Empezar», «Recientes»…).
fn section_title(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .font(theme::semibold(15.0))
            .color(theme::TEXT),
    );
    ui.add_space(4.0);
}

/// Una acción de «Empezar»: icono, nombre en acento y una pista.
fn action(ui: &mut egui::Ui, icon: Icon, name: &str, hint: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name));
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, CornerRadius::same(theme::RADIUS_BUTTON), theme::HOVER);
    }
    crate::icons::paint(
        painter,
        icon,
        pos2(rect.left() + 18.0, rect.center().y),
        theme::ACCENT,
    );
    painter.text(
        pos2(rect.left() + 40.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        name,
        FontId::proportional(13.5),
        if response.hovered() {
            theme::TEXT
        } else {
            theme::ACCENT
        },
    );
    let name_w = painter
        .layout_no_wrap(name.to_string(), FontId::proportional(13.5), theme::TEXT)
        .size()
        .x;
    painter.text(
        pos2(rect.left() + 40.0 + name_w + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        hint,
        FontId::proportional(12.0),
        theme::TEXT_4,
    );
    response.clicked()
}

/// Casilla de tamaño compacta: «32 × 32» en mono.
fn size_chip(ui: &mut egui::Ui, w: u32, h: u32, selected: bool) -> egui::Response {
    let label = format!("{w} × {h}");
    let galley = ui
        .painter()
        .layout_no_wrap(label.clone(), FontId::monospace(12.0), theme::TEXT);
    let (rect, response) =
        ui.allocate_exact_size(vec2(galley.size().x + 20.0, 28.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, selected, &label));
    let painter = ui.painter();
    let (fill, stroke, text) = if selected {
        (theme::ACCENT_SOFT, theme::ACCENT, theme::TEXT)
    } else if response.hovered() {
        (theme::HOVER, theme::LINE_STRONG, theme::TEXT)
    } else {
        (Color32::TRANSPARENT, theme::LINE_SOFT, theme::TEXT_2)
    };
    painter.rect_filled(rect, CornerRadius::same(theme::RADIUS_FIELD), fill);
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_FIELD),
        Stroke::new(1.0, stroke),
        StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        &label,
        FontId::monospace(12.0),
        text,
    );
    response
}

/// Una fila de recientes o encargos: miniatura sobre cuadros, nombre,
/// carpeta y, a la derecha, tamaño y cuándo.
fn recent_row(
    ui: &mut egui::Ui,
    name: &str,
    folder: &str,
    meta: &str,
    texture: &TextureHandle,
    canvas: &lume_domain::Canvas,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 58.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name));
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, CornerRadius::same(theme::RADIUS_BUTTON), theme::HOVER);
    }
    let thumb = Rect::from_min_size(rect.min + vec2(8.0, 7.0), vec2(44.0, 44.0));
    painter.rect_filled(thumb, CornerRadius::same(4), theme::CHECKER_A);
    let half = thumb.width() / 2.0;
    for (dx, dy) in [(0.0, 0.0), (half, half)] {
        painter.rect_filled(
            Rect::from_min_size(thumb.min + vec2(dx, dy), vec2(half, half)),
            0.0,
            theme::CHECKER_B,
        );
    }
    let (w, h) = (canvas.width() as f32, canvas.height() as f32);
    let k = (thumb.width() / w).min(thumb.height() / h);
    painter.image(
        texture.id(),
        Rect::from_center_size(thumb.center(), vec2(w * k, h * k)),
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    painter.rect_stroke(
        thumb,
        CornerRadius::same(4),
        Stroke::new(1.0, theme::LINE_SOFT),
        StrokeKind::Inside,
    );
    let x = thumb.right() + 14.0;
    let right = painter.text(
        pos2(rect.right() - 12.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        meta,
        FontId::proportional(12.0),
        theme::TEXT_3,
    );
    let room = (right.left() - x - 16.0).max(40.0);
    let fit = |text: &str, font: FontId| {
        let mut t = text.to_string();
        while t.chars().count() > 4
            && painter
                .layout_no_wrap(t.clone(), font.clone(), theme::TEXT)
                .size()
                .x
                > room
        {
            let keep: String = t.chars().skip(2).collect();
            t = format!("…{keep}");
        }
        t
    };
    let has_folder = !folder.is_empty();
    painter.text(
        pos2(
            x,
            if has_folder {
                rect.center().y - 9.0
            } else {
                rect.center().y
            },
        ),
        egui::Align2::LEFT_CENTER,
        name,
        theme::semibold(13.5),
        theme::TEXT,
    );
    if has_folder {
        painter.text(
            pos2(x, rect.center().y + 10.0),
            egui::Align2::LEFT_CENTER,
            fit(folder, FontId::proportional(12.0)),
            FontId::proportional(12.0),
            theme::TEXT_4,
        );
    }
    response
}

/// Atajos y consejos útiles.
fn tips(ui: &mut egui::Ui) {
    section_title(ui, "Atajos y consejos");
    Frame::NONE
        .fill(theme::CARD)
        .stroke(Stroke::new(1.0, theme::LINE_SOFT))
        .corner_radius(CornerRadius::same(theme::RADIUS_CARD))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let rows: [(&str, &str); 7] = [
                ("Ctrl+N", "Inicio en una pestaña nueva"),
                ("Ctrl+O", "Abrir un archivo"),
                ("Ctrl+S", "Guardar"),
                ("Ctrl+Z / Ctrl+Y", "Deshacer y rehacer"),
                (
                    "B · E · G · L · U · O · I",
                    "Lápiz, borrador, relleno, línea, rectángulo, elipse, cuentagotas",
                ),
                (
                    "Ctrl+V",
                    "Pegar una imagen de referencia en el mensaje al agente",
                ),
                (
                    "lumeart setup",
                    "Usar LumeArt desde Claude Code o Codex (en una terminal)",
                ),
            ];
            egui::Grid::new("atajos")
                .num_columns(2)
                .spacing(vec2(18.0, 8.0))
                .show(ui, |ui| {
                    for (keys, what) in rows {
                        ui.label(
                            RichText::new(keys)
                                .font(theme::mono(12.0))
                                .color(theme::ACCENT),
                        );
                        ui.label(RichText::new(what).size(12.5).color(theme::TEXT_2));
                        ui.end_row();
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_times() {
        let now = 1_793_188_845; // 31 oct 2026 aprox.
        assert_eq!(relative_time(now, now - 10), "hace un momento");
        assert_eq!(relative_time(now, now - 5 * 60), "hace 5 min");
        assert_eq!(relative_time(now, now - 2 * 3600), "hace 2 h");
        assert_eq!(relative_time(now, now - 30 * 3600), "ayer");
        // 1 761 609 600 = 28 oct 2025 00:00 UTC.
        assert_eq!(relative_time(now, 1_761_609_600), "28 oct");
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_389), (2025, 10, 28));
    }

    #[test]
    fn size_validation() {
        let mut home = HomeScreen::default();
        assert_eq!(home.size(), Ok((32, 32)));
        for (w, h) in [
            ("0", "32"),
            ("32", "513"),
            ("abc", "8"),
            ("", "8"),
            ("-4", "4"),
            ("3.5", "4"),
        ] {
            home.width = w.into();
            home.height = h.into();
            assert_eq!(home.size(), Err(SIZE_ERROR), "{w}×{h}");
        }
        home.width = " 64 ".into();
        home.height = "32".into();
        assert_eq!(home.size(), Ok((64, 32)));
    }
}
