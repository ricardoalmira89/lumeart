//! Panel del agente (columna derecha del studio, `01-studio-pixel.png`):
//! proveedor y modelo, revisión crítica, la conversación (mensajes, bloques
//! de herramienta plegables con muestras de color, miniaturas de revisión y
//! el paso en curso) y la redacción con adjuntos, Detener y Enviar.
//!
//! Como el resto de la UI, solo emite [`Intent`]s: la ejecución la lleva el
//! [`AgentSupervisor`](crate::agent_run::AgentSupervisor), que le pasa los
//! eventos con [`AgentPanel::apply`].

use std::collections::HashMap;

use eframe::egui::{
    self, Align, Color32, CornerRadius, Layout, Margin, RichText, Sense, Stroke, StrokeKind,
    TextureHandle, pos2, vec2,
};
use lume_app::ports::{AgentEvent, Attachment, ContentPart, ToolOutput};
use lume_app::{ProviderKind, Settings};
use lume_domain::Rgba;
use lume_domain::batch::split_batch;
use serde_json::Value;

use crate::intent::Intent;
use crate::settings_screen::{ANTHROPIC_MODELS, CLAUDE_CODE_MODELS, OPENAI_MODELS};
use crate::theme;

/// Comandos que se ven de un bloque de herramienta sin desplegarlo.
const PREVIEW_LINES: usize = 3;
/// Alto de la redacción (sin adjuntos) al abrir: tres líneas y los botones.
pub const COMPOSER_HEIGHT: f32 = 136.0;
/// Lo que queda siempre para la conversación al agrandar la redacción.
const CONVERSATION_MIN: f32 = 80.0;
/// Alto de la fila de miniaturas de los adjuntos (dentro del cuadro).
const ATTACHMENTS_HEIGHT: f32 = 68.0;
/// Lado de la miniatura de un adjunto.
const ATTACHMENT_THUMB: f32 = 52.0;
/// Encargo cuando se envía solo una imagen.
pub const IMAGE_ONLY_PROMPT: &str = "Dibuja lo que se ve en la imagen adjunta.";
/// Caracteres como mucho del título y del resumen de un bloque (más, con
/// «…»): en una línea no caben más y ensancharían el panel.
const TITLE_CHARS: usize = 60;
const RESULT_CHARS: usize = 22;

/// El texto, cortado a `max` caracteres con «…» si es más largo.
pub fn ellipsize(text: &str, max: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", cut.trim_end())
    }
}

/// Lo que el panel necesita saber del resto de la ventana en cada fotograma.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentView {
    pub settings: Settings,
    /// Hay una ejecución en marcha.
    pub running: bool,
    /// Versión que se está viendo en el lienzo (si no es la actual).
    pub preview_version: Option<u32>,
    /// Modo animación: el agente cambia `anim.js` y se pueden pedir variantes.
    pub animation: bool,
    /// Modo vector: el agente cambia `dibujo.js` (y se le puede señalar una
    /// zona con el lazo).
    pub vector: bool,
    /// Animación de un dibujo vectorial: el agente cambia su esqueleto.
    pub vector_anim: bool,
}

/// Pestañas del panel en el modo pixel art: la conversación, el código que
/// va dibujando y el historial con miniaturas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelTab {
    #[default]
    Agent,
    Code,
    History,
}

impl PanelTab {
    pub const ALL: [PanelTab; 3] = [PanelTab::Agent, PanelTab::Code, PanelTab::History];

    pub fn label(self) -> &'static str {
        match self {
            PanelTab::Agent => "Conversación",
            PanelTab::Code => "Código",
            PanelTab::History => "Historial",
        }
    }
}

/// Dibuja el contenido de una pestaña que no es la conversación.
pub type TabContent<'a> = &'a mut dyn FnMut(&mut egui::Ui, PanelTab, &mut Vec<Intent>);

/// Estado de una vuelta en la caja del bucle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnState {
    Done,
    Current,
    Pending,
}

/// La caja «Bucle de refinado»: vuelta N de M, una casilla por vuelta (con
/// la miniatura de su versión), contexto y comprobaciones.
#[derive(Default, Clone)]
pub struct LoopBox {
    /// Vueltas como máximo (M).
    pub of: u32,
    /// Vuelta en curso (N), si hay bucle en marcha.
    pub current: Option<u32>,
    /// Vueltas terminadas: (vuelta, versión).
    pub done: Vec<(u32, u32)>,
    /// Comprobaciones de la última vuelta: (pasan, total).
    pub checks: Option<(usize, usize)>,
    /// Misma conversación (si no, contexto vacío).
    pub resume: bool,
    /// Lienzo de cada vuelta terminada, para su miniatura.
    /// Imagen de cada vuelta terminada (el lienzo o el dibujo vectorial).
    canvases: HashMap<u32, std::sync::Arc<egui::ColorImage>>,
    thumbnails: HashMap<u32, TextureHandle>,
}

impl LoopBox {
    pub fn state(&self, turn: u32) -> TurnState {
        if self.done.iter().any(|(t, _)| *t == turn) {
            TurnState::Done
        } else if self.current == Some(turn) {
            TurnState::Current
        } else {
            TurnState::Pending
        }
    }

    /// Nombre accesible de una casilla («Vuelta 2: en curso»).
    pub fn label(&self, turn: u32) -> String {
        let state = match self.state(turn) {
            TurnState::Done => "hecha",
            TurnState::Current => "en curso",
            TurnState::Pending => "pendiente",
        };
        format!("Vuelta {turn}: {state}")
    }
}

/// Una llamada a herramienta en la conversación.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolItem {
    /// Herramienta (`draw`, `Bash`, `Read`…).
    pub name: String,
    /// Lo que se muestra: `draw`, `lumeart batch`, `Read revision.png`…
    pub title: String,
    /// Comandos de dibujo, si los hay.
    pub commands: Vec<String>,
    /// Resumen del resultado («48 ok», «2 ok · 1 falla», «ok #ff0000ff»…).
    pub result: Option<String>,
    pub is_error: bool,
    /// Desplegado (se ven todos los comandos, la orden entera y la salida).
    pub open: bool,
    /// La orden entera, si el título la recorta (un `Bash` largo).
    pub full: Option<String>,
    /// Lo que devolvió la herramienta (el error entero, si falló).
    pub output: Option<String>,
}

/// Un elemento de la conversación.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvItem {
    /// Lo que pidió el usuario y sus adjuntos.
    User {
        text: String,
        attachments: Vec<String>,
    },
    /// Texto del agente (plan, notas, resumen), en markdown.
    Text(String),
    /// Lo que pensó el agente (plegado: se ven las primeras líneas).
    Thinking {
        text: String,
        open: bool,
    },
    Tool(ToolItem),
    /// Imagen que miró el agente al revisar (PNG).
    Review {
        png: Vec<u8>,
    },
    /// Cómo terminó la ejecución.
    End {
        text: String,
        error: bool,
    },
    /// Empieza una vuelta del bucle («Empieza la vuelta 2 de 8»).
    Turn(String),
    /// Lo que dijo el crítico de una vuelta.
    Critic {
        text: String,
        ok: bool,
    },
}

