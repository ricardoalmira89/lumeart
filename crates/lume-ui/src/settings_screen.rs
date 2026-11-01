//! Pantalla de Ajustes (`07-ajustes-proveedores.png`, `Proveedores.dc.html`):
//! proveedores del agente, bucle de refinado y vista.
//!
//! Como el resto de la UI, no guarda nada por sí misma: cada cambio sale como
//! un [`Intent`] (`UpdateSettings`, `SaveApiKey`…) y lo atiende el
//! controlador. Las claves solo se ven enmascaradas ([`KeyStatus`]); lo que se
//! escribe en el campo se borra en cuanto se envía.

use std::path::PathBuf;

use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Layout, Margin, Pos2, Rect, RichText, Sense, Shape,
    Stroke, StrokeKind, UiBuilder, pos2, text::LayoutJob, vec2,
};
use lume_app::ports::Secret;
use lume_app::settings::MAX_TURNS;
use lume_app::{ContextPolicy, ProviderKind, Settings};

use crate::intent::Intent;
use crate::theme;

/// Modelos de Claude Code (`claude -p --model`). Vacío: el que tenga
/// configurado Claude Code.
pub const CLAUDE_CODE_MODELS: &[(&str, &str)] = &[
    ("", "Por defecto"),
    ("opus", "Opus"),
    ("sonnet", "Sonnet"),
    ("haiku", "Haiku"),
];

/// Modelos de la Messages API de Anthropic.
pub const ANTHROPIC_MODELS: &[(&str, &str)] = &[
    ("claude-opus-5-5", "Opus 5.5"),
    ("claude-sonnet-5-5", "Sonnet 5.5"),
    ("claude-haiku-5-5", "Haiku 5.5"),
    ("claude-fable-5-1", "Fable 5.1"),
];

/// Modelos de OpenAI.
pub const OPENAI_MODELS: &[(&str, &str)] = &[("gpt-5", "GPT-5"), ("gpt-5-mini", "GPT-5 mini")];

/// Secciones de la navegación de la izquierda.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    General,
    #[default]
    Providers,
    Canvas,
    Shortcuts,
    Export,
    Server,
}

impl Section {
    pub const ALL: [Section; 6] = [
        Section::General,
        Section::Providers,
        Section::Canvas,
        Section::Shortcuts,
        Section::Export,
        Section::Server,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Section::General => "General",
            Section::Providers => "Proveedores de IA",
            Section::Canvas => "Lienzo",
            Section::Shortcuts => "Atajos de teclado",
            Section::Export => "Exportación",
            Section::Server => "Servidor local",
        }
    }
}

/// Qué claves hay en el llavero, enmascaradas («sk-ant-••••»).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyStatus {
    pub anthropic: Option<String>,
    pub openai: Option<String>,
    /// Si no se pudo leer el llavero, por qué.
    pub error: Option<String>,
}

impl KeyStatus {
    pub fn get(&self, provider: ProviderKind) -> Option<&str> {
        match provider {
            ProviderKind::Anthropic => self.anthropic.as_deref(),
            ProviderKind::OpenAi => self.openai.as_deref(),
            ProviderKind::ClaudeCode | ProviderKind::Local => None,
        }
    }
}

/// Estado de la pantalla de Ajustes (lo que no son ajustes guardados).
#[derive(Debug, Default)]
pub struct SettingsScreen {
    pub section: Section,
    /// Ruta de `claude`, si está instalado.
    pub claude_path: Option<PathBuf>,
    /// Dirección de la API local («127.0.0.1:39223»).
    pub server: Option<String>,
    /// Último aviso («Clave guardada en el llavero») o error.
    pub notice: Option<String>,
    /// Lo que se está escribiendo en cada campo de clave.
    anthropic_key: String,
    openai_key: String,
    /// Proveedor cuya clave guardada se está cambiando («Cambiar»).
    changing: Option<ProviderKind>,
}

impl SettingsScreen {
    pub fn new(claude_path: Option<PathBuf>, server: Option<String>) -> Self {
        Self {
            claude_path,
            server,
            ..Self::default()
        }
    }

