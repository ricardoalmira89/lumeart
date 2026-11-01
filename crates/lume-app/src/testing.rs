//! Dobles en memoria de los puertos, para tests (de esta capa y de los
//! adaptadores). No tocan disco, red ni procesos.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use lume_domain::{Canvas, Document, DomainError, ErrorCode, RunId};

use crate::ports::{
    AgentEvent, AgentRunner, AgentTask, AgentTools, BoxFuture, CancellationToken, ContentPart,
    DocumentStore, EventSink, ModelProvider, ModelRequest, ModelResponse, PortError, RasterImage,
    RasterOptions, Rasterizer, RecentEntry, RecentStore, RunEnd, RunSummary, ScriptEngine,
    ScriptInstance, ScriptKind, ScriptLimits, Secret, SecretStore, ToolBox, ToolOutput, ToolSpec,
    Usage,
};
use crate::settings::{Settings, SettingsStore};

fn not_found(path: &Path) -> DomainError {
    DomainError::new(
        ErrorCode::Io,
        format!("open {}: no such file or directory", path.display()),
    )
}

/// Lo que guarda un GIF falso de [`MemoryStore`]: fotogramas, tamaño,
/// duraciones y si va en bucle.
pub type MemGif = (usize, (u32, u32), Vec<u32>, bool);

/// Una exportación hecha en un [`MemoryStore`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    pub path: PathBuf,
    pub scale: u32,
    pub canvas: Canvas,
}

/// `DocumentStore` en memoria. Las rutas dentro de una carpeta marcada con
/// [`MemoryStore::missing_dir`] fallan como si no existiera.
#[derive(Debug, Default)]
pub struct MemoryStore {
    exports: Mutex<Vec<Export>>,
    files: Mutex<Vec<(PathBuf, Vec<u8>)>>,
    images: Mutex<HashMap<PathBuf, Canvas>>,
    documents: Mutex<HashMap<PathBuf, Document>>,
    runs: Mutex<HashMap<PathBuf, Document>>,
    missing: Mutex<Vec<PathBuf>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Las escrituras y lecturas dentro de `dir` fallan con `io`.
    pub fn missing_dir(&self, dir: impl Into<PathBuf>) {
        self.missing.lock().unwrap().push(dir.into());
    }

    /// Deja una imagen lista para `read_png`.
    pub fn put_image(&self, path: impl Into<PathBuf>, canvas: Canvas) {
        self.images.lock().unwrap().insert(path.into(), canvas);
    }

    /// Deja una ejecución de pxcli lista para `import_pxcli_run`.
    pub fn put_pxcli_run(&self, dir: impl Into<PathBuf>, document: Document) {
        self.runs.lock().unwrap().insert(dir.into(), document);
    }

    /// Exportaciones hechas, en orden.
    pub fn exports(&self) -> Vec<Export> {
        self.exports.lock().unwrap().clone()
    }

    /// Archivos escritos con `write_file`, en orden.
    pub fn files(&self) -> Vec<(PathBuf, Vec<u8>)> {
        self.files.lock().unwrap().clone()
    }

    /// Ancho, alto y bytes RGBA de lo que codificó `encode_image`.
    pub fn decode_image(bytes: &[u8]) -> Option<(u32, u32, &[u8])> {
        let rest = bytes.strip_prefix(b"MEMIMG")?;
        let w = u32::from_be_bytes(rest.get(0..4)?.try_into().ok()?);
        let h = u32::from_be_bytes(rest.get(4..8)?.try_into().ok()?);
        Some((w, h, rest.get(8..)?))
    }

    /// Fotogramas, tamaño, duraciones y bucle de lo que codificó `encode_gif`.
    pub fn decode_gif(bytes: &[u8]) -> Option<MemGif> {
        let text = std::str::from_utf8(bytes.strip_prefix(b"MEMGIF ")?).ok()?;
        let mut parts = text.split(' ');
        let frames = parts.next()?.parse().ok()?;
        let (w, h) = parts.next()?.split_once('x')?;
        let size = (w.parse().ok()?, h.parse().ok()?);
        let durations = parts
            .next()?
            .split(',')
            .filter(|d| !d.is_empty())
            .map(|d| d.parse().ok())
            .collect::<Option<Vec<u32>>>()?;
        let looping = parts.next()? == "bucle";
        Some((frames, size, durations, looping))
    }

    fn check(&self, path: &Path) -> Result<(), DomainError> {
        if self
            .missing
            .lock()
            .unwrap()
            .iter()
            .any(|d| path.starts_with(d))
        {
            return Err(not_found(path));
        }
        Ok(())
    }
}

