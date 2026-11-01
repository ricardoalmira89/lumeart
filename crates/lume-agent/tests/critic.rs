//! El crítico: veredicto con el esquema de pxcli, solo lectura (no puede
//! cambiar el lienzo) y, si su respuesta no se entiende, «sin veredicto»
//! sin romper nada.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use lume_agent::critic::critic_task;
use lume_agent::{ApiRunner, ClaudeCodeRunner, SessionTools, Verdict, review};
use lume_app::Session;
use lume_app::ports::{
    AgentLimits, AgentTask, CancellationToken, ContentPart, EventSink, ModelResponse, PortError,
    PreparedAgent, StopReason, Usage,
};
use lume_app::testing::{FakeProvider, MemoryStore};
#[cfg(unix)]
use lume_domain::Canvas;
use lume_domain::{Document, Origin};

const VERDICT: &str = "Hay un problema.\n```json\n{\"aprobado\": false, \"problemas\": [{\"gravedad\": \"alta\", \"zona\": [0, 0, 4, 4], \"problema\": \"falta el tapón\", \"arreglo\": \"añade un tapón marrón\"}, {\"gravedad\": \"baja\", \"problema\": \"el brillo es pequeño\"}]}\n```";

fn session() -> (tokio::runtime::Runtime, Session) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let session = {
        let _guard = rt.enter();
        Session::spawn(
            Document::new("pocion", 8, 8).unwrap(),
            Arc::new(MemoryStore::new()),
        )
    };
    rt.block_on(
        session
            .handle
            .line("fill_rect 2 2 4 4 #8b3fe0", Origin::User, None),
    )
    .unwrap();
    (rt, session)
}

fn artist_task() -> AgentTask {
    AgentTask {
        run: 3,
        goal: "una poción morada con tapón".into(),
        context: "Notas: la vuelta anterior quedó sin brillo.".into(),
        attachments: vec![],
        model: "m".into(),
        limits: AgentLimits::default(),
        canvas: (8, 8),
        palette: None,
        continuing: true,
        resume: None,
        anim: None,
        vector: None,
    }
}

fn reply(content: Vec<ContentPart>) -> ModelResponse {
    let stop = if content
        .iter()
        .any(|p| matches!(p, ContentPart::ToolUse { .. }))
    {
        StopReason::ToolUse
    } else {
        StopReason::EndTurn
    };
    ModelResponse {
        content,
        stop,
        usage: Usage::default(),
    }
}

fn tool(id: &str, name: &str, input: &str) -> ContentPart {
    ContentPart::ToolUse {
        id: id.into(),
        name: name.into(),
        input_json: input.into(),
    }
}

fn critic(provider: Arc<FakeProvider>, session: &Session) -> PreparedAgent {
    PreparedAgent {
        runner: Arc::new(ApiRunner::new(provider).as_critic()),
        // Las herramientas completas: `review` las deja en solo lectura.
        tools: Arc::new(SessionTools::new(
            session.handle.clone(),
            Arc::new(MemoryStore::new()),
            3,
        )),
        model: "m".into(),
    }
}

