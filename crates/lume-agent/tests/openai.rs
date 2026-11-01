//! `OpenAiProvider` y `OpenAiCompatibleProvider` con respuestas grabadas de
//! Chat Completions (OpenAI y Ollama): traducción, errores, dirección y
//! modelo configurables, contra un servidor falso en 127.0.0.1. El test con
//! la API de verdad necesita `OPENAI_API_KEY` y va con `#[ignore]`.

use std::sync::Arc;
use std::time::Duration;

use lume_agent::http::RetryPolicy;
use lume_agent::openai::{Flavor, error_from, parse_response, request_body};
use lume_agent::{ApiRunner, OpenAiCompatibleProvider, OpenAiProvider, SessionTools};
use lume_app::ports::{
    AgentLimits, AgentTask, CancellationToken, ContentPart, EventSink, Message, ModelRequest,
    PortError, Role, RunEnd, Secret, StopReason,
};
use lume_app::testing::MemoryStore;
use lume_app::{AgentRunner, ModelProvider, Session};
use lume_domain::{Document, Rgba};
use serde_json::Value;

mod common;
use common::fake_api;

const TEXT: &str = include_str!("fixtures/openai/text.json");
const TOOL_CALLS: &str = include_str!("fixtures/openai/tool_calls.json");
const REFUSAL: &str = include_str!("fixtures/openai/refusal.json");
const ERROR_401: &str = include_str!("fixtures/openai/error_401.json");
const ERROR_429: &str = include_str!("fixtures/openai/error_429.json");
const ERROR_QUOTA: &str = include_str!("fixtures/openai/error_quota.json");
const OLLAMA_TOOL_CALLS: &str = include_str!("fixtures/openai/ollama_tool_calls.json");
const OLLAMA_TEXT: &str = include_str!("fixtures/openai/ollama_text.json");
const OLLAMA_404: &str = include_str!("fixtures/openai/ollama_404.json");

/// Clave de mentira (no es una clave real).
const FAKE_KEY: &str = "sk-proj-prueba-no-es-real";

fn request(model: &str, messages: Vec<Message>) -> ModelRequest {
    ModelRequest {
        model: model.into(),
        system: "Eres un artista de pixel art.".into(),
        messages,
        tools: lume_agent::tools::specs(),
        max_tokens: 16_000,
    }
}

fn fast() -> RetryPolicy {
    RetryPolicy {
        max_retries: 2,
        base_delay: Duration::ZERO,
        max_delay: Duration::from_millis(10),
    }
}

// --- Traducción ---------------------------------------------------------

#[test]
fn a_text_response_is_read() {
    let r = parse_response(TEXT).unwrap();
    assert_eq!(r.stop, StopReason::EndTurn);
    assert_eq!(
        r.content,
        vec![ContentPart::Text(
            "He dibujado una poción morada con el tapón de corcho.".into()
        )]
    );
    // La parte en caché se cuenta aparte.
    assert_eq!(
        (
            r.usage.input_tokens,
            r.usage.output_tokens,
            r.usage.cached_tokens
        ),
        (2117 - 1024, 46, 1024)
    );
}

#[test]
fn tool_calls_are_read() {
    let r = parse_response(TOOL_CALLS).unwrap();
    assert_eq!(r.stop, StopReason::ToolUse);
    assert_eq!(r.content.len(), 2, "sin texto: content es null");
    let ContentPart::ToolUse {
        id,
        name,
        input_json,
    } = &r.content[0]
    else {
        panic!("se esperaba una llamada");
    };
    assert_eq!(
        (id.as_str(), name.as_str()),
        ("call_9pw1qnYScqvGrCH58HWCvFH6", "draw")
    );
    let input: Value = serde_json::from_str(input_json).unwrap();
    assert_eq!(
        input["commands"],
        "fill_rect 10 18 12 10 #8b3fe0\nset_pixel 21 24 #5b2aa6"
    );
    assert!(matches!(&r.content[1], ContentPart::ToolUse { name, .. } if name == "view_canvas"));

    // Ollama: `content` vacío y una llamada.
    let r = parse_response(OLLAMA_TOOL_CALLS).unwrap();
    assert_eq!(r.stop, StopReason::ToolUse);
    assert_eq!(r.content.len(), 1);
    assert_eq!((r.usage.input_tokens, r.usage.cached_tokens), (412, 0));
}

