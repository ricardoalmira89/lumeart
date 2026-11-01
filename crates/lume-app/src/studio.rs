//! Lo que otro agente (por MCP) puede hacer con el studio, además de los
//! encargos: leer el documento entero, cambiar su dibujo vectorial o su
//! animación, deshacer, guardar versiones y, si hay ventana, manejarla
//! (pestañas, modo, abrir y guardar, el agente del panel).
//!
//! [`StudioAccess`] lo implementan [`SessionStudio`] (una sesión de este
//! proceso, sin ventana) y la raíz (el studio abierto, por su API). Las
//! peticiones a la ventana pasan por un [`WindowChannel`]: la API las deja y
//! la ventana las atiende en su siguiente fotograma.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use lume_domain::anim::animation::{AnimEdit, Animation};
use lume_domain::vec::drawing::{VecEdit, VectorDrawing};
use lume_domain::{Document, Origin};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use crate::ports::{BoxFuture, DocumentStore, PortError};
use crate::session::SessionHandle;

/// Qué tipo de documento se crea.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DocKind {
    #[default]
    Pixel,
    Vector,
    Animation,
}

/// Una petición a la ventana.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WindowRequest {
    /// Solo el estado.
    State,
    /// Documento nuevo en una pestaña nueva.
    New {
        doc: DocKind,
        width: u32,
        height: u32,
        #[serde(default)]
        palette: Option<String>,
    },
    /// Abrir un `.lume` (o un PNG) en una pestaña nueva.
    Open {
        path: PathBuf,
    },
    /// Guardar el de la pestaña activa (en su archivo o en `path`).
    Save {
        #[serde(default)]
        path: Option<PathBuf>,
    },
    /// Pasar a una pestaña (por su número de orden; con `tab` en la
    /// petición, por su id).
    SelectTab {
        #[serde(default)]
        index: usize,
    },
    CloseTab {
        #[serde(default)]
        index: usize,
    },
    /// Renombrar el documento activo.
    Rename {
        name: String,
    },
    /// Modo de la pestaña activa: `pixel`, `vector` o `animation`.
    SetMode {
        mode: DocKind,
    },
    /// Pedir algo al agente del panel (sobre la pestaña activa).
    AskAgent {
        prompt: String,
    },
    StopAgent,
}

/// Una pestaña de la ventana.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabInfo {
    /// Id fijo de la pestaña (no cambia al cerrar otras): el `tab` de las
    /// herramientas.
    #[serde(default)]
    pub id: u64,
    pub index: usize,
    pub title: String,
    pub mode: DocKind,
    pub dirty: bool,
    pub active: bool,
    /// Su agente está trabajando.
    #[serde(default)]
    pub agent_running: bool,
    /// Es un encargo (`create_art`), con su número.
    #[serde(default)]
    pub job: Option<u64>,
}

/// Cómo está la ventana tras una petición.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WindowState {
    pub tabs: Vec<TabInfo>,
    /// La pestaña de la que habla la respuesta (la creada, la pedida o la
    /// activa).
    #[serde(default)]
    pub tab: Option<u64>,
    /// Ruta del archivo de la pestaña activa, si tiene.
    pub file: Option<String>,
    /// El agente del panel está trabajando.
    pub agent_running: bool,
    /// Lo último que dijo el agente del panel (sus últimos mensajes).
    pub agent_messages: Vec<String>,
    /// Aviso de la ventana (por ejemplo, por qué no se pudo).
    pub notice: Option<String>,
}

/// Lo que despierta a la ventana.
pub type Wake = Arc<dyn Fn() + Send + Sync>;

/// Una petición en espera de la ventana: la petición, la pestaña a la que va
/// (`None`: la activa) y dónde contestar.
pub type WindowCall = (
    WindowRequest,
    Option<u64>,
    oneshot::Sender<Result<WindowState, String>>,
);

/// Las sesiones de las pestañas de la ventana, por id, y cuál es la activa.
/// La ventana lo publica en cada fotograma; la API lo usa para que cada
/// petición vaya a su pestaña.
#[derive(Clone, Default)]
pub struct TabSessions {
    inner: Arc<Mutex<Published>>,
}

