//! El núcleo del `RefineLoop` con el agente de verdad (`ApiRunner` +
//! herramientas sobre la sesión) y `FakeProvider`: cada vuelta empieza con
//! el contexto vacío, las notas pasan de una vuelta a la siguiente y cada
//! vuelta guarda su versión.

use std::sync::Arc;

use lume_agent::{ApiRunner, SessionTools};
use lume_app::ports::{
    CANVAS_ATTACHMENT, CancellationToken, ContentPart, ModelResponse, StopReason, Usage,
};
use lume_app::testing::{FakeBackend, FakeProvider, MemoryStore};
use lume_app::{LoopEnd, LoopEvent, LoopSink, RefineLoop, Session, Settings};
use lume_domain::{Document, Origin};

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
            input_tokens: 50,
            output_tokens: 10,
            cached_tokens: 0,
        },
    }
}

fn tool(id: &str, name: &str, input: serde_json::Value) -> ContentPart {
    ContentPart::ToolUse {
        id: id.into(),
        name: name.into(),
        input_json: input.to_string(),
    }
}

fn text(t: &str) -> ContentPart {
    ContentPart::Text(t.into())
}

const RUN: u64 = 4;

fn setup(
    provider: Arc<FakeProvider>,
    max_turns: u32,
) -> (tokio::runtime::Runtime, Session, RefineLoop) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let store = Arc::new(MemoryStore::new());
    let session = {
        let _guard = rt.enter();
        Session::spawn(Document::new("pocion", 8, 8).unwrap(), store.clone())
    };
    let tools = Arc::new(SessionTools::new(
        session.handle.clone(),
        store.clone(),
        RUN,
    ));
    let backend = Arc::new(FakeBackend::new(Arc::new(ApiRunner::new(provider)), tools));
    let settings = Settings {
        max_turns,
        ..Settings::default()
    };
    let refine = RefineLoop::new(backend, session.handle.clone(), store, settings);
    (rt, session, refine)
}

fn first_text(request: &lume_app::ports::ModelRequest) -> String {
    match &request.messages[0].content[0] {
        ContentPart::Text(t) => t.clone(),
        other => panic!("se esperaba texto: {other:?}"),
    }
}

