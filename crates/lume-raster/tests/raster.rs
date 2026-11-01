//! El adaptador `ResvgRasterizer`: fondo, recorte, errores, imágenes y
//! fuentes.

use lume_app::ports::{PortError, RasterImage, RasterOptions, Rasterizer};
use lume_domain::Rgba;
use lume_raster::{ResvgRasterizer, serif_family};

fn px(img: &RasterImage, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * img.width + x) * 4) as usize;
    img.rgba[i..i + 4].try_into().unwrap()
}

fn raster(svg: &str, w: f64, h: f64, options: RasterOptions) -> RasterImage {
    ResvgRasterizer.rasterize(svg, w, h, &options).unwrap()
}

const HALVES: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100"><rect x="10" y="10" width="40" height="80" fill="#ff0000"/><rect x="50" y="10" width="40" height="80" fill="#0000ff"/></svg>"##;

#[test]
fn transparency_is_kept_unless_there_is_a_background() {
    let img = raster(
        HALVES,
        100.0,
        100.0,
        RasterOptions {
            width: Some(100),
            ..Default::default()
        },
    );
    assert_eq!((img.width, img.height), (100, 100));
    assert_eq!(px(&img, 2, 2), [0, 0, 0, 0]);
    assert_eq!(px(&img, 30, 50), [255, 0, 0, 255]);
    assert_eq!(px(&img, 70, 50), [0, 0, 255, 255]);
    let img = raster(
        HALVES,
        100.0,
        100.0,
        RasterOptions {
            width: Some(100),
            background: Some(Rgba::parse("#1e2a3a").unwrap()),
            ..Default::default()
        },
    );
    assert_eq!(px(&img, 2, 2), [0x1e, 0x2a, 0x3a, 255]);
    assert_eq!(px(&img, 30, 50), [255, 0, 0, 255]);
}

#[test]
fn the_crop_zooms_in_and_the_width_scales() {
    // La mitad derecha a 50 px de ancho: todo azul salvo los bordes.
    let img = raster(
        HALVES,
        100.0,
        100.0,
        RasterOptions {
            width: Some(50),
            crop: Some([50.0, 10.0, 40.0, 80.0]),
            ..Default::default()
        },
    );
    assert_eq!((img.width, img.height), (50, 100));
    assert_eq!(px(&img, 25, 50), [0, 0, 255, 255]);
    assert_eq!(px(&img, 1, 1), [0, 0, 255, 255]);
    // Sin ancho, 1024; el dibujo entero se amplía.
    let img = raster(HALVES, 100.0, 100.0, RasterOptions::default());
    assert_eq!((img.width, img.height), (1024, 1024));
    assert_eq!(px(&img, 300, 512), [255, 0, 0, 255]);
}

#[test]
fn the_overlay_and_grid_go_on_top() {
    let img = raster(
        HALVES,
        100.0,
        100.0,
        RasterOptions {
            width: Some(100),
            overlay: Some(
                r##"<rect x="40" y="40" width="20" height="20" fill="#00ff00"/>"##.into(),
            ),
            ..Default::default()
        },
    );
    assert_eq!(px(&img, 45, 50), [0, 255, 0, 255]);
    assert_eq!(px(&img, 30, 30), [255, 0, 0, 255]);
    // La rejilla cada 50 unidades: una línea rosa en x = 50.
    let plain = raster(
        HALVES,
        100.0,
        100.0,
        RasterOptions {
            width: Some(400),
            ..Default::default()
        },
    );
    let grid = raster(
        HALVES,
        100.0,
        100.0,
        RasterOptions {
            width: Some(400),
            grid: Some(50.0),
            ..Default::default()
        },
    );
    assert_ne!(px(&grid, 200, 300), px(&plain, 200, 300));
    assert_eq!(px(&grid, 100, 300), px(&plain, 100, 300));
}

#[test]
fn errors_say_what_is_wrong() {
    let err = ResvgRasterizer
        .rasterize(HALVES, 10.0, 100.0, &RasterOptions::default())
        .unwrap_err();
    assert_eq!(
        err,
        PortError::Invalid("tamaño de salida no válido: 1024x10240".into())
    );
    let err = ResvgRasterizer
        .rasterize("<svg><g></svg>", 10.0, 10.0, &RasterOptions::default())
        .unwrap_err();
    assert!(
        matches!(&err, PortError::Invalid(m) if m.starts_with("SVG no válido")),
        "{err}"
    );
}

/// Un PNG rojo de 2 × 2.
const RED_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAEUlEQVR4nGP4z8DwH4QZYAwAR8oH+WdZbrcAAAAASUVORK5CYII=";

#[test]
fn only_embedded_images_are_drawn() {
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 10" width="20" height="10"><image x="0" y="0" width="10" height="10" href="data:image/png;base64,{RED_PNG}"/><image x="10" y="0" width="10" height="10" href="/etc/hostname"/><image x="10" y="0" width="10" height="10" href="imagen.png"/></svg>"#
    );
    let img = raster(
        &svg,
        20.0,
        10.0,
        RasterOptions {
            width: Some(20),
            ..Default::default()
        },
    );
    assert_eq!(px(&img, 5, 5), [255, 0, 0, 255]);
    assert_eq!(
        px(&img, 15, 5),
        [0, 0, 0, 0],
        "un archivo del disco no se lee"
    );
}

#[test]
fn text_always_finds_a_font() {
    assert!(!serif_family().is_empty());
    for family in ["serif", "sans-serif", "monospace", "Una Que No Existe"] {
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 50" width="200" height="50"><text x="5" y="40" font-size="36" font-family="{family}">Hola</text></svg>"#
        );
        let img = raster(
            &svg,
            200.0,
            50.0,
            RasterOptions {
                width: Some(200),
                ..Default::default()
            },
        );
        let inked = img.rgba.chunks_exact(4).filter(|p| p[3] > 128).count();
        assert!(inked > 100, "{family}: {inked} píxeles");
    }
}
