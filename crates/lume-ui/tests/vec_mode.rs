//! Modo vector con el `StudioApp` real, el motor de `dibujo.js` de verdad
//! (QuickJS), el rasterizador y el buscador de formas (resvg) y el agente
//! real con respuestas grabadas (`ApiRunner` + `FakeProvider`): el lazo dice
//! qué formas quedan debajo y su línea en `dibujo.js`, y cada cambio del
//! agente guarda una versión y se puede deshacer.

use std::sync::Arc;
use std::time::Duration;

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use lume_agent::{ApiRunner, SessionTools, VecTools};
use lume_app::ports::{
    CancellationToken, ContentPart, ModelResponse, PortError, PreparedAgent, StopReason, Usage,
};
use lume_app::testing::{FakeProvider, MemorySecretStore, MemorySettingsStore, MemoryStore};
use lume_app::{AgentBackend, ModelProvider, Session, Settings};
use lume_domain::vec::drawing::{VecEdit, VectorDrawing};
use lume_domain::{Document, Origin};
use lume_raster::ResvgRasterizer;
use lume_script::QuickJsVec;
use lume_ui::{FakeDialogs, Mode, Screen, StudioApp, StudioOptions, VecServices, VecTool};

/// Un mar con una ola blanca (líneas 3 y 4), un sol (línea 7) y dos flores
/// que salen de una función.
const DRAWING: &str = r##"canvas({ width: 400, height: 300, background: "#102030" })
layer("mar", () => {
  rect(0, 150, 400, 150, { fill: "#2f6f73" })
  circle(100, 200, 30, { fill: "#ffffff" })
})
layer("sol", () => {
  circle(300, 80, 40, { fill: "#e8803a" })
})
function flor(x, y) {
  circle(x, y, 10, { fill: "#ffcc00" })
}
flor(40, 40)
flor(80, 40)
"##;

/// Lo que escribe el agente: la ola, más grande (cambia la línea 4).
const RETOUCHED: &str = r##"canvas({ width: 400, height: 300, background: "#102030" })
layer("mar", () => {
  rect(0, 150, 400, 150, { fill: "#2f6f73" })
  circle(100, 200, 36, { fill: "#f4fbff" })
})
layer("sol", () => {
  circle(300, 80, 40, { fill: "#e8803a" })
})
function flor(x, y) {
  circle(x, y, 10, { fill: "#ffcc00" })
}
flor(40, 40)
flor(80, 40)
"##;

struct World {
    runtime: tokio::runtime::Runtime,
    session: Session,
}

fn world(drawing: Option<&str>) -> World {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let session = {
        let _guard = runtime.enter();
        Session::spawn(
            Document::new("mar", 16, 16).unwrap(),
            Arc::new(MemoryStore::new()),
        )
    };
    if let Some(source) = drawing {
        runtime
            .block_on(
                session
                    .handle
                    .edit_vector(VecEdit::Set(Some(VectorDrawing::new(source))), Origin::User),
            )
            .unwrap();
    }
    World { runtime, session }
}

fn services() -> VecServices {
    VecServices {
        exporter: None,
        engine: Arc::new(QuickJsVec::default()),
        rasterizer: Arc::new(ResvgRasterizer),
        shapes: Arc::new(ResvgRasterizer),
    }
}

/// `AgentBackend` con el `ApiRunner` real sobre respuestas grabadas y las
/// herramientas de verdad (con las de `dibujo.js`).
struct RealAgent {
    provider: Arc<FakeProvider>,
    session: lume_app::SessionHandle,
}

impl AgentBackend for RealAgent {
    fn prepare(&self, _settings: &Settings, run: u64) -> Result<PreparedAgent, PortError> {
        let tools = SessionTools::new(self.session.clone(), Arc::new(MemoryStore::new()), run)
            .with_vec(VecTools {
                engine: Arc::new(QuickJsVec::default()),
                rasterizer: Arc::new(ResvgRasterizer),
            });
        Ok(PreparedAgent {
            runner: Arc::new(ApiRunner::new(
                self.provider.clone() as Arc<dyn ModelProvider>
            )),
            tools: Arc::new(tools),
            model: "modelo-de-prueba".into(),
        })
    }