/// La conversación se traduce: sistema, usuario con imagen, asistente con
/// llamadas, un mensaje `tool` por resultado y las imágenes de los
/// resultados en un mensaje de usuario detrás.
#[test]
fn the_conversation_is_translated_with_images() {
    let assistant = parse_response(TOOL_CALLS).unwrap().content;
    let messages = vec![
        Message {
            role: Role::User,
            content: vec![
                ContentPart::Text("Petición: una poción".into()),
                ContentPart::Text("Imagen de referencia".into()),
                ContentPart::Image {
                    media_type: "image/png".into(),
                    data: b"PNG-REF".to_vec(),
                },
            ],
        },
        Message {
            role: Role::Assistant,
            content: assistant,
        },
        Message {
            role: Role::User,
            content: vec![
                ContentPart::ToolResult {
                    id: "call_9pw1qnYScqvGrCH58HWCvFH6".into(),
                    content: vec![ContentPart::Text("batch: 2 ok, 0 failed".into())],
                    is_error: false,
                },
                ContentPart::ToolResult {
                    id: "call_ZJ7c3Rzqf0aVKXy1t6PNGx1d".into(),
                    content: vec![
                        ContentPart::Text("Lienzo de 32×32".into()),
                        ContentPart::Image {
                            media_type: "image/png".into(),
                            data: b"PNG-LIENZO".to_vec(),
                        },
                    ],
                    is_error: false,
                },
                // De Anthropic: aquí no se envía.
                ContentPart::Opaque {
                    provider: "anthropic".into(),
                    json: "{\"type\":\"thinking\"}".into(),
                },
            ],
        },
    ];
    let body = request_body(&request("gpt-5", messages), Flavor::OpenAi);
    let msgs = body["messages"].as_array().unwrap();
    let roles: Vec<&str> = msgs.iter().map(|m| m["role"].as_str().unwrap()).collect();
    assert_eq!(
        roles,
        ["system", "user", "assistant", "tool", "tool", "user"]
    );

    assert_eq!(msgs[0]["content"], "Eres un artista de pixel art.");
    let user = msgs[1]["content"].as_array().unwrap();
    assert_eq!(
        user[0],
        serde_json::json!({"type": "text", "text": "Petición: una poción"})
    );
    assert_eq!(user[2]["type"], "image_url");
    assert_eq!(
        user[2]["image_url"]["url"],
        "data:image/png;base64,UE5HLVJFRg=="
    );

    assert!(msgs[2]["content"].is_null());
    assert_eq!(msgs[2]["tool_calls"][0]["type"], "function");
    assert_eq!(msgs[2]["tool_calls"][0]["function"]["name"], "draw");
    assert!(msgs[2]["tool_calls"][0]["function"]["arguments"].is_string());

    assert_eq!(msgs[3]["tool_call_id"], "call_9pw1qnYScqvGrCH58HWCvFH6");
    assert_eq!(msgs[3]["content"], "batch: 2 ok, 0 failed");
    assert_eq!(msgs[4]["tool_call_id"], "call_ZJ7c3Rzqf0aVKXy1t6PNGx1d");
    assert!(
        msgs[4]["content"]
            .as_str()
            .unwrap()
            .starts_with("Lienzo de 32×32")
    );
    let images = msgs[5]["content"].as_array().unwrap();
    assert_eq!(
        images[0]["text"],
        "Imagen de call_ZJ7c3Rzqf0aVKXy1t6PNGx1d:"
    );
    assert_eq!(
        images[1]["image_url"]["url"],
        "data:image/png;base64,UE5HLUxJRU5aTw=="
    );
    assert!(!body.to_string().contains("thinking"));
}

