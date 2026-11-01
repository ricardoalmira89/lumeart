//! `lumeart mcp`: el servidor MCP por la entrada y la salida estándar.
//!
//! Si hay un studio (o una sesión de `lumeart start`) abierto, los encargos y
//! el lienzo van a él por su API ([`RemoteArt`]) y aparecen allí; si no, se
//! hacen en este proceso sin ventana ([`LocalArt`]). Nada se escribe en la
//! salida estándar salvo el protocolo MCP.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use lume_api::{ApiServer, write_state_file};
use lume_app::art::{ArtAgent, ArtAgents, LocalArt};
use lume_app::ports::{BoxFuture, CancellationToken, PortError};
use lume_app::{
    ArtId, ArtService, ArtSpec, ArtStatus, DocumentStore, SecretStore, Session, SessionHandle,
    SettingsStore,
};
use lume_cli::http::{self, HttpError, HttpResponse};
use lume_domain::{Document, SessionInfo};
use lume_mcp::LumeMcp;
use lume_store::{FsProject, FsStore};

use crate::agent::WindowAgent;

/// Monta el agente de cada encargo: una API propia para su sesión (la que
/// usa Claude Code con `lumeart batch`) y el proveedor de los ajustes.
pub struct JobAgents {
    pub store: Arc<dyn DocumentStore>,
    pub secrets: Arc<dyn SecretStore>,
    pub claude: Option<PathBuf>,
    pub lumeart: PathBuf,
    /// Cada encargo trabaja en `<work_dir>/encargo-N/`.
    pub work_dir: PathBuf,
    /// Motor de animación para las herramientas de `anim.js`.
    pub anim: Option<Arc<dyn lume_app::AnimEngine>>,
    /// Motor de `dibujo.js` y rasterizador para las herramientas vectoriales.
    pub vector: Option<lume_agent::VecTools>,
    /// El plugin oficial lleva el bucle (variante A de `lumeart eval`).
    pub ralph: Option<lume_agent::RalphPlugin>,
}

/// Al soltarlo, para la API del encargo y borra su archivo de estado.
struct JobApi {
    stop: CancellationToken,
    session_file: PathBuf,
}

impl Drop for JobApi {
    fn drop(&mut self) {
        self.stop.cancel();
        let _ = std::fs::remove_file(&self.session_file);
    }
}

impl ArtAgents for JobAgents {
    fn agent(
        &self,
        session: SessionHandle,
        id: ArtId,
    ) -> BoxFuture<'_, Result<ArtAgent, PortError>> {
        Box::pin(async move {
            let io = |e: std::io::Error| PortError::Io(e.to_string());
            let server = ApiServer::start(session.clone(), self.store.clone())
                .await
                .map_err(io)?;
            let dir = self.work_dir.join(format!("encargo-{id}"));
            let session_file = dir.join("session.json");
            let guard = JobApi {
                stop: server.stop_requested(),
                session_file: session_file.clone(),
            };
            write_state_file(&session_file, &server.info()).map_err(io)?;
            let agent = WindowAgent {
                session,
                store: self.store.clone(),
                secrets: self.secrets.clone(),
                claude: self.claude.clone(),
                lumeart: self.lumeart.clone(),
                session_file,
                work_dir: dir,
                anim: self.anim.clone(),
                vector: self.vector.clone(),
                ralph: self.ralph.clone(),
            };
            Ok(ArtAgent {
                backend: Arc::new(agent),
                guard: Box::new(guard),
            })
        })
    }
}

/// Los encargos de una sesión abierta en otro proceso (el studio), por su
/// API local.
pub struct RemoteArt {
    pub info: SessionInfo,
}

impl RemoteArt {
    async fn call(
        &self,
        method: &'static str,
        route: String,
        body: Vec<u8>,
    ) -> Result<HttpResponse, PortError> {
        let info = self.info.clone();
        let response = tokio::task::spawn_blocking(move || {
            http::request(info.port, &info.token, method, &route, &body)
        })
        .await
        .map_err(|e| PortError::Unavailable(e.to_string()))?
        .map_err(|e| match e {
            HttpError::Connect(_) => PortError::Unavailable(
                "el studio ya no contesta; ábrelo otra vez o reinicia el servidor MCP".into(),
            ),
            HttpError::Io(m) => PortError::Io(m),
        })?;
        let text = || response.text().trim_end().to_string();
        match response.status {
            200 => Ok(response),
            400 => Err(PortError::Invalid(text())),
            503 => Err(PortError::Unavailable(text())),
            504 => Err(PortError::Timeout),
            _ => Err(PortError::Io(text())),
        }
    }

