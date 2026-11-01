//! `Intent`: lo que la interfaz quiere que pase. La UI nunca toca el
//! documento: emite intents y el `StudioApp` los envía a la `Session`.

use lume_app::ports::{Attachment, Secret};
use lume_app::{ProviderKind, Settings};
use lume_domain::Command;

/// Formatos de exportación del dibujo vectorial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VecFormat {
    Png,
    Svg,
    Pdf,
    Ai,
}

impl VecFormat {
    pub const ALL: [VecFormat; 4] = [
        VecFormat::Png,
        VecFormat::Svg,
        VecFormat::Pdf,
        VecFormat::Ai,
    ];

    pub fn label(self) -> &'static str {
        match self {
            VecFormat::Png => "PNG",
            VecFormat::Svg => "SVG",
            VecFormat::Pdf => "PDF",
            VecFormat::Ai => "Illustrator (.ai)",
        }
    }

    pub fn ext(self) -> &'static str {
        match self {
            VecFormat::Png => "png",
            VecFormat::Svg => "svg",
            VecFormat::Pdf => "pdf",
            VecFormat::Ai => "ai",
        }
    }

    /// Qué es, para el diálogo de guardar.
    pub fn kind(self) -> &'static str {
        match self {
            VecFormat::Png => "Imagen PNG",
            VecFormat::Svg => "Dibujo SVG",
            VecFormat::Pdf => "Documento PDF",
            VecFormat::Ai => "Adobe Illustrator",
        }
    }
}

/// Modo del studio (selector de la cabecera).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    #[default]
    Pixel,
    Vector,
    Animation,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Pixel, Mode::Vector, Mode::Animation];

    pub fn label(self) -> &'static str {
        match self {
            Mode::Pixel => "Pixel art",
            Mode::Vector => "Vector",
            Mode::Animation => "Animación",
        }
    }

    /// ¿Está disponible? (Todos lo están.)
    pub fn available(self) -> bool {
        true
    }
}