/// Dos vueltas: la primera dibuja y escribe notas; la segunda las recibe en
/// el bloque de estado, empieza sin mensajes de la primera y termina con la
/// promesa.
#[test]
fn turns_start_empty_and_inherit_only_the_state() {
    let provider = Arc::new(FakeProvider::new([
        // Vuelta 1.
        reply(vec![
            text("Empiezo por el líquido."),
            tool(
                "t1",
                "draw",
                serde_json::json!({"commands": "fill_rect 1 4 6 3 #8b3fe0"}),
            ),
            tool(
                "t2",
                "write_notes",
                serde_json::json!({"text": "Hice el líquido morado. Falta el tapón de corcho."}),
            ),
        ]),
        reply(vec![text("Vuelta 1 terminada.")]),
        // Vuelta 2.
        reply(vec![tool(
            "t3",
            "draw",
            serde_json::json!({"commands": "fill_rect 3 1 2 2 #a86b3c"}),
        )]),
        reply(vec![text("Añadí el tapón. <promise>LISTO</promise>")]),
    ]));
    let (rt, session, refine) = setup(provider.clone(), 3);
    let (sink, mut rx) = LoopSink::channel();
    let summary = rt.block_on(refine.run(
        RUN,
        "una poción morada con tapón",
        vec![],
        sink,
        CancellationToken::new(),
    ));
    assert_eq!(summary.end, LoopEnd::Done);
    assert_eq!(summary.turns, 2);
    assert_eq!(summary.usage.total(), 4 * 60);

    let requests = provider.requests.lock().unwrap().clone();
    assert_eq!(requests.len(), 4);
    // Vuelta 1: un solo mensaje, sin notas, sin lienzo (vacío).
    assert_eq!(requests[0].messages.len(), 1);
    let first = first_text(&requests[0]);
    assert!(
        first.starts_with("Petición: una poción morada con tapón"),
        "{first}"
    );
    assert!(first.contains("vuelta 1 de 3"), "{first}");
    assert!(first.contains("(todavía no hay notas)"), "{first}");
    assert_eq!(requests[0].messages[0].content.len(), 1, "sin adjuntos");

    // Vuelta 2: conversación nueva (nada de la vuelta 1), con las notas y
    // el lienzo como imagen.
    let turn2 = &requests[2];
    assert_eq!(
        turn2.messages.len(),
        1,
        "la vuelta 2 empieza sin mensajes de la 1"
    );
    let second = first_text(turn2);
    assert!(second.contains("vuelta 2 de 3"), "{second}");
    assert!(
        second.contains("Hice el líquido morado. Falta el tapón de corcho."),
        "{second}"
    );
    assert!(!second.contains("Empiezo por el líquido."), "{second}");
    assert!(
        second.contains("Sigue trabajando sobre el dibujo actual"),
        "{second}"
    );
    assert!(
        second.contains("El lienzo al empezar esta vuelta va adjunto"),
        "{second}"
    );
    let content = &turn2.messages[0].content;
    assert_eq!(content[1], ContentPart::Text(CANVAS_ATTACHMENT.into()));
    assert!(
        matches!(&content[2], ContentPart::Image { media_type, .. } if media_type == "image/png")
    );

    // Una versión por vuelta, del agente y con su número de vuelta.
    let versions = session.handle.view().borrow().versions.clone();
    assert_eq!(versions.len(), 2);
    for (i, v) in versions.iter().enumerate() {
        assert_eq!(v.number as usize, i + 1);
        assert_eq!(v.author, Origin::Agent { run: RUN });
        assert_eq!(v.message, format!("Vuelta {} de 3", i + 1));
    }
    assert_eq!(summary.versions, [1, 2]);
    // Las notas quedan en el documento.
    let doc = rt.block_on(session.handle.snapshot()).unwrap();
    assert_eq!(
        doc.notes,
        "Hice el líquido morado. Falta el tapón de corcho."
    );

    // Eventos para la interfaz.
    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }
    let marks: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            LoopEvent::TurnStarted { turn, of } => Some(format!("empieza {turn}/{of}")),
            LoopEvent::TurnFinished { turn, version, .. } => {
                Some(format!("termina {turn} v{version}"))
            }
            LoopEvent::Agent(_) | LoopEvent::Critic { .. } => None,
        })
        .collect();
    assert_eq!(
        marks,
        ["empieza 1/3", "termina 1 v1", "empieza 2/3", "termina 2 v2"]
    );
}

/// Sin promesa, el bucle hace todas las vueltas, cada una con su versión.
#[test]
fn without_the_promise_all_turns_run() {
    let mut responses = Vec::new();
    for i in 0..2 {
        responses.push(reply(vec![tool(
            &format!("t{i}"),
            "draw",
            serde_json::json!({"commands": format!("set_pixel {i} 0 red")}),
        )]));
        responses.push(reply(vec![text("Sigo mañana.")]));
    }
    let provider = Arc::new(FakeProvider::new(responses));
    let (rt, session, refine) = setup(provider, 2);
    let summary = rt.block_on(refine.run(
        RUN,
        "dos píxeles",
        vec![],
        LoopSink::channel().0,
        CancellationToken::new(),
    ));
    assert_eq!((summary.end, summary.turns), (LoopEnd::MaxTurns, 2));
    assert_eq!(session.handle.view().borrow().versions.len(), 2);
}

// --- T:38: comprobaciones y límites -------------------------------------

use lume_app::Delivery;
use lume_app::ModelProvider;
use lume_app::ports::{BoxFuture, ModelRequest, PortError};

fn setup_with(
    provider: Arc<dyn lume_app::ModelProvider>,
    doc: Document,
    settings: Settings,
) -> (tokio::runtime::Runtime, Session, RefineLoop) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let store = Arc::new(MemoryStore::new());
    let session = {
        let _guard = rt.enter();
        Session::spawn(doc, store.clone())
    };
    let tools = Arc::new(SessionTools::new(
        session.handle.clone(),
        store.clone(),
        RUN,
    ));
    let backend = Arc::new(FakeBackend::new(Arc::new(ApiRunner::new(provider)), tools));
    let refine = RefineLoop::new(backend, session.handle.clone(), store, settings);
    (rt, session, refine)
}

