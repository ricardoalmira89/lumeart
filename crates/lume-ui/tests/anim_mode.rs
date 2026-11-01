//! Modo animación con el `StudioApp` real y el motor de `anim.js` de
//! verdad: crear la animación desde el dibujo, pistas con rombos donde cada
//! pieza cambia, duración por fotograma reversible y reproducción.

use std::sync::Arc;
use std::time::Duration;

use eframe::egui;
use egui::Key;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use lume_app::ports::CancellationToken;
use lume_app::testing::{MemorySecretStore, MemorySettingsStore, MemoryStore};
use lume_app::{AnimEngine, Session};
use lume_domain::anim::ActionSpec;
use lume_domain::anim::animation::{AnimEdit, Animation};
use lume_domain::{Document, Origin};
use lume_script::QuickJsAnim;
use lume_ui::{FakeDialogs, Mode, Screen, StudioApp, StudioOptions};

/// Un reloj de arena: marco fijo, arena que baja hasta el fotograma 2.
const SOURCE: &str = r##"config({ name: "reloj", cell: [12, 12] });
part("marco", () => { fill_rect(2, 0, 8, 1, "#7a4a2a"); fill_rect(2, 11, 8, 1, "#7a4a2a"); });
part("arena", ({ nivel = 0 }) => { fill_rect(4, 2 + nivel, 4, 3 - nivel, "#e8c070"); });
rest(() => { draw("marco"); draw("arena"); });
action("caer", { frames: 4, fps: 10 }, (f) => { draw("marco"); draw("arena", { nivel: Math.min(f, 2) }); });
action("salto", { frames: 3, fps: 8, loop: false }, (f) => { move(0, -f, () => { draw("marco"); }); });
"##;

struct World {
    runtime: tokio::runtime::Runtime,
    session: Session,
}

fn world(animation: Option<&str>) -> World {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let session = {
        let _guard = runtime.enter();
        Session::spawn(
            Document::new("reloj", 12, 12).unwrap(),
            Arc::new(MemoryStore::new()),
        )
    };
    runtime
        .block_on(session.handle.batch(
            "fill_rect 2 0 8 1 #7a4a2a\nfill_rect 2 11 8 1 #7a4a2a\nfill_rect 4 2 4 3 #e8c070",
            Origin::User,
            None,
        ))
        .unwrap();
    if let Some(source) = animation {
        runtime
            .block_on(
                session
                    .handle
                    .edit_animation(AnimEdit::Set(Some(Animation::new(source))), Origin::User),
            )
            .unwrap();
    }
    World { runtime, session }
}

fn app(w: &World, step_dt: f32) -> Harness<'static, StudioApp> {
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
        agent: None,
        server: None,
        screen: Screen::Studio,
        art: None,
        anim_engine: Some(Arc::new(QuickJsAnim) as Arc<dyn AnimEngine>),
        variant_agents: None,
        vector: None,
        autosave: None,
        window: None,
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .with_step_dt(step_dt)
        .build_eframe(move |cc| StudioApp::new(cc, options));
    harness.run_steps(2);
    harness
}