/// Estado del panel: la conversación y la redacción.
#[derive(Default, Clone)]
pub struct AgentPanel {
    pub items: Vec<ConvItem>,
    /// Texto que se está escribiendo.
    pub draft: String,
    /// Imágenes de referencia para el próximo envío.
    pub attachments: Vec<Attachment>,
    /// «Seguir con este dibujo»: el agente trabaja sobre lo que hay.
    pub continue_drawing: bool,
    /// Pasos de la ejecución en marcha (llamadas al modelo vistas).
    pub steps: u32,
    /// Último comando de la ejecución en marcha, para el «paso en curso».
    pub last_command: Option<String>,
    /// Lo último que pensó el agente, si es lo más reciente que hizo (se ve
    /// en «Pensando…»).
    pub thought: Option<String>,
    /// Miniaturas de los adjuntos (por etiqueta y tamaño).
    attachment_textures: HashMap<(String, usize), Option<TextureHandle>>,
    /// Tokens gastados en esta sesión (entrada + salida).
    pub tokens: u64,
    /// La caja del bucle de la última ejecución.
    pub loop_box: Option<LoopBox>,
    /// Alto de la redacción sin adjuntos (se cambia arrastrando la línea de
    /// encima; 0 es el de siempre, [`COMPOSER_HEIGHT`]).
    pub composer_height: f32,
    /// Pestaña abierta (solo hay pestañas en pixel art).
    pub tab: PanelTab,
    textures: HashMap<usize, Option<TextureHandle>>,
}

impl std::fmt::Debug for AgentPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentPanel")
            .field("items", &self.items.len())
            .field("steps", &self.steps)
            .field("tokens", &self.tokens)
            .finish_non_exhaustive()
    }
}

impl AgentPanel {
    /// Un mensaje para la próxima vuelta del bucle en marcha.
    pub fn message_for_next_turn(&mut self, text: &str) {
        self.items.push(ConvItem::User {
            text: text.to_string(),
            attachments: vec!["para la próxima vuelta".into()],
        });
    }

    /// Empieza una ejecución: apunta el mensaje del usuario.
    pub fn started(&mut self, prompt: &str, attachments: &[Attachment]) {
        self.loop_box = None;
        self.items.push(ConvItem::User {
            text: prompt.to_string(),
            attachments: attachments.iter().map(|a| a.label.clone()).collect(),
        });
        self.steps = 0;
        self.last_command = None;
    }

    /// Añade lo que pasó en el bucle. `canvas`: el lienzo cuando terminó la
    /// vuelta (para la miniatura de su casilla).
    pub fn apply_loop(
        &mut self,
        event: &lume_app::LoopEvent,
        canvas: Option<std::sync::Arc<egui::ColorImage>>,
        resume: bool,
    ) {
        use lume_app::LoopEvent;
        match event {
            LoopEvent::TurnStarted { turn, of } => {
                let lb = self.loop_box.get_or_insert_with(LoopBox::default);
                lb.of = *of;
                lb.current = Some(*turn);
                lb.resume = resume;
                self.steps = 0;
                self.last_command = None;
                self.items
                    .push(ConvItem::Turn(format!("Empieza la vuelta {turn} de {of}")));
            }
            LoopEvent::Agent(event) => self.apply(event),
            LoopEvent::Critic { verdict, .. } => {
                let (text, ok) = match verdict {
                    lume_app::critic::Verdict::Ready(c) => {
                        let serious = c.serious();
                        if serious.is_empty() {
                            ("Crítico: sin problemas graves.".to_string(), true)
                        } else {
                            let list: Vec<&str> =
                                serious.iter().map(|i| i.problem.as_str()).collect();
                            (format!("Crítico: {}.", list.join("; ")), false)
                        }
                    }
                    lume_app::critic::Verdict::None { reason } => {
                        (format!("Crítico sin veredicto: {reason}."), true)
                    }
                };
                self.items.push(ConvItem::Critic { text, ok });
            }
            LoopEvent::TurnFinished {
                turn,
                version,
                checks,
                ..
            } => {
                let lb = self.loop_box.get_or_insert_with(LoopBox::default);
                lb.done.push((*turn, *version));
                lb.checks = Some((checks.iter().filter(|c| c.passed).count(), checks.len()));
                if let Some(canvas) = canvas {
                    lb.canvases.insert(*turn, canvas);
                    lb.thumbnails.remove(turn);
                }
            }
        }
    }

    /// Termina la ejecución en marcha con este texto.
    pub fn finished(&mut self, text: impl Into<String>, error: bool) {
        if let Some(lb) = &mut self.loop_box {
            lb.current = None;
        }
        self.items.push(ConvItem::End {
            text: text.into(),
            error,
        });
        self.last_command = None;
    }

    /// Añade a la conversación lo que hizo el agente.
    pub fn apply(&mut self, event: &AgentEvent) {
        match event {
            AgentEvent::Text(text) => {
                if !text.trim().is_empty() {
                    self.thought = None;
                    self.items.push(ConvItem::Text(text.trim().to_string()));
                }
            }
            AgentEvent::Thinking(text) => {
                if !text.trim().is_empty() {
                    self.thought = Some(text.trim().to_string());
                    self.items.push(ConvItem::Thinking {
                        text: text.trim().to_string(),
                        open: false,
                    });
                }
            }
            AgentEvent::ToolCall { name, input_json } => {
                self.thought = None;
                let item = tool_item(name, input_json);
                self.last_command = item
                    .commands
                    .last()
                    .cloned()
                    .or_else(|| Some(item.title.clone()));
                self.items.push(ConvItem::Tool(item));
            }
            AgentEvent::ToolResult { name, output } => {
                let pending = self.items.iter_mut().rev().find_map(|item| match item {
                    ConvItem::Tool(tool) if tool.name == *name && tool.result.is_none() => {
                        Some(tool)
                    }
                    _ => None,
                });
                if let Some(tool) = pending {
                    tool.result = Some(result_summary(output));
                    tool.is_error = output.is_error;
                    let text = output_text(output);
                    tool.output = (!text.trim().is_empty()).then_some(text);
                    // Claude Code rechaza lo que no sea `lumeart` en la
                    // terminal: se explica en vez de dejar su aviso en inglés.
                    if let Some(original) = tool.output.clone().filter(|t| blocked(t)) {
                        tool.result = Some("bloqueado".into());
                        tool.output = Some(format!(
                            "Claude Code no deja ejecutar esto: en la terminal el agente solo puede usar lumeart (es una protección de LumeArt, nadie tiene que aprobar nada). El agente sigue y lo hace de otra forma.\n\n{original}"
                        ));
                    }
                }
                for part in &output.content {
                    if let ContentPart::Image { data, .. } = part {
                        self.items.push(ConvItem::Review { png: data.clone() });
                    }
                }
            }
            AgentEvent::Usage(usage) => {
                self.steps += 1;
                self.tokens += usage.total();
            }
        }
    }