    async fn status_of(
        &self,
        method: &'static str,
        route: String,
        body: Vec<u8>,
    ) -> Result<ArtStatus, PortError> {
        let response = self.call(method, route, body).await?;
        serde_json::from_slice(&response.body)
            .map_err(|e| PortError::Invalid(format!("respuesta del studio: {e}")))
    }
}

impl ArtService for RemoteArt {
    fn create(&self, spec: ArtSpec) -> BoxFuture<'_, Result<ArtStatus, PortError>> {
        Box::pin(async move {
            let body = serde_json::to_vec(&spec).map_err(|e| PortError::Invalid(e.to_string()))?;
            self.status_of("POST", "/v1/art".into(), body).await
        })
    }

    fn status(&self, id: ArtId) -> BoxFuture<'_, Result<ArtStatus, PortError>> {
        Box::pin(async move {
            self.status_of("GET", format!("/v1/art/{id}"), Vec::new())
                .await
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<ArtStatus>, PortError>> {
        Box::pin(async move {
            let response = self.call("GET", "/v1/art".into(), Vec::new()).await?;
            serde_json::from_slice(&response.body)
                .map_err(|e| PortError::Invalid(format!("respuesta del studio: {e}")))
        })
    }

    fn preview(&self, id: ArtId, side: u32) -> BoxFuture<'_, Result<Vec<u8>, PortError>> {
        Box::pin(async move {
            let route = format!("/v1/art/{id}/preview.png?side={side}");
            Ok(self.call("GET", route, Vec::new()).await?.body)
        })
    }

    fn draw(
        &self,
        script: String,
        cwd: Option<PathBuf>,
    ) -> BoxFuture<'_, Result<String, PortError>> {
        Box::pin(async move {
            let mut route = "/v1/commands?batch=1".to_string();
            if let Some(cwd) = cwd {
                route.push_str(&format!(
                    "&cwd={}",
                    http::encode_query(&cwd.to_string_lossy())
                ));
            }
            Ok(self.call("POST", route, script.into_bytes()).await?.text())
        })
    }

    fn canvas_png(&self, side: u32) -> BoxFuture<'_, Result<Vec<u8>, PortError>> {
        Box::pin(async move {
            let route = format!("/v1/canvas.png?side={side}");
            Ok(self.call("GET", route, Vec::new()).await?.body)
        })
    }
}

/// El documento del studio abierto (su pestaña activa) y su ventana, por la
/// API local.
pub struct RemoteStudio {
    pub art: Arc<RemoteArt>,
}

/// `?tab=N` (o `&tab=N`) para una ruta, si hay pestaña.
fn with_tab(route: &str, tab: Option<u64>) -> String {
    match tab {
        None => route.to_string(),
        Some(t) if route.contains('?') => format!("{route}&tab={t}"),
        Some(t) => format!("{route}?tab={t}"),
    }
}

impl lume_app::studio::StudioAccess for RemoteStudio {
    fn snapshot(&self, tab: Option<u64>) -> BoxFuture<'_, Result<Document, PortError>> {
        Box::pin(async move {
            let bytes = self
                .art
                .call("GET", with_tab("/v1/document.lume", tab), Vec::new())
                .await?
                .body;
            let tmp = std::env::temp_dir().join(format!(
                "lumeart-mcp-{}-{}.lume",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
            std::fs::write(&tmp, &bytes).map_err(|e| PortError::Io(e.to_string()))?;
            let doc = lume_app::DocumentStore::open_document(&FsStore, &tmp)
                .map_err(|e| PortError::Invalid(e.message));
            let _ = std::fs::remove_file(&tmp);
            doc
        })
    }

    fn set_vector(&self, tab: Option<u64>, source: String) -> BoxFuture<'_, Result<(), PortError>> {
        Box::pin(async move {
            self.art
                .call("PUT", with_tab("/v1/vector", tab), source.into_bytes())
                .await
                .map(|_| ())
        })
    }

