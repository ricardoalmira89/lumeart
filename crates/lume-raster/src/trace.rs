//! El calco de la referencia con el algoritmo de vtracer (su motor,
//! `visioncortex`), en vez del Image Trace de VectorCraft que usaba pxcli.
//!
//! 1. La referencia se encaja y centra en el lienzo (como `vec ref`):
//!    resvg la decodifica (PNG, JPEG, GIF o WebP) y la escala.
//! 2. En color, se reduce a los colores pedidos (median cut afinado con
//!    unas vueltas de k-means, siempre igual para la misma imagen) y se
//!    traza en regiones que no se solapan; en «lines», lo más oscuro que el
//!    umbral es tinta y se trazan solo sus manchas.
//! 3. Cada región es un `<path d="…" fill="#rrggbb" transform="…"/>` en
//!    coordenadas del dibujo. Quitar o ajustar colores es cosa de
//!    `lume_domain::vec::trace::post_trace`.

use std::collections::HashMap;

use lume_app::ports::PortError;
use lume_domain::vec::trace::TraceOptions;
use visioncortex::{BinaryImage, ColorImage, CompoundPath, PathSimplifyMode, PointF64};

use crate::render;

/// Lado máximo con que se traza (una referencia más grande se traza a esta
/// escala y los trazos se amplían).
const MAX_TRACE_SIDE: u32 = 2048;

/// La referencia encajada en un lienzo de `width` × `height` (escalado a
/// `scale`), como RGBA sin premultiplicar.
fn place(
    image: &[u8],
    mime: &str,
    width: u32,
    height: u32,
    scale: f64,
) -> Result<Vec<u8>, PortError> {
    let (w, h) = (
        (f64::from(width) * scale).round().max(1.0) as u32,
        (f64::from(height) * scale).round().max(1.0) as u32,
    );
    let data = base64(image);
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{w}" height="{h}"><image x="0" y="0" width="{width}" height="{height}" preserveAspectRatio="xMidYMid meet" href="data:{mime};base64,{data}"/></svg>"#
    );
    Ok(render(&svg, w, h, None)?.rgba)
}

