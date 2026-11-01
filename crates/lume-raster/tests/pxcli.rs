//! Rasterizado comparado con pxcli: los casos de `fixtures/cases.json`
//! pintados por pxcli con Chrome (`scripts/vec-golden/`) y el texto de su
//! rejilla y de sus números (`fixtures/pxcli.json`).

mod common;

use std::path::{Path, PathBuf};

use lume_app::ports::{RasterImage, RasterOptions, Rasterizer};
use lume_domain::Rgba;
use lume_domain::vec::raster::{grid_lines, num};
use lume_raster::ResvgRasterizer;
use serde_json::Value;

/// Diferencia media por píxel (0–255, ver `common::mean_diff`) que se
/// admite con la imagen de Chrome en cada caso. Documentado en
/// `docs/02-decisiones.md`.
const MAX_MEAN_DIFF: f64 = 4.0;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn json(name: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(fixtures().join(name)).unwrap()).unwrap()
}

fn options(o: &Value) -> RasterOptions {
    RasterOptions {
        width: o["width"].as_u64().map(|w| w as u32),
        crop: o["crop"].as_array().map(|c| {
            let v: Vec<f64> = c.iter().map(|x| x.as_f64().unwrap()).collect();
            [v[0], v[1], v[2], v[3]]
        }),
        grid: o["grid"].as_f64(),
        background: o["background"].as_str().map(|c| Rgba::parse(c).unwrap()),
        overlay: o["overlay"].as_str().map(str::to_string),
    }
}

/// Un caso pintado por LumeArt y por pxcli (Chrome).
fn case(name: &str) -> (RasterImage, RasterImage) {
    let cases = json("cases.json");
    let c = cases["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap_or_else(|| panic!("no hay caso {name}"));
    let svg = std::fs::read_to_string(fixtures().join(c["svg"].as_str().unwrap())).unwrap();
    let ours = ResvgRasterizer
        .rasterize(
            &svg,
            c["width"].as_f64().unwrap(),
            c["height"].as_f64().unwrap(),
            &options(&c["options"]),
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
    let chrome = common::read_png(&fixtures().join(format!("{name}.png")));
    (ours, chrome)
}

fn case_names() -> Vec<String> {
    json("cases.json")["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect()
}

/// La rejilla es el mismo texto que la de pxcli, y los números se escriben
/// igual.
#[test]
fn grid_markup_is_pxclis() {
    let golden = json("pxcli.json");
    for g in golden["grids"].as_array().unwrap() {
        let b: Vec<f64> = g["box"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        assert_eq!(
            grid_lines(b[0], b[1], b[2], b[3], b[4]),
            g["text"].as_str().unwrap(),
            "rejilla {b:?}"
        );
    }
    for n in golden["numbers"].as_array().unwrap() {
        let v = n["value"].as_f64().unwrap();
        assert_eq!(num(v), n["text"].as_str().unwrap(), "num({v})");
    }
}

/// Cada caso (entero, recorte, rejilla, fondo, superposición, texto) sale
/// del mismo tamaño que en pxcli y casi igual que con Chrome.
#[test]
fn every_case_looks_like_chrome() {
    let mut report = Vec::new();
    for name in case_names() {
        // Los casos con texto se pintaron con Chrome en Linux y sus fuentes
        // (Georgia, Lato, DejaVu): en otro sistema salen otras fuentes y la
        // comparación no dice nada del rasterizado.
        if name.starts_with("texto") && !cfg!(target_os = "linux") {
            continue;
        }
        let (ours, chrome) = case(&name);
        assert_eq!(
            (ours.width, ours.height),
            (chrome.width, chrome.height),
            "{name}: tamaño"
        );
        let diff = common::mean_diff(&ours, &chrome);
        report.push(format!("{name}: {diff:.3}"));
        assert!(diff < MAX_MEAN_DIFF, "{name}: diferencia media {diff:.3}");
    }
    eprintln!("{}", report.join("\n"));
}

/// Los píxeles que cambia una capa (con ella menos sin ella).
fn layer(with: &RasterImage, without: &RasterImage) -> Vec<bool> {
    with.rgba
        .chunks_exact(4)
        .zip(without.rgba.chunks_exact(4))
        .map(|(a, b)| (0..3).map(|i| a[i].abs_diff(b[i]) as u32).sum::<u32>() > 60)
        .collect()
}

/// Parte de los píxeles de `a` que tienen uno de `b` a un píxel o menos.
fn covered(a: &[bool], b: &[bool], width: usize) -> f64 {
    let height = a.len() / width;
    let near = |x: usize, y: usize| {
        (y.saturating_sub(1)..=(y + 1).min(height - 1))
            .any(|yy| (x.saturating_sub(1)..=(x + 1).min(width - 1)).any(|xx| b[yy * width + xx]))
    };
    let total = a.iter().filter(|p| **p).count();
    let hit = a
        .iter()
        .enumerate()
        .filter(|(i, p)| **p && near(i % width, i / width))
        .count();
    hit as f64 / total.max(1) as f64
}

/// La rejilla cae en los mismos sitios que la de pxcli: se aísla la capa
/// de la rejilla (con rejilla menos sin ella) en las dos y casi todos sus
/// píxeles coinciden, a un píxel como mucho.
#[test]
fn the_grid_lands_where_pxclis_does() {
    for (with, without) in [
        ("faro-recorte-rejilla", "faro-recorte-sin-rejilla"),
        ("texto-rejilla", "texto-ancho"),
    ] {
        let (ours_with, chrome_with) = case(with);
        let (ours_without, chrome_without) = case(without);
        let ours = layer(&ours_with, &ours_without);
        let chrome = layer(&chrome_with, &chrome_without);
        let width = ours_with.width as usize;
        let recall = covered(&chrome, &ours, width);
        let precision = covered(&ours, &chrome, width);
        eprintln!("{with}: {recall:.3} de pxcli están en LumeArt, {precision:.3} al revés");
        assert!(
            recall > 0.9 && precision > 0.9,
            "{with}: {recall:.3} / {precision:.3}"
        );
    }
}
