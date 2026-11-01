//! El servidor MCP con un cliente en proceso: lista las herramientas con sus
//! esquemas, `create_art` devuelve el número al momento, `wait_art` da el
//! estado, las rutas y la vista previa (con notificaciones de progreso) y
//! los archivos quedan en la carpeta de `lumeart.toml`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use lume_agent::SessionTools;
use lume_app::art::{ArtAgent, ArtAgents, LocalArt};
use lume_app::ports::{
    AgentRunner, AgentTask, BoxFuture, CancellationToken, EventSink, PortError, RunSummary, ToolBox,
};
use lume_app::testing::{FakeBackend, MemorySettingsStore, ScriptedRunner, ScriptedStep};
use lume_app::{ArtId, DocumentStore, Session, SessionHandle, Settings};
use lume_domain::{Document, Rgba};
use lume_mcp::LumeMcp;
use lume_store::{FsProject, FsStore};
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ClientRequest, ContentBlock, ProgressNotificationParam,
    ReadResourceRequestParams, Request, ServerResult,
};
use rmcp::service::{NotificationContext, PeerRequestOptions, RunningService};
use rmcp::{ClientHandler, RoleClient, ServiceExt};
use tokio::sync::watch;

/// Runner que espera a que el test abra la puerta y luego sigue su guion.
struct Gated {
    open: watch::Receiver<bool>,
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
            let mut open = self.open.clone();
            let _ = open.wait_for(|o| *o).await;
            self.script.run(task, tools, events, cancel).await
        })
    }
}

/// Agentes de prueba: dibujan un cofre marrón de 4×4 y lo dan por bueno.
struct TestAgents {
    store: Arc<dyn DocumentStore>,
    open: watch::Receiver<bool>,
    goals: Arc<Mutex<Vec<String>>>,
}

impl ArtAgents for TestAgents {
    fn agent(
        &self,
        session: SessionHandle,
        id: ArtId,
    ) -> BoxFuture<'_, Result<ArtAgent, PortError>> {
        Box::pin(async move {
            let runner = Arc::new(Recorder {
                goals: self.goals.clone(),
                inner: Gated {
                    open: self.open.clone(),
                    script: ScriptedRunner::new(vec![
                        ScriptedStep::Tool {
                            name: "draw".into(),
                            input_json: r#"{"commands":"fill_rect 2 2 4 4 #8b5a2b"}"#.into(),
                        },
                        ScriptedStep::Say("Cofre dibujado.\n<promise>LISTO</promise>".into()),
                    ]),
                },
            });
            let tools: ToolBox = Arc::new(SessionTools::new(session, self.store.clone(), id));
            Ok(ArtAgent {
                backend: Arc::new(FakeBackend::new(runner, tools)),
                guard: Box::new(()),
            })
        })
    }
}

/// Apunta el encargo que recibe el agente.
struct Recorder {
    goals: Arc<Mutex<Vec<String>>>,
    inner: Gated,
}

impl AgentRunner for Recorder {
    fn run<'a>(
        &'a self,
        task: AgentTask,
        tools: ToolBox,
        events: EventSink,
        cancel: CancellationToken,
    ) -> BoxFuture<'a, Result<RunSummary, PortError>> {
        self.goals.lock().unwrap().push(task.goal.clone());
        self.inner.run(task, tools, events, cancel)
    }
}

/// Cliente que guarda las notificaciones de progreso.
#[derive(Clone, Default)]
struct Client {
    progress: Arc<Mutex<Vec<ProgressNotificationParam>>>,
}

impl ClientHandler for Client {
    async fn on_progress(
        &self,
        params: ProgressNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) {
        self.progress.lock().unwrap().push(params);
    }
}

struct Fixture {
    client: RunningService<RoleClient, Client>,
    progress: Arc<Mutex<Vec<ProgressNotificationParam>>>,
    open: watch::Sender<bool>,
    goals: Arc<Mutex<Vec<String>>>,
    _main: Session,
}

