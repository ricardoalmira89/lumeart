//! `OpenAiProvider` y `OpenAiCompatibleProvider`: la API de Chat Completions
//! con *function calling* e imágenes. La misma traducción sirve para OpenAI y
//! para los servidores compatibles (Ollama, LM Studio, llama.cpp), que solo
//! cambian la dirección, la clave (opcional) y el nombre de `max_tokens`.
//!
//! - Las imágenes van como `image_url` con una URL `data:`.
//! - Los mensajes `tool` solo admiten texto: si un resultado trae imágenes
//!   (`view_canvas`), van en un mensaje de usuario justo después.
//! - Los bloques opacos de otros proveedores no se envían.
//!
//! La clave llega como [`Secret`] y nunca se escribe en errores ni en `Debug`;
//! si la API la repite (enmascarada) en un error, se borra.

use base64::Engine;
use lume_app::ModelProvider;
use lume_app::ports::{
    BoxFuture, ContentPart, ModelRequest, ModelResponse, PortError, Role, Secret, StopReason, Usage,
};
use serde_json::{Value, json};

use crate::http::{self, RetryPolicy};

/// Dirección de la API de OpenAI (con `/v1`).
pub const OPENAI_URL: &str = "https://api.openai.com/v1";

/// Qué servidor hay detrás.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    /// OpenAI: clave obligatoria y `max_completion_tokens`.
    OpenAi,
    /// Compatible (Ollama, LM Studio, llama.cpp): clave opcional y
    /// `max_tokens`, que es lo que entienden todos.
    Compatible,
}

/// Cliente de Chat Completions (lo comparten los dos proveedores).
struct ChatClient {
    client: reqwest::Client,
    flavor: Flavor,
    base_url: String,
    key: Option<Secret>,
    default_model: Option<String>,
    retry: RetryPolicy,
}

impl ChatClient {
    fn new(flavor: Flavor, base_url: &str, key: Option<Secret>) -> Self {
        Self {
            client: http::client(),
            flavor,
            base_url: base_url.trim_end_matches('/').to_string(),
            key,
            default_model: None,
            retry: RetryPolicy::default(),
        }
    }

    fn name(&self) -> &'static str {
        match self.flavor {
            Flavor::OpenAi => "OpenAI",
            Flavor::Compatible => "el servidor local",
        }
    }

    async fn complete(&self, mut request: ModelRequest) -> Result<ModelResponse, PortError> {
        if request.model.trim().is_empty() {
            match &self.default_model {
                Some(model) => request.model = model.clone(),
                None => {
                    return Err(PortError::Invalid(
                        "falta el modelo: elige uno en Ajustes".into(),
                    ));
                }
            }
        }
        let body = request_body(&request, self.flavor);
        let flavor = self.flavor;
        let text = http::send_with_retries(
            &self.retry,
            self.name(),
            || {
                let mut builder = self
                    .client
                    .post(format!("{}/chat/completions", self.base_url))
                    .header("content-type", "application/json")
                    .json(&body);
                if let Some(key) = &self.key {
                    builder = builder.bearer_auth(key.expose());
                }
                builder
            },
            |status, text| failure(flavor, status, text),
        )
        .await?;
        parse_response(&text)
    }
}

impl std::fmt::Debug for ChatClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatClient")
            .field("flavor", &self.flavor)
            .field("base_url", &self.base_url)
            .field("has_key", &self.key.is_some())
            .field("default_model", &self.default_model)
            .finish_non_exhaustive()
    }
}

/// La API de OpenAI.
#[derive(Debug)]
pub struct OpenAiProvider(ChatClient);

impl OpenAiProvider {
    /// Con la clave del usuario (sale del `SecretStore`).
    pub fn new(key: Secret) -> Self {
        Self(ChatClient::new(Flavor::OpenAi, OPENAI_URL, Some(key)))
    }

