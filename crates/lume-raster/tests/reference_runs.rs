//! Todas las ejecuciones vectoriales de `reference/runs/` (si existe; la
//! crea `scripts/setup-reference.sh`): su `final.svg` se rasteriza sin
//! errores y queda casi igual que el `final.png` que hizo pxcli con Chrome.
//! Sin `reference/`, se salta.

mod common;

use std::path::Path;

use lume_app::ports::{RasterOptions, Rasterizer};
use lume_raster::ResvgRasterizer;

/// Diferencia media por píxel admitida en cada ejecución (la misma que en
/// `tests/pxcli.rs`) y en el conjunto.
const MAX_MEAN_DIFF: f64 = 4.0;
const MAX_AVERAGE_DIFF: f64 = 1.0;

/// Rasteriza una ejecución y, si tiene `final.png`, devuelve su diferencia
/// con él.
fn check(dir: &Path) -> Option<(f64, String)> {
    let name = dir.file_name().unwrap().to_string_lossy().to_string();
    let svg = std::fs::read_to_string(dir.join("final.svg")).unwrap();
    let meta: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("meta.json")).unwrap()).unwrap();
    let (w, h) = (
        meta["width"].as_f64().unwrap(),
        meta["height"].as_f64().unwrap(),
    );
    // pxcli guardaba final.png al ancho del dibujo.
    let image = ResvgRasterizer
        .rasterize(
            &svg,
            w,
            h,
            &RasterOptions {
                width: Some(w as u32),
                ..RasterOptions::default()
            },
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
    assert_eq!((image.width, image.height), (w as u32, h as u32), "{name}");
    if !dir.join("final.png").is_file() {
        return None;
    }
    let chrome = common::read_png(&dir.join("final.png"));
    let diff = common::mean_diff(&image, &chrome);
    assert!(diff < MAX_MEAN_DIFF, "{name}: diferencia media {diff:.3}");
    Some((diff, name))
}

#[test]
fn every_vector_run_rasterizes_like_chrome() {
    let runs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reference/runs");
    if !runs.is_dir() {
        eprintln!("sin reference/runs: se salta (ejecuta scripts/setup-reference.sh)");
        return;
    }
    let mut dirs: Vec<_> = std::fs::read_dir(&runs)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|d| d.join("final.svg").is_file())
        .collect();
    dirs.sort();
    // Repartidas entre hilos: son casi noventa y algunas pesan megas.
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = dirs.len().div_ceil(threads).max(1);
    let mut diffs: Vec<(f64, String)> = std::thread::scope(|scope| {
        let handles: Vec<_> = dirs
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || part.iter().filter_map(|d| check(d)).collect::<Vec<_>>())
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    assert!(
        !dirs.is_empty(),
        "reference/runs no tiene dibujos vectoriales"
    );
    diffs.sort_by(|a, b| b.0.total_cmp(&a.0));
    let average = diffs.iter().map(|d| d.0).sum::<f64>() / diffs.len().max(1) as f64;
    eprintln!(
        "{} SVG rasterizados, {} comparados con Chrome; media {average:.3}, la mayor {:.3} ({})",
        dirs.len(),
        diffs.len(),
        diffs.first().map_or(0.0, |d| d.0),
        diffs.first().map_or("", |d| d.1.as_str())
    );
    assert!(average < MAX_AVERAGE_DIFF, "media {average:.3}");
}