impl DocumentStore for MemoryStore {
    /// `MEMIMG`, ancho y alto (u32 big-endian) y los bytes RGBA tal cual.
    fn encode_image(&self, img: &lume_domain::anim::Image) -> Result<Vec<u8>, DomainError> {
        let mut out = b"MEMIMG".to_vec();
        out.extend_from_slice(&img.width().to_be_bytes());
        out.extend_from_slice(&img.height().to_be_bytes());
        for c in img.pixels() {
            out.extend_from_slice(&[c.r, c.g, c.b, c.a]);
        }
        Ok(out)
    }

    /// `MEMGIF <fotogramas> <ancho>x<alto> <duraciones> <bucle|una-vez>`.
    fn encode_gif(
        &self,
        frames: &[lume_domain::anim::Image],
        durations_ms: &[u32],
        looping: bool,
    ) -> Result<Vec<u8>, DomainError> {
        let size = frames.first().map_or((0, 0), |f| f.size());
        let durations: Vec<String> = durations_ms.iter().map(u32::to_string).collect();
        Ok(format!(
            "MEMGIF {} {}x{} {} {}",
            frames.len(),
            size.0,
            size.1,
            durations.join(","),
            if looping { "bucle" } else { "una-vez" }
        )
        .into_bytes())
    }

    fn write_file(&self, path: &Path, bytes: &[u8]) -> Result<(), DomainError> {
        self.check(path)?;
        self.files
            .lock()
            .unwrap()
            .push((path.to_path_buf(), bytes.to_vec()));
        Ok(())
    }

    fn export_png(&self, canvas: &Canvas, path: &Path, scale: u32) -> Result<(), DomainError> {
        if !(1..=64).contains(&scale) {
            return Err(DomainError::invalid_args("scale must be between 1 and 64"));
        }
        self.check(path)?;
        self.exports.lock().unwrap().push(Export {
            path: path.to_path_buf(),
            scale,
            canvas: canvas.clone(),
        });
        let (w, h, bytes) = canvas.scaled_rgba_bytes(scale);
        if let Ok(scaled) = Canvas::from_rgba_bytes(i64::from(w), i64::from(h), &bytes) {
            self.images
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), scaled);
        }
        Ok(())
    }

    /// No es un PNG de verdad: `MEMPNG`, ancho y alto (u32 big-endian) y los
    /// bytes RGBA ya ampliados. Basta para comparar en tests.
    fn encode_png(&self, canvas: &Canvas, scale: u32) -> Result<Vec<u8>, DomainError> {
        if !(1..=64).contains(&scale) {
            return Err(DomainError::invalid_args("scale must be between 1 and 64"));
        }
        let (w, h, bytes) = canvas.scaled_rgba_bytes(scale);
        let mut out = b"MEMPNG".to_vec();
        out.extend_from_slice(&w.to_be_bytes());
        out.extend_from_slice(&h.to_be_bytes());
        out.extend_from_slice(&bytes);
        Ok(out)
    }

    fn read_png(&self, path: &Path) -> Result<Canvas, DomainError> {
        self.check(path)?;
        self.images
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| not_found(path))
    }

    fn save_document(&self, document: &Document, path: &Path) -> Result<(), DomainError> {
        self.check(path)?;
        self.documents
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), document.clone());
        Ok(())
    }

    fn open_document(&self, path: &Path) -> Result<Document, DomainError> {
        self.check(path)?;
        self.documents
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| not_found(path))
    }

    fn import_pxcli_run(&self, dir: &Path) -> Result<Document, DomainError> {
        self.runs
            .lock()
            .unwrap()
            .get(dir)
            .cloned()
            .ok_or_else(|| not_found(&dir.join("final.png")))
    }
}

/// `Rasterizer` falso: devuelve una imagen transparente del tamaño pedido y
/// apunta los SVG recibidos.
#[derive(Debug, Default)]
pub struct FakeRasterizer {
    pub calls: Mutex<Vec<String>>,
}

impl Rasterizer for FakeRasterizer {
    fn rasterize(
        &self,
        svg: &str,
        width: f64,
        height: f64,
        options: &RasterOptions,
    ) -> Result<RasterImage, PortError> {
        if !svg.trim_start().starts_with("<svg") {
            return Err(PortError::Invalid("no es un SVG".into()));
        }
        self.calls.lock().unwrap().push(svg.to_string());
        let out_w = options.width.unwrap_or(1024).max(1);
        let out_h = ((f64::from(out_w) * height / width.max(1.0)).round() as u32).max(1);
        Ok(RasterImage {
            width: out_w,
            height: out_h,
            rgba: vec![0; out_w as usize * out_h as usize * 4],
        })
    }
}

/// `ScriptEngine` falso: cada función devuelve el JSON que se le haya dado
/// con [`FakeScriptEngine::respond`]; una función sin respuesta da error.
#[derive(Debug, Default)]
pub struct FakeScriptEngine {
    responses: Mutex<HashMap<String, String>>,
    pub loaded: Mutex<Vec<(ScriptKind, String)>>,
}