/// El crítico ve el dibujo, intenta dibujar y deshacer (no puede) y da su
/// veredicto con el esquema de pxcli.
#[test]
fn the_critic_reads_but_cannot_touch_the_canvas() {
    let (rt, s) = session();
    let before = s.handle.view().borrow().clone();
    let provider = Arc::new(FakeProvider::new([
        reply(vec![
            tool("t1", "view_canvas", "{}"),
            tool("t2", "draw", r#"{"commands":"clear #ff0000"}"#),
            tool("t3", "undo", "{}"),
        ]),
        reply(vec![ContentPart::Text(VERDICT.into())]),
    ]));
    let verdict = rt.block_on(review(
        critic(provider.clone(), &s),
        critic_task(&artist_task()),
        EventSink::channel().0,
        CancellationToken::new(),
    ));
    let Verdict::Ready(critique) = verdict else {
        panic!("se esperaba un veredicto: {verdict:?}");
    };
    assert!(!critique.approved);
    assert_eq!(critique.issues.len(), 2);
    assert_eq!(critique.serious().len(), 1);
    assert_eq!(critique.serious()[0].problem, "falta el tapón");

    // El lienzo y el historial siguen igual.
    let after = s.handle.view().borrow().clone();
    assert_eq!(after.canvas, before.canvas);
    assert_eq!(after.history, before.history);

    let requests = provider.requests.lock().unwrap().clone();
    let tools: Vec<&str> = requests[0].tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(tools, ["view_canvas", "get_pixel", "palette"]);
    assert_eq!(requests[0].system, lume_agent::prompts::critic_system());
    let ContentPart::Text(first) = &requests[0].messages[0].content[0] else {
        panic!()
    };
    assert!(
        first.starts_with("Revisa este dibujo de 8x8 píxeles"),
        "{first}"
    );
    assert!(
        first.contains("Encargo original: una poción morada con tapón"),
        "{first}"
    );
    assert!(first.contains("`view_canvas`"), "{first}");
    assert!(
        !first.contains("Notas"),
        "sin el estado del artista: {first}"
    );
    // Los resultados de draw y undo fueron errores para el modelo.
    let results = &requests[1].messages[2].content;
    let errors: Vec<bool> = results
        .iter()
        .map(|p| matches!(p, ContentPart::ToolResult { is_error: true, .. }))
        .collect();
    assert_eq!(errors, [false, true, true]);
}

/// Un JSON que no se entiende (o un crítico que falla) da «sin veredicto»,
/// no un error.
#[test]
fn an_unreadable_verdict_is_no_verdict() {
    let (rt, s) = session();
    let provider = Arc::new(FakeProvider::new([reply(vec![ContentPart::Text(
        "Me parece bien, aunque el tapón… ```json\n{\"aprobado\": \"quizá\"}\n```".into(),
    )])]));
    let verdict = rt.block_on(review(
        critic(provider, &s),
        critic_task(&artist_task()),
        EventSink::channel().0,
        CancellationToken::new(),
    ));
    assert_eq!(
        verdict,
        Verdict::None {
            reason: "el crítico no devolvió su veredicto en JSON".into()
        }
    );

    let failing = FakeProvider::default();
    failing.push_error(PortError::RateLimited("demasiadas".into()));
    let verdict = rt.block_on(review(
        critic(Arc::new(failing), &s),
        critic_task(&artist_task()),
        EventSink::channel().0,
        CancellationToken::new(),
    ));
    assert!(
        matches!(verdict, Verdict::None { ref reason } if reason.starts_with("el crítico falló")),
        "{verdict:?}"
    );
}

// --- Claude Code como crítico ----------------------------------------------

fn claude_critic(claude: Option<PathBuf>, root: &Path) -> ClaudeCodeRunner {
    ClaudeCodeRunner::new(
        claude,
        "/usr/bin/lumeart".into(),
        root.join("session.json"),
        root.join("trabajo"),
    )
    .as_critic()
}

/// Como crítico, Claude Code solo tiene `Read`: ni Bash ni `lumeart`.
#[test]
fn the_claude_code_critic_only_reads() {
    let root = tempfile::tempdir().unwrap();
    let args = claude_critic(None, root.path()).args(&artist_task());
    let after = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .map(|i| args[i + 1].clone())
    };
    assert_eq!(after("--tools").as_deref(), Some("Read"));
    assert_eq!(after("--allowedTools").as_deref(), Some("Read"));
    assert!(!args.iter().any(|a| a.contains("Bash")), "{args:?}");
}

#[cfg(unix)]
#[test]
fn the_claude_code_critic_looks_at_a_png_and_answers() {
    use std::os::unix::fs::PermissionsExt;
    let (rt, s) = session();
    let root = tempfile::tempdir().unwrap();
    let result = serde_json::json!({
        "type": "result", "subtype": "success", "is_error": false, "num_turns": 2,
        "result": VERDICT, "usage": {}
    });
    let claude = root.path().join("claude");
    std::fs::write(
        &claude,
        format!(
            "#!/bin/sh\ncat > prompt.txt\nprintf '%s\\n' \"$@\" > args.txt\nprintf '%s\\n' '{}'\n",
            result.to_string().replace('\'', "'\\''")
        ),
    )
    .unwrap();
    std::fs::set_permissions(&claude, std::fs::Permissions::from_mode(0o755)).unwrap();
    let runner = claude_critic(Some(claude), root.path());
    let dir = runner.run_dir(&artist_task());
    let prepared = PreparedAgent {
        runner: Arc::new(runner),
        tools: Arc::new(SessionTools::new(
            s.handle.clone(),
            Arc::new(MemoryStore::new()),
            3,
        )),
        model: String::new(),
    };
    let verdict = rt.block_on(review(
        prepared,
        critic_task(&artist_task()),
        EventSink::channel().0,
        CancellationToken::new(),
    ));
    assert!(
        matches!(&verdict, Verdict::Ready(c) if c.serious().len() == 1),
        "{verdict:?}"
    );
    // Miró el dibujo en un PNG de su carpeta (el de `view_canvas`).
    let png = std::fs::read(dir.join("dibujo.png")).unwrap();
    assert!(!png.is_empty());
    let prompt = std::fs::read_to_string(dir.join("prompt.txt")).unwrap();
    assert!(prompt.contains("./dibujo.png"), "{prompt}");
    assert!(dir.ends_with("run-3-critica"));
    // Y el lienzo, intacto.
    let mut expected = Canvas::new(8, 8).unwrap();
    expected
        .fill_rect(2, 2, 4, 4, lume_domain::Rgba::parse("#8b3fe0").unwrap())
        .unwrap();
    assert_eq!(*s.handle.view().borrow().canvas, expected);
}
