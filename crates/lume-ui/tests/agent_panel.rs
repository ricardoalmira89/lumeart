//! Panel del agente con el `StudioApp` real (arnés de eframe de
//! egui_kittest, sin ventana) y el agente de verdad: `ApiRunner` +
//! herramientas sobre la sesión + `FakeProvider` (respuestas grabadas).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use lume_agent::{ApiRunner, SessionTools};
use lume_app::ports::{
    AgentTask, BoxFuture, CancellationToken, ContentPart, EventSink, ModelRequest, ModelResponse,
    PortError, RunEnd, RunSummary, StopReason, ToolBox, Usage,
};
use lume_app::testing::{
    FakeBackend, FakeProvider, FakeTools, MemorySecretStore, MemorySettingsStore, MemoryStore,
};
use lume_app::{
    AgentBackend, AgentRunner, ModelProvider, ProviderKind, Session, Settings, SettingsStore,
};
use lume_domain::{Canvas, Document, Origin, Rgba};
use lume_ui::{FakeDialogs, Screen, StudioApp, StudioOptions};

struct World {
    runtime: tokio::runtime::Runtime,
    session: Session,
    settings: Arc<MemorySettingsStore>,
    dialogs: Arc<FakeDialogs>,
}

fn world() -> World {
    world_with(Settings::default())
}

/// Con estos ajustes guardados.
fn world_with(settings: Settings) -> World {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let session = {
        let _guard = runtime.enter();
        Session::spawn(
            Document::new("pocion", 16, 16).unwrap(),
            Arc::new(MemoryStore::new()),
        )
    };
    World {
        runtime,
        session,
        settings: Arc::new(MemorySettingsStore::new(settings)),
        dialogs: Arc::new(FakeDialogs::default()),
    }
}

fn app(w: &World, backend: Arc<dyn AgentBackend>, screen: Screen) -> Harness<'static, StudioApp> {
    app_with(w, backend, screen, Harness::builder())
}

/// Con un arnés a medida (por ejemplo, con fotogramas más cortos).
fn app_with(
    w: &World,
    backend: Arc<dyn AgentBackend>,
    screen: Screen,
    builder: egui_kittest::HarnessBuilder<StudioApp>,
) -> Harness<'static, StudioApp> {
    let options = StudioOptions {
        session: w.session.handle.clone(),
        stop: CancellationToken::new(),
        runtime: w.runtime.handle().clone(),
        store: Arc::new(MemoryStore::new()),
        recents: None,
        dialogs: w.dialogs.clone(),
        settings: w.settings.clone(),
        secrets: Arc::new(MemorySecretStore::default()),
        claude_path: None,
        agent: Some(backend),
        server: None,
        screen,
        art: None,
        anim_engine: None,
        variant_agents: None,
        vector: None,
        autosave: None,
        window: None,
    };
    let mut harness = builder
        .with_size(egui::vec2(1440.0, 900.0))
        .build_eframe(move |cc| StudioApp::new(cc, options));
    harness.run_steps(3);
    harness
}