    /// Otra dirección (con `/v1`; pruebas con un servidor local).
    pub fn with_base_url(mut self, url: &str) -> Self {
        self.0.base_url = url.trim_end_matches('/').to_string();
        self
    }

    /// Modelo para las peticiones que no lo traen.
    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.0.default_model = Some(model.into());
        self
    }

    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.0.retry = retry;
        self
    }
}

impl ModelProvider for OpenAiProvider {
    fn id(&self) -> &str {
        "openai"
    }

    fn complete<'a>(
        &'a self,
        request: ModelRequest,
    ) -> BoxFuture<'a, Result<ModelResponse, PortError>> {
        Box::pin(self.0.complete(request))
    }
}

/// Un servidor compatible con la API de OpenAI en la dirección que se dé
/// (`http://localhost:11434/v1` para Ollama).
#[derive(Debug)]
pub struct OpenAiCompatibleProvider(ChatClient);

impl OpenAiCompatibleProvider {
    pub fn new(base_url: &str) -> Self {
        Self(ChatClient::new(Flavor::Compatible, base_url, None))
    }

    /// Clave, si el servidor la pide.
    pub fn with_key(mut self, key: Secret) -> Self {
        self.0.key = Some(key);
        self
    }

    /// Modelo para las peticiones que no lo traen.
    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.0.default_model = Some(model.into());
        self
    }

    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.0.retry = retry;
        self
    }
}

impl ModelProvider for OpenAiCompatibleProvider {
    fn id(&self) -> &str {
        "openai-compatible"
    }

    fn complete<'a>(
        &'a self,
        request: ModelRequest,
    ) -> BoxFuture<'a, Result<ModelResponse, PortError>> {
        Box::pin(self.0.complete(request))
    }
}

/// Cuerpo JSON de la petición.
pub fn request_body(request: &ModelRequest, flavor: Flavor) -> Value {
    let mut messages = Vec::new();
    if !request.system.is_empty() {
        messages.push(json!({"role": "system", "content": request.system}));
    }
    for message in &request.messages {
        match message.role {
            Role::User => user_messages(&message.content, &mut messages),
            Role::Assistant => {
                if let Some(m) = assistant_message(&message.content) {
                    messages.push(m);
                }
            }
        }
    }
    let tools: Vec<Value> = request
        .tools
        .iter()
        .map(|tool| {
            let parameters = serde_json::from_str::<Value>(&tool.input_schema_json)
                .unwrap_or_else(|_| json!({"type": "object"}));
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": parameters,
                },
            })
        })
        .collect();
    let mut body = json!({"model": request.model, "messages": messages});
    let max_key = match flavor {
        Flavor::OpenAi => "max_completion_tokens",
        Flavor::Compatible => "max_tokens",
    };
    body[max_key] = json!(request.max_tokens);
    if !tools.is_empty() {
        body["tools"] = json!(tools);
    }
    body
}

/// `data:image/png;base64,…`
fn data_url(media_type: &str, data: &[u8]) -> String {
    format!(
        "data:{media_type};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(data)
    )
}

/// Texto e imágenes de un mensaje de usuario: texto plano si no hay imágenes
/// (lo entienden todos los servidores) o una lista de partes.
fn user_content(parts: &[&ContentPart]) -> Option<Value> {
    let has_images = parts.iter().any(|p| matches!(p, ContentPart::Image { .. }));
    if !has_images {
        let text: Vec<&str> = parts
            .iter()
            .filter_map(|p| match p {
                ContentPart::Text(t) if !t.is_empty() => Some(t.as_str()),
                _ => None,
            })
            .collect();
        return (!text.is_empty()).then(|| json!(text.join("\n\n")));
    }
    let content: Vec<Value> = parts
        .iter()
        .filter_map(|p| match p {
            ContentPart::Text(t) if !t.is_empty() => Some(json!({"type": "text", "text": t})),
            ContentPart::Image { media_type, data } => Some(json!({
                "type": "image_url",
                "image_url": {"url": data_url(media_type, data)},
            })),
            _ => None,
        })
        .collect();
    Some(json!(content))
}

