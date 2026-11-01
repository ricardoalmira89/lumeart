//! Núcleo de dominio de LumeArt.
//!
//! Contiene los tipos puros del producto: `Rgba`, `Canvas`, `Palette`,
//! `Command`, `Patch`, `History`, `Recipe`, el protocolo de texto compatible
//! con pxcli, el modelo de animación y el encuadre de los dibujos vectoriales.
//!
//! Regla: este crate no depende de ningún otro crate de LumeArt ni hace E/S
//! (nada de archivos, red, hilos ni reloj). Ver `docs/03-arquitectura.md`.

pub mod anim;
pub mod batch;
pub mod canvas;
pub mod color;
pub mod command;
pub mod discovery;
pub mod document;
pub mod error;
pub mod eval;
pub mod gomath;
pub mod gosort;
pub mod guide;
pub mod history;
pub mod palette;
pub mod patch;
pub mod protocol;
pub mod recipe;
pub mod replay;
pub mod vec;

pub use batch::BatchReport;
pub use canvas::{Canvas, CellChange, MAX_CANVAS_SIZE, SizeChange};
pub use color::Rgba;
pub use command::Command;
pub use discovery::SessionInfo;
pub use document::{Document, Version};
pub use error::{DomainError, ErrorCode};
pub use guide::{ArtGuide, GUIDE_FILE, GuideError, GuidePalette, Naming};
pub use history::{History, HistoryEntry, Origin, RecipeSwap, RunId};
pub use palette::{Palette, PaletteColor};
pub use patch::{AnimChange, Patch, PatchStep};
pub use protocol::{Executed, Request, Response};
pub use recipe::Recipe;

/// Versión del formato y del protocolo que entiende este núcleo.
pub const PROTOCOL_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    #[test]
    fn protocol_version_is_set() {
        assert_eq!(super::PROTOCOL_VERSION, 1);
    }
}