fn wait_until(
    h: &mut Harness<'static, StudioApp>,
    what: &str,
    cond: impl Fn(&Harness<'static, StudioApp>) -> bool,
) {
    for _ in 0..400 {
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

/// Al modo animación y a que esté dibujada.
fn animation_mode(h: &mut Harness<'static, StudioApp>) {
    h.get_by_label("Animación").click();
    h.run_steps(2);
    assert_eq!(h.state().shell.mode, Mode::Animation);
}

fn rendered(h: &Harness<'static, StudioApp>) -> bool {
    h.state().shell.anim.result.is_some()
}

fn durations(w: &World, frames: u32) -> Vec<u32> {
    let spec = ActionSpec {
        name: "caer".into(),
        frames,
        fps: 10,
        looping: true,
    };
    w.session
        .handle
        .view()
        .borrow()
        .animation
        .as_ref()
        .map(|a| a.durations_for(&spec))
        .unwrap_or_default()
}

#[test]
fn the_animation_is_created_from_the_drawing() {
    let w = world(None);
    let mut h = app(&w, 0.25);
    animation_mode(&mut h);
    h.get_by_label("Este dibujo aún no tiene animación");
    h.get_by_label("Crear animación").click();
    wait_until(&mut h, "la animación dibujada", rendered);
    let vm = w.session.handle.view().borrow().clone();
    let source = &vm
        .animation
        .as_ref()
        .expect("la animación del documento")
        .source;
    assert!(source.contains("part(\"base\""), "{source}");
    assert!(
        source.contains("los mismos comandos de pxcli"),
        "usa la receta: {source}"
    );
    // Una acción para empezar (en la lista y en la línea de tiempo) y el
    // reposo idéntico a la base.
    let names: Vec<String> = h
        .state()
        .shell
        .anim
        .result
        .as_ref()
        .unwrap()
        .actions
        .iter()
        .map(|a| a.spec.name.clone())
        .collect();
    assert_eq!(names, ["anim"]);
    assert_eq!(h.query_all_by_label("anim").count(), 2);
    h.get_by_label("El reposo coincide con la base");
    h.get_by_label("Solo colores de la paleta");
    h.get_by_label("Fotograma 8");
    // Y se puede deshacer como cualquier cambio.
    h.key_press_modifiers(egui::Modifiers::COMMAND, Key::Z);
    wait_until(&mut h, "deshacer la animación", |h| !rendered(h));
    assert!(w.session.handle.view().borrow().animation.is_none());
    h.get_by_label("Crear animación");
}

#[test]
fn tracks_show_diamonds_where_each_part_changes() {
    let w = world(Some(SOURCE));
    let mut h = app(&w, 0.25);
    animation_mode(&mut h);
    wait_until(&mut h, "la animación dibujada", rendered);
    // Pistas de «caer»: el marco solo al empezar; la arena hasta que se para.
    for (label, want) in [
        ("marco: cambia en el fotograma 1", true),
        ("marco: cambia en el fotograma 2", false),
        ("arena: cambia en el fotograma 1", true),
        ("arena: cambia en el fotograma 2", true),
        ("arena: cambia en el fotograma 3", true),
        ("arena: cambia en el fotograma 4", false),
    ] {
        assert_eq!(has(&h, label), want, "{label}");
    }
    h.get_by_label("0,40 s en bucle");
    // En «salto», el marco se mueve en cada fotograma.
    h.get_by_label("salto").click();
    h.run_steps(2);
    for f in 1..=3 {
        assert!(
            has(&h, &format!("marco: cambia en el fotograma {f}")),
            "salto f{f}"
        );
    }
    assert!(
        !has(&h, "arena: cambia en el fotograma 1"),
        "salto no dibuja la arena"
    );
    h.get_by_label("0,38 s una vez");
}

#[test]
fn changing_a_frame_duration_can_be_undone() {
    let w = world(Some(SOURCE));
    let mut h = app(&w, 0.25);
    animation_mode(&mut h);
    wait_until(&mut h, "la animación dibujada", rendered);
    assert_eq!(durations(&w, 4), [100, 100, 100, 100]);

    h.get_by_label("Duración del fotograma 2").click();
    h.run_steps(2);
    h.get_by_label("Duración del fotograma 2").type_text("150");
    h.key_press(Key::Enter);
    wait_until(&mut h, "la duración nueva", |_| {
        durations(&w, 4) == [100, 150, 100, 100]
    });
    let vm = w.session.handle.view().borrow().clone();
    assert_eq!(
        vm.history.last().unwrap().label,
        "duración · caer f2 · 150 ms"
    );

    // Deshacer (Ctrl+Z) la devuelve y rehacer la repite.
    h.key_press_modifiers(egui::Modifiers::COMMAND, Key::Z);
    wait_until(&mut h, "deshacer", |_| {
        durations(&w, 4) == [100, 100, 100, 100]
    });
    h.key_press_modifiers(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, Key::Z);
    wait_until(&mut h, "rehacer", |_| {
        durations(&w, 4) == [100, 150, 100, 100]
    });
    h.get_by_label("0,45 s en bucle");
}

#[test]
fn playback_follows_the_frame_durations() {
    let w = world(Some(SOURCE));
    // Cada paso de la ventana avanza 50 ms.
    let mut h = app(&w, 0.05);
    animation_mode(&mut h);
    wait_until(&mut h, "la animación dibujada", rendered);
    w.runtime
        .block_on(w.session.handle.edit_animation(
            AnimEdit::FrameDuration {
                action: ActionSpec {
                    name: "caer".into(),
                    frames: 4,
                    fps: 10,
                    looping: true,
                },
                frame: 1,
                ms: 300,
            },
            Origin::User,
        ))
        .unwrap();
    h.run_steps(2);
    let frame = |h: &Harness<'static, StudioApp>| h.state().shell.anim.player.frame;
    h.get_by_label("Reproducir").click();
    let mut seen = Vec::new();
    for _ in 0..16 {
        h.step();
        seen.push(frame(&h));
    }
    // El primer paso arranca el reloj; después, 100 ms en el 0 (dos pasos),
    // 300 en el 1 (seis), 100 en el 2 y en el 3, y en bucle vuelve al 0.
    assert_eq!(seen, [0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 3, 3, 0, 0, 1]);
    h.get_by_label("Pausa").click();
    h.run_steps(3);
    let stopped = frame(&h);
    h.run_steps(5);
    assert_eq!(frame(&h), stopped, "en pausa no avanza");
}
