//! Codificar y decodificar PNG con el crate `png` (Rust puro).
//!
//! Se escribe siempre RGBA de 8 bits **sin premultiplicar**, tal como guarda
//! los colores el lienzo. (pxcli pasaba sus colores por `image.RGBA` de Go,
//! que los trata como premultiplicados: los píxeles semitransparentes salían
//! alterados en el PNG. Ver `docs/02-decisiones.md`.)

use std::fs;
use std::io::Cursor;
use std::path::Path;

use lume_domain::anim::Image;
use lume_domain::{Canvas, DomainError, ErrorCode, Rgba};

use crate::io_error;

/// Escala máxima de exportación (como pxcli).
pub const MAX_SCALE: u32 = 64;

/// PNG RGBA del lienzo con cada píxel ampliado a `scale` × `scale`.
pub fn encode_png(canvas: &Canvas, scale: u32) -> Result<Vec<u8>, DomainError> {
    if !(1..=MAX_SCALE).contains(&scale) {
        return Err(DomainError::invalid_args(format!(
            "scale must be between 1 and {MAX_SCALE}"
        )));
    }
    let (width, height, data) = canvas.scaled_rgba_bytes(scale);
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(png_error)?;
    writer.write_image_data(&data).map_err(png_error)?;
    writer.finish().map_err(png_error)?;
    Ok(out)
}

/// Lee un PNG de cualquier tipo de color y profundidad como lienzo RGBA de 8
/// bits (las imágenes de 16 bits se reducen tomando el byte alto, como Go).
pub fn decode_png(bytes: &[u8]) -> Result<Canvas, DomainError> {
    let (width, height, rgba) = decode_rgba(bytes)?;
    Canvas::from_rgba_bytes(i64::from(width), i64::from(height), &rgba)
}

/// Un PNG como bytes RGBA de 8 bits sin premultiplicar, de cualquier tamaño.
fn decode_rgba(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), DomainError> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(png_error)?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| DomainError::new(ErrorCode::Io, "png: image too large"))?;
    let mut buf = vec![0; size];
    let info = reader.next_frame(&mut buf).map_err(png_error)?;
    let buf = &buf[..info.buffer_size()];
    let rgba: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => buf.to_vec(),
        png::ColorType::Rgb => buf
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => buf
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Grayscale => buf.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => {
            return Err(DomainError::new(
                ErrorCode::Io,
                "png: indexed image was not expanded",
            ));
        }
    };
    Ok((info.width, info.height, rgba))
}

/// Un PNG como imagen de cualquier tamaño (sin premultiplicar).
pub fn decode_image(bytes: &[u8]) -> Result<Image, DomainError> {
    let (width, height, rgba) = decode_rgba(bytes)?;
    let pixels = rgba
        .chunks_exact(4)
        .map(|p| Rgba::new(p[0], p[1], p[2], p[3]))
        .collect();
    Image::from_pixels(width, height, pixels)
        .ok_or_else(|| DomainError::new(ErrorCode::Io, "png: tamaño incoherente"))
}

/// Lee un archivo PNG como imagen de cualquier tamaño.
pub fn read_image(path: &Path) -> Result<Image, DomainError> {
    let bytes = fs::read(path).map_err(|err| io_error("open", path, &err))?;
    decode_image(&bytes)
}

/// Una imagen de animación como la carga pxcli (`anim.LoadPNG`): con el alfa
/// premultiplicado de `image.RGBA`.
pub fn read_anim_image(path: &Path) -> Result<Image, DomainError> {
    Ok(read_image(path)?.premultiplied())
}

/// PNG con los bytes RGBA de una imagen tal cual (sin conversión): para
/// guardar en el `.lume` exactamente lo que hay en memoria.
pub fn encode_image_raw(img: &Image) -> Result<Vec<u8>, DomainError> {
    let data: Vec<u8> = img
        .pixels()
        .iter()
        .flat_map(|c| [c.r, c.g, c.b, c.a])
        .collect();
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, img.width(), img.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(png_error)?;
    writer.write_image_data(&data).map_err(png_error)?;
    writer.finish().map_err(png_error)?;
    Ok(out)
}

