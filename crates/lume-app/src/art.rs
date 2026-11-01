//! Encargos de arte: lo que piden otros agentes por MCP.
//!
//! Cada encargo es un documento con su propia sesión y su bucle de
//! refinado. Al terminar, su PNG y su `.lume` quedan donde diga el encargo
//! (la carpeta de `lumeart.toml`). [`ArtService`] es el puerto que usa el
//! servidor MCP: [`LocalArt`] hace los encargos en este proceso (el studio o
//! `lumeart mcp` sin ventana) y la raíz tiene otra implementación que se los
//! pasa por HTTP al studio abierto.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use lume_domain::guide::ArtGuide;
use lume_domain::{Document, Origin, Palette, Rgba, RunId};
use serde::{Deserialize, Serialize};

use crate::ports::{AgentBackend, BoxFuture, CancellationToken, DocumentStore, PortError};
use crate::refine::{LoopEnd, LoopEvent, LoopMessages, LoopSink, RefineLoop};
use crate::session::{Session, SessionHandle};
use crate::settings::SettingsStore;

/// Número de un encargo (también es el de la ejecución del agente).
pub type ArtId = RunId;

/// Plantilla del encargo para el agente: `{name}`, `{width}`, `{height}`,
/// `{description}` y `{details}`.
pub const ART_GOAL: &str = include_str!("../prompts/art_goal.md");

/// Paleta de un encargo (en texto, para poder viajar por la API).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteSpec {
    pub name: String,
    /// Colores `#rrggbbaa`.
    pub colors: Vec<String>,
}

impl PaletteSpec {
    /// La paleta de una guía, si limita los colores. Una lista propia se
    /// llama «del proyecto».
    pub fn of_guide(guide: &ArtGuide) -> Option<PaletteSpec> {
        let palette = guide.palette.palette()?;
        Some(PaletteSpec {
            name: palette
                .name
                .clone()
                .unwrap_or_else(|| "del proyecto".into()),
            colors: palette
                .colors()
                .iter()
                .map(|c| c.color.to_string())
                .collect(),
        })
    }

    fn palette(&self) -> Result<Palette, PortError> {
        let mut palette = Palette::new();
        palette.name = Some(self.name.clone());
        for color in &self.colors {
            let color = Rgba::parse(color)
                .map_err(|e| PortError::Invalid(format!("color de la paleta: {}", e.message)))?;
            palette.push(color, None);
        }
        Ok(palette)
    }
}

/// Lo que hay que dibujar, ya resuelto con la guía del proyecto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtSpec {
    /// Nombre de archivo, sin extensión (ya con la convención de la guía).
    pub name: String,
    /// Qué es y para qué sirve en el juego.
    pub description: String,
    pub size: (i64, i64),
    /// Vista (frontal, lateral, cenital…), si se pidió.
    pub view: String,
    /// Estilo: el de la guía y lo que añada el encargo.
    pub style: String,
    pub palette: Option<PaletteSpec>,
    /// Dónde quedan los archivos.
    pub png: PathBuf,
    pub lume: PathBuf,
    /// Revisión: el documento del que se parte y los cambios pedidos.
    pub base: Option<PathBuf>,
    pub changes: String,
}

impl ArtSpec {
    /// El encargo tal como lo lee el agente.
    pub fn goal(&self) -> String {
        let mut details = String::new();
        for (label, value) in [
            ("Vista", &self.view),
            ("Estilo", &self.style),
            ("Cambios pedidos en esta revisión", &self.changes),
        ] {
            let value = value.trim();
            if !value.is_empty() {
                details.push_str(&format!("{label}: {value}\n"));
            }
        }
        crate::refine::fill(
            ART_GOAL,
            &[
                ("name", &self.name),
                ("width", &self.size.0.to_string()),
                ("height", &self.size.1.to_string()),
                ("description", self.description.trim()),
                ("details", &details),
            ],
        )
    }
}

/// En qué punto está un encargo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtState {
    Running,
    Done,
    Failed,
    Cancelled,
}

impl ArtState {
    pub fn as_str(self) -> &'static str {
        match self {
            ArtState::Running => "running",
            ArtState::Done => "done",
            ArtState::Failed => "failed",
            ArtState::Cancelled => "cancelled",
        }
    }

    /// En español, para mostrarlo.
    pub fn label(self) -> &'static str {
        match self {
            ArtState::Running => "en marcha",
            ArtState::Done => "terminado",
            ArtState::Failed => "falló",
            ArtState::Cancelled => "cancelado",
        }
    }
}