    fn prepare_critic(&self, settings: &Settings, run: u64) -> Result<PreparedAgent, PortError> {
        self.prepare(settings, run)
    }
}

fn app(w: &World, agent: Option<Arc<dyn AgentBackend>>) -> Harness<'static, StudioApp> {
    let options = StudioOptions {
        session: w.session.handle.clone(),
        stop: CancellationToken::new(),
        runtime: w.runtime.handle().clone(),
        store: Arc::new(MemoryStore::new()),
        recents: None,
        dialogs: Arc::new(FakeDialogs::default()),
        settings: Arc::new(MemorySettingsStore::default()),
        secrets: Arc::new(MemorySecretStore::default()),
        claude_path: None,
        agent,
        server: None,
        screen: Screen::Studio,
        art: None,
        anim_engine: None,
        variant_agents: None,
        vector: Some(services()),
        autosave: None,
        window: None,
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_eframe(move |cc| StudioApp::new(cc, options));
    harness.run_steps(2);
    harness.get_by_label("Vector").click();
    harness.run_steps(2);
    assert_eq!(harness.state().shell.mode, Mode::Vector);
    harness
}

/// Avanza fotogramas hasta que se cumpla `cond` (máximo ~10 s).
fn wait_until(
    h: &mut Harness<'static, StudioApp>,
    what: &str,
    cond: impl Fn(&Harness<'static, StudioApp>) -> bool,
) {
    for _ in 0..1000 {
        h.step();
        if cond(h) {
            h.step();
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("no llegó: {what}");
}

fn wait_for_drawing(h: &mut Harness<'static, StudioApp>, source: &str) {
    wait_until(h, "el dibujo", |h| {
        h.state()
            .shell
            .vec
            .view
            .as_ref()
            .is_some_and(|v| v.source == source && v.image.is_ok())
    });
}

fn button(h: &mut Harness<'static, StudioApp>, pos: egui::Pos2, pressed: bool) {
    h.event(egui::Event::PointerMoved(pos));
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
}

/// Arrastra el ratón por el contorno de `points` (en unidades del dibujo),
/// como se rodea una zona con el lazo.
fn lasso(h: &mut Harness<'static, StudioApp>, points: &[[f64; 2]]) {
    let screen = |p: [f64; 2]| h.state().shell.vec.screen_point(p).unwrap();
    let mut path = Vec::new();
    for (i, a) in points.iter().enumerate() {
        let b = points[(i + 1) % points.len()];
        for k in 0..8 {
            let t = f64::from(k) / 8.0;
            path.push(screen([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]));
        }
    }
    path.push(path[0]);
    button(h, path[0], true);
    for p in &path[1..] {
        h.event(egui::Event::PointerMoved(*p));
        h.step();
    }
    button(h, *path.last().unwrap(), false);
    h.run_steps(2);
}

/// Escribe en el recuadro de la zona.
fn type_zone(h: &mut Harness<'static, StudioApp>, text: &str) {
    h.get_by_label("Qué cambiar en la zona").focus();
    h.step();
    h.get_by_label("Qué cambiar en la zona").type_text(text);
    h.step();
}

/// Un lazo alrededor de la ola: el recuadro dice qué formas hay debajo, el
/// panel de código señala sus líneas (3 y 4) y «Quitar» la suelta. Con
/// «Añadir a la cola» pasa a la cola de correcciones (numeradas; se pueden
/// quitar antes de ejecutarlas), sin pedir nada al agente.
#[test]
fn the_lasso_finds_the_shapes_under_it_and_their_lines() {
    let w = world(Some(DRAWING));
    let mut h = app(&w, None);
    wait_for_drawing(&mut h, DRAWING);
    h.get_by_label("CAPAS");
    h.get_by_label("Ocultar la capa «mar»");
    h.get_by_label("Ocultar la capa «sol»");

    h.key_press(egui::Key::L);
    h.step();
    assert_eq!(h.state().shell.vec.tool, VecTool::Lasso);
    let wave = [[60.0, 160.0], [140.0, 160.0], [140.0, 240.0], [60.0, 240.0]];
    lasso(&mut h, &wave);

    let zone = h.state().shell.vec.zone.clone().expect("la zona");
    assert_eq!(zone.number, 1);
    assert_eq!(zone.zone.summary(), "2 formas de la capa «mar»");
    assert_eq!(zone.zone.lines(), [3, 4]);
    assert_eq!(zone.zone.main_layer(), Some("mar"));
    // El fondo (línea 1) pasa por la zona, pero no es de ella.
    assert!(zone.zone.hits.iter().any(|hit| hit.line == 1 && hit.big));
    h.get_by_label("Corrección 1");
    h.get_by_label(" · 2 formas de la capa «mar»");
    h.get_by_label("corrección 1 · líneas 3, 4");
    h.get_by_label("Qué cambiar en la zona");

    h.get_by_label("Quitar").click();
    h.run_steps(2);
    assert!(h.state().shell.vec.zone.is_none());
    assert!(h.query_by_label("corrección 1 · líneas 3, 4").is_none());

    // Otra vez la ola, ahora a la cola.
    lasso(&mut h, &wave);
    type_zone(&mut h, "una ola más grande");
    h.get_by_label("Añadir a la cola").click();
    h.run_steps(2);
    assert!(h.state().shell.vec.zone.is_none());
    let queue = &h.state().shell.vec.corrections;
    assert_eq!(queue.len(), 1);
    assert_eq!(
        (queue[0].number, queue[0].text.as_str()),
        (1, "una ola más grande")
    );
    assert!(!queue[0].sent, "no va al agente hasta ejecutarla");

    // La segunda, alrededor de las flores (una línea por llamada).
    lasso(
        &mut h,
        &[[20.0, 20.0], [100.0, 20.0], [100.0, 60.0], [20.0, 60.0]],
    );
    let zone = h.state().shell.vec.zone.clone().expect("la zona 2");
    assert_eq!(zone.number, 2);
    assert_eq!(zone.zone.summary(), "2 formas");
    assert_eq!(zone.zone.lines(), [10]);
    h.get_by_label("corrección 2 · línea 10");
    type_zone(&mut h, "flores rojas");
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().shell.vec.corrections.len(), 2);
    h.get_by_label("una ola más grande");
    h.get_by_label("flores rojas");
    assert!(
        !h.get_by_label("Ejecutar correcciones")
            .accesskit_node()
            .is_disabled()
    );

    // Quitar la primera: la otra pasa a ser la 1.
    h.get_by_label("Quitar la corrección 1").click();
    h.run_steps(2);
    let queue = &h.state().shell.vec.corrections;
    assert_eq!(queue.len(), 1);
    assert_eq!(
        (queue[0].number, queue[0].text.as_str()),
        (1, "flores rojas")
    );
    assert!(h.query_by_label("una ola más grande").is_none());
}

/// Con la herramienta de selección, un clic dice qué forma hay y su línea.
#[test]
fn a_click_shows_the_shape_and_its_line() {
    let w = world(Some(DRAWING));
    let mut h = app(&w, None);
    wait_for_drawing(&mut h, DRAWING);
    assert_eq!(h.state().shell.vec.tool, VecTool::Select);
    let at = h.state().shell.vec.screen_point([300.0, 80.0]).unwrap();
    button(&mut h, at, true);
    button(&mut h, at, false);
    h.run_steps(2);
    let hit = h.state().shell.vec.selected.clone().expect("la forma");
    assert_eq!((hit.line, hit.layer.as_str()), (7, "sol"));
    h.get_by_label("línea 7 · circle · capa «sol»");
}

/// Ocultar una capa la quita del dibujo al instante: se ven los trozos del
/// dibujo (capa a capa) sin el de esa capa.
#[test]
fn hiding_a_layer_repaints_without_it() {
    let w = world(Some(DRAWING));
    let mut h = app(&w, None);
    wait_for_drawing(&mut h, DRAWING);
    // El centro del sol, en una imagen del ancho del dibujo (400).
    let at = |width: u32| {
        let k = f64::from(width) / 400.0;
        let (x, y) = ((300.0 * k) as usize, (80.0 * k) as usize);
        (y * width as usize + x) * 4
    };
    let pixel = |h: &Harness<'static, StudioApp>| {
        let view = h.state().shell.vec.view.clone().unwrap();
        let image = view.image.as_ref().unwrap();
        let i = at(image.width);
        image.rgba[i..i + 3].to_vec()
    };
    assert_eq!(pixel(&h), [0xe8, 0x80, 0x3a]);
    h.get_by_label("Ocultar la capa «sol»").click();
    h.step();
    // Lo que se ve: los trozos de las capas visibles, uno sobre otro.
    let shown = |h: &Harness<'static, StudioApp>| -> Option<[u8; 3]> {
        let vec = &h.state().shell.vec;
        let key = vec.view.as_ref()?.key;
        let (slice_key, slices) = vec.slices.as_ref()?;
        if *slice_key != key {
            return None;
        }
        let mut rgb = [0.0f64; 3];
        for slice in slices.iter() {
            if slice.layer.is_some_and(|l| vec.hidden.contains(&l)) {
                continue;
            }
            let i = at(slice.image.width);
            let p = &slice.image.rgba[i..i + 4];
            let a = f64::from(p[3]) / 255.0;
            for c in 0..3 {
                rgb[c] = f64::from(p[c]) * a + rgb[c] * (1.0 - a);
            }
        }
        Some(rgb.map(|v| v.round() as u8))
    };
    wait_until(&mut h, "el dibujo sin el sol", |h| {
        h.state().shell.vec.hidden.len() == 1 && shown(h) == Some([0x10, 0x20, 0x30])
    });
    // El dibujo entero no cambia: el sol sigue en él.
    assert_eq!(pixel(&h), [0xe8, 0x80, 0x3a]);
    h.get_by_label("Mostrar la capa «sol»");
}