    /// Dibuja el panel en `ui` (la columna entera). Con `tabs`, debajo de
    /// la caja del bucle van las pestañas Conversación, Código e Historial, y
    /// `tabs` dibuja las dos últimas.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        view: &AgentView,
        out: &mut Vec<Intent>,
        tabs: Option<TabContent<'_>>,
    ) {
        let full = ui.max_rect();
        self.header(ui, view, out);
        ui.add_space(12.0);
        if self.loop_box.is_some() {
            self.loop_box_ui(ui, view, out);
            ui.add_space(10.0);
        }
        if tabs.is_some() {
            self.tab_bar(ui);
            ui.add_space(10.0);
        } else {
            self.tab = PanelTab::Agent;
        }
        let attachments = if self.attachments.is_empty() {
            0.0
        } else {
            ATTACHMENTS_HEIGHT
        };
        // La redacción crece arrastrando la línea que la separa de la
        // conversación, sin dejar a esta sin sitio.
        let top = ui.cursor().top();
        let tallest = (full.bottom() - top - attachments - CONVERSATION_MIN).max(COMPOSER_HEIGHT);
        let body = self.composer_height.clamp(COMPOSER_HEIGHT, tallest);
        let composer_height = body + attachments;
        let conversation_height = (full.bottom() - top - composer_height).max(60.0);
        match (self.tab, tabs) {
            (PanelTab::Agent, _) | (_, None) => {
                egui::ScrollArea::vertical()
                    .id_salt("conversacion-agente")
                    .max_height(conversation_height)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        self.conversation(ui, view);
                    });
            }
            (tab, Some(content)) => {
                let rect = egui::Rect::from_min_size(
                    pos2(full.left(), top),
                    vec2(full.width(), conversation_height),
                );
                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                    ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                    content(ui, tab, out);
                });
            }
        }
        let composer_top = full.bottom() - composer_height;
        let handle = egui::Rect::from_min_max(
            pos2(full.left() - 16.0, composer_top - 4.0),
            pos2(full.right() + 16.0, composer_top + 4.0),
        );
        let drag = ui.interact(
            handle,
            ui.make_persistent_id("asa-redaccion"),
            Sense::click_and_drag(),
        );
        drag.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Splitter, true, "Alto del mensaje")
        });
        let drag = drag
            .on_hover_cursor(egui::CursorIcon::ResizeVertical)
            .on_hover_text("Arrastra para cambiar el alto; doble clic para volver al de siempre");
        if drag.double_clicked() {
            self.composer_height = COMPOSER_HEIGHT;
        } else if drag.dragged() && ui.ctx().current_pass_index() == 0 {
            // Solo en la primera pasada: si egui repite el fotograma con la
            // misma entrada, el arrastre no cuenta dos veces.
            self.composer_height = body - drag.drag_delta().y;
        }
        let line = if drag.hovered() || drag.dragged() {
            theme::LINE_STRONG
        } else {
            theme::LINE
        };
        ui.painter().hline(
            (full.left() - 16.0)..=(full.right() + 16.0),
            composer_top - 0.5,
            Stroke::new(if drag.dragged() { 2.0 } else { 1.0 }, line),
        );
        let rect = egui::Rect::from_min_max(pos2(full.left(), composer_top + 16.0), full.max);
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            self.composer(ui, view, out);
        });
    }

    /// Pestañas Conversación · Código · Historial (como el selector de modo).
    fn tab_bar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(theme::BG)
            .stroke(Stroke::new(1.0, theme::LINE_SOFT))
            .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
            .inner_margin(Margin::same(2))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let width = (ui.available_width() / PanelTab::ALL.len() as f32).floor();
                ui.horizontal(|ui| {
                    for tab in PanelTab::ALL {
                        let selected = self.tab == tab;
                        let color = if selected { theme::TEXT } else { theme::TEXT_3 };
                        let button =
                            egui::Button::new(RichText::new(tab.label()).size(12.5).color(color))
                                .fill(if selected {
                                    theme::HOVER
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .stroke(Stroke::NONE)
                                .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
                                .min_size(vec2(width, 26.0));
                        let response = ui.add(button);
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(egui::Role::Tab, true, selected, tab.label())
                        });
                        if response.clicked() {
                            self.tab = tab;
                        }
                    }
                });
            });
    }

    fn header(&mut self, ui: &mut egui::Ui, view: &AgentView, out: &mut Vec<Intent>) {
        let settings = &view.settings;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("AGENTE")
                    .font(theme::semibold(11.0))
                    .color(theme::TEXT_3)
                    .extra_letter_spacing(0.66),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if crate::settings_screen::switch(ui, settings.critic, "Revisión crítica").clicked()
                {
                    out.push(Intent::UpdateSettings(Settings {
                        critic: !settings.critic,
                        ..settings.clone()
                    }));
                }
                ui.label(
                    RichText::new("Revisión crítica")
                        .size(12.5)
                        .color(theme::TEXT_2),
                );
            });
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.spacing_mut().interact_size.y = 32.0;
            let width = ui.available_width();
            let mut provider = settings.provider;
            let combo = egui::ComboBox::from_id_salt("proveedor-agente")
                .width(width * 0.62 - 8.0)
                .selected_text(
                    RichText::new(provider_name(provider))
                        .size(13.0)
                        .color(theme::TEXT),
                )
                .show_ui(ui, |ui| {
                    for p in ProviderKind::ALL {
                        let enabled = p != ProviderKind::Local;
                        ui.add_enabled_ui(enabled, |ui| {
                            ui.selectable_value(&mut provider, p, provider_name(p))
                                .on_disabled_hover_text("Próximamente");
                        });
                    }
                });
            let _ = combo.response.accessible_name("Proveedor del agente");
            if provider != settings.provider {
                out.push(Intent::UpdateSettings(Settings {
                    provider,
                    ..settings.clone()
                }));
            }
            let (models, current) = match settings.provider {
                ProviderKind::ClaudeCode => (CLAUDE_CODE_MODELS, &settings.claude_code_model),
                ProviderKind::Anthropic => (ANTHROPIC_MODELS, &settings.anthropic_model),
                ProviderKind::OpenAi => (OPENAI_MODELS, &settings.openai_model),
                ProviderKind::Local => (&[][..], &settings.openai_model),
            };
            let shown = models
                .iter()
                .find(|(id, _)| *id == current.as_str())
                .map(|(_, label)| label.to_string())
                .unwrap_or_else(|| current.clone());
            let mut chosen = current.clone();
            let combo = egui::ComboBox::from_id_salt("modelo-agente")
                .width(ui.available_width())
                .selected_text(RichText::new(shown).size(13.0).color(theme::TEXT))
                .show_ui(ui, |ui| {
                    for (id, label) in models {
                        ui.selectable_value(&mut chosen, (*id).to_string(), *label);
                    }
                });
            let _ = combo.response.accessible_name("Modelo del agente");
            if chosen != *current {
                let mut next = settings.clone();
                match settings.provider {
                    ProviderKind::ClaudeCode => next.claude_code_model = chosen,
                    ProviderKind::Anthropic => next.anthropic_model = chosen,
                    ProviderKind::OpenAi => next.openai_model = chosen,
                    ProviderKind::Local => {}
                }
                out.push(Intent::UpdateSettings(next));
            }
        });
    }

    fn conversation(&mut self, ui: &mut egui::Ui, view: &AgentView) {
        // Nada puede ensanchar el panel.
        ui.set_max_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 12.0;
        if self.items.is_empty() {
            ui.add_space(8.0);
            let text = if view.vector_anim {
                "Pide la animación que quieras («que la tapa se abra y se quede abierta») y el agente le pondrá esqueleto al dibujo y la acción."
            } else if view.animation {
                "Describe cómo quieres que se mueva y el agente cambiará la animación, o pide variantes para elegir."
            } else if view.vector {
                "Pide un dibujo o un cambio y el agente escribirá dibujo.js. Para retocar solo una parte, rodéala con el lazo."
            } else {
                "Describe qué quieres dibujar y el agente lo hará en el lienzo, paso a paso."
            };
            ui.add(egui::Label::new(RichText::new(text).size(13.0).color(theme::TEXT_3)).wrap());
        }
        for i in 0..self.items.len() {
            match self.items[i].clone() {
                ConvItem::User { text, attachments } => user_bubble(ui, &text, &attachments),
                ConvItem::Text(text) => agent_line(ui, |ui| {
                    crate::markdown::show(
                        ui,
                        &text,
                        crate::markdown::Style {
                            size: 13.0,
                            color: theme::TEXT_BODY,
                            italic: false,
                        },
                    );
                }),
                ConvItem::Thinking { text, open } => {
                    let toggled = agent_line(ui, |ui| thinking_block(ui, i, &text, open));
                    if toggled && let ConvItem::Thinking { open, .. } = &mut self.items[i] {
                        *open = !*open;
                    }
                }
                ConvItem::Tool(tool) => {
                    let toggled = agent_line(ui, |ui| tool_block(ui, i, &tool));
                    if toggled && let ConvItem::Tool(t) = &mut self.items[i] {
                        t.open = !t.open;
                    }
                }
                ConvItem::Review { png } => {
                    let texture = self
                        .textures
                        .entry(i)
                        .or_insert_with(|| review_texture(ui.ctx(), i, &png))
                        .clone();
                    agent_line(ui, |ui| review(ui, texture.as_ref()));
                }
                ConvItem::End { text, error } => agent_line(ui, |ui| {
                    let color = if error { theme::DANGER } else { theme::TEXT_3 };
                    ui.add(egui::Label::new(RichText::new(text).size(12.0).color(color)).wrap());
                }),
                ConvItem::Turn(text) => {
                    ui.label(
                        RichText::new(text)
                            .font(theme::semibold(11.5))
                            .color(theme::TEXT_3),
                    );
                }
                ConvItem::Critic { text, ok } => agent_line(ui, |ui| {
                    let color = if ok { theme::TEXT_2 } else { theme::WARN };
                    ui.add(egui::Label::new(RichText::new(text).size(12.5).color(color)).wrap());
                }),
            }
        }
        if view.running {
            self.progress(ui);
        }
    }

    /// La caja «Bucle de refinado».
    fn loop_box_ui(&mut self, ui: &mut egui::Ui, view: &AgentView, out: &mut Vec<Intent>) {
        let Some(lb) = self.loop_box.as_mut() else {
            return;
        };
        egui::Frame::NONE
            .fill(theme::LOOP_BOX)
            .stroke(Stroke::new(1.0, theme::LINE_SOFT))
            .corner_radius(CornerRadius::same(theme::RADIUS_CARD_SMALL))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Bucle de refinado")
                            .font(theme::semibold(13.0))
                            .color(theme::TEXT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let shown = lb.current.or(lb.done.last().map(|(t, _)| *t)).unwrap_or(0);
                        ui.label(
                            RichText::new(format!("vuelta {shown} de {}", lb.of))
                                .font(theme::mono(12.0))
                                .color(theme::ACCENT),
                        );
                    });
                });
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(5.0, 5.0);
                    for turn in 1..=lb.of {
                        let version = lb.done.iter().find(|(t, _)| *t == turn).map(|(_, v)| *v);
                        if let (Some(canvas), false) =
                            (lb.canvases.get(&turn), lb.thumbnails.contains_key(&turn))
                        {
                            let texture = ui.ctx().load_texture(
                                format!("vuelta-{turn}"),
                                (**canvas).clone(),
                                egui::TextureOptions::LINEAR,
                            );
                            lb.thumbnails.insert(turn, texture);
                        }
                        let state = lb.state(turn);
                        let selected = version.is_some() && version == view.preview_version;
                        let response = turn_square(
                            ui,
                            state,
                            lb.thumbnails.get(&turn),
                            selected,
                            &lb.label(turn),
                        );
                        if let (true, Some(version)) = (response.clicked(), version) {
                            out.push(Intent::ShowVersion(version));
                        }
                    }
                });
                ui.horizontal(|ui| {
                    let context = if lb.resume {
                        "Misma conversación"
                    } else {
                        "Contexto vacío en cada vuelta"
                    };
                    ui.label(RichText::new(context).size(12.0).color(theme::TEXT_3));
                    if let Some((passed, total)) = lb.checks {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("Comprobaciones: {passed} de {total}"))
                                    .size(12.0)
                                    .color(theme::TEXT_3),
                            );
                        });
                    }
                });
            });
    }

    /// «Paso en curso»: caja acento con el último comando.
    fn progress(&self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(theme::ACCENT_SOFT)
            .stroke(Stroke::new(1.0, theme::ACCENT_RING))
            .corner_radius(CornerRadius::same(theme::RADIUS_CARD_SMALL))
            .inner_margin(Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new().size(14.0).color(theme::ACCENT));
                    let title = if self.steps == 0 {
                        "Pensando…".to_string()
                    } else {
                        format!("Paso {}", self.steps)
                    };
                    ui.label(
                        RichText::new(title)
                            .font(theme::semibold(13.0))
                            .color(theme::TEXT),
                    );
                });
                if let Some(thought) = &self.thought {
                    // Las últimas líneas de lo que está pensando.
                    let tail: Vec<&str> =
                        thought.lines().filter(|l| !l.trim().is_empty()).collect();
                    let tail = tail[tail.len().saturating_sub(4)..].join("\n");
                    ui.add(
                        egui::Label::new(
                            RichText::new(ellipsize(&tail, 420))
                                .size(12.0)
                                .italics()
                                .color(theme::TEXT_2),
                        )
                        .wrap(),
                    );
                } else if let Some(command) = &self.last_command {
                    ui.add(
                        egui::Label::new(
                            RichText::new(command)
                                .font(theme::mono(12.0))
                                .color(theme::TEXT_2),
                        )
                        .truncate(),
                    );
                }
            });
    }

    fn composer(&mut self, ui: &mut egui::Ui, view: &AgentView, out: &mut Vec<Intent>) {
        ui.spacing_mut().item_spacing.y = 8.0;
        let hint = if view.vector_anim {
            "Pide una animación: «que la tapa se abra despacio»"
        } else if view.vector {
            "Pide un cambio, o usa el lazo para señalar una zona"
        } else if self.items.is_empty() && view.animation {
            "Describe cómo quieres que se mueva"
        } else if self.items.is_empty() {
            "Describe qué quieres dibujar"
        } else {
            "Escribe para dirigir la próxima vuelta"
        };
        // Un solo cuadro con el texto y, debajo, las miniaturas de los
        // adjuntos. Llena lo que deja la fila de botones; si el texto no
        // cabe, se desplaza dentro.
        let attachments = if self.attachments.is_empty() {
            0.0
        } else {
            ATTACHMENTS_HEIGHT
        };
        let text_height = (ui.available_height() - attachments - 48.0).max(48.0);
        let mut remove = None;
        let frame = egui::Frame::NONE
            .fill(theme::BAR_ALT)
            .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 0.0;
                let edit = egui::ScrollArea::vertical()
                    .id_salt("redaccion-agente")
                    .max_height(text_height)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.draft)
                                .hint_text(RichText::new(hint).color(theme::TEXT_4))
                                .desired_rows(3)
                                .desired_width(f32::INFINITY)
                                .min_size(vec2(0.0, text_height))
                                // Con un marco propio egui no usa `margin`: el
                                // relleno va en el marco.
                                .frame(egui::Frame::NONE.inner_margin(Margin::symmetric(12, 10))),
                        )
                    })
                    .inner;
                if !self.attachments.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.spacing_mut().item_spacing.x = 8.0;
                        for (i, a) in self.attachments.iter().enumerate() {
                            let key = (a.label.clone(), a.data.len());
                            let texture = self
                                .attachment_textures
                                .entry(key)
                                .or_insert_with(|| attachment_texture(ui.ctx(), a))
                                .clone();
                            if attachment_thumb(ui, a, texture.as_ref()) {
                                remove = Some(i);
                            }
                        }
                    });
                    ui.add_space(10.0);
                }
                edit
            });
        let edit = frame.inner;
        // Borde del cuadro: acento con el foco (el del texto va sin marco).
        let stroke = if edit.has_focus() {
            Stroke::new(1.0, theme::ACCENT_RING)
        } else {
            Stroke::new(1.0, theme::LINE_STRONG)
        };
        ui.painter().rect_stroke(
            frame.response.rect,
            CornerRadius::same(theme::RADIUS_FIELD),
            stroke,
            StrokeKind::Inside,
        );
        if let Some(i) = remove {
            self.attachments.remove(i);
        }
        let _ = edit.clone().accessible_name("Mensaje para el agente");
        let send_key = edit.has_focus()
            && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter));
        // Ctrl+V con una imagen en el portapapeles (una captura, una imagen
        // copiada): va como referencia, igual que con «+». Solo en la primera
        // pasada del fotograma, que puede repetirse con la misma entrada.
        if edit.has_focus() && !view.running && ui.ctx().current_pass_index() == 0 {
            let pasted: Vec<_> = ui.input(|i| {
                i.events
                    .iter()
                    .filter_map(|e| match e {
                        egui::Event::PasteImage(image) => Some(image.clone()),
                        _ => None,
                    })
                    .collect()
            });
            for image in pasted {
                if let Some(attachment) = pasted_attachment(&image, &self.attachments) {
                    out.push(Intent::AgentPasteImage(attachment));
                }
            }
        }
        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let attach = ui
                .add_enabled(
                    !view.running,
                    egui::Button::new(RichText::new("+").size(16.0).color(theme::TEXT_2))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                        .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
                        .min_size(vec2(32.0, 32.0)),
                )
                .on_hover_text("Adjuntar una imagen de referencia");
            let attach = attach.accessible_name("Adjuntar referencia");
            if attach.clicked() {
                out.push(Intent::AgentAttach);
            }
            if view.animation {
                // En animación: varias propuestas en paralelo para comparar.
                let can_ask = !view.running && !self.draft.trim().is_empty();
                let variants = ui.add_enabled(
                    can_ask,
                    egui::Button::new(RichText::new("Pedir 3 variantes").size(12.5))
                        .min_size(vec2(0.0, 32.0)),
                );
                if variants
                    .on_disabled_hover_text("Escribe qué variantes quieres")
                    .clicked()
                {
                    out.push(Intent::RequestVariants {
                        prompt: self.draft.trim().to_string(),
                    });
                    self.draft.clear();
                }
            } else if !view.vector && !view.vector_anim {
                // En vector el agente siempre sigue sobre dibujo.js.
                let follow = ui.add(
                    egui::Button::selectable(
                        self.continue_drawing,
                        RichText::new("Seguir con este dibujo").size(12.5).color(
                            if self.continue_drawing {
                                theme::TEXT
                            } else {
                                theme::TEXT_3
                            },
                        ),
                    )
                    .min_size(vec2(0.0, 32.0)),
                );
                if follow.clicked() {
                    self.continue_drawing = !self.continue_drawing;
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Mientras dibuja, lo escrito va a la vuelta siguiente del bucle.
                // Sin texto basta con una imagen (se pide dibujarla).
                let image_only =
                    self.draft.trim().is_empty() && !self.attachments.is_empty() && !view.running;
                let can_send = !self.draft.trim().is_empty() || image_only;
                let send = ui.add_enabled(
                    can_send,
                    egui::Button::new(
                        RichText::new("Enviar")
                            .font(theme::semibold(13.0))
                            .color(theme::ACCENT_INK),
                    )
                    .fill(theme::ACCENT)
                    .stroke(Stroke::NONE)
                    .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
                    .min_size(vec2(0.0, 32.0)),
                );
                if can_send && (send.clicked() || send_key) {
                    let prompt = if image_only {
                        IMAGE_ONLY_PROMPT.to_string()
                    } else {
                        self.draft.trim().to_string()
                    };
                    out.push(Intent::AgentSend {
                        prompt,
                        // Los mensajes al bucle en marcha van sin imágenes.
                        attachments: if view.running {
                            Vec::new()
                        } else {
                            std::mem::take(&mut self.attachments)
                        },
                        continuing: self.continue_drawing,
                    });
                    self.draft.clear();
                }
                let stop = ui.add_enabled(
                    view.running,
                    egui::Button::new(RichText::new("Detener").size(13.0).color(theme::DANGER))
                        .fill(theme::DANGER_BG)
                        .stroke(Stroke::new(1.0, theme::DANGER_LINE))
                        .corner_radius(CornerRadius::same(theme::RADIUS_BUTTON))
                        .min_size(vec2(0.0, 32.0)),
                );
                let stop = stop.accessible_name("Detener");
                if stop.clicked() {
                    out.push(Intent::AgentStop);
                }
            });
        });
    }
}