    /// Dibuja los ajustes en todo `ui` (su propia ventana) y devuelve lo que
    /// pide el usuario. Esc o cerrar la ventana los cierra.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        settings: &Settings,
        keys: &KeyStatus,
    ) -> Vec<Intent> {
        let mut out = Vec::new();
        let mut next = settings.clone();
        let full = ui.max_rect();
        // Va en su propia ventana: el título lo pone el sistema.
        let nav = Rect::from_min_max(full.min, pos2(full.left() + 240.0, full.bottom()));
        let main = Rect::from_min_max(pos2(nav.right(), full.top()), full.max);
        let painter = ui.painter();
        painter.rect_filled(full, 0.0, theme::BG);
        painter.rect_filled(nav, 0.0, theme::BAR);
        painter.vline(
            nav.right() - 0.5,
            nav.y_range(),
            Stroke::new(1.0, theme::LINE),
        );
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) && ui.memory(|m| m.focused().is_none()) {
            out.push(Intent::CloseSettings);
        }

        // Navegación.
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(nav.shrink2(vec2(12.0, 20.0)))
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for section in Section::ALL {
                    if nav_item(ui, section.label(), self.section == section).clicked() {
                        self.section = section;
                    }
                }
            },
        );

        // Contenido.
        ui.scope_builder(UiBuilder::new().max_rect(main), |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Frame::NONE
                        .inner_margin(Margin::symmetric(56, 28))
                        .show(ui, |ui| match self.section {
                            Section::Providers => self.providers(ui, &mut next, keys, &mut out),
                            Section::Server => {
                                let text = match &self.server {
                                    Some(addr) => format!(
                                        "La CLI y los agentes hablan con esta ventana por la API local en {addr} (solo desde este equipo)."
                                    ),
                                    None => "Esta ventana no tiene API local.".to_string(),
                                };
                                page_title(ui, Section::Server.label(), &text);
                            }
                            other => page_title(ui, other.label(), "Próximamente."),
                        });
                });
        });

        if next != *settings {
            out.push(Intent::UpdateSettings(next));
        }
        out
    }

    fn providers(
        &mut self,
        ui: &mut egui::Ui,
        next: &mut Settings,
        keys: &KeyStatus,
        out: &mut Vec<Intent>,
    ) {
        page_title(
            ui,
            "Proveedores de IA",
            "El agente puede dibujar con tu suscripción de Claude Code o con la clave de API del proveedor que prefieras. Las claves se guardan en el llavero del sistema, nunca en el proyecto.",
        );
        if let Some(message) = self.notice.clone().or_else(|| keys.error.clone()) {
            ui.add_space(12.0);
            let error =
                message.starts_with("No se pudo") || keys.error.as_deref() == Some(&message);
            ui.label(RichText::new(message).size(12.5).color(if error {
                theme::DANGER
            } else {
                theme::OK
            }));
        }
        ui.add_space(20.0);

        let width = ui.available_width().min(1080.0);
        let col = ((width - 16.0) / 2.0).floor();
        let left = ui.cursor().left();
        let mut y = ui.cursor().top();

        // Fila 1: Claude Code · Anthropic API.
        let h = card_row(
            ui,
            [left, left + col + 16.0],
            y,
            col,
            [
                card_style(next.provider == ProviderKind::ClaudeCode),
                card_style(next.provider == ProviderKind::Anthropic),
            ],
            |ui, i| {
                if i == 0 {
                    self.claude_code_card(ui, next);
                } else {
                    self.api_card(ui, ProviderKind::Anthropic, next, keys, out);
                }
            },
        );
        y += h + 16.0;
        // Fila 2: OpenAI · modelos locales.
        let h = card_row(
            ui,
            [left, left + col + 16.0],
            y,
            col,
            [
                card_style(next.provider == ProviderKind::OpenAi),
                CardStyle::Dashed,
            ],
            |ui, i| {
                if i == 0 {
                    self.api_card(ui, ProviderKind::OpenAi, next, keys, out);
                } else {
                    local_card(ui, next);
                }
            },
        );
        y += h + 20.0;
        // Fila 3: bucle de refinado · revisión y vista.
        let h = card_row(
            ui,
            [left, left + col + 16.0],
            y,
            col,
            [CardStyle::Section, CardStyle::Section],
            |ui, i| {
                if i == 0 {
                    loop_card(ui, next);
                } else {
                    review_card(ui, next);
                }
            },
        );
        let _ = h;
    }

    fn claude_code_card(&mut self, ui: &mut egui::Ui, next: &mut Settings) {
        let default = next.provider == ProviderKind::ClaudeCode;
        let mut desc = LayoutJob::default();
        let body = |size: f32, mono: bool| egui::TextFormat {
            font_id: if mono {
                theme::mono(size)
            } else {
                FontId::proportional(size)
            },
            color: theme::TEXT_2,
            ..Default::default()
        };
        desc.append("Usa tu sesión de Claude Code con ", 0.0, body(13.0, false));
        desc.append("claude -p", 0.0, body(12.0, true));
        desc.append(". No necesita clave.", 0.0, body(13.0, false));
        card_header(
            ui,
            BoxIcon::Terminal,
            "Claude Code",
            default.then_some(Badge::Default),
            desc,
        );

        // ¿Está instalado?
        egui::Frame::NONE
            .fill(theme::BAR_ALT)
            .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let found = self.claude_path.is_some();
                    let (dot, _) = ui.allocate_exact_size(vec2(7.0, 7.0), Sense::hover());
                    ui.painter().circle_filled(
                        dot.center(),
                        3.5,
                        if found { theme::OK } else { theme::WARN },
                    );
                    match &self.claude_path {
                        Some(path) => {
                            ui.label(
                                RichText::new("Detectado en")
                                    .size(12.0)
                                    .color(theme::TEXT_2),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(path.display().to_string())
                                        .font(theme::mono(12.0))
                                        .color(theme::TEXT),
                                )
                                .truncate(),
                            );
                        }
                        None => {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(
                                        "No encuentro `claude`. Instala Claude Code para usarlo.",
                                    )
                                    .size(12.0)
                                    .color(theme::TEXT_2),
                                )
                                .wrap(),
                            );
                        }
                    }
                });
            });

        model_row(
            ui,
            "Modelo de Claude Code",
            &mut next.claude_code_model,
            CLAUDE_CODE_MODELS,
            None,
            false,
        );
        ui.label(
            RichText::new("Solo puede usar los comandos de lumeart y leer imágenes del dibujo.")
                .size(12.0)
                .color(theme::TEXT_3),
        );
        if !default
            && ui
                .add(text_button("Usar por defecto"))
                .accessible_name("Usar Claude Code por defecto")
                .clicked()
        {
            next.provider = ProviderKind::ClaudeCode;
        }
    }

    fn api_card(
        &mut self,
        ui: &mut egui::Ui,
        provider: ProviderKind,
        next: &mut Settings,
        keys: &KeyStatus,
        out: &mut Vec<Intent>,
    ) {
        let (title, letter, desc, models) = match provider {
            ProviderKind::Anthropic => (
                "Anthropic API",
                "A",
                "Modelos Claude con tu propia clave de API.",
                ANTHROPIC_MODELS,
            ),
            _ => (
                "OpenAI API",
                "O",
                "Modelos de OpenAI con visión y uso de herramientas.",
                OPENAI_MODELS,
            ),
        };
        let saved = keys.get(provider).map(str::to_string);
        let default = next.provider == provider;
        let badge = if default {
            Badge::Default
        } else if saved.is_some() {
            Badge::Saved
        } else {
            Badge::Unset
        };
        let mut job = LayoutJob::default();
        job.append(
            desc,
            0.0,
            egui::TextFormat {
                font_id: FontId::proportional(13.0),
                color: theme::TEXT_2,
                ..Default::default()
            },
        );
        card_header(ui, BoxIcon::Letter(letter), title, Some(badge), job);

        ui.label(
            RichText::new("Clave de API")
                .size(12.0)
                .color(theme::TEXT_2),
        );
        let changing = self.changing == Some(provider);
        let width = ui.available_width();
        match (&saved, changing) {
            (Some(masked), false) => {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let button = outline_button("Cambiar", 32.0);
                    let field =
                        width - 8.0 - button_width(ui, "Cambiar", FontId::proportional(12.5));
                    masked_field(ui, masked, field, &format!("Clave de API de {title}"));
                    if ui
                        .add(button)
                        .accessible_name(format!("Cambiar la clave de {title}"))
                        .clicked()
                    {
                        self.changing = Some(provider);
                    }
                });
            }
            _ => {
                let buffer = match provider {
                    ProviderKind::Anthropic => &mut self.anthropic_key,
                    _ => &mut self.openai_key,
                };
                let mut save = false;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let field = width - 8.0 - button_width(ui, "Guardar", theme::semibold(12.5));
                    let response = ui.add(
                        egui::TextEdit::singleline(buffer)
                            .password(true)
                            .hint_text(
                                RichText::new("Pega tu clave")
                                    .font(theme::mono(12.5))
                                    .color(theme::TEXT_4),
                            )
                            .font(theme::mono(12.5))
                            .margin(vec2(10.0, 8.0))
                            .background_color(theme::BAR_ALT)
                            .margin(vec2(10.0, 8.0))
                            .desired_width(field),
                    );
                    let response = response.accessible_name(format!("Clave de API de {title}"));
                    let enter =
                        response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let clicked = ui
                        .add_enabled(!buffer.trim().is_empty(), accent_button("Guardar"))
                        .accessible_name(format!("Guardar la clave de {title}"))
                        .clicked();
                    save = (clicked || enter) && !buffer.trim().is_empty();
                });
                if save {
                    let key = Secret::new(std::mem::take(buffer).trim());
                    out.push(Intent::SaveApiKey { provider, key });
                    self.changing = None;
                }
                if changing {
                    ui.horizontal(|ui| {
                        if ui.add(text_button("Cancelar")).clicked() {
                            buffer.clear();
                            self.changing = None;
                        }
                        if ui
                            .add(text_button("Quitar la clave"))
                            .accessible_name(format!("Quitar la clave de {title}"))
                            .clicked()
                        {
                            buffer.clear();
                            self.changing = None;
                            out.push(Intent::DeleteApiKey(provider));
                        }
                    });
                }
            }
        }

        let model = match provider {
            ProviderKind::Anthropic => &mut next.anthropic_model,
            _ => &mut next.openai_model,
        };
        // Sin clave no se elige modelo; «Probar» llega con el agente.
        model_row(
            ui,
            &format!("Modelo de {title}"),
            model,
            models,
            saved.is_none().then_some("Guarda la clave para elegir"),
            saved.is_some(),
        );
        if saved.is_some()
            && !default
            && ui
                .add(text_button("Usar por defecto"))
                .accessible_name(format!("Usar {title} por defecto"))
                .clicked()
        {
            next.provider = provider;
        }
    }
}