/// Un mensaje de usuario puede dar varios: un mensaje `tool` por resultado
/// y, si hay imágenes en los resultados, un mensaje de usuario con ellas.
fn user_messages(content: &[ContentPart], out: &mut Vec<Value>) {
    let mut images: Vec<ContentPart> = Vec::new();
    let mut plain: Vec<&ContentPart> = Vec::new();
    for part in content {
        match part {
            ContentPart::ToolResult {
                id,
                content,
                is_error,
            } => {
                let mut text: Vec<&str> = content
                    .iter()
                    .filter_map(|p| match p {
                        ContentPart::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                let attached: Vec<&ContentPart> = content
                    .iter()
                    .filter(|p| matches!(p, ContentPart::Image { .. }))
                    .collect();
                if !attached.is_empty() {
                    text.push("(la imagen va en el mensaje siguiente)");
                    images.push(ContentPart::Text(format!("Imagen de {id}:")));
                    images.extend(attached.into_iter().cloned());
                }
                let mut joined = text.join("\n");
                if *is_error && !joined.starts_with("err") {
                    joined = format!("error: {joined}");
                }
                out.push(json!({"role": "tool", "tool_call_id": id, "content": joined}));
            }
            ContentPart::Text(_) | ContentPart::Image { .. } => plain.push(part),
            ContentPart::ToolUse { .. } | ContentPart::Opaque { .. } => {}
        }
    }
    let mut extra: Vec<&ContentPart> = images.iter().collect();
    plain.append(&mut extra);
    if let Some(content) = user_content(&plain) {
        out.push(json!({"role": "user", "content": content}));
    }
}

fn assistant_message(content: &[ContentPart]) -> Option<Value> {
    let text: Vec<&str> = content
        .iter()
        .filter_map(|p| match p {
            ContentPart::Text(t) if !t.is_empty() => Some(t.as_str()),
            _ => None,
        })
        .collect();
    let calls: Vec<Value> = content
        .iter()
        .filter_map(|p| match p {
            ContentPart::ToolUse {
                id,
                name,
                input_json,
            } => Some(json!({
                "id": id,
                "type": "function",
                "function": {"name": name, "arguments": input_json},
            })),
            _ => None,
        })
        .collect();
    if text.is_empty() && calls.is_empty() {
        return None;
    }
    let mut message = json!({
        "role": "assistant",
        "content": if text.is_empty() { Value::Null } else { json!(text.join("\n\n")) },
    });
    if !calls.is_empty() {
        message["tool_calls"] = json!(calls);
    }
    Some(message)
}

/// Lee una respuesta correcta.
pub fn parse_response(text: &str) -> Result<ModelResponse, PortError> {
    let value: Value = serde_json::from_str(text)
        .map_err(|e| PortError::Provider(format!("respuesta ilegible: {e}")))?;
    let choice = &value["choices"][0];
    if choice.is_null() {
        return Err(PortError::Provider(
            "la respuesta no trae ninguna opción".into(),
        ));
    }
    let message = &choice["message"];
    if let Some(refusal) = message["refusal"].as_str().filter(|r| !r.is_empty()) {
        return Err(PortError::Provider(format!(
            "el modelo rechazó la petición: {refusal}"
        )));
    }
    let mut content = Vec::new();
    if let Some(text) = message["content"].as_str().filter(|t| !t.is_empty()) {
        content.push(ContentPart::Text(text.to_string()));
    }
    for (i, call) in message["tool_calls"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        let function = &call["function"];
        // Los argumentos llegan como texto JSON (algunos servidores, como
        // objeto).
        let arguments = match &function["arguments"] {
            Value::String(s) => s.clone(),
            Value::Null => "{}".to_string(),
            other => other.to_string(),
        };
        let id = call["id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("call_{i}"));
        content.push(ContentPart::ToolUse {
            id,
            name: function["name"].as_str().unwrap_or_default().to_string(),
            input_json: arguments,
        });
    }
    let has_calls = content
        .iter()
        .any(|p| matches!(p, ContentPart::ToolUse { .. }));
    let stop = match choice["finish_reason"].as_str().unwrap_or("") {
        "content_filter" => {
            return Err(PortError::Provider(
                "el filtro de contenido del proveedor cortó la respuesta".into(),
            ));
        }
        "tool_calls" | "function_call" => StopReason::ToolUse,
        "length" => StopReason::MaxTokens,
        _ if has_calls => StopReason::ToolUse,
        "stop" | "" => StopReason::EndTurn,
        other => StopReason::Other(other.to_string()),
    };
    let count = |v: &Value| v.as_u64().unwrap_or(0);
    let prompt = count(&value["usage"]["prompt_tokens"]);
    let cached = count(&value["usage"]["prompt_tokens_details"]["cached_tokens"]);
    let usage = Usage {
        input_tokens: prompt.saturating_sub(cached),
        output_tokens: count(&value["usage"]["completion_tokens"]),
        cached_tokens: cached,
    };
    Ok(ModelResponse {
        content,
        stop,
        usage,
    })
}

/// Error claro a partir de una respuesta HTTP fallida.
pub fn error_from(flavor: Flavor, status: u16, body: &str) -> PortError {
    failure(flavor, status, body).error
}

fn failure(flavor: Flavor, status: u16, body: &str) -> http::Failure {
    let parsed: Option<Value> = serde_json::from_str(body).ok();
    let error = parsed.as_ref().map(|v| &v["error"]);
    // Algunos servidores compatibles devuelven `{"error": "texto"}`.
    let message = error
        .and_then(|e| e["message"].as_str().or_else(|| e.as_str()))
        .map(http::scrub_keys)
        .unwrap_or_else(|| {
            let snippet: String = body.chars().take(200).collect();
            if snippet.trim().is_empty() {
                format!("HTTP {status}")
            } else {
                http::scrub_keys(&snippet)
            }
        });
    let code = error.and_then(|e| e["code"].as_str()).unwrap_or("");
    let who = match flavor {
        Flavor::OpenAi => "OpenAI",
        Flavor::Compatible => "El servidor local",
    };
    let (error, retryable) = match status {
        401 => (
            PortError::Auth(format!(
                "{who} la rechaza ({message}). Revísala en Ajustes › Proveedores de IA"
            )),
            false,
        ),
        403 => (
            PortError::Auth(format!("sin permiso para esta petición ({message})")),
            false,
        ),
        // Sin saldo: esperar no sirve.
        429 if code == "insufficient_quota" => (
            PortError::Provider(format!("la cuenta no tiene saldo ({message})")),
            false,
        ),
        429 => (
            PortError::RateLimited(format!(
                "{message}. Espera un poco o revisa los límites de tu cuenta"
            )),
            true,
        ),
        404 => (
            PortError::Invalid(format!("no existe (¿el modelo es correcto?): {message}")),
            false,
        ),
        408 => (PortError::Timeout, true),
        500..=599 => (
            PortError::Unavailable(format!(
                "{who} no responde ahora (HTTP {status}): {message}"
            )),
            true,
        ),
        _ => (
            PortError::Invalid(format!("petición rechazada (HTTP {status}): {message}")),
            http::is_retryable_status(status),
        ),
    };
    http::Failure { error, retryable }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_not_in_debug() {
        let p = OpenAiProvider::new(Secret::new("sk-proj-NO-SE-VE"));
        assert!(!format!("{p:?}").contains("NO-SE-VE"));
        let c = OpenAiCompatibleProvider::new("http://localhost:11434/v1")
            .with_key(Secret::new("sk-local-NO-SE-VE"));
        assert!(!format!("{c:?}").contains("NO-SE-VE"));
    }
}