/// Lo publicado: (id, sesión) de cada pestaña y el id de la activa.
type Published = (Vec<(u64, SessionHandle)>, Option<u64>);

impl TabSessions {
    /// Pone las pestañas y la activa.
    pub fn publish(&self, tabs: Vec<(u64, SessionHandle)>, active: Option<u64>) {
        if let Ok(mut inner) = self.inner.lock() {
            *inner = (tabs, active);
        }
    }

    /// La sesión de la pestaña `id`.
    pub fn get(&self, id: u64) -> Option<SessionHandle> {
        let inner = self.inner.lock().ok()?;
        inner
            .0
            .iter()
            .find(|(t, _)| *t == id)
            .map(|(_, s)| s.clone())
    }

    /// La sesión de la pestaña activa (si la ventana ya publicó).
    pub fn active(&self) -> Option<SessionHandle> {
        let id = self.inner.lock().ok()?.1?;
        self.get(id)
    }

    /// Los ids de las pestañas.
    pub fn ids(&self) -> Vec<u64> {
        self.inner
            .lock()
            .map(|i| i.0.iter().map(|(id, _)| *id).collect())
            .unwrap_or_default()
    }
}

/// El canal entre la API (o quien sea) y la ventana. La ventana se queda
/// con el receptor ([`WindowChannel::take_inbox`]) y llama a `wake` para
/// que se repinte al llegar algo.
#[derive(Clone)]
pub struct WindowChannel {
    /// Las sesiones de las pestañas.
    pub tabs: TabSessions,
    tx: mpsc::UnboundedSender<WindowCall>,
    inbox: Arc<Mutex<Option<mpsc::UnboundedReceiver<WindowCall>>>>,
    wake: Arc<Mutex<Option<Wake>>>,
}

impl Default for WindowChannel {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tabs: TabSessions::default(),
            tx,
            inbox: Arc::new(Mutex::new(Some(rx))),
            wake: Arc::new(Mutex::new(None)),
        }
    }
}

impl std::fmt::Debug for WindowChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WindowChannel")
    }
}

impl WindowChannel {
    /// El receptor de las peticiones (solo una vez: lo coge la ventana).
    pub fn take_inbox(&self) -> Option<mpsc::UnboundedReceiver<WindowCall>> {
        self.inbox.lock().ok()?.take()
    }

    /// Qué hacer para despertar a la ventana.
    pub fn set_wake(&self, wake: Arc<dyn Fn() + Send + Sync>) {
        if let Ok(mut w) = self.wake.lock() {
            *w = Some(wake);
        }
    }

    /// Manda una petición (a la pestaña `tab`, o a la activa) y espera la
    /// respuesta de la ventana.
    pub async fn request(
        &self,
        request: WindowRequest,
        tab: Option<u64>,
    ) -> Result<WindowState, PortError> {
        let (reply, answer) = oneshot::channel();
        self.tx
            .send((request, tab, reply))
            .map_err(|_| PortError::Unavailable("la ventana está cerrada".into()))?;
        if let Some(wake) = self.wake.lock().ok().and_then(|w| w.clone()) {
            wake();
        }
        match tokio::time::timeout(std::time::Duration::from_secs(30), answer).await {
            Ok(Ok(result)) => result.map_err(PortError::Invalid),
            Ok(Err(_)) => Err(PortError::Unavailable("la ventana no contestó".into())),
            Err(_) => Err(PortError::Timeout),
        }
    }
}

