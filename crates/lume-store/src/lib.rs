//! Adaptador de almacenamiento de LumeArt.
//!
//! Implementa `DocumentStore` (formato `.lume`: ZIP con `manifest.json`,
//! `canvas.png`, `recipe.txt`, `anim.js`, `versions/` y `agent/`), la
//! codificación y lectura de PNG, la lista de recientes, `config.toml` y el
//! puerto `SecretStore` sobre el llavero del sistema (`keyring`).
//!
//! Regla: depende de `lume-app` y `lume-domain`, nunca de otro adaptador.

pub mod compat;
pub mod config;
pub mod gif_io;
pub mod guide;
pub mod lume_file;
pub mod png_io;
pub mod recents;
pub mod secrets;

use std::path::Path;

use lume_app::DocumentStore;
use lume_domain::{Canvas, Document, DomainError, ErrorCode};

pub use config::FsSettings;
pub use guide::FsProject;
pub use lume_file::{decode_lume, encode_lume};
pub use png_io::{
    decode_image, decode_png, encode_anim_png, encode_png, encode_rgba_png, read_anim_image,
    read_image, read_text, write_anim_png, write_bytes, write_rgba_png,
};
pub use recents::FsRecents;
pub use secrets::KeyringSecrets;

/// `DocumentStore` sobre el sistema de archivos.
#[derive(Debug, Clone, Copy, Default)]
pub struct FsStore;

impl DocumentStore for FsStore {
    fn encode_image(&self, img: &lume_domain::anim::Image) -> Result<Vec<u8>, DomainError> {
        encode_anim_png(img)
    }

    fn encode_gif(
        &self,
        frames: &[lume_domain::anim::Image],
        durations_ms: &[u32],
        looping: bool,
    ) -> Result<Vec<u8>, DomainError> {
        gif_io::encode_gif(frames, durations_ms, looping)
    }

    fn write_file(&self, path: &Path, bytes: &[u8]) -> Result<(), DomainError> {
        std::fs::write(path, bytes).map_err(|err| io_error("open", path, &err))
    }

    fn export_png(&self, canvas: &Canvas, path: &Path, scale: u32) -> Result<(), DomainError> {
        png_io::write_png(canvas, path, scale)
    }

    fn read_png(&self, path: &Path) -> Result<Canvas, DomainError> {
        png_io::read_png(path)
    }

    fn encode_png(&self, canvas: &Canvas, scale: u32) -> Result<Vec<u8>, DomainError> {
        png_io::encode_png(canvas, scale)
    }

    fn save_document(&self, document: &Document, path: &Path) -> Result<(), DomainError> {
        lume_file::write_lume(document, path)
    }

    fn open_document(&self, path: &Path) -> Result<Document, DomainError> {
        lume_file::read_lume(path)
    }

    fn import_pxcli_run(&self, dir: &Path) -> Result<Document, DomainError> {
        lume_file::import_pxcli_run(dir)
    }
}

/// Error `io` con un mensaje como los de Go en pxcli:
/// `open /ruta/x.png: no such file or directory`.
pub(crate) fn io_error(op: &str, path: &Path, err: &std::io::Error) -> DomainError {
    // Windows contesta «acceso denegado» al escribir sobre una carpeta; Go en
    // Unix (y pxcli) dice «is a directory».
    let text = if path.is_dir() && err.kind() != std::io::ErrorKind::NotFound {
        "is a directory".to_string()
    } else {
        go_error_text(err)
    };
    DomainError::new(ErrorCode::Io, format!("{op} {}: {text}", path.display()))
}

/// Texto de un error de E/S al estilo de Go: sin «(os error N)» y con la
/// primera letra en minúscula («no such file or directory»). Los errores
/// habituales salen con el texto de Unix en todos los sistemas, para que el
/// protocolo diga lo mismo en Linux, Windows y macOS.
fn go_error_text(err: &std::io::Error) -> String {
    match err.kind() {
        std::io::ErrorKind::NotFound => return "no such file or directory".into(),
        std::io::ErrorKind::PermissionDenied => return "permission denied".into(),
        std::io::ErrorKind::IsADirectory => return "is a directory".into(),
        _ => {}
    }
    let text = err.to_string();
    let text = match text.rfind(" (os error ") {
        Some(i) => &text[..i],
        None => &text,
    };
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_domain::Rgba;

    #[test]
    fn export_and_read_back_through_the_port() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.png");
        let mut c = Canvas::new(4, 4).unwrap();
        c.set(1, 2, Rgba::new(1, 2, 3, 4)).unwrap();
        let store: &dyn DocumentStore = &FsStore;
        store.export_png(&c, &path, 1).unwrap();
        assert_eq!(store.read_png(&path).unwrap(), c);

        store.export_png(&c, &path, 10).unwrap();
        let big = store.read_png(&path).unwrap();
        assert_eq!((big.width(), big.height()), (40, 40));
        assert_eq!(big.get(19, 29).unwrap(), Rgba::new(1, 2, 3, 4));
    }

    // Como TestCanvasExportPNGIOError de pxcli.
    #[test]
    fn writing_to_an_impossible_path_is_an_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("out.png");
        let c = Canvas::new(1, 1).unwrap();
        let err = FsStore.export_png(&c, &path, 1).unwrap_err();
        assert_eq!(err.code, ErrorCode::Io);
        assert_eq!(
            err.message,
            format!("open {}: no such file or directory", path.display())
        );
        // Una carpeta no se puede sobrescribir como archivo.
        let err = FsStore.export_png(&c, dir.path(), 1).unwrap_err();
        assert_eq!(err.code, ErrorCode::Io);
        assert!(err.message.ends_with(": is a directory"), "{}", err.message);
    }

    #[test]
    fn reading_a_missing_file_is_an_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nada.png");
        let err = FsStore.read_png(&path).unwrap_err();
        assert_eq!(err.code, ErrorCode::Io);
        assert_eq!(
            err.message,
            format!("open {}: no such file or directory", path.display())
        );
    }
}