fn draw(id: &str, commands: &str) -> ContentPart {
    tool(id, "draw", serde_json::json!({ "commands": commands }))
}

fn run_loop(rt: &tokio::runtime::Runtime, refine: &RefineLoop) -> lume_app::LoopSummary {
    rt.block_on(refine.run(
        RUN,
        "una poción",
        vec![],
        LoopSink::channel().0,
        CancellationToken::new(),
    ))
}

fn settings(max_turns: u32) -> Settings {
    Settings {
        max_turns,
        ..Settings::default()
    }
}

/// La promesa sola no termina el bucle si falla una comprobación: la
/// siguiente vuelta lo sabe por el bloque de estado.
#[test]
fn the_promise_alone_does_not_end_the_loop() {
    // Vuelta 1: promete sin dibujar. Vuelta 2: dibuja con un color fuera de
    // la paleta Game Boy y promete. Vuelta 3: lo arregla y promete.
    let provider = Arc::new(FakeProvider::new([
        reply(vec![text("Ya está. <promise>LISTO</promise>")]),
        reply(vec![draw("t1", "fill_rect 0 0 4 4 #ff0000")]),
        reply(vec![text("Hecho. <promise>LISTO</promise>")]),
        reply(vec![draw("t2", "fill_rect 0 0 4 4 #306230")]),
        reply(vec![text("Ahora sí. <promise>LISTO</promise>")]),
    ]));
    let mut doc = Document::new("gb", 8, 8).unwrap();
    doc.palette = lume_domain::Palette::fixed("game-boy");
    let (rt, _session, refine) = setup_with(provider.clone(), doc, settings(5));
    let summary = run_loop(&rt, &refine);
    assert_eq!((summary.end.clone(), summary.turns), (LoopEnd::Done, 3));
    assert!(summary.checks.iter().all(|c| c.passed));

    let requests = provider.requests.lock().unwrap().clone();
    let turn2 = first_text(&requests[1]);
    assert!(
        turn2.contains("Comprobaciones que no se cumplieron"),
        "{turn2}"
    );
    assert!(turn2.contains("- El lienzo está vacío."), "{turn2}");
    let turn3 = first_text(&requests[3]);
    assert!(
        turn3.contains("- Hay colores fuera de la paleta fija: #ff0000."),
        "{turn3}"
    );
    assert!(!turn3.contains("El lienzo está vacío"), "{turn3}");
}

/// Dos vueltas seguidas sin progreso detienen el bucle (si el ajuste está
/// activo); sin el ajuste, sigue hasta el máximo.
#[test]
fn two_turns_without_progress_stop_the_loop() {
    let script = || {
        vec![
            reply(vec![draw("t1", "fill_rect 0 0 6 6 #8b3fe0")]),
            reply(vec![text("Base hecha.")]),
            reply(vec![text("Lo miro y no cambio nada.")]),
            reply(vec![text("Sigo sin cambiar nada.")]),
            reply(vec![text("Nada.")]),
            reply(vec![text("Nada otra vez.")]),
        ]
    };
    let (rt, _s, refine) = setup_with(
        Arc::new(FakeProvider::new(script())),
        Document::new("p", 8, 8).unwrap(),
        settings(6),
    );
    let summary = run_loop(&rt, &refine);
    assert_eq!(
        (summary.end.clone(), summary.turns),
        (LoopEnd::NoProgress, 3)
    );
    assert_eq!(
        summary.reason(),
        "Se paró porque dos vueltas seguidas no avanzaron. Se entrega la versión 3 (la última que pasó las comprobaciones)."
    );

    let (rt, _s, refine) = setup_with(
        Arc::new(FakeProvider::new(script())),
        Document::new("p", 8, 8).unwrap(),
        Settings {
            stop_without_progress: false,
            ..settings(5)
        },
    );
    let summary = run_loop(&rt, &refine);
    assert_eq!((summary.end, summary.turns), (LoopEnd::MaxTurns, 5));
}