/// PNG de una imagen de animación como lo escribe pxcli (`anim.SavePNG`):
/// cada color pasa por [`pxcli_png_color`].
pub fn encode_anim_png(img: &Image) -> Result<Vec<u8>, DomainError> {
    let data: Vec<u8> = img
        .pixels()
        .iter()
        .flat_map(|&c| {
            let c = pxcli_png_color(c);
            [c.r, c.g, c.b, c.a]
        })
        .collect();
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, img.width(), img.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(png_error)?;
    writer.write_image_data(&data).map_err(png_error)?;
    writer.finish().map_err(png_error)?;
    Ok(out)
}

/// Escribe una imagen de animación (como `anim.SavePNG` de pxcli).
pub fn write_anim_png(path: &Path, img: &Image) -> Result<(), DomainError> {
    let bytes = encode_anim_png(img)?;
    fs::write(path, bytes).map_err(|err| io_error("open", path, &err))
}

/// Lee un archivo de texto (con el error de Go si falla).
pub fn read_text(path: &Path) -> Result<String, DomainError> {
    fs::read_to_string(path).map_err(|err| io_error("open", path, &err))
}

/// Exporta el lienzo a un archivo PNG.
pub fn write_png(canvas: &Canvas, path: &Path, scale: u32) -> Result<(), DomainError> {
    let bytes = encode_png(canvas, scale)?;
    fs::write(path, bytes).map_err(|err| io_error("open", path, &err))
}

/// Lee un archivo PNG como lienzo.
pub fn read_png(path: &Path) -> Result<Canvas, DomainError> {
    let bytes = fs::read(path).map_err(|err| io_error("open", path, &err))?;
    decode_png(&bytes)
}

/// Color con el que pxcli escribía un píxel en su PNG: Go trataba el color
/// como premultiplicado y lo «despremultiplicaba» con aritmética de 32 bits y
/// truncado a 8 bits (`image/png`, caso `cbTC8A` con `*image.RGBA`). Solo
/// sirve para comparar con los `final.png` de pxcli (banco de compatibilidad).
pub fn pxcli_png_color(c: lume_domain::Rgba) -> lume_domain::Rgba {
    match c.a {
        0 => lume_domain::Rgba::TRANSPARENT,
        255 => c,
        a => {
            const M: u32 = 0x101 * 0xffff;
            let a16 = u32::from(a) * 0x101;
            let ch = |v: u8| ((u32::from(v) * M / a16) >> 8) as u8;
            lume_domain::Rgba::new(ch(c.r), ch(c.g), ch(c.b), a)
        }
    }
}

fn png_error(err: impl std::fmt::Display) -> DomainError {
    DomainError::new(ErrorCode::Io, format!("png: {err}"))
}

/// PNG de una imagen RGBA sin premultiplicar (lo que da el rasterizador).
pub fn encode_rgba_png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, DomainError> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(png_error)?;
    writer.write_image_data(rgba).map_err(png_error)?;
    writer.finish().map_err(png_error)?;
    Ok(out)
}

/// Guarda una imagen RGBA como PNG (con el error de Go si falla).
pub fn write_rgba_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), DomainError> {
    let bytes = encode_rgba_png(width, height, rgba)?;
    write_bytes(path, &bytes)
}