/// Una imagen pegada como adjunto PNG: «imagen pegada», «imagen pegada 2»…
pub fn pasted_attachment(image: &egui::ColorImage, already: &[Attachment]) -> Option<Attachment> {
    let [w, h] = image.size;
    let rgba: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|p| p.to_srgba_unmultiplied())
        .collect();
    let data = crate::encode_png(w as u32, h as u32, &rgba)?;
    let n = already
        .iter()
        .filter(|a| a.label.starts_with("imagen pegada"))
        .count();
    Some(Attachment {
        label: if n == 0 {
            "imagen pegada".into()
        } else {
            format!("imagen pegada {}", n + 1)
        },
        media_type: "image/png".into(),
        data,
    })
}

/// Nombre de un proveedor en la interfaz.
pub fn provider_name(p: ProviderKind) -> &'static str {
    match p {
        ProviderKind::ClaudeCode => "Claude Code",
        ProviderKind::Anthropic => "Anthropic API",
        ProviderKind::OpenAi => "OpenAI API",
        ProviderKind::Local => "Modelos locales",
    }
}

/// Tokens para la barra de estado: «850 tokens», «18,2 k tokens».
pub fn tokens_label(tokens: u64) -> String {
    if tokens < 1000 {
        format!("{tokens} tokens")
    } else if tokens < 1_000_000 {
        format!("{:.1} k tokens", tokens as f64 / 1000.0).replace('.', ",")
    } else {
        format!("{:.1} M tokens", tokens as f64 / 1_000_000.0).replace('.', ",")
    }
}

