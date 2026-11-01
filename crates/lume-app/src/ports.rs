//! Puertos: los *traits* que implementan los adaptadores. La aplicación solo
//! conoce estos contratos, nunca los adaptadores concretos.
//!
//! | Puerto | Adaptador | Para qué |
//! | --- | --- | --- |
//! | [`DocumentStore`] | `lume-store` | `.lume`, PNG, importar pxcli |
//! | [`Rasterizer`] | `lume-raster` | SVG → imagen (modo vector) |
//! | [`ScriptEngine`] | `lume-script` | `anim.js` y `dibujo.js` (QuickJS) |
//! | [`ModelProvider`] | `lume-agent` | API de Anthropic, OpenAI, compatibles |
//! | [`AgentRunner`] | `lume-agent` | `claude -p` o bucle propio sobre un `ModelProvider` |
//! | [`SecretStore`] | `lume-store` | claves de API en el llavero del sistema |
//!
//! Los puertos asíncronos devuelven [`BoxFuture`] para poder usarse como
//! `dyn Trait`. Hay dobles en memoria para tests en [`crate::testing`].

use std::fmt;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use lume_domain::{Canvas, Document, DomainError, Rgba, RunId};
use tokio::sync::mpsc;

pub use tokio_util::sync::CancellationToken;

/// Futuro en caja, enviable entre hilos.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Error de un puerto (salvo `DocumentStore`, que usa los códigos del
/// protocolo). Los mensajes se muestran al usuario: van en español.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PortError {
    #[error("error de archivo: {0}")]
    Io(String),
    #[error("datos no válidos: {0}")]
    Invalid(String),
    #[error("se agotó el tiempo")]
    Timeout,
    #[error("cancelado")]
    Cancelled,
    #[error("la clave de API no es válida: {0}")]
    Auth(String),
    #[error("el proveedor limita las peticiones: {0}")]
    RateLimited(String),
    #[error("no disponible: {0}")]
    Unavailable(String),
    #[error("error del proveedor: {0}")]
    Provider(String),
}

// ---------------------------------------------------------------------------
// Documentos
// ---------------------------------------------------------------------------

/// Guardar y abrir documentos e imágenes. Lo implementa `lume-store`.
///
/// Los errores de archivo usan el código `io` del protocolo, con un mensaje
/// como los de pxcli (`open <ruta>: no such file or directory`).
pub trait DocumentStore: Send + Sync {
    /// Escribe el lienzo como PNG RGBA, ampliando cada píxel a un bloque de
    /// `scale` × `scale` (1–64, como `export --scale` de pxcli).
    fn export_png(&self, canvas: &Canvas, path: &Path, scale: u32) -> Result<(), DomainError>;

    /// Lee un PNG (cualquier tipo de color) como lienzo RGBA.
    fn read_png(&self, path: &Path) -> Result<Canvas, DomainError>;

    /// Los bytes del PNG que escribiría `export_png` (para servirlo por la
    /// API o enseñárselo al agente sin pasar por disco).
    fn encode_png(&self, canvas: &Canvas, scale: u32) -> Result<Vec<u8>, DomainError>;

    /// Guarda el documento en un archivo `.lume` (lienzo, receta, versiones,
    /// paleta y notas; el historial de deshacer no se guarda).
    fn save_document(&self, document: &Document, path: &Path) -> Result<(), DomainError>;

    /// Abre un archivo `.lume`, migrando formatos antiguos.
    fn open_document(&self, path: &Path) -> Result<Document, DomainError>;

    /// Importa una carpeta de una ejecución de pxcli (`final.png` +
    /// `recipe.txt`, y `meta.json` si está).
    fn import_pxcli_run(&self, dir: &Path) -> Result<Document, DomainError>;

    /// PNG de una imagen de animación (hojas, tiras), como lo escribe pxcli.
    fn encode_image(&self, img: &lume_domain::anim::Image) -> Result<Vec<u8>, DomainError>;

    /// GIF animado con estos fotogramas (del mismo tamaño) y sus duraciones.
    fn encode_gif(
        &self,
        frames: &[lume_domain::anim::Image],
        durations_ms: &[u32],
        looping: bool,
    ) -> Result<Vec<u8>, DomainError>;

    /// Escribe un archivo (al exportar una animación).
    fn write_file(&self, path: &Path, bytes: &[u8]) -> Result<(), DomainError>;
}