    fn set_animation(
        &self,
        tab: Option<u64>,
        source: String,
    ) -> BoxFuture<'_, Result<(), PortError>> {
        Box::pin(async move {
            self.art
                .call("PUT", with_tab("/v1/animation", tab), source.into_bytes())
                .await
                .map(|_| ())
        })
    }

    fn undo(
        &self,
        tab: Option<u64>,
        steps: u32,
        redo: bool,
    ) -> BoxFuture<'_, Result<u32, PortError>> {
        Box::pin(async move {
            let route = with_tab(&format!("/v1/history?steps={steps}&redo={redo}"), tab);
            let r = self.art.call("POST", route, Vec::new()).await?;
            Ok(r.text().trim().parse().unwrap_or(0))
        })
    }

    fn save_version(
        &self,
        tab: Option<u64>,
        message: String,
    ) -> BoxFuture<'_, Result<u32, PortError>> {
        Box::pin(async move {
            let r = self
                .art
                .call("POST", with_tab("/v1/versions", tab), message.into_bytes())
                .await?;
            r.text()
                .trim()
                .parse()
                .map_err(|_| PortError::Invalid(r.text()))
        })
    }

    fn restore_version(
        &self,
        tab: Option<u64>,
        number: u32,
    ) -> BoxFuture<'_, Result<(), PortError>> {
        Box::pin(async move {
            let route = with_tab(&format!("/v1/versions/{number}/restore"), tab);
            self.art.call("POST", route, Vec::new()).await.map(|_| ())
        })
    }

    fn window(
        &self,
        tab: Option<u64>,
        request: lume_app::studio::WindowRequest,
    ) -> BoxFuture<'_, Result<lume_app::studio::WindowState, PortError>> {
        Box::pin(async move {
            let body =
                serde_json::to_vec(&request).map_err(|e| PortError::Invalid(e.to_string()))?;
            let r = self
                .art
                .call("POST", with_tab("/v1/window", tab), body)
                .await?;
            serde_json::from_slice(&r.body)
                .map_err(|e| PortError::Invalid(format!("respuesta de la ventana: {e}")))
        })
    }
}

/// Los motores con que el MCP pinta y exporta (los mismos que la ventana).
fn engines() -> lume_mcp::Engines {
    let (_, services) = crate::vec_cmd::window_vector(
        std::env::temp_dir().join(format!("lumeart-mcp-vec-{}", std::process::id())),
    );
    lume_mcp::Engines {
        vector: services.engine,
        rasterizer: services.rasterizer,
        anim: Arc::new(lume_script::QuickJsAnim),
        store: Arc::new(FsStore),
        exporter: services.exporter,
    }
}

/// Un studio visto: su archivo de estado y cómo hablarle.
type Seen = (SessionInfo, Arc<RemoteArt>, Arc<RemoteStudio>);

/// Lleva cada llamada al studio abierto (si lo hay en ese momento) o a la
/// sesión propia de este proceso.
struct Router {
    state: Option<PathBuf>,
    local: Arc<dyn ArtService>,
    local_studio: Arc<dyn lume_app::studio::StudioAccess>,
    /// El último studio visto, para no comprobarlo en cada llamada.
    remote: std::sync::Mutex<Option<Seen>>,
}

impl Router {
    /// A quién va esta llamada.
    async fn current(&self) -> (Arc<dyn ArtService>, Arc<dyn lume_app::studio::StudioAccess>) {
        let info = self
            .state
            .as_deref()
            .and_then(|p| lume_cli::read_session(p).ok());
        let Some(info) = info else {
            return (self.local.clone(), self.local_studio.clone());
        };
        if let Ok(cache) = self.remote.lock()
            && let Some((seen, art, studio)) = cache.as_ref()
            && *seen == info
        {
            return (art.clone(), studio.clone());
        }
        let probe = info.clone();
        let alive = tokio::task::spawn_blocking(move || crate::daemon::is_alive(&probe))
            .await
            .unwrap_or(false);
        if !alive {
            return (self.local.clone(), self.local_studio.clone());
        }
        eprintln!("lumeart mcp: con el studio abierto (puerto {})", info.port);
        let art = Arc::new(RemoteArt { info: info.clone() });
        let studio = Arc::new(RemoteStudio { art: art.clone() });
        if let Ok(mut cache) = self.remote.lock() {
            *cache = Some((info, art.clone(), studio.clone()));
        }
        (art, studio)
    }

    /// El studio dejó de contestar: la próxima vez se vuelve a mirar.
    fn forget(&self, e: &PortError) {
        if matches!(e, PortError::Unavailable(_))
            && let Ok(mut cache) = self.remote.lock()
        {
            *cache = None;
        }
    }
}