impl FakeScriptEngine {
    pub fn respond(&self, function: &str, json: &str) {
        self.responses
            .lock()
            .unwrap()
            .insert(function.to_string(), json.to_string());
    }
}

struct FakeInstance {
    responses: HashMap<String, String>,
}

impl ScriptInstance for FakeInstance {
    fn call(&mut self, function: &str, _args_json: &str) -> Result<String, PortError> {
        self.responses
            .get(function)
            .cloned()
            .ok_or_else(|| PortError::Invalid(format!("{function} is not defined")))
    }
}

impl ScriptEngine for FakeScriptEngine {
    fn load(
        &self,
        kind: ScriptKind,
        source: &str,
        _limits: ScriptLimits,
    ) -> Result<Box<dyn ScriptInstance>, PortError> {
        self.loaded.lock().unwrap().push((kind, source.to_string()));
        Ok(Box::new(FakeInstance {
            responses: self.responses.lock().unwrap().clone(),
        }))
    }
}

/// `ModelProvider` que reproduce respuestas grabadas, en orden, y apunta las
/// peticiones. Sin red ni coste.
#[derive(Debug, Default)]
pub struct FakeProvider {
    responses: Mutex<VecDeque<Result<ModelResponse, PortError>>>,
    pub requests: Mutex<Vec<ModelRequest>>,
}

impl FakeProvider {
    pub fn new(responses: impl IntoIterator<Item = ModelResponse>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().map(Ok).collect()),
            requests: Mutex::default(),
        }
    }

    /// Añade un error a la cola (por ejemplo, `RateLimited`).
    pub fn push_error(&self, error: PortError) {
        self.responses.lock().unwrap().push_back(Err(error));
    }
}

impl ModelProvider for FakeProvider {
    fn id(&self) -> &str {
        "fake"
    }

    fn complete<'a>(
        &'a self,
        request: ModelRequest,
    ) -> BoxFuture<'a, Result<ModelResponse, PortError>> {
        self.requests.lock().unwrap().push(request);
        let next = self.responses.lock().unwrap().pop_front();
        Box::pin(async move {
            next.unwrap_or_else(|| {
                Err(PortError::Provider(
                    "no quedan respuestas grabadas".to_string(),
                ))
            })
        })
    }
}

/// Un paso de un [`ScriptedRunner`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptedStep {
    /// El agente escribe un texto.
    Say(String),
    /// El agente usa una herramienta.
    Tool { name: String, input_json: String },
}

/// `AgentRunner` que sigue un guion fijo de textos y herramientas. Respeta la
/// cancelación y el límite de pasos.
#[derive(Debug, Default)]
pub struct ScriptedRunner {
    pub steps: Vec<ScriptedStep>,
}

impl ScriptedRunner {
    pub fn new(steps: Vec<ScriptedStep>) -> Self {
        Self { steps }
    }
}

impl AgentRunner for ScriptedRunner {
    fn run<'a>(
        &'a self,
        task: AgentTask,
        tools: ToolBox,
        events: EventSink,
        cancel: CancellationToken,
    ) -> BoxFuture<'a, Result<RunSummary, PortError>> {
        Box::pin(async move {
            let mut summary = RunSummary {
                steps: 0,
                usage: Usage::default(),
                final_text: String::new(),
                end: RunEnd::Finished,
                conversation: None,
                loops: None,
            };
            for step in &self.steps {
                if cancel.is_cancelled() {
                    summary.end = RunEnd::Cancelled;
                    return Ok(summary);
                }
                if summary.steps >= task.limits.max_steps {
                    summary.end = RunEnd::StepLimit;
                    return Ok(summary);
                }
                summary.steps += 1;
                match step {
                    ScriptedStep::Say(text) => {
                        events.emit(AgentEvent::Text(text.clone()));
                        summary.final_text = text.clone();
                    }
                    ScriptedStep::Tool { name, input_json } => {
                        events.emit(AgentEvent::ToolCall {
                            name: name.clone(),
                            input_json: input_json.clone(),
                        });
                        let output = tools.call(name, input_json).await;
                        events.emit(AgentEvent::ToolResult {
                            name: name.clone(),
                            output,
                        });
                    }
                }
            }
            Ok(summary)
        })
    }
}

/// Herramientas falsas: apuntan las llamadas y devuelven `ok`.
#[derive(Debug, Default)]
pub struct FakeTools {
    pub calls: Mutex<Vec<(String, String)>>,
}

impl AgentTools for FakeTools {
    fn specs(&self) -> Vec<ToolSpec> {
        vec![ToolSpec {
            name: "draw".into(),
            description: "Dibuja con comandos del protocolo".into(),
            input_schema_json: r#"{"type":"object","properties":{"commands":{"type":"string"}}}"#
                .into(),
        }]
    }

