//! Encargos por MCP en la ventana: la lista los enseña mientras se hacen y
//! abrir uno muestra su lienzo y su panel del agente; desde ahí se le
//! pueden mandar mensajes, seguirlo y volver al dibujo propio.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use eframe::egui;
use egui::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use lume_agent::SessionTools;
use lume_app::art::{ArtAgent, ArtAgents};
use lume_app::ports::{
    AgentRunner, AgentTask, BoxFuture, CancellationToken, EventSink, PortError, RunSummary, ToolBox,
};
use lume_app::testing::{
    FakeBackend, MemorySecretStore, MemorySettingsStore, MemoryStore, ScriptedRunner, ScriptedStep,
};
use lume_app::{
    ArtId, ArtService, ArtSpec, ArtState, DocumentStore, LocalArt, Session, SessionHandle, Settings,
};
use lume_domain::{Document, Rgba};
use lume_ui::{FakeDialogs, Screen, StudioApp, StudioOptions};
use tokio::sync::Semaphore;

const BROWN: Rgba = Rgba::rgb(0x8b, 0x5a, 0x2b);

/// Runner que pide permiso al test antes de cada vuelta.
struct Gated {
    gate: Arc<Semaphore>,
    script: ScriptedRunner,
}

impl AgentRunner for Gated {
    fn run<'a>(
        &'a self,
        task: AgentTask,
        tools: ToolBox,
        events: EventSink,
        cancel: CancellationToken,
    ) -> BoxFuture<'a, Result<RunSummary, PortError>> {
        Box::pin(async move {
            self.gate.acquire().await.unwrap().forget();
            self.script.run(task, tools, events, cancel).await
        })
    }
}

/// Dibuja un cofre marrón y lo da por bueno (cuando se le deja).
struct Agents {
    store: Arc<dyn DocumentStore>,
    gate: Arc<Semaphore>,
}

impl ArtAgents for Agents {
    fn agent(
        &self,
        session: SessionHandle,
        id: ArtId,
    ) -> BoxFuture<'_, Result<ArtAgent, PortError>> {
        Box::pin(async move {
            let runner = Arc::new(Gated {
                gate: self.gate.clone(),
                script: ScriptedRunner::new(vec![
                    ScriptedStep::Tool {
                        name: "draw".into(),
                        input_json: r#"{"commands":"fill_rect 2 2 4 4 #8b5a2b"}"#.into(),
                    },
                    ScriptedStep::Say("Cofre listo.\n<promise>LISTO</promise>".into()),
                ]),
            });
            let tools: ToolBox = Arc::new(SessionTools::new(session, self.store.clone(), id));
            Ok(ArtAgent {
                backend: Arc::new(FakeBackend::new(runner, tools)),
                guard: Box::new(()),
            })
        })
    }
}

struct World {
    runtime: tokio::runtime::Runtime,
    main: Session,
    art: Arc<LocalArt>,
    gate: Arc<Semaphore>,
}

fn world() -> World {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let store: Arc<dyn DocumentStore> = Arc::new(MemoryStore::new());
    let gate = Arc::new(Semaphore::new(0));
    let (main, art) = {
        let _guard = runtime.enter();
        let main = Session::spawn(Document::new("mio", 16, 16).unwrap(), store.clone());
        let art = Arc::new(LocalArt::new(
            main.handle.clone(),
            store.clone(),
            Arc::new(MemorySettingsStore::new(Settings {
                max_turns: 3,
                critic: false,
                ..Settings::default()
            })),
            Arc::new(Agents {
                store,
                gate: gate.clone(),
            }),
        ));
        (main, art)
    };
    World {
        runtime,
        main,
        art,
        gate,
    }
}

