//! `AnthropicProvider`: la Messages API de Anthropic por HTTP (no hay SDK
//! oficial para Rust), con herramientas e imágenes.
//!
//! - Versión de la API `2026-10-21` (sin `temperature`, `top_p` ni `top_k`).
//! - El razonamiento (`thinking`) y otros bloques internos vuelven al modelo
//!   tal cual, como [`ContentPart::Opaque`].
//! - Caché automática del prefijo (`cache_control` en la petición).
//! - Si el modelo lo admite, `fallbacks: "default"`: si los filtros de
//!   seguridad rechazan una petición, la API la repite con otro modelo.
//! - Reintentos con espera creciente para 408, 409, 429, 5xx y 529 y para los
//!   fallos de conexión; respeta `retry-after`.
//!
//! La clave llega como [`Secret`] y nunca se escribe en errores ni en `Debug`.

use base64::Engine;
use lume_app::ModelProvider;
use lume_app::ports::{
    BoxFuture, ContentPart, Message, ModelRequest, ModelResponse, PortError, Role, Secret,
    StopReason, Usage,
};
use serde_json::{Value, json};

use crate::http;

/// Dirección de la API.
pub const API_URL: &str = "https://api.anthropic.com";
/// Versión de la API que se envía en `anthropic-version`.
pub const API_VERSION: &str = "2026-10-21";
/// Cabecera beta de `fallbacks: "default"`.
pub const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
/// Nombre de este proveedor en los bloques opacos.
pub const PROVIDER_ID: &str = "anthropic";

pub use crate::http::RetryPolicy;

/// Proveedor de la Messages API.
pub struct AnthropicProvider {
    client: reqwest::Client,
    key: Secret,
    base_url: String,
    retry: RetryPolicy,
    effort: Option<String>,
}

impl std::fmt::Debug for AnthropicProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnthropicProvider")
            .field("base_url", &self.base_url)
            .field("retry", &self.retry)
            .field("effort", &self.effort)
            .finish_non_exhaustive()
    }
}

impl AnthropicProvider {
    /// Con la clave del usuario (sale del `SecretStore`).
    pub fn new(key: Secret) -> Self {
        Self {
            client: http::client(),
            key,
            base_url: API_URL.to_string(),
            retry: RetryPolicy::default(),
            effort: Some("high".to_string()),
        }
    }

    /// Otra dirección (pruebas con un servidor local).
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    /// Esfuerzo (`low`, `medium`, `high`, `xhigh`, `max`) o `None` para el
    /// del modelo.
    pub fn with_effort(mut self, effort: Option<&str>) -> Self {
        self.effort = effort.map(str::to_string);
        self
    }
}

impl ModelProvider for AnthropicProvider {
    fn id(&self) -> &str {
        PROVIDER_ID
    }

    fn complete<'a>(
        &'a self,
        request: ModelRequest,
    ) -> BoxFuture<'a, Result<ModelResponse, PortError>> {
        Box::pin(async move {
            let body = request_body(&request, self.effort.as_deref());
            let fallbacks = supports_fallbacks(&request.model);
            let text = http::send_with_retries(
                &self.retry,
                "Anthropic",
                || {
                    let mut builder = self
                        .client
                        .post(format!("{}/v1/messages", self.base_url))
                        .header("x-api-key", self.key.expose())
                        .header("anthropic-version", API_VERSION)
                        .header("content-type", "application/json")
                        .json(&body);
                    if fallbacks {
                        builder = builder.header("anthropic-beta", FALLBACK_BETA);
                    }
                    builder
                },
                |status, text| http::Failure {
                    error: error_from(status, text),
                    retryable: http::is_retryable_status(status),
                },
            )
            .await?;
            parse_response(&text)
        })
    }
}

/// ¿Admite el modelo `fallbacks: "default"`? Claude Haiku 5.5 no tiene
/// respaldo en el servidor.
pub fn supports_fallbacks(model: &str) -> bool {
    [
        "claude-opus-5",
        "claude-sonnet-5-5",
        "claude-fable-5",
        "claude-mythos-5",
    ]
    .iter()
    .any(|prefix| model.starts_with(prefix))
}

/// Cuerpo JSON de la petición.
pub fn request_body(request: &ModelRequest, effort: Option<&str>) -> Value {
    let tools: Vec<Value> = request
        .tools
        .iter()
        .map(|tool| {
            let schema = serde_json::from_str::<Value>(&tool.input_schema_json)
                .unwrap_or_else(|_| json!({"type": "object"}));
            json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": schema,
            })
        })
        .collect();
    let messages: Vec<Value> = request.messages.iter().filter_map(message_json).collect();
    let mut body = json!({
        "model": request.model,
        "max_tokens": request.max_tokens,
        "messages": messages,
        // Caché automática del prefijo (instrucciones, herramientas e historia).
        "cache_control": {"type": "ephemeral"},
    });
    if !request.system.is_empty() {
        body["system"] = json!(request.system);
    }
    if !tools.is_empty() {
        body["tools"] = json!(tools);
    }
    if let Some(effort) = effort {
        body["output_config"] = json!({ "effort": effort });
    }
    if supports_fallbacks(&request.model) {
        body["fallbacks"] = json!("default");
    }
    body
}

fn message_json(message: &Message) -> Option<Value> {
    let content: Vec<Value> = message.content.iter().filter_map(block_json).collect();
    if content.is_empty() {
        return None;
    }
    let role = match message.role {
        Role::User => "user",
        Role::Assistant => "assistant",
    };
    Some(json!({ "role": role, "content": content }))
}

