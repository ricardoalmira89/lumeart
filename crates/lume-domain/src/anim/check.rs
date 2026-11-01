//! Comprobaciones de una animación dibujada (`internal/anim/check.go`): el
//! reposo es la base, los colores salen de su paleta, y cada acción se mueve,
//! cabe en su celda y cierra bien el bucle.

use std::collections::{BTreeMap, HashSet};

use crate::color::Rgba;

use super::{AnimResult, Image, Level, diff_count, hex, palette, same_image};

/// Añade al resultado los avisos y errores de las comprobaciones.
pub fn check(res: &mut AnimResult) {
    if !res.errors().is_empty() && res.actions.is_empty() {
        return; // el script ni siquiera cargó
    }

    if let Some(base) = &res.base {
        match &res.rest {
            None => res.issue(
                Level::Warning,
                0,
                "reposo",
                "falta rest(): sin pose de reposo no se puede comprobar que las piezas reproducen la base",
            ),
            Some(rest) => {
                let expected = place_base(res, base);
                if !same_image(&expected, rest) {
                    let (n, x, y, want, got) = first_diff(&expected, rest);
                    let msg = format!(
                        "la pose de reposo no coincide con la base en {n} píxeles (por ejemplo en {},{}: debería ser {} y es {})",
                        x - res.origin.0,
                        y - res.origin.1,
                        hex(want),
                        hex(got)
                    );
                    res.issue(Level::Error, 0, "reposo", msg);
                }
            }
        }
    }

    let mut allowed: HashSet<Rgba> = HashSet::new();
    if let Some(base) = &res.base {
        allowed.extend(palette(base));
        allowed.extend(res.colors.iter().copied());
    }

    let mut found = Vec::new();
    for a in &res.actions {
        if a.images.is_empty() {
            continue;
        }
        let (line, name) = (a.line, a.spec.name.clone());
        let mut add = |level: Level, msg: String| found.push((level, line, name.clone(), msg));

        // Colores que no están en la paleta de la base.
        if res.base.is_some() {
            let mut extra: BTreeMap<String, usize> = BTreeMap::new();
            for (f, img) in a.images.iter().enumerate() {
                for c in palette(img) {
                    if !allowed.contains(&c) {
                        extra.entry(hex(c)).or_insert(f);
                    }
                }
            }
            if !extra.is_empty() {
                let mut list: Vec<String> =
                    extra.iter().map(|(h, f)| format!("{h} (f{f})")).collect();
                list.sort();
                if list.len() > 6 {
                    list.truncate(6);
                    list.push("…".into());
                }
                add(
                    Level::Warning,
                    format!(
                        "usa colores que no están en la base: {}. Si son a propósito, decláralos en config({{ colors: [...] }})",
                        list.join(", ")
                    ),
                );
            }
        }

        // Píxeles fuera de la celda.
        let clipped: Vec<String> = a
            .out_of_bounds
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(f, _)| format!("f{f}"))
            .collect();
        let total: usize = a.out_of_bounds.iter().sum();
        if total > 0 {
            add(
                Level::Warning,
                format!(
                    "{total} píxeles se salen de la celda ({}): agranda la celda o recoloca el movimiento",
                    clipped.join(", ")
                ),
            );
        }

        // Fotogramas vacíos y acciones que no se mueven.
        for (f, img) in a.images.iter().enumerate() {
            if palette(img).is_empty() {
                add(Level::Warning, format!("el frame {f} está vacío"));
            }
        }
        let frames = a.spec.frames as usize;
        if frames > 1 && a.images.len() >= frames {
            let moves = (1..frames).any(|f| !same_image(&a.images[f], &a.images[0]));
            if !moves {
                add(
                    Level::Warning,
                    "no se mueve: todos los frames son iguales".into(),
                );
            }
        }

        // Un bucle no debería dar un salto al volver al principio.
        if a.spec.looping && frames >= 3 && a.images.len() >= frames {
            let sum: usize = (0..frames - 1)
                .map(|f| diff_count(&a.images[f], &a.images[f + 1]))
                .sum();
            let avg = sum as f64 / (frames - 1) as f64;
            let wrap = diff_count(&a.images[frames - 1], &a.images[0]);
            if wrap as f64 > 2.0 * avg + 4.0 {
                add(
                    Level::Warning,
                    format!(
                        "el bucle da un salto al volver al principio: del último frame al primero cambian {wrap} píxeles y entre frames normalmente {avg:.0}"
                    ),
                );
            }
        }
    }
    for (level, line, place, msg) in found {
        res.issue(level, line, place, msg);
    }
}

/// La base dentro de una celda, en el origen configurado.
pub fn place_base(res: &AnimResult, base: &Image) -> Image {
    let mut out = Image::new(res.cell.0, res.cell.1);
    for y in 0..i64::from(base.height()) {
        for x in 0..i64::from(base.width()) {
            let p = base.get(x, y);
            if p.a > 0 {
                out.set(x + res.origin.0, y + res.origin.1, p);
            }
        }
    }
    out
}

/// Cuántos píxeles difieren y el primero (x, y, el esperado y el que hay).
fn first_diff(a: &Image, b: &Image) -> (usize, i64, i64, Rgba, Rgba) {
    let (mut n, mut fx, mut fy) = (0, -1, -1);
    let (mut want, mut got) = (Rgba::TRANSPARENT, Rgba::TRANSPARENT);
    for y in 0..i64::from(a.height()) {
        for x in 0..i64::from(a.width()) {
            let (pa, pb) = (a.get(x, y), b.get(x, y));
            if pa != pb {
                if n == 0 {
                    (fx, fy, want, got) = (x, y, pa, pb);
                }
                n += 1;
            }
        }
    }
    (n, fx, fy, want, got)
}

/// «Celda 12x12. Fila 1: reposo. Fila 2: andar (6 frames, 8 fps, bucle).»
/// (como `pxcli anim check`).
pub fn legend(res: &AnimResult) -> String {
    let mut text = format!("Celda {}x{}.", res.cell.0, res.cell.1);
    let mut row = 1;
    if res.rest.is_some() {
        text.push_str(&format!(" Fila {row}: reposo."));
        row += 1;
    }
    for a in &res.actions {
        let kind = if a.spec.looping { "bucle" } else { "una vez" };
        text.push_str(&format!(
            " Fila {row}: {} ({} frames, {} fps, {kind}).",
            a.spec.name, a.spec.frames, a.spec.fps
        ));
        row += 1;
    }
    text
}

/// Una línea por problema («aviso (línea 3, andar): …», «ERROR (reposo): …»)
/// o «Sin problemas.», y cuántos errores hay.
pub fn issue_lines(res: &AnimResult) -> (Vec<String>, usize) {
    let mut errors = 0;
    let mut lines = Vec::new();
    for issue in &res.issues {
        let label = if issue.level == Level::Error {
            errors += 1;
            "ERROR"
        } else {
            "aviso"
        };
        let mut place = String::new();
        if issue.line > 0 {
            place.push_str(&format!("línea {}", issue.line));
        }
        if !issue.place.is_empty() {
            if !place.is_empty() {
                place.push_str(", ");
            }
            place.push_str(&issue.place);
        }
        if !place.is_empty() {
            place = format!(" ({place})");
        }
        lines.push(format!("{label}{place}: {}", issue.msg));
    }
    if res.issues.is_empty() {
        lines.push("Sin problemas.".into());
    }
    (lines, errors)
}
