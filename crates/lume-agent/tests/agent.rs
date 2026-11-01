//! El núcleo del agente: `ApiRunner` con `FakeProvider` (respuestas grabadas,
//! sin red) y las herramientas contra una `Session` de verdad.

use std::sync::Arc;
use std::time::Duration;

use lume_agent::{ApiRunner, SessionTools, TOOL_NAMES};
use lume_app::ports::{
    AgentEvent, AgentLimits, AgentTask, BoxFuture, CancellationToken, ContentPart, EventSink,
    ModelRequest, ModelResponse, PortError, RunEnd, StopReason, ToolBox, Usage,
};
use lume_app::testing::{FakeProvider, MemoryStore};
use lume_app::{AgentRunner, ModelProvider, Session};
use lume_domain::{Canvas, Document, Origin, Palette};

const RUN: u64 = 7;

fn session(doc: Document) -> (tokio::runtime::Runtime, Session) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let session = {
        let _guard = runtime.enter();
        Session::spawn(doc, Arc::new(MemoryStore::new()))
    };
    (runtime, session)
}

fn tools(session: &Session) -> ToolBox {
    Arc::new(SessionTools::new(
        session.handle.clone(),
        Arc::new(MemoryStore::new()),
        RUN,
    ))
}

fn task(max_steps: u32) -> AgentTask {
    AgentTask {
        run: RUN,
        goal: "Dibuja una poción".into(),
        context: "Notas: nada todavía.".into(),
        attachments: vec![],
        model: "modelo-de-prueba".into(),
        limits: AgentLimits {
            max_steps,
            max_tokens: 1_000_000,
        },
        canvas: (8, 8),
        palette: None,
        continuing: false,
        resume: None,
        anim: None,
        vector: None,
    }
}

fn tool_use(id: &str, name: &str, input: &str) -> ContentPart {
    ContentPart::ToolUse {
        id: id.into(),
        name: name.into(),
        input_json: input.into(),
    }
}

fn draw(id: &str, commands: &str) -> ContentPart {
    let input = serde_json::json!({ "commands": commands }).to_string();
    tool_use(id, "draw", &input)
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
        usage: Usage {
            input_tokens: 100,
            output_tokens: 20,
            cached_tokens: 0,
        },
    }
}

fn text(t: &str) -> ContentPart {
    ContentPart::Text(t.into())
}

/// El lienzo que quedaría aplicando estas líneas a un documento vacío.
fn expected(w: u32, h: u32, lines: &[&str]) -> Canvas {
    let mut doc = Document::new("esperado", i64::from(w), i64::from(h)).unwrap();
    for line in lines {
        let _ = doc.handle_line(line, Origin::User);
    }
    doc.canvas().clone()
}

fn canvas(session: &Session) -> Canvas {
    (*session.handle.view().borrow().canvas).clone()
}

const DRAW_A: &str = "fill_rect 0 0 8 8 #1d2b53\nset_pixel 3 3 #ff004d";
const DRAW_B: &str = "line 0 7 7 7 #00e436; set_pixel 4 4 #ffec27";

