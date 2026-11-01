//! Cerrar la ventana con cambios sin guardar pregunta antes (con el
//! `StudioApp` real en el arnés de eframe de egui_kittest, sin ventana).

use std::sync::Arc;

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use lume_app::Session;
use lume_app::ports::CancellationToken;
use lume_app::testing::MemoryStore;
use lume_domain::{Document, Origin};
use lume_ui::{FakeDialogs, Screen, StudioApp, StudioOptions};

struct World {
    runtime: tokio::runtime::Runtime,
    session: Session,
    stop: CancellationToken,
    dialogs: Arc<FakeDialogs>,
    store: Arc<MemoryStore>,
}

fn world() -> World {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let store = Arc::new(MemoryStore::new());
    let session = {
        let _guard = runtime.enter();
        Session::spawn(Document::new("cofre", 8, 8).unwrap(), store.clone())
    };
    World {
        runtime,
        session,
        stop: CancellationToken::new(),
        dialogs: Arc::new(FakeDialogs::default()),
        store,
    }
}

fn app(w: &World) -> Harness<'static, StudioApp> {
    let options = StudioOptions {
        session: w.session.handle.clone(),
        stop: w.stop.clone(),
        runtime: w.runtime.handle().clone(),
        store: w.store.clone(),
        recents: None,
        dialogs: w.dialogs.clone(),
        settings: Arc::new(lume_app::testing::MemorySettingsStore::default()),
        secrets: Arc::new(lume_app::testing::MemorySecretStore::default()),
        claude_path: None,
        agent: None,
        server: None,
        screen: Screen::Studio,
        art: None,
        anim_engine: None,
        variant_agents: None,
        vector: None,
        autosave: None,
        window: None,
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_eframe(move |cc| StudioApp::new(cc, options));
    harness.run();
    harness
}

fn request_close(harness: &mut Harness<'static, StudioApp>) {
    harness
        .input_mut()
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .events
        .push(egui::ViewportEvent::Close);
    harness.run();
}

#[test]
fn closing_with_unsaved_changes_asks_first() {
    let w = world();
    let mut harness = app(&w);
    w.runtime
        .block_on(
            w.session
                .handle
                .line("set_pixel 1 1 red", Origin::User, None),
        )
        .unwrap();
    harness.run();
    assert!(harness.state().controller.is_dirty());
    harness.get_by_label("•");

    request_close(&mut harness);
    assert!(harness.state().is_asking_to_close());
    assert!(
        !harness.state().close_allowed(),
        "no se cierra sin preguntar"
    );
    harness.get_by_label("¿Guardar los cambios?");

    // Cancelar: sigue abierta y sin aviso.
    harness.get_by_label("Cancelar").click();
    harness.run();
    assert!(!harness.state().is_asking_to_close());
    assert!(!harness.state().close_allowed());

    // Guardar: pregunta dónde, guarda y entonces se cierra.
    *w.dialogs.save.lock().unwrap() = Some("/juego/cofre.lume".into());
    request_close(&mut harness);
    harness.get_by_label("Guardar").click();
    harness.run();
    assert!(harness.state().close_allowed());
    assert!(!harness.state().controller.is_dirty());
    assert!(
        lume_app::DocumentStore::open_document(
            &*w.store,
            std::path::Path::new("/juego/cofre.lume")
        )
        .is_ok()
    );
}

#[test]
fn discarding_closes_and_a_clean_document_closes_without_asking() {
    let w = world();
    let mut harness = app(&w);
    // Sin cambios: se cierra sin preguntar.
    request_close(&mut harness);
    assert!(!harness.state().is_asking_to_close());
    assert!(harness.state().close_allowed());

    let w = world();
    let mut harness = app(&w);
    w.runtime
        .block_on(
            w.session
                .handle
                .line("set_pixel 1 1 red", Origin::User, None),
        )
        .unwrap();
    harness.run();
    request_close(&mut harness);
    harness.get_by_label("No guardar").click();
    harness.run();
    assert!(harness.state().close_allowed());
}

/// `lumeart stop` con cambios sin guardar también pregunta.
#[test]
fn stop_with_unsaved_changes_asks() {
    let w = world();
    let mut harness = app(&w);
    w.runtime
        .block_on(
            w.session
                .handle
                .line("set_pixel 1 1 red", Origin::User, None),
        )
        .unwrap();
    harness.run();
    w.stop.cancel();
    harness.run();
    assert!(harness.state().is_asking_to_close());
    harness.get_by_label("¿Guardar los cambios?");

    // Cancelar: la ventana sigue abierta con el dibujo y no vuelve a preguntar.
    harness.get_by_label("Cancelar").click();
    harness.run();
    harness.run();
    assert!(!harness.state().is_asking_to_close());
    assert!(!harness.state().close_allowed());
    assert!(harness.query_by_label("¿Guardar los cambios?").is_none());
}

/// Ctrl+E abre «Exportar PNG»; la escala elegida es la que se exporta.
#[test]
fn the_export_dialog_uses_the_chosen_scale() {
    let w = world();
    let mut harness = app(&w);
    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::E);
    harness.run();
    harness.get_by_label("Exportar PNG");
    harness.get_by_label("4×").click();
    harness.run();
    harness.get_by_label("8 × 8 → 32 × 32 px");
    *w.dialogs.png.lock().unwrap() = Some("/juego/cofre.png".into());
    harness.get_by_label("Exportar…").click();
    harness.run();
    let exports = w.store.exports();
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].scale, 4);
    assert_eq!(
        exports[0].path,
        std::path::PathBuf::from("/juego/cofre.png")
    );
    // El diálogo se cierra y queda un aviso.
    assert!(harness.query_by_label("Exportar PNG").is_none());
    harness.get_by_label("Exportado a /juego/cofre.png (4×)");
}
