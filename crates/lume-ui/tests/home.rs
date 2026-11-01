//! Pantalla de inicio con `egui_kittest` (sin ventana).

use std::path::PathBuf;

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use lume_app::RecentEntry;
use lume_domain::{Canvas, Rgba};
use lume_ui::{HomeScreen, Intent, theme};

struct State {
    home: HomeScreen,
    recents: Vec<RecentEntry>,
    intents: Vec<Intent>,
    themed: bool,
}

fn recent(name: &str, w: i64, opened_at: u64) -> RecentEntry {
    let mut canvas = Canvas::new(w, w).unwrap();
    canvas
        .fill_rect(1, 1, 4, 4, Rgba::rgb(0x8b, 0x3f, 0xe0))
        .unwrap();
    RecentEntry {
        path: PathBuf::from(format!("/arte/{name}.lume")),
        name: name.into(),
        thumbnail: canvas,
        opened_at,
    }
}

const NOW: u64 = 1_793_188_845;

fn harness(recents: Vec<RecentEntry>) -> Harness<'static, State> {
    let state = State {
        home: HomeScreen::default(),
        recents,
        intents: Vec::new(),
        themed: false,
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, state: &mut State| {
                if !state.themed {
                    theme::apply(ui.ctx());
                    state.themed = true;
                }
                let screen = ui.ctx().content_rect();
                let intents = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(screen), |ui| {
                        ui.set_clip_rect(screen);
                        state.home.show(ui, &state.recents, NOW)
                    })
                    .inner;
                state.intents.extend(intents);
            },
            state,
        );
    harness.run();
    harness
}

#[test]
fn creating_a_blank_32x32_canvas() {
    let mut harness = harness(Vec::new());
    harness.get_by_label("Nuevo pixel art…").click();
    harness.run();
    assert!(harness.state().home.asking_size);
    harness.get_by_label("16 × 16").click();
    harness.run();
    assert_eq!(harness.state().home.size(), Ok((16, 16)));
    harness.get_by_label("32 × 32").click();
    harness.run();
    assert_eq!(harness.state().home.size(), Ok((32, 32)));
    harness.get_by_label("Crear lienzo en blanco").click();
    harness.run();
    assert_eq!(
        harness.state().intents,
        vec![
            Intent::NewDocument {
                width: 32,
                height: 32,
                palette: None
            },
            Intent::SwitchMode(lume_ui::Mode::Pixel),
        ]
    );
    assert!(!harness.state().home.asking_size);
}

#[test]
fn a_fixed_palette_and_custom_size() {
    let mut harness = harness(Vec::new());
    harness.state_mut().home.asking_size = true;
    harness.state_mut().home.width = "48".into();
    harness.state_mut().home.height = "24".into();
    harness.state_mut().home.palette = 2;
    harness.run();
    harness.get_by_label("Crear lienzo en blanco").click();
    harness.run();
    assert_eq!(
        harness.state().intents,
        vec![
            Intent::NewDocument {
                width: 48,
                height: 24,
                palette: Some("game-boy".into())
            },
            Intent::SwitchMode(lume_ui::Mode::Pixel),
        ]
    );
}

/// Con una proporción elegida, el alto sigue al ancho.
#[test]
fn the_pixel_size_follows_the_proportion() {
    let mut harness = harness(Vec::new());
    harness.state_mut().home.asking_size = true;
    harness.run();
    harness.get_by_label("16:9").click();
    harness.run();
    assert_eq!(harness.state().home.pixel_ratio, Some((16, 9)));
    // 32 de ancho a 16:9: 18 de alto.
    assert_eq!(harness.state().home.size(), Ok((32, 18)));
    harness.get_by_label("Libre").click();
    harness.run();
    assert_eq!(harness.state().home.pixel_ratio, None);
}

#[test]
fn invalid_sizes_disable_the_button_with_a_message() {
    for (w, h) in [("0", "32"), ("600", "32"), ("32", "abc"), ("", "")] {
        let mut harness = harness(Vec::new());
        harness.state_mut().home.asking_size = true;
        harness.state_mut().home.width = w.into();
        harness.state_mut().home.height = h.into();
        harness.run();
        let button = harness.get_by_label("Crear lienzo en blanco");
        assert!(button.accesskit_node().is_disabled(), "{w}×{h}");
        harness.get_by_label(lume_ui::home::SIZE_ERROR);
        button.click();
        harness.run();
        assert!(harness.state().intents.is_empty(), "{w}×{h}");
    }
}

/// «Nuevo dibujo vectorial…»: tamaño, proporción y fondo antes de crearlo.
#[test]
fn a_new_vector_drawing_with_its_size_and_proportion() {
    let mut harness = harness(Vec::new());
    harness.get_by_label("Nuevo dibujo vectorial…").click();
    harness.run();
    assert!(harness.state().home.asking_vector);
    harness.get_by_label("16:9").click();
    harness.run();
    assert_eq!(harness.state().home.vector_size.size(), Ok((1024, 576)));
    harness
        .get_by_label("Fondo transparente (un objeto suelto)")
        .click();
    harness.run();
    harness.get_by_label("Crear dibujo en blanco").click();
    harness.run();
    assert_eq!(
        harness.state().intents,
        vec![
            Intent::NewDocument {
                width: 32,
                height: 32,
                palette: None
            },
            Intent::SwitchMode(lume_ui::Mode::Vector),
            Intent::NewVector {
                width: 1024,
                height: 576,
                transparent: true
            },
        ]
    );
}

#[test]
fn recents_show_and_open() {
    let mut harness = harness(vec![
        recent("pocion_morada", 32, NOW - 300),
        recent("seta_roja", 16, NOW - 7200),
    ]);
    harness.get_by_label("Recientes");
    harness.get_by_label("seta_roja");
    harness.get_by_label("pocion_morada").click();
    harness.run();
    assert_eq!(
        harness.state().intents,
        vec![Intent::OpenRecent(PathBuf::from(
            "/arte/pocion_morada.lume"
        ))]
    );
    // Sin recientes, un aviso.
    let harness = harness_empty();
    harness.get_by_label("Aún no hay dibujos recientes.");
}

fn harness_empty() -> Harness<'static, State> {
    harness(Vec::new())
}
