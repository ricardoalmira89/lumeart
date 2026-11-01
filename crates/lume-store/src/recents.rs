//! Lista de documentos recientes (`recents.json` en la carpeta de datos del
//! usuario). La miniatura se lee del propio `.lume`.

use std::path::{Path, PathBuf};

use lume_app::{PortError, RecentEntry, RecentStore};
use serde::{Deserialize, Serialize};

/// Cuántos recientes se guardan.
pub const MAX_RECENTS: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Item {
    path: PathBuf,
    opened_at: u64,
}

/// `RecentStore` sobre un archivo JSON.
#[derive(Debug, Clone)]
pub struct FsRecents {
    file: PathBuf,
}

impl FsRecents {
    pub fn new(file: impl Into<PathBuf>) -> Self {
        Self { file: file.into() }
    }

    fn read(&self) -> Vec<Item> {
        std::fs::read(&self.file)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    fn write(&self, items: &[Item]) -> Result<(), PortError> {
        if let Some(dir) = self.file.parent() {
            std::fs::create_dir_all(dir).map_err(|e| PortError::Io(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(items).expect("la lista siempre se serializa");
        std::fs::write(&self.file, json).map_err(|e| PortError::Io(e.to_string()))
    }
}

impl RecentStore for FsRecents {
    fn list(&self) -> Vec<RecentEntry> {
        let mut items = self.read();
        items.sort_by_key(|item| std::cmp::Reverse(item.opened_at));
        items
            .into_iter()
            .filter_map(|item| {
                let document = crate::lume_file::read_lume(&item.path).ok()?;
                Some(RecentEntry {
                    name: document.name.clone(),
                    thumbnail: document.canvas().clone(),
                    path: item.path,
                    opened_at: item.opened_at,
                })
            })
            .collect()
    }

    fn touch(&self, path: &Path, opened_at: u64) -> Result<(), PortError> {
        let mut items = self.read();
        items.retain(|i| i.path != path);
        items.insert(
            0,
            Item {
                path: path.to_path_buf(),
                opened_at,
            },
        );
        items.sort_by_key(|item| std::cmp::Reverse(item.opened_at));
        items.truncate(MAX_RECENTS);
        self.write(&items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_domain::{Document, Origin, Rgba};

    fn save(dir: &Path, name: &str, w: i64) -> PathBuf {
        let mut doc = Document::new(name, w, w).unwrap();
        doc.apply(
            lume_domain::Command::SetPixel {
                x: 0,
                y: 0,
                color: Rgba::rgb(255, 0, 0),
            },
            Origin::User,
        )
        .unwrap();
        let path = dir.join(format!("{name}.lume"));
        crate::lume_file::write_lume(&doc, &path).unwrap();
        path
    }

    #[test]
    fn touch_and_list_newest_first_with_thumbnails() {
        let dir = tempfile::tempdir().unwrap();
        let recents = FsRecents::new(dir.path().join("datos/recents.json"));
        assert!(recents.list().is_empty());
        let a = save(dir.path(), "pocion", 32);
        let b = save(dir.path(), "seta", 16);
        recents.touch(&a, 100).unwrap();
        recents.touch(&b, 200).unwrap();
        let list = recents.list();
        assert_eq!(
            list.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            vec!["seta", "pocion"]
        );
        assert_eq!(list[0].thumbnail.width(), 16);
        assert_eq!(list[1].thumbnail.get(0, 0).unwrap(), Rgba::rgb(255, 0, 0));
        // Volver a abrir uno lo sube arriba sin repetirlo.
        recents.touch(&a, 300).unwrap();
        let list = recents.list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "pocion");
        assert_eq!(list[0].opened_at, 300);
    }

    #[test]
    fn missing_files_are_skipped_and_the_list_is_capped() {
        let dir = tempfile::tempdir().unwrap();
        let recents = FsRecents::new(dir.path().join("recents.json"));
        recents
            .touch(&dir.path().join("no-existe.lume"), 5)
            .unwrap();
        assert!(recents.list().is_empty());
        for i in 0..20 {
            recents
                .touch(&dir.path().join(format!("{i}.lume")), i)
                .unwrap();
        }
        let raw: Vec<Item> =
            serde_json::from_slice(&std::fs::read(dir.path().join("recents.json")).unwrap())
                .unwrap();
        assert_eq!(raw.len(), MAX_RECENTS);
        assert_eq!(raw[0].opened_at, 19);
    }
}