/// Lo que se puede hacer con el documento del studio (el de la pestaña
/// activa, si hay ventana).
///
/// Cada operación va a la pestaña `tab` (su id) o, con `None`, a la activa.
pub trait StudioAccess: Send + Sync {
    /// El documento entero: lienzo, historial, versiones, dibujo vectorial,
    /// animación y notas.
    fn snapshot(&self, tab: Option<u64>) -> BoxFuture<'_, Result<Document, PortError>>;
    /// Pone el código de `dibujo.js` (lo crea si no había).
    fn set_vector(&self, tab: Option<u64>, source: String) -> BoxFuture<'_, Result<(), PortError>>;
    /// Pone el código de `anim.js` (lo crea si no había).
    fn set_animation(
        &self,
        tab: Option<u64>,
        source: String,
    ) -> BoxFuture<'_, Result<(), PortError>>;
    /// Deshace (o rehace) `steps` pasos; devuelve cuántos se hicieron.
    fn undo(
        &self,
        tab: Option<u64>,
        steps: u32,
        redo: bool,
    ) -> BoxFuture<'_, Result<u32, PortError>>;
    /// Guarda una versión; devuelve su número.
    fn save_version(
        &self,
        tab: Option<u64>,
        message: String,
    ) -> BoxFuture<'_, Result<u32, PortError>>;
    fn restore_version(
        &self,
        tab: Option<u64>,
        number: u32,
    ) -> BoxFuture<'_, Result<(), PortError>>;
    /// Una petición a la ventana (o, sin ventana, lo que tenga sentido sobre
    /// la sesión: estado, nuevo, abrir y guardar).
    fn window(
        &self,
        tab: Option<u64>,
        request: WindowRequest,
    ) -> BoxFuture<'_, Result<WindowState, PortError>>;
}

/// Sin ventana solo hay una pestaña, la 0.
fn only_tab(tab: Option<u64>) -> Result<(), PortError> {
    match tab {
        None | Some(0) => Ok(()),
        Some(t) => Err(PortError::Invalid(format!(
            "no hay pestaña {t}: sin la ventana de LumeArt abierta solo hay un documento (la pestaña 0)"
        ))),
    }
}

fn app(e: crate::AppError) -> PortError {
    PortError::Invalid(e.to_string())
}

/// Una sesión de este proceso, sin ventana.
pub struct SessionStudio {
    pub session: SessionHandle,
    pub store: Arc<dyn DocumentStore>,
    /// Dónde está guardado el documento, si lo está.
    pub path: Mutex<Option<PathBuf>>,
}

impl SessionStudio {
    pub fn new(session: SessionHandle, store: Arc<dyn DocumentStore>) -> Self {
        Self {
            session,
            store,
            path: Mutex::new(None),
        }
    }

    fn state(&self) -> WindowState {
        let vm = self.session.view().borrow().clone();
        let path = self.path.lock().ok().and_then(|p| p.clone());
        let mode = if vm.vector.is_some() {
            DocKind::Vector
        } else if vm.animation.is_some() {
            DocKind::Animation
        } else {
            DocKind::Pixel
        };
        WindowState {
            tabs: vec![TabInfo {
                id: 0,
                index: 0,
                title: match &path {
                    Some(p) => p
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                    None => format!("{}.lume", vm.name),
                },
                mode,
                dirty: false,
                active: true,
                agent_running: false,
                job: None,
            }],
            tab: Some(0),
            file: path.map(|p| p.display().to_string()),
            ..WindowState::default()
        }
    }
}

/// Un documento nuevo del tipo pedido (el vectorial, con su `dibujo.js`
/// vacío de `width` × `height`; el de animación, con un `anim.js` vacío).
pub fn new_document(
    doc: DocKind,
    width: u32,
    height: u32,
    palette: Option<&str>,
) -> Result<Document, PortError> {
    let (pw, ph) = match doc {
        DocKind::Vector => (32, 32),
        _ => (width, height),
    };
    let mut document = Document::new("sin título", i64::from(pw), i64::from(ph))
        .map_err(|e| PortError::Invalid(e.message))?;
    document.palette = palette.and_then(lume_domain::Palette::fixed);
    if doc == DocKind::Vector {
        let source = lume_domain::vec::template(width, height, false);
        document.vector = Some(VectorDrawing::new(source));
    }
    Ok(document)
}