fn local_card(ui: &mut egui::Ui, next: &Settings) {
    let mut job = LayoutJob::default();
    job.append(
        "Qwen y otros modelos en tu equipo, con cualquier servidor compatible con la API de OpenAI: Ollama, LM Studio o llama.cpp.",
        0.0,
        egui::TextFormat {
            font_id: FontId::proportional(13.0),
            color: theme::TEXT_2,
            ..Default::default()
        },
    );
    card_header(
        ui,
        BoxIcon::Server,
        "Modelos locales",
        Some(Badge::Soon),
        job,
    );
    ui.label(
        RichText::new("Dirección del servidor")
            .size(12.0)
            .color(theme::TEXT_3),
    );
    let mut url = next.local_url.clone();
    ui.add_enabled(
        false,
        egui::TextEdit::singleline(&mut url)
            .font(theme::mono(12.5))
            .background_color(theme::BAR_ALT)
            .margin(vec2(10.0, 8.0))
            .desired_width(ui.available_width()),
    )
    .accessible_name("Dirección del servidor local");
}

fn loop_card(ui: &mut egui::Ui, next: &mut Settings) {
    ui.spacing_mut().item_spacing.y = 12.0;
    ui.label(
        RichText::new("Bucle de refinado")
            .font(theme::semibold(14.5))
            .color(theme::TEXT),
    );
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Vueltas máximas")
                .size(13.0)
                .color(theme::TEXT),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            stepper(ui, &mut next.max_turns);
        });
    });
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        ui.label(RichText::new("Contexto de cada vuelta").size(13.0).color(theme::TEXT));
        segmented(
            ui,
            &mut next.context,
            &[(ContextPolicy::Fresh, "Vacío (recomendado)"), (ContextPolicy::Resume, "Misma conversación")],
        );
        let explain = match next.context {
            ContextPolicy::Fresh => "Cada vuelta empieza limpia y se orienta con el lienzo y las notas del agente.",
            ContextPolicy::Resume => {
                "Cada vuelta sigue la conversación anterior: recuerda más, pero gasta más tokens y arrastra los errores."
            }
        };
        ui.add(egui::Label::new(RichText::new(explain).size(12.0).color(theme::TEXT_3)).wrap());
    });
    switch_row(
        ui,
        "Parar si dos vueltas no avanzan",
        None,
        &mut next.stop_without_progress,
    );
}