async fn start(project: &Path, cwd: &Path) -> Fixture {
    let store: Arc<dyn DocumentStore> = Arc::new(FsStore);
    let main = Session::spawn(Document::new("principal", 8, 8).unwrap(), store.clone());
    let (open, gate) = watch::channel(false);
    let goals = Arc::new(Mutex::new(Vec::new()));
    let settings = Settings {
        max_turns: 3,
        critic: false,
        ..Settings::default()
    };
    let art = LocalArt::new(
        main.handle.clone(),
        store.clone(),
        Arc::new(MemorySettingsStore::new(settings)),
        Arc::new(TestAgents {
            store,
            open: gate,
            goals: goals.clone(),
        }),
    );
    let server = LumeMcp::new(Arc::new(art), Arc::new(FsProject), cwd.to_path_buf());
    let (server_io, client_io) = tokio::io::duplex(1 << 20);
    tokio::spawn(async move {
        let running = server.serve(server_io).await.unwrap();
        let _ = running.waiting().await;
    });
    let handler = Client::default();
    let progress = handler.progress.clone();
    let client = handler.serve(client_io).await.unwrap();
    assert!(project.exists());
    Fixture {
        client,
        progress,
        open,
        goals,
        _main: main,
    }
}

fn args(json: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    json.as_object().unwrap().clone()
}

