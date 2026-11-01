//! `Session`: la única dueña de un documento abierto (patrón actor).
//!
//! Es una tarea `tokio` con una bandeja `mpsc`. Ventana, CLI, API, MCP y
//! agente le envían mensajes con un `oneshot` para la respuesta; la sesión los
//! aplica de uno en uno, en orden de llegada, así que no hay cerrojos sobre el
//! lienzo ni carreras. Tras cada cambio publica un `ViewModel` nuevo (canal
//! `watch`: la ventana lee el último) y un evento con el patch (canal
//! `broadcast`: agente, SSE, registro).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use lume_domain::anim::animation::AnimEdit;
use lume_domain::protocol::parse_request;
use lume_domain::vec::drawing::VecEdit;
use lume_domain::{
    BatchReport, Command, Document, DomainError, Executed, Origin, Patch, Request, Response,
};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::task::JoinHandle;

use crate::ports::DocumentStore;
use crate::view::{Change, SessionEvent, ViewModel};

/// Tamaño de la bandeja de mensajes.
const INBOX: usize = 256;
/// Eventos que se guardan para suscriptores lentos.
const EVENTS: usize = 1024;

/// Error de la capa de aplicación.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AppError {
    /// La sesión ya terminó.
    #[error("session closed")]
    Closed,
    #[error(transparent)]
    Domain(#[from] DomainError),
}

/// Resultado de una línea del protocolo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineResult {
    pub response: Response,
    /// Revisión del documento tras la petición.
    pub revision: u64,
    /// La petición era `stop`: quien sirve la sesión debe pararla.
    pub stop: bool,
}

/// Mensajes que entiende la sesión.
enum Msg {
    Line {
        line: String,
        origin: Origin,
        cwd: Option<PathBuf>,
        reply: oneshot::Sender<LineResult>,
    },
    Batch {
        script: String,
        origin: Origin,
        cwd: Option<PathBuf>,
        reply: oneshot::Sender<Result<BatchReport, DomainError>>,
    },
    Apply {
        command: Command,
        origin: Origin,
        /// Etiqueta del paso de historial («Lápiz · 3 píxeles»); sin ella, el
        /// nombre del comando.
        label: Option<String>,
        reply: oneshot::Sender<Result<Patch, DomainError>>,
    },
    Snapshot {
        reply: oneshot::Sender<Document>,
    },
    Replace {
        document: Box<Document>,
        reply: oneshot::Sender<()>,
    },
    SetName {
        name: String,
        reply: oneshot::Sender<()>,
    },
    SetNotes {
        notes: String,
        reply: oneshot::Sender<()>,
    },
    EditAnimation {
        edit: AnimEdit,
        origin: Origin,
        reply: oneshot::Sender<Result<bool, DomainError>>,
    },
    EditVector {
        edit: VecEdit,
        origin: Origin,
        reply: oneshot::Sender<Result<bool, DomainError>>,
    },
    CreateVersion {
        author: Origin,
        message: String,
        reply: oneshot::Sender<u32>,
    },
    RestoreVersion {
        number: u32,
        origin: Origin,
        reply: oneshot::Sender<Result<(), DomainError>>,
    },
    Close,
}

/// Mango para hablar con una sesión. Se puede clonar y enviar entre tareas.
#[derive(Clone)]
pub struct SessionHandle {
    inbox: mpsc::Sender<Msg>,
    view: watch::Receiver<Arc<ViewModel>>,
    events: broadcast::Sender<SessionEvent>,
}

/// Una sesión en marcha: su mango y la tarea que la ejecuta.
pub struct Session {
    pub handle: SessionHandle,
    task: JoinHandle<Document>,
}

impl Session {
    /// Arranca la sesión de un documento en el runtime de `tokio` actual.
    pub fn spawn(document: Document, store: Arc<dyn DocumentStore>) -> Session {
        let (inbox, rx) = mpsc::channel(INBOX);
        let (view_tx, view) = watch::channel(Arc::new(ViewModel::of(&document, 0, None)));
        let (events, _) = broadcast::channel(EVENTS);
        let actor = Actor {
            document,
            store,
            revision: 0,
            view: view_tx,
            events: events.clone(),
        };
        let task = tokio::spawn(actor.run(rx));
        Session {
            handle: SessionHandle {
                inbox,
                view,
                events,
            },
            task,
        }
    }

