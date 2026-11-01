//! `AnthropicProvider` con respuestas JSON grabadas: la traducción del formato
//! común, los errores y los reintentos contra un servidor HTTP falso en
//! 127.0.0.1 (sin salir del equipo). El test con la API de verdad necesita
//! `ANTHROPIC_API_KEY` y va con `#[ignore]`.

use std::sync::Arc;
use std::time::Duration;

mod common;
use common::fake_api;

use lume_agent::anthropic::{
    API_VERSION, AnthropicProvider, FALLBACK_BETA, RetryPolicy, error_from, parse_response,
    request_body,
};
use lume_agent::{ApiRunner, SessionTools};
use lume_app::ports::{
    AgentLimits, AgentTask, CancellationToken, ContentPart, EventSink, Message, ModelRequest,
    PortError, Role, RunEnd, Secret, StopReason, ToolSpec,
};
use lume_app::testing::MemoryStore;
use lume_app::{AgentRunner, ModelProvider, Session};
use lume_domain::{Document, Rgba};
use serde_json::Value;

const TEXT: &str = include_str!("fixtures/anthropic/text.json");
const TOOL_USE: &str = include_str!("fixtures/anthropic/tool_use.json");
const REFUSAL: &str = include_str!("fixtures/anthropic/refusal.json");
const ERROR_401: &str = include_str!("fixtures/anthropic/error_401.json");
const ERROR_429: &str = include_str!("fixtures/anthropic/error_429.json");
const ERROR_529: &str = include_str!("fixtures/anthropic/error_529.json");

/// Clave de mentira para las pruebas (no es una clave real).
const FAKE_KEY: &str = "sk-ant-prueba-no-es-real";

fn draw_spec() -> ToolSpec {
    lume_agent::tools::specs().remove(0)
}

fn request(model: &str, messages: Vec<Message>) -> ModelRequest {
    ModelRequest {
        model: model.into(),
        system: "Eres un artista de pixel art.".into(),
        messages,
        tools: vec![draw_spec()],
        max_tokens: 16_000,
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
            "He dibujado una poción morada con el tapón de corcho y un brillo arriba a la izquierda."
                .into()
        )]
    );
    assert_eq!(
        (
            r.usage.input_tokens,
            r.usage.output_tokens,
            r.usage.cached_tokens
        ),
        (2095, 37, 1800)
    );
}

#[test]
fn a_tool_call_is_read_and_thinking_is_kept_opaque() {
    let r = parse_response(TOOL_USE).unwrap();
    assert_eq!(r.stop, StopReason::ToolUse);
    assert_eq!(r.content.len(), 3);
    let ContentPart::Opaque { provider, json } = &r.content[0] else {
        panic!("el razonamiento se guarda opaco: {:?}", r.content[0]);
    };
    assert_eq!(provider, "anthropic");
    let thinking: Value = serde_json::from_str(json).unwrap();
    assert_eq!(thinking["type"], "thinking");
    assert_eq!(
        thinking["signature"],
        "EqQBCkgIARABGAIiQL2jZ3JlbmRhLXBydWViYQ=="
    );
    assert_eq!(
        r.content[1],
        ContentPart::Text("Empiezo por el cuerpo de la poción.".into())
    );
    let ContentPart::ToolUse {
        id,
        name,
        input_json,
    } = &r.content[2]
    else {
        panic!("se esperaba tool_use");
    };
    assert_eq!(
        (id.as_str(), name.as_str()),
        ("toolu_01A09q90qw90lq917835lq9", "draw")
    );
    let input: Value = serde_json::from_str(input_json).unwrap();
    assert_eq!(
        input["commands"],
        "fill_rect 10 18 12 10 #8b3fe0\nset_pixel 21 24 #5b2aa6"
    );
    // Entrada sin caché + escritura en caché; la lectura de caché aparte.
    assert_eq!(
        (
            r.usage.input_tokens,
            r.usage.output_tokens,
            r.usage.cached_tokens
        ),
        (1520 + 2048, 212, 0)
    );
}