#[test]
fn the_request_body_uses_functions_and_the_right_token_limit() {
    let body = request_body(
        &request("gpt-5", vec![Message::user_text("hola")]),
        Flavor::OpenAi,
    );
    assert_eq!(body["model"], "gpt-5");
    assert_eq!(body["max_completion_tokens"], 16_000);
    assert!(body.get("max_tokens").is_none());
    assert_eq!(
        body["messages"][1]["content"], "hola",
        "texto plano sin imágenes"
    );
    let names: Vec<&str> = body["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["function"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, lume_agent::TOOL_NAMES);
    assert_eq!(body["tools"][0]["type"], "function");
    assert_eq!(
        body["tools"][0]["function"]["parameters"]["required"][0],
        "commands"
    );
    for absent in ["temperature", "top_p"] {
        assert!(body.get(absent).is_none());
    }
    // Compatibles: `max_tokens`, que entienden todos.
    let body = request_body(
        &request("qwen2.5vl:7b", vec![Message::user_text("hola")]),
        Flavor::Compatible,
    );
    assert_eq!(body["max_tokens"], 16_000);
    assert!(body.get("max_completion_tokens").is_none());
}

#[test]
fn refusals_are_clear_errors() {
    let err = parse_response(REFUSAL).unwrap_err();
    assert_eq!(
        err.to_string(),
        "error del proveedor: el modelo rechazó la petición: I can't help with that request."
    );
}

/// Clave inválida, límite de uso y falta de saldo: errores claros y sin la
/// clave (OpenAI la repite enmascarada en el mensaje).
#[test]
fn errors_are_clear_and_never_show_the_key() {
    let err = error_from(Flavor::OpenAi, 401, ERROR_401);
    assert!(matches!(err, PortError::Auth(_)));
    let text = err.to_string();
    assert!(
        text.starts_with(
            "la clave de API no es válida: OpenAI la rechaza (Incorrect API key provided: [clave]"
        ),
        "{text}"
    );
    assert!(!text.contains("sk-"), "{text}");
    assert!(
        text.ends_with("Revísala en Ajustes › Proveedores de IA"),
        "{text}"
    );

    let err = error_from(Flavor::OpenAi, 429, ERROR_429);
    assert!(matches!(err, PortError::RateLimited(_)), "{err}");
    let err = error_from(Flavor::OpenAi, 429, ERROR_QUOTA);
    assert!(
        matches!(err, PortError::Provider(ref m) if m.starts_with("la cuenta no tiene saldo")),
        "{err}"
    );
    let err = error_from(Flavor::Compatible, 404, OLLAMA_404);
    assert!(
        matches!(err, PortError::Invalid(ref m) if m.contains("qwen9")),
        "{err}"
    );
    let err = error_from(
        Flavor::Compatible,
        500,
        r#"{"error":"llama runner crashed"}"#,
    );
    assert!(
        matches!(err, PortError::Unavailable(ref m) if m.contains("El servidor local") && m.contains("llama runner crashed")),
        "{err}"
    );
}

// --- HTTP ---------------------------------------------------------------

#[test]
fn openai_requests_use_the_key_and_a_configurable_base_url() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let api = fake_api(&rt, vec![(200, TEXT)]);
    let p = OpenAiProvider::new(Secret::new(FAKE_KEY))
        .with_base_url(&format!("{}/v1", api.url))
        .with_retry(fast());
    assert_eq!(p.id(), "openai");
    let r = rt
        .block_on(p.complete(request("gpt-5", vec![Message::user_text("hola")])))
        .unwrap();
    assert_eq!(r.stop, StopReason::EndTurn);
    let got = api.received.lock().unwrap()[0].clone();
    assert!(
        got.head.starts_with("POST /v1/chat/completions "),
        "{}",
        got.head
    );
    assert_eq!(
        got.header("authorization"),
        Some(format!("Bearer {FAKE_KEY}"))
    );
    assert_eq!(got.body["model"], "gpt-5");
}

/// Un servidor local: sin clave, con su dirección y su modelo por defecto.
#[test]
fn compatible_servers_take_a_base_url_and_a_model() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let api = fake_api(&rt, vec![(200, OLLAMA_TEXT), (200, OLLAMA_TEXT)]);
    let p = OpenAiCompatibleProvider::new(&format!("{}/v1/", api.url))
        .with_default_model("qwen2.5vl:7b")
        .with_retry(fast());
    assert_eq!(p.id(), "openai-compatible");
    // Sin modelo en la petición: el de la configuración.
    rt.block_on(p.complete(request("", vec![Message::user_text("hola")])))
        .unwrap();
    // Con modelo: manda la petición.
    rt.block_on(p.complete(request("llava:13b", vec![Message::user_text("hola")])))
        .unwrap();
    let got = api.received.lock().unwrap().clone();
    assert!(
        got[0].head.starts_with("POST /v1/chat/completions "),
        "{}",
        got[0].head
    );
    assert_eq!(got[0].header("authorization"), None);
    assert_eq!(got[0].body["model"], "qwen2.5vl:7b");
    assert_eq!(got[1].body["model"], "llava:13b");

    // Sin modelo en ningún sitio: error claro, sin llamar al servidor.
    let bare = OpenAiCompatibleProvider::new(&api.url);
    let err = rt
        .block_on(bare.complete(request("", vec![Message::user_text("hola")])))
        .unwrap_err();
    assert_eq!(
        err,
        PortError::Invalid("falta el modelo: elige uno en Ajustes".into())
    );
    assert_eq!(api.received.lock().unwrap().len(), 2);

    // Con clave, si el servidor la pide.
    let api = fake_api(&rt, vec![(200, OLLAMA_TEXT)]);
    let p = OpenAiCompatibleProvider::new(&api.url)
        .with_key(Secret::new("clave-local"))
        .with_retry(fast());
    rt.block_on(p.complete(request("m", vec![Message::user_text("hola")])))
        .unwrap();
    assert_eq!(
        api.received.lock().unwrap()[0]
            .header("authorization")
            .as_deref(),
        Some("Bearer clave-local")
    );
}