/// El bloque de una llamada a herramienta a partir de su entrada.
pub fn tool_item(name: &str, input_json: &str) -> ToolItem {
    let input: Value = serde_json::from_str(input_json).unwrap_or(Value::Null);
    let mut full = None;
    let lines =
        |script: &str| -> Vec<String> { split_batch(script).into_iter().map(|l| l.text).collect() };
    let (title, commands) = match name {
        "draw" => (
            "draw".to_string(),
            lines(input["commands"].as_str().unwrap_or_default()),
        ),
        "Bash" => {
            let command = input["command"].as_str().unwrap_or_default().trim();
            match batch_script(command) {
                Some(script) => ("lumeart batch".to_string(), lines(script)),
                None => {
                    let title = ellipsize(command.lines().next().unwrap_or_default(), TITLE_CHARS);
                    if title != command {
                        full = Some(command.to_string());
                    }
                    (title, Vec::new())
                }
            }
        }
        "Read" => {
            let path = input["file_path"].as_str().unwrap_or_default();
            let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
            (format!("Read {file}"), Vec::new())
        }
        other => (other.to_string(), Vec::new()),
    };
    ToolItem {
        name: name.to_string(),
        title,
        commands,
        result: None,
        is_error: false,
        open: false,
        full,
        output: None,
    }
}