/// Un guion de llamadas a `draw` (más `view_canvas`) deja el lienzo esperado;
/// el modelo recibe los resultados y las herramientas.
#[test]
fn a_scripted_run_draws_the_expected_canvas() {
    let (rt, s) = session(Document::new("pocion", 8, 8).unwrap());
    let provider = Arc::new(FakeProvider::new([
        reply(vec![text("Empiezo por el fondo."), draw("t1", DRAW_A)]),
        reply(vec![
            draw("t2", DRAW_B),
            tool_use("t3", "view_canvas", "{}"),
        ]),
        reply(vec![text("He dibujado la poción.")]),
    ]));
    let runner = ApiRunner::new(provider.clone());
    let (sink, mut rx) = EventSink::channel();
    let summary = rt
        .block_on(runner.run(task(10), tools(&s), sink, CancellationToken::new()))
        .unwrap();

    assert_eq!(summary.end, RunEnd::Finished);
    assert_eq!(summary.steps, 3);
    assert_eq!(summary.final_text, "He dibujado la poción.");
    assert_eq!(summary.usage.total(), 360);
    let all: Vec<&str> = DRAW_A.lines().chain(DRAW_B.split(';')).collect();
    assert_eq!(canvas(&s), expected(8, 8, &all));

    // Lo que vio el modelo.
    let requests = provider.requests.lock().unwrap().clone();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].model, "modelo-de-prueba");
    let names: Vec<&str> = requests[0].tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, TOOL_NAMES);
    // Instrucciones y encargo salen de `prompts/`.
    assert_eq!(
        requests[0].system,
        lume_agent::prompts::pixel_system(&task(10))
    );
    assert!(requests[0].system.contains("Tamaño 8x8."));
    let first = &requests[0].messages[0].content[0];
    assert_eq!(
        first,
        &ContentPart::Text(
            "Petición: Dibuja una poción\n\nLienzo: 8x8 píxeles.\n\nNotas: nada todavía.".into()
        )
    );
    let ContentPart::ToolResult {
        id,
        content,
        is_error,
    } = &requests[1].messages[2].content[0]
    else {
        panic!("se esperaba el resultado de draw");
    };
    assert_eq!(id, "t1");
    assert!(!is_error);
    assert_eq!(
        content,
        &vec![ContentPart::Text("batch: 2 ok, 0 failed".into())]
    );
    // `view_canvas` devolvió una imagen.
    let ContentPart::ToolResult { content, .. } = &requests[2].messages[4].content[1] else {
        panic!("se esperaba el resultado de view_canvas");
    };
    assert!(
        content.iter().any(
            |p| matches!(p, ContentPart::Image { media_type, .. } if media_type == "image/png")
        )
    );

    // Eventos para la interfaz, en orden.
    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }
    let calls: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::ToolCall { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(calls, ["draw", "draw", "view_canvas"]);
    assert_eq!(
        events.first(),
        Some(&AgentEvent::Usage(Usage {
            input_tokens: 100,
            output_tokens: 20,
            cached_tokens: 0
        }))
    );
    assert!(events.contains(&AgentEvent::Text("He dibujado la poción.".into())));
}

/// Cada `draw` del agente es un paso del historial con `Origin::Agent` y se
/// deshace de uno en uno.
#[test]
fn agent_commands_are_in_the_history_and_undo_per_draw() {
    let (rt, s) = session(Document::new("pocion", 8, 8).unwrap());
    let provider = Arc::new(FakeProvider::new([
        reply(vec![draw("t1", DRAW_A)]),
        reply(vec![draw("t2", DRAW_B)]),
        reply(vec![text("Listo.")]),
    ]));
    rt.block_on(ApiRunner::new(provider).run(
        task(10),
        tools(&s),
        EventSink::channel().0,
        CancellationToken::new(),
    ))
    .unwrap();

    let history = s.handle.view().borrow().history.clone();
    let agent: Vec<_> = history
        .iter()
        .filter(|h| h.origin == Origin::Agent { run: RUN })
        .collect();
    assert_eq!(agent.len(), 2, "{history:?}");
    assert_eq!(history.len(), 2, "solo los pasos del agente");

    rt.block_on(s.handle.line("undo", Origin::User, None))
        .unwrap();
    assert_eq!(
        canvas(&s),
        expected(8, 8, &DRAW_A.lines().collect::<Vec<_>>())
    );
    rt.block_on(s.handle.line("undo", Origin::User, None))
        .unwrap();
    assert_eq!(canvas(&s), Canvas::new(8, 8).unwrap());
}

/// Un proveedor que pide dibujar y cancela la ejecución al responder.
struct CancelWhileAnswering {
    cancel: CancellationToken,
}

impl ModelProvider for CancelWhileAnswering {
    fn id(&self) -> &str {
        "cancela"
    }

    fn complete<'a>(
        &'a self,
        _request: ModelRequest,
    ) -> BoxFuture<'a, Result<ModelResponse, PortError>> {
        // Se cancela justo cuando llega la respuesta (antes de ejecutar la
        // herramienta que pide).
        Box::pin(async move {
            self.cancel.cancel();
            Ok(reply(vec![draw("t1", DRAW_A)]))
        })
    }
}

/// Un proveedor que nunca responde.
struct Silent;

impl ModelProvider for Silent {
    fn id(&self) -> &str {
        "mudo"
    }

    fn complete<'a>(
        &'a self,
        _request: ModelRequest,
    ) -> BoxFuture<'a, Result<ModelResponse, PortError>> {
        Box::pin(std::future::pending())
    }
}