/// Estado de un encargo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtStatus {
    pub id: ArtId,
    pub name: String,
    pub description: String,
    pub state: ArtState,
    /// Vuelta en curso (o la última) y cuántas puede haber.
    pub turn: u32,
    pub max_turns: u32,
    /// Qué está haciendo o cómo terminó, en una frase.
    pub message: String,
    pub png: PathBuf,
    pub lume: PathBuf,
    /// Si ya se escribieron los archivos.
    pub written: bool,
    /// Versiones del documento (una por vuelta).
    pub versions: usize,
    /// Revisión del documento cuando se escribieron los archivos.
    #[serde(default)]
    pub revision: u64,
}

/// Lo que ofrece LumeArt a otros agentes: encargos y el lienzo de siempre.
pub trait ArtService: Send + Sync {
    /// Empieza un encargo y devuelve su estado al momento (no espera).
    fn create(&self, spec: ArtSpec) -> BoxFuture<'_, Result<ArtStatus, PortError>>;
    fn status(&self, id: ArtId) -> BoxFuture<'_, Result<ArtStatus, PortError>>;
    /// Todos los encargos de esta sesión, del más antiguo al más nuevo.
    fn list(&self) -> BoxFuture<'_, Result<Vec<ArtStatus>, PortError>>;
    /// PNG del lienzo del encargo tal como está ahora, ampliado para que su
    /// lado mayor quede cerca de `side` píxeles.
    fn preview(&self, id: ArtId, side: u32) -> BoxFuture<'_, Result<Vec<u8>, PortError>>;
    /// Un `batch` del protocolo en el lienzo principal; devuelve su salida.
    fn draw(
        &self,
        script: String,
        cwd: Option<PathBuf>,
    ) -> BoxFuture<'_, Result<String, PortError>>;
    /// PNG del lienzo principal, ampliado como `preview`.
    fn canvas_png(&self, side: u32) -> BoxFuture<'_, Result<Vec<u8>, PortError>>;
}

/// El agente de un encargo y lo que hay que soltar al terminar (por
/// ejemplo, la API que usa Claude Code para dibujar en esa sesión).
pub struct ArtAgent {
    pub backend: Arc<dyn AgentBackend>,
    pub guard: Box<dyn std::any::Any + Send + Sync>,
}

/// Monta el agente que dibuja en la sesión de un encargo. Lo implementa la
/// raíz, que conoce los adaptadores.
pub trait ArtAgents: Send + Sync {
    fn agent(
        &self,
        session: SessionHandle,
        id: ArtId,
    ) -> BoxFuture<'_, Result<ArtAgent, PortError>>;
}

/// Lo que pasó en un encargo, en orden (para el panel del agente).
#[derive(Debug, Clone, PartialEq)]
pub enum JobEntry {
    /// Empieza una ejecución: el encargo de otro agente o un mensaje del
    /// usuario desde la ventana.
    Started {
        prompt: String,
    },
    Loop(LoopEvent),
    /// Terminó la ejecución, con la frase de cómo acabó.
    Finished {
        message: String,
        error: bool,
    },
}

/// Aviso de que algo cambió en los encargos (la ventana se repinta).
pub type ArtNotify = Arc<dyn Fn() + Send + Sync>;

struct Job {
    spec: Mutex<ArtSpec>,
    status: Arc<Mutex<ArtStatus>>,
    session: SessionHandle,
    cancel: Mutex<CancellationToken>,
    /// Buzón del bucle en marcha (para dirigirlo desde la ventana).
    messages: Mutex<Option<LoopMessages>>,
    log: Arc<Mutex<Vec<JobEntry>>>,
}

/// Encargos en este proceso.
pub struct LocalArt {
    main: SessionHandle,
    store: Arc<dyn DocumentStore>,
    settings: Arc<dyn SettingsStore>,
    agents: Arc<dyn ArtAgents>,
    jobs: Mutex<BTreeMap<ArtId, Arc<Job>>>,
    /// Sesiones de los encargos (se cierran al soltar `LocalArt`).
    sessions: Mutex<Vec<Session>>,
    notify: Arc<Mutex<Option<ArtNotify>>>,
}

/// Lado del PNG que se busca para las vistas previas.
pub const PREVIEW_SIDE: i64 = 256;

