//! Variantes y exportación del modo animación con el `StudioApp` real, el
//! motor de `anim.js` de verdad y un agente de prueba que cambia `anim.js`
//! con la herramienta `write_anim`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use lume_agent::SessionTools;
use lume_app::art::{ArtAgent, ArtAgents};
use lume_app::ports::{BoxFuture, CancellationToken, PortError, ToolBox};
use lume_app::testing::{
    FakeBackend, MemorySecretStore, MemorySettingsStore, MemoryStore, ScriptedRunner, ScriptedStep,
};
use lume_app::{AnimEngine, ArtId, DocumentStore, Session, SessionHandle};
use lume_domain::anim::animation::{AnimEdit, Animation};
use lume_domain::{Document, Origin};
use lume_script::QuickJsAnim;
use lume_ui::shell::VariantCardState;
use lume_ui::{FakeDialogs, Screen, StudioApp, StudioOptions};

const SOURCE: &str = r##"config({ name: "reloj", cell: [12, 12] });
part("marco", () => { fill_rect(2, 0, 8, 1, "#7a4a2a"); fill_rect(2, 11, 8, 1, "#7a4a2a"); });
part("arena", ({ nivel = 0 }) => { fill_rect(4, 2 + nivel, 4, 3 - nivel, "#e8c070"); });
rest(() => { draw("marco"); draw("arena"); });
action("caer", { frames: 4, fps: 10 }, (f) => { draw("marco"); draw("arena", { nivel: Math.min(f, 2) }); });
action("salto", { frames: 3, fps: 8, loop: false }, (f) => { move(0, -f, () => { draw("marco"); }); });
"##;

/// El `anim.js` de una variante: «caer» a otros fps.
fn variant_source(run: ArtId) -> String {
    SOURCE.replace(
        "frames: 4, fps: 10",
        &format!("frames: 4, fps: {}", 4 + run % 10),
    )
}

/// Cada variante: cambia anim.js con `write_anim` y lo resume.
struct VariantAgents {
    store: Arc<dyn DocumentStore>,
}

impl ArtAgents for VariantAgents {
    fn agent(
        &self,
        session: SessionHandle,
        id: ArtId,
    ) -> BoxFuture<'_, Result<ArtAgent, PortError>> {
        Box::pin(async move {
            let input = serde_json::json!({ "source": variant_source(id) }).to_string();
            let runner = Arc::new(ScriptedRunner::new(vec![
                ScriptedStep::Tool {
                    name: "write_anim".into(),
                    input_json: input,
                },
                ScriptedStep::Say(format!("Arena a {} fps.", 4 + id % 10)),
            ]));
            let tools: ToolBox = Arc::new(
                SessionTools::new(session, self.store.clone(), id).with_anim(Arc::new(QuickJsAnim)),
            );
            Ok(ArtAgent {
                backend: Arc::new(FakeBackend::new(runner, tools)),
                guard: Box::new(()),
            })
        })
    }
}

struct World {
    runtime: tokio::runtime::Runtime,
    session: Session,
    store: Arc<MemoryStore>,
    dialogs: Arc<FakeDialogs>,
}

fn world() -> World {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let store = Arc::new(MemoryStore::new());
    let session = {
        let _guard = runtime.enter();
        Session::spawn(Document::new("reloj", 12, 12).unwrap(), store.clone())
    };
    runtime
        .block_on(session.handle.batch(
            "fill_rect 2 0 8 1 #7a4a2a\nfill_rect 2 11 8 1 #7a4a2a\nfill_rect 4 2 4 3 #e8c070",
            Origin::User,
            None,
        ))
        .unwrap();
    runtime
        .block_on(
            session
                .handle
                .edit_animation(AnimEdit::Set(Some(Animation::new(SOURCE))), Origin::User),
        )
        .unwrap();
    World {
        runtime,
        session,
        store,
        dialogs: Arc::new(FakeDialogs::default()),
    }
}

