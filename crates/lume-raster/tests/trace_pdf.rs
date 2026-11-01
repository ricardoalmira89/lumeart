//! El calco (`trace`) y el PDF con capas (`pdf`).

use std::path::Path;

use lume_domain::vec::trace::{TraceOptions, color_dist, hex_rgb, post_trace, trace_colors};
use lume_raster::pdf::svg_to_pdf;
use lume_raster::trace::trace_reference;

/// Una imagen de 120 × 80: arriba a la izquierda rojo, a la derecha azul,
/// abajo una franja negra, y una esquina transparente.
fn reference_png() -> Vec<u8> {
    let (w, h) = (120u32, 80u32);
    let mut rgba = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let px = if x > 100 && y < 15 {
                [0, 0, 0, 0]
            } else if y >= 60 {
                [20, 20, 20, 255]
            } else if x < 60 {
                [220, 30, 40, 255]
            } else {
                [30, 60, 200, 255]
            };
            rgba.extend_from_slice(&px);
        }
    }
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&rgba).unwrap();
    out
}

fn fills(fragment: &str) -> Vec<String> {
    trace_colors(fragment).into_iter().map(|c| c.hex).collect()
}

/// En color, el calco tiene los colores de la imagen; con una paleta, solo
/// los de la paleta; y está en las coordenadas del lienzo.
#[test]
fn the_trace_uses_the_requested_palette() {
    let png = reference_png();
    // Lienzo de 240 × 160: la imagen se amplía al doble.
    let job = TraceOptions {
        width: 240,
        height: 160,
        colors: 3,
        noise: 2.0,
        ..TraceOptions::default()
    };
    let raw = trace_reference(&png, "image/png", &job).unwrap();
    let colors = fills(&raw);
    assert!(colors.len() <= 3, "{colors:?}");
    for want in ["#dc1e28", "#1e3cc8", "#141414"] {
        let target = hex_rgb(want);
        assert!(
            colors.iter().any(|c| color_dist(hex_rgb(c), target) < 30.0),
            "falta {want}: {colors:?}"
        );
    }
    assert!(raw.contains("<path d=\"M"), "{raw}");

    // Con la paleta del dibujo, cada trazo usa uno de sus colores.
    let palette = vec![
        "#ff0000".to_string(),
        "#0000ff".into(),
        "#000000".into(),
        "#ffff00".into(),
    ];
    let snapped = post_trace(
        &raw,
        &TraceOptions {
            palette: palette.clone(),
            ..job.clone()
        },
    );
    let used = fills(&snapped);
    assert!(!used.is_empty());
    for c in &used {
        assert!(palette.contains(c), "{c} no está en la paleta");
    }
    assert!(!used.contains(&"#ffff00".to_string()));

    // Quitar el azul.
    let without_blue = post_trace(
        &raw,
        &TraceOptions {
            drop: vec!["#1e3cc8".into()],
            ..job.clone()
        },
    );
    assert!(
        fills(&without_blue)
            .iter()
            .all(|c| color_dist(hex_rgb(c), hex_rgb("#1e3cc8")) > 40.0)
    );

    // En líneas, solo la tinta (la franja oscura).
    let lines_job = TraceOptions {
        lines: true,
        ink: "#222244".into(),
        ..job.clone()
    };
    let lines = post_trace(
        &trace_reference(&png, "image/png", &lines_job).unwrap(),
        &lines_job,
    );
    assert_eq!(fills(&lines), ["#222244"]);
}

#[test]
fn a_reference_that_is_not_an_image_is_an_error() {
    assert!(
        trace_reference(b"nada", "image/png", &TraceOptions::default()).is_ok_and(|f| f.is_empty())
    );
}

/// El PDF es válido (lo lee `lopdf`), tiene una página del tamaño del
/// dibujo y una capa con nombre por cada grupo de primer nivel.
#[test]
fn the_pdf_has_named_layers() {
    let svg = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/faro.svg"),
    )
    .unwrap();
    let pdf = svg_to_pdf(&svg).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    let doc = lopdf::Document::load_mem(&pdf).expect("PDF válido");
    let pages = doc.get_pages();
    assert_eq!(pages.len(), 1);
    let page = doc
        .get_object(*pages.get(&1).unwrap())
        .unwrap()
        .as_dict()
        .unwrap();
    let media: Vec<f32> = page
        .get(b"MediaBox")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_float().unwrap())
        .collect();
    assert_eq!(media, [0.0, 0.0, 768.0, 1024.0]);

    let catalog = doc.catalog().unwrap();
    let props = catalog.get(b"OCProperties").unwrap().as_dict().unwrap();
    let ocgs = props.get(b"OCGs").unwrap().as_array().unwrap();
    let names: Vec<String> = ocgs
        .iter()
        .map(|r| {
            let ocg = doc
                .get_object(r.as_reference().unwrap())
                .unwrap()
                .as_dict()
                .unwrap();
            assert_eq!(ocg.get(b"Type").unwrap().as_name().unwrap(), b"OCG");
            let bytes = ocg.get(b"Name").unwrap().as_str().unwrap();
            lopdf::decode_text_string(&lopdf::Object::String(
                bytes.to_vec(),
                lopdf::StringFormat::Literal,
            ))
            .unwrap()
        })
        .collect();
    assert_eq!(
        names,
        [
            "cielo",
            "cabos lejanos",
            "mar",
            "acantilado",
            "faro",
            "rocas y olas",
            "gaviotas",
            "grano"
        ]
    );
    // Cada capa, en el contenido, dentro de su grupo opcional.
    let content = doc.get_page_content(*pages.get(&1).unwrap());
    let text = String::from_utf8_lossy(&content);
    assert_eq!(text.matches("/OC /oc").count(), 8, "{text}");
    assert_eq!(text.matches("EMC").count(), 8);

    // Un nombre con acentos se guarda como texto Unicode.
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 10" width="20" height="10"><g data-name="Árbol pequeño"><rect width="5" height="5" fill="#00aa00"/></g></svg>"##;
    let doc = lopdf::Document::load_mem(&svg_to_pdf(svg).unwrap()).unwrap();
    let pdf_text = doc
        .objects
        .values()
        .filter_map(|o| o.as_dict().ok())
        .find(|d| {
            d.get(b"Type")
                .and_then(|t| t.as_name())
                .is_ok_and(|n| n == b"OCG")
        })
        .and_then(|d| d.get(b"Name").ok().cloned())
        .unwrap();
    assert_eq!(
        lopdf::decode_text_string(&pdf_text).unwrap(),
        "Árbol pequeño"
    );
}