/// Base64 estándar (para incrustar la imagen en el SVG).
fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(ABC[(n >> 18) as usize & 63] as char);
        out.push(ABC[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ABC[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ABC[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Un píxel opaco (o casi) cuenta; los demás son el fondo transparente.
fn opaque(px: &[u8]) -> bool {
    px[3] >= 128
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// Los colores distintos de los píxeles opacos, con cuántos hay de cada
/// uno, en un orden fijo.
fn histogram(rgba: &[u8]) -> Vec<([u8; 3], u64)> {
    let mut counts: HashMap<[u8; 3], u64> = HashMap::new();
    for px in rgba.chunks_exact(4).filter(|p| opaque(p)) {
        *counts.entry([px[0], px[1], px[2]]).or_default() += 1;
    }
    let mut out: Vec<([u8; 3], u64)> = counts.into_iter().collect();
    out.sort_unstable();
    out
}

/// Hasta `k` colores que representan bien a `colors`: k-means, empezando
/// por el color más abundante y añadiendo cada vez el que más pesa por su
/// distancia a los ya elegidos (k-means++ sin azar, siempre igual).
fn palette(colors: &[([u8; 3], u64)], k: usize) -> Vec<[f64; 3]> {
    if colors.is_empty() {
        return Vec::new();
    }
    let points: Vec<([f64; 3], f64)> = colors
        .iter()
        .map(|(c, n)| {
            (
                [f64::from(c[0]), f64::from(c[1]), f64::from(c[2])],
                *n as f64,
            )
        })
        .collect();
    let first = points
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.1.total_cmp(&b.1.1).then(b.0.cmp(&a.0)))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let mut centers = vec![points[first].0];
    let mut best_d: Vec<f64> = points.iter().map(|(p, _)| dist2(*p, centers[0])).collect();
    while centers.len() < k {
        let pick = points
            .iter()
            .enumerate()
            .map(|(i, (_, n))| (i, n * best_d[i]))
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)));
        match pick {
            Some((i, score)) if score > 0.0 => {
                let c = points[i].0;
                for (d, (p, _)) in best_d.iter_mut().zip(&points) {
                    *d = d.min(dist2(*p, c));
                }
                centers.push(c);
            }
            _ => break,
        }
    }
    for _ in 0..10 {
        let mut sums = vec![([0.0; 3], 0.0); centers.len()];
        for (p, n) in &points {
            let best = nearest(&centers, *p);
            for (sum, v) in sums[best].0.iter_mut().zip(p) {
                *sum += v * n;
            }
            sums[best].1 += n;
        }
        let mut moved = false;
        for (center, (sum, n)) in centers.iter_mut().zip(sums) {
            if n > 0.0 {
                let next = sum.map(|s| s / n);
                moved |= dist2(next, *center) > 1e-6;
                *center = next;
            }
        }
        if !moved {
            break;
        }
    }
    centers
}

fn nearest(centers: &[[f64; 3]], p: [f64; 3]) -> usize {
    let mut best = 0;
    let mut best_d = f64::INFINITY;
    for (i, c) in centers.iter().enumerate() {
        let d = dist2(*c, p);
        if d < best_d {
            best = i;
            best_d = d;
        }
    }
    best
}

/// Índice de un píxel transparente (fuera de la referencia).
const EMPTY: u8 = u8::MAX;

/// Cada píxel opaco con el índice del color más cercano de la paleta (los
/// demás, [`EMPTY`]), y la paleta.
fn quantize(rgba: &[u8], colors: usize) -> (Vec<u8>, Vec<[u8; 3]>) {
    let hist = histogram(rgba);
    let centers = palette(&hist, colors.clamp(1, 64));
    let rounded: Vec<[u8; 3]> = centers
        .iter()
        .map(|c| c.map(|v| v.round().clamp(0.0, 255.0) as u8))
        .collect();
    let lookup: HashMap<[u8; 3], u8> = hist
        .iter()
        .map(|(c, _)| {
            let p = [f64::from(c[0]), f64::from(c[1]), f64::from(c[2])];
            (*c, nearest(&centers, p) as u8)
        })
        .collect();
    let index = rgba
        .chunks_exact(4)
        .map(|px| {
            if opaque(px) {
                lookup[&[px[0], px[1], px[2]]]
            } else {
                EMPTY
            }
        })
        .collect();
    (index, rounded)
}

/// Las manchas de menos de `min_area` píxeles pasan al color vecino que
/// más las rodea (para no dejar huecos al quitarlas).
fn despeckle(index: &mut [u8], w: usize, h: usize, min_area: usize) {
    if min_area <= 1 {
        return;
    }
    let mut seen = vec![false; index.len()];
    let mut stack = Vec::new();
    let mut members = Vec::new();
    for start in 0..index.len() {
        if seen[start] || index[start] == EMPTY {
            continue;
        }
        let color = index[start];
        members.clear();
        stack.push(start);
        seen[start] = true;
        let mut around: HashMap<u8, usize> = HashMap::new();
        while let Some(i) = stack.pop() {
            members.push(i);
            let (x, y) = (i % w, i / w);
            let neighbours = [
                (x > 0).then(|| i - 1),
                (x + 1 < w).then(|| i + 1),
                (y > 0).then(|| i - w),
                (y + 1 < h).then(|| i + w),
            ];
            for n in neighbours.into_iter().flatten() {
                if index[n] == color {
                    if !seen[n] {
                        seen[n] = true;
                        stack.push(n);
                    }
                } else if index[n] != EMPTY {
                    *around.entry(index[n]).or_default() += 1;
                }
            }
        }
        if members.len() < min_area
            && let Some((&to, _)) = around
                .iter()
                .max_by_key(|(c, n)| (**n, std::cmp::Reverse(**c)))
        {
            for &i in &members {
                index[i] = to;
            }
        }
    }
}

/// Los parámetros de vtracer que salen de las opciones de pxcli.
struct Fit {
    speckle_area: usize,
    corner: f64,
    length: f64,
    splice: f64,
}

fn fit(job: &TraceOptions, scale: f64) -> Fit {
    let deg = |d: f64| d / 180.0 * std::f64::consts::PI;
    let noise = (job.noise * scale).max(0.0);
    Fit {
        // Las manchas de menos de `noise` píxeles de lado.
        speckle_area: (noise * noise).round() as usize,
        // Más «esquinas», umbral de ángulo más bajo (50 → 60°, el de vtracer).
        corner: deg((120.0 - 1.2 * job.corners.clamp(0.0, 100.0)).clamp(0.0, 180.0)),
        // Más fidelidad, segmentos más cortos (de 10 a 3,5 px; 60 → 6,1).
        length: 10.0 - 6.5 * job.paths.clamp(0.0, 100.0) / 100.0,
        splice: deg(45.0),
    }
}

/// Un trazo de vtracer en coordenadas del dibujo.
fn path_element(path: &CompoundPath, color: [u8; 3], scale: f64) -> String {
    let (d, offset) = path.to_svg_string(true, PointF64::default(), Some(2));
    let hex = format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]);
    if (scale - 1.0).abs() < 1e-9 {
        format!(
            r#"<path d="{d}" fill="{hex}" transform="translate({},{})"/>"#,
            offset.x, offset.y
        )
    } else {
        let k = 1.0 / scale;
        format!(
            r#"<path d="{d}" fill="{hex}" transform="scale({k}) translate({},{})"/>"#,
            offset.x, offset.y
        )
    }
}