/// Cada límite termina con su motivo: vueltas, tokens, Detener y un fallo.
#[test]
fn every_limit_has_its_reason() {
    // Vueltas.
    let provider = Arc::new(FakeProvider::new([
        reply(vec![draw("t1", "set_pixel 0 0 red")]),
        reply(vec![text("uno")]),
        reply(vec![draw("t2", "set_pixel 1 0 red")]),
        reply(vec![text("dos")]),
    ]));
    let (rt, _s, refine) = setup_with(provider, Document::new("p", 8, 8).unwrap(), settings(2));
    let summary = run_loop(&rt, &refine);
    assert_eq!(summary.end, LoopEnd::MaxTurns);
    assert_eq!(
        summary.reason(),
        "Se hicieron las 2 vueltas sin terminar. Se entrega la versión 2 (la última que pasó las comprobaciones)."
    );

    // Tokens: 60 por respuesta, 2 respuestas por vuelta; con 200, para tras
    // la segunda vuelta (240).
    let provider = Arc::new(FakeProvider::new([
        reply(vec![draw("t1", "set_pixel 0 0 red")]),
        reply(vec![text("uno")]),
        reply(vec![draw("t2", "set_pixel 1 0 red")]),
        reply(vec![text("dos")]),
    ]));
    let (rt, _s, mut refine) = setup_with(provider, Document::new("p", 8, 8).unwrap(), settings(8));
    refine.token_budget = 200;
    let summary = run_loop(&rt, &refine);
    assert_eq!(
        (summary.end.clone(), summary.turns),
        (LoopEnd::TokenBudget, 2)
    );
    assert!(
        summary
            .reason()
            .starts_with("Se paró al gastar el presupuesto de 200 tokens (usó 240)."),
        "{}",
        summary.reason()
    );

    // Detener: cancelado mientras responde el modelo de la vuelta 1.
    let cancel = CancellationToken::new();
    let provider: Arc<dyn ModelProvider> = Arc::new(CancelOnCall {
        cancel: cancel.clone(),
    });
    let (rt, _s, refine) = setup_with(provider, Document::new("p", 8, 8).unwrap(), settings(8));
    let summary = rt.block_on(refine.run(RUN, "x", vec![], LoopSink::channel().0, cancel));
    assert_eq!(summary.end, LoopEnd::Cancelled);
    assert!(
        summary.reason().starts_with("Detenido en la vuelta 1."),
        "{}",
        summary.reason()
    );

    // Un fallo del proveedor.
    let failing = FakeProvider::default();
    failing.push_error(PortError::Auth("clave rechazada".into()));
    let (rt, _s, refine) = setup_with(
        Arc::new(failing),
        Document::new("p", 8, 8).unwrap(),
        settings(8),
    );
    let summary = run_loop(&rt, &refine);
    assert_eq!(
        summary.reason(),
        "Falló la vuelta 1: la clave de API no es válida: clave rechazada"
    );
    assert_eq!(summary.delivered, None, "no hay versiones");
}

/// Un proveedor que cancela el bucle cuando lo llaman y responde texto.
struct CancelOnCall {
    cancel: CancellationToken,
}

impl ModelProvider for CancelOnCall {
    fn id(&self) -> &str {
        "cancela"
    }

    fn complete<'a>(
        &'a self,
        _request: ModelRequest,
    ) -> BoxFuture<'a, Result<ModelResponse, PortError>> {
        Box::pin(async move {
            self.cancel.cancel();
            Ok(reply(vec![text("me cancelan")]))
        })
    }
}

/// Al terminar sin éxito se entrega la última versión que pasó: si la última
/// vuelta falló una comprobación, el lienzo vuelve a la anterior.
#[test]
fn the_last_passing_version_is_delivered() {
    let provider = Arc::new(FakeProvider::new([
        reply(vec![draw("t1", "fill_rect 0 0 4 4 #8b3fe0")]),
        reply(vec![text("Bien.")]),
        // Vuelta 2: el último lote falla.
        reply(vec![draw(
            "t2",
            "set_pixel 1 1 #ffffff\nset_pixel 99 99 red",
        )]),
        reply(vec![text("Algo falló.")]),
    ]));
    let (rt, session, refine) =
        setup_with(provider, Document::new("p", 8, 8).unwrap(), settings(2));
    let summary = run_loop(&rt, &refine);
    assert_eq!(summary.end, LoopEnd::MaxTurns);
    assert_eq!(summary.delivered, Some((1, Delivery::LastPassing)));
    assert!(!summary.checks[1].passed, "{:?}", summary.checks);
    assert_eq!(
        summary.checks[1].detail,
        "El último lote de comandos tuvo errores."
    );
    let doc = rt.block_on(session.handle.snapshot()).unwrap();
    assert_eq!(
        doc.canvas(),
        &doc.version(1).unwrap().canvas,
        "vuelve a la versión 1"
    );
}