/// Escribe un archivo (con el error de Go si falla).
pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), DomainError> {
    fs::write(path, bytes).map_err(|err| io_error("open", path, &err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_domain::Rgba;

    /// Lienzo con colores opacos, semitransparentes y transparentes.
    fn sample() -> Canvas {
        let mut c = Canvas::new(3, 2).unwrap();
        c.set(0, 0, Rgba::rgb(255, 0, 0)).unwrap();
        c.set(1, 0, Rgba::rgb(0, 255, 0)).unwrap();
        c.set(2, 0, Rgba::new(0xcf, 0xdd, 0xe3, 0xaa)).unwrap();
        c.set(0, 1, Rgba::rgb(0, 0, 255)).unwrap();
        c.set(1, 1, Rgba::TRANSPARENT).unwrap();
        c.set(2, 1, Rgba::new(10, 20, 30, 0)).unwrap();
        c
    }

    // Como TestCanvasExportPNG de pxcli, y además alfa intermedio y color con
    // alfa 0: se conserva cada byte.
    #[test]
    fn encode_then_decode_keeps_every_pixel() {
        let c = sample();
        let back = decode_png(&encode_png(&c, 1).unwrap()).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn scale_makes_solid_blocks() {
        let c = sample();
        let big = decode_png(&encode_png(&c, 10).unwrap()).unwrap();
        assert_eq!((big.width(), big.height()), (30, 20));
        for y in 0..20i64 {
            for x in 0..30i64 {
                assert_eq!(
                    big.get(x, y).unwrap(),
                    c.get(x / 10, y / 10).unwrap(),
                    "({x},{y})"
                );
            }
        }
    }

    #[test]
    fn invalid_scale() {
        let c = sample();
        for scale in [0, 65] {
            let err = encode_png(&c, scale).unwrap_err();
            assert_eq!(err.code, ErrorCode::InvalidArgs);
            assert_eq!(err.message, "scale must be between 1 and 64");
        }
        assert!(encode_png(&c, 64).is_ok());
    }

    #[test]
    fn decodes_other_color_types() {
        // RGB opaco de 8 bits.
        let mut rgb = Vec::new();
        {
            let mut e = png::Encoder::new(&mut rgb, 2, 1);
            e.set_color(png::ColorType::Rgb);
            e.set_depth(png::BitDepth::Eight);
            let mut w = e.write_header().unwrap();
            w.write_image_data(&[1, 2, 3, 4, 5, 6]).unwrap();
        }
        let c = decode_png(&rgb).unwrap();
        assert_eq!(c.get(1, 0).unwrap(), Rgba::rgb(4, 5, 6));

        // Gris con alfa de 16 bits: se toma el byte alto.
        let mut ga = Vec::new();
        {
            let mut e = png::Encoder::new(&mut ga, 1, 1);
            e.set_color(png::ColorType::GrayscaleAlpha);
            e.set_depth(png::BitDepth::Sixteen);
            let mut w = e.write_header().unwrap();
            w.write_image_data(&[0x12, 0x34, 0x80, 0xff]).unwrap();
        }
        assert_eq!(
            decode_png(&ga).unwrap().get(0, 0).unwrap(),
            Rgba::new(0x12, 0x12, 0x12, 0x80)
        );

        // Paleta.
        let mut idx = Vec::new();
        {
            let mut e = png::Encoder::new(&mut idx, 2, 1);
            e.set_color(png::ColorType::Indexed);
            e.set_depth(png::BitDepth::Eight);
            e.set_palette(vec![255, 0, 0, 0, 0, 255]);
            let mut w = e.write_header().unwrap();
            w.write_image_data(&[1, 0]).unwrap();
        }
        let c = decode_png(&idx).unwrap();
        assert_eq!(c.get(0, 0).unwrap(), Rgba::rgb(0, 0, 255));
        assert_eq!(c.get(1, 0).unwrap(), Rgba::rgb(255, 0, 0));
    }

    #[test]
    fn not_a_png_is_an_io_error() {
        let err = decode_png(b"hola").unwrap_err();
        assert_eq!(err.code, ErrorCode::Io);
        assert!(err.message.starts_with("png: "), "{}", err.message);
    }

    /// Valores obtenidos exportando con el `canvas.go` original de pxcli.
    #[test]
    fn pxcli_png_color_matches_go() {
        let cases = [
            ("#cfdde3aa", "#374c55aa"),
            ("#ff000080", "#fd000080"),
            ("#10203000", "#00000000"),
            ("#12345678", "#266eb778"),
            ("#0180fe01", "#ffffff01"),
            ("#abcdefff", "#abcdefff"),
        ];
        for (input, want) in cases {
            let got = pxcli_png_color(input.parse().unwrap());
            assert_eq!(got.to_string(), want, "{input}");
        }
    }
}