/// Escala para que el lado mayor del lienzo quede cerca de `side` píxeles.
pub fn scale_for(size: (i64, i64), side: i64) -> u32 {
    let longest = size.0.max(size.1).max(1);
    (side / longest).clamp(1, 32) as u32
}

fn ping(notify: &Mutex<Option<ArtNotify>>) {
    let callback = notify.lock().ok().and_then(|n| n.clone());
    if let Some(callback) = callback {
        callback();
    }
}

impl LocalArt {
    /// `main` es la sesión del lienzo principal (`draw`, `view_canvas`).
    pub fn new(
        main: SessionHandle,
        store: Arc<dyn DocumentStore>,
        settings: Arc<dyn SettingsStore>,
        agents: Arc<dyn ArtAgents>,
    ) -> Self {
        Self {
            main,
            store,
            settings,
            agents,
            jobs: Mutex::default(),
            sessions: Mutex::default(),
            notify: Arc::default(),
        }
    }

    /// Llama a `notify` cada vez que cambia algo de un encargo.
    pub fn set_notify(&self, notify: ArtNotify) {
        if let Ok(mut slot) = self.notify.lock() {
            *slot = Some(notify);
        }
    }

    fn job(&self, id: ArtId) -> Result<Arc<Job>, PortError> {
        self.jobs
            .lock()
            .map_err(|_| poisoned())?
            .get(&id)
            .cloned()
            .ok_or_else(|| unknown(id))
    }

    /// La sesión de un encargo (para abrirlo en la ventana).
    pub fn session(&self, id: ArtId) -> Option<SessionHandle> {
        self.job(id).ok().map(|j| j.session.clone())
    }

    /// Los encargos, del más antiguo al más nuevo.
    pub fn jobs(&self) -> Vec<ArtStatus> {
        let jobs: Vec<Arc<Job>> = match self.jobs.lock() {
            Ok(jobs) => jobs.values().cloned().collect(),
            Err(_) => return Vec::new(),
        };
        jobs.iter()
            .filter_map(|j| j.status.lock().ok().map(|s| s.clone()))
            .collect()
    }

    /// Lo que pasó en un encargo a partir de la entrada `from`.
    pub fn log(&self, id: ArtId, from: usize) -> Vec<JobEntry> {
        self.job(id)
            .ok()
            .and_then(|j| {
                j.log
                    .lock()
                    .ok()
                    .map(|log| log.get(from..).unwrap_or_default().to_vec())
            })
            .unwrap_or_default()
    }

    /// Pide parar un encargo.
    pub fn cancel(&self, id: ArtId) -> bool {
        match self
            .job(id)
            .ok()
            .and_then(|j| j.cancel.lock().ok().map(|c| c.clone()))
        {
            Some(cancel) => {
                cancel.cancel();
                true
            }
            None => false,
        }
    }

    /// Un mensaje para la vuelta siguiente del encargo en marcha. `false`
    /// si no está en marcha.
    pub fn send_message(&self, id: ArtId, text: &str) -> bool {
        let messages = self
            .job(id)
            .ok()
            .and_then(|j| j.messages.lock().ok().and_then(|m| m.clone()));
        match messages {
            Some(messages) => {
                messages.send(text);
                true
            }
            None => false,
        }
    }

    fn job_status(&self, id: ArtId) -> Result<ArtStatus, PortError> {
        let job = self.job(id)?;
        let status = job.status.lock().map_err(|_| poisoned())?.clone();
        Ok(status)
    }