    /// Cierra la sesión y espera a que su tarea termine. Devuelve el
    /// documento tal como quedó.
    pub async fn close(self) -> Document {
        let _ = self.handle.inbox.send(Msg::Close).await;
        self.task
            .await
            .expect("la tarea de la sesión no debe entrar en pánico")
    }
}

impl SessionHandle {
    /// ¿Hablan con la misma sesión?
    pub fn same(&self, other: &SessionHandle) -> bool {
        self.inbox.same_channel(&other.inbox)
    }

    async fn ask<T>(&self, make: impl FnOnce(oneshot::Sender<T>) -> Msg) -> Result<T, AppError> {
        let (reply, answer) = oneshot::channel();
        self.inbox
            .send(make(reply))
            .await
            .map_err(|_| AppError::Closed)?;
        answer.await.map_err(|_| AppError::Closed)
    }

    /// Ejecuta una línea del protocolo. `cwd` es la carpeta de quien la envía
    /// (para resolver la ruta relativa de `export`).
    pub async fn line(
        &self,
        line: impl Into<String>,
        origin: Origin,
        cwd: Option<PathBuf>,
    ) -> Result<LineResult, AppError> {
        let line = line.into();
        self.ask(|reply| Msg::Line {
            line,
            origin,
            cwd,
            reply,
        })
        .await
    }

    /// Ejecuta un `batch` como un solo paso de deshacer.
    pub async fn batch(
        &self,
        script: impl Into<String>,
        origin: Origin,
        cwd: Option<PathBuf>,
    ) -> Result<BatchReport, AppError> {
        let script = script.into();
        Ok(self
            .ask(|reply| Msg::Batch {
                script,
                origin,
                cwd,
                reply,
            })
            .await??)
    }

    /// Aplica un comando (el lápiz y el agente interno lo crean directamente).
    pub async fn apply(&self, command: Command, origin: Origin) -> Result<Patch, AppError> {
        Ok(self
            .ask(|reply| Msg::Apply {
                command,
                origin,
                label: None,
                reply,
            })
            .await??)
    }

    /// Aplica un comando como un paso de historial con esta etiqueta (un
    /// trazo del lápiz: «Lápiz · 3 píxeles»).
    pub async fn apply_labeled(
        &self,
        command: Command,
        origin: Origin,
        label: impl Into<String>,
    ) -> Result<Patch, AppError> {
        let label = Some(label.into());
        Ok(self
            .ask(|reply| Msg::Apply {
                command,
                origin,
                label,
                reply,
            })
            .await??)
    }

    /// Sustituye el documento de la sesión (abrir un archivo o empezar uno
    /// nuevo). El historial de deshacer empieza de cero.
    pub async fn replace_document(&self, document: Document) -> Result<(), AppError> {
        let document = Box::new(document);
        self.ask(|reply| Msg::Replace { document, reply }).await
    }

    /// Cambia el nombre del documento (al guardarlo con otro nombre). No es un
    /// cambio del dibujo: no crece la revisión.
    pub async fn set_name(&self, name: impl Into<String>) -> Result<(), AppError> {
        let name = name.into();
        self.ask(|reply| Msg::SetName { name, reply }).await
    }

    /// Sustituye las notas del documento (`notas.md`).
    pub async fn set_notes(&self, notes: impl Into<String>) -> Result<(), AppError> {
        let notes = notes.into();
        self.ask(|reply| Msg::SetNotes { notes, reply }).await
    }

    /// Cambia la animación del documento (un paso de deshacer). `false` si
    /// no cambió nada.
    pub async fn edit_animation(&self, edit: AnimEdit, origin: Origin) -> Result<bool, AppError> {
        Ok(self
            .ask(|reply| Msg::EditAnimation {
                edit,
                origin,
                reply,
            })
            .await??)
    }

    /// Cambia el dibujo vectorial del documento (un paso de deshacer).
    /// `false` si no cambió nada.
    pub async fn edit_vector(&self, edit: VecEdit, origin: Origin) -> Result<bool, AppError> {
        Ok(self
            .ask(|reply| Msg::EditVector {
                edit,
                origin,
                reply,
            })
            .await??)
    }

    /// Guarda una versión con el lienzo y la receta de ahora. Devuelve su
    /// número.
    pub async fn create_version(
        &self,
        author: Origin,
        message: impl Into<String>,
    ) -> Result<u32, AppError> {
        let message = message.into();
        self.ask(|reply| Msg::CreateVersion {
            author,
            message,
            reply,
        })
        .await
    }