/// Avanza fotogramas hasta que se cumpla `cond` (máximo ~5 s).
fn wait_until(
    h: &mut Harness<'static, StudioApp>,
    what: &str,
    cond: impl Fn(&Harness<'static, StudioApp>) -> bool,
) {
    for _ in 0..500 {
        h.step();
        if cond(h) {
            h.step();
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("no llegó: {what}");
}

fn has(h: &Harness<'static, StudioApp>, label: &str) -> bool {
    h.query_by_label(label).is_some()
}

fn send(h: &mut Harness<'static, StudioApp>, text: &str) {
    h.get_by_label("Mensaje para el agente").focus();
    h.step();
    h.get_by_label("Mensaje para el agente").type_text(text);
    h.step();
    h.get_by_label("Enviar").click();
    h.run_steps(2);
}

/// `FakeProvider` que da cada respuesta solo cuando se le deja (un permiso
/// por respuesta): así se ve la ejecución paso a paso.
struct Gated {
    inner: FakeProvider,
    gate: Arc<tokio::sync::Semaphore>,
}

impl ModelProvider for Gated {
    fn id(&self) -> &str {
        "gated"
    }

    fn complete<'a>(
        &'a self,
        request: ModelRequest,
    ) -> BoxFuture<'a, Result<ModelResponse, PortError>> {
        Box::pin(async move {
            self.gate.acquire().await.unwrap().forget();
            self.inner.complete(request).await
        })
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
        usage: Usage {
            input_tokens: 100,
            output_tokens: 20,
            cached_tokens: 0,
        },
    }
}

fn draw(id: &str, commands: &str) -> ContentPart {
    ContentPart::ToolUse {
        id: id.into(),
        name: "draw".into(),
        input_json: serde_json::json!({ "commands": commands }).to_string(),
    }
}

/// `AgentBackend` con el `ApiRunner` real sobre un proveedor dado y las
/// herramientas de verdad sobre la sesión.
struct RealAgent {
    provider: Arc<dyn ModelProvider>,
    session: lume_app::SessionHandle,
}

impl AgentBackend for RealAgent {
    fn prepare(
        &self,
        _settings: &Settings,
        run: u64,
    ) -> Result<lume_app::ports::PreparedAgent, PortError> {
        Ok(lume_app::ports::PreparedAgent {
            runner: Arc::new(ApiRunner::new(self.provider.clone())),
            tools: Arc::new(SessionTools::new(
                self.session.clone(),
                Arc::new(MemoryStore::new()),
                run,
            )),
            model: "modelo-de-prueba".into(),
        })
    }

    fn prepare_critic(
        &self,
        settings: &Settings,
        run: u64,
    ) -> Result<lume_app::ports::PreparedAgent, PortError> {
        self.prepare(settings, run)
    }
}

/// Una ejecución con `FakeProvider` se ve paso a paso en el panel: el
/// mensaje, «Pensando…», el texto y el bloque de `draw` con su resultado,
/// el paso en curso, la pastilla de la cabecera y, al final, cómo terminó.
#[test]
fn a_run_shows_up_step_by_step() {
    let w = world();
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let provider = Arc::new(Gated {
        inner: FakeProvider::new([
            reply(vec![
                ContentPart::Text("Empiezo por el líquido.".into()),
                draw("t1", "fill_rect 2 8 12 6 #8b3fe0\nset_pixel 7 4 #a86b3c"),
            ]),
            reply(vec![ContentPart::Text(
                "He dibujado la poción. <promise>LISTO</promise>".into(),
            )]),
        ]),
        gate: gate.clone(),
    });
    let backend = Arc::new(RealAgent {
        provider,
        session: w.session.handle.clone(),
    });
    let mut h = app(&w, backend, Screen::Studio);
    assert!(h.get_by_label("Detener").accesskit_node().is_disabled());

    send(&mut h, "una poción morada");
    assert!(h.state().agent.running());
    h.get_by_label("una poción morada");
    h.get_by_label("Pensando…");
    h.get_by_label("Vuelta 1 de 8");
    h.get_by_label("Vuelta 1: en curso");
    h.get_by_label("Vuelta 2: pendiente");
    assert!(!h.get_by_label("Detener").accesskit_node().is_disabled());
    assert!(h.get_by_label("Enviar").accesskit_node().is_disabled());

    // Paso 1: texto y draw.
    gate.add_permits(1);
    wait_until(&mut h, "el resultado de draw", |h| has(h, "2 ok"));
    h.get_by_label("Empiezo por el líquido.");
    h.get_by_label("draw · 2");
    h.get_by_label("· 2 comandos");
    h.get_by_label("Paso 1");
    h.get_by_label("Vuelta 1 de 8");
    h.get_by_label("set_pixel 7 4 #a86b3c");
    let canvas = w.session.handle.view().borrow().canvas.clone();
    assert_eq!(canvas.get(7, 4).unwrap(), Rgba::parse("#a86b3c").unwrap());

    // Paso 2: termina.
    gate.add_permits(1);
    let end =
        "Terminado en la vuelta 1: el agente lo da por bueno y pasan todas las comprobaciones.";
    wait_until(&mut h, "el final", |h| has(h, end));
    h.get_by_label("He dibujado la poción. <promise>LISTO</promise>");
    assert!(!h.state().agent.running());
    assert!(h.query_by_label("Pensando…").is_none());
    assert!(h.query_by_label("Vuelta 1 de 8").is_none(), "sin pastilla");
    h.get_by_label("Vuelta 1: hecha");
    h.get_by_label("Comprobaciones: 2 de 2");
    h.get_by_label("Claude Code · 240 tokens");
    assert!(h.get_by_label("Detener").accesskit_node().is_disabled());
}

/// Un proveedor que nunca contesta.
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

/// «Detener» cancela la ejecución.
#[test]
fn stop_cancels_the_run() {
    let w = world();
    let backend = Arc::new(RealAgent {
        provider: Arc::new(Silent),
        session: w.session.handle.clone(),
    });
    let mut h = app(&w, backend, Screen::Studio);
    send(&mut h, "un cofre");
    h.get_by_label("Pensando…");
    h.get_by_label("Detener").click();
    wait_until(&mut h, "la cancelación", |h| {
        h.query_by_label_contains("Detenido en la vuelta 1.")
            .is_some()
    });
    assert!(!h.state().agent.running());
    assert_eq!(
        w.session.handle.view().borrow().canvas,
        Arc::new(Canvas::new(16, 16).unwrap())
    );
    // Se puede volver a pedir.
    h.get_by_label("Mensaje para el agente").focus();
    h.step();
    h.get_by_label("Mensaje para el agente")
        .type_text("otra cosa");
    h.step();
    assert!(!h.get_by_label("Enviar").accesskit_node().is_disabled());
}

/// Runner que apunta los encargos y termina enseguida.
#[derive(Default)]
struct Recorder {
    tasks: Mutex<Vec<AgentTask>>,
}

impl AgentRunner for Recorder {
    fn run<'a>(
        &'a self,
        task: AgentTask,
        _tools: ToolBox,
        _events: EventSink,
        _cancel: CancellationToken,
    ) -> BoxFuture<'a, Result<RunSummary, PortError>> {
        self.tasks.lock().unwrap().push(task);
        Box::pin(async {
            Ok(RunSummary {
                steps: 1,
                usage: Usage::default(),
                final_text: "listo".into(),
                end: RunEnd::Finished,
                conversation: None,
                loops: None,
            })
        })
    }
}