    async fn start(&self, spec: ArtSpec) -> Result<ArtStatus, PortError> {
        let name = spec.name.trim().to_string();
        if name.is_empty() {
            return Err(PortError::Invalid("el encargo necesita un nombre".into()));
        }
        if spec.description.trim().is_empty() && spec.changes.trim().is_empty() {
            return Err(PortError::Invalid(
                "el encargo necesita una descripción".into(),
            ));
        }
        let busy = self
            .jobs()
            .iter()
            .any(|s| s.state == ArtState::Running && s.png == spec.png);
        if busy {
            return Err(PortError::Invalid(format!(
                "ya hay un encargo en marcha para «{name}»; espera a que termine con wait_art"
            )));
        }

        let mut document = match &spec.base {
            Some(path) => self
                .store
                .open_document(path)
                .map_err(|e| PortError::Io(e.message))?,
            None => Document::new(&name, spec.size.0, spec.size.1)
                .map_err(|e| PortError::Invalid(e.message))?,
        };
        document.name = name.clone();
        if let Some(palette) = &spec.palette {
            document.palette = Some(palette.palette()?);
        }
        let settings = self.settings.load()?;
        let id = self.next_id();

        let session = Session::spawn(document, self.store.clone());
        let handle = session.handle.clone();
        self.sessions.lock().map_err(|_| poisoned())?.push(session);
        let agent = self.agents.agent(handle.clone(), id).await?;

        let job = Arc::new(Job {
            spec: Mutex::new(spec.clone()),
            status: Arc::new(Mutex::new(ArtStatus {
                id,
                name,
                description: spec.description.trim().to_string(),
                state: ArtState::Running,
                turn: 0,
                max_turns: settings.max_turns.max(1),
                message: "Empezando".into(),
                png: spec.png.clone(),
                lume: spec.lume.clone(),
                written: false,
                versions: 0,
                revision: 0,
            })),
            session: handle,
            cancel: Mutex::new(CancellationToken::new()),
            messages: Mutex::new(None),
            log: Arc::default(),
        });
        self.jobs
            .lock()
            .map_err(|_| poisoned())?
            .insert(id, job.clone());
        let prompt = if spec.changes.trim().is_empty() {
            spec.description.trim().to_string()
        } else {
            spec.changes.trim().to_string()
        };
        self.run(id, &job, agent, settings, prompt, spec.goal());
        self.job_status(id)
    }

    /// Sigue un encargo terminado con lo que pide el usuario: otra vez el
    /// bucle sobre el mismo dibujo y, al final, los mismos archivos.
    pub async fn continue_job(&self, id: ArtId, changes: &str) -> Result<(), PortError> {
        let job = self.job(id)?;
        if self.job_status(id)?.state == ArtState::Running {
            return Err(PortError::Invalid(format!(
                "el encargo {id} sigue en marcha"
            )));
        }
        let changes = changes.trim();
        if changes.is_empty() {
            return Err(PortError::Invalid("falta qué hay que cambiar".into()));
        }
        let spec = {
            let mut spec = job.spec.lock().map_err(|_| poisoned())?;
            spec.changes = changes.to_string();
            spec.clone()
        };
        let settings = self.settings.load()?;
        let agent = self.agents.agent(job.session.clone(), id).await?;
        self.run(id, &job, agent, settings, changes.to_string(), spec.goal());
        Ok(())
    }

    /// Lanza el bucle de un encargo y, al terminar, escribe sus archivos.
    fn run(
        &self,
        id: ArtId,
        job: &Arc<Job>,
        agent: ArtAgent,
        settings: crate::settings::Settings,
        prompt: String,
        goal: String,
    ) {
        let max_turns = settings.max_turns.max(1);
        let refine = RefineLoop::new(
            agent.backend.clone(),
            job.session.clone(),
            self.store.clone(),
            settings,
        );
        let cancel = CancellationToken::new();
        if let Ok(mut slot) = job.cancel.lock() {
            *slot = cancel.clone();
        }
        if let Ok(mut slot) = job.messages.lock() {
            *slot = Some(refine.messages());
        }
        if let Ok(mut s) = job.status.lock() {
            s.state = ArtState::Running;
            s.turn = 0;
            s.max_turns = max_turns;
            s.message = "Empezando".into();
        }
        if let Ok(mut log) = job.log.lock() {
            log.push(JobEntry::Started { prompt });
        }
        ping(&self.notify);

        let job = job.clone();
        let store = self.store.clone();
        let notify = self.notify.clone();
        tokio::spawn(async move {
            let (sink, mut events) = LoopSink::channel();
            let running = refine.run(id, &goal, Vec::new(), sink, cancel);
            tokio::pin!(running);
            let summary = loop {
                tokio::select! {
                    event = events.recv() => {
                        let Some(event) = event else { continue };
                        if let LoopEvent::TurnStarted { turn, of } = &event
                            && let Ok(mut s) = job.status.lock()
                        {
                            s.turn = *turn;
                            s.max_turns = *of;
                            s.message = format!("Vuelta {turn} de {of}");
                        }
                        if let Ok(mut log) = job.log.lock() {
                            log.push(JobEntry::Loop(event));
                        }
                        ping(&notify);
                    }
                    summary = &mut running => break summary,
                }
            };
            while let Ok(event) = events.try_recv() {
                if let Ok(mut log) = job.log.lock() {
                    log.push(JobEntry::Loop(event));
                }
            }
            let spec = job.spec.lock().ok().map(|s| s.clone());
            let finished = match spec {
                Some(spec) => finish(&job.session, store.as_ref(), &spec).await,
                None => Err(poisoned()),
            };
            let entry = if let Ok(mut s) = job.status.lock() {
                s.turn = summary.turns.max(s.turn);
                s.message = summary.reason();
                match finished {
                    Ok(Some((versions, revision))) => {
                        s.written = true;
                        s.versions = versions;
                        s.revision = revision;
                        s.state = match summary.end {
                            LoopEnd::Cancelled => ArtState::Cancelled,
                            _ => ArtState::Done,
                        };
                    }
                    Ok(None) => {
                        s.state = match summary.end {
                            LoopEnd::Cancelled => ArtState::Cancelled,
                            _ => ArtState::Failed,
                        };
                        s.message
                            .push_str(" El lienzo quedó vacío: no se escribió nada.");
                    }
                    Err(e) => {
                        s.state = ArtState::Failed;
                        s.message
                            .push_str(&format!(" No se pudieron escribir los archivos: {e}"));
                    }
                }
                Some(JobEntry::Finished {
                    message: s.message.clone(),
                    error: s.state == ArtState::Failed,
                })
            } else {
                None
            };
            if let Ok(mut slot) = job.messages.lock() {
                *slot = None;
            }
            if let (Some(entry), Ok(mut log)) = (entry, job.log.lock()) {
                log.push(entry);
            }
            drop(agent);
            ping(&notify);
        });
    }