impl StudioAccess for SessionStudio {
    fn snapshot(&self, tab: Option<u64>) -> BoxFuture<'_, Result<Document, PortError>> {
        Box::pin(async move {
            only_tab(tab)?;
            self.session.snapshot().await.map_err(app)
        })
    }

    fn set_vector(&self, tab: Option<u64>, source: String) -> BoxFuture<'_, Result<(), PortError>> {
        Box::pin(async move {
            only_tab(tab)?;
            let has = self.session.view().borrow().vector.is_some();
            let edit = if has {
                VecEdit::Source(source)
            } else {
                VecEdit::Set(Some(VectorDrawing::new(source)))
            };
            self.session
                .edit_vector(edit, Origin::Cli)
                .await
                .map(|_| ())
                .map_err(app)
        })
    }

    fn set_animation(
        &self,
        tab: Option<u64>,
        source: String,
    ) -> BoxFuture<'_, Result<(), PortError>> {
        Box::pin(async move {
            only_tab(tab)?;
            let has = self.session.view().borrow().animation.is_some();
            let edit = if has {
                AnimEdit::Source(source)
            } else {
                AnimEdit::Set(Some(Animation {
                    source,
                    ..Animation::default()
                }))
            };
            self.session
                .edit_animation(edit, Origin::Cli)
                .await
                .map(|_| ())
                .map_err(app)
        })
    }

    fn undo(
        &self,
        tab: Option<u64>,
        steps: u32,
        redo: bool,
    ) -> BoxFuture<'_, Result<u32, PortError>> {
        Box::pin(async move {
            only_tab(tab)?;
            let line = if redo { "redo" } else { "undo" };
            let mut done = 0;
            for _ in 0..steps {
                let r = self
                    .session
                    .line(line, Origin::Cli, None)
                    .await
                    .map_err(app)?;
                if !r.response.is_ok() {
                    break;
                }
                done += 1;
            }
            Ok(done)
        })
    }

    fn save_version(
        &self,
        tab: Option<u64>,
        message: String,
    ) -> BoxFuture<'_, Result<u32, PortError>> {
        Box::pin(async move {
            only_tab(tab)?;
            self.session
                .create_version(Origin::Cli, message)
                .await
                .map_err(app)
        })
    }

    fn restore_version(
        &self,
        tab: Option<u64>,
        number: u32,
    ) -> BoxFuture<'_, Result<(), PortError>> {
        Box::pin(async move {
            only_tab(tab)?;
            self.session
                .restore_version(number, Origin::Cli)
                .await
                .map_err(app)
        })
    }

    fn window(
        &self,
        tab: Option<u64>,
        request: WindowRequest,
    ) -> BoxFuture<'_, Result<WindowState, PortError>> {
        Box::pin(async move {
            only_tab(tab)?;
            match request {
                WindowRequest::State => {}
                WindowRequest::New {
                    doc,
                    width,
                    height,
                    palette,
                } => {
                    let document = new_document(doc, width, height, palette.as_deref())?;
                    self.session.replace_document(document).await.map_err(app)?;
                    if let Ok(mut p) = self.path.lock() {
                        *p = None;
                    }
                }
                WindowRequest::Open { path } => {
                    let document = self
                        .store
                        .open_document(&path)
                        .map_err(|e| PortError::Invalid(e.message))?;
                    self.session.replace_document(document).await.map_err(app)?;
                    if let Ok(mut p) = self.path.lock() {
                        *p = Some(path);
                    }
                }
                WindowRequest::Save { path } => {
                    let path = path
                        .or_else(|| self.path.lock().ok().and_then(|p| p.clone()))
                        .ok_or_else(|| {
                            PortError::Invalid(
                                "el documento no tiene archivo: di dónde guardarlo (path)".into(),
                            )
                        })?;
                    let document = self.session.snapshot().await.map_err(app)?;
                    self.store
                        .save_document(&document, &path)
                        .map_err(|e| PortError::Invalid(e.message))?;
                    if let Ok(mut p) = self.path.lock() {
                        *p = Some(path);
                    }
                }
                WindowRequest::Rename { name } => {
                    self.session.set_name(name).await.map_err(app)?;
                }
                WindowRequest::SelectTab { .. }
                | WindowRequest::CloseTab { .. }
                | WindowRequest::SetMode { .. }
                | WindowRequest::AskAgent { .. }
                | WindowRequest::StopAgent => {
                    return Err(PortError::Unavailable(
                        "no hay ninguna ventana de LumeArt abierta: ábrela para usar pestañas, modos o su agente".into(),
                    ));
                }
            }
            Ok(self.state())
        })
    }
}