/// Un documento reciente para la pantalla de inicio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentEntry {
    pub path: std::path::PathBuf,
    /// Nombre del documento.
    pub name: String,
    /// Lienzo actual (para la miniatura y el tamaño).
    pub thumbnail: Canvas,
    /// Cuándo se abrió o guardó por última vez (segundos desde 1970).
    pub opened_at: u64,
}

/// Lista de documentos recientes. Lo implementa `lume-store`.
pub trait RecentStore: Send + Sync {
    /// Recientes que aún existen, del más nuevo al más viejo.
    fn list(&self) -> Vec<RecentEntry>;

    /// Apunta que se abrió o guardó `path` en el instante `opened_at`.
    fn touch(&self, path: &Path, opened_at: u64) -> Result<(), PortError>;
}

// ---------------------------------------------------------------------------
// Rasterizado (modo vector)
// ---------------------------------------------------------------------------

/// Opciones de rasterizado (las de `RasterOptions` en pxcli).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RasterOptions {
    /// Ancho de salida en píxeles (el alto sigue la proporción). `None`: 1024.
    pub width: Option<u32>,
    /// Recorte `x, y, ancho, alto` en unidades del dibujo; `None`: todo.
    pub crop: Option<[f64; 4]>,
    /// Paso de la rejilla con coordenadas, en unidades del dibujo.
    pub grid: Option<f64>,
    /// Color de fondo; `None` conserva la transparencia.
    pub background: Option<Rgba>,
    /// SVG en unidades del dibujo que se pinta encima (una selección…).
    pub overlay: Option<String>,
}

/// Imagen RGBA de 8 bits sin premultiplicar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterImage {
    pub width: u32,
    pub height: u32,
    /// 4 bytes por píxel, por filas.
    pub rgba: Vec<u8>,
}

/// Convierte SVG en imagen sin Chrome. Lo implementa `lume-raster` (resvg).
/// Guarda un dibujo vectorial (su SVG limpio) en un formato de edición: el
/// formato sale de la extensión (`.svg`, `.pdf`, `.ai`…). Devuelve avisos.
pub trait VectorExporter: Send + Sync {
    fn export(&self, svg: &str, path: &std::path::Path) -> Result<Vec<String>, PortError>;
}

pub trait Rasterizer: Send + Sync {
    /// Rasteriza `svg`, cuyo dibujo mide `width` × `height` unidades.
    fn rasterize(
        &self,
        svg: &str,
        width: f64,
        height: f64,
        options: &RasterOptions,
    ) -> Result<RasterImage, PortError>;
}

// ---------------------------------------------------------------------------
// Scripts (animación y vector)
// ---------------------------------------------------------------------------

/// Qué programa se carga (y con qué librería de apoyo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptKind {
    /// `anim.js`, con `prelude.js`.
    Animation,
    /// `dibujo.js`, con `lib.js`.
    Drawing,
}

/// Límites de ejecución de un script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptLimits {
    /// Tiempo máximo por llamada.
    pub time: Duration,
    /// Memoria máxima del motor.
    pub memory_bytes: usize,
}

impl Default for ScriptLimits {
    fn default() -> Self {
        Self {
            time: Duration::from_secs(5),
            memory_bytes: 64 * 1024 * 1024,
        }
    }
}

/// Un programa cargado: se le llama por funciones con argumentos JSON.
pub trait ScriptInstance: Send {
    /// Llama a `function` con `args_json` (un array JSON) y devuelve el
    /// resultado en JSON.
    fn call(&mut self, function: &str, args_json: &str) -> Result<String, PortError>;
}

/// Motor de JavaScript sin acceso a archivos ni red. Lo implementa
/// `lume-script` (QuickJS).
pub trait ScriptEngine: Send + Sync {
    fn load(
        &self,
        kind: ScriptKind,
        source: &str,
        limits: ScriptLimits,
    ) -> Result<Box<dyn ScriptInstance>, PortError>;
}

/// Carga la imagen base de una animación por su nombre (relativo a
/// `anim.js`), con los colores como los guarda pxcli
/// ([`Image::premultiplied`](lume_domain::anim::Image::premultiplied)).
pub type BaseLoader = Arc<dyn Fn(&str) -> Result<lume_domain::anim::Image, String> + Send + Sync>;

