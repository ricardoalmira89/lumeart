//! Hojas de una animación (`internal/anim/sheet.go`): la hoja para el juego,
//! una tira por acción y la hoja para revisar.

use crate::color::Rgba;

use super::{Action, AnimResult, Image};

/// Copia `src` en `dst` desde (x0, y0), ampliada `scale` veces.
fn blit(dst: &mut Image, src: &Image, x0: i64, y0: i64, scale: u32) {
    let s = i64::from(scale);
    for y in 0..i64::from(src.height()) * s {
        for x in 0..i64::from(src.width()) * s {
            dst.set(x0 + x, y0 + y, src.get(x / s, y / s));
        }
    }
}

/// Como [`blit`], pero solo los píxeles visibles (se ven los cuadros).
fn blit_over(dst: &mut Image, src: &Image, x0: i64, y0: i64, scale: u32) {
    let s = i64::from(scale);
    for y in 0..i64::from(src.height()) * s {
        for x in 0..i64::from(src.width()) * s {
            let p = src.get(x / s, y / s);
            if p.a > 0 {
                dst.set(x0 + x, y0 + y, p);
            }
        }
    }
}

fn fill(img: &mut Image, x0: i64, y0: i64, x1: i64, y1: i64, c: Rgba) {
    for y in y0.max(0)..y1.min(i64::from(img.height())) {
        for x in x0.max(0)..x1.min(i64::from(img.width())) {
            img.set(x, y, c);
        }
    }
}

/// Todas las acciones en una imagen: una fila por acción y una columna por
/// fotograma, sin huecos. Las filas más cortas dejan celdas transparentes.
pub fn sheet(res: &AnimResult, scale: u32) -> Image {
    let scale = scale.max(1);
    let cols = res.actions.iter().map(|a| a.spec.frames).max().unwrap_or(0);
    let (cw, ch) = (res.cell.0 * scale, res.cell.1 * scale);
    let mut out = Image::new(cols.max(1) * cw, (res.actions.len() as u32).max(1) * ch);
    for (row, a) in res.actions.iter().enumerate() {
        for (f, img) in a.images.iter().enumerate() {
            blit(
                &mut out,
                img,
                f as i64 * i64::from(cw),
                row as i64 * i64::from(ch),
                scale,
            );
        }
    }
    out
}

/// Una acción en una sola fila.
pub fn strip(action: &Action, cell: (u32, u32), scale: u32) -> Image {
    let scale = scale.max(1);
    let mut out = Image::new(action.spec.frames.max(1) * cell.0 * scale, cell.1 * scale);
    for (f, img) in action.images.iter().enumerate() {
        blit(
            &mut out,
            img,
            f as i64 * i64::from(cell.0 * scale),
            0,
            scale,
        );
    }
    out
}

/// La hoja para mirar: fondo de cuadros (uno por píxel de la celda), un
/// hueco entre celdas y el reposo como primera fila.
pub fn review(res: &AnimResult, scale: u32) -> Image {
    let scale = scale.max(1);
    let mut rows: Vec<Vec<&Image>> = Vec::new();
    if let Some(rest) = &res.rest {
        rows.push(vec![rest]);
    }
    for a in &res.actions {
        rows.push(a.images.iter().collect());
    }
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0).max(1) as i64;
    let gap = i64::from((scale / 2).max(2));
    let (cw, ch) = (i64::from(res.cell.0 * scale), i64::from(res.cell.1 * scale));
    let width = cols * (cw + gap) + gap;
    let height = (rows.len() as i64).max(1) * (ch + gap) + gap;
    let mut out = Image::new(width as u32, height as u32);
    out.fill(Rgba::rgb(70, 74, 82));
    let (light, dark) = (Rgba::rgb(205, 208, 214), Rgba::rgb(178, 182, 190));
    let s = i64::from(scale);
    for (ri, row) in rows.iter().enumerate() {
        for (ci, img) in row.iter().enumerate() {
            let (x0, y0) = (gap + ci as i64 * (cw + gap), gap + ri as i64 * (ch + gap));
            for y in 0..i64::from(res.cell.1) {
                for x in 0..i64::from(res.cell.0) {
                    let c = if (x + y) % 2 == 1 { dark } else { light };
                    fill(
                        &mut out,
                        x0 + x * s,
                        y0 + y * s,
                        x0 + (x + 1) * s,
                        y0 + (y + 1) * s,
                        c,
                    );
                }
            }
            blit_over(&mut out, img, x0, y0, scale);
        }
    }
    out
}
