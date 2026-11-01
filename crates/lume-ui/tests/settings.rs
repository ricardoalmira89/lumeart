//! Pantalla de Ajustes con el `StudioApp` real (arnés de eframe de
//! egui_kittest, sin ventana): los cambios se guardan, las claves van al
//! llavero y «Mostrar cada comando en el lienzo» enciende y apaga el resalte.

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;
use egui::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use lume_app::ports::CancellationToken;
use lume_app::testing::{MemorySecretStore, MemorySettingsStore, MemoryStore};
use lume_app::{ContextPolicy, ProviderKind, SecretStore, Session, Settings, SettingsStore};
use lume_domain::{Document, Origin};
use lume_ui::{FakeDialogs, Screen, StudioApp, StudioOptions};

struct World {
    runtime: tokio::runtime::Runtime,
    session: Session,
    settings: Arc<MemorySettingsStore>,
    secrets: Arc<MemorySecretStore>,
}

fn world(settings: Settings) -> World {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let store = Arc::new(MemoryStore::new());
    let session = {
        let _guard = runtime.enter();
        Session::spawn(Document::new("cofre", 32, 32).unwrap(), store)
    };
    World {
        runtime,
        session,
        settings: Arc::new(MemorySettingsStore::new(settings)),
        secrets: Arc::new(MemorySecretStore::default()),
    }
}

fn app(w: &World, screen: Screen, claude: Option<PathBuf>) -> Harness<'static, StudioApp> {
    let options = StudioOptions {
        session: w.session.handle.clone(),
        stop: CancellationToken::new(),
        runtime: w.runtime.handle().clone(),
        store: Arc::new(MemoryStore::new()),
        recents: None,
        dialogs: Arc::new(FakeDialogs::default()),
        settings: w.settings.clone(),
        secrets: w.secrets.clone(),
        claude_path: claude,
        agent: None,
        server: Some("127.0.0.1:39223".into()),
        screen,
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

/// Ctrl+, abre Ajustes en su propia ventana (en el arnés, dentro de la
/// principal); lo de detrás no cambia.
fn open_settings(harness: &mut Harness<'static, StudioApp>) {
    let before = harness.state().screen;
    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Comma);
    harness.run();
    assert!(harness.state().settings_open());
    assert_eq!(harness.state().screen, before);
}

/// Pulsa un control de la ventana de Ajustes. Detrás sigue el studio (que
/// puede tener otro control con el mismo nombre, como «Revisión crítica» en
/// el panel del agente), así que se busca dentro de la ventana y se desplaza
/// hasta él antes de pulsarlo.
fn click_in_settings(harness: &mut Harness<'static, StudioApp>, role: Option<Role>, label: &str) {
    let inside = |harness: &Harness<'static, StudioApp>| {
        let window = harness
            .get_by_label("Ajustes · LumeArt")
            .accesskit_node()
            .bounding_box()
            .expect("la ventana de Ajustes");
        let nodes: Vec<_> = match role {
            Some(role) => harness.get_all_by_role_and_label(role, label).collect(),
            None => harness.get_all_by_label(label).collect(),
        };
        nodes
            .into_iter()
            .position(|n| {
                n.accesskit_node()
                    .bounding_box()
                    .is_some_and(|b| b.x0 >= window.x0 && b.x1 <= window.x1)
            })
            .unwrap_or_else(|| panic!("«{label}» no está en la ventana de Ajustes"))
    };
    // Desplazarse hasta él (`false`) o pulsarlo (`true`).
    let act = |harness: &Harness<'static, StudioApp>, i: usize, click: bool| {
        let nodes: Vec<_> = match role {
            Some(role) => harness.get_all_by_role_and_label(role, label).collect(),
            None => harness.get_all_by_label(label).collect(),
        };
        if click {
            nodes[i].click();
        } else {
            nodes[i].scroll_to_me();
        }
    };
    let i = inside(harness);
    act(harness, i, false);
    harness.run();
    let i = inside(harness);
    act(harness, i, true);
    harness.run();
}

fn saved(w: &World) -> Settings {
    w.settings.load().unwrap()
}