/// Con el crítico: sus problemas llegan a la vuelta siguiente y, si ninguna
/// versión pasa, se entrega la de menos problemas graves.
#[test]
fn the_critic_steers_the_loop_and_picks_the_best_version() {
    let verdict = |n: usize| {
        let issues: Vec<serde_json::Value> = (0..n)
            .map(|i| serde_json::json!({"gravedad": "alta", "zona": [0, 0, 2, 2], "problema": format!("problema {i}"), "arreglo": "arréglalo"}))
            .collect();
        format!(
            "```json\n{}\n```",
            serde_json::json!({"aprobado": false, "problemas": issues})
        )
    };
    let provider = Arc::new(FakeProvider::new([
        // Vuelta 1 y su crítico (1 problema grave).
        reply(vec![draw("t1", "fill_rect 0 0 4 4 #8b3fe0")]),
        reply(vec![text("Vuelta 1. <promise>LISTO</promise>")]),
        reply(vec![text(&verdict(1))]),
        // Vuelta 2 y su crítico (2 problemas graves).
        reply(vec![draw("t2", "fill_rect 4 4 4 4 #ff0000")]),
        reply(vec![text("Vuelta 2.")]),
        reply(vec![text(&verdict(2))]),
    ]));
    let (rt, session, refine) = setup_with(
        provider.clone(),
        Document::new("p", 8, 8).unwrap(),
        Settings {
            critic: true,
            ..settings(2)
        },
    );
    let summary = run_loop(&rt, &refine);
    assert_eq!(
        summary.end,
        LoopEnd::MaxTurns,
        "la promesa con problemas graves no basta"
    );
    assert_eq!(summary.delivered, Some((1, Delivery::BestByCritic)));
    assert!(
        summary
            .reason()
            .ends_with("Se entrega la versión 1 (la mejor según el crítico).")
    );
    let doc = rt.block_on(session.handle.snapshot()).unwrap();
    assert_eq!(doc.canvas(), &doc.version(1).unwrap().canvas);

    // La vuelta 2 recibió lo que dijo el crítico de la 1.
    let requests = provider.requests.lock().unwrap().clone();
    let turn2 = first_text(&requests[3]);
    assert!(
        turn2.contains("Lo que vio el crítico en la vuelta anterior"),
        "{turn2}"
    );
    assert!(
        turn2.contains(
            "1. [alta] en la zona x 0, y 0, 2×2: problema 0 Arreglo propuesto: arréglalo"
        ),
        "{turn2}"
    );
    assert!(
        turn2.contains("- El crítico ve 1 problema grave."),
        "{turn2}"
    );
    // El crítico solo tenía herramientas de mirar.
    let critic_tools: Vec<&str> = requests[2].tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(critic_tools, ["view_canvas", "get_pixel", "palette"]);
}

// --- T:39: dirigir el bucle y misma conversación --------------------------

/// `FakeProvider` que, en la llamada número `at`, manda un mensaje al bucle
/// (como si el usuario escribiera mientras el agente dibuja).
struct MessageDuring {
    inner: FakeProvider,
    at: usize,
    calls: std::sync::Mutex<usize>,
    messages: std::sync::Mutex<Option<lume_app::LoopMessages>>,
}

impl ModelProvider for MessageDuring {
    fn id(&self) -> &str {
        "con-mensaje"
    }

    fn complete<'a>(
        &'a self,
        request: ModelRequest,
    ) -> BoxFuture<'a, Result<ModelResponse, PortError>> {
        let mut calls = self.calls.lock().unwrap();
        *calls += 1;
        if *calls == self.at
            && let Some(messages) = self.messages.lock().unwrap().as_ref()
        {
            messages.send("Hazla un poco más grande y con brillo.");
        }
        drop(calls);
        self.inner.complete(request)
    }
}

