//! `lumeart anim init|check|sheet|frame|export`, como `pxcli anim`
//! (`internal/cli/anim.go`): mismos argumentos, salidas y errores. Trabaja
//! con archivos y no necesita una sesión abierta. Además, `export --json` y
//! `export --godot` guardan los datos de la hoja y un `SpriteFrames` de
//! Godot 4.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use lume_app::AnimEngine;
use lume_app::ports::BaseLoader;
use lume_cli::{AnimArgs, AnimCommand, AnimExportArgs, AnimFrameArgs, AnimInitArgs, AnimSheetArgs};
use lume_domain::anim::check::check;
use lume_domain::anim::export::{sheet_meta, sprite_frames_tres};
use lume_domain::anim::project::{ProjectOptions, new_project, parse_action_flags, safe_name};
use lume_domain::anim::sheet::{review, sheet, strip};
use lume_domain::anim::{Action, ActionSpec, AnimResult, Image};
use lume_script::QuickJsAnim;

/// Ejecuta un subcomando de `anim`. Devuelve el código de salida.
pub fn run(args: &AnimArgs, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let result = match &args.command {
        AnimCommand::Init(a) => init(a, stdout),
        AnimCommand::Check(a) => check_cmd(&a.file, stdout),
        AnimCommand::Sheet(a) => sheet_cmd(a, stdout),
        AnimCommand::Frame(a) => frame_cmd(a),
        AnimCommand::Export(a) => export_cmd(a, stdout),
    };
    match result {
        Ok(()) => 0,
        Err(e) => {
            let _ = writeln!(stderr, "Error: {e}");
            1
        }
    }
}

/// `filepath.Dir` de Go: la carpeta, o `.` si no hay.
fn go_dir(path: &str) -> String {
    match Path::new(path).parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_string_lossy().into_owned(),
        _ => ".".into(),
    }
}

/// `filepath.Join(dir, name)` de Go (sin `./` delante).
fn go_join(dir: &str, name: &str) -> String {
    if dir == "." {
        name.to_string()
    } else {
        Path::new(dir).join(name).to_string_lossy().into_owned()
    }
}