/// Se abre desde el studio (Ctrl+,) y desde Inicio («Ajustes») en su propia
/// ventana, sin tapar lo de detrás, y Esc la cierra.
#[test]
fn settings_open_from_both_screens_and_go_back() {
    let w = world(Settings::default());
    let mut harness = app(&w, Screen::Studio, None);
    open_settings(&mut harness);
    harness.get_by_role_and_label(Role::Tab, "Proveedores de IA");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().settings_open());
    assert_eq!(harness.state().screen, Screen::Studio);

    // Inicio va en una pestaña del studio.
    let mut harness = app(&w, Screen::Home, None);
    assert_eq!(harness.state().screen, Screen::Studio);
    harness
        .get_all_by_label("Ajustes")
        .next()
        .expect("el botón Ajustes de Inicio")
        .click();
    harness.run();
    assert!(harness.state().settings_open());
    harness.get_by_role_and_label(Role::Tab, "Proveedores de IA");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().settings_open());
}

/// Cada cambio se guarda en el `SettingsStore` y otra ejecución (otra app con
/// el mismo almacén) lo encuentra.
#[test]
fn changes_are_saved_and_loaded_again() {
    let w = world(Settings::default());
    let mut harness = app(&w, Screen::Studio, None);
    open_settings(&mut harness);

    click_in_settings(&mut harness, None, "Más vueltas");
    click_in_settings(&mut harness, None, "Más vueltas");
    click_in_settings(&mut harness, None, "Misma conversación");
    click_in_settings(&mut harness, Some(Role::Switch), "Revisión crítica");
    click_in_settings(
        &mut harness,
        Some(Role::Switch),
        "Parar si dos vueltas no avanzan",
    );

    let s = saved(&w);
    assert_eq!(s.max_turns, 10);
    assert_eq!(s.context, ContextPolicy::Resume);
    assert!(s.critic);
    assert!(!s.stop_without_progress);

    // «Otra ejecución»: los valores salen del almacén.
    drop(harness);
    let mut harness = app(&w, Screen::Studio, None);
    assert_eq!(harness.state().controller.settings, s);
    open_settings(&mut harness);
    harness.get_by_label("10");
    click_in_settings(&mut harness, None, "Menos vueltas");
    assert_eq!(saved(&w).max_turns, 9);
}

/// El modelo de Claude Code se elige en el desplegable.
#[test]
fn the_claude_code_model_is_chosen_from_the_list() {
    let w = world(Settings::default());
    let mut harness = app(&w, Screen::Studio, None);
    open_settings(&mut harness);
    harness.get_by_label("Modelo de Claude Code").click();
    harness.run();
    harness.get_by_label("Haiku").click();
    harness.run();
    assert_eq!(saved(&w).claude_code_model, "haiku");
}

/// Si `claude` está instalado se muestra su ruta; si no, se dice.
#[test]
fn the_claude_binary_is_shown_when_found() {
    let w = world(Settings::default());
    let mut harness = app(&w, Screen::Studio, Some("/opt/claude/bin/claude".into()));
    open_settings(&mut harness);
    harness.get_by_label("Detectado en");
    harness.get_by_label("/opt/claude/bin/claude");

    let mut harness = app(&w, Screen::Studio, None);
    open_settings(&mut harness);
    harness.get_by_label_contains("No encuentro `claude`");
    assert!(harness.query_by_label("Detectado en").is_none());
}