/// Un mensaje enviado en la vuelta 2 aparece en el bloque de estado de la 3,
/// sin detener el bucle.
#[test]
fn a_message_sent_in_turn_two_reaches_turn_three() {
    let provider = Arc::new(MessageDuring {
        inner: FakeProvider::new([
            reply(vec![draw("t1", "fill_rect 0 0 4 4 #8b3fe0")]),
            reply(vec![text("uno")]),
            reply(vec![draw("t2", "fill_rect 4 4 2 2 #8b3fe0")]),
            reply(vec![text("dos")]),
            reply(vec![draw("t3", "fill_rect 0 0 8 8 #8b3fe0")]),
            reply(vec![text("tres <promise>LISTO</promise>")]),
        ]),
        at: 3,
        calls: std::sync::Mutex::new(0),
        messages: std::sync::Mutex::new(None),
    });
    let (rt, _s, refine) = setup_with(
        provider.clone(),
        Document::new("p", 8, 8).unwrap(),
        settings(4),
    );
    *provider.messages.lock().unwrap() = Some(refine.messages());
    let summary = run_loop(&rt, &refine);
    assert_eq!((summary.end, summary.turns), (LoopEnd::Done, 3));

    let requests = provider.inner.requests.lock().unwrap().clone();
    let turn2 = first_text(&requests[2]);
    let turn3 = first_text(&requests[4]);
    assert!(!turn2.contains("Mensajes del usuario"), "{turn2}");
    assert!(
        turn3.contains("### Mensajes del usuario durante el bucle"),
        "{turn3}"
    );
    assert!(
        turn3.contains("- Hazla un poco más grande y con brillo."),
        "{turn3}"
    );
    assert!(turn3.contains("vuelta 3 de 4"), "{turn3}");
}

/// Con `Resume` («misma conversación»), la vuelta 2 recibe la conversación
/// de la 1 y su encargo nuevo detrás.
#[test]
fn with_resume_turn_two_gets_turn_one_conversation() {
    let provider = Arc::new(FakeProvider::new([
        reply(vec![
            text("Empiezo."),
            draw("t1", "fill_rect 0 0 4 4 #8b3fe0"),
        ]),
        reply(vec![text("Vuelta 1 hecha.")]),
        reply(vec![draw("t2", "fill_rect 4 4 4 4 #8b3fe0")]),
        reply(vec![text("Listo. <promise>LISTO</promise>")]),
    ]));
    let (rt, _s, refine) = setup_with(
        provider.clone(),
        Document::new("p", 8, 8).unwrap(),
        Settings {
            context: lume_app::ContextPolicy::Resume,
            ..settings(3)
        },
    );
    let summary = run_loop(&rt, &refine);
    assert_eq!((summary.end, summary.turns), (LoopEnd::Done, 2));

    let requests = provider.requests.lock().unwrap().clone();
    let turn2 = &requests[2];
    // Usuario (vuelta 1), asistente, resultados, asistente, usuario (vuelta 2).
    assert_eq!(turn2.messages.len(), 5, "{:#?}", turn2.messages);
    assert_eq!(
        turn2.messages[..3],
        requests[1].messages[..],
        "la vuelta 1 entera"
    );
    let ContentPart::Text(last) = &turn2.messages[3].content[0] else {
        panic!()
    };
    assert_eq!(last, "Vuelta 1 hecha.");
    let ContentPart::Text(new) = &turn2.messages[4].content[0] else {
        panic!()
    };
    assert!(new.contains("vuelta 2 de 3"), "{new}");
    assert!(new.contains("Sigues la misma conversación"), "{new}");
    assert!(!new.contains("contexto vacío"), "{new}");
}

/// Rasterizador que no se usa (el test no mira el dibujo).
struct NoRaster;

impl lume_app::ports::Rasterizer for NoRaster {
    fn rasterize(
        &self,
        _svg: &str,
        _width: f64,
        _height: f64,
        _options: &lume_app::ports::RasterOptions,
    ) -> Result<lume_app::ports::RasterImage, PortError> {
        Err(PortError::Unavailable("sin rasterizador".into()))
    }
}

