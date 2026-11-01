//! Capa de aplicación de LumeArt.
//!
//! Aquí viven la `Session` (una por documento abierto, patrón actor: es la
//! única que escribe en el documento), los casos de uso, el bus de eventos, el
//! `ViewModel` y los *traits* de los puertos que implementan los adaptadores:
//! `DocumentStore`, `Rasterizer`, `ScriptEngine`, `AgentRunner`,
//! `ModelProvider`, `SecretStore` y `SettingsStore`.
//!
//! Regla: solo depende de `lume-domain`.

pub mod art;
pub mod critic;
pub mod eval;
pub mod ports;
pub mod refine;
pub mod session;
pub mod settings;
pub mod studio;
pub mod testing;
pub mod variants;
pub mod view;

pub use art::{
    ArtAgent, ArtAgents, ArtId, ArtNotify, ArtService, ArtSpec, ArtState, ArtStatus, AssetFile,
    GuideFile, JobEntry, LocalArt, PaletteSpec, ProjectStore,
};
pub use eval::{EvalAgents, EvalOutcome, EvalRunner};
pub use lume_domain as domain;
pub use ports::{
    AgentBackend, AgentRunner, AnimEngine, BaseLoader, DocumentStore, ModelProvider, PortError,
    Rasterizer, RecentEntry, RecentStore, ScriptEngine, SecretStore, VecEngine, VecHelper,
};
pub use refine::{
    CheckResult, CompletionPolicy, Delivery, LoopEnd, LoopEvent, LoopMessages, LoopSink,
    LoopSummary, RefineLoop, TurnReport,
};
pub use session::{AppError, LineResult, Session, SessionHandle};
pub use settings::{ContextPolicy, DrawSpeed, ProviderKind, Settings, SettingsStore};
pub use variants::{Variant, VariantState, Variants, anim_goal};
pub use view::{Change, HistoryItem, SessionEvent, VersionItem, ViewModel};