/// Cambiar de proveedor en el panel se guarda y lo usa la siguiente
/// ejecución.
#[test]
fn changing_provider_is_used_by_the_next_run() {
    // Una vuelta: el runner de prueba no dibuja.
    let w = world_with(Settings {
        max_turns: 1,
        ..Settings::default()
    });
    let recorder = Arc::new(Recorder::default());
    let backend = Arc::new(FakeBackend::new(
        recorder.clone(),
        Arc::new(FakeTools::default()),
    ));
    let mut h = app(&w, backend.clone(), Screen::Studio);

    send(&mut h, "primero");
    wait_until(&mut h, "fin", |h| {
        !h.state().agent.running()
            && h.query_by_label_contains("Se hizo la única vuelta")
                .is_some()
    });
    assert_eq!(
        backend.prepared.lock().unwrap()[0].0,
        ProviderKind::ClaudeCode
    );

    h.get_by_label("Proveedor del agente").click();
    h.run_steps(2);
    h.get_by_label("Anthropic API").click();
    h.run_steps(3);
    assert_eq!(w.settings.load().unwrap().provider, ProviderKind::Anthropic);

    send(&mut h, "segundo");
    wait_until(&mut h, "fin", |h| {
        !h.state().agent.running()
            && h.query_all_by_label_contains("Se hizo la única vuelta")
                .count()
                == 2
    });
    let prepared = backend.prepared.lock().unwrap().clone();
    assert_eq!(prepared[1].0, ProviderKind::Anthropic);
    assert_eq!(prepared[1].1, "claude-opus-5-5");
    let tasks = recorder.tasks.lock().unwrap();
    assert_eq!(tasks[1].goal, "segundo");
    assert_eq!(tasks[1].model, "claude-opus-5-5");
}