/// Motor de `anim.js`: lo ejecuta y dibuja el reposo y cada fotograma de
/// cada acción. Los fallos del script nunca son un error: vuelven como
/// avisos en el resultado. Lo implementa `lume-script` (QuickJS).
pub trait AnimEngine: Send + Sync {
    fn render(&self, source: &str, load_base: BaseLoader) -> lume_domain::anim::AnimResult;
}

/// Las formas de un dibujo vectorial que se ven en una zona o en un punto.
/// Trabaja sobre el SVG del studio (cada forma con `data-i`, `data-l` y
/// `data-c`) de un dibujo de `width` × `height` unidades. Lo implementa
/// `lume-raster`.
pub trait ShapeFinder: Send + Sync {
    /// Las formas que se ven dentro del polígono `zone`, de más a menos
    /// superficie.
    fn shapes_under(
        &self,
        studio_svg: &str,
        width: f64,
        height: f64,
        zone: &[[f64; 2]],
    ) -> Result<Vec<lume_domain::vec::studio::ShapeHit>, PortError>;

    /// La forma de encima en `point` (sin contar las que cubren casi todo
    /// el lienzo), si hay alguna.
    fn shape_at(
        &self,
        studio_svg: &str,
        width: f64,
        height: f64,
        point: [f64; 2],
    ) -> Result<Option<lume_domain::vec::studio::ShapeHit>, PortError>;
}

/// Una ayuda de `dibujo.js` fuera del motor: recibe el trabajo en JSON y
/// devuelve un fragmento SVG, o el mensaje de error. Así llegan el calco de
/// la referencia (`trace()`) y VectorCraft (`freeform`, `revolve`…).
pub type VecHelper = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

/// Motor de `dibujo.js`: lo ejecuta con la librería de dibujo y devuelve el
/// SVG. Los fallos del script nunca son un error: vuelven como avisos en el
/// resultado, con lo dibujado hasta el fallo. Lo implementa `lume-script`
/// (QuickJS).
pub trait VecEngine: Send + Sync {
    fn render(&self, source: &str) -> lume_domain::vec::VecResult;
    /// Como `render`, y además la escena partida en capas y piezas (si no
    /// hay errores), para exportarla a un juego.
    fn render_scene(&self, source: &str) -> lume_domain::vec::VecResult;
}

// ---------------------------------------------------------------------------
// Modelos de IA
// ---------------------------------------------------------------------------

/// Quién habla en un mensaje.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    User,
    Assistant,
}

/// Una parte del contenido de un mensaje (formato común a los proveedores).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentPart {
    Text(String),
    /// Imagen (PNG, JPEG…) en bytes.
    Image {
        media_type: String,
        data: Vec<u8>,
    },
    /// El modelo pide usar una herramienta.
    ToolUse {
        id: String,
        name: String,
        input_json: String,
    },
    /// Resultado de una herramienta.
    ToolResult {
        id: String,
        content: Vec<ContentPart>,
        is_error: bool,
    },
    /// Un bloque propio de un proveedor (el razonamiento de Claude, por
    /// ejemplo) que hay que devolverle sin cambios en la conversación. Los
    /// demás proveedores lo ignoran.
    Opaque {
        provider: String,
        json: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentPart>,
}

impl Message {
    pub fn user_text(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentPart::Text(text.into())],
        }
    }

    pub fn assistant_text(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentPart::Text(text.into())],
        }
    }
}

/// Descripción de una herramienta para el modelo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    /// Esquema JSON de la entrada.
    pub input_schema_json: String,
}

/// Petición a un modelo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRequest {
    pub model: String,
    pub system: String,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSpec>,
    pub max_tokens: u32,
}

/// Por qué terminó la respuesta del modelo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
    Other(String),
}

/// Tokens consumidos.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
}

impl Usage {
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    pub fn add(&mut self, other: Usage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cached_tokens += other.cached_tokens;
    }
}

/// Respuesta de un modelo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelResponse {
    pub content: Vec<ContentPart>,
    pub stop: StopReason,
    pub usage: Usage,
}

/// Un proveedor de modelos (patrón Strategy): Anthropic, OpenAI,
/// compatibles con OpenAI. Lo implementa `lume-agent`.
pub trait ModelProvider: Send + Sync {
    /// Identificador (`anthropic`, `openai`, `openai-compatible`…).
    fn id(&self) -> &str;

