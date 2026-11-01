//! API local de LumeArt.
//!
//! Servidor `axum` en `127.0.0.1` con puerto libre y token aleatorio,
//! publicados en un archivo de estado en la carpeta de datos del usuario.
//! Toda petición lleva `Authorization: Bearer <token>`; sin él, 401.
//!
//! | Ruta | Qué hace |
//! | --- | --- |
//! | `POST /v1/commands` | Líneas del protocolo → una respuesta por línea (`ok`, `err …`). Con `?batch=1`, el texto es un `batch`: un paso de deshacer y la salida de `batch` (líneas + `batch: N ok, M failed`; cabecera `x-lumeart-failed`). `?cwd=` es la carpeta del cliente (para `export` con ruta relativa) |
//! | `GET /v1/canvas.png?scale=N` | PNG del lienzo actual |
//! | `GET /v1/events` | Eventos (SSE): `change`, `command`, `closed` |
//! | `POST /v1/session/new?size=WxH` | `new W H` |
//! | `POST /v1/undo`, `POST /v1/redo` | `undo`, `redo` |
//! | `POST /v1/stop` | Pide parar el servidor |
//! | `POST /v1/art` | Empieza un encargo (`ArtSpec` en JSON) y devuelve su estado |
//! | `GET /v1/art`, `GET /v1/art/{id}` | Encargos y su estado (JSON) |
//! | `GET /v1/art/{id}/preview.png?side=N` | Lienzo del encargo, ampliado hacia N px |
//!
//! `GET /v1/canvas.png` admite también `?side=N` en vez de `scale`. Las rutas
//! de encargos solo responden si la sesión los acepta (el studio y
//! `lumeart start`); si no, 503.
//!
//! Regla: depende de `lume-app` y `lume-domain`, nunca de otro adaptador.

use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path as AxumPath, Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use lume_app::art::scale_for;
use lume_app::ports::CancellationToken;
use lume_app::{
    AppError, ArtId, ArtService, ArtSpec, DocumentStore, PortError, SessionEvent, SessionHandle,
};
use lume_domain::discovery::AUTH_SCHEME;
use lume_domain::{DomainError, Origin, SessionInfo};
use serde::Deserialize;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

/// Cabecera con el número de comandos que fallaron en un `batch`.
pub const FAILED_HEADER: &str = "x-lumeart-failed";

/// Estado compartido por las rutas.
#[derive(Clone)]
struct ApiState {
    session: SessionHandle,
    store: Arc<dyn DocumentStore>,
    token: Arc<str>,
    stop: CancellationToken,
    art: Option<Arc<dyn ArtService>>,
    /// El documento de la sesión para el MCP (vector, animación, versiones).
    studio: Arc<lume_app::studio::SessionStudio>,
    /// La ventana, si la hay (pestañas, modo, su agente).
    window: Option<lume_app::studio::WindowChannel>,
}

impl ApiState {
    /// La sesión de la pestaña `tab` (su id) o, sin `tab`, la de la pestaña
    /// activa. Sin ventana solo está la sesión propia (la pestaña 0).
    #[allow(clippy::result_large_err)]
    fn session_for(&self, tab: Option<u64>) -> Result<SessionHandle, Response> {
        let missing = |t: u64| {
            text(
                StatusCode::BAD_REQUEST,
                format!("no hay pestaña {t}: mira las que hay con list_tabs\n"),
            )
        };
        match (&self.window, tab) {
            (Some(window), Some(t)) => window.tabs.get(t).ok_or_else(|| missing(t)),
            (Some(window), None) => {
                Ok(window.tabs.active().unwrap_or_else(|| self.session.clone()))
            }
            (None, None | Some(0)) => Ok(self.session.clone()),
            (None, Some(t)) => Err(missing(t)),
        }
    }