/// Cancelar detiene la ejecución en el siguiente paso: la herramienta que
/// pidió el modelo ya no se ejecuta, y una espera al modelo se corta.
#[test]
fn cancelling_stops_at_the_next_step() {
    let (rt, s) = session(Document::new("pocion", 8, 8).unwrap());
    let cancel = CancellationToken::new();
    let runner = ApiRunner::new(Arc::new(CancelWhileAnswering {
        cancel: cancel.clone(),
    }));
    let summary = rt
        .block_on(runner.run(task(10), tools(&s), EventSink::channel().0, cancel))
        .unwrap();
    assert_eq!(summary.end, RunEnd::Cancelled);
    assert_eq!(summary.steps, 1);
    assert_eq!(canvas(&s), Canvas::new(8, 8).unwrap(), "no dibujó nada");
    assert_eq!(summary.status(task(10).limits), "Cancelado tras 1 paso");

    // Cancelado de antemano: ni llama al modelo.
    let provider = Arc::new(FakeProvider::new([reply(vec![text("hola")])]));
    let cancel = CancellationToken::new();
    cancel.cancel();
    let summary = rt
        .block_on(ApiRunner::new(provider.clone()).run(
            task(10),
            tools(&s),
            EventSink::channel().0,
            cancel,
        ))
        .unwrap();
    assert_eq!((summary.end, summary.steps), (RunEnd::Cancelled, 0));
    assert!(provider.requests.lock().unwrap().is_empty());

    // Esperando a un modelo que no contesta.
    let cancel = CancellationToken::new();
    let later = cancel.clone();
    rt.spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        later.cancel();
    });
    let summary = rt
        .block_on(async {
            tokio::time::timeout(
                Duration::from_secs(5),
                ApiRunner::new(Arc::new(Silent)).run(
                    task(10),
                    tools(&s),
                    EventSink::channel().0,
                    cancel,
                ),
            )
            .await
        })
        .expect("la cancelación corta la espera")
        .unwrap();
    assert_eq!((summary.end, summary.steps), (RunEnd::Cancelled, 0));
}

/// Al llegar al límite de pasos (o de tokens) termina con un estado claro.
#[test]
fn limits_end_the_run_with_a_clear_status() {
    let (rt, s) = session(Document::new("pocion", 8, 8).unwrap());
    let endless = || {
        (0..10)
            .map(|i| {
                reply(vec![draw(
                    &format!("t{i}"),
                    &format!("set_pixel {i} 0 red"),
                )])
            })
            .collect::<Vec<_>>()
    };
    let limits = AgentLimits {
        max_steps: 3,
        max_tokens: 1_000_000,
    };
    let summary = rt
        .block_on(ApiRunner::new(Arc::new(FakeProvider::new(endless()))).run(
            AgentTask { limits, ..task(3) },
            tools(&s),
            EventSink::channel().0,
            CancellationToken::new(),
        ))
        .unwrap();
    assert_eq!((summary.end, summary.steps), (RunEnd::StepLimit, 3));
    assert_eq!(
        summary.status(limits),
        "Se paró al llegar al límite de 3 pasos"
    );
    // Los tres pasos dibujaron.
    let lines = [
        "set_pixel 0 0 red",
        "set_pixel 1 0 red",
        "set_pixel 2 0 red",
    ];
    assert_eq!(canvas(&s), expected(8, 8, &lines));

    // Tokens: cada respuesta gasta 120; con 300 se para tras la tercera.
    let limits = AgentLimits {
        max_steps: 50,
        max_tokens: 300,
    };
    let summary = rt
        .block_on(ApiRunner::new(Arc::new(FakeProvider::new(endless()))).run(
            AgentTask { limits, ..task(50) },
            tools(&s),
            EventSink::channel().0,
            CancellationToken::new(),
        ))
        .unwrap();
    assert_eq!((summary.end, summary.steps), (RunEnd::TokenLimit, 3));
    assert_eq!(
        summary.status(limits),
        "Se paró al llegar al límite de 300 tokens (usó 360)"
    );
}

/// Un error del proveedor termina la ejecución con ese error.
#[test]
fn provider_errors_are_returned() {
    let (rt, s) = session(Document::new("pocion", 8, 8).unwrap());
    let provider = FakeProvider::default();
    provider.push_error(PortError::Auth("clave rechazada".into()));
    let result = rt.block_on(ApiRunner::new(Arc::new(provider)).run(
        task(10),
        tools(&s),
        EventSink::channel().0,
        CancellationToken::new(),
    ));
    assert_eq!(result, Err(PortError::Auth("clave rechazada".into())));
}