    /// Vuelve a una versión (un paso de deshacer).
    pub async fn restore_version(&self, number: u32, origin: Origin) -> Result<(), AppError> {
        Ok(self
            .ask(|reply| Msg::RestoreVersion {
                number,
                origin,
                reply,
            })
            .await??)
    }

    /// Copia del documento completo (para guardarlo, por ejemplo).
    pub async fn snapshot(&self) -> Result<Document, AppError> {
        self.ask(|reply| Msg::Snapshot { reply }).await
    }

    /// El último `ViewModel` publicado y los que vengan.
    pub fn view(&self) -> watch::Receiver<Arc<ViewModel>> {
        self.view.clone()
    }

    /// Suscripción a los eventos de la sesión.
    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.events.subscribe()
    }

    /// ¿Sigue viva la sesión?
    pub fn is_open(&self) -> bool {
        !self.inbox.is_closed()
    }
}

/// Estado de la tarea de la sesión.
struct Actor {
    document: Document,
    store: Arc<dyn DocumentStore>,
    revision: u64,
    view: watch::Sender<Arc<ViewModel>>,
    events: broadcast::Sender<SessionEvent>,
}

impl Actor {
    async fn run(mut self, mut inbox: mpsc::Receiver<Msg>) -> Document {
        while let Some(msg) = inbox.recv().await {
            match msg {
                Msg::Line {
                    line,
                    origin,
                    cwd,
                    reply,
                } => {
                    let result = self.line(&line, origin, cwd.as_deref());
                    let _ = reply.send(result);
                }
                Msg::Batch {
                    script,
                    origin,
                    cwd,
                    reply,
                } => {
                    let result = self.batch(&script, origin, cwd.as_deref());
                    let _ = reply.send(result);
                }
                Msg::Apply {
                    command,
                    origin,
                    label,
                    reply,
                } => {
                    let result = self.apply(command, origin, label);
                    let _ = reply.send(result);
                }
                Msg::Snapshot { reply } => {
                    let _ = reply.send(self.document.clone());
                }
                Msg::SetName { name, reply } => {
                    self.document.name = name;
                    let last = self.view.borrow().last_change.clone();
                    let view = ViewModel::of(&self.document, self.revision, last);
                    self.view.send_replace(Arc::new(view));
                    let _ = reply.send(());
                }
                Msg::SetNotes { notes, reply } => {
                    self.document.notes = notes;
                    self.touched();
                    let _ = reply.send(());
                }
                Msg::EditAnimation {
                    edit,
                    origin,
                    reply,
                } => {
                    let label = edit.label();
                    let result = self.document.edit_animation(edit, origin);
                    if let Ok(true) = result {
                        self.changed(origin, label, Patch::new());
                    }
                    let _ = reply.send(result);
                }
                Msg::EditVector {
                    edit,
                    origin,
                    reply,
                } => {
                    let label = edit.label();
                    let result = self.document.edit_vector(edit, origin);
                    if let Ok(true) = result {
                        self.changed(origin, label, Patch::new());
                    }
                    let _ = reply.send(result);
                }
                Msg::CreateVersion {
                    author,
                    message,
                    reply,
                } => {
                    let number = self.document.create_version(author, message).number;
                    self.touched();
                    let _ = reply.send(number);
                }
                Msg::RestoreVersion {
                    number,
                    origin,
                    reply,
                } => {
                    let result = self.document.restore_version(number, origin);
                    let result = result.map(|patch| {
                        self.changed(origin, format!("volver a la versión {number}"), patch);
                    });
                    let _ = reply.send(result);
                }
                Msg::Replace { document, reply } => {
                    let patch = Patch::between(self.document.canvas(), document.canvas());
                    self.document = *document;
                    self.changed(Origin::User, "abrir".to_string(), patch);
                    let _ = reply.send(());
                }
                Msg::Close => break,
            }
        }
        let _ = self.events.send(SessionEvent::Closed);
        self.document
    }

    /// El documento cambió sin tocar el dibujo (notas, versiones): revisión
    /// nueva (hay algo que guardar) y `ViewModel`, sin evento de cambio.
    fn touched(&mut self) {
        self.revision += 1;
        let last = self.view.borrow().last_change.clone();
        let view = ViewModel::of(&self.document, self.revision, last);
        self.view.send_replace(Arc::new(view));
    }