    /// El documento de esa pestaña para el MCP.
    #[allow(clippy::result_large_err)]
    fn studio_for(
        &self,
        tab: Option<u64>,
    ) -> Result<Arc<lume_app::studio::SessionStudio>, Response> {
        let session = self.session_for(tab)?;
        // Sin ventana, la sesión propia (que recuerda dónde se guardó).
        if self.window.is_none() {
            return Ok(self.studio.clone());
        }
        Ok(Arc::new(lume_app::studio::SessionStudio::new(
            session,
            self.store.clone(),
        )))
    }
}

/// `?tab=N`: la pestaña a la que va la petición (sin él, la activa).
#[derive(Debug, Default, Deserialize)]
struct TabQuery {
    tab: Option<u64>,
}

/// Rutas de la API. `stop` se cancela cuando un cliente pide parar.
pub fn router(
    session: SessionHandle,
    store: Arc<dyn DocumentStore>,
    token: &str,
    stop: CancellationToken,
) -> Router {
    router_with_art(session, store, token, stop, None)
}

/// Rutas de la API con los encargos de `art`.
pub fn router_with_art(
    session: SessionHandle,
    store: Arc<dyn DocumentStore>,
    token: &str,
    stop: CancellationToken,
    art: Option<Arc<dyn ArtService>>,
) -> Router {
    router_full(session, store, token, stop, art, None)
}

/// Todas las rutas: encargos y, con `window`, las peticiones a la ventana.
pub fn router_full(
    session: SessionHandle,
    store: Arc<dyn DocumentStore>,
    token: &str,
    stop: CancellationToken,
    art: Option<Arc<dyn ArtService>>,
    window: Option<lume_app::studio::WindowChannel>,
) -> Router {
    let studio = Arc::new(lume_app::studio::SessionStudio::new(
        session.clone(),
        store.clone(),
    ));
    let state = ApiState {
        session,
        store,
        token: Arc::from(token),
        stop,
        art,
        studio,
        window,
    };
    Router::new()
        .route("/v1/commands", post(commands))
        .route("/v1/canvas.png", get(canvas_png))
        .route("/v1/events", get(events))
        .route("/v1/session/new", post(new_session))
        .route("/v1/undo", post(undo))
        .route("/v1/redo", post(redo))
        .route("/v1/stop", post(stop_server))
        .route("/v1/art", post(art_create).get(art_list))
        .route("/v1/art/{id}", get(art_status))
        .route("/v1/art/{id}/preview.png", get(art_preview))
        .route("/v1/document.lume", get(document_lume))
        .route("/v1/vector", axum::routing::put(put_vector))
        .route("/v1/animation", axum::routing::put(put_animation))
        .route("/v1/history", post(history_steps))
        .route("/v1/versions", post(version_create))
        .route("/v1/versions/{n}/restore", post(version_restore))
        .route("/v1/window", post(window_request))
        .layer(middleware::from_fn_with_state(state.clone(), require_token))
        .with_state(state)
}

/// Comparación que tarda lo mismo acierte o no (para no filtrar el token).
fn same_token(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn require_token(State(state): State<ApiState>, request: Request, next: Next) -> Response {
    let ok = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix(AUTH_SCHEME))
        .and_then(|v| v.strip_prefix(' '))
        .is_some_and(|token| same_token(token.as_bytes(), state.token.as_bytes()));
    if !ok {
        return text(
            StatusCode::UNAUTHORIZED,
            "err unauthorized missing or invalid token\n".to_string(),
        );
    }
    next.run(request).await
}

fn text(status: StatusCode, body: String) -> Response {
    let mut response = (status, body).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response
}

fn domain_error(err: &DomainError) -> String {
    format!("{}\n", lume_domain::Response::Err(err.clone()))
}

fn app_error(err: AppError) -> Response {
    match err {
        AppError::Closed => text(
            StatusCode::SERVICE_UNAVAILABLE,
            "err closed session closed\n".into(),
        ),
        AppError::Domain(e) => text(StatusCode::UNPROCESSABLE_ENTITY, domain_error(&e)),
    }
}