    fn next_id(&self) -> ArtId {
        self.jobs
            .lock()
            .map(|jobs| jobs.keys().next_back().map_or(1, |last| last + 1))
            .unwrap_or(1)
    }
}

/// Escribe el PNG y el `.lume` del encargo si el lienzo tiene algo (la
/// carpeta ya existe: la prepara quien hace el encargo). Devuelve cuántas
/// versiones tiene el documento y su revisión, o `None` si estaba vacío.
async fn finish(
    session: &SessionHandle,
    store: &dyn DocumentStore,
    spec: &ArtSpec,
) -> Result<Option<(usize, u64)>, PortError> {
    let document = session
        .snapshot()
        .await
        .map_err(|e| PortError::Unavailable(e.to_string()))?;
    if !document.canvas().pixels().iter().any(|p| p.a > 0) {
        return Ok(None);
    }
    store
        .export_png(document.canvas(), &spec.png, 1)
        .map_err(|e| PortError::Io(e.message))?;
    store
        .save_document(&document, &spec.lume)
        .map_err(|e| PortError::Io(e.message))?;
    let revision = session.view().borrow().revision;
    Ok(Some((document.versions().len(), revision)))
}

/// PNG del lienzo de una sesión, ampliado hacia `side` píxeles.
fn png_of(
    session: &SessionHandle,
    store: &dyn DocumentStore,
    side: u32,
) -> Result<Vec<u8>, PortError> {
    let canvas = session.view().borrow().canvas.clone();
    let size = (i64::from(canvas.width()), i64::from(canvas.height()));
    let scale = scale_for(size, i64::from(side));
    store
        .encode_png(&canvas, scale)
        .map_err(|e| PortError::Invalid(e.message))
}

fn unknown(id: ArtId) -> PortError {
    PortError::Invalid(format!("no hay ningún encargo con el número {id}"))
}

fn poisoned() -> PortError {
    PortError::Unavailable("el estado de los encargos se rompió".into())
}