fn app(w: &World, screen: Screen) -> Harness<'static, StudioApp> {
    let options = StudioOptions {
        session: w.main.handle.clone(),
        stop: CancellationToken::new(),
        runtime: w.runtime.handle().clone(),
        store: Arc::new(MemoryStore::new()),
        recents: None,
        dialogs: Arc::new(FakeDialogs::default()),
        settings: Arc::new(MemorySettingsStore::default()),
        secrets: Arc::new(MemorySecretStore::default()),
        claude_path: None,
        agent: None,
        server: None,
        screen,
        art: Some(w.art.clone()),
        anim_engine: None,
        variant_agents: None,
        vector: None,
        autosave: None,
        window: None,
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_eframe(move |cc| StudioApp::new(cc, options));
    harness.run_steps(3);
    harness
}

/// Un encargo como los que llegan por MCP.
fn order(w: &World, name: &str) -> ArtId {
    let dir = PathBuf::from("/proyecto/assets/art");
    let spec = ArtSpec {
        name: name.into(),
        description: format!("Un {name} de madera que el jugador abre"),
        size: (8, 8),
        view: String::new(),
        style: String::new(),
        palette: None,
        png: dir.join(format!("{name}.png")),
        lume: dir.join(format!("{name}.lume")),
        base: None,
        changes: String::new(),
    };
    w.runtime.block_on(w.art.create(spec)).unwrap().id
}

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

/// ¿Hay algo con esa etiqueta? (Un encargo sale en Inicio y como pestaña.)
fn has(h: &Harness<'static, StudioApp>, label: &str) -> bool {
    h.query_all_by_label(label).next().is_some()
}

/// Pulsa la tarjeta del encargo en Inicio (no su pestaña).
fn click_card(h: &mut Harness<'static, StudioApp>, label: &str) {
    h.get_all_by_label(label)
        .find(|n| n.accesskit_node().role() != Role::Tab)
        .expect("la tarjeta del encargo")
        .click();
}

fn job_state(w: &World, id: ArtId) -> ArtState {
    w.art.jobs().into_iter().find(|s| s.id == id).unwrap().state
}

#[test]
fn a_job_shows_up_while_it_runs() {
    let w = world();
    let mut h = app(&w, Screen::Home);
    assert!(!has(&h, "Encargos"), "sin encargos no hay sección");

    let id = order(&w, "cofre");
    wait_until(&mut h, "la tarjeta del encargo", |h| has(h, "cofre"));
    h.get_by_label("Encargos");
    assert_eq!(h.state().home.jobs.len(), 1);
    assert_eq!(h.state().home.jobs[0].state_text, "vuelta 1 de 3");
    assert_eq!(job_state(&w, id), ArtState::Running);

    // También sale como pestaña; al abrirla, la cabecera y el menú Agente
    // lo listan.
    h.get_by_role_and_label(Role::Tab, "cofre").click();
    h.run_steps(2);
    assert_eq!(h.state().viewing, Some(id));
    h.get_by_label("Encargos · 1 en marcha");
    h.get_by_label("Agente").click();
    h.run_steps(2);
    h.get_by_label("cofre · vuelta 1 de 3");

    // Al terminar, la lista lo dice.
    w.gate.add_permits(1);
    wait_until(&mut h, "el final", |h| has(h, "Encargos · 1"));
    assert_eq!(job_state(&w, id), ArtState::Done);
    assert_eq!(h.state().shell.jobs[0].state_text, "terminado");
}

