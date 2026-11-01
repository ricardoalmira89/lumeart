//! Diálogos de archivo. La ventana usa los nativos (`rfd`: portal XDG en
//! Linux, sin GTK); los tests, [`FakeDialogs`].

use std::path::PathBuf;
use std::sync::Mutex;

/// Elegir archivos y carpetas.
pub trait FileDialogs: Send + Sync {
    /// Un `.lume` para abrir.
    fn open_lume(&self) -> Option<PathBuf>;
    /// Dónde guardar un `.lume` (con un nombre sugerido).
    fn save_lume(&self, suggested: &str) -> Option<PathBuf>;
    /// Dónde exportar un PNG (con un nombre sugerido).
    fn save_png(&self, suggested: &str) -> Option<PathBuf>;
    /// Una carpeta (la de una ejecución de pxcli).
    fn pick_folder(&self) -> Option<PathBuf>;
    /// Una imagen de referencia (PNG, JPEG, GIF o WebP).
    fn pick_image(&self) -> Option<PathBuf>;
    /// Abre una carpeta en el gestor de archivos del sistema.
    fn open_folder(&self, path: &std::path::Path);
    /// Dónde guardar un archivo de un tipo (`kind`, «Imagen PNG») con su
    /// extensión (`ext`, sin punto).
    fn save_as(&self, suggested: &str, kind: &str, ext: &str) -> Option<PathBuf>;
}

/// Diálogos nativos.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeDialogs;

impl FileDialogs for NativeDialogs {
    fn open_lume(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Abrir dibujo")
            .add_filter("Dibujo de LumeArt", &["lume"])
            .pick_file()
    }

    fn save_lume(&self, suggested: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Guardar dibujo")
            .add_filter("Dibujo de LumeArt", &["lume"])
            .set_file_name(format!("{suggested}.lume"))
            .save_file()
            .map(|p| with_extension(p, "lume"))
    }

    fn save_png(&self, suggested: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Exportar PNG")
            .add_filter("Imagen PNG", &["png"])
            .set_file_name(format!("{suggested}.png"))
            .save_file()
            .map(|p| with_extension(p, "png"))
    }

    fn pick_folder(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Importar una ejecución de pxcli")
            .pick_folder()
    }

    fn pick_image(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Imagen de referencia")
            .add_filter("Imagen", &["png", "jpg", "jpeg", "gif", "webp"])
            .pick_file()
    }

    fn save_as(&self, suggested: &str, kind: &str, ext: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Exportar")
            .add_filter(kind, &[ext])
            .set_file_name(format!("{suggested}.{ext}"))
            .save_file()
            .map(|p| with_extension(p, ext))
    }

    fn open_folder(&self, path: &std::path::Path) {
        let opener = if cfg!(target_os = "windows") {
            "explorer"
        } else if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let _ = std::process::Command::new(opener).arg(path).spawn();
    }
}

/// Añade la extensión si el usuario no la escribió.
pub fn with_extension(path: PathBuf, ext: &str) -> PathBuf {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
    {
        path
    } else {
        let mut name = path.into_os_string();
        name.push(".");
        name.push(ext);
        PathBuf::from(name)
    }
}

/// Diálogos falsos para tests: devuelven las rutas que se les den (o
/// `None`, como si el usuario cancelara) y apuntan los nombres sugeridos.
#[derive(Debug, Default)]
pub struct FakeDialogs {
    pub open: Mutex<Option<PathBuf>>,
    pub save: Mutex<Option<PathBuf>>,
    pub png: Mutex<Option<PathBuf>>,
    pub folder: Mutex<Option<PathBuf>>,
    pub image: Mutex<Option<PathBuf>>,
    pub suggested: Mutex<Vec<String>>,
    /// Carpetas que se pidió abrir.
    pub opened: Mutex<Vec<PathBuf>>,
}

impl FileDialogs for FakeDialogs {
    fn open_lume(&self) -> Option<PathBuf> {
        self.open.lock().unwrap().clone()
    }

    fn save_lume(&self, suggested: &str) -> Option<PathBuf> {
        self.suggested.lock().unwrap().push(suggested.to_string());
        self.save.lock().unwrap().clone()
    }

    fn save_png(&self, suggested: &str) -> Option<PathBuf> {
        self.suggested.lock().unwrap().push(suggested.to_string());
        self.png.lock().unwrap().clone()
    }

    fn pick_folder(&self) -> Option<PathBuf> {
        self.folder.lock().unwrap().clone()
    }

    fn pick_image(&self) -> Option<PathBuf> {
        self.image.lock().unwrap().clone()
    }

    fn open_folder(&self, path: &std::path::Path) {
        self.opened.lock().unwrap().push(path.to_path_buf());
    }

    fn save_as(&self, suggested: &str, _kind: &str, ext: &str) -> Option<PathBuf> {
        self.suggested.lock().unwrap().push(suggested.to_string());
        self.save
            .lock()
            .unwrap()
            .clone()
            .map(|p| p.with_extension(ext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_are_added_when_missing() {
        assert_eq!(
            with_extension("/a/cofre".into(), "lume"),
            PathBuf::from("/a/cofre.lume")
        );
        assert_eq!(
            with_extension("/a/cofre.LUME".into(), "lume"),
            PathBuf::from("/a/cofre.LUME")
        );
        assert_eq!(
            with_extension("/a/mi.juego/x".into(), "png"),
            PathBuf::from("/a/mi.juego/x.png")
        );
    }
}
