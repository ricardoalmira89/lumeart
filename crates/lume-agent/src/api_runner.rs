//! `ApiRunner`: el bucle del agente sobre un `ModelProvider` (Anthropic,
//! OpenAI, compatibles). Envía la conversación, ejecuta las herramientas que
//! pida el modelo y devuelve sus resultados, hasta que el modelo termina, se
//! llega a un límite o se cancela.
//!
//! Un paso es una llamada al modelo. La cancelación se mira antes de cada
//! llamada y de cada herramienta, y también corta la espera al modelo.

use std::sync::Arc;

use lume_app::ports::{
    AgentEvent, AgentTask, BoxFuture, CancellationToken, ContentPart, Conversation, EventSink,
    Message, ModelRequest, PortError, Role, RunEnd, RunSummary, ToolBox, Usage,
};
use lume_app::{AgentRunner, ModelProvider};

use crate::critic::AgentRole;
use crate::prompts;

/// Tokens de salida por respuesta del modelo.
pub const MAX_OUTPUT_TOKENS: u32 = 16_000;

/// Agente que habla con un modelo por API. Por defecto usa las
/// instrucciones de pixel art de `prompts/`.
pub struct ApiRunner {
    provider: Arc<dyn ModelProvider>,
    system: Option<String>,
    role: AgentRole,
}

impl ApiRunner {
    pub fn new(provider: Arc<dyn ModelProvider>) -> Self {
        Self {
            provider,
            system: None,
            role: AgentRole::Artist,
        }
    }

    /// Como crítico: sus instrucciones y su encargo (mira el dibujo con
    /// `view_canvas` y devuelve un veredicto). Las herramientas de solo
    /// lectura las pone [`review`](crate::critic::review).
    pub fn as_critic(mut self) -> Self {
        self.role = AgentRole::Critic;
        self
    }

    /// Con otras instrucciones de sistema (el crítico, por ejemplo).
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }
}

/// Primer mensaje: el encargo (con el estado heredado) y las imágenes
/// adjuntas.
fn first_message(task: &AgentTask, role: AgentRole) -> Message {
    let text = match role {
        AgentRole::Artist => prompts::pixel_task(task),
        AgentRole::Critic => prompts::critic_task_api(task),
    };
    let mut content = vec![ContentPart::Text(text)];
    for attachment in &task.attachments {
        content.push(ContentPart::Text(attachment.label.clone()));
        content.push(ContentPart::Image {
            media_type: attachment.media_type.clone(),
            data: attachment.data.clone(),
        });
    }
    Message {
        role: Role::User,
        content,
    }
}

impl AgentRunner for ApiRunner {
    fn run<'a>(
        &'a self,
        task: AgentTask,
        tools: ToolBox,
        events: EventSink,
        cancel: CancellationToken,
    ) -> BoxFuture<'a, Result<RunSummary, PortError>> {
        Box::pin(async move {
            let specs = tools.specs();
            let system = self.system.clone().unwrap_or_else(|| match self.role {
                AgentRole::Artist => prompts::pixel_system(&task),
                AgentRole::Critic => prompts::critic_system(),
            });
            // Con `Resume`, la conversación de antes y el encargo nuevo detrás;
            // si no, solo el encargo (contexto vacío).
            let mut messages = match &task.resume {
                Some(Conversation::Messages(previous)) => previous.clone(),
                _ => Vec::new(),
            };
            messages.push(first_message(&task, self.role));
            let mut summary = RunSummary {
                steps: 0,
                usage: Usage::default(),
                final_text: String::new(),
                end: RunEnd::Finished,
                conversation: None,
                loops: None,
            };
            let end = 'run: loop {
                if cancel.is_cancelled() {
                    break RunEnd::Cancelled;
                }
                if summary.steps >= task.limits.max_steps {
                    break RunEnd::StepLimit;
                }
                if summary.usage.total() >= task.limits.max_tokens {
                    break RunEnd::TokenLimit;
                }

                let request = ModelRequest {
                    model: task.model.clone(),
                    system: system.clone(),
                    messages: messages.clone(),
                    tools: specs.clone(),
                    max_tokens: MAX_OUTPUT_TOKENS,
                };
                let response = tokio::select! {
                    biased;
                    _ = cancel.cancelled() => break 'run RunEnd::Cancelled,
                    response = self.provider.complete(request) => response?,
                };
                summary.steps += 1;
                summary.usage.add(response.usage);
                events.emit(AgentEvent::Usage(response.usage));

                let mut calls = Vec::new();
                for part in &response.content {
                    match part {
                        ContentPart::Text(text) if !text.trim().is_empty() => {
                            events.emit(AgentEvent::Text(text.clone()));
                            summary.final_text = text.clone();
                        }
                        ContentPart::ToolUse {
                            id,
                            name,
                            input_json,
                        } => calls.push((id.clone(), name.clone(), input_json.clone())),
                        // El razonamiento de Claude llega como bloque opaco.
                        ContentPart::Opaque { json, .. } => {
                            if let Ok(block) = serde_json::from_str::<serde_json::Value>(json)
                                && block["type"] == "thinking"
                                && let Some(text) = block["thinking"].as_str()
                                && !text.trim().is_empty()
                            {
                                events.emit(AgentEvent::Thinking(text.to_string()));
                            }
                        }
                        _ => {}
                    }
                }
                messages.push(Message {
                    role: Role::Assistant,
                    content: response.content,
                });
                if calls.is_empty() {
                    break RunEnd::Finished;
                }

                let mut results = Vec::new();
                for (id, name, input_json) in calls {
                    if cancel.is_cancelled() {
                        break 'run RunEnd::Cancelled;
                    }
                    events.emit(AgentEvent::ToolCall {
                        name: name.clone(),
                        input_json: input_json.clone(),
                    });
                    let output = tools.call(&name, &input_json).await;
                    events.emit(AgentEvent::ToolResult {
                        name,
                        output: output.clone(),
                    });
                    results.push(ContentPart::ToolResult {
                        id,
                        content: output.content,
                        is_error: output.is_error,
                    });
                }
                messages.push(Message {
                    role: Role::User,
                    content: results,
                });
            };
            summary.end = end;
            // Solo una conversación completa se puede retomar.
            if end == RunEnd::Finished {
                summary.conversation = Some(Conversation::Messages(messages));
            }
            Ok(summary)
        })
    }
}
