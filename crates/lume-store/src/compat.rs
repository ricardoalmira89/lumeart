//! Banco de compatibilidad con pxcli: para una ejecución guardada por pxcli
//! (`final.png` + `recipe.txt` + `meta.json`), reproduce la receta con el
//! dominio de LumeArt y compara píxel a píxel con `final.png`.
//!
//! La comparación tiene en cuenta cómo escribía pxcli los píxeles
//! semitransparentes en sus PNG (ver [`crate::png_io::pxcli_png_color`]).

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use lume_domain::{DomainError, ErrorCode, Recipe};
use serde_json::Value;

use crate::png_io::{pxcli_png_color, read_png};

/// Resultado de reproducir una ejecución de pxcli.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunComparison {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub commands: usize,
    /// Píxeles iguales a los de `final.png`.
    pub matching: usize,
    pub total: usize,
}

impl RunComparison {
    /// Porcentaje de píxeles iguales (0–100).
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.matching as f64 * 100.0 / self.total as f64
    }

    /// ¿Reproduce `final.png` exactamente?
    pub fn is_exact(&self) -> bool {
        self.total > 0 && self.matching == self.total
    }
}

/// Carpetas de `root` con `final.png` y `recipe.txt` (ejecuciones de pixel
/// art que se pueden reproducir), ordenadas por nombre.
pub fn pixel_art_runs(root: &Path) -> Vec<PathBuf> {
    let mut runs: Vec<PathBuf> = std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.join("final.png").is_file() && p.join("recipe.txt").is_file())
        .collect();
    runs.sort();
    runs
}

/// Reproduce la receta de una ejecución y la compara con su `final.png`.
///
/// El lienzo de partida tiene el tamaño de `meta.json` (`width`, `height`) o,
/// si no está, el de `final.png`. Si el tamaño final no coincide, no coincide
/// ningún píxel.
pub fn compare_pxcli_run(dir: &Path) -> Result<RunComparison, DomainError> {
    let expected = read_png(&dir.join("final.png"))?;
    let text = std::fs::read_to_string(dir.join("recipe.txt")).map_err(|e| {
        DomainError::new(
            ErrorCode::Io,
            format!("{}: {e}", dir.join("recipe.txt").display()),
        )
    })?;
    let recipe = Recipe::parse(&text)?;
    let start = std::fs::read(dir.join("meta.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .and_then(|m| {
            let w = u32::try_from(m.get("width")?.as_u64()?).ok()?;
            let h = u32::try_from(m.get("height")?.as_u64()?).ok()?;
            Some((w, h))
        })
        .unwrap_or((expected.width(), expected.height()));
    let canvas = recipe.replay(start.0, start.1)?;

    let total = expected.pixels().len();
    let matching = if (canvas.width(), canvas.height()) == (expected.width(), expected.height()) {
        canvas
            .pixels()
            .iter()
            .zip(expected.pixels())
            .filter(|(got, want)| pxcli_png_color(**got) == **want)
            .count()
    } else {
        0
    };
    Ok(RunComparison {
        name: dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        width: expected.width(),
        height: expected.height(),
        commands: recipe.len(),
        matching,
        total,
    })
}

/// Informe legible: una línea por ejecución y el total.
pub fn report(results: &[RunComparison]) -> String {
    let mut out = String::new();
    for r in results {
        let _ = writeln!(
            out,
            "{:>7.3} %  {:>3}x{:<3}  {:>4} comandos  {}",
            r.percent(),
            r.width,
            r.height,
            r.commands,
            r.name
        );
    }
    let exact = results.iter().filter(|r| r.is_exact()).count();
    let pixels: usize = results.iter().map(|r| r.total).sum();
    let matching: usize = results.iter().map(|r| r.matching).sum();
    let pixel_percent = if pixels == 0 {
        0.0
    } else {
        matching as f64 * 100.0 / pixels as f64
    };
    let run_percent = if results.is_empty() {
        0.0
    } else {
        exact as f64 * 100.0 / results.len() as f64
    };
    let _ = writeln!(
        out,
        "ejecuciones idénticas: {exact} de {} ({run_percent:.1} %) · píxeles iguales: {pixel_percent:.3} %",
        results.len()
    );
    out
}