/// Sin dibujo vectorial, «Crear dibujo vectorial» pone la plantilla (un
/// paso de deshacer).
#[test]
fn creating_the_vector_drawing_is_undoable() {
    let w = world(None);
    let mut h = app(&w, None);
    h.get_by_label("Nuevo dibujo vectorial");
    h.get_by_label("Crear dibujo vectorial").click();
    h.run_steps(2);
    let template = lume_domain::vec::template(1024, 1024, false);
    let vector = w.session.handle.view().borrow().vector.clone();
    assert_eq!(vector.unwrap().source, template);
    wait_for_drawing(&mut h, &template);
    // En la cabecera y en la barra de estado: el tamaño del dibujo.
    assert_eq!(h.query_all_by_label("1024 × 1024").count(), 2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
    h.run_steps(3);
    wait_until(&mut h, "deshecho", |h| h.state().shell.vec.view.is_none());
    assert!(w.session.handle.view().borrow().vector.is_none());
}

/// El dibujo en blanco del tamaño y la proporción elegidos.
#[test]
fn the_blank_drawing_takes_the_chosen_size() {
    let w = world(None);
    let mut h = app(&w, None);
    h.get_by_label("1920 × 1080").click();
    h.run_steps(2);
    assert_eq!(h.state().shell.vec.new_size.size(), Ok((1920, 1080)));
    assert_eq!(h.state().shell.vec.new_size.ratio, Some((16, 9)));
    h.get_by_label("Fondo transparente (un objeto suelto)")
        .click();
    h.run_steps(2);
    h.get_by_label("Crear dibujo vectorial").click();
    h.run_steps(2);
    let vector = w.session.handle.view().borrow().vector.clone();
    assert_eq!(
        vector.unwrap().source,
        lume_domain::vec::template(1920, 1080, true)
    );
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

/// Ejecutar la corrección de la cola: el agente recibe la petición con su
/// número, la zona, sus formas y sus líneas; su cambio de `dibujo.js` es una
/// versión nueva, el panel de código señala las líneas que tocó y Ctrl+Z lo
/// deshace.
#[test]
fn each_agent_change_is_a_version_and_can_be_undone() {
    let w = world(Some(DRAWING));
    let provider = Arc::new(FakeProvider::new([
        reply(vec![
            ContentPart::Text("Hago la ola más grande y más blanca.".into()),
            ContentPart::ToolUse {
                id: "t1".into(),
                name: "write_vec".into(),
                input_json: serde_json::json!({ "source": RETOUCHED }).to_string(),
            },
        ]),
        reply(vec![ContentPart::Text(
            "Ola retocada. <promise>LISTO</promise>".into(),
        )]),
    ]));
    let backend = Arc::new(RealAgent {
        provider: provider.clone(),
        session: w.session.handle.clone(),
    });
    let mut h = app(&w, Some(backend));
    wait_for_drawing(&mut h, DRAWING);
    h.state_mut().shell.vec.tool = VecTool::Lasso;
    h.step();
    lasso(
        &mut h,
        &[[60.0, 160.0], [140.0, 160.0], [140.0, 240.0], [60.0, 240.0]],
    );
    assert!(h.state().shell.vec.zone.is_some());
    let versions_before = w.session.handle.view().borrow().versions.len();

    type_zone(&mut h, "una ola más grande");
    h.get_by_label("Añadir a la cola").click();
    h.run_steps(2);
    assert!(provider.requests.lock().unwrap().is_empty(), "aún no");
    h.get_by_label("Ejecutar correcciones").click();
    h.run_steps(2);
    // La conversación muestra lo que escribió el usuario, con su número.
    h.get_by_label_contains("1. una ola más grande");

    wait_until(&mut h, "el final de la ejecución", |h| {
        !h.state().agent.running()
            && h.state()
                .shell
                .vec
                .view
                .as_ref()
                .is_some_and(|v| v.source == RETOUCHED)
    });

    // El encargo llevaba la zona: sus formas y sus líneas.
    let requests = provider.requests.lock().unwrap();
    let first = format!("{:?}", requests[0].messages);
    assert!(first.contains("una ola más grande"), "{first}");
    assert!(
        first.contains("Corrección 1: una ola más grande"),
        "{first}"
    );
    assert!(first.contains("línea 4"), "{first}");
    drop(requests);

    // Una versión por el cambio del agente (y la de la vuelta).
    let vm = w.session.handle.view().borrow().clone();
    assert_eq!(vm.vector.as_ref().unwrap().source, RETOUCHED);
    let messages: Vec<&str> = vm.versions[versions_before..]
        .iter()
        .map(|v| v.message.as_str())
        .collect();
    assert!(messages.contains(&"Agente · línea 4"), "{messages:?}");
    assert_eq!(h.state().shell.vec.agent_lines, Some((4, 4)));
    h.get_by_label("el agente edita la línea 4");

    // Ctrl+Z vuelve al dibujo de antes.
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
    h.run_steps(2);
    wait_for_drawing(&mut h, DRAWING);
    assert_eq!(
        w.session
            .handle
            .view()
            .borrow()
            .vector
            .as_ref()
            .unwrap()
            .source,
        DRAWING
    );
    assert_eq!(h.state().shell.vec.agent_lines, None);

    // Y la versión del agente sigue ahí: volver a ella trae su dibujo.js.
    let number = vm
        .versions
        .iter()
        .find(|v| v.message == "Agente · línea 4")
        .unwrap()
        .number;
    h.state_mut().shell.preview = None;
    let restored = w.runtime.block_on(
        w.session
            .handle
            .restore_version(number, lume_domain::Origin::User),
    );
    assert!(restored.is_ok());
    wait_for_drawing(&mut h, RETOUCHED);
}

/// «Repetir trazado» vuelve a pintar el dibujo forma a forma y termina en
/// el dibujo entero.
#[test]
fn repeating_the_drawing_goes_shape_by_shape() {
    let w = world(Some(DRAWING));
    let mut h = app(&w, None);
    wait_for_drawing(&mut h, DRAWING);
    h.get_by_label("Repetir trazado").click();
    h.step();
    let mut shown = Vec::new();
    for _ in 0..1000 {
        h.step();
        if let Some(frame) = &h.state().shell.vec.replay {
            if shown.last() != Some(&frame.shown) {
                shown.push(frame.shown);
            }
        } else if !shown.is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    // Cinco formas: el fondo, el mar, la ola, el sol y dos flores.
    let total = lume_domain::vec::studio::shape_count(
        &h.state().shell.vec.view.as_ref().unwrap().result.svg,
    );
    assert!(shown.len() >= 2, "{shown:?}");
    assert!(shown.windows(2).all(|w| w[0] < w[1]), "{shown:?}");
    assert_eq!(shown.last(), Some(&total), "{shown:?}");
    assert!(h.state().shell.vec.replay.is_none());
}