    fn complete<'a>(
        &'a self,
        request: ModelRequest,
    ) -> BoxFuture<'a, Result<ModelResponse, PortError>>;
}

// ---------------------------------------------------------------------------
// Agente
// ---------------------------------------------------------------------------

/// Resultado de una herramienta del agente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutput {
    pub content: Vec<ContentPart>,
    pub is_error: bool,
}

impl ToolOutput {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ContentPart::Text(text.into())],
            is_error: false,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            content: vec![ContentPart::Text(text.into())],
            is_error: true,
        }
    }
}

/// Herramientas que el agente puede usar (`draw`, `view_canvas`,
/// `get_pixel`, `undo`, `palette`…). La implementación real trabaja contra la
/// `Session`.
pub trait AgentTools: Send + Sync {
    fn specs(&self) -> Vec<ToolSpec>;

    fn call<'a>(&'a self, name: &'a str, input_json: &'a str) -> BoxFuture<'a, ToolOutput>;
}

/// Caja de herramientas compartida.
pub type ToolBox = Arc<dyn AgentTools>;

/// Etiqueta del adjunto con el lienzo al empezar una vuelta del bucle (los
/// runners lo distinguen de las imágenes de referencia).
pub const CANVAS_ATTACHMENT: &str = "Lienzo al empezar la vuelta";

/// Una imagen adjunta al encargo (referencia, lienzo actual…).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub label: String,
    pub media_type: String,
    pub data: Vec<u8>,
}

/// Límites de una ejecución del agente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentLimits {
    pub max_steps: u32,
    pub max_tokens: u64,
}

impl Default for AgentLimits {
    fn default() -> Self {
        Self {
            max_steps: 60,
            max_tokens: 400_000,
        }
    }
}

/// Encargo para una ejecución del agente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTask {
    pub run: RunId,
    /// Lo que pidió el usuario.
    pub goal: String,
    /// Estado heredado (notas, comprobaciones que fallaron, crítico…).
    pub context: String,
    pub attachments: Vec<Attachment>,
    pub model: String,
    pub limits: AgentLimits,
    /// Tamaño del lienzo (ancho, alto), para las instrucciones del agente.
    pub canvas: (u32, u32),
    /// Paleta fija del dibujo, si la hay.
    pub palette: Option<lume_domain::Palette>,
    /// El lienzo ya tiene dibujo y se sigue sobre él.
    pub continuing: bool,
    /// Conversación de una ejecución anterior que se retoma
    /// (`ContextPolicy::Resume`); `None`: contexto vacío.
    pub resume: Option<Conversation>,
    /// Trabajo de animación: el `anim.js` y la base de ahora (Claude Code
    /// los recibe como archivos; los runners por API usan las herramientas).
    pub anim: Option<AnimTask>,
    /// Trabajo vectorial: el `dibujo.js` de ahora (Claude Code lo recibe
    /// como archivo; los runners por API usan las herramientas).
    pub vector: Option<VecTask>,
}

/// Lo que necesita el agente para cambiar una animación.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimTask {
    /// `anim.js` actual.
    pub source: String,
    /// La base, en PNG.
    pub base_png: Vec<u8>,
}

/// Lo que necesita el agente para cambiar un dibujo vectorial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VecTask {
    /// `dibujo.js` actual.
    pub source: String,
}

/// La conversación de una ejecución, para retomarla en la siguiente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conversation {
    /// Los mensajes (runners por API).
    Messages(Vec<Message>),
    /// La sesión de Claude Code (`claude -p --resume <id>`).
    ClaudeSession(String),
}

/// Lo que va pasando durante una ejecución (para la interfaz y los registros).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    Text(String),
    /// Lo que va pensando el modelo (su razonamiento), si lo enseña.
    Thinking(String),
    ToolCall {
        name: String,
        input_json: String,
    },
    ToolResult {
        name: String,
        output: ToolOutput,
    },
    Usage(Usage),
}

/// Canal por el que el agente informa de lo que hace.
#[derive(Debug, Clone)]
pub struct EventSink(mpsc::UnboundedSender<AgentEvent>);

impl EventSink {
    pub fn channel() -> (EventSink, mpsc::UnboundedReceiver<AgentEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (EventSink(tx), rx)
    }

    /// Envía un evento (si nadie escucha, se pierde sin error).
    pub fn emit(&self, event: AgentEvent) {
        let _ = self.0.send(event);
    }
}