/// `filepath.Base` de Go.
fn go_base(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// El nombre de archivo sin su extensión.
fn stem(path: &str) -> String {
    Path::new(&go_base(path))
        .file_stem()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Ejecuta `anim.js` desde el disco; la base tiene que estar en su carpeta.
fn render_file(path: &str) -> Result<AnimResult, String> {
    let source = lume_store::read_text(Path::new(path)).map_err(|e| e.message)?;
    let dir = go_dir(path);
    let loader: BaseLoader = Arc::new(move |name: &str| {
        if name != go_base(name) || name.starts_with('.') {
            return Err("la base tiene que estar en la misma carpeta que anim.js".into());
        }
        lume_store::read_anim_image(Path::new(&go_join(&dir, name))).map_err(|e| e.message)
    });
    Ok(QuickJsAnim.render(&source, loader))
}

/// `anim.js` dibujado y comprobado.
fn load(path: &str) -> Result<AnimResult, String> {
    let mut res = render_file(path)?;
    check(&mut res);
    Ok(res)
}

fn legend(w: &mut dyn Write, res: &AnimResult) {
    let _ = writeln!(w, "{}", lume_domain::anim::check::legend(res));
}

/// Los problemas, uno por línea; error si hay errores.
fn issues(w: &mut dyn Write, res: &AnimResult) -> Result<(), String> {
    let (lines, errors) = lume_domain::anim::check::issue_lines(res);
    for line in lines {
        let _ = writeln!(w, "{line}");
    }
    if errors > 0 {
        return Err(format!("{errors} errores"));
    }
    Ok(())
}

fn scale_of(scale: i64) -> u32 {
    scale.clamp(1, i64::from(u32::MAX)) as u32
}

fn save(path: &str, img: &Image) -> Result<(), String> {
    lume_store::write_anim_png(Path::new(path), img).map_err(|e| e.message)
}

fn check_cmd(file: &str, out: &mut dyn Write) -> Result<(), String> {
    let res = load(file)?;
    legend(out, &res);
    issues(out, &res)
}

fn sheet_cmd(a: &AnimSheetArgs, out: &mut dyn Write) -> Result<(), String> {
    let res = load(&a.file)?;
    save(&a.out, &review(&res, scale_of(a.scale)))?;
    let _ = writeln!(out, "{} guardado.", a.out);
    legend(out, &res);
    issues(out, &res)
}

fn frame_cmd(a: &AnimFrameArgs) -> Result<(), String> {
    let res = load(&a.file)?;
    let found = res
        .actions
        .iter()
        .enumerate()
        .find(|(i, x)| x.spec.name == a.action || (a.action.is_empty() && *i == 0));
    let Some((_, action)) = found else {
        return Err(format!(
            "no existe la acción {}",
            lume_domain::error::go_quote(&a.action)
        ));
    };
    let frames = i64::from(action.spec.frames);
    if a.frame < 0 || a.frame >= frames {
        return Err(format!(
            "{} tiene {frames} frames (0 a {})",
            action.spec.name,
            frames - 1
        ));
    }
    let one = Action {
        spec: ActionSpec {
            name: String::new(),
            frames: 1,
            fps: 0,
            looping: false,
        },
        line: 0,
        images: vec![action.images[a.frame as usize].clone()],
        out_of_bounds: vec![0],
        part_calls: vec![Vec::new()],
    };
    save(&a.out, &strip(&one, res.cell, scale_of(a.scale)))
}

fn export_cmd(a: &AnimExportArgs, out: &mut dyn Write) -> Result<(), String> {
    let res = load(&a.file)?;
    if !res.errors().is_empty() && !a.force {
        let _ = issues(out, &res);
        return Err("anim.js tiene errores: corrígelos o usa --force".into());
    }
    for file in export(&res, &go_dir(&a.file), a)? {
        let _ = writeln!(out, "{file}");
    }
    Ok(())
}

/// Guarda la hoja (y lo demás que se pida) y devuelve los archivos escritos.
/// Los nombres llevan el tamaño de la celda: `reloj_32x32.png`.
fn export(res: &AnimResult, dir: &str, a: &AnimExportArgs) -> Result<Vec<String>, String> {
    let scale = scale_of(a.scale);
    let name = safe_name(&res.name);
    let (cw, ch) = (res.cell.0 * scale, res.cell.1 * scale);
    let out = if a.out.is_empty() {
        go_join(dir, &format!("{name}_{cw}x{ch}.png"))
    } else {
        a.out.clone()
    };
    save(&out, &sheet(res, scale))?;
    let mut files = vec![out.clone()];
    if a.strips {
        for action in &res.actions {
            let path = go_join(
                &go_dir(&out),
                &format!("{name}_{}_{cw}x{ch}.png", safe_name(&action.spec.name)),
            );
            save(&path, &strip(action, res.cell, scale))?;
            files.push(path);
        }
    }
    let with_ext = |ext: &str| {
        let path = Path::new(&out);
        match path.extension() {
            Some(_) => path.with_extension(ext).to_string_lossy().into_owned(),
            None => format!("{out}.{ext}"),
        }
    };
    if a.json {
        let path = with_ext("json");
        let meta = sheet_meta(res, &go_base(&out), scale, &[]);
        let text = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())? + "\n";
        std::fs::write(&path, text).map_err(|e| format!("open {path}: {e}"))?;
        files.push(path);
    }
    if a.godot {
        let path = with_ext("tres");
        let texture = godot_path(Path::new(&path), Path::new(&out));
        std::fs::write(&path, sprite_frames_tres(res, &texture, scale))
            .map_err(|e| format!("open {path}: {e}"))?;
        files.push(path);
    }
    Ok(files)
}

/// Ruta de la hoja para el `.tres`: `res://…` si el `.tres` está dentro de
/// un proyecto de Godot (hay un `project.godot` más arriba); si no, el
/// nombre de la hoja (relativo al `.tres`, en la misma carpeta).
fn godot_path(tres: &Path, sheet: &Path) -> String {
    let fallback = go_base(&sheet.to_string_lossy());
    let absolute = |p: &Path| -> Option<PathBuf> {
        let dir = p
            .parent()
            .filter(|d| !d.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        Some(std::fs::canonicalize(dir).ok()?.join(p.file_name()?))
    };
    let (Some(tres), Some(sheet)) = (absolute(tres), absolute(sheet)) else {
        return fallback;
    };
    let Some(root) = tres
        .parent()
        .and_then(|d| d.ancestors().find(|a| a.join("project.godot").is_file()))
    else {
        return fallback;
    };
    match sheet.strip_prefix(root) {
        Ok(rel) => {
            let parts: Vec<String> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            format!("res://{}", parts.join("/"))
        }
        Err(_) => fallback,
    }
}

fn init(a: &AnimInitArgs, out: &mut dyn Write) -> Result<(), String> {
    if a.from.is_empty() {
        return Err("falta --from base.png".into());
    }
    let base = lume_store::read_anim_image(Path::new(&a.from))
        .map_err(|e| format!("no se puede leer la base: {}", e.message))?;
    let actions = parse_action_flags(&a.actions)?;
    let recipe_lines = if a.recipe.is_empty() {
        Vec::new()
    } else {
        lume_store::read_text(Path::new(&a.recipe))
            .map_err(|e| e.message)?
            .split('\n')
            .map(String::from)
            .collect()
    };
    let mut cell = base.size();
    if !a.cell.is_empty() {
        let (w, h) = lume_cli::parse_canvas_size(&a.cell).map_err(|e| e.0)?;
        let clamp = |v: i64| v.clamp(0, i64::from(u32::MAX)) as u32;
        cell = (clamp(w), clamp(h));
    } else if a.grow > 0 {
        let grow = a.grow.min(i64::from(u32::MAX)) as u32;
        cell = (cell.0.saturating_add(grow), cell.1.saturating_add(grow));
    }
    let mut name = a.name.clone();
    if name.is_empty() {
        name = stem(&a.out);
        if name == "anim" {
            name = stem(&a.from);
        }
    }
    let (source, note) = new_project(ProjectOptions {
        name,
        base: base.clone(),
        recipe_lines,
        cell,
        anchor: a.anchor.clone(),
        actions,
    })?;
    if Path::new(&a.out).exists() && !a.force {
        return Err(format!(
            "{} ya existe (usa --force para sobrescribirlo)",
            a.out
        ));
    }
    let dir = go_dir(&a.out);
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {dir}: {e}"))?;
    save(&go_join(&dir, "base.png"), &base)?;
    std::fs::write(&a.out, source).map_err(|e| format!("open {}: {e}", a.out))?;
    let _ = writeln!(
        out,
        "{} creado ({note}). Celda {}x{}.",
        a.out, cell.0, cell.1
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_like_go() {
        assert_eq!(go_dir("anim.js"), ".");
        assert_eq!(go_dir("anim/anim.js"), "anim");
        assert_eq!(go_join(".", "base.png"), "base.png");
        // Como filepath.Join de Go: con el separador del sistema.
        assert_eq!(
            go_join("anim", "base.png"),
            format!("anim{}base.png", std::path::MAIN_SEPARATOR)
        );
        assert_eq!(go_base("a/b/c.png"), "c.png");
        assert_eq!(stem("anim/anim.js"), "anim");
    }

    #[test]
    fn godot_paths_use_the_project_root() {
        let dir = tempfile::tempdir().unwrap();
        let sheet = dir.path().join("arte/personajes/reloj_12x12.png");
        let tres = sheet.with_extension("tres");
        assert_eq!(godot_path(&tres, &sheet), "reloj_12x12.png", "sin proyecto");
        std::fs::create_dir_all(sheet.parent().unwrap()).unwrap();
        std::fs::write(dir.path().join("project.godot"), "").unwrap();
        assert_eq!(
            godot_path(&tres, &sheet),
            "res://arte/personajes/reloj_12x12.png"
        );
    }
}
