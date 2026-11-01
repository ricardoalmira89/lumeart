//! Ajustes con el `config.toml` de verdad: persisten entre ejecuciones y las
//! claves de API nunca acaban en `config.toml`, en los `.lume` ni en los logs.
//!
//! El llavero del sistema no se toca en los tests: se usa el doble en memoria
//! (el adaptador `KeyringSecrets` tiene sus propios tests con el llavero de
//! pruebas de `keyring`).

use std::io::Read;
use std::sync::Arc;

use lume_app::ports::Secret;
use lume_app::testing::MemorySecretStore;
use lume_app::{ContextPolicy, ProviderKind, SecretStore, Session, Settings};
use lume_domain::{Document, Origin};
use lume_store::{FsSettings, FsStore};
use lume_ui::{AppController, FakeDialogs, Intent};

const KEY: &str = "sk-ant-api03-NUNCA-EN-DISCO-7f3a9c";

struct Run {
    _runtime: tokio::runtime::Runtime,
    session: Session,
    controller: AppController,
}

/// Una «ejecución» de LumeArt: sesión y controlador con el `config.toml` de
/// `dir` y el llavero `secrets`.
fn run(dir: &std::path::Path, secrets: Arc<MemorySecretStore>) -> Run {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let session = {
        let _guard = runtime.enter();
        Session::spawn(Document::new("cofre", 8, 8).unwrap(), Arc::new(FsStore))
    };
    let (controller, problem) = AppController::new(
        session.handle.clone(),
        runtime.handle().clone(),
        Arc::new(FsStore),
        None,
        Arc::new(FakeDialogs::default()),
    )
    .with_settings(
        Arc::new(FsSettings::new(dir.join("lumeart").join("config.toml"))),
        secrets,
    );
    assert_eq!(problem, None);
    Run {
        _runtime: runtime,
        session,
        controller,
    }
}

#[test]
fn settings_persist_between_runs() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = Arc::new(MemorySecretStore::default());
    let mut first = run(dir.path(), secrets.clone());
    assert_eq!(first.controller.settings, Settings::default());
    let changed = Settings {
        claude_code_model: "opus".into(),
        max_turns: 14,
        context: ContextPolicy::Resume,
        critic: true,
        show_commands: false,
        ..Settings::default()
    };
    let outcome = first
        .controller
        .handle(Intent::UpdateSettings(changed.clone()));
    assert_eq!(outcome.error, None);
    drop(first);

    let second = run(dir.path(), secrets);
    assert_eq!(second.controller.settings, changed);
}

#[test]
fn api_keys_never_reach_config_documents_or_logs() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = Arc::new(MemorySecretStore::default());
    let mut r = run(dir.path(), secrets.clone());

    let intent = Intent::SaveApiKey {
        provider: ProviderKind::Anthropic,
        key: Secret::new(KEY),
    };
    // Ni el intent ni el estado de las claves enseñan la clave en `Debug`
    // (lo que acabaría en un log).
    assert!(!format!("{intent:?}").contains("NUNCA"));
    let outcome = r.controller.handle(intent);
    assert_eq!(outcome.error, None);
    assert_eq!(
        secrets.get("anthropic-api-key").unwrap().unwrap().expose(),
        KEY
    );
    assert!(!format!("{:?}", r.controller.keys).contains("NUNCA"));
    assert!(
        r.controller
            .keys
            .anthropic
            .as_deref()
            .unwrap()
            .starts_with("sk-ant-•")
    );
    assert!(!format!("{outcome:?}").contains("NUNCA"));

    // Usar Anthropic por defecto y guardar un dibujo.
    let settings = Settings {
        provider: ProviderKind::Anthropic,
        ..r.controller.settings.clone()
    };
    r.controller.handle(Intent::UpdateSettings(settings));
    r.controller
        .runtime
        .block_on(
            r.session
                .handle
                .line("set_pixel 1 1 red", Origin::User, None),
        )
        .unwrap();
    let lume = dir.path().join("cofre.lume");
    let outcome = r.controller.handle(Intent::SaveTo(lume.clone()));
    assert_eq!(outcome.error, None);

    // `config.toml`: el proveedor sí, la clave no.
    let config = std::fs::read_to_string(dir.path().join("lumeart").join("config.toml")).unwrap();
    assert!(config.contains("provider = \"anthropic\""), "{config}");
    assert!(!config.contains("NUNCA"), "{config}");
    assert!(!config.contains("sk-ant"), "{config}");

    // El `.lume`: ningún archivo de dentro lleva la clave.
    let mut zip = zip::ZipArchive::new(std::fs::File::open(&lume).unwrap()).unwrap();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("NUNCA"), "{}", entry.name());
    }

    // Y nada más en la carpeta (ni temporales) la contiene.
    for entry in walk(dir.path()) {
        let bytes = std::fs::read(&entry).unwrap();
        assert!(
            !String::from_utf8_lossy(&bytes).contains("NUNCA"),
            "{}",
            entry.display()
        );
    }
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}