fn app(w: &World) -> Harness<'static, StudioApp> {
    let store: Arc<dyn DocumentStore> = w.store.clone();
    let options = StudioOptions {
        session: w.session.handle.clone(),
        stop: CancellationToken::new(),
        runtime: w.runtime.handle().clone(),
        store: store.clone(),
        recents: None,
        dialogs: w.dialogs.clone(),
        settings: Arc::new(MemorySettingsStore::default()),
        secrets: Arc::new(MemorySecretStore::default()),
        claude_path: None,
        agent: None,
        server: None,
        screen: Screen::Studio,
        art: None,
        anim_engine: Some(Arc::new(QuickJsAnim) as Arc<dyn AnimEngine>),
        variant_agents: Some(Arc::new(VariantAgents { store })),
        vector: None,
        autosave: None,
        window: None,
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_eframe(move |cc| StudioApp::new(cc, options));
    harness.run_steps(2);
    harness.get_by_label("Animación").click();
    harness.run_steps(2);
    wait_until(&mut harness, "la animación dibujada", |h| {
        h.state().shell.anim.result.is_some()
    });
    harness
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

fn all_ready(h: &Harness<'static, StudioApp>, n: usize) -> bool {
    let cards = &h.state().shell.anim.variants;
    cards.len() == n
        && cards
            .iter()
            .all(|c| c.state == VariantCardState::Ready && c.result.is_some())
}

#[test]
fn three_variants_are_made_and_choosing_one_makes_it_current() {
    let w = world();
    let mut h = app(&w);
    // En modo animación, el panel habla de movimiento, no de dibujar.
    h.get_by_label(
        "Describe cómo quieres que se mueva y el agente cambiará la animación, o pide variantes para elegir.",
    );
    h.get_by_label("Mensaje para el agente").focus();
    h.step();
    h.get_by_label("Mensaje para el agente")
        .type_text("que la arena caiga a otro ritmo");
    h.step();
    h.get_by_label("Pedir 3 variantes").click();
    wait_until(&mut h, "las tres variantes", |h| all_ready(h, 3));
    for letter in ["A", "B", "C"] {
        h.get_by_label(&format!("Variante {letter}"));
    }
    h.get_by_label("Variantes de «caer»");
    h.get_by_label("3 variantes listas");
    // Tres candidatas distintas y ninguna es todavía la animación.
    let sources: Vec<String> = h
        .state()
        .shell
        .anim
        .variants
        .iter()
        .map(|c| c.source.clone().unwrap())
        .collect();
    assert_eq!(sources.len(), 3);
    assert!(sources[0] != sources[1] && sources[1] != sources[2] && sources[0] != sources[2]);
    assert_eq!(
        w.session
            .handle
            .view()
            .borrow()
            .animation
            .as_ref()
            .unwrap()
            .source,
        SOURCE
    );
    // Cada tarjeta se dibujó con su anim.js (fps distintos en «caer»).
    let fps: Vec<u32> = h
        .state()
        .shell
        .anim
        .variants
        .iter()
        .map(|c| c.result.as_ref().unwrap().actions[0].spec.fps)
        .collect();
    assert!(fps[0] != fps[1] && fps[1] != fps[2], "{fps:?}");

    // Mezclar: una variante más.
    h.get_by_label("Qué mezclar").focus();
    h.step();
    h.get_by_label("Qué mezclar")
        .type_text("el ritmo de A con el de B");
    h.step();
    h.get_by_label("Crear mezcla").click();
    wait_until(&mut h, "la mezcla", |h| all_ready(h, 4));
    assert!(h.state().shell.anim.variants[3].mix);
    h.get_by_label("Mezcla D");

    // Elegir la B: su anim.js es ahora la animación y queda una versión.
    let versions = w.session.handle.view().borrow().versions.len();
    h.get_by_label("Elegir B").click();
    wait_until(&mut h, "la variante elegida", |_| {
        w.session
            .handle
            .view()
            .borrow()
            .animation
            .as_ref()
            .map(|a| a.source.clone())
            == Some(sources[1].clone())
    });
    let vm = w.session.handle.view().borrow().clone();
    assert_eq!(vm.versions.len(), versions + 1);
    assert_eq!(vm.versions.last().unwrap().message, "Variante B elegida");
    assert!(!h.state().shell.anim.showing_variants, "vuelve al editor");
    assert!(h.state().shell.anim.variants.is_empty());
    // Y se puede deshacer.
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
    wait_until(&mut h, "deshacer la elección", |_| {
        w.session
            .handle
            .view()
            .borrow()
            .animation
            .as_ref()
            .map(|a| a.source.clone())
            == Some(SOURCE.to_string())
    });
}

/// Lo que se escribió en la carpeta: nombre y contenido.
fn written(w: &World) -> Vec<(String, Vec<u8>)> {
    w.store
        .files()
        .into_iter()
        .map(|(p, b)| (p.file_name().unwrap().to_string_lossy().into_owned(), b))
        .collect()
}

fn export(h: &mut Harness<'static, StudioApp>, format: &str, scale: &str) {
    h.get_by_label_contains("Exportar").click();
    h.run();
    h.get_by_label("Exportar animación");
    h.get_by_label(format).click();
    h.run();
    h.get_by_label(&format!("Escala {scale}")).click();
    h.run();
    let button = egui_kittest::kittest::NodeT::accesskit_node(&h.get_by_label("Exportar"))
        .bounding_box()
        .unwrap();
    assert!(
        button.y1 < 900.0,
        "el diálogo cabe en la ventana: {button:?}"
    );
    h.get_by_label("Exportar").click();
    h.run();
    assert!(
        h.query_by_label("Exportar animación").is_none(),
        "el diálogo se cierra"
    );
}

#[test]
fn the_export_dialog_writes_each_format_at_the_chosen_scale() {
    let w = world();
    *w.dialogs.folder.lock().unwrap() = Some(PathBuf::from("/juego/arte"));
    let mut h = app(&w);

    // Hoja PNG a 2×: 4 columnas (caer) × 2 filas, celdas de 24.
    export(&mut h, "Hoja de sprites PNG", "2×");
    let files = written(&w);
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["reloj_24x24.png", "reloj_24x24.json"]);
    let (sw, sh, _) = MemoryStore::decode_image(&files[0].1).unwrap();
    assert_eq!((sw, sh), (96, 48));
    let json = String::from_utf8(files[1].1.clone()).unwrap();
    assert!(
        json.contains("\"durations\": [100, 100, 100, 100]"),
        "{json}"
    );
    assert!(json.contains("\"durations\": [125, 125, 125]"), "{json}");
    assert_eq!(
        w.dialogs.opened.lock().unwrap().as_slice(),
        [PathBuf::from("/juego/arte")]
    );
    assert!(
        w.store
            .files()
            .iter()
            .all(|(p, _)| p.starts_with("/juego/arte"))
    );

    // Una tira por acción, a 4×.
    let before = w.store.files().len();
    export(&mut h, "Una tira por acción", "4×");
    let files = &written(&w)[before..];
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [
            "reloj_caer_48x48.png",
            "reloj_salto_48x48.png",
            "reloj_48x48.json"
        ]
    );
    assert_eq!(MemoryStore::decode_image(&files[0].1).unwrap().0, 4 * 48);
    assert_eq!(MemoryStore::decode_image(&files[1].1).unwrap().0, 3 * 48);

    // GIF por acción, a 1×, con sus duraciones y su bucle.
    let before = w.store.files().len();
    export(&mut h, "GIF animado", "1×");
    let files = &written(&w)[before..];
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        ["reloj_caer.gif", "reloj_salto.gif", "reloj_12x12.json"]
    );
    assert_eq!(
        MemoryStore::decode_gif(&files[0].1).unwrap(),
        (4, (12, 12), vec![100, 100, 100, 100], true)
    );
    assert_eq!(
        MemoryStore::decode_gif(&files[1].1).unwrap(),
        (3, (12, 12), vec![125, 125, 125], false)
    );

    // Godot: la hoja y su SpriteFrames, a 8×.
    let before = w.store.files().len();
    export(&mut h, "Godot · SpriteFrames", "8×");
    let files = &written(&w)[before..];
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        ["reloj_96x96.png", "reloj_96x96.tres", "reloj_96x96.json"]
    );
    let tres = String::from_utf8(files[1].1.clone()).unwrap();
    assert!(tres.contains("path=\"reloj_96x96.png\""), "{tres}");
    assert!(tres.contains("region = Rect2(288, 0, 96, 96)"), "{tres}");
    assert!(tres.contains("\"name\": &\"salto\""), "{tres}");
}