impl ArtService for LocalArt {
    fn create(&self, spec: ArtSpec) -> BoxFuture<'_, Result<ArtStatus, PortError>> {
        Box::pin(self.start(spec))
    }

    fn status(&self, id: ArtId) -> BoxFuture<'_, Result<ArtStatus, PortError>> {
        Box::pin(async move { self.job_status(id) })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<ArtStatus>, PortError>> {
        Box::pin(async move {
            let ids: Vec<ArtId> = self
                .jobs
                .lock()
                .map_err(|_| poisoned())?
                .keys()
                .copied()
                .collect();
            ids.into_iter().map(|id| self.job_status(id)).collect()
        })
    }

    fn preview(&self, id: ArtId, side: u32) -> BoxFuture<'_, Result<Vec<u8>, PortError>> {
        Box::pin(async move {
            let session = self.session(id).ok_or_else(|| unknown(id))?;
            png_of(&session, self.store.as_ref(), side)
        })
    }

    fn draw(
        &self,
        script: String,
        cwd: Option<PathBuf>,
    ) -> BoxFuture<'_, Result<String, PortError>> {
        Box::pin(async move {
            let report = self
                .main
                .batch(script, Origin::Cli, cwd)
                .await
                .map_err(|e| PortError::Invalid(e.to_string()))?;
            Ok(report.output())
        })
    }

    fn canvas_png(&self, side: u32) -> BoxFuture<'_, Result<Vec<u8>, PortError>> {
        Box::pin(async move { png_of(&self.main, self.store.as_ref(), side) })
    }
}

/// La guía de arte de un proyecto, tal como está en disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuideFile {
    pub path: PathBuf,
    pub text: String,
    pub guide: ArtGuide,
}

impl GuideFile {
    /// Carpeta del proyecto (la de `lumeart.toml`).
    pub fn project(&self) -> &Path {
        self.path.parent().unwrap_or(Path::new("."))
    }
}

/// Un recurso que ya está en la carpeta de salida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetFile {
    pub name: String,
    pub png: Option<PathBuf>,
    pub lume: Option<PathBuf>,
    /// Versiones del `.lume`, si se pudo leer.
    pub versions: Option<usize>,
}

/// La guía y los recursos del proyecto del juego. Lo implementa `lume-store`.
pub trait ProjectStore: Send + Sync {
    /// Busca `lumeart.toml` desde `dir` hacia arriba. Un archivo que no vale
    /// es un error con todos sus problemas.
    fn find_guide(&self, dir: &Path) -> Result<Option<GuideFile>, PortError>;
    /// Los recursos (`.png` y `.lume`) de una carpeta, por nombre.
    fn list_assets(&self, dir: &Path) -> Result<Vec<AssetFile>, PortError>;
    /// Crea la carpeta de salida si no existe.
    fn prepare_output(&self, dir: &Path) -> Result<(), PortError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> ArtSpec {
        ArtSpec {
            name: "cofre_de_madera".into(),
            description: "Un cofre que el jugador abre.".into(),
            size: (16, 16),
            view: "frontal".into(),
            style: "pixel art de 16 bits".into(),
            palette: None,
            png: "a.png".into(),
            lume: "a.lume".into(),
            base: None,
            changes: String::new(),
        }
    }

    #[test]
    fn the_goal_says_what_to_draw() {
        let goal = spec().goal();
        assert!(
            goal.contains("Recurso: cofre_de_madera (16x16 píxeles, pixel art)."),
            "{goal}"
        );
        assert!(
            goal.contains("Qué es y para qué sirve en el juego: Un cofre que el jugador abre."),
            "{goal}"
        );
        assert!(
            goal.contains("Vista: frontal\nEstilo: pixel art de 16 bits\n"),
            "{goal}"
        );
        assert!(!goal.contains("Cambios"), "{goal}");
        let revision = ArtSpec {
            changes: "la tapa más clara".into(),
            view: String::new(),
            ..spec()
        };
        assert!(
            revision
                .goal()
                .contains("Cambios pedidos en esta revisión: la tapa más clara")
        );
        assert!(!revision.goal().contains("Vista"));
    }

    #[test]
    fn preview_scale() {
        assert_eq!(scale_for((16, 16), 256), 16);
        assert_eq!(scale_for((300, 20), 256), 1);
        assert_eq!(scale_for((1, 1), 256), 32);
    }

    #[test]
    fn guide_palettes() {
        let mut guide = ArtGuide::default();
        assert_eq!(PaletteSpec::of_guide(&guide), None);
        guide.palette = lume_domain::GuidePalette::Custom(vec![Rgba::rgb(1, 2, 3)]);
        let spec = PaletteSpec::of_guide(&guide).unwrap();
        assert_eq!(spec.name, "del proyecto");
        assert_eq!(spec.colors, ["#010203ff"]);
        assert_eq!(spec.palette().unwrap().len(), 1);
        guide.palette = lume_domain::GuidePalette::Fixed("game-boy".into());
        assert_eq!(PaletteSpec::of_guide(&guide).unwrap().name, "game-boy");
    }
}
