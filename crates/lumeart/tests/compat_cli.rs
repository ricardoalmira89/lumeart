//! Compatibilidad de extremo a extremo: las recetas de ejecuciones reales de
//! pxcli, enviadas con el binario `lumeart` (comando a comando o con
//! `batch`), producen el mismo PNG que `final.png`.
//!
//! Los PNG de pxcli guardaban mal los píxeles semitransparentes; la
//! comparación lo tiene en cuenta (`lume_store::lume_file::matches_pxcli_png`).

mod common;

use std::path::{Path, PathBuf};

use common::{Lumeart, text};
use lume_store::compat::pixel_art_runs;
use lume_store::lume_file::matches_pxcli_png;
use lume_store::png_io::read_png;

/// Tamaño de partida de una ejecución (`meta.json`, o el de `final.png`).
fn start_size(run: &Path) -> String {
    let meta: Option<serde_json::Value> = std::fs::read(run.join("meta.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok());
    match meta {
        Some(m) if m["width"].is_u64() => format!("{}x{}", m["width"], m["height"]),
        _ => {
            let png = read_png(&run.join("final.png")).unwrap();
            format!("{}x{}", png.width(), png.height())
        }
    }
}

/// Exporta el lienzo de la sesión, la cierra y compara con `final.png`.
fn export_and_compare(lumeart: &Lumeart, run: &Path, name: &str) {
    let out = lumeart.run(&["export", name]);
    assert_eq!(text(&out.stdout), "ok\n", "{}", text(&out.stderr));
    let out = lumeart.run(&["stop"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let ours = read_png(&lumeart.cwd.join(name)).unwrap();
    let theirs = read_png(&run.join("final.png")).unwrap();
    assert!(
        matches_pxcli_png(&ours, &theirs),
        "{}: el PNG exportado por lumeart no coincide con final.png",
        run.display()
    );
}

fn fixtures() -> Vec<PathBuf> {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../lume-store/tests/fixtures/pxcli-runs");
    let runs = pixel_art_runs(&root);
    assert!(runs.len() >= 3, "faltan fixtures en {}", root.display());
    runs
}

/// Cada línea de la receta como un comando de la CLI (como lo hacía el agente
/// con pxcli), con el arranque al estilo del agente de ejemplo de pxcli.
#[test]
fn fixture_recipes_command_by_command() {
    for run in fixtures() {
        let dir = tempfile::tempdir().unwrap();
        let lumeart = Lumeart::new(dir.path());
        let size = format!("--size={}", start_size(&run));
        let out = lumeart.run(&["start", &size, "--headless=true", "--scale", "10"]);
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

        let recipe = std::fs::read_to_string(run.join("recipe.txt")).unwrap();
        let mut sent = 0;
        for line in recipe.lines().filter(|l| !l.trim().is_empty()) {
            let args: Vec<&str> = line.split_whitespace().collect();
            let out = lumeart.run(&args);
            assert_eq!(text(&out.stdout), "ok\n", "{line}: {}", text(&out.stderr));
            sent += 1;
        }
        assert!(sent > 10);
        export_and_compare(&lumeart, &run, "lumeart.png");
    }
}

/// La receta entera con `lumeart batch` por la entrada estándar.
fn batch_run(run: &Path) {
    let dir = tempfile::tempdir().unwrap();
    let lumeart = Lumeart::new(dir.path());
    let out = lumeart.run(&["start", "--headless", "--size", &start_size(run)]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let recipe = std::fs::read_to_string(run.join("recipe.txt")).unwrap();
    let out = lumeart.run_with_stdin(&["batch"], &recipe);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{stdout}{}", text(&out.stderr));
    assert!(stdout.ends_with(" ok, 0 failed\n"), "{stdout}");
    export_and_compare(&lumeart, run, "batch.png");
}

#[test]
fn fixture_recipes_with_batch() {
    for run in fixtures() {
        batch_run(&run);
    }
}

/// Todas las ejecuciones de pixel art de `reference/runs` (si existe).
#[test]
fn all_reference_recipes_with_batch() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reference/runs");
    if !root.is_dir() {
        eprintln!("se salta: no existe reference/runs (scripts/setup-reference.sh)");
        return;
    }
    let runs = pixel_art_runs(&root);
    assert!(!runs.is_empty());
    for run in &runs {
        batch_run(run);
    }
    eprintln!(
        "{} ejecuciones de pxcli reproducidas con `lumeart batch`",
        runs.len()
    );
}

/// `--headless=false` (el agente de ejemplo de pxcli abre la ventana) se
/// entiende; sin pantalla, responde `renderer_unavailable` en vez de colgarse.
/// Solo en Linux, donde «sin pantalla» es quitar `DISPLAY`/`WAYLAND_DISPLAY`.
#[cfg(target_os = "linux")]
#[test]
fn headless_false_without_a_display() {
    let dir = tempfile::tempdir().unwrap();
    let lumeart = Lumeart::new(dir.path());
    let out = lumeart.run_without_display(&[
        "start",
        "--size=32x32",
        "--headless=false",
        "--scale",
        "10",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("renderer_unavailable"),
        "{}",
        text(&out.stderr)
    );
    assert!(!lumeart.state.exists());
}
