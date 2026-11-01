//! Los dobles en memoria cumplen los contratos de los puertos.

use std::sync::Arc;

use lume_app::DocumentStore;
use lume_app::ports::{
    AgentEvent, AgentLimits, AgentRunner, AgentTask, CancellationToken, ContentPart, EventSink,
    Message, ModelProvider, ModelRequest, ModelResponse, PortError, RasterOptions, Rasterizer,
    RunEnd, ScriptEngine, ScriptKind, ScriptLimits, Secret, SecretStore, StopReason, ToolBox,
    Usage,
};
use lume_app::testing::{
    FakeProvider, FakeRasterizer, FakeScriptEngine, FakeTools, MemorySecretStore, MemoryStore,
    ScriptedRunner, ScriptedStep,
};
use lume_domain::{Canvas, Document, ErrorCode, Rgba};

#[test]
fn memory_store_round_trips() {
    let store = MemoryStore::new();
    let mut c = Canvas::new(2, 2).unwrap();
    c.set(1, 1, Rgba::rgb(1, 2, 3)).unwrap();
    store.export_png(&c, "/a.png".as_ref(), 1).unwrap();
    assert_eq!(store.read_png("/a.png".as_ref()).unwrap(), c);
    store.export_png(&c, "/b.png".as_ref(), 3).unwrap();
    assert_eq!(store.read_png("/b.png".as_ref()).unwrap().width(), 6);
    assert_eq!(
        store.export_png(&c, "/c.png".as_ref(), 0).unwrap_err().code,
        ErrorCode::InvalidArgs
    );

    let doc = Document::new("d", 3, 3).unwrap();
    store.save_document(&doc, "/d.lume".as_ref()).unwrap();
    assert_eq!(store.open_document("/d.lume".as_ref()).unwrap(), doc);
    let err = store.open_document("/nada.lume".as_ref()).unwrap_err();
    assert_eq!(err.message, "open /nada.lume: no such file or directory");

    store.put_pxcli_run("/runs/x", doc.clone());
    assert_eq!(store.import_pxcli_run("/runs/x".as_ref()).unwrap(), doc);
    assert_eq!(
        store.import_pxcli_run("/runs/y".as_ref()).unwrap_err().code,
        ErrorCode::Io
    );
}