#[test]
fn retries_and_errors_over_http() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    // 503 y 429 se reintentan.
    let api = fake_api(&rt, vec![(503, "{}"), (429, ERROR_429), (200, TEXT)]);
    let p = OpenAiProvider::new(Secret::new(FAKE_KEY))
        .with_base_url(&api.url)
        .with_retry(fast());
    rt.block_on(p.complete(request("gpt-5", vec![Message::user_text("hola")])))
        .unwrap();
    assert_eq!(api.received.lock().unwrap().len(), 3);

    // Sin saldo y clave mala: sin reintentos.
    for (status, body) in [(429, ERROR_QUOTA), (401, ERROR_401)] {
        let api = fake_api(&rt, vec![(status, body), (200, TEXT)]);
        let p = OpenAiProvider::new(Secret::new(FAKE_KEY))
            .with_base_url(&api.url)
            .with_retry(fast());
        let err = rt
            .block_on(p.complete(request("gpt-5", vec![Message::user_text("hola")])))
            .unwrap_err();
        assert!(!err.to_string().contains(FAKE_KEY));
        assert_eq!(api.received.lock().unwrap().len(), 1, "{err}");
    }
}

/// El agente completo con un servidor compatible (como Ollama): pide `draw` y
/// el lienzo cambia.
#[test]
fn the_runner_draws_through_a_compatible_server() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let api = fake_api(&rt, vec![(200, OLLAMA_TOOL_CALLS), (200, OLLAMA_TEXT)]);
    let session = {
        let _guard = rt.enter();
        Session::spawn(
            Document::new("prueba", 4, 4).unwrap(),
            Arc::new(MemoryStore::new()),
        )
    };
    let tools = Arc::new(SessionTools::new(
        session.handle.clone(),
        Arc::new(MemoryStore::new()),
        2,
    ));
    let task = AgentTask {
        run: 2,
        goal: "un píxel rojo".into(),
        context: String::new(),
        attachments: vec![],
        model: "qwen2.5vl:7b".into(),
        limits: AgentLimits::default(),
        canvas: (4, 4),
        palette: None,
        continuing: false,
        resume: None,
        anim: None,
        vector: None,
    };
    let provider = OpenAiCompatibleProvider::new(&format!("{}/v1", api.url)).with_retry(fast());
    let summary = rt
        .block_on(ApiRunner::new(Arc::new(provider)).run(
            task,
            tools,
            EventSink::channel().0,
            CancellationToken::new(),
        ))
        .unwrap();
    assert_eq!((summary.end, summary.steps), (RunEnd::Finished, 2));
    assert_eq!(
        summary.final_text,
        "Listo: un píxel rojo arriba a la izquierda."
    );
    let canvas = session.handle.view().borrow().canvas.clone();
    assert_eq!(canvas.get(0, 0).unwrap(), Rgba::parse("#ff0000").unwrap());
    let second = api.received.lock().unwrap()[1].body.clone();
    assert_eq!(second["messages"][3]["role"], "tool");
    assert_eq!(second["messages"][3]["tool_call_id"], "call_k2v8sl1x");
    assert_eq!(second["messages"][3]["content"], "batch: 1 ok, 0 failed");
}

/// `OPENAI_API_KEY=… cargo test -p lume-agent --test openai -- --ignored`
/// (modelo: `LUMEART_TEST_MODEL` o `gpt-5`). Gasta unos céntimos.
#[test]
#[ignore = "usa la API de OpenAI (necesita OPENAI_API_KEY)"]
fn the_real_api_draws_a_pixel() {
    let key = std::env::var("OPENAI_API_KEY").expect("falta OPENAI_API_KEY");
    let model = std::env::var("LUMEART_TEST_MODEL").unwrap_or_else(|_| "gpt-5".into());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let session = {
        let _guard = rt.enter();
        Session::spawn(
            Document::new("prueba", 4, 4).unwrap(),
            Arc::new(MemoryStore::new()),
        )
    };
    let tools = Arc::new(SessionTools::new(
        session.handle.clone(),
        Arc::new(MemoryStore::new()),
        1,
    ));
    let task = AgentTask {
        run: 1,
        goal: "Pinta solo el píxel (0, 0) de rojo #ff0000 con una llamada a draw y termina sin revisar.".into(),
        context: String::new(),
        attachments: vec![],
        model,
        limits: AgentLimits {
            max_steps: 6,
            max_tokens: 200_000,
        },
        canvas: (4, 4),
        palette: None,
        continuing: false,
        resume: None,
        anim: None,
        vector: None,
    };
    let summary = rt
        .block_on(
            ApiRunner::new(Arc::new(OpenAiProvider::new(Secret::new(key)))).run(
                task,
                tools,
                EventSink::channel().0,
                CancellationToken::new(),
            ),
        )
        .unwrap();
    eprintln!("{summary:?}");
    let canvas = session.handle.view().borrow().canvas.clone();
    assert_eq!(canvas.get(0, 0).unwrap(), Rgba::parse("#ff0000").unwrap());
}