/// Por qué terminó una ejecución.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEnd {
    Finished,
    StepLimit,
    TokenLimit,
    Cancelled,
}

/// Resumen de una ejecución.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSummary {
    pub steps: u32,
    pub usage: Usage,
    /// Último texto del agente.
    pub final_text: String,
    pub end: RunEnd,
    /// La conversación, si se puede retomar (solo si terminó bien).
    pub conversation: Option<Conversation>,
    /// Vueltas del bucle propio del runner, si lo lleva (el plugin oficial
    /// de Claude Code en `lumeart eval`); `None` en una ejecución normal.
    pub loops: Option<u32>,
}

impl RunSummary {
    /// Cómo terminó, para la interfaz y los registros («Se paró al llegar
    /// al límite de 60 pasos»).
    pub fn status(&self, limits: AgentLimits) -> String {
        let steps = |n: u32| {
            if n == 1 {
                "1 paso".to_string()
            } else {
                format!("{n} pasos")
            }
        };
        match self.end {
            RunEnd::Finished => format!("Terminado en {}", steps(self.steps)),
            RunEnd::StepLimit => {
                format!("Se paró al llegar al límite de {}", steps(limits.max_steps))
            }
            RunEnd::TokenLimit => format!(
                "Se paró al llegar al límite de {} tokens (usó {})",
                limits.max_tokens,
                self.usage.total()
            ),
            RunEnd::Cancelled => format!("Cancelado tras {}", steps(self.steps)),
        }
    }
}

/// Ejecuta el agente: `claude -p` (`ClaudeCodeRunner`) o un bucle propio
/// sobre un `ModelProvider` (`ApiRunner`). Lo implementa `lume-agent`.
pub trait AgentRunner: Send + Sync {
    fn run<'a>(
        &'a self,
        task: AgentTask,
        tools: ToolBox,
        events: EventSink,
        cancel: CancellationToken,
    ) -> BoxFuture<'a, Result<RunSummary, PortError>>;
}

/// Un agente listo para una ejecución.
#[derive(Clone)]
pub struct PreparedAgent {
    pub runner: Arc<dyn AgentRunner>,
    pub tools: ToolBox,
    /// Modelo que usará (va en el `AgentTask`).
    pub model: String,
}

impl fmt::Debug for PreparedAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedAgent")
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

/// Prepara el agente del proveedor elegido en los ajustes (Claude Code,
/// Anthropic, OpenAI…) con sus herramientas sobre la sesión. Lo monta
/// `lumeart`, que es quien conoce los adaptadores; la interfaz solo pide.
pub trait AgentBackend: Send + Sync {
    fn prepare(
        &self,
        settings: &crate::settings::Settings,
        run: RunId,
    ) -> Result<PreparedAgent, PortError>;

    /// El crítico del mismo proveedor: sus instrucciones y herramientas para
    /// mirar el dibujo (quien lo lanza las deja en solo lectura).
    fn prepare_critic(
        &self,
        settings: &crate::settings::Settings,
        run: RunId,
    ) -> Result<PreparedAgent, PortError>;
}

// ---------------------------------------------------------------------------
// Secretos
// ---------------------------------------------------------------------------

/// Un secreto (clave de API). No se muestra en `Debug` ni tiene `Display`,
/// para que no acabe en logs por descuido.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// El valor, solo para enviarlo al proveedor.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Para mostrarla en Ajustes: el prefijo público del proveedor
    /// (`sk-ant-`, `sk-proj-`…, hasta el último guion de los 8 primeros
    /// caracteres) y puntos. Nunca muestra la parte secreta.
    pub fn masked(&self) -> String {
        let head: String = self.0.chars().take(8).collect();
        let prefix = match head.rfind('-') {
            Some(i) if self.0.chars().count() > 16 => &head[..=i],
            _ => "",
        };
        format!("{prefix}{}", "•".repeat(16))
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(****)")
    }
}

/// Claves de API. Lo implementa `lume-store` sobre el llavero del sistema.
/// Es el único sitio por el que pasan: nunca en archivos, logs ni tests.
pub trait SecretStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<Secret>, PortError>;
    fn set(&self, key: &str, value: &Secret) -> Result<(), PortError>;
    fn delete(&self, key: &str) -> Result<(), PortError>;
}