#[test]
fn fake_rasterizer_keeps_the_aspect_ratio() {
    let r = FakeRasterizer::default();
    let img = r
        .rasterize(
            "<svg/>",
            200.0,
            100.0,
            &RasterOptions {
                width: Some(64),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        (img.width, img.height, img.rgba.len()),
        (64, 32, 64 * 32 * 4)
    );
    assert!(matches!(
        r.rasterize("hola", 1.0, 1.0, &RasterOptions::default()),
        Err(PortError::Invalid(_))
    ));
}

#[test]
fn fake_script_engine_answers_what_it_was_told() {
    let engine = FakeScriptEngine::default();
    engine.respond("frame", r#"["set_pixel 0 0 red"]"#);
    let mut script = engine
        .load(
            ScriptKind::Animation,
            "action('idle', ...)",
            ScriptLimits::default(),
        )
        .unwrap();
    assert_eq!(
        script.call("frame", r#"["idle", 0]"#).unwrap(),
        r#"["set_pixel 0 0 red"]"#
    );
    assert!(script.call("other", "[]").is_err());
}

#[tokio::test]
async fn fake_provider_replays_responses_and_records_requests() {
    let provider = FakeProvider::new([ModelResponse {
        content: vec![ContentPart::Text("hola".into())],
        stop: StopReason::EndTurn,
        usage: Usage {
            input_tokens: 10,
            output_tokens: 2,
            cached_tokens: 0,
        },
    }]);
    provider.push_error(PortError::RateLimited("espera 30 s".into()));
    let request = ModelRequest {
        model: "m".into(),
        system: "s".into(),
        messages: vec![Message::user_text("dibuja")],
        tools: vec![],
        max_tokens: 100,
    };
    let first = provider.complete(request.clone()).await.unwrap();
    assert_eq!(first.usage.total(), 12);
    assert_eq!(
        provider.complete(request.clone()).await.unwrap_err(),
        PortError::RateLimited("espera 30 s".into())
    );
    assert!(matches!(
        provider.complete(request).await,
        Err(PortError::Provider(_))
    ));
    assert_eq!(provider.requests.lock().unwrap().len(), 3);
    assert_eq!(provider.id(), "fake");
}

fn task(max_steps: u32) -> AgentTask {
    AgentTask {
        run: 1,
        goal: "un cofre".into(),
        context: String::new(),
        attachments: vec![],
        model: "m".into(),
        limits: AgentLimits {
            max_steps,
            max_tokens: 1000,
        },
        canvas: (16, 16),
        palette: None,
        continuing: false,
        resume: None,
        anim: None,
        vector: None,
    }
}

#[tokio::test]
async fn scripted_runner_uses_the_tools_and_reports_events() {
    let tools = Arc::new(FakeTools::default());
    let runner = ScriptedRunner::new(vec![
        ScriptedStep::Say("Empiezo por el contorno.".into()),
        ScriptedStep::Tool {
            name: "draw".into(),
            input_json: r#"{"commands":"fill_rect 0 0 4 4 brown"}"#.into(),
        },
        ScriptedStep::Say("Listo.".into()),
    ]);
    let (sink, mut events) = EventSink::channel();
    let toolbox: ToolBox = tools.clone();
    let summary = runner
        .run(task(10), toolbox, sink, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(summary.steps, 3);
    assert_eq!(summary.end, RunEnd::Finished);
    assert_eq!(summary.final_text, "Listo.");
    assert_eq!(tools.calls.lock().unwrap()[0].0, "draw");
    let mut kinds = Vec::new();
    while let Ok(ev) = events.try_recv() {
        kinds.push(match ev {
            AgentEvent::Text(_) => "text",
            AgentEvent::Thinking(_) => "thinking",
            AgentEvent::ToolCall { .. } => "call",
            AgentEvent::ToolResult { .. } => "result",
            AgentEvent::Usage(_) => "usage",
        });
    }
    assert_eq!(kinds, vec!["text", "call", "result", "text"]);

    // Límite de pasos y cancelación.
    let (sink, _) = EventSink::channel();
    let s = runner
        .run(task(1), tools.clone(), sink, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!((s.steps, s.end), (1, RunEnd::StepLimit));
    let cancel = CancellationToken::new();
    cancel.cancel();
    let (sink, _) = EventSink::channel();
    let s = runner.run(task(10), tools, sink, cancel).await.unwrap();
    assert_eq!((s.steps, s.end), (0, RunEnd::Cancelled));
}

#[test]
fn memory_secret_store_and_secret_never_print_their_value() {
    let secrets = MemorySecretStore::default();
    assert_eq!(secrets.get("anthropic").unwrap(), None);
    secrets
        .set("anthropic", &Secret::new("sk-ant-12345"))
        .unwrap();
    let got = secrets.get("anthropic").unwrap().unwrap();
    assert_eq!(got.expose(), "sk-ant-12345");
    assert_eq!(format!("{got:?}"), "Secret(****)");
    secrets.delete("anthropic").unwrap();
    assert_eq!(secrets.get("anthropic").unwrap(), None);
}

/// La versión enmascarada solo enseña el prefijo público del proveedor.
#[test]
fn masked_secrets_only_show_the_public_prefix() {
    let key = "sk-ant-api03-SECRETO-muy-largo-0123456789";
    let masked = Secret::new(key).masked();
    assert_eq!(masked, format!("sk-ant-{}", "•".repeat(16)));
    assert!(!masked.contains("SECRETO"));
    let masked = Secret::new("sk-proj-abcdefghijklmnopqrstuvwxyz").masked();
    assert_eq!(masked, format!("sk-proj-{}", "•".repeat(16)));
    // Corta o sin prefijo: solo puntos.
    assert_eq!(Secret::new("abc-def").masked(), "•".repeat(16));
    assert_eq!(Secret::new("0123456789abcdefghij").masked(), "•".repeat(16));
}

#[test]
fn memory_settings_store_keeps_what_was_saved() {
    use lume_app::testing::MemorySettingsStore;
    use lume_app::{Settings, SettingsStore};
    let store = MemorySettingsStore::default();
    assert_eq!(store.load().unwrap(), Settings::default());
    let changed = Settings {
        max_turns: 12,
        ..Settings::default()
    };
    store.save(&changed).unwrap();
    assert_eq!(store.load().unwrap(), changed);
    assert_eq!(*store.saves.lock().unwrap(), 1);
}