    fn call<'a>(&'a self, name: &'a str, input_json: &'a str) -> BoxFuture<'a, ToolOutput> {
        self.calls
            .lock()
            .unwrap()
            .push((name.to_string(), input_json.to_string()));
        Box::pin(async move {
            ToolOutput {
                content: vec![ContentPart::Text("ok".into())],
                is_error: false,
            }
        })
    }
}

/// `SecretStore` en memoria.
#[derive(Debug, Default)]
pub struct MemorySecretStore {
    secrets: Mutex<HashMap<String, Secret>>,
}

impl SecretStore for MemorySecretStore {
    fn get(&self, key: &str) -> Result<Option<Secret>, PortError> {
        Ok(self.secrets.lock().unwrap().get(key).cloned())
    }

    fn set(&self, key: &str, value: &Secret) -> Result<(), PortError> {
        self.secrets
            .lock()
            .unwrap()
            .insert(key.to_string(), value.clone());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), PortError> {
        self.secrets.lock().unwrap().remove(key);
        Ok(())
    }
}

/// `AgentBackend` de pruebas: da siempre el mismo runner y herramientas y
/// apunta con qué proveedor y modelo se le pidió cada ejecución.
pub struct FakeBackend {
    pub runner: std::sync::Arc<dyn AgentRunner>,
    pub tools: ToolBox,
    pub prepared: Mutex<Vec<(crate::settings::ProviderKind, String, RunId)>>,
    /// Ejecuciones para las que se pidió el crítico.
    pub critics: Mutex<Vec<RunId>>,
    /// Si se da, `prepare` falla con este error.
    pub error: Mutex<Option<PortError>>,
}

impl FakeBackend {
    pub fn new(runner: std::sync::Arc<dyn AgentRunner>, tools: ToolBox) -> Self {
        Self {
            runner,
            tools,
            prepared: Mutex::default(),
            critics: Mutex::default(),
            error: Mutex::default(),
        }
    }
}

impl crate::ports::AgentBackend for FakeBackend {
    fn prepare(
        &self,
        settings: &crate::settings::Settings,
        run: RunId,
    ) -> Result<crate::ports::PreparedAgent, PortError> {
        if let Some(err) = self.error.lock().unwrap().clone() {
            return Err(err);
        }
        let model = match settings.provider {
            crate::settings::ProviderKind::ClaudeCode => settings.claude_code_model.clone(),
            crate::settings::ProviderKind::Anthropic => settings.anthropic_model.clone(),
            crate::settings::ProviderKind::OpenAi => settings.openai_model.clone(),
            crate::settings::ProviderKind::Local => String::new(),
        };
        self.prepared
            .lock()
            .unwrap()
            .push((settings.provider, model.clone(), run));
        Ok(crate::ports::PreparedAgent {
            runner: self.runner.clone(),
            tools: self.tools.clone(),
            model,
        })
    }

    fn prepare_critic(
        &self,
        settings: &crate::settings::Settings,
        run: RunId,
    ) -> Result<crate::ports::PreparedAgent, PortError> {
        self.critics.lock().unwrap().push(run);
        let mut prepared = self.prepare(settings, run)?;
        self.prepared.lock().unwrap().pop();
        prepared.model = format!("{} (crítico)", prepared.model);
        Ok(prepared)
    }
}

/// `SettingsStore` en memoria: empieza con los ajustes que se le den (o los
/// de fábrica) y cuenta cuántas veces se guardó.
#[derive(Debug, Default)]
pub struct MemorySettingsStore {
    pub settings: Mutex<Settings>,
    pub saves: Mutex<usize>,
}

impl MemorySettingsStore {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: Mutex::new(settings),
            saves: Mutex::new(0),
        }
    }
}

impl SettingsStore for MemorySettingsStore {
    fn load(&self) -> Result<Settings, PortError> {
        Ok(self.settings.lock().unwrap().clone())
    }

    fn save(&self, settings: &Settings) -> Result<(), PortError> {
        *self.settings.lock().unwrap() = settings.clone();
        *self.saves.lock().unwrap() += 1;
        Ok(())
    }
}

/// `RecentStore` en memoria: devuelve las entradas que se le den y apunta
/// los `touch`.
#[derive(Debug, Default)]
pub struct MemoryRecents {
    pub entries: Mutex<Vec<RecentEntry>>,
    pub touched: Mutex<Vec<(PathBuf, u64)>>,
}

impl RecentStore for MemoryRecents {
    fn list(&self) -> Vec<RecentEntry> {
        self.entries.lock().unwrap().clone()
    }

    fn touch(&self, path: &Path, opened_at: u64) -> Result<(), PortError> {
        self.touched
            .lock()
            .unwrap()
            .push((path.to_path_buf(), opened_at));
        Ok(())
    }
}