#[derive(Debug, Default, Deserialize)]
struct CommandsQuery {
    batch: Option<String>,
    cwd: Option<String>,
    /// Ejecución del agente que manda los comandos (`LUMEART_RUN`).
    run: Option<u64>,
    /// Pestaña a la que van (sin ella, la activa).
    tab: Option<u64>,
}

async fn commands(
    State(state): State<ApiState>,
    Query(query): Query<CommandsQuery>,
    body: String,
) -> Response {
    let session = match state.session_for(query.tab) {
        Ok(s) => s,
        Err(r) => return r,
    };
    let cwd = query.cwd.map(PathBuf::from);
    let origin = query
        .run
        .map(|run| Origin::Agent { run })
        .unwrap_or(Origin::Cli);
    let batch = query
        .batch
        .as_deref()
        .is_some_and(|b| !matches!(b, "" | "0" | "false"));
    if batch {
        return match session.batch(body, origin, cwd).await {
            Ok(report) => {
                let failed = report.failed;
                let mut response = text(StatusCode::OK, report.output());
                response
                    .headers_mut()
                    .insert(FAILED_HEADER, HeaderValue::from(failed));
                response
            }
            Err(err) => app_error(err),
        };
    }

    let lines: Vec<&str> = body.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return text(
            StatusCode::OK,
            "err invalid_command command is required\n".into(),
        );
    }
    let mut out = String::new();
    let mut stop = false;
    for line in lines {
        match session.line(line, origin, cwd.clone()).await {
            Ok(result) => {
                out.push_str(&result.response.to_string());
                out.push('\n');
                stop |= result.stop;
            }
            Err(err) => return app_error(err),
        }
    }
    if stop {
        state.stop.cancel();
    }
    text(StatusCode::OK, out)
}

#[derive(Debug, Default, Deserialize)]
struct CanvasQuery {
    scale: Option<String>,
    /// Lado aproximado en píxeles (alternativa a `scale`).
    side: Option<u32>,
    tab: Option<u64>,
}

async fn canvas_png(State(state): State<ApiState>, Query(query): Query<CanvasQuery>) -> Response {
    let session = match state.session_for(query.tab) {
        Ok(s) => s,
        Err(r) => return r,
    };
    let canvas = session.view().borrow().canvas.clone();
    let scale = match (query.side, query.scale.as_deref().map(str::parse::<u32>)) {
        (Some(side), _) => scale_for(
            (i64::from(canvas.width()), i64::from(canvas.height())),
            i64::from(side),
        ),
        (None, None) => 1,
        (None, Some(Ok(scale))) => scale,
        (None, Some(Err(_))) => {
            return text(
                StatusCode::BAD_REQUEST,
                "err invalid_args scale must be an integer\n".into(),
            );
        }
    };
    match state.store.encode_png(&canvas, scale) {
        Ok(bytes) => {
            let mut response = Response::new(Body::from(bytes));
            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
            response
        }
        Err(err) => text(StatusCode::BAD_REQUEST, domain_error(&err)),
    }
}

fn origin_json(origin: Origin) -> serde_json::Value {
    match origin {
        Origin::User => "user".into(),
        Origin::Cli => "cli".into(),
        Origin::Agent { run } => serde_json::json!({ "agent": run }),
    }
}

/// Un evento de la sesión como evento SSE (`change`, `command` o `closed`).
fn sse_event(event: SessionEvent) -> Event {
    match event {
        SessionEvent::Changed(change) => Event::default().event("change").data(
            serde_json::json!({
                "revision": change.revision,
                "origin": origin_json(change.origin),
                "label": change.label,
                "cells": change.patch.cell_count(),
            })
            .to_string(),
        ),
        SessionEvent::Command {
            origin,
            line,
            response,
        } => Event::default().event("command").data(
            serde_json::json!({
                "origin": origin_json(origin),
                "line": line,
                "response": response,
            })
            .to_string(),
        ),
        SessionEvent::Closed => Event::default().event("closed").data("{}"),
    }
}