/// Desde Inicio se crea el lienzo del tamaño elegido y se le pide al agente
/// en su panel; las imágenes adjuntas en el panel van en el encargo.
#[test]
fn draw_from_home_and_attachments() {
    let w = world_with(Settings {
        max_turns: 1,
        ..Settings::default()
    });
    let recorder = Arc::new(Recorder::default());
    let backend = Arc::new(FakeBackend::new(
        recorder.clone(),
        Arc::new(FakeTools::default()),
    ));
    let mut h = app(&w, backend, Screen::Home);
    h.get_by_label("Nuevo pixel art…").click();
    h.run_steps(2);
    h.get_by_label("24 × 24").click();
    h.run_steps(2);
    h.get_by_label("Crear lienzo en blanco").click();
    wait_until(&mut h, "el lienzo nuevo", |h| {
        h.state().controller.session.view().borrow().canvas.width() == 24
    });
    // Unos fotogramas para que se cierre el diálogo de Inicio.
    h.run_steps(4);
    send(&mut h, "un cofre del tesoro");
    wait_until(&mut h, "el encargo", |_| {
        !recorder.tasks.lock().unwrap().is_empty()
    });
    {
        let tasks = recorder.tasks.lock().unwrap();
        assert_eq!(tasks[0].goal, "un cofre del tesoro");
        assert_eq!(tasks[0].canvas, (24, 24), "el tamaño elegido en Inicio");
    }

    // Adjuntar una referencia en el panel.
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("referencia.png");
    std::fs::write(&image, b"PNG-REF").unwrap();
    *w.dialogs.image.lock().unwrap() = Some(image);
    wait_until(&mut h, "fin", |h| !h.state().agent.running());
    h.get_by_label("Adjuntar referencia").click();
    h.run_steps(2);
    h.get_by_label_contains("referencia.png");
    send(&mut h, "con esta referencia");
    wait_until(&mut h, "segundo encargo", |_| {
        recorder.tasks.lock().unwrap().len() == 2
    });
    let tasks = recorder.tasks.lock().unwrap();
    assert_eq!(tasks[1].attachments.len(), 1);
    assert_eq!(tasks[1].attachments[0].label, "referencia.png");
    assert_eq!(tasks[1].attachments[0].data, b"PNG-REF");
}

// --- T:40: el bucle en la interfaz ------------------------------------------

/// Gated con un guion de dos vueltas: dibuja en cada una y promete en la 2.
fn two_turns(gate: Arc<tokio::sync::Semaphore>) -> Arc<Gated> {
    Arc::new(Gated {
        inner: FakeProvider::new([
            reply(vec![draw("t1", "fill_rect 2 8 12 6 #8b3fe0")]),
            reply(vec![ContentPart::Text("Vuelta uno.".into())]),
            reply(vec![draw("t2", "fill_rect 6 2 4 4 #a86b3c")]),
            reply(vec![ContentPart::Text(
                "Listo. <promise>LISTO</promise>".into(),
            )]),
        ]),
        gate,
    })
}

/// Las casillas reflejan vueltas hechas, en curso y pendientes, y la
/// pastilla dice la vuelta.
#[test]
fn loop_boxes_follow_the_turns() {
    let w = world_with(Settings {
        max_turns: 3,
        ..Settings::default()
    });
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let backend = Arc::new(RealAgent {
        provider: two_turns(gate.clone()),
        session: w.session.handle.clone(),
    });
    let mut h = app(&w, backend, Screen::Studio);
    send(&mut h, "una poción");
    h.get_by_label("Bucle de refinado");
    h.get_by_label("vuelta 1 de 3");
    h.get_by_label("Vuelta 1: en curso");
    h.get_by_label("Vuelta 2: pendiente");
    h.get_by_label("Vuelta 3: pendiente");
    h.get_by_label("Contexto vacío en cada vuelta");

    gate.add_permits(2);
    wait_until(&mut h, "la vuelta 2", |h| has(h, "Vuelta 2: en curso"));
    h.get_by_label("Vuelta 1: hecha");
    h.get_by_label("Vuelta 3: pendiente");
    h.get_by_label("Vuelta 2 de 3");
    h.get_by_label("Comprobaciones: 2 de 2");

    gate.add_permits(2);
    wait_until(&mut h, "el final", |h| !h.state().agent.running());
    h.get_by_label("Vuelta 1: hecha");
    h.get_by_label("Vuelta 2: hecha");
    h.get_by_label("Vuelta 3: pendiente");
    h.get_by_label("vuelta 2 de 3");
}