/// Un trabajo vectorial sobre un documento que también tiene píxeles y
/// paleta fija: el agente recibe `dibujo.js` y no el lienzo ni la paleta;
/// su cambio es una versión «Agente · …» y la vuelta termina con la promesa.
#[test]
fn a_vector_job_gets_dibujo_js_not_the_pixel_canvas() {
    const BEFORE: &str =
        "canvas({ width: 100, height: 100 })\ncircle(50, 50, 20, { fill: \"#e8803a\" })\n";
    const AFTER: &str =
        "canvas({ width: 100, height: 100 })\ncircle(50, 50, 30, { fill: \"#e8803a\" })\n";
    let provider = Arc::new(FakeProvider::new([
        reply(vec![
            text("Agrando el círculo."),
            tool("t1", "write_vec", serde_json::json!({ "source": AFTER })),
        ]),
        reply(vec![text("Hecho. <promise>LISTO</promise>")]),
    ]));
    let rt = tokio::runtime::Runtime::new().unwrap();
    let store = Arc::new(MemoryStore::new());
    let mut doc = Document::new("mar", 8, 8).unwrap();
    doc.palette = lume_domain::Palette::fixed("game-boy");
    let session = {
        let _guard = rt.enter();
        Session::spawn(doc, store.clone())
    };
    rt.block_on(
        session
            .handle
            .batch("fill_rect 0 0 4 4 #0f380f", Origin::User, None),
    )
    .unwrap();
    rt.block_on(session.handle.edit_vector(
        lume_domain::vec::drawing::VecEdit::Set(Some(
            lume_domain::vec::drawing::VectorDrawing::new(BEFORE),
        )),
        Origin::User,
    ))
    .unwrap();
    let tools = Arc::new(
        SessionTools::new(session.handle.clone(), store.clone(), RUN).with_vec(
            lume_agent::VecTools {
                engine: Arc::new(lume_script::QuickJsVec::default()),
                rasterizer: Arc::new(NoRaster),
            },
        ),
    );
    let backend = Arc::new(FakeBackend::new(
        Arc::new(ApiRunner::new(provider.clone())),
        tools,
    ));
    let mut refine = RefineLoop::new(backend, session.handle.clone(), store, settings(3));
    refine.vector = true;
    let summary = rt.block_on(refine.run(
        RUN,
        "un círculo más grande",
        vec![],
        LoopSink::channel().0,
        CancellationToken::new(),
    ));
    assert_eq!(summary.end, LoopEnd::Done, "{summary:?}");

    let requests = provider.requests.lock().unwrap();
    let first = &requests[0].messages[0].content;
    assert!(
        !first.iter().any(|p| matches!(p, ContentPart::Image { .. })),
        "sin el lienzo de píxeles: {first:?}"
    );
    assert!(
        !first
            .iter()
            .any(|p| *p == ContentPart::Text(CANVAS_ATTACHMENT.into())),
        "{first:?}"
    );
    let task = first_text(&requests[0]);
    assert!(task.contains("Petición: un círculo más grande"), "{task}");
    // El código lo lee con `read_vec`; el texto de seguir es el vectorial.
    assert!(task.contains("`read_vec`"), "{task}");
    assert!(
        task.contains("dibujo.js ya tiene lo que hay dibujado"),
        "{task}"
    );
    assert!(
        !task.contains("game-boy"),
        "sin la paleta de píxeles: {task}"
    );
    drop(requests);

    let vm = session.handle.view().borrow().clone();
    assert_eq!(vm.vector.as_ref().unwrap().source, AFTER);
    let messages: Vec<&str> = vm.versions.iter().map(|v| v.message.as_str()).collect();
    assert_eq!(messages, ["Agente · línea 2", "Vuelta 1 de 3"]);
    // Y se puede deshacer.
    rt.block_on(session.handle.line("undo", Origin::User, None))
        .unwrap();
    assert_eq!(
        session
            .handle
            .view()
            .borrow()
            .vector
            .as_ref()
            .unwrap()
            .source,
        BEFORE
    );
}