/// La clave se escribe, va al llavero y solo se ve enmascarada; con clave se
/// puede usar por defecto; al quitarla, vuelve Claude Code.
#[test]
fn api_keys_go_to_the_keyring_and_are_only_shown_masked() {
    const KEY: &str = "sk-ant-api03-SECRETO-DE-PRUEBA-0123456789";
    let w = world(Settings::default());
    let mut harness = app(&w, Screen::Studio, None);
    open_settings(&mut harness);
    assert_eq!(harness.get_all_by_label("Sin configurar").count(), 2);
    assert!(
        harness
            .query_by_label("Usar Anthropic API por defecto")
            .is_none()
    );

    let field = harness.get_by_label("Clave de API de Anthropic API");
    field.focus();
    harness.run();
    harness
        .get_by_label("Clave de API de Anthropic API")
        .type_text(KEY);
    harness.run();
    harness
        .get_by_label("Guardar la clave de Anthropic API")
        .click();
    harness.run();

    let stored = w.secrets.get("anthropic-api-key").unwrap().unwrap();
    assert_eq!(stored.expose(), KEY);
    let masked = format!("sk-ant-{}", "•".repeat(16));
    let shown = harness
        .get_by_label("Clave de API de Anthropic API")
        .value();
    assert_eq!(shown.as_deref(), Some(masked.as_str()));
    harness.get_by_label("Clave guardada");
    assert_eq!(
        harness.get_all_by_label("Sin configurar").count(),
        1,
        "OpenAI"
    );
    harness.get_by_label("Clave guardada en el llavero del sistema.");
    // En ningún sitio de la pantalla aparece la clave.
    assert!(harness.query_by_label_contains("SECRETO").is_none());
    // Nada de la clave en los ajustes guardados.
    assert!(!format!("{:?}", saved(&w)).contains("SECRETO"));

    harness
        .get_by_label("Usar Anthropic API por defecto")
        .click();
    harness.run();
    assert_eq!(saved(&w).provider, ProviderKind::Anthropic);
    harness.get_by_label("Modelo de Anthropic API").click();
    harness.run();
    harness.get_by_label("Sonnet 5.5").click();
    harness.run();
    assert_eq!(saved(&w).anthropic_model, "claude-sonnet-5-5");

    harness
        .get_by_label("Cambiar la clave de Anthropic API")
        .click();
    harness.run();
    harness
        .get_by_label("Quitar la clave de Anthropic API")
        .click();
    harness.run();
    assert_eq!(w.secrets.get("anthropic-api-key").unwrap(), None);
    assert_eq!(saved(&w).provider, ProviderKind::ClaudeCode);
}

/// «Mostrar cada comando en el lienzo» apaga el resalte de T:26 en el studio
/// (y se guarda); el menú Ver hace lo mismo. De fábrica está apagado (se ve
/// dibujar al agente con el pincel; el resalte verde tapaba el dibujo).
#[test]
fn show_commands_turns_the_live_highlight_on_and_off() {
    let w = world(Settings::default());
    let mut harness = app(&w, Screen::Studio, None);
    w.runtime
        .block_on(
            w.session
                .handle
                .line("set_pixel 1 1 #ff0000", Origin::Cli, None),
        )
        .unwrap();
    harness.run();
    assert!(!harness.state().shell.show_commands, "apagado de fábrica");
    assert!(harness.query_by_label("cli · set_pixel 1 1").is_none());

    let w = world(Settings {
        show_commands: true,
        ..Settings::default()
    });
    let mut harness = app(&w, Screen::Studio, None);
    w.runtime
        .block_on(
            w.session
                .handle
                .line("set_pixel 3 4 #ff0000", Origin::Cli, None),
        )
        .unwrap();
    harness.run();
    harness.get_by_label("cli · set_pixel 3 4");

    open_settings(&mut harness);
    click_in_settings(
        &mut harness,
        Some(Role::Switch),
        "Mostrar cada comando en el lienzo",
    );
    assert!(!saved(&w).show_commands);
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().settings_open());
    assert!(!harness.state().shell.show_commands);
    assert!(harness.query_by_label("cli · set_pixel 3 4").is_none());

    // Al abrir otra vez con el ajuste guardado, sigue apagado.
    let mut harness = app(&w, Screen::Studio, None);
    assert!(!harness.state().shell.show_commands);
    assert!(harness.query_by_label("cli · set_pixel 3 4").is_none());

    // Menú Ver › Mostrar cada comando en el lienzo: se enciende y se guarda.
    harness.get_by_label("Ver").click();
    harness.run();
    harness
        .get_by_label("Mostrar cada comando en el lienzo")
        .click();
    harness.run();
    assert!(saved(&w).show_commands);
    harness.key_press(egui::Key::Escape);
    harness.run();
    harness.get_by_label("cli · set_pixel 3 4");
}