/// ¿Es el aviso de Claude Code de que una orden necesita permiso?
fn blocked(text: &str) -> bool {
    text.contains("requires approval")
        || text.contains("requested permissions")
        || text.contains("haven't granted")
}

/// El texto que devolvió una herramienta (sin las imágenes).
pub fn output_text(output: &ToolOutput) -> String {
    output
        .content
        .iter()
        .filter_map(|p| match p {
            ContentPart::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// El guion de `lumeart batch '…'` (o `pxcli batch`), sin las comillas.
fn batch_script(command: &str) -> Option<&str> {
    let rest = ["lumeart batch", "pxcli batch"]
        .iter()
        .find_map(|prefix| command.strip_prefix(prefix))?
        .trim();
    let quote = rest.chars().next().filter(|c| *c == '\'' || *c == '"');
    Some(match quote {
        Some(q) => rest[1..].trim_end().strip_suffix(q).unwrap_or(&rest[1..]),
        None => rest,
    })
}

/// «48 ok», «2 ok · 1 falla», o la primera línea del resultado.
pub fn result_summary(output: &ToolOutput) -> String {
    let text: String = output
        .content
        .iter()
        .filter_map(|p| match p {
            ContentPart::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    // Puede haber varios `batch:` (Claude Code encadena lotes): se suman.
    let (mut ok, mut failed, mut found) = (0usize, 0usize, false);
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("batch: ") {
            let mut nums = rest
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse::<usize>().ok());
            if let (Some(o), Some(f)) = (nums.next(), nums.next()) {
                ok += o;
                failed += f;
                found = true;
            }
        }
    }
    if found {
        return match failed {
            0 => format!("{ok} ok"),
            1 => format!("{ok} ok · 1 falla"),
            n => format!("{ok} ok · {n} fallan"),
        };
    }
    let first = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default();
    if first.is_empty()
        && output
            .content
            .iter()
            .any(|p| matches!(p, ContentPart::Image { .. }))
    {
        return "imagen".to_string();
    }
    ellipsize(first, RESULT_CHARS)
}

/// Agente: línea vertical a la izquierda y el contenido sangrado. Devuelve
/// lo que devuelva `content`.
fn agent_line<R>(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.vertical(|ui| {
        ui.set_width(ui.available_width());
        content(ui)
    })
    .inner
}

fn user_bubble(ui: &mut egui::Ui, text: &str, attachments: &[String]) {
    ui.with_layout(Layout::top_down(Align::Max), |ui| {
        egui::Frame::NONE
            .fill(theme::HOVER)
            .corner_radius(CornerRadius::same(theme::RADIUS_CARD))
            .inner_margin(Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.set_max_width(290.0);
                ui.add(egui::Label::new(RichText::new(text).size(13.0).color(theme::TEXT)).wrap());
            });
        for name in attachments {
            egui::Frame::NONE
                .stroke(Stroke::new(1.0, theme::LINE_STRONG))
                .corner_radius(CornerRadius::same(theme::RADIUS_FIELD))
                .inner_margin(Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.label(RichText::new(name).size(12.0).color(theme::TEXT_2));
                });
        }
    });
}

/// Bloque de herramienta. Devuelve si se pulsó la cabecera (plegar).
fn tool_block(ui: &mut egui::Ui, index: usize, tool: &ToolItem) -> bool {
    let mut toggled = false;
    egui::Frame::NONE
        .fill(theme::BAR_ALT)
        .stroke(Stroke::new(1.0, theme::LINE_STRONG))
        .corner_radius(CornerRadius::same(theme::RADIUS_CARD_SMALL))
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 6.0;
            let header = ui
                .horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    status_icon(ui, tool.result.is_some(), tool.is_error);
                    // El resultado a la derecha; el título ocupa lo que quede y
                    // se recorta si no cabe (nunca ensancha el panel).
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(result) = &tool.result {
                            let color = if tool.is_error {
                                theme::DANGER
                            } else {
                                theme::TEXT_2
                            };
                            ui.label(RichText::new(result).size(12.0).color(color));
                        }
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&tool.title)
                                        .font(theme::mono(12.5))
                                        .color(theme::TEXT),
                                )
                                .truncate(),
                            );
                            if !tool.commands.is_empty() {
                                let n = tool.commands.len();
                                let plural = if n == 1 { "comando" } else { "comandos" };
                                ui.label(
                                    RichText::new(format!("· {n} {plural}"))
                                        .size(12.0)
                                        .color(theme::TEXT_3),
                                );
                            }
                        });
                    });
                })
                .response;
            let header = ui.interact(
                header.rect,
                ui.make_persistent_id(("bloque-herramienta", index)),
                Sense::click(),
            );
            let label = format!("{} · {}", tool.title, tool.commands.len());
            header.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &label));
            if header.clicked() {
                toggled = true;
            }
            let shown = if tool.open {
                tool.commands.len()
            } else {
                tool.commands.len().min(PREVIEW_LINES)
            };
            for command in &tool.commands[..shown] {
                command_line(ui, command);
            }
            // Desplegado: la orden entera y lo que devolvió (el error, si
            // falló), para saber qué pasó.
            let detail = tool.full.is_some() || tool.output.is_some();
            if tool.open {
                if let Some(full) = &tool.full {
                    detail_box(ui, full, theme::TEXT_2);
                }
                if let Some(output) = &tool.output {
                    let color = if tool.is_error {
                        theme::DANGER
                    } else {
                        theme::TEXT_3
                    };
                    detail_box(ui, &ellipsize(output, 4000), color);
                }
            }
            let hidden = tool.commands.len() - shown;
            let more = if hidden > 0 {
                let plural = if hidden == 1 { "comando" } else { "comandos" };
                Some(format!("+ {hidden} {plural} ▾"))
            } else if tool.open && (tool.commands.len() > PREVIEW_LINES || detail) {
                Some("Ocultar ▴".to_string())
            } else if detail && tool.is_error {
                Some("Ver el error ▾".to_string())
            } else if detail {
                Some("Ver la salida ▾".to_string())
            } else {
                None
            };
            if let Some(text) = more {
                let button = ui
                    .add(
                        egui::Button::new(
                            RichText::new(text)
                                .font(theme::mono(11.5))
                                .color(theme::ACCENT),
                        )
                        .frame(false),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if button.clicked() {
                    toggled = true;
                }
            }
        });
    toggled
}

