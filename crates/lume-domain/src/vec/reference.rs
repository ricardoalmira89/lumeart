//! La imagen de referencia sobre el lienzo del dibujo (`reference.go` de
//! pxcli): escalada para caber, centrada, de modo que un punto leído en ella
//! es un punto de `dibujo.js`. `vec ref` la muestra con su rejilla y
//! `vec compare` la pone junto al dibujo o debajo de él.
//!
//! Aquí solo se compone el SVG; leer la imagen y rasterizar es cosa de
//! quien llama.

use super::raster::{grid_lines, num};

/// Prefijo de los archivos de referencia en la carpeta del dibujo.
pub const REFERENCE_PREFIX: &str = "referencia-";

/// El tipo de una imagen por sus primeros bytes (los que acepta pxcli:
/// PNG, JPEG, GIF y WebP), como `http.DetectContentType`.
pub fn sniff_image(head: &[u8]) -> Option<&'static str> {
    if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if head.len() >= 14 && &head[..4] == b"RIFF" && &head[8..14] == b"WEBPVP" {
        Some("image/webp")
    } else {
        None
    }
}

/// El error de un archivo que no es una imagen válida.
pub fn not_an_image(path: &str) -> String {
    format!("{path} no es una imagen PNG, JPG, GIF o WebP")
}

/// `<image>` con la referencia (ya en base64) encajada en `w` × `h`, con
/// opacidad si está entre 0 y 1.
pub fn image_element(mime: &str, base64: &str, w: u32, h: u32, opacity: f64) -> String {
    let op = if opacity > 0.0 && opacity < 1.0 {
        format!(r#" opacity="{}""#, num(opacity))
    } else {
        String::new()
    };
    format!(
        r#"<image x="0" y="0" width="{w}" height="{h}" preserveAspectRatio="xMidYMid meet"{op} href="data:{mime};base64,{base64}"/>"#
    )
}

/// El `<svg>` raíz de un dibujo convertido en uno colocado en (`x`, `y`)
/// dentro de otro.
pub fn nested(drawing: &str, x: u32, y: u32, w: u32, h: u32, extra: &str) -> String {
    let open =
        format!(r#"<svg x="{x}" y="{y}" width="{w}" height="{h}" viewBox="0 0 {w} {h}"{extra}>"#);
    match drawing.find('>') {
        Some(end) if drawing.starts_with("<svg") => format!("{open}{}", &drawing[end + 1..]),
        _ => drawing.to_string(),
    }
}

/// La referencia sola sobre el lienzo (`ReferencePNG`), en un fondo gris.
pub fn reference_svg(image: &str, w: u32, h: u32) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}"><rect width="{w}" height="{h}" fill="#202020"/>{image}</svg>"##
    )
}

/// Cómo se comparan el dibujo y la referencia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareMode {
    /// A la izquierda el dibujo, a la derecha la referencia.
    Side,
    /// El dibujo al 55 % sobre la referencia.
    Over,
}

impl CompareMode {
    /// `lado` (o nada) y `encima`.
    pub fn parse(mode: &str) -> Result<Self, String> {
        match mode {
            "" | "lado" => Ok(CompareMode::Side),
            "encima" => Ok(CompareMode::Over),
            other => Err(format!(
                "modo {} no válido (lado o encima)",
                crate::error::go_quote(other)
            )),
        }
    }
}

/// Lo que hay que rasterizar para comparar: el SVG, su tamaño en unidades,
/// el ancho de salida y el paso de la rejilla que pone el rasterizador.
#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    pub svg: String,
    pub width: u32,
    pub height: u32,
    pub out_width: u32,
    pub grid: Option<f64>,
}