fn review_card(ui: &mut egui::Ui, next: &mut Settings) {
    ui.spacing_mut().item_spacing.y = 12.0;
    ui.label(
        RichText::new("Revisión y vista")
            .font(theme::semibold(14.5))
            .color(theme::TEXT),
    );
    switch_row(
        ui,
        "Revisión crítica",
        Some(
            "Otro modelo revisa cada vuelta; el bucle no termina mientras encuentre problemas graves. Gasta más tokens.",
        ),
        &mut next.critic,
    );
    let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(line, 0.0, theme::LINE);
    switch_row(
        ui,
        "Mostrar cada comando en el lienzo",
        Some("Resalta los píxeles del último comando mientras el agente dibuja."),
        &mut next.show_commands,
    );
}

// ---------------------------------------------------------------------------
// Piezas
// ---------------------------------------------------------------------------

fn page_title(ui: &mut egui::Ui, title: &str, text: &str) {
    ui.spacing_mut().item_spacing.y = 6.0;
    ui.label(
        RichText::new(title)
            .font(theme::semibold(22.0))
            .color(theme::TEXT),
    );
    ui.scope(|ui| {
        ui.set_max_width(880.0);
        ui.add(egui::Label::new(RichText::new(text).size(14.0).color(theme::TEXT_2)).wrap());
    });
}