impl ArtService for Router {
    fn create(&self, spec: ArtSpec) -> BoxFuture<'_, Result<ArtStatus, PortError>> {
        Box::pin(async move {
            let r = self.current().await.0.create(spec).await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn status(&self, id: ArtId) -> BoxFuture<'_, Result<ArtStatus, PortError>> {
        Box::pin(async move {
            let r = self.current().await.0.status(id).await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<ArtStatus>, PortError>> {
        Box::pin(async move {
            let r = self.current().await.0.list().await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn preview(&self, id: ArtId, side: u32) -> BoxFuture<'_, Result<Vec<u8>, PortError>> {
        Box::pin(async move {
            let r = self.current().await.0.preview(id, side).await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn draw(
        &self,
        script: String,
        cwd: Option<PathBuf>,
    ) -> BoxFuture<'_, Result<String, PortError>> {
        Box::pin(async move {
            let r = self.current().await.0.draw(script, cwd).await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn canvas_png(&self, side: u32) -> BoxFuture<'_, Result<Vec<u8>, PortError>> {
        Box::pin(async move {
            let r = self.current().await.0.canvas_png(side).await;
            r.inspect_err(|e| self.forget(e))
        })
    }
}

impl lume_app::studio::StudioAccess for Router {
    fn snapshot(&self, tab: Option<u64>) -> BoxFuture<'_, Result<Document, PortError>> {
        Box::pin(async move {
            let r = self.current().await.1.snapshot(tab).await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn set_vector(&self, tab: Option<u64>, source: String) -> BoxFuture<'_, Result<(), PortError>> {
        Box::pin(async move {
            let r = self.current().await.1.set_vector(tab, source).await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn set_animation(
        &self,
        tab: Option<u64>,
        source: String,
    ) -> BoxFuture<'_, Result<(), PortError>> {
        Box::pin(async move {
            let r = self.current().await.1.set_animation(tab, source).await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn undo(
        &self,
        tab: Option<u64>,
        steps: u32,
        redo: bool,
    ) -> BoxFuture<'_, Result<u32, PortError>> {
        Box::pin(async move {
            let r = self.current().await.1.undo(tab, steps, redo).await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn save_version(
        &self,
        tab: Option<u64>,
        message: String,
    ) -> BoxFuture<'_, Result<u32, PortError>> {
        Box::pin(async move {
            let r = self.current().await.1.save_version(tab, message).await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn restore_version(
        &self,
        tab: Option<u64>,
        number: u32,
    ) -> BoxFuture<'_, Result<(), PortError>> {
        Box::pin(async move {
            let r = self.current().await.1.restore_version(tab, number).await;
            r.inspect_err(|e| self.forget(e))
        })
    }

    fn window(
        &self,
        tab: Option<u64>,
        request: lume_app::studio::WindowRequest,
    ) -> BoxFuture<'_, Result<lume_app::studio::WindowState, PortError>> {
        Box::pin(async move {
            let r = self.current().await.1.window(tab, request).await;
            r.inspect_err(|e| self.forget(e))
        })
    }
}

/// `lumeart mcp`. Devuelve el código de salida.
pub fn run(state: Option<&Path>, cwd: PathBuf) -> i32 {
    let runtime = tokio::runtime::Runtime::new().expect("runtime de tokio");
    let work_dir = std::env::temp_dir().join(format!("lumeart-mcp-{}", std::process::id()));
    let result = runtime.block_on(async {
        // Una sesión propia por si no hay ventana; con ventana, todo va a
        // ella. Se mira en cada llamada: da igual abrir antes una u otra.
        let store: Arc<dyn DocumentStore> = Arc::new(FsStore);
        let session = Session::spawn(
            Document::new("sin título", 32, 32).map_err(|e| e.to_string())?,
            store.clone(),
        );
        let local_studio: Arc<dyn lume_app::studio::StudioAccess> = Arc::new(
            lume_app::studio::SessionStudio::new(session.handle.clone(), store.clone()),
        );
        let local: Arc<dyn ArtService> = Arc::new(local_art(
            session.handle.clone(),
            store,
            crate::daemon::settings_store(),
            work_dir.clone(),
        ));
        let router = Arc::new(Router {
            state: state.map(Path::to_path_buf),
            local,
            local_studio,
            remote: std::sync::Mutex::new(None),
        });
        let served = LumeMcp::new(router.clone(), Arc::new(FsProject), cwd)
            .with_studio(router, engines())
            .serve_stdio()
            .await;
        session.close().await;
        served
    });
    if work_dir.exists() {
        let _ = std::fs::remove_dir_all(&work_dir);
    }
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {e}");
            1
        }
    }
}

/// Encargos en este proceso, con el agente de los ajustes.
pub fn local_art(
    main: SessionHandle,
    store: Arc<dyn DocumentStore>,
    settings: Arc<dyn SettingsStore>,
    work_dir: PathBuf,
) -> LocalArt {
    LocalArt::new(
        main,
        store.clone(),
        settings,
        Arc::new(job_agents(store, work_dir)),
    )
}

/// Los agentes de los encargos y de las variantes (cada uno con su sesión).
pub fn job_agents(store: Arc<dyn DocumentStore>, work_dir: PathBuf) -> JobAgents {
    JobAgents {
        store: store.clone(),
        secrets: Arc::new(lume_store::KeyringSecrets::default()),
        claude: lume_agent::find_claude(),
        lumeart: std::env::current_exe().unwrap_or_else(|_| PathBuf::from("lumeart")),
        work_dir,
        anim: Some(Arc::new(lume_script::QuickJsAnim)),
        vector: None,
        ralph: None,
    }
}