/// La conversación vuelve a la API con el razonamiento intacto, la llamada a
/// la herramienta como objeto y los resultados con imágenes en base64.
#[test]
fn the_conversation_is_sent_back_with_images_and_thinking() {
    let assistant = parse_response(TOOL_USE).unwrap().content;
    let messages = vec![
        Message {
            role: Role::User,
            content: vec![
                ContentPart::Text("Petición: una poción".into()),
                ContentPart::Text(String::new()),
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
                    id: "toolu_01A09q90qw90lq917835lq9".into(),
                    content: vec![
                        ContentPart::Text("Lienzo de 32×32".into()),
                        ContentPart::Image {
                            media_type: "image/png".into(),
                            data: b"PNG-LIENZO".to_vec(),
                        },
                    ],
                    is_error: false,
                },
                ContentPart::ToolResult {
                    id: "toolu_otro".into(),
                    content: vec![ContentPart::Text("err out_of_bounds".into())],
                    is_error: true,
                },
                // De otro proveedor: no se envía.
                ContentPart::Opaque {
                    provider: "openai".into(),
                    json: "{\"type\":\"reasoning\"}".into(),
                },
            ],
        },
    ];
    let body = request_body(&request("claude-opus-5-5", messages), Some("high"));
    let msgs = body["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 3);

    let user = &msgs[0]["content"];
    assert_eq!(user.as_array().unwrap().len(), 2, "sin el texto vacío");
    assert_eq!(user[1]["type"], "image");
    assert_eq!(user[1]["source"]["type"], "base64");
    assert_eq!(user[1]["source"]["media_type"], "image/png");
    assert_eq!(user[1]["source"]["data"], "UE5HLVJFRg==");

    let assistant = &msgs[1];
    assert_eq!(assistant["role"], "assistant");
    assert_eq!(assistant["content"][0]["type"], "thinking");
    assert_eq!(
        assistant["content"][0]["signature"],
        "EqQBCkgIARABGAIiQL2jZ3JlbmRhLXBydWViYQ=="
    );
    assert_eq!(assistant["content"][2]["type"], "tool_use");
    assert_eq!(
        assistant["content"][2]["input"]["commands"],
        "fill_rect 10 18 12 10 #8b3fe0\nset_pixel 21 24 #5b2aa6"
    );

    let results = msgs[2]["content"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["type"], "tool_result");
    assert_eq!(results[0]["tool_use_id"], "toolu_01A09q90qw90lq917835lq9");
    assert_eq!(results[0]["content"][1]["type"], "image");
    assert_eq!(
        results[0]["content"][1]["source"]["data"],
        "UE5HLUxJRU5aTw=="
    );
    assert!(results[0].get("is_error").is_none());
    assert_eq!(results[1]["is_error"], true);
}

#[test]
fn the_request_body_follows_the_current_api() {
    let body = request_body(
        &request("claude-opus-5-5", vec![Message::user_text("hola")]),
        Some("high"),
    );
    assert_eq!(body["model"], "claude-opus-5-5");
    assert_eq!(body["max_tokens"], 16_000);
    assert_eq!(body["system"], "Eres un artista de pixel art.");
    assert_eq!(body["tools"][0]["name"], "draw");
    assert_eq!(body["tools"][0]["input_schema"]["type"], "object");
    assert_eq!(body["tools"][0]["input_schema"]["required"][0], "commands");
    assert_eq!(body["cache_control"]["type"], "ephemeral");
    assert_eq!(body["output_config"]["effort"], "high");
    assert_eq!(body["fallbacks"], "default");
    for removed in ["temperature", "top_p", "top_k", "thinking"] {
        assert!(body.get(removed).is_none(), "{removed}");
    }
    // Haiku no tiene respaldo en el servidor; sin esfuerzo no se envía.
    let body = request_body(
        &request("claude-haiku-5-5", vec![Message::user_text("hola")]),
        None,
    );
    assert!(body.get("fallbacks").is_none());
    assert!(body.get("output_config").is_none());
}