/// Clic en una vuelta muestra esa versión en el lienzo (sin tocar el
/// dibujo); «Volver a esta» la restaura.
#[test]
fn a_turn_shows_its_version_and_can_be_restored() {
    let w = world_with(Settings {
        max_turns: 3,
        ..Settings::default()
    });
    let gate = Arc::new(tokio::sync::Semaphore::new(4));
    let backend = Arc::new(RealAgent {
        provider: two_turns(gate),
        session: w.session.handle.clone(),
    });
    let mut h = app(&w, backend, Screen::Studio);
    send(&mut h, "una poción");
    wait_until(&mut h, "el final", |h| !h.state().agent.running());
    let doc = w.runtime.block_on(w.session.handle.snapshot()).unwrap();
    let v1 = doc.version(1).unwrap().canvas.clone();
    let current = doc.canvas().clone();
    assert_ne!(v1, current);

    h.get_by_label("Vuelta 1: hecha").click();
    h.run_steps(3);
    h.get_by_label("Versión 1 de 2");
    h.get_by_label("Vuelta 1 de 3");
    let preview = h
        .state()
        .shell
        .preview
        .clone()
        .expect("viendo la versión 1");
    assert_eq!(*preview.canvas, v1);
    assert_eq!(
        *w.session.handle.view().borrow().canvas,
        current,
        "solo se ve; el dibujo no cambia"
    );

    // Siguiente y anterior.
    h.get_by_label("Versión siguiente").click();
    h.run_steps(3);
    h.get_by_label("Versión 2 de 2");
    h.get_by_label("Versión anterior").click();
    h.run_steps(3);
    h.get_by_label("Versión 1 de 2");

    h.get_by_label("Volver a esta").click();
    h.run_steps(3);
    assert!(h.state().shell.preview.is_none());
    assert_eq!(*w.session.handle.view().borrow().canvas, v1);
    h.get_by_label("De vuelta a la versión 1");

    // «Ver la actual» solo cierra la vista.
    h.get_by_label("Vuelta 2: hecha").click();
    h.run_steps(3);
    h.get_by_label("Ver la actual").click();
    h.run_steps(3);
    assert!(h.state().shell.preview.is_none());
    assert_eq!(*w.session.handle.view().borrow().canvas, v1);
}

/// Los ajustes del bucle (vueltas, contexto) se usan en la siguiente
/// ejecución.
#[test]
fn loop_settings_apply_to_the_next_run() {
    let w = world();
    let recorder = Arc::new(Recorder::default());
    let backend = Arc::new(FakeBackend::new(
        recorder.clone(),
        Arc::new(FakeTools::default()),
    ));
    let mut h = app(&w, backend, Screen::Studio);
    let changed = Settings {
        max_turns: 2,
        context: lume_app::ContextPolicy::Resume,
        stop_without_progress: false,
        ..Settings::default()
    };
    h.state_mut()
        .controller
        .handle(lume_ui::Intent::UpdateSettings(changed.clone()));
    assert_eq!(w.settings.load().unwrap(), changed);

    send(&mut h, "un cofre");
    wait_until(&mut h, "el final", |h| !h.state().agent.running());
    h.get_by_label("vuelta 2 de 2");
    h.get_by_label("Vuelta 2: hecha");
    h.get_by_label("Misma conversación");
    assert_eq!(recorder.tasks.lock().unwrap().len(), 2, "dos vueltas");
    assert!(
        h.query_by_label_contains("Se hicieron las 2 vueltas sin terminar.")
            .is_some()
    );
}