fn nav_item(ui: &mut egui::Ui, text: &str, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, selected, text));
    let bg = if selected {
        theme::HOVER
    } else if response.hovered() {
        theme::FIELD
    } else {
        Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(theme::RADIUS_BUTTON), bg);
    let color = if selected || response.hovered() {
        theme::TEXT
    } else {
        theme::TEXT_2
    };
    let font = if selected {
        theme::semibold(13.0)
    } else {
        FontId::proportional(13.0)
    };
    painter.text(
        pos2(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        font,
        color,
    );
    response
}

#[derive(Clone, Copy)]
enum CardStyle {
    /// Proveedor por defecto: borde acento de 1,5.
    Selected,
    /// Proveedor: fondo de tarjeta y borde.
    Normal,
    /// Próximamente: borde discontinuo y sin fondo.
    Dashed,
    /// Sección de ajustes: solo borde suave.
    Section,
}

fn card_style(selected: bool) -> CardStyle {
    if selected {
        CardStyle::Selected
    } else {
        CardStyle::Normal
    }
}

/// Dos tarjetas de ancho `width` en `xs`, a la altura `y`, con la misma altura
/// (la de la más alta). Devuelve esa altura.
fn card_row(
    ui: &mut egui::Ui,
    xs: [f32; 2],
    y: f32,
    width: f32,
    styles: [CardStyle; 2],
    mut content: impl FnMut(&mut egui::Ui, usize),
) -> f32 {
    let backgrounds = [ui.painter().add(Shape::Noop), ui.painter().add(Shape::Noop)];
    let mut height: f32 = 0.0;
    for (i, x) in xs.into_iter().enumerate() {
        let margin = match styles[i] {
            CardStyle::Section => Margin::symmetric(18, 16),
            _ => Margin::same(18),
        };
        let rect = Rect::from_min_size(pos2(x, y), vec2(width, 10_000.0));
        let used = ui
            .scope_builder(
                UiBuilder::new()
                    .max_rect(rect)
                    .layout(Layout::top_down(Align::Min)),
                |ui| {
                    egui::Frame::NONE.inner_margin(margin).show(ui, |ui| {
                        ui.set_width(width - margin.sum().x);
                        ui.spacing_mut().item_spacing.y = 14.0;
                        content(ui, i);
                    })
                },
            )
            .response
            .rect;
        height = height.max(used.height());
    }
    for (i, x) in xs.into_iter().enumerate() {
        let rect = Rect::from_min_size(pos2(x, y), vec2(width, height));
        let radius = CornerRadius::same(theme::RADIUS_CARD);
        let shape = match styles[i] {
            CardStyle::Selected => Shape::Rect(egui::epaint::RectShape::new(
                rect,
                radius,
                theme::CARD,
                Stroke::new(1.5, theme::ACCENT),
                StrokeKind::Inside,
            )),
            CardStyle::Normal => Shape::Rect(egui::epaint::RectShape::new(
                rect,
                radius,
                theme::CARD,
                Stroke::new(1.0, theme::LINE_STRONG),
                StrokeKind::Inside,
            )),
            CardStyle::Section => Shape::Rect(egui::epaint::RectShape::stroke(
                rect,
                radius,
                Stroke::new(1.0, theme::LINE),
                StrokeKind::Inside,
            )),
            CardStyle::Dashed => {
                let points = rounded_rect_path(rect.shrink(0.5), f32::from(theme::RADIUS_CARD));
                Shape::Vec(Shape::dashed_line(
                    &points,
                    Stroke::new(1.0, theme::LINE_STRONG),
                    4.0,
                    3.0,
                ))
            }
        };
        ui.painter().set(backgrounds[i], shape);
    }
    height
}