// --- Herramientas ---------------------------------------------------------

fn call(rt: &tokio::runtime::Runtime, tools: &ToolBox, name: &str, input: &str) -> (String, bool) {
    let out = rt.block_on(tools.call(name, input));
    let text = out
        .content
        .iter()
        .filter_map(|p| match p {
            ContentPart::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    (text, out.is_error)
}

#[test]
fn the_tools_work_against_the_session() {
    let (rt, s) = session(Document::new("pocion", 4, 4).unwrap());
    let t = tools(&s);
    let specs: Vec<String> = t.specs().into_iter().map(|s| s.name).collect();
    assert_eq!(specs, TOOL_NAMES);

    let (out, err) = call(&rt, &t, "palette", "");
    assert_eq!(
        out,
        "Sin paleta fija. El lienzo está vacío: puedes usar cualquier color."
    );
    assert!(!err);

    let input = serde_json::json!({"commands": "set_pixel 1 2 #ff0000\nset_pixel 9 9 red\nnope"});
    let (out, err) = call(&rt, &t, "draw", &input.to_string());
    assert!(out.ends_with("batch: 1 ok, 2 failed"), "{out}");
    assert!(out.contains("line 2:"), "{out}");
    assert!(!err, "algo funcionó");

    let (out, err) = call(&rt, &t, "get_pixel", r#"{"x":1,"y":2}"#);
    assert_eq!((out.as_str(), err), ("ok #ff0000ff", false));
    let (out, err) = call(&rt, &t, "get_pixel", r#"{"x":10,"y":0}"#);
    assert!(err && out.starts_with("err"), "{out}");

    let (out, _) = call(&rt, &t, "palette", "{}");
    assert_eq!(out, "Sin paleta fija. Colores del lienzo (1): #ff0000.");

    let (out, err) = call(&rt, &t, "undo", "{}");
    assert_eq!((out.as_str(), err), ("ok", false));
    assert_eq!(canvas(&s), Canvas::new(4, 4).unwrap());
    let (out, err) = call(&rt, &t, "undo", "{}");
    assert!(err && out.starts_with("err no_history"), "{out}");

    // Todo falla: error para el modelo.
    let (out, err) = call(&rt, &t, "draw", r#"{"commands":"set_pixel 99 99 red"}"#);
    assert!(err, "{out}");
    // Exportar no está permitido.
    let (out, err) = call(
        &rt,
        &t,
        "draw",
        r#"{"commands":"set_pixel 0 0 red\nlumeart export a.png"}"#,
    );
    assert!(err && out.contains("export no está disponible"), "{out}");
    assert_eq!(canvas(&s), Canvas::new(4, 4).unwrap(), "no se aplicó nada");
    // Entradas malas y herramientas que no existen.
    assert!(call(&rt, &t, "draw", r#"{"cmds":"x"}"#).1);
    assert!(call(&rt, &t, "draw", "{no es json").1);
    assert!(call(&rt, &t, "get_pixel", r#"{"x":"1"}"#).1);
    let (out, err) = call(&rt, &t, "bash", "{}");
    assert!(err && out.contains("draw, view_canvas"), "{out}");

    // view_canvas: texto y PNG ampliado (4×4 → ×32 = 128 px; el máximo es 32).
    let out = rt.block_on(t.call("view_canvas", "{}"));
    assert!(!out.is_error);
    assert!(
        matches!(&out.content[0], ContentPart::Text(t) if t.contains("4×4") && t.contains("×32"))
    );
    assert!(
        matches!(&out.content[1], ContentPart::Image { media_type, .. } if media_type == "image/png")
    );
}

#[test]
fn the_palette_tool_lists_a_fixed_palette_and_strays() {
    let mut doc = Document::new("gb", 4, 4).unwrap();
    doc.palette = Palette::fixed("game-boy");
    let (rt, s) = session(doc);
    let t = tools(&s);
    rt.block_on(s.handle.line("set_pixel 0 0 #ff0000", Origin::User, None))
        .unwrap();
    let (out, _) = call(&rt, &t, "palette", "{}");
    assert!(
        out.starts_with("Paleta fija game-boy (4 colores); usa solo estos: #"),
        "{out}"
    );
    assert!(
        out.ends_with("Colores del lienzo fuera de la paleta: #ff0000."),
        "{out}"
    );
}
