//! Comparar con las imágenes de Chrome.

#![allow(dead_code)]

use std::path::Path;

use lume_app::ports::RasterImage;

/// Un PNG como RGBA de 8 bits (RGB y gris pasan a opacos).
pub fn read_png(path: &Path) -> RasterImage {
    let file = std::fs::File::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();
    buf.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => buf
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Grayscale => buf.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        other => panic!("{}: PNG {other:?}", path.display()),
    };
    RasterImage {
        width: info.width,
        height: info.height,
        rgba,
    }
}

/// Diferencia media por píxel entre dos imágenes del mismo tamaño, de 0 a
/// 255: la media de |Δ| en los cuatro canales con el color premultiplicado
/// por el alfa (un píxel transparente no cuenta su color).
pub fn mean_diff(a: &RasterImage, b: &RasterImage) -> f64 {
    assert_eq!(
        (a.width, a.height),
        (b.width, b.height),
        "tamaños distintos"
    );
    let pre = |p: &[u8]| {
        let a = u32::from(p[3]);
        [
            u32::from(p[0]) * a / 255,
            u32::from(p[1]) * a / 255,
            u32::from(p[2]) * a / 255,
            a,
        ]
    };
    let mut total: u64 = 0;
    for (p, q) in a.rgba.chunks_exact(4).zip(b.rgba.chunks_exact(4)) {
        let (p, q) = (pre(p), pre(q));
        for i in 0..4 {
            total += u64::from(p[i].abs_diff(q[i]));
        }
    }
    total as f64 / (a.rgba.len() as f64)
}