#[test]
fn a_refusal_is_a_clear_error() {
    let err = parse_response(REFUSAL).unwrap_err();
    assert_eq!(
        err.to_string(),
        "error del proveedor: el modelo rechazó la petición (categoría: cyber); prueba a reformularla"
    );
}

/// Clave inválida y límite de uso: errores claros, en español.
#[test]
fn bad_keys_and_rate_limits_are_clear_errors() {
    let err = error_from(401, ERROR_401);
    assert!(matches!(err, PortError::Auth(_)));
    assert_eq!(
        err.to_string(),
        "la clave de API no es válida: Anthropic la rechaza (invalid x-api-key). Revísala en Ajustes › Proveedores de IA"
    );
    let err = error_from(429, ERROR_429);
    assert!(matches!(err, PortError::RateLimited(_)));
    assert!(
        err.to_string()
            .starts_with("el proveedor limita las peticiones: Number of request tokens"),
        "{err}"
    );
    assert!(matches!(
        error_from(529, ERROR_529),
        PortError::Unavailable(_)
    ));
    assert!(matches!(error_from(403, ERROR_401), PortError::Auth(_)));
    assert!(
        matches!(error_from(400, "<html>mal</html>"), PortError::Invalid(m) if m.contains("<html>mal</html>"))
    );
    assert!(matches!(error_from(500, ""), PortError::Unavailable(m) if m.contains("HTTP 500")));
}

// --- HTTP contra un servidor falso ---------------------------------------

fn provider(url: &str) -> AnthropicProvider {
    AnthropicProvider::new(Secret::new(FAKE_KEY))
        .with_base_url(url)
        .with_retry(RetryPolicy {
            max_retries: 2,
            base_delay: Duration::ZERO,
            max_delay: Duration::from_millis(10),
        })
}

#[test]
fn requests_carry_the_key_version_and_beta_headers() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let api = fake_api(&rt, vec![(200, TOOL_USE)]);
    let p = provider(&api.url);
    let response = rt
        .block_on(p.complete(request("claude-opus-5-5", vec![Message::user_text("hola")])))
        .unwrap();
    assert_eq!(response.stop, StopReason::ToolUse);

    let received = api.received.lock().unwrap().clone();
    assert_eq!(received.len(), 1);
    let r = &received[0];
    assert!(r.head.starts_with("POST /v1/messages "), "{}", r.head);
    assert_eq!(r.header("x-api-key").as_deref(), Some(FAKE_KEY));
    assert_eq!(r.header("anthropic-version").as_deref(), Some(API_VERSION));
    assert_eq!(r.header("anthropic-beta").as_deref(), Some(FALLBACK_BETA));
    assert_eq!(r.body["model"], "claude-opus-5-5");
    assert_eq!(r.body["messages"][0]["content"][0]["text"], "hola");
}

#[test]
fn overloaded_and_rate_limited_requests_are_retried() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let api = fake_api(&rt, vec![(529, ERROR_529), (429, ERROR_429), (200, TEXT)]);
    let response = rt
        .block_on(provider(&api.url).complete(request(
            "claude-haiku-5-5",
            vec![Message::user_text("hola")],
        )))
        .unwrap();
    assert_eq!(response.stop, StopReason::EndTurn);
    assert_eq!(api.received.lock().unwrap().len(), 3);
    // Haiku: sin cabecera beta de respaldo.
    assert_eq!(
        api.received.lock().unwrap()[0].header("anthropic-beta"),
        None
    );

    // Siempre limitado: tras los reintentos, error de límite.
    let api = fake_api(&rt, vec![(429, ERROR_429); 5]);
    let err = rt
        .block_on(
            provider(&api.url)
                .complete(request("claude-opus-5-5", vec![Message::user_text("hola")])),
        )
        .unwrap_err();
    assert!(matches!(err, PortError::RateLimited(_)), "{err}");
    assert_eq!(api.received.lock().unwrap().len(), 3, "1 + 2 reintentos");
}