/// Un mensaje escrito mientras dibuja va a la vuelta siguiente sin detener
/// el bucle.
#[test]
fn messages_while_drawing_go_to_the_next_turn() {
    let w = world_with(Settings {
        max_turns: 3,
        ..Settings::default()
    });
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let provider = two_turns(gate.clone());
    let backend = Arc::new(RealAgent {
        provider: provider.clone(),
        session: w.session.handle.clone(),
    });
    let mut h = app(&w, backend, Screen::Studio);
    send(&mut h, "una poción");
    assert!(h.state().agent.running());
    send(&mut h, "con un brillo arriba");
    assert!(h.state().agent.running(), "no se detiene");
    h.get_by_label("con un brillo arriba");
    h.get_by_label("para la próxima vuelta");

    gate.add_permits(4);
    wait_until(&mut h, "el final", |h| !h.state().agent.running());
    let requests = provider.inner.requests.lock().unwrap().clone();
    let ContentPart::Text(turn2) = &requests[2].messages[0].content[0] else {
        panic!()
    };
    assert!(turn2.contains("- con un brillo arriba"), "{turn2}");
}

// --- Pegar imágenes y agrandar la redacción ---------------------------------

/// El studio con un runner que solo apunta los encargos, a 60 fotogramas
/// por segundo (el arnés da cada clic en un fotograma; a 4 por segundo, dos
/// clics nunca serían un doble clic).
fn recorder_app(w: &World) -> (Arc<Recorder>, Harness<'static, StudioApp>) {
    let recorder = Arc::new(Recorder::default());
    let backend = Arc::new(FakeBackend::new(
        recorder.clone(),
        Arc::new(FakeTools::default()),
    ));
    let builder = Harness::builder().with_step_dt(1.0 / 60.0);
    (recorder, app_with(w, backend, Screen::Studio, builder))
}

/// Ctrl+V con una imagen en el portapapeles (egui la entrega como
/// `PasteImage`) la adjunta como referencia PNG, igual que el «+»; sin el
/// foco en el mensaje no hace nada.
#[test]
fn pasting_an_image_attaches_it() {
    let w = world_with(Settings {
        max_turns: 1,
        ..Settings::default()
    });
    let (recorder, mut h) = recorder_app(&w);
    let rgba = [255, 0, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255];
    let image = Arc::new(egui::ColorImage::from_rgba_unmultiplied([3, 1], &rgba));

    h.event(egui::Event::PasteImage(image.clone()));
    h.run_steps(2);
    assert!(h.query_by_label_contains("imagen pegada").is_none());

    h.get_by_label("Mensaje para el agente").focus();
    h.step();
    h.event(egui::Event::PasteImage(image.clone()));
    h.run_steps(2);
    h.get_by_label_contains("imagen pegada");
    h.event(egui::Event::PasteImage(image));
    h.run_steps(2);
    h.get_by_label_contains("imagen pegada 2");

    send(&mut h, "como en la captura");
    wait_until(&mut h, "el encargo", |_| {
        !recorder.tasks.lock().unwrap().is_empty()
    });
    let tasks = recorder.tasks.lock().unwrap();
    let labels: Vec<&str> = tasks[0]
        .attachments
        .iter()
        .map(|a| a.label.as_str())
        .collect();
    assert_eq!(labels, ["imagen pegada", "imagen pegada 2"]);
    let pasted = &tasks[0].attachments[0];
    assert_eq!(pasted.media_type, "image/png");
    assert_eq!(
        lume_ui::decode_png(&pasted.data),
        Some((3, 1, rgba.to_vec())),
        "los mismos píxeles"
    );
}