fn block_json(part: &ContentPart) -> Option<Value> {
    match part {
        // La API no admite bloques de texto vacíos.
        ContentPart::Text(text) if text.is_empty() => None,
        ContentPart::Text(text) => Some(json!({"type": "text", "text": text})),
        ContentPart::Image { media_type, data } => Some(json!({
            "type": "image",
            "source": {
                "type": "base64",
                "media_type": media_type,
                "data": base64::engine::general_purpose::STANDARD.encode(data),
            },
        })),
        ContentPart::ToolUse {
            id,
            name,
            input_json,
        } => {
            let input = serde_json::from_str::<Value>(input_json).unwrap_or_else(|_| json!({}));
            Some(json!({"type": "tool_use", "id": id, "name": name, "input": input}))
        }
        ContentPart::ToolResult {
            id,
            content,
            is_error,
        } => {
            let inner: Vec<Value> = content.iter().filter_map(block_json).collect();
            let mut block = json!({"type": "tool_result", "tool_use_id": id, "content": inner});
            if *is_error {
                block["is_error"] = json!(true);
            }
            Some(block)
        }
        ContentPart::Opaque { provider, json } if provider == PROVIDER_ID => {
            serde_json::from_str(json).ok()
        }
        ContentPart::Opaque { .. } => None,
    }
}

/// Lee una respuesta correcta de la API.
pub fn parse_response(text: &str) -> Result<ModelResponse, PortError> {
    let value: Value = serde_json::from_str(text)
        .map_err(|e| PortError::Provider(format!("respuesta de Anthropic ilegible: {e}")))?;
    let stop = match value["stop_reason"].as_str().unwrap_or("") {
        "end_turn" | "stop_sequence" => StopReason::EndTurn,
        "tool_use" => StopReason::ToolUse,
        "max_tokens" => StopReason::MaxTokens,
        "refusal" => {
            let category = value["stop_details"]["category"]
                .as_str()
                .map(|c| format!(" (categoría: {c})"))
                .unwrap_or_default();
            return Err(PortError::Provider(format!(
                "el modelo rechazó la petición{category}; prueba a reformularla"
            )));
        }
        other => StopReason::Other(other.to_string()),
    };
    let mut content = Vec::new();
    for block in value["content"].as_array().into_iter().flatten() {
        match block["type"].as_str() {
            Some("text") => {
                content.push(ContentPart::Text(
                    block["text"].as_str().unwrap_or_default().to_string(),
                ));
            }
            Some("tool_use") => content.push(ContentPart::ToolUse {
                id: block["id"].as_str().unwrap_or_default().to_string(),
                name: block["name"].as_str().unwrap_or_default().to_string(),
                input_json: block["input"].to_string(),
            }),
            // Razonamiento, avisos de respaldo y demás: vuelven tal cual.
            _ => content.push(ContentPart::Opaque {
                provider: PROVIDER_ID.to_string(),
                json: block.to_string(),
            }),
        }
    }
    let count = |key: &str| value["usage"][key].as_u64().unwrap_or(0);
    let usage = Usage {
        input_tokens: count("input_tokens") + count("cache_creation_input_tokens"),
        output_tokens: count("output_tokens"),
        cached_tokens: count("cache_read_input_tokens"),
    };
    Ok(ModelResponse {
        content,
        stop,
        usage,
    })
}

/// Error claro a partir de una respuesta HTTP fallida.
pub fn error_from(status: u16, body: &str) -> PortError {
    let parsed: Option<Value> = serde_json::from_str(body).ok();
    let message = parsed
        .as_ref()
        .and_then(|v| v["error"]["message"].as_str())
        .map(http::scrub_keys)
        .unwrap_or_else(|| {
            let snippet: String = body.chars().take(200).collect();
            if snippet.trim().is_empty() {
                format!("HTTP {status}")
            } else {
                snippet
            }
        });
    match status {
        401 => PortError::Auth(format!(
            "Anthropic la rechaza ({message}). Revísala en Ajustes › Proveedores de IA"
        )),
        403 => PortError::Auth(format!("sin permiso para esta petición ({message})")),
        402 => PortError::Provider(format!("problema de facturación en Anthropic ({message})")),
        404 => PortError::Invalid(format!("no existe (¿el modelo es correcto?): {message}")),
        429 => PortError::RateLimited(format!(
            "{message}. Espera un poco o revisa los límites de tu cuenta"
        )),
        408 => PortError::Timeout,
        500..=599 => PortError::Unavailable(format!(
            "Anthropic no responde ahora (HTTP {status}): {message}"
        )),
        _ => PortError::Invalid(format!("petición rechazada (HTTP {status}): {message}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallbacks_only_for_models_that_have_them() {
        assert!(supports_fallbacks("claude-opus-5-5"));
        assert!(supports_fallbacks("claude-sonnet-5-5"));
        assert!(supports_fallbacks("claude-fable-5-1"));
        assert!(!supports_fallbacks("claude-haiku-5-5"));
        assert!(!supports_fallbacks("claude-sonnet-4-6"));
    }

    #[test]
    fn the_key_is_not_in_debug() {
        let p = AnthropicProvider::new(Secret::new("sk-ant-NO-SE-VE"));
        assert!(!format!("{p:?}").contains("NO-SE-VE"));
    }
}