async fn call(f: &Fixture, tool: &str, json: serde_json::Value) -> CallToolResult {
    f.client
        .call_tool(CallToolRequestParams::new(tool.to_string()).with_arguments(args(json)))
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

fn image(result: &CallToolResult) -> Option<Vec<u8>> {
    use base64::Engine;
    result.content.iter().find_map(|c| match c {
        ContentBlock::Image(image) => {
            assert_eq!(image.mime_type, "image/png");
            base64::engine::general_purpose::STANDARD
                .decode(&image.data)
                .ok()
        }
        _ => None,
    })
}

fn project() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("lumeart.toml"),
        "style = \"pixel art limpio, contorno oscuro\"\nsize = \"16x16\"\noutput = \"assets/art\"\n",
    )
    .unwrap();
    let cwd = dir.path().join("src/escenas");
    std::fs::create_dir_all(&cwd).unwrap();
    (dir, cwd)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_client_lists_the_tools_and_their_schemas() {
    let (dir, cwd) = project();
    let f = start(dir.path(), &cwd).await;
    let tools = f.client.list_all_tools().await.unwrap();
    let mut names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    names.sort_unstable();
    // Las de encargos y las del studio (el documento, sus pestañas y la
    // ventana).
    let mut expected: Vec<&str> = lume_mcp::TOOLS
        .iter()
        .chain(lume_mcp::STUDIO_TOOLS.iter())
        .copied()
        .collect();
    expected.sort_unstable();
    assert_eq!(names, expected);
    let schema = |name: &str| {
        let tool = tools.iter().find(|t| t.name == name).unwrap();
        assert!(
            tool.description.as_deref().is_some_and(|d| !d.is_empty()),
            "{name} sin descripción"
        );
        serde_json::Value::Object((*tool.input_schema).clone())
    };
    // Las del documento van a una pestaña (`tab`, opcional); las que crean
    // pestañas no lo necesitan.
    for name in lume_mcp::STUDIO_TOOLS {
        let input = schema(name);
        let has_tab = input["properties"].get("tab").is_some();
        let creates = matches!(name, "new_document" | "open_document" | "list_tabs");
        assert_eq!(has_tab, !creates, "{name}: tab");
        let required = input["required"].as_array().cloned().unwrap_or_default();
        assert!(!required.contains(&"tab".into()), "{name}: tab es opcional");
    }
    let create = schema("create_art");
    assert_eq!(create["type"], "object");
    let required: Vec<&str> = create["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(required, ["description", "name"]);
    for key in ["description", "name", "kind", "size", "view", "style"] {
        assert!(
            create["properties"][key].is_object(),
            "falta {key}: {create}"
        );
    }
    assert!(
        create["properties"]["description"]["description"]
            .as_str()
            .unwrap()
            .contains("para qué sirve")
    );
    let wait = schema("wait_art");
    assert!(wait["properties"]["id"].is_object());
    assert!(wait["properties"]["timeout_seconds"].is_object());
    assert!(schema("revise_art")["properties"]["changes"].is_object());
    assert!(schema("draw")["properties"]["commands"].is_object());
    assert!(schema("export")["properties"]["path"].is_object());

    // La guía, como herramienta y como recurso.
    let guide = call(&f, "art_guide", serde_json::json!({})).await;
    let text = texts(&guide).join("\n");
    assert!(
        text.contains("- Estilo: pixel art limpio, contorno oscuro"),
        "{text}"
    );
    assert!(text.contains("- Tamaño por defecto: 16x16"), "{text}");
    assert!(text.contains("size = \"16x16\""), "{text}");
    let resources = f.client.list_all_resources().await.unwrap();
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].uri, "lumeart://guia");
    let read = f
        .client
        .read_resource(ReadResourceRequestParams::new("lumeart://guia"))
        .await
        .unwrap();
    assert_eq!(read.contents.len(), 1);
    f.client.cancel().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn create_returns_at_once_and_wait_brings_the_files() {
    let (dir, cwd) = project();
    let f = start(dir.path(), &cwd).await;

    // El agente no puede dibujar hasta que abramos la puerta: create_art
    // vuelve igualmente con el número.
    let created = call(
        &f,
        "create_art",
        serde_json::json!({
            "description": "Cofre de madera que el jugador abre",
            "name": "Cofre de Madera",
            "view": "frontal",
        }),
    )
    .await;
    assert_ne!(created.is_error, Some(true), "{:?}", texts(&created));
    let json = json_of(&created);
    let id = json["id"].as_u64().unwrap();
    assert_eq!(json["state"], "running");
    assert_eq!(json["name"], "cofre_de_madera");
    assert!(texts(&created)[0].contains(&format!("Llama a wait_art con id {id}")));

    // Esperar poco: sigue en marcha.
    let waiting = call(
        &f,
        "wait_art",
        serde_json::json!({ "id": id, "timeout_seconds": 1 }),
    )
    .await;
    assert_eq!(json_of(&waiting)["state"], "running");
    assert!(texts(&waiting)[0].contains("Vuelve a llamar a wait_art"));
    assert!(
        !dir.path().join("assets/art/cofre_de_madera.png").exists(),
        "aún no hay archivos"
    );

    // Se abre la puerta y wait_art (con token de progreso) espera al final.
    f.open.send(true).unwrap();
    let handle = f
        .client
        .send_cancellable_request(
            ClientRequest::CallToolRequest(Request::new(
                CallToolRequestParams::new("wait_art")
                    .with_arguments(args(serde_json::json!({ "id": id, "timeout_seconds": 20 }))),
            )),
            PeerRequestOptions::no_options(),
        )
        .await
        .unwrap();
    let ServerResult::CallToolResult(done) = handle.await_response().await.unwrap() else {
        panic!("se esperaba el resultado de wait_art");
    };
    let status = json_of(&done);
    assert_eq!(status["state"], "done", "{:?}", texts(&done));
    assert_eq!(status["turn"], 1);
    assert_eq!(status["written"], true);
    let png = dir.path().join("assets/art/cofre_de_madera.png");
    let lume = dir.path().join("assets/art/cofre_de_madera.lume");
    assert_eq!(status["png"], png.to_string_lossy().as_ref());
    assert_eq!(status["lume"], lume.to_string_lossy().as_ref());
    let summary = texts(&done)[0].clone();
    assert!(summary.contains("terminado"), "{summary}");
    assert!(
        summary.contains("assets/art/cofre_de_madera.png"),
        "{summary}"
    );

    // Los archivos, en la carpeta de la guía (no en la del agente).
    let canvas = FsStore.read_png(&png).unwrap();
    assert_eq!((canvas.width(), canvas.height()), (16, 16));
    assert_eq!(canvas.get(3, 3).unwrap(), Rgba::rgb(0x8b, 0x5a, 0x2b));
    let document = FsStore.open_document(&lume).unwrap();
    assert_eq!(document.name, "cofre_de_madera");
    assert_eq!(document.versions().len(), 1);
    assert!(!cwd.join("assets").exists());

    // La vista previa es el dibujo ampliado.
    let preview = lume_store::decode_png(&image(&done).expect("vista previa")).unwrap();
    assert_eq!(preview.width(), 256);
    // Y llegó al menos una notificación de progreso con la vuelta.
    let progress = f.progress.lock().unwrap().clone();
    assert!(!progress.is_empty(), "sin notificaciones de progreso");
    assert_eq!(progress.last().unwrap().total, Some(3.0));

    // El agente recibió el encargo con la guía.
    let goal = f.goals.lock().unwrap()[0].clone();
    assert!(
        goal.contains("Recurso: cofre_de_madera (16x16 píxeles"),
        "{goal}"
    );
    assert!(
        goal.contains("Cofre de madera que el jugador abre"),
        "{goal}"
    );
    assert!(goal.contains("Vista: frontal"), "{goal}");
    assert!(
        goal.contains("Estilo: pixel art limpio, contorno oscuro"),
        "{goal}"
    );

    // Otro encargo con el mismo nombre no pisa el archivo.
    let again = call(
        &f,
        "create_art",
        serde_json::json!({ "description": "Otro cofre", "name": "cofre_de_madera" }),
    )
    .await;
    assert_eq!(again.is_error, Some(true));
    assert!(texts(&again)[0].contains("usa revise_art"));

    // list_art lo ve con su versión.
    let list = call(&f, "list_art", serde_json::json!({})).await;
    let listed = json_of(&list);
    assert_eq!(listed["assets"][0]["name"], "cofre_de_madera");
    assert_eq!(listed["assets"][0]["versions"], 1);
    assert_eq!(listed["jobs"][0]["state"], "done");

    // revise_art sigue el mismo dibujo y guarda una versión más.
    let revised = call(
        &f,
        "revise_art",
        serde_json::json!({ "name": "cofre_de_madera", "changes": "la tapa más clara" }),
    )
    .await;
    assert_ne!(revised.is_error, Some(true), "{:?}", texts(&revised));
    let second = json_of(&revised)["id"].as_u64().unwrap();
    assert_ne!(second, id);
    let done = call(
        &f,
        "wait_art",
        serde_json::json!({ "id": second, "timeout_seconds": 20 }),
    )
    .await;
    assert_eq!(json_of(&done)["state"], "done", "{:?}", texts(&done));
    assert_eq!(FsStore.open_document(&lume).unwrap().versions().len(), 2);
    let goal = f.goals.lock().unwrap()[1].clone();
    assert!(
        goal.contains("Cambios pedidos en esta revisión: la tapa más clara"),
        "{goal}"
    );
    f.client.cancel().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_requests_come_back_as_tool_errors() {
    let (dir, cwd) = project();
    let f = start(dir.path(), &cwd).await;
    for (tool, json, expected) in [
        (
            "create_art",
            serde_json::json!({ "description": "x", "name": "a", "kind": "animation" }),
            "todavía no",
        ),
        (
            "create_art",
            serde_json::json!({ "description": "x", "name": "a", "size": "grande" }),
            "size: \"grande\" no es un tamaño",
        ),
        (
            "create_art",
            serde_json::json!({ "description": "x", "name": "¿?" }),
            "no tiene letras ni cifras",
        ),
        (
            "wait_art",
            serde_json::json!({ "id": 99 }),
            "no hay ningún encargo con el número 99",
        ),
        (
            "revise_art",
            serde_json::json!({ "name": "nada", "changes": "más rojo" }),
            "no hay ningún recurso «nada»",
        ),
    ] {
        let result = call(&f, tool, json.clone()).await;
        assert_eq!(result.is_error, Some(true), "{tool} {json}");
        let text = texts(&result).join("\n");
        assert!(text.contains(expected), "{tool} {json}: {text}");
    }

    // Una guía mal escrita se explica con su línea.
    std::fs::write(dir.path().join("lumeart.toml"), "size = \"16\"\n").unwrap();
    let result = call(&f, "art_guide", serde_json::json!({})).await;
    assert_eq!(result.is_error, Some(true));
    let text = texts(&result).join("\n");
    assert!(text.contains("línea 1: size"), "{text}");
    f.client.cancel().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn the_low_level_tools_use_the_main_canvas() {
    let (dir, cwd) = project();
    let f = start(dir.path(), &cwd).await;
    let drawn = call(
        &f,
        "draw",
        serde_json::json!({ "commands": "set_pixel 1 1 red\nfill_rect 9 9 2 2 blue" }),
    )
    .await;
    let output = texts(&drawn).join("");
    assert!(output.contains("batch: 1 ok, 1 failed"), "{output}");
    let view = call(&f, "view_canvas", serde_json::json!({})).await;
    let canvas = lume_store::decode_png(&image(&view).unwrap()).unwrap();
    // 8 px ampliados ×32 (el máximo).
    assert_eq!(canvas.width(), 256);
    assert_eq!(canvas.get(32, 32).unwrap(), Rgba::rgb(255, 0, 0));
    let exported = call(
        &f,
        "export",
        serde_json::json!({ "path": "lienzo.png", "scale": 2 }),
    )
    .await;
    assert_ne!(exported.is_error, Some(true), "{:?}", texts(&exported));
    let file = FsStore.read_png(&cwd.join("lienzo.png")).unwrap();
    assert_eq!(file.width(), 16);
    assert_eq!(file.get(2, 2).unwrap(), Rgba::rgb(255, 0, 0));
    f.client.cancel().await.unwrap();
}