/// Contorno cerrado de un rectángulo redondeado (para el borde discontinuo).
fn rounded_rect_path(rect: Rect, r: f32) -> Vec<Pos2> {
    let mut points = Vec::new();
    let corners = [
        (pos2(rect.right() - r, rect.top() + r), -90.0_f32),
        (pos2(rect.right() - r, rect.bottom() - r), 0.0),
        (pos2(rect.left() + r, rect.bottom() - r), 90.0),
        (pos2(rect.left() + r, rect.top() + r), 180.0),
    ];
    for (center, start) in corners {
        for step in 0..=6 {
            let a = (start + 15.0 * step as f32).to_radians();
            points.push(center + vec2(a.cos(), a.sin()) * r);
        }
    }
    points.push(points[0]);
    points
}

#[derive(Clone, Copy)]
enum BoxIcon {
    Terminal,
    Server,
    Letter(&'static str),
}

#[derive(Clone, Copy)]
enum Badge {
    Default,
    Saved,
    Unset,
    Soon,
}

impl Badge {
    fn look(self) -> (&'static str, Color32, Color32) {
        match self {
            Badge::Default => ("Por defecto", theme::ACCENT_SOFT, theme::ACCENT),
            Badge::Saved => (
                "Clave guardada",
                Color32::from_rgba_unmultiplied(0x5B, 0xD6, 0x9A, 31),
                theme::OK,
            ),
            Badge::Unset => ("Sin configurar", theme::HOVER, theme::TEXT_2),
            Badge::Soon => (
                "Próximamente",
                Color32::from_rgba_unmultiplied(0x7A, 0xB8, 0xFF, 31),
                theme::INFO,
            ),
        }
    }
}

/// Icono, título con su etiqueta y descripción de una tarjeta de proveedor.
fn card_header(
    ui: &mut egui::Ui,
    icon: BoxIcon,
    title: &str,
    badge: Option<Badge>,
    desc: LayoutJob,
) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        let (rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
        let soon = matches!(icon, BoxIcon::Server);
        let painter = ui.painter();
        painter.rect_filled(
            rect,
            CornerRadius::same(theme::RADIUS_BUTTON),
            if soon { theme::FIELD } else { theme::HOVER },
        );
        let color = if soon { theme::TEXT_2 } else { theme::TEXT };
        let stroke = Stroke::new(1.6, color);
        let c = rect.center();
        match icon {
            BoxIcon::Letter(letter) => {
                painter.text(
                    c,
                    egui::Align2::CENTER_CENTER,
                    letter,
                    theme::semibold(13.0),
                    color,
                );
            }
            BoxIcon::Terminal => {
                // Ventana con «>_».
                let r = Rect::from_center_size(c, vec2(13.5, 10.5));
                painter.rect_stroke(r, CornerRadius::same(2), stroke, StrokeKind::Middle);
                painter.line(
                    vec![
                        c + vec2(-3.75, -1.5),
                        c + vec2(-1.5, 0.0),
                        c + vec2(-3.75, 1.5),
                    ],
                    stroke,
                );
                painter.line_segment([c + vec2(0.0, 2.25), c + vec2(3.75, 2.25)], stroke);
            }
            BoxIcon::Server => {
                for dy in [-2.6, 2.6] {
                    let r = Rect::from_center_size(c + vec2(0.0, dy), vec2(12.0, 5.25));
                    painter.rect_stroke(r, CornerRadius::same(1), stroke, StrokeKind::Middle);
                    painter.circle_filled(c + vec2(-3.0, dy), 0.8, color);
                }
            }
        }
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 3.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let title_color = if soon { theme::TEXT_BODY } else { theme::TEXT };
                ui.label(
                    RichText::new(title)
                        .font(theme::semibold(14.5))
                        .color(title_color),
                );
                if let Some(badge) = badge {
                    let (text, fill, color) = badge.look();
                    egui::Frame::NONE
                        .fill(fill)
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(7, 3))
                        .show(ui, |ui| {
                            ui.label(RichText::new(text).font(theme::semibold(11.0)).color(color));
                        });
                }
            });
            ui.add(egui::Label::new(desc).wrap());
        });
    });
}

