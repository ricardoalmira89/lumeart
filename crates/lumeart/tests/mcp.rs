//! `lumeart mcp` con el binario real y un cliente MCP: con un studio abierto
//! (aquí, una sesión con su API y sus encargos en este proceso), el encargo
//! se hace en él; sin studio, el servidor trabaja sin ventana.

use std::path::Path;
use std::sync::Arc;

use lume_agent::SessionTools;
use lume_api::{ApiServer, write_state_file};
use lume_app::art::{ArtAgent, ArtAgents, LocalArt};
use lume_app::ports::{BoxFuture, PortError, ToolBox};
use lume_app::testing::{FakeBackend, MemorySettingsStore, ScriptedRunner, ScriptedStep};
use lume_app::{ArtId, ArtService, ArtState, DocumentStore, Session, SessionHandle, Settings};
use lume_domain::{Document, Rgba};
use lume_store::FsStore;
use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock};
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};

/// El agente de prueba del «studio»: pinta una llave amarilla y termina.
struct StudioAgents {
    store: Arc<dyn DocumentStore>,
}

impl ArtAgents for StudioAgents {
    fn agent(
        &self,
        session: SessionHandle,
        id: ArtId,
    ) -> BoxFuture<'_, Result<ArtAgent, PortError>> {
        Box::pin(async move {
            let runner = Arc::new(ScriptedRunner::new(vec![
                ScriptedStep::Tool {
                    name: "draw".into(),
                    input_json: r#"{"commands":"line 1 1 6 6 #ffd700"}"#.into(),
                },
                ScriptedStep::Say("Llave lista.\n<promise>LISTO</promise>".into()),
            ]));
            let tools: ToolBox = Arc::new(SessionTools::new(session, self.store.clone(), id));
            Ok(ArtAgent {
                backend: Arc::new(FakeBackend::new(runner, tools)),
                guard: Box::new(()),
            })
        })
    }
}

/// Lanza `lumeart --session <state> mcp` en `cwd` y se conecta como cliente.
async fn connect(state: &Path, cwd: &Path, config: &Path) -> RunningService<RoleClient, ()> {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_lumeart"));
    command
        .arg("--session")
        .arg(state)
        .arg("mcp")
        .current_dir(cwd)
        .env("XDG_CONFIG_HOME", config)
        .env_remove("LUMEART_SESSION");
    let transport = TokioChildProcess::new(command).unwrap();
    ().serve(transport).await.unwrap()
}

async fn call(
    client: &RunningService<RoleClient, ()>,
    tool: &str,
    json: serde_json::Value,
) -> CallToolResult {
    client
        .call_tool(
            CallToolRequestParams::new(tool.to_string())
                .with_arguments(json.as_object().unwrap().clone()),
        )
        .await
        .unwrap()
}