/// Textura de la miniatura de un adjunto (solo PNG; los demás van con su
/// nombre).
fn attachment_texture(ctx: &egui::Context, a: &Attachment) -> Option<TextureHandle> {
    let (w, h, rgba) = crate::decode_image(&a.data)?;
    let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
    Some(ctx.load_texture(
        format!("adjunto-{}-{}", a.label, a.data.len()),
        image,
        egui::TextureOptions::LINEAR,
    ))
}

/// La miniatura de un adjunto con su «×» para quitarlo (devuelve si se
/// pulsó). Sin miniatura, una casilla con el nombre.
fn attachment_thumb(ui: &mut egui::Ui, a: &Attachment, texture: Option<&TextureHandle>) -> bool {
    let size = vec2(ATTACHMENT_THUMB, ATTACHMENT_THUMB);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let response = response.on_hover_text(&a.label);
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(6), theme::BG);
    match texture {
        Some(texture) => {
            let [w, h] = texture.size();
            let k = (ATTACHMENT_THUMB / w.max(h) as f32).min(f32::MAX);
            let image = egui::Rect::from_center_size(rect.center(), vec2(w as f32, h as f32) * k);
            painter.image(
                texture.id(),
                image,
                egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        None => {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                ellipsize(&a.label, 8),
                egui::FontId::proportional(10.5),
                theme::TEXT_3,
            );
        }
    }
    painter.rect_stroke(
        rect,
        CornerRadius::same(6),
        Stroke::new(1.0, theme::LINE_STRONG),
        StrokeKind::Inside,
    );
    // «×» arriba a la derecha, sobre la miniatura.
    let close = egui::Rect::from_center_size(rect.right_top() + vec2(-9.0, 9.0), vec2(16.0, 16.0));
    let x = ui
        .interact(close, response.id.with("quitar"), Sense::click())
        .on_hover_text("Quitar");
    x.widget_info(|| {
        egui::WidgetInfo::labeled(egui::Role::Button, true, format!("Quitar {}", a.label))
    });
    let painter = ui.painter();
    painter.circle_filled(
        close.center(),
        8.0,
        if x.hovered() {
            theme::DANGER
        } else {
            Color32::from_black_alpha(200)
        },
    );
    painter.text(
        close.center(),
        egui::Align2::CENTER_CENTER,
        "×",
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
    x.clicked()
}

/// Texto largo en mono dentro de un recuadro (la orden entera o la salida).
fn detail_box(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::NONE
        .fill(theme::BG)
        .stroke(Stroke::new(1.0, theme::LINE_SOFT))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(
                egui::Label::new(RichText::new(text).font(theme::mono(11.5)).color(color))
                    .wrap()
                    .selectable(true),
            );
        });
}

/// Lo que pensó el agente: plegado se ven las tres primeras líneas.
/// Devuelve si se pulsó para desplegar o plegar.
fn thinking_block(ui: &mut egui::Ui, index: usize, text: &str, open: bool) -> bool {
    let style = crate::markdown::Style {
        size: 12.5,
        color: theme::TEXT_3,
        italic: true,
    };
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let long = lines.len() > 3 || text.chars().count() > 280;
    let title = ui
        .add(
            egui::Button::new(
                RichText::new(if open {
                    "Pensamiento ▴"
                } else {
                    "Pensamiento ▾"
                })
                .font(theme::semibold(11.5))
                .color(theme::TEXT_3),
            )
            .frame(false),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let _ = index;
    if open || !long {
        crate::markdown::show(ui, text, style);
    } else {
        let preview = ellipsize(&lines[..lines.len().min(3)].join("\n"), 280);
        crate::markdown::show(ui, &preview, style);
    }
    title.clicked() && long
}

/// Una casilla del bucle: miniatura (hecha), borde acento (en curso) o
/// discontinua (pendiente).
fn turn_square(
    ui: &mut egui::Ui,
    state: TurnState,
    thumbnail: Option<&TextureHandle>,
    selected: bool,
    label: &str,
) -> egui::Response {
    let sense = if state == TurnState::Done {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(36.0, 36.0), sense);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let painter = ui.painter();
    let radius = CornerRadius::same(6);
    match state {
        TurnState::Done => {
            painter.rect_filled(rect, radius, theme::BG);
            if let Some(texture) = thumbnail {
                let [w, h] = texture.size();
                let scale = (30.0 / w as f32).min(30.0 / h as f32);
                let size = vec2(w as f32, h as f32) * scale;
                let image = egui::Rect::from_center_size(rect.center(), size);
                painter.image(
                    texture.id(),
                    image,
                    egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            let border = if selected {
                Stroke::new(1.5, theme::TEXT)
            } else if response.hovered() {
                Stroke::new(1.0, theme::TEXT_3)
            } else {
                Stroke::new(1.0, theme::LINE_STRONG)
            };
            painter.rect_stroke(rect, radius, border, StrokeKind::Inside);
        }
        TurnState::Current => {
            painter.rect(
                rect,
                radius,
                theme::ACCENT_SOFT,
                Stroke::new(1.5, theme::ACCENT),
                StrokeKind::Inside,
            );
        }
        TurnState::Pending => {
            let r = rect.shrink(0.5);
            let pts = vec![
                r.left_top(),
                r.right_top(),
                r.right_bottom(),
                r.left_bottom(),
                r.left_top(),
            ];
            for shape in
                egui::Shape::dashed_line(&pts, Stroke::new(1.0, theme::LINE_STRONG), 3.0, 3.0)
            {
                painter.add(shape);
            }
        }
    }
    response
}

/// Icono de estado de un bloque: punto (en marcha), ✓ (bien) o ✕ (error),
/// dibujado (no todas las fuentes tienen esos símbolos).
fn status_icon(ui: &mut egui::Ui, done: bool, error: bool) {
    let (rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
    let c = rect.center();
    let painter = ui.painter();
    if !done {
        painter.circle_stroke(c, 3.5, Stroke::new(1.4, theme::TEXT_3));
    } else if error {
        let stroke = Stroke::new(1.6, theme::DANGER);
        painter.line_segment([c + vec2(-3.5, -3.5), c + vec2(3.5, 3.5)], stroke);
        painter.line_segment([c + vec2(3.5, -3.5), c + vec2(-3.5, 3.5)], stroke);
    } else {
        painter.line(
            vec![
                c + vec2(-4.0, 0.0),
                c + vec2(-1.2, 3.0),
                c + vec2(4.0, -3.5),
            ],
            Stroke::new(1.6, theme::TEXT_3),
        );
    }
}

/// Una línea de comando: el nombre en azul, los argumentos y una muestra del
/// color.
fn command_line(ui: &mut egui::Ui, command: &str) {
    command_line_colored(ui, command, LineColors::NORMAL);
}

/// Colores de una línea de comando: el nombre y lo demás.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineColors {
    pub name: Color32,
    pub args: Color32,
}

impl LineColors {
    pub const NORMAL: LineColors = LineColors {
        name: theme::INFO,
        args: theme::TEXT_2,
    };
    /// Lo que aún no se ha pintado.
    pub const MUTED: LineColors = LineColors {
        name: theme::TEXT_4,
        args: theme::TEXT_4,
    };
    /// Lo que se está pintando ahora.
    pub const NOW: LineColors = LineColors {
        name: theme::ACCENT,
        args: theme::TEXT,
    };
}

/// Una línea de comando: nombre, argumentos y, si acaba en un color, su
/// muestra y el código.
pub fn command_line_colored(ui: &mut egui::Ui, command: &str, colors: LineColors) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let mut words = command.split_whitespace();
        let name = words.next().unwrap_or_default();
        let rest: Vec<&str> = words.collect();
        ui.label(
            RichText::new(name)
                .font(theme::mono(12.0))
                .color(colors.name),
        );
        let (color, args): (Option<Rgba>, Vec<&str>) = match rest.last() {
            Some(last) if Rgba::parse(last).is_ok() => {
                (Rgba::parse(last).ok(), rest[..rest.len() - 1].to_vec())
            }
            _ => (None, rest.clone()),
        };
        if !args.is_empty() {
            ui.label(
                RichText::new(args.join(" "))
                    .font(theme::mono(12.0))
                    .color(colors.args),
            );
        }
        if let Some(c) = color {
            let (rect, _) = ui.allocate_exact_size(vec2(9.0, 9.0), Sense::hover());
            ui.painter().rect(
                rect,
                CornerRadius::same(2),
                crate::shell::color32(c),
                Stroke::new(1.0, theme::LINE_STRONG),
                StrokeKind::Inside,
            );
            ui.label(
                RichText::new(rest.last().copied().unwrap_or_default().to_uppercase())
                    .font(theme::mono(12.0))
                    .color(colors.args),
            );
        }
    });
}