#[test]
fn a_bad_key_is_not_retried() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let api = fake_api(&rt, vec![(401, ERROR_401), (200, TEXT)]);
    let err = rt
        .block_on(
            provider(&api.url)
                .complete(request("claude-opus-5-5", vec![Message::user_text("hola")])),
        )
        .unwrap_err();
    assert!(matches!(err, PortError::Auth(_)), "{err}");
    assert!(!err.to_string().contains(FAKE_KEY));
    assert_eq!(api.received.lock().unwrap().len(), 1);
}

#[test]
fn an_unreachable_api_is_unavailable() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    // Un puerto que se abre y se cierra: nadie escucha.
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let err = rt
        .block_on(
            provider(&format!("http://127.0.0.1:{port}"))
                .complete(request("claude-opus-5-5", vec![Message::user_text("hola")])),
        )
        .unwrap_err();
    assert!(matches!(err, PortError::Unavailable(_)), "{err}");
    assert!(!err.to_string().contains(FAKE_KEY));
}

/// El agente completo (`ApiRunner` + herramientas + sesión) con la API falsa:
/// el modelo pide `draw` y el lienzo cambia.
#[test]
fn the_runner_draws_through_the_anthropic_provider() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let api = fake_api(&rt, vec![(200, TOOL_USE), (200, TEXT)]);
    let session = {
        let _guard = rt.enter();
        Session::spawn(
            Document::new("pocion", 32, 32).unwrap(),
            Arc::new(MemoryStore::new()),
        )
    };
    let tools = Arc::new(SessionTools::new(
        session.handle.clone(),
        Arc::new(MemoryStore::new()),
        3,
    ));
    let task = AgentTask {
        run: 3,
        goal: "una poción".into(),
        context: String::new(),
        attachments: vec![],
        model: "claude-opus-5-5".into(),
        limits: AgentLimits::default(),
        canvas: (32, 32),
        palette: None,
        continuing: false,
        resume: None,
        anim: None,
        vector: None,
    };
    let summary = rt
        .block_on(ApiRunner::new(Arc::new(provider(&api.url))).run(
            task,
            tools,
            EventSink::channel().0,
            CancellationToken::new(),
        ))
        .unwrap();
    assert_eq!((summary.end, summary.steps), (RunEnd::Finished, 2));
    let canvas = session.handle.view().borrow().canvas.clone();
    assert_eq!(canvas.get(21, 24).unwrap(), Rgba::parse("#5b2aa6").unwrap());

    // La segunda petición lleva el razonamiento y el resultado de draw.
    let second = api.received.lock().unwrap()[1].body.clone();
    assert_eq!(second["messages"][1]["content"][0]["type"], "thinking");
    assert_eq!(second["messages"][2]["content"][0]["type"], "tool_result");
    assert_eq!(
        second["messages"][2]["content"][0]["content"][0]["text"],
        "batch: 2 ok, 0 failed"
    );
}

// --- La API de verdad ------------------------------------------------------

/// `ANTHROPIC_API_KEY=… cargo test -p lume-agent --test anthropic -- --ignored`
/// (modelo: `LUMEART_TEST_MODEL` o `claude-opus-5-5`). Gasta unos céntimos.
#[test]
#[ignore = "usa la API de Anthropic (necesita ANTHROPIC_API_KEY)"]
fn the_real_api_draws_a_pixel() {
    let key = std::env::var("ANTHROPIC_API_KEY").expect("falta ANTHROPIC_API_KEY");
    let model = std::env::var("LUMEART_TEST_MODEL").unwrap_or_else(|_| "claude-opus-5-5".into());
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
    let provider = AnthropicProvider::new(Secret::new(key)).with_effort(Some("low"));
    let summary = rt
        .block_on(ApiRunner::new(Arc::new(provider)).run(
            task,
            tools,
            EventSink::channel().0,
            CancellationToken::new(),
        ))
        .unwrap();
    eprintln!("{summary:?}");
    let canvas = session.handle.view().borrow().canvas.clone();
    assert_eq!(canvas.get(0, 0).unwrap(), Rgba::parse("#ff0000").unwrap());
    assert!(summary.usage.total() > 0);
}