async fn events(State(state): State<ApiState>, Query(q): Query<TabQuery>) -> Response {
    let session = match state.session_for(q.tab) {
        Ok(s) => s,
        Err(r) => return r,
    };
    // Si un cliente se queda atrás, se saltan los eventos perdidos.
    let stream = BroadcastStream::new(session.subscribe())
        .filter_map(|event| event.ok())
        .map(|event| Ok::<_, Infallible>(sse_event(event)));
    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

#[derive(Debug, Default, Deserialize)]
struct NewQuery {
    size: Option<String>,
}

async fn new_session(State(state): State<ApiState>, Query(query): Query<NewQuery>) -> Response {
    let size = query.size.unwrap_or_else(|| "32x32".into());
    let Some((w, h)) = size.split_once(['x', 'X']) else {
        return text(
            StatusCode::BAD_REQUEST,
            format!("err invalid_args invalid size {size:?}: expected WxH\n"),
        );
    };
    one_line(&state, &format!("new {w} {h}")).await
}

async fn one_line(state: &ApiState, line: &str) -> Response {
    let session = match state.session_for(None) {
        Ok(s) => s,
        Err(r) => return r,
    };
    match session.line(line, Origin::Cli, None).await {
        Ok(result) => text(StatusCode::OK, format!("{}\n", result.response)),
        Err(err) => app_error(err),
    }
}

async fn undo(State(state): State<ApiState>) -> Response {
    one_line(&state, "undo").await
}

async fn redo(State(state): State<ApiState>) -> Response {
    one_line(&state, "redo").await
}

async fn stop_server(State(state): State<ApiState>) -> Response {
    state.stop.cancel();
    text(StatusCode::OK, "ok\n".into())
}

// --- El documento y la ventana (para el MCP) -------------------------------

/// El documento entero como `.lume` (lienzo, historial, versiones, dibujo
/// vectorial, animación y notas).
async fn document_lume(State(state): State<ApiState>, Query(q): Query<TabQuery>) -> Response {
    use lume_app::studio::StudioAccess;
    let studio = match state.studio_for(q.tab) {
        Ok(s) => s,
        Err(r) => return r,
    };
    let document = match studio.snapshot(None).await {
        Ok(d) => d,
        Err(e) => return port_error(e),
    };
    let tmp = std::env::temp_dir().join(format!("lumeart-api-{}.lume", generate_token()));
    let saved = state.store.save_document(&document, &tmp);
    let bytes = saved
        .map_err(|e| e.message)
        .and_then(|()| std::fs::read(&tmp).map_err(|e| e.to_string()));
    let _ = std::fs::remove_file(&tmp);
    match bytes {
        Ok(bytes) => {
            let mut response = (StatusCode::OK, bytes).into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );
            response
        }
        Err(e) => text(StatusCode::INTERNAL_SERVER_ERROR, format!("{e}\n")),
    }
}

async fn put_vector(
    State(state): State<ApiState>,
    Query(q): Query<TabQuery>,
    body: String,
) -> Response {
    use lume_app::studio::StudioAccess;
    let studio = match state.studio_for(q.tab) {
        Ok(s) => s,
        Err(r) => return r,
    };
    match studio.set_vector(None, body).await {
        Ok(()) => text(StatusCode::OK, "ok\n".into()),
        Err(e) => port_error(e),
    }
}

async fn put_animation(
    State(state): State<ApiState>,
    Query(q): Query<TabQuery>,
    body: String,
) -> Response {
    use lume_app::studio::StudioAccess;
    let studio = match state.studio_for(q.tab) {
        Ok(s) => s,
        Err(r) => return r,
    };
    match studio.set_animation(None, body).await {
        Ok(()) => text(StatusCode::OK, "ok\n".into()),
        Err(e) => port_error(e),
    }
}

#[derive(Deserialize)]
struct StepsQuery {
    steps: Option<u32>,
    redo: Option<bool>,
    tab: Option<u64>,
}