    /// Publica un cambio: revisión nueva, `ViewModel` y evento.
    fn changed(&mut self, origin: Origin, label: String, patch: Patch) {
        self.changed_by_lines(origin, label, patch, Vec::new());
    }

    /// Como `changed`, con las líneas del protocolo que hicieron el cambio.
    fn changed_by_lines(
        &mut self,
        origin: Origin,
        label: String,
        patch: Patch,
        lines: Vec<String>,
    ) {
        self.revision += 1;
        let change = Change {
            revision: self.revision,
            origin,
            label,
            patch: Arc::new(patch),
            lines: Arc::new(lines),
        };
        let view = ViewModel::of(&self.document, self.revision, Some(change.clone()));
        self.view.send_replace(Arc::new(view));
        let _ = self.events.send(SessionEvent::Changed(change));
    }

    fn apply(
        &mut self,
        command: Command,
        origin: Origin,
        label: Option<String>,
    ) -> Result<Patch, DomainError> {
        let Some(label) = label else {
            let text = command.to_string();
            let result = self.document.apply(command, origin);
            if let Ok(patch) = &result {
                self.changed(origin, text, patch.clone());
            }
            return result;
        };
        // Con etiqueta: una transacción de un comando, para que el historial
        // muestre la etiqueta.
        self.document.commit();
        self.document.begin(origin, label.clone())?;
        match self.document.apply(command, origin) {
            Ok(patch) => {
                self.document.commit();
                self.changed(origin, label, patch.clone());
                Ok(patch)
            }
            Err(err) => {
                let _ = self.document.rollback();
                Err(err)
            }
        }
    }

    fn line(&mut self, line: &str, origin: Origin, cwd: Option<&Path>) -> LineResult {
        let request = parse_request(line);
        let mutates = matches!(
            request,
            Ok(Request::Apply(_) | Request::Undo | Request::Redo | Request::New { .. })
        );
        let before = mutates.then(|| self.document.canvas().clone());
        let mut stop = false;
        let response = match request {
            Err(err) => Response::Err(err),
            Ok(request) => match self.document.execute(request, origin) {
                Executed::Reply(response) => response,
                Executed::Export { path, scale } => {
                    let path = resolve(cwd, &path);
                    match self.store.export_png(self.document.canvas(), &path, scale) {
                        Ok(()) => Response::ok(),
                        Err(err) => Response::Err(err),
                    }
                }
                Executed::Stop => {
                    stop = true;
                    Response::ok()
                }
            },
        };
        if let (Some(before), true) = (before, response.is_ok()) {
            let patch = Patch::between(&before, self.document.canvas());
            let line = line.trim().to_string();
            self.changed_by_lines(origin, line.clone(), patch, vec![line]);
        }
        let _ = self.events.send(SessionEvent::Command {
            origin,
            line: line.trim().to_string(),
            response: response.to_string(),
        });
        LineResult {
            response,
            revision: self.revision,
            stop,
        }
    }

    fn batch(
        &mut self,
        script: &str,
        origin: Origin,
        cwd: Option<&Path>,
    ) -> Result<BatchReport, DomainError> {
        let before = self.document.canvas().clone();
        let store = Arc::clone(&self.store);
        let resolve_path = |p: &str| resolve(cwd, p).to_string_lossy().into_owned();
        let report = self.document.run_batch(
            script,
            origin,
            &resolve_path,
            &mut |canvas, path, scale| store.export_png(canvas, Path::new(path), scale),
        )?;
        if report.ok > 0 {
            let patch = Patch::between(&before, self.document.canvas());
            let lines = lume_domain::batch::split_batch(script)
                .into_iter()
                .map(|l| l.text)
                .collect();
            self.changed_by_lines(origin, "batch".to_string(), patch, lines);
        }
        let _ = self.events.send(SessionEvent::Command {
            origin,
            line: format!("batch ({} comandos)", report.total()),
            response: report.summary(),
        });
        Ok(report)
    }
}

/// Ruta relativa a la carpeta de quien envía la petición, limpia de `.` y
/// `..` sin tocar el disco (como `filepath.Abs` de Go en pxcli).
fn resolve(cwd: Option<&Path>, path: &str) -> PathBuf {
    let path = Path::new(path);
    let joined = match cwd {
        Some(cwd) if path.is_relative() => cwd.join(path),
        _ => path.to_path_buf(),
    };
    let mut clean = PathBuf::new();
    for part in joined.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                clean.pop();
            }
            other => clean.push(other.as_os_str()),
        }
    }
    clean
}