/// «Modelo» + desplegable. Con `locked`, deshabilitado mostrando ese texto;
/// con `probe`, el botón «Probar» a la derecha.
fn model_row(
    ui: &mut egui::Ui,
    name: &str,
    model: &mut String,
    options: &[(&str, &str)],
    locked: Option<&str>,
    probe: bool,
) {
    let enabled = locked.is_none();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (label, _) = ui.allocate_exact_size(vec2(64.0, 32.0), Sense::hover());
        ui.painter().text(
            pos2(label.left(), label.center().y),
            egui::Align2::LEFT_CENTER,
            "Modelo",
            FontId::proportional(12.5),
            theme::TEXT_2,
        );
        let current = options
            .iter()
            .find(|(id, _)| *id == model.as_str())
            .map(|(_, label)| (*label).to_string())
            .unwrap_or_else(|| model.clone());
        let shown = locked.map(str::to_string).unwrap_or(current);
        let extra_width = if probe {
            button_width(ui, "Probar", FontId::proportional(12.5)) + 8.0
        } else {
            0.0
        };
        let width = ui.available_width() - extra_width;
        ui.add_enabled_ui(enabled, |ui| {
            ui.spacing_mut().interact_size.y = 32.0;
            let combo = egui::ComboBox::from_id_salt(name)
                .selected_text(RichText::new(shown).size(13.0))
                .width(width)
                .height(240.0)
                .show_ui(ui, |ui| {
                    for (id, label) in options {
                        ui.selectable_value(model, (*id).to_string(), *label);
                    }
                });
            let _ = combo.response.accessible_name(name);
        });
        if probe {
            ui.add_enabled(false, outline_button("Probar", 32.0))
                .on_disabled_hover_text("Llega con el agente");
        }
    });
}

fn switch_row(ui: &mut egui::Ui, title: &str, desc: Option<&str>, on: &mut bool) {
    ui.horizontal(|ui| {
        let width = ui.available_width() - 48.0;
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing.y = 3.0;
            ui.label(RichText::new(title).size(13.0).color(theme::TEXT));
            if let Some(desc) = desc {
                ui.add(
                    egui::Label::new(RichText::new(desc).size(12.5).color(theme::TEXT_2)).wrap(),
                );
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if switch(ui, *on, title).clicked() {
                *on = !*on;
            }
        });
    });
}

/// Interruptor de 36 × 20.
pub(crate) fn switch(ui: &mut egui::Ui, on: bool, label: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(36.0, 20.0), Sense::click());
    let enabled = ui.is_enabled();
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Switch, enabled, on, label));
    let (track, knob, x) = if on {
        (theme::ACCENT, theme::ACCENT_INK, rect.right() - 10.0)
    } else {
        (theme::LINE_STRONG, theme::TEXT_2, rect.left() + 10.0)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(10), track);
    painter.circle_filled(pos2(x, rect.center().y), 8.0, knob);
    response
}