async fn history_steps(State(state): State<ApiState>, Query(q): Query<StepsQuery>) -> Response {
    use lume_app::studio::StudioAccess;
    let studio = match state.studio_for(q.tab) {
        Ok(s) => s,
        Err(r) => return r,
    };
    match studio
        .undo(None, q.steps.unwrap_or(1), q.redo.unwrap_or(false))
        .await
    {
        Ok(done) => text(StatusCode::OK, format!("{done}\n")),
        Err(e) => port_error(e),
    }
}

async fn version_create(
    State(state): State<ApiState>,
    Query(q): Query<TabQuery>,
    body: String,
) -> Response {
    use lume_app::studio::StudioAccess;
    let studio = match state.studio_for(q.tab) {
        Ok(s) => s,
        Err(r) => return r,
    };
    match studio.save_version(None, body).await {
        Ok(n) => text(StatusCode::OK, format!("{n}\n")),
        Err(e) => port_error(e),
    }
}

async fn version_restore(
    State(state): State<ApiState>,
    AxumPath(n): AxumPath<u32>,
    Query(q): Query<TabQuery>,
) -> Response {
    use lume_app::studio::StudioAccess;
    let studio = match state.studio_for(q.tab) {
        Ok(s) => s,
        Err(r) => return r,
    };
    match studio.restore_version(None, n).await {
        Ok(()) => text(StatusCode::OK, "ok\n".into()),
        Err(e) => port_error(e),
    }
}

/// Una petición a la ventana (JSON de `WindowRequest`); sin ventana, sobre
/// la sesión.
async fn window_request(
    State(state): State<ApiState>,
    Query(q): Query<TabQuery>,
    body: String,
) -> Response {
    use lume_app::studio::{StudioAccess, WindowRequest};
    let request: WindowRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => return text(StatusCode::BAD_REQUEST, format!("petición: {e}\n")),
    };
    let result = match &state.window {
        Some(window) => window.request(request, q.tab).await,
        None => state.studio.window(q.tab, request).await,
    };
    match result {
        Ok(window_state) => {
            let mut response = (
                StatusCode::OK,
                serde_json::to_string(&window_state).unwrap_or_default(),
            )
                .into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
            response
        }
        Err(e) => port_error(e),
    }
}

// --- Encargos ---------------------------------------------------------------