/// `ComparePNG` de pxcli sin rasterizar. `out_width` 0: el de pxcli.
pub fn compare_svg(
    drawing: &str,
    image: &str,
    w: u32,
    h: u32,
    mode: CompareMode,
    grid: f64,
    out_width: u32,
) -> Comparison {
    let grid_svg = || grid_lines(0.0, 0.0, f64::from(w), f64::from(h), grid);
    match mode {
        CompareMode::Side => {
            let gap = (w / 30).max(8);
            let tw = 2 * w + gap;
            let mut b = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {tw} {h}" width="{tw}" height="{h}"><rect width="{tw}" height="{h}" fill="#202020"/>"##
            );
            b.push_str(&nested(drawing, 0, 0, w, h, ""));
            b.push_str(&format!(
                r#"<svg x="{}" y="0" width="{w}" height="{h}" viewBox="0 0 {w} {h}">{image}"#,
                w + gap
            ));
            if grid > 0.0 {
                b.push_str(&grid_svg());
            }
            b.push_str("</svg>");
            if grid > 0.0 {
                b.push_str(&format!(
                    r#"<svg x="0" y="0" width="{w}" height="{h}" viewBox="0 0 {w} {h}">{}</svg>"#,
                    grid_svg()
                ));
            }
            b.push_str("</svg>");
            Comparison {
                svg: b,
                width: tw,
                height: h,
                out_width: if out_width == 0 {
                    (2 * 1024).min(tw)
                } else {
                    out_width
                },
                grid: None,
            }
        }
        CompareMode::Over => {
            let mut b = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}"><rect width="{w}" height="{h}" fill="#202020"/>{image}"##
            );
            b.push_str(&nested(drawing, 0, 0, w, h, r#" opacity=".55""#));
            b.push_str("</svg>");
            Comparison {
                svg: b,
                width: w,
                height: h,
                out_width: if out_width == 0 { 1024 } else { out_width },
                grid: (grid > 0.0).then_some(grid),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_are_recognised_by_their_bytes() {
        assert_eq!(sniff_image(b"\x89PNG\r\n\x1a\nresto"), Some("image/png"));
        assert_eq!(sniff_image(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
        assert_eq!(sniff_image(b"GIF89a..."), Some("image/gif"));
        assert_eq!(sniff_image(b"RIFF\x10\0\0\0WEBPVP8 "), Some("image/webp"));
        assert_eq!(sniff_image(b"hola"), None);
        assert_eq!(
            not_an_image("referencia-1.txt"),
            "referencia-1.txt no es una imagen PNG, JPG, GIF o WebP"
        );
    }

    #[test]
    fn the_comparison_is_pxclis() {
        let drawing = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 200" width="300" height="200"><circle r="5"/></svg>"#;
        let image = image_element("image/png", "QUJD", 300, 200, 1.0);
        assert_eq!(
            image,
            r#"<image x="0" y="0" width="300" height="200" preserveAspectRatio="xMidYMid meet" href="data:image/png;base64,QUJD"/>"#
        );
        assert!(image_element("image/png", "x", 1, 1, 0.5).contains(r#" opacity="0.5""#));
        let side = compare_svg(drawing, &image, 300, 200, CompareMode::Side, 0.0, 0);
        assert_eq!((side.width, side.height, side.out_width), (610, 200, 610));
        assert!(side.svg.contains(r#"<svg x="0" y="0" width="300" height="200" viewBox="0 0 300 200"><circle r="5"/></svg>"#), "{}", side.svg);
        assert!(side.svg.contains(
            r#"<svg x="310" y="0" width="300" height="200" viewBox="0 0 300 200"><image"#
        ));
        assert!(!side.svg.contains("<line"));
        let gridded = compare_svg(drawing, &image, 300, 200, CompareMode::Side, 100.0, 0);
        // La rejilla, en las dos mitades.
        assert_eq!(gridded.svg.matches(">100</text>").count(), 4);
        let over = compare_svg(drawing, &image, 300, 200, CompareMode::Over, 50.0, 0);
        assert_eq!(
            (over.width, over.out_width, over.grid),
            (300, 1024, Some(50.0))
        );
        assert!(
            over.svg
                .contains(r#"viewBox="0 0 300 200" opacity=".55"><circle"#)
        );
        assert_eq!(CompareMode::parse(""), Ok(CompareMode::Side));
        assert_eq!(
            CompareMode::parse("arriba"),
            Err("modo \"arriba\" no válido (lado o encima)".into())
        );
    }
}