/// − n + para las vueltas máximas.
fn stepper(ui: &mut egui::Ui, value: &mut u32) {
    egui::Frame::NONE
        .stroke(Stroke::new(1.0, theme::LINE_STRONG))
        .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            // De derecha a izquierda: «+», número, «−».
            if step_button(ui, "+", "Más vueltas", *value < *MAX_TURNS.end()).clicked() {
                *value += 1;
            }
            let (rect, response) = ui.allocate_exact_size(vec2(32.0, 28.0), Sense::hover());
            let shown = value.to_string();
            response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &shown));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                value.to_string(),
                theme::mono(12.5),
                theme::TEXT,
            );
            if step_button(ui, "−", "Menos vueltas", *value > *MAX_TURNS.start()).clicked() {
                *value -= 1;
            }
        });
}

fn step_button(ui: &mut egui::Ui, text: &str, name: &str, enabled: bool) -> egui::Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(28.0, 28.0), sense);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, name));
    let color = if !enabled {
        theme::TEXT_4
    } else if response.hovered() {
        theme::TEXT
    } else {
        theme::TEXT_2
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(15.0),
        color,
    );
    response
}

/// Control segmentado de ancho completo.
fn segmented<T: Copy + PartialEq>(ui: &mut egui::Ui, value: &mut T, options: &[(T, &str)]) {
    egui::Frame::NONE
        .fill(theme::BG)
        .stroke(Stroke::new(1.0, theme::LINE_SOFT))
        .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
        .inner_margin(Margin::same(2))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let width = (ui.available_width() / options.len() as f32).floor();
            ui.horizontal(|ui| {
                for (option, label) in options {
                    let selected = *value == *option;
                    let (rect, response) =
                        ui.allocate_exact_size(vec2(width, 28.0), Sense::click());
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(egui::Role::RadioButton, true, selected, *label)
                    });
                    if selected {
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(6), theme::HOVER);
                    }
                    let color = if selected || response.hovered() {
                        theme::TEXT
                    } else {
                        theme::TEXT_3
                    };
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        *label,
                        FontId::proportional(12.5),
                        color,
                    );
                    if response.clicked() {
                        *value = *option;
                    }
                }
            });
        });
}

/// Campo que muestra una clave guardada, enmascarada y sin editar.
fn masked_field(ui: &mut egui::Ui, masked: &str, width: f32, name: &str) {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 32.0), Sense::hover());
    // Para los lectores de pantalla: «Clave de API de …» con la clave
    // enmascarada como valor (de solo lectura).
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(egui::Role::TextInput, true, name);
        info.current_text_value = Some(masked.to_string());
        info
    });
    let painter = ui.painter();
    painter.rect(
        rect,
        CornerRadius::same(theme::RADIUS_BUTTON),
        theme::BAR_ALT,
        Stroke::new(1.0, theme::LINE_STRONG),
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.left() + 10.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        masked,
        theme::mono(12.5),
        theme::TEXT,
    );
}

fn outline_button(text: &str, height: f32) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(text.to_string())
            .size(12.5)
            .color(theme::TEXT),
    )
    .fill(Color32::TRANSPARENT)
    .stroke(Stroke::new(1.0, theme::LINE_STRONG))
    .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
    .min_size(vec2(0.0, height))
}

fn accent_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(text.to_string())
            .font(theme::semibold(12.5))
            .color(theme::ACCENT_INK),
    )
    .fill(theme::ACCENT)
    .stroke(Stroke::NONE)
    .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
    .min_size(vec2(0.0, 32.0))
}

fn text_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(text.to_string())
            .size(12.5)
            .color(theme::TEXT_2),
    )
    .fill(Color32::TRANSPARENT)
    .stroke(Stroke::NONE)
    .corner_radius(CornerRadius::same(6))
    .min_size(vec2(0.0, 28.0))
}

/// Ancho de un botón de texto con esta letra y su relleno.
fn button_width(ui: &egui::Ui, text: &str, font: FontId) -> f32 {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font, theme::TEXT);
    galley.size().x + 2.0 * ui.spacing().button_padding.x
}