/// Un error de los encargos: código HTTP y el mensaje sin prefijo.
fn port_error(err: PortError) -> Response {
    let (status, message) = match err {
        PortError::Invalid(m) => (StatusCode::BAD_REQUEST, m),
        PortError::Unavailable(m) => (StatusCode::SERVICE_UNAVAILABLE, m),
        PortError::Timeout => (StatusCode::GATEWAY_TIMEOUT, PortError::Timeout.to_string()),
        other => (StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
    };
    text(status, message)
}

/// Respuesta de una sesión que no acepta encargos.
fn no_art() -> Response {
    text(
        StatusCode::SERVICE_UNAVAILABLE,
        "esta sesión no acepta encargos".into(),
    )
}

fn json(value: &impl serde::Serialize) -> Response {
    let body = serde_json::to_string(value).expect("los estados siempre se serializan");
    let mut response = (StatusCode::OK, body).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
}

async fn art_create(State(state): State<ApiState>, body: String) -> Response {
    let Some(art) = state.art.clone() else {
        return no_art();
    };
    let spec: ArtSpec = match serde_json::from_str(&body) {
        Ok(spec) => spec,
        Err(e) => return text(StatusCode::BAD_REQUEST, format!("encargo no válido: {e}")),
    };
    match art.create(spec).await {
        Ok(status) => json(&status),
        Err(e) => port_error(e),
    }
}

async fn art_list(State(state): State<ApiState>) -> Response {
    let Some(art) = state.art.clone() else {
        return no_art();
    };
    match art.list().await {
        Ok(list) => json(&list),
        Err(e) => port_error(e),
    }
}

async fn art_status(State(state): State<ApiState>, AxumPath(id): AxumPath<ArtId>) -> Response {
    let Some(art) = state.art.clone() else {
        return no_art();
    };
    match art.status(id).await {
        Ok(status) => json(&status),
        Err(e) => port_error(e),
    }
}

#[derive(Debug, Default, Deserialize)]
struct PreviewQuery {
    side: Option<u32>,
}

async fn art_preview(
    State(state): State<ApiState>,
    AxumPath(id): AxumPath<ArtId>,
    Query(query): Query<PreviewQuery>,
) -> Response {
    let Some(art) = state.art.clone() else {
        return no_art();
    };
    match art
        .preview(id, query.side.unwrap_or(lume_app::art::PREVIEW_SIDE as u32))
        .await
    {
        Ok(bytes) => {
            let mut response = Response::new(Body::from(bytes));
            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
            response
        }
        Err(e) => port_error(e),
    }
}

/// Token aleatorio de 32 bytes en hexadecimal.
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("el sistema debe dar números aleatorios");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Servidor de la API en marcha.
pub struct ApiServer {
    pub addr: SocketAddr,
    token: String,
    stop: CancellationToken,
    task: JoinHandle<std::io::Result<()>>,
}

impl ApiServer {
    /// Arranca la API en `127.0.0.1` con un puerto libre y un token nuevo.
    pub async fn start(
        session: SessionHandle,
        store: Arc<dyn DocumentStore>,
    ) -> std::io::Result<ApiServer> {
        Self::start_with_art(session, store, None).await
    }

    /// Igual, aceptando encargos con `art`.
    pub async fn start_with_art(
        session: SessionHandle,
        store: Arc<dyn DocumentStore>,
        art: Option<Arc<dyn ArtService>>,
    ) -> std::io::Result<ApiServer> {
        Self::start_full(session, store, art, None).await
    }

    /// Igual, y con la ventana (`window`) para las peticiones del MCP.
    pub async fn start_full(
        session: SessionHandle,
        store: Arc<dyn DocumentStore>,
        art: Option<Arc<dyn ArtService>>,
        window: Option<lume_app::studio::WindowChannel>,
    ) -> std::io::Result<ApiServer> {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let addr = listener.local_addr()?;
        let token = generate_token();
        let stop = CancellationToken::new();
        let app = router_full(session, store, &token, stop.clone(), art, window);
        let shutdown = stop.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move { shutdown.cancelled().await })
                .await
        });
        Ok(ApiServer {
            addr,
            token,
            stop,
            task,
        })
    }

    /// Datos para el archivo de estado.
    pub fn info(&self) -> SessionInfo {
        SessionInfo {
            port: self.addr.port(),
            pid: std::process::id(),
            token: self.token.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Se cancela cuando un cliente pide parar (`stop` o `POST /v1/stop`).
    pub fn stop_requested(&self) -> CancellationToken {
        self.stop.clone()
    }

    /// Para el servidor y espera a que termine.
    pub async fn shutdown(self) -> std::io::Result<()> {
        self.stop.cancel();
        self.task
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?
    }
}

/// Ruta por defecto del archivo de estado: `<datos del usuario>/lumeart/session.json`
/// (en Linux, `~/.local/share/lumeart/session.json`).
pub fn default_state_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "lumeart").map(|d| d.data_dir().join("session.json"))
}

/// Escribe el archivo de estado (solo legible por el usuario en Unix).
pub fn write_state_file(path: &Path, info: &SessionInfo) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_string_pretty(info).expect("SessionInfo siempre se serializa");
    std::fs::write(path, json)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Lee el archivo de estado.
pub fn read_state_file(path: &Path) -> std::io::Result<SessionInfo> {
    let bytes = std::fs::read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Cabeceras con el token para un cliente.
pub fn auth_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("{AUTH_SCHEME} {token}")).expect("el token es ASCII"),
    );
    headers
}