fn texts(result: &CallToolResult) -> Vec<String> {
    result
        .content
        .iter()
        .filter_map(|c| match c {
            ContentBlock::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .collect()
}

fn json_of(result: &CallToolResult) -> serde_json::Value {
    texts(result)
        .iter()
        .find_map(|t| serde_json::from_str(t).ok())
        .expect("un bloque con JSON")
}

#[tokio::test(flavor = "multi_thread")]
async fn with_the_studio_open_the_job_runs_there() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("juego");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(
        project.join("lumeart.toml"),
        "size = \"8x8\"\noutput = \"arte\"\nnaming = \"kebab-case\"\n",
    )
    .unwrap();

    // El «studio»: su lienzo, sus encargos y su API, con su archivo de estado.
    let store: Arc<dyn DocumentStore> = Arc::new(FsStore);
    let main = Session::spawn(Document::new("studio", 8, 8).unwrap(), store.clone());
    let art = Arc::new(LocalArt::new(
        main.handle.clone(),
        store.clone(),
        Arc::new(MemorySettingsStore::new(Settings {
            max_turns: 2,
            critic: false,
            ..Settings::default()
        })),
        Arc::new(StudioAgents {
            store: store.clone(),
        }),
    ));
    let server = ApiServer::start_with_art(
        main.handle.clone(),
        store.clone(),
        Some(art.clone() as Arc<dyn ArtService>),
    )
    .await
    .unwrap();
    let state = dir.path().join("session.json");
    write_state_file(&state, &server.info()).unwrap();

    let client = connect(&state, &project, &dir.path().join("config")).await;
    let created = call(
        &client,
        "create_art",
        serde_json::json!({ "description": "Llave dorada", "name": "Llave Dorada" }),
    )
    .await;
    assert_ne!(created.is_error, Some(true), "{:?}", texts(&created));
    let id = json_of(&created)["id"].as_u64().unwrap();

    let done = call(
        &client,
        "wait_art",
        serde_json::json!({ "id": id, "timeout_seconds": 30 }),
    )
    .await;
    let status = json_of(&done);
    assert_eq!(status["state"], "done", "{:?}", texts(&done));
    assert!(
        done.content
            .iter()
            .any(|c| matches!(c, ContentBlock::Image(_))),
        "sin vista previa"
    );

    // El encargo se hizo en el studio: está en su lista y tiene su sesión.
    let jobs = art.list().await.unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, id);
    assert_eq!(jobs[0].name, "llave-dorada");
    assert_eq!(jobs[0].state, ArtState::Done);
    let session = art
        .session(id)
        .expect("la sesión del encargo, en el studio");
    assert_eq!(
        session.view().borrow().canvas.get(3, 3).unwrap(),
        Rgba::rgb(0xff, 0xd7, 0x00)
    );

    // Y los archivos, en la carpeta de la guía del juego.
    let png = project.join("arte/llave-dorada.png");
    assert!(png.exists());
    assert!(project.join("arte/llave-dorada.lume").exists());
    assert_eq!(FsStore.read_png(&png).unwrap().width(), 8);

    // Las herramientas de bajo nivel dibujan en el lienzo del studio.
    let drawn = call(
        &client,
        "draw",
        serde_json::json!({ "commands": "set_pixel 0 0 red" }),
    )
    .await;
    assert!(
        texts(&drawn)[0].contains("1 ok, 0 failed"),
        "{:?}",
        texts(&drawn)
    );
    assert_eq!(
        main.handle.view().borrow().canvas.get(0, 0).unwrap(),
        Rgba::rgb(255, 0, 0)
    );

    client.cancel().await.unwrap();
    server.shutdown().await.unwrap();
    main.close().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn without_a_studio_it_works_on_its_own() {
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("no-hay-sesion.json");
    let client = connect(&state, dir.path(), &dir.path().join("config")).await;
    let tools = client.list_all_tools().await.unwrap();
    // Sin ventana también están las del documento (sobre su propia sesión).
    assert_eq!(
        tools.len(),
        lume_mcp::TOOLS.len() + lume_mcp::STUDIO_TOOLS.len()
    );

    let guide = call(&client, "art_guide", serde_json::json!({})).await;
    assert!(
        texts(&guide)[0].contains("aún no tiene lumeart.toml"),
        "{:?}",
        texts(&guide)
    );
    let drawn = call(
        &client,
        "draw",
        serde_json::json!({ "commands": "new 4 4\nset_pixel 1 1 blue" }),
    )
    .await;
    assert!(
        texts(&drawn)[0].contains("2 ok, 0 failed"),
        "{:?}",
        texts(&drawn)
    );
    let exported = call(&client, "export", serde_json::json!({ "path": "azul.png" })).await;
    assert_ne!(exported.is_error, Some(true), "{:?}", texts(&exported));
    let canvas = FsStore.read_png(&dir.path().join("azul.png")).unwrap();
    assert_eq!(canvas.get(1, 1).unwrap(), Rgba::rgb(0, 0, 255));
    client.cancel().await.unwrap();
}