fn review_texture(ctx: &egui::Context, index: usize, png: &[u8]) -> Option<TextureHandle> {
    // Claude Code puede devolver la imagen en otro formato (JPEG, WebP).
    let (w, h, rgba) = crate::decode_image(png)?;
    let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
    Some(ctx.load_texture(
        format!("revision-{index}"),
        image,
        egui::TextureOptions::NEAREST,
    ))
}

fn review(ui: &mut egui::Ui, texture: Option<&TextureHandle>) {
    ui.horizontal(|ui| {
        let size = vec2(72.0, 72.0);
        match texture {
            Some(texture) => {
                let [w, h] = texture.size();
                let scale = (size.x / w as f32).min(size.y / h as f32);
                let image = egui::Image::new((texture.id(), vec2(w as f32, h as f32) * scale))
                    .corner_radius(CornerRadius::same(6));
                ui.add(image);
            }
            None => {
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(6), theme::FIELD);
            }
        }
        ui.label(RichText::new("Revisión").size(12.0).color(theme::TEXT_3));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_app::ports::Usage;

    #[test]
    fn draw_calls_become_tool_blocks_with_their_commands() {
        let input = serde_json::json!({"commands": "fill_rect 13 3 6 4 #A86B3C\nline 12 7 19 7 #BFD9E6; set_pixel 1 1 red"});
        let item = tool_item("draw", &input.to_string());
        assert_eq!(item.title, "draw");
        assert_eq!(
            item.commands,
            [
                "fill_rect 13 3 6 4 #A86B3C",
                "line 12 7 19 7 #BFD9E6",
                "set_pixel 1 1 red"
            ]
        );
    }

    #[test]
    fn claude_code_batches_are_read_from_bash() {
        let input = serde_json::json!({"command": "lumeart batch '\nfill_rect 12 20 8 8 #f3e2b8\nset_pixel 1 1 #fff\n'"});
        let item = tool_item("Bash", &input.to_string());
        assert_eq!(item.title, "lumeart batch");
        assert_eq!(
            item.commands,
            ["fill_rect 12 20 8 8 #f3e2b8", "set_pixel 1 1 #fff"]
        );
        let item = tool_item(
            "Bash",
            r#"{"command":"lumeart export --scale 10 revision.png"}"#,
        );
        assert_eq!(item.title, "lumeart export --scale 10 revision.png");
        assert!(item.commands.is_empty());
        let item = tool_item("Read", r#"{"file_path":"/tmp/run-1/revision.png"}"#);
        assert_eq!(item.title, "Read revision.png");
    }

    #[test]
    fn results_are_summarised() {
        let out = |t: &str| ToolOutput::text(t);
        assert_eq!(result_summary(&out("batch: 48 ok, 0 failed")), "48 ok");
        assert_eq!(
            result_summary(&out("line 2: x -> err\nbatch: 1 ok, 1 failed")),
            "1 ok · 1 falla"
        );
        assert_eq!(
            result_summary(&out("batch: 22 ok, 0 failed\nbatch: 32 ok, 2 failed\nok")),
            "54 ok · 2 fallan"
        );
        assert_eq!(result_summary(&out("ok #ff0000ff")), "ok #ff0000ff");
    }

    #[test]
    fn events_build_the_conversation() {
        let mut panel = AgentPanel::default();
        panel.started("una poción", &[]);
        panel.apply(&AgentEvent::Usage(Usage {
            input_tokens: 100,
            output_tokens: 20,
            cached_tokens: 0,
        }));
        panel.apply(&AgentEvent::Text("Empiezo por el fondo.".into()));
        panel.apply(&AgentEvent::ToolCall {
            name: "draw".into(),
            input_json: r#"{"commands":"set_pixel 1 1 red\nset_pixel 2 2 blue"}"#.into(),
        });
        assert_eq!(panel.last_command.as_deref(), Some("set_pixel 2 2 blue"));
        panel.apply(&AgentEvent::ToolResult {
            name: "draw".into(),
            output: ToolOutput::text("batch: 2 ok, 0 failed"),
        });
        panel.apply(&AgentEvent::ToolResult {
            name: "view_canvas".into(),
            output: ToolOutput {
                content: vec![ContentPart::Image {
                    media_type: "image/png".into(),
                    data: vec![1, 2],
                }],
                is_error: false,
            },
        });
        panel.finished("Terminado en 1 paso", false);
        assert_eq!(panel.steps, 1);
        assert_eq!(panel.tokens, 120);
        assert_eq!(panel.items.len(), 5);
        let ConvItem::Tool(tool) = &panel.items[2] else {
            panic!()
        };
        assert_eq!(tool.result.as_deref(), Some("2 ok"));
        assert!(matches!(panel.items[3], ConvItem::Review { .. }));
        assert_eq!(
            panel.items[4],
            ConvItem::End {
                text: "Terminado en 1 paso".into(),
                error: false
            }
        );
    }

    #[test]
    fn long_titles_and_results_are_cut() {
        let item = tool_item(
            "Bash",
            r#"{"command":"lumeart export --scale 10 una-revision-con-un-nombre-muy-largo.png"}"#,
        );
        assert_eq!(item.title.chars().count(), TITLE_CHARS);
        assert!(item.title.ends_with('…'));
        let long = ToolOutput::text(
            "The file /tmp/lumeart-agente-660120/run-1/notas.md has been updated successfully.",
        );
        let summary = result_summary(&long);
        assert_eq!(summary.chars().count(), RESULT_CHARS);
        assert_eq!(ellipsize("corto", 10), "corto");
    }

    #[test]
    fn token_labels() {
        assert_eq!(tokens_label(850), "850 tokens");
        assert_eq!(tokens_label(18_240), "18,2 k tokens");
        assert_eq!(tokens_label(2_500_000), "2,5 M tokens");
    }
}