/// Lo que pide la interfaz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// Exportar una acción del esqueleto vectorial: hoja de sprites (PNG en
    /// fila) o GIF, a `fps` fotogramas por segundo y `width` píxeles de ancho.
    ExportRigAction {
        action: String,
        gif: bool,
        fps: u32,
        width: u32,
    },
    /// Exportar el dibujo vectorial (PNG al ancho dado, SVG, PDF o .ai).
    ExportVector {
        format: VecFormat,
        width: u32,
    },
    /// Aplicar un comando al documento como un paso de historial con esta
    /// etiqueta (un trazo del lápiz: «Lápiz · 3 píxeles»).
    Apply {
        command: Command,
        label: String,
    },
    Undo,
    Redo,
    /// Deshacer varios pasos seguidos (volver a una entrada del historial).
    UndoSteps(usize),
    /// Cambiar de modo.
    SwitchMode(Mode),
    /// Exportar (abre el diálogo de exportación).
    Export,
    /// Empezar un lienzo en blanco de este tamaño, con una paleta fija
    /// (`pico-8`, `game-boy`) o libre.
    NewDocument {
        width: u32,
        height: u32,
        palette: Option<String>,
    },
    /// Abrir un documento reciente.
    OpenRecent(std::path::PathBuf),
    /// Elegir un archivo para abrir (diálogo).
    OpenFile,
    /// Abrir este `.lume`.
    OpenPath(std::path::PathBuf),
    /// Guardar (si aún no tiene archivo, como «Guardar como»).
    Save,
    /// Guardar con otro nombre (diálogo).
    SaveAs,
    /// Guardar en este archivo.
    SaveTo(std::path::PathBuf),
    /// Exportar el lienzo a PNG con esta escala (diálogo para la ruta).
    ExportPng {
        scale: u32,
    },
    /// Exportar el lienzo a este PNG con esta escala.
    ExportPngTo {
        path: std::path::PathBuf,
        scale: u32,
    },
    /// Importar una carpeta de pxcli (diálogo).
    ImportPxcli,
    /// Importar esta carpeta de pxcli.
    ImportPxcliFrom(std::path::PathBuf),
    /// Volver a la pantalla de inicio.
    GoHome,
    /// Encargar un dibujo al agente (fase 5).
    Describe {
        prompt: String,
    },
    /// Abrir Ajustes.
    OpenSettings,
    /// Salir de Ajustes a la pantalla de antes.
    CloseSettings,
    /// Guardar estos ajustes en lugar de los actuales.
    UpdateSettings(Settings),
    /// Encender o apagar el resalte del último comando (menú Ver).
    ShowCommands(bool),
    /// Velocidad a la que se ve dibujar al agente (se guarda).
    DrawSpeed(lume_app::DrawSpeed),
    /// Guardar la clave de API de un proveedor en el llavero.
    SaveApiKey {
        provider: ProviderKind,
        key: Secret,
    },
    /// Borrar del llavero la clave de un proveedor.
    DeleteApiKey(ProviderKind),
    /// Pedir algo al agente (empieza una ejecución).
    AgentSend {
        prompt: String,
        attachments: Vec<Attachment>,
        /// Seguir sobre el dibujo actual en vez de empezar de cero.
        continuing: bool,
    },
    /// Detener la ejecución del agente.
    AgentStop,
    /// Elegir una imagen de referencia para el próximo envío (diálogo).
    AgentAttach,
    /// Cambiar el nombre del documento activo (F2 en su pestaña).
    RenameDocument(String),
    /// Enseñar la pestaña de documento `n`.
    SelectTab(usize),
    /// Cerrar la pestaña de documento `n` (pregunta si tiene cambios).
    CloseTab(usize),
    /// Una imagen pegada del portapapeles (Ctrl+V en la redacción), como
    /// referencia para el próximo envío.
    AgentPasteImage(Attachment),
    /// Ver una versión en el lienzo (sin restaurarla).
    ShowVersion(u32),
    /// Volver a una versión (un paso de deshacer).
    RestoreVersion(u32),
    /// Dejar de ver la versión y volver al dibujo actual.
    ClosePreview,
    /// Abrir en la ventana un encargo (su lienzo y su panel del agente).
    OpenJob(lume_app::ArtId),
    /// Volver del encargo al dibujo propio de la ventana.
    OpenMain,
    /// Crear la animación del dibujo (su receta como pieza, un reposo y una
    /// acción).
    CreateAnimation,
    /// Crear el dibujo vectorial del documento (un `dibujo.js` vacío).
    CreateVector,
    /// Añadir una línea al final de `dibujo.js` (lo que se dibujó con una
    /// herramienta del modo vector; se puede deshacer).
    AddVectorCode(String),
    /// Crear el dibujo vectorial en blanco con este tamaño (y fondo
    /// transparente o de color).
    NewVector {
        width: u32,
        height: u32,
        transparent: bool,
    },
    /// Se terminó de rodear una zona con el lazo (los puntos están en el
    /// modo vector).
    VectorZone,
    /// Clic con la herramienta de selección (el punto está en el modo
    /// vector): la forma de debajo y su línea.
    VectorSelect,
    /// Pedir al agente, en una sola pasada, todas las correcciones de la
    /// cola que esperan (cada una con su zona).
    RunCorrections,
    /// Volver a dibujar el dibujo desde cero, forma a forma.
    ReplayVector,
    /// Cambiar lo que dura un fotograma de una acción (se puede deshacer).
    SetFrameDuration {
        action: lume_domain::anim::ActionSpec,
        frame: u32,
        ms: u32,
    },
    /// Dejar un texto en la redacción del agente (por ejemplo, «Pedir
    /// arreglo al agente»).
    AgentDraft(String),
    /// Pedir variantes de la animación (tres, de la acción que se ve).
    RequestVariants {
        prompt: String,
    },
    /// Elegir una variante: pasa a ser la animación (una versión nueva).
    ChooseVariant(usize),
    /// Una variante más que mezcla las listas según el texto.
    MixVariants(String),
    /// Ver las variantes (`true`) o volver al editor (`false`).
    ShowVariants(bool),
    /// Exportar la animación (diálogo «Exportar animación»).
    ExportAnimation {
        format: lume_domain::anim::export::ExportFormat,
        scale: u32,
        file_name: String,
        open_folder: bool,
    },
}