/// La redacción se agranda arrastrando la línea que la separa de la
/// conversación (sin dejarla sin sitio) y vuelve a su alto con doble clic.
#[test]
fn the_composer_can_be_resized() {
    let w = world();
    let (_recorder, mut h) = recorder_app(&w);
    let before = h.get_by_label("Mensaje para el agente").rect();
    let drag_to = |h: &mut Harness<'static, StudioApp>, dy: f32| {
        let from = h.get_by_label("Alto del mensaje").rect().center();
        h.hover_at(from);
        h.step();
        h.drag_at(from);
        h.step();
        for k in 1..=4 {
            h.hover_at(from + egui::vec2(0.0, dy * k as f32 / 4.0));
            h.step();
        }
        h.drop_at(from + egui::vec2(0.0, dy));
        h.run_steps(2);
    };

    drag_to(&mut h, -150.0);
    let taller = h.get_by_label("Mensaje para el agente").rect();
    assert!(
        (taller.height() - before.height() - 150.0).abs() <= 1.0,
        "{before:?} → {taller:?}"
    );
    assert!((taller.bottom() - before.bottom()).abs() <= 1.0);

    // Hasta arriba del todo: la conversación conserva su sitio.
    drag_to(&mut h, -2000.0);
    let tallest = h.get_by_label("Mensaje para el agente").rect();
    let header = h.get_by_label("Proveedor del agente").rect();
    assert!(
        tallest.top() - header.bottom() >= 80.0,
        "{header:?} {tallest:?}"
    );

    // Doble clic: el alto de siempre.
    for _ in 0..2 {
        h.get_by_label("Alto del mensaje").click();
        h.step();
    }
    h.run_steps(2);
    let reset = h.get_by_label("Mensaje para el agente").rect();
    assert!((reset.height() - before.height()).abs() <= 1.0, "{reset:?}");
}

/// «+ N comandos» despliega el bloque entero y «Ocultar» lo vuelve a plegar.
#[test]
fn a_tool_block_unfolds_from_its_more_line() {
    let w = world_with(Settings {
        max_turns: 1,
        ..Settings::default()
    });
    let provider = Arc::new(FakeProvider::new([
        reply(vec![draw(
            "t1",
            "set_pixel 1 1 #ff0000\nset_pixel 2 2 #ff0000\nset_pixel 3 3 #ff0000\nset_pixel 4 4 #ff0000\nset_pixel 5 5 #ff0000",
        )]),
        reply(vec![ContentPart::Text("<promise>LISTO</promise>".into())]),
    ]));
    let backend = Arc::new(RealAgent {
        provider,
        session: w.session.handle.clone(),
    });
    let mut h = app(&w, backend, Screen::Studio);
    send(&mut h, "puntos");
    wait_until(&mut h, "el resultado", |h| has(h, "5 ok"));
    // Cada línea: nombre, argumentos y color; plegado se ven las tres primeras.
    h.get_by_label("3 3");
    assert!(h.query_by_label("5 5").is_none());
    h.get_by_label("+ 2 comandos ▾").click();
    // La conversación baja sola hasta el final del bloque: se deja asentar.
    h.run_steps(8);
    h.get_by_label("5 5");
    h.get_by_label("Ocultar ▴").click();
    h.run_steps(8);
    assert!(h.query_by_label("5 5").is_none());
    h.get_by_label("+ 2 comandos ▾");
}

/// Un historial que ya estaba antes de abrir la ventana (como al abrir un
/// archivo) tiene miniaturas en su pestaña: se sacan del documento.
#[test]
fn the_history_tab_has_a_thumbnail_per_step() {
    let w = world();
    for script in [
        "fill_rect 0 0 4 4 #ff0000",
        "fill_rect 4 4 4 4 #00ff00",
        "set_pixel 15 15 #0000ff",
    ] {
        w.runtime
            .block_on(w.session.handle.batch(script, Origin::Cli, None))
            .unwrap();
    }
    let (_recorder, mut h) = recorder_app(&w);
    h.get_by_role_and_label(egui::Role::Tab, "Historial")
        .click();
    h.run_steps(3);
    assert_eq!(h.get_all_by_label("batch · 1 comando — cli").count(), 3);
    let thumbs = &h.state().shell.history_thumbs;
    for i in 0..3 {
        assert!(thumbs.canvas(i).is_some(), "miniatura {i}");
    }
    // Cada una es el dibujo tras su paso.
    assert_eq!(thumbs.canvas(0).unwrap().get(5, 5).unwrap().a, 0);
    assert_eq!(thumbs.canvas(1).unwrap().get(5, 5).unwrap().g, 255);
    assert_eq!(thumbs.canvas(1).unwrap().get(15, 15).unwrap().a, 0);
    assert_eq!(thumbs.canvas(2).unwrap().get(15, 15).unwrap().b, 255);
}