#[test]
fn opening_a_job_shows_its_canvas_and_its_agent_panel() {
    let w = world();
    let mut h = app(&w, Screen::Home);
    let id = order(&w, "cofre");
    wait_until(&mut h, "la tarjeta", |h| has(h, "cofre"));
    click_card(&mut h, "cofre");
    h.run_steps(3);

    // Se ve el encargo: su archivo, su sesión y su bucle en el panel.
    let state = h.state();
    assert_eq!(state.screen, Screen::Studio);
    assert_eq!(state.viewing, Some(id));
    assert_eq!(state.controller.file_name(), "cofre.lume");
    assert_eq!(state.controller.session.view().borrow().name, "cofre");
    let lb = state
        .shell
        .agent
        .loop_box
        .as_ref()
        .expect("la caja del bucle");
    assert_eq!((lb.current, lb.of), (Some(1), 3));
    h.get_by_label("Un cofre de madera que el jugador abre");
    h.get_by_label("Vuelta 1 de 3");

    // El agente dibuja: el lienzo y el panel lo enseñan.
    w.gate.add_permits(1);
    wait_until(&mut h, "el final", |h| {
        job_state_of(h, id) == ArtState::Done
    });
    wait_until(&mut h, "el final en el panel", |h| {
        h.state()
            .shell
            .agent
            .loop_box
            .as_ref()
            .is_some_and(|lb| lb.current.is_none())
    });
    let state = h.state();
    assert_eq!(
        state
            .controller
            .session
            .view()
            .borrow()
            .canvas
            .get(3, 3)
            .unwrap(),
        BROWN
    );
    let lb = state.shell.agent.loop_box.as_ref().unwrap();
    assert_eq!(lb.done, [(1, 1)]);
    h.get_by_label("Vuelta 1: hecha");
    assert!(
        !state.controller.is_dirty(),
        "el encargo ya escribió sus archivos"
    );
    // El dibujo propio sigue intacto.
    assert!(
        w.main
            .handle
            .view()
            .borrow()
            .canvas
            .pixels()
            .iter()
            .all(|p| p.a == 0)
    );

    // Volver a mi dibujo.
    h.get_by_label("Agente").click();
    h.run_steps(2);
    h.get_by_label("Volver a mi dibujo").click();
    h.run_steps(3);
    let state = h.state();
    assert_eq!(state.viewing, None);
    assert_eq!(state.controller.file_name(), "mio.lume");
    assert!(
        state.shell.agent.loop_box.is_none(),
        "el panel del dibujo propio"
    );

    // Y de nuevo al encargo, desde su pestaña: su panel sigue como estaba.
    h.get_by_role_and_label(Role::Tab, "cofre").click();
    h.run_steps(3);
    assert_eq!(h.state().viewing, Some(id));
    assert_eq!(
        h.state().shell.agent.loop_box.as_ref().unwrap().done,
        [(1, 1)]
    );
}

fn job_state_of(h: &Harness<'static, StudioApp>, id: ArtId) -> ArtState {
    h.state()
        .art
        .as_ref()
        .unwrap()
        .jobs()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap()
        .state
}

fn send(h: &mut Harness<'static, StudioApp>, text: &str) {
    h.get_by_label("Mensaje para el agente").focus();
    h.step();
    h.get_by_label("Mensaje para el agente").type_text(text);
    h.step();
    h.get_by_label("Enviar").click();
    h.run_steps(2);
}

#[test]
fn the_user_can_steer_and_continue_a_job() {
    let w = world();
    let mut h = app(&w, Screen::Home);
    let id = order(&w, "cofre");
    wait_until(&mut h, "la tarjeta", |h| has(h, "cofre"));
    click_card(&mut h, "cofre");
    h.run_steps(3);

    // En marcha: el mensaje va a la vuelta siguiente del encargo.
    send(&mut h, "que la tapa sea más clara");
    h.get_by_label("para la próxima vuelta");
    w.gate.add_permits(1);
    wait_until(&mut h, "el final", |h| {
        job_state_of(h, id) == ArtState::Done
    });

    // Terminado: enviar sigue el encargo sobre el mismo dibujo.
    send(&mut h, "añade un candado dorado");
    wait_until(&mut h, "la segunda ejecución", |h| {
        job_state_of(h, id) == ArtState::Running
    });
    h.get_by_label("añade un candado dorado");
    w.gate.add_permits(1);
    wait_until(&mut h, "el segundo final", |h| {
        job_state_of(h, id) == ArtState::Done
    });
    let doc = w
        .runtime
        .block_on(w.art.session(id).unwrap().snapshot())
        .unwrap();
    assert_eq!(doc.versions().len(), 2);
    assert!(
        w.art.log(id, 0).len() > 4,
        "el registro tiene las dos ejecuciones"
    );
}