/// El calco de `image` (`mime`) sobre un lienzo de `job.width` ×
/// `job.height`: los trazos tal como salen, sin quitar ni ajustar colores.
pub fn trace_reference(image: &[u8], mime: &str, job: &TraceOptions) -> Result<String, PortError> {
    let side = job.width.max(job.height).max(1);
    let scale = if side > MAX_TRACE_SIDE {
        f64::from(MAX_TRACE_SIDE) / f64::from(side)
    } else {
        1.0
    };
    let rgba = place(image, mime, job.width, job.height, scale)?;
    let (w, h) = (
        (f64::from(job.width) * scale).round().max(1.0) as usize,
        (f64::from(job.height) * scale).round().max(1.0) as usize,
    );
    let fit = fit(job, scale);
    let mut out = String::new();
    if job.lines {
        // Tinta: lo opaco más oscuro que el umbral.
        let threshold = f64::from(job.threshold.min(255));
        let mut pixels = vec![255u8; w * h * 4];
        for (dst, px) in pixels.chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
            let lum =
                0.2126 * f64::from(px[0]) + 0.7152 * f64::from(px[1]) + 0.0722 * f64::from(px[2]);
            if opaque(px) && lum < threshold {
                dst.copy_from_slice(&[0, 0, 0, 255]);
            }
        }
        let img = ColorImage {
            pixels,
            width: w,
            height: h,
        };
        let binary = img.to_binary_image(|c| c.r < 128);
        let clusters = binary.to_clusters(false);
        for i in 0..clusters.len() {
            let cluster = clusters.get_cluster(i);
            if cluster.size() >= fit.speckle_area {
                let path = cluster.to_compound_path(
                    PathSimplifyMode::Spline,
                    fit.corner,
                    fit.length,
                    10,
                    fit.splice,
                );
                out.push_str(&path_element(&path, [0, 0, 0], scale));
            }
        }
        return Ok(out);
    }
    let (mut index, colors) = quantize(&rgba, job.colors.clamp(2, 64) as usize);
    despeckle(&mut index, w, h, fit.speckle_area);
    // Regiones que no se solapan («abutting» en VectorCraft): cada color con
    // sus manchas, primero los que más ocupan.
    let mut area = vec![0usize; colors.len()];
    for &i in &index {
        if i != EMPTY {
            area[i as usize] += 1;
        }
    }
    let mut order: Vec<usize> = (0..colors.len()).filter(|&c| area[c] > 0).collect();
    order.sort_by_key(|&c| (std::cmp::Reverse(area[c]), c));
    for c in order {
        let mut mask = BinaryImage::new_w_h(w, h);
        for (i, &v) in index.iter().enumerate() {
            if v as usize == c {
                mask.set_pixel(i % w, i / w, true);
            }
        }
        let clusters = mask.to_clusters(false);
        for k in 0..clusters.len() {
            let cluster = clusters.get_cluster(k);
            if cluster.size() == 0 {
                continue;
            }
            let path = cluster.to_compound_path(
                PathSimplifyMode::Spline,
                fit.corner,
                fit.length,
                10,
                fit.splice,
            );
            out.push_str(&path_element(&path, colors[c], scale));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_palette_keeps_the_main_colours() {
        let colors = vec![
            ([250, 10, 10], 100),
            ([240, 20, 15], 80),
            ([10, 10, 240], 120),
            ([20, 30, 230], 60),
            ([250, 250, 250], 300),
        ];
        let p = palette(&colors, 3);
        assert_eq!(p.len(), 3);
        for want in [
            [245.0, 15.0, 12.0],
            [14.0, 17.0, 236.0],
            [250.0, 250.0, 250.0],
        ] {
            assert!(p.iter().any(|c| dist2(*c, want) < 100.0), "{p:?}");
        }
        assert!(palette(&[], 4).is_empty());
        assert_eq!(base64(b"hola"), "aG9sYQ==");
        assert_eq!(base64(b"abc"), "YWJj");
    }
}
