//! `ViewModel` y eventos que publica la `Session`.

use std::sync::Arc;

use lume_domain::{Canvas, Document, Origin, Palette, Patch};

/// Un paso del historial tal como lo muestra la interfaz
/// («batch · 22 comandos — agente»).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryItem {
    pub label: String,
    pub origin: Origin,
    pub commands: usize,
}

/// Una versión tal como la muestra la interfaz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionItem {
    pub number: u32,
    pub author: Origin,
    pub message: String,
}

/// Instantánea de solo lectura del documento. La ventana lee siempre la
/// última; cada cambio publica una nueva con `revision` mayor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewModel {
    /// Crece en 1 con cada cambio del documento.
    pub revision: u64,
    pub name: String,
    pub canvas: Arc<Canvas>,
    pub palette: Palette,
    /// Pasos que se pueden deshacer, del más antiguo al más reciente.
    pub history: Vec<HistoryItem>,
    pub can_undo: bool,
    pub can_redo: bool,
    pub versions: Vec<VersionItem>,
    /// Número de comandos de la receta.
    pub recipe_len: usize,
    /// El último cambio (para resaltarlo en el lienzo).
    pub last_change: Option<Change>,
    /// La animación del documento, si tiene.
    pub animation: Option<Arc<lume_domain::anim::animation::Animation>>,
    /// El dibujo vectorial del documento, si tiene.
    pub vector: Option<Arc<lume_domain::vec::drawing::VectorDrawing>>,
}

impl ViewModel {
    /// Instantánea de un documento (la `Session` la publica tras cada cambio).
    pub fn of(document: &Document, revision: u64, last_change: Option<Change>) -> Self {
        let history = document.history();
        Self {
            revision,
            name: document.name.clone(),
            canvas: Arc::new(document.canvas().clone()),
            palette: document.palette(),
            history: history
                .entries()
                .iter()
                .map(|e| HistoryItem {
                    label: e.label.clone(),
                    origin: e.origin,
                    commands: e.command_count(),
                })
                .collect(),
            can_undo: history.can_undo(),
            can_redo: history.can_redo(),
            versions: document
                .versions()
                .iter()
                .map(|v| VersionItem {
                    number: v.number,
                    author: v.author,
                    message: v.message.clone(),
                })
                .collect(),
            recipe_len: document.recipe().len(),
            last_change,
            animation: document.animation.clone().map(Arc::new),
            vector: document.vector.clone().map(Arc::new),
        }
    }
}

/// Un cambio del documento: quién, qué y qué celdas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub revision: u64,
    pub origin: Origin,
    /// Lo que se hizo, en el protocolo (`set_pixel 21 24 #5b2aa6ff`, `undo`,
    /// `batch`…).
    pub label: String,
    /// Celdas cambiadas (y cambio de tamaño, si lo hubo).
    pub patch: Arc<Patch>,
    /// Las líneas del protocolo que lo hicieron, en orden (las de un `batch`,
    /// o la única de un comando suelto). Vacío si el cambio no vino de
    /// líneas (deshacer en la ventana, abrir, volver a una versión…). La
    /// ventana las usa para verlo dibujar paso a paso y en la pestaña Código.
    pub lines: Arc<Vec<String>>,
}

/// Lo que publica la `Session` en su canal de eventos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    /// El documento cambió.
    Changed(Change),
    /// Se ejecutó una petición del protocolo (cambie o no el documento): la
    /// línea y su respuesta, como el registro del studio de pxcli.
    Command {
        origin: Origin,
        line: String,
        response: String,
    },
    /// La sesión terminó.
    Closed,
}
