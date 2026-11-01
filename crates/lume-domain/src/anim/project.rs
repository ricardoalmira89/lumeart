//! Una animación nueva a partir de un dibujo base (`internal/anim/project.go`)
//! y los nombres de archivo de sus exportaciones.

use super::recipe::{clean_recipe, paint, parse_recipe, recipe_from_image};
use super::skeleton::{SkeletonOptions, origin, skeleton};
use super::{ActionSpec, Image, MAX_ACTIONS, MAX_CELL, MAX_FRAMES, same_image};
use crate::error::go_quote;

/// Qué animación crear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOptions {
    pub name: String,
    /// La base, como la carga pxcli (alfa premultiplicado).
    pub base: Image,
    /// Los comandos de pxcli que dibujaron la base, si se saben.
    pub recipe_lines: Vec<String>,
    /// Tamaño de la celda; `(0, 0)` = el de la base.
    pub cell: (u32, u32),
    /// Dónde va la base en una celda más grande: `bottom` (por defecto) o
    /// `center`.
    pub anchor: String,
    pub actions: Vec<ActionSpec>,
}

/// El `anim.js` de partida y una nota de cómo se hizo la receta. Usa los
/// comandos originales si reproducen la base exactamente; si no, rehace la
/// receta desde los píxeles.
pub fn new_project(mut o: ProjectOptions) -> Result<(String, String), String> {
    let bsize = o.base.size();
    if o.cell == (0, 0) {
        o.cell = bsize;
    }
    if o.cell.0 < bsize.0 || o.cell.1 < bsize.1 {
        return Err(format!(
            "la celda ({}x{}) no puede ser más pequeña que la base ({}x{})",
            o.cell.0, o.cell.1, bsize.0, bsize.1
        ));
    }
    if o.cell.0 > MAX_CELL || o.cell.1 > MAX_CELL {
        return Err(format!("la celda no puede pasar de {MAX_CELL}x{MAX_CELL}"));
    }
    if o.actions.is_empty() {
        o.actions = vec![ActionSpec {
            name: "anim".into(),
            frames: 8,
            fps: 8,
            looping: true,
        }];
    }
    if o.actions.len() > MAX_ACTIONS {
        return Err(format!("como mucho {MAX_ACTIONS} acciones"));
    }
    let mut seen = std::collections::HashSet::new();
    for a in &o.actions {
        if a.name.is_empty() || !seen.insert(a.name.clone()) {
            return Err(format!(
                "nombre de acción vacío o repetido: {}",
                go_quote(&a.name)
            ));
        }
        if a.frames < 1 || a.frames > MAX_FRAMES || a.fps < 1 || a.fps > 60 {
            return Err(format!(
                "{}: frames entre 1 y {MAX_FRAMES}, fps entre 1 y 60",
                a.name
            ));
        }
    }
    if o.name.trim().is_empty() {
        o.name = "anim".into();
    }

    let mut recipe = None;
    if !o.recipe_lines.is_empty()
        && let Ok(cmds) = parse_recipe(&o.recipe_lines)
    {
        let cmds = clean_recipe(&cmds, bsize.0, bsize.1);
        let mut img = Image::new(bsize.0, bsize.1);
        paint(&mut img, &cmds, None);
        if same_image(&img, &o.base) {
            recipe = Some(cmds);
        }
    }
    let from_pixels = recipe.is_none();
    let recipe = recipe.unwrap_or_else(|| recipe_from_image(&o.base));
    let note = if from_pixels {
        format!(
            "receta reconstruida desde los píxeles, {} comandos",
            recipe.len()
        )
    } else {
        format!(
            "receta con los {} comandos originales que quedan visibles",
            recipe.len()
        )
    };
    let src = skeleton(&SkeletonOptions {
        name: o.name,
        base: "base.png".into(),
        cell: o.cell,
        origin: origin(bsize, o.cell, &o.anchor),
        recipe,
        from_pixels,
        actions: o.actions,
    });
    Ok((src, note))
}

/// Un nombre para archivos: minúsculas ASCII, cifras y guiones (`SafeName`
/// de pxcli). Vacío: `anim`.
pub fn safe_name(s: &str) -> String {
    let lower = s.trim().to_lowercase();
    let mut out = String::new();
    let mut dash = false;
    for c in lower.chars() {
        let c = match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            'ñ' => 'n',
            other => other,
        };
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() { "anim".into() } else { out }
}

/// `--action nombre:frames:fps[:once]` de `anim init` (pxcli).
pub fn parse_action_flags<S: AsRef<str>>(flags: &[S]) -> Result<Vec<ActionSpec>, String> {
    let mut specs = Vec::new();
    for flag in flags {
        let flag = flag.as_ref();
        let parts: Vec<&str> = flag.split(':').collect();
        let mut spec = ActionSpec {
            name: parts[0].trim().to_string(),
            frames: 8,
            fps: 8,
            looping: true,
        };
        if spec.name.is_empty() {
            return Err(format!("acción sin nombre en {}", go_quote(flag)));
        }
        if let Some(n) = parts.get(1) {
            spec.frames =
                parse_count(n).ok_or_else(|| format!("frames no válidos en {}", go_quote(flag)))?;
        }
        if let Some(n) = parts.get(2) {
            spec.fps =
                parse_count(n).ok_or_else(|| format!("fps no válidos en {}", go_quote(flag)))?;
        }
        if parts.get(3) == Some(&"once") {
            spec.looping = false;
        }
        specs.push(spec);
    }
    Ok(specs)
}

/// Un entero como `strconv.Atoi`; los negativos quedan en 0 (y luego fallan
/// en la validación de la acción).
fn parse_count(text: &str) -> Option<u32> {
    let n: i64 = text.parse().ok()?;
    Some(n.clamp(0, i64::from(u32::MAX)) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_names() {
        assert_eq!(safe_name("Reloj de Arena"), "reloj-de-arena");
        assert_eq!(safe_name("  ¡Poción mágica! 2 "), "pocion-magica-2");
        assert_eq!(safe_name("ÉPICA"), "epica");
        assert_eq!(safe_name("??"), "anim");
        assert_eq!(safe_name(""), "anim");
    }

    #[test]
    fn action_flags() {
        let specs = parse_action_flags(&["andar:6:10", "saltar:4:8:once", "quieto"]).unwrap();
        assert_eq!(specs[0].frames, 6);
        assert_eq!(specs[0].fps, 10);
        assert!(!specs[1].looping);
        assert_eq!(
            (specs[2].frames, specs[2].fps, specs[2].looping),
            (8, 8, true)
        );
        assert_eq!(
            parse_action_flags(&[":3"]).unwrap_err(),
            "acción sin nombre en \":3\""
        );
        assert_eq!(
            parse_action_flags(&["a:x"]).unwrap_err(),
            "frames no válidos en \"a:x\""
        );
        assert_eq!(
            parse_action_flags(&["a:2:y"]).unwrap_err(),
            "fps no válidos en \"a:2:y\""
        );
    }
}
