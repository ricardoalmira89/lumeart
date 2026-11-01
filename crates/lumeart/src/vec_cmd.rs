//! `lumeart vec init|check|render|svg|ref|compare|godot|poses|calco|export`, como `pxcli vec`
//! (`internal/cli/vec.go`): mismos argumentos, salidas y errores. Trabaja
//! con archivos y no necesita una sesión abierta. El SVG lo da `dibujo.js`
//! con QuickJS y el PNG, resvg (en vez de Chrome).

use std::io::Write;
use std::path::{Component, Path, PathBuf};

use base64::Engine;
use lume_app::VecEngine;
use lume_app::ports::{PortError, RasterOptions, Rasterizer};
use lume_cli::{
    VecArgs, VecCalcoArgs, VecCommand, VecCompareArgs, VecExportArgs, VecGodotArgs, VecInitArgs,
    VecPosesArgs, VecRefArgs, VecRenderArgs, VecSvgArgs,
};
use lume_domain::Rgba;
use lume_domain::vec::reference::{
    CompareMode, REFERENCE_PREFIX, compare_svg, image_element, not_an_image, reference_svg,
    sniff_image,
};
use lume_domain::vec::rig::{
    atlas_svg, auto_scale, godot_name, godot_scene, pack_atlas, poses_svg,
};
use lume_domain::vec::scene::{PixelBox, alpha_bounds, crop, padded, plan_scene, scene_tscn};
use lume_domain::vec::trace::{TraceOptions, post_trace, trace_colors, trace_svg};
use lume_domain::vec::{VecResult, errors_message, issue_lines, parse_crop, parse_size, template};
use lume_raster::ResvgRasterizer;
use lume_script::QuickJsVec;

/// Ejecuta un subcomando de `vec`. Devuelve el código de salida.
pub fn run(args: &VecArgs, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let result = match &args.command {
        VecCommand::Init(a) => init(a, stdout),
        VecCommand::Check(a) => check(&a.file, stdout),
        VecCommand::Render(a) => render(a, stdout),
        VecCommand::Svg(a) => svg(a, stdout),
        VecCommand::Ref(a) => reference(a, stdout),
        VecCommand::Compare(a) => compare(a, stdout),
        VecCommand::Godot(a) => godot(a, stdout),
        VecCommand::Poses(a) => poses(a, stdout),
        VecCommand::Calco(a) => calco(a, stdout),
        VecCommand::Export(a) => export(a, stdout),
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

/// El motor de `dibujo.js` con sus ayudas: el calco (la referencia, junto
/// al dibujo) y VectorCraft si está instalado.
fn engine(path: &str) -> QuickJsVec {
    QuickJsVec {
        trace: Some(crate::vectorcraft::tracer(PathBuf::from(go_dir(path)))),
        vectorcraft: crate::vectorcraft::helper(),
    }
}

/// El modo vector de la ventana: el motor de `dibujo.js` (el calco busca
/// la referencia en `dir`, la carpeta de trabajo del agente), el
/// rasterizador y el buscador de formas del lazo.
pub fn window_vector(dir: PathBuf) -> (lume_agent::VecTools, lume_ui::VecServices) {
    let engine: std::sync::Arc<dyn VecEngine> = std::sync::Arc::new(QuickJsVec {
        trace: Some(crate::vectorcraft::tracer(dir)),
        vectorcraft: crate::vectorcraft::helper(),
    });
    let rasterizer = std::sync::Arc::new(ResvgRasterizer);
    let tools = lume_agent::VecTools {
        engine: engine.clone(),
        rasterizer: rasterizer.clone(),
    };
    let services = lume_ui::VecServices {
        engine,
        rasterizer: rasterizer.clone(),
        shapes: rasterizer,
        exporter: Some(std::sync::Arc::new(WindowExporter)),
    };
    (tools, services)
}

/// Exportar desde la ventana: lo mismo que `lumeart vec export`.
pub(crate) struct WindowExporter;

impl lume_app::ports::VectorExporter for WindowExporter {
    fn export(&self, svg: &str, path: &Path) -> Result<Vec<String>, lume_app::ports::PortError> {
        let fail = |m: String| lume_app::ports::PortError::Invalid(m);
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "svg" => lume_store::write_bytes(path, format!("{svg}\n").as_bytes())
                .map(|()| Vec::new())
                .map_err(|e| fail(e.message)),
            "pdf" | "ai" => {
                let pdf = lume_raster::pdf::svg_to_pdf(svg).map_err(|e| fail(port_message(e)))?;
                lume_store::write_bytes(path, &pdf)
                    .map(|()| Vec::new())
                    .map_err(|e| fail(e.message))
            }
            "vectorcraft" | "eps" => {
                let out = go_abs(&path.to_string_lossy()).map_err(fail)?;
                crate::vectorcraft::convert(svg, &out).map_err(fail)
            }
            other => Err(fail(format!("formato .{other} no soportado"))),
        }
    }
}

/// Lee y ejecuta `dibujo.js`.
fn load(path: &str) -> Result<VecResult, String> {
    let source = lume_store::read_text(Path::new(path))
        .map_err(|e| format!("no se puede leer {path}: {}", e.message))?;
    Ok(engine(path).render(&source))
}

/// Lee y ejecuta `dibujo.js`, con la escena partida en piezas.
fn load_scene(path: &str) -> Result<VecResult, String> {
    let source = lume_store::read_text(Path::new(path))
        .map_err(|e| format!("no se puede leer {path}: {}", e.message))?;
    Ok(engine(path).render_scene(&source))
}

/// Los problemas, uno por línea; error si hay errores (para que el agente
/// sepa por el código de salida que algo va mal).
fn issues(w: &mut dyn Write, res: &VecResult) -> Result<(), String> {
    for line in issue_lines(res) {
        let _ = writeln!(w, "{line}");
    }
    match res.errors().len() {
        0 => Ok(()),
        n => Err(errors_message(n)),
    }
}

fn port_message(e: PortError) -> String {
    match e {
        PortError::Invalid(m) | PortError::Unavailable(m) | PortError::Io(m) => m,
        other => other.to_string(),
    }
}

/// El ancho de salida tal como lo pasa pxcli (0 o menos: el de por defecto).
fn out_width(width: i64) -> Option<u32> {
    (width > 0).then(|| width.min(i64::from(u32::MAX)) as u32)
}

/// Rasteriza y guarda el PNG.
fn save_png(
    out: &str,
    svg: &str,
    width: f64,
    height: f64,
    options: &RasterOptions,
) -> Result<(), String> {
    let image = ResvgRasterizer
        .rasterize(svg, width, height, options)
        .map_err(port_message)?;
    lume_store::write_rgba_png(Path::new(out), image.width, image.height, &image.rgba)
        .map_err(|e| e.message)
}

fn init(a: &VecInitArgs, out: &mut dyn Write) -> Result<(), String> {
    let (w, h) = parse_size(&a.size)?;
    if Path::new(&a.out).exists() && !a.force {
        return Err(format!(
            "{} ya existe (usa --force para sobrescribirlo)",
            a.out
        ));
    }
    let dir = go_dir(&a.out);
    if dir != "." {
        std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {dir}: {e}"))?;
    }
    lume_store::write_bytes(Path::new(&a.out), template(w, h, a.transparent).as_bytes())
        .map_err(|e| e.message)?;
    let _ = writeln!(out, "{} creado: lienzo de {w}x{h}.", a.out);
    Ok(())
}

fn check(file: &str, out: &mut dyn Write) -> Result<(), String> {
    let res = load(file)?;
    let _ = writeln!(
        out,
        "{}x{} · {} formas · SVG de {} KB",
        res.width,
        res.height,
        res.count,
        res.clean.len() / 1024
    );
    issues(out, &res)
}

fn render(a: &VecRenderArgs, out: &mut dyn Write) -> Result<(), String> {
    let res = load(&a.file)?;
    let mut options = RasterOptions {
        width: out_width(a.width),
        grid: (a.grid > 0.0).then_some(a.grid),
        ..RasterOptions::default()
    };
    if !a.bg.is_empty() {
        options.background = Some(Rgba::parse(&a.bg).map_err(|e| format!("--bg: {}", e.message))?);
    }
    if !a.crop.is_empty() {
        options.crop = Some(parse_crop(&a.crop)?);
    }
    if res.clean.is_empty() {
        return issues(out, &res);
    }
    save_png(
        &a.out,
        &res.clean,
        res.width as f64,
        res.height as f64,
        &options,
    )?;
    let what = if a.crop.is_empty() {
        "dibujo entero".to_string()
    } else {
        format!("zona {}", a.crop)
    };
    let _ = writeln!(
        out,
        "{} guardado ({what}, {} formas, ancho {} px)",
        a.out, res.count, a.width
    );
    issues(out, &res)
}

fn svg(a: &VecSvgArgs, out: &mut dyn Write) -> Result<(), String> {
    let res = load(&a.file)?;
    if !res.clean.is_empty() {
        lume_store::write_bytes(Path::new(&a.out), format!("{}\n", res.clean).as_bytes())
            .map_err(|e| e.message)?;
        let _ = writeln!(
            out,
            "{} guardado ({} formas, {} KB)",
            a.out,
            res.count,
            res.clean.len() / 1024
        );
    }
    issues(out, &res)
}

/// La `referencia-*` más nueva de `dir` que sea una imagen (`FindReference`).
fn find_reference(dir: &Path) -> Option<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(REFERENCE_PREFIX))
        .collect();
    names.sort();
    names.into_iter().rev().find(|name| {
        std::fs::read(dir.join(name))
            .ok()
            .is_some_and(|bytes| sniff_image(&bytes[..bytes.len().min(512)]).is_some())
    })
}

fn reference_path(given: &str) -> Result<String, String> {
    if !given.is_empty() {
        return Ok(given.to_string());
    }
    find_reference(Path::new(".")).ok_or_else(|| {
        "no hay imagen de referencia (referencia-*.png) en esta carpeta: indica una con --ref"
            .to_string()
    })
}

/// El `<image>` de la referencia en `path` (`referenceImage`).
fn reference_image(path: &str, w: u32, h: u32, opacity: f64) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|_| not_an_image(path))?;
    let mime = sniff_image(&bytes[..bytes.len().min(512)]).ok_or_else(|| not_an_image(path))?;
    let data = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(image_element(mime, &data, w, h, opacity))
}

/// Tamaño del lienzo como entero positivo para colocar la referencia.
fn canvas_size(res: &VecResult) -> (u32, u32) {
    let side = |v: i64| v.clamp(0, i64::from(u32::MAX)) as u32;
    (side(res.width), side(res.height))
}

fn reference(a: &VecRefArgs, out: &mut dyn Write) -> Result<(), String> {
    let given = a.image.clone().unwrap_or_else(|| a.reference.clone());
    let path = reference_path(&given)?;
    let res = load(&a.file)?;
    let (w, h) = canvas_size(&res);
    let image = reference_image(&path, w, h, 1.0)?;
    save_png(
        &a.out,
        &reference_svg(&image, w, h),
        f64::from(w),
        f64::from(h),
        &RasterOptions {
            width: out_width(a.width),
            grid: (a.grid > 0.0).then_some(a.grid),
            ..RasterOptions::default()
        },
    )?;
    let _ = writeln!(
        out,
        "{} guardado: {path} encajada en el lienzo de {w}x{h} (las coordenadas son las de dibujo.js)",
        a.out
    );
    Ok(())
}

fn compare(a: &VecCompareArgs, out: &mut dyn Write) -> Result<(), String> {
    let path = reference_path(&a.reference)?;
    let res = load(&a.file)?;
    if res.clean.is_empty() {
        return Err("el dibujo está vacío".into());
    }
    let mode = CompareMode::parse(&a.mode)?;
    let (w, h) = canvas_size(&res);
    let image = reference_image(&path, w, h, 1.0)?;
    let comparison = compare_svg(
        &res.clean,
        &image,
        w,
        h,
        mode,
        a.grid,
        out_width(a.width).unwrap_or(0),
    );
    save_png(
        &a.out,
        &comparison.svg,
        f64::from(comparison.width),
        f64::from(comparison.height),
        &RasterOptions {
            width: Some(comparison.out_width),
            grid: comparison.grid,
            ..RasterOptions::default()
        },
    )?;
    let what = match mode {
        CompareMode::Side => "izquierda el dibujo, derecha la referencia",
        CompareMode::Over => "el dibujo al 55 % sobre la referencia",
    };
    let _ = writeln!(out, "{} guardado ({what})", a.out);
    issues(out, &res)
}

/// `filepath.Abs` de Go: la ruta absoluta y limpia (sin `.` ni `..`).
fn go_abs(path: &str) -> Result<PathBuf, String> {
    let joined = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join(path);
    let mut out = PathBuf::new();
    for part in joined.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    Ok(out)
}

/// La carpeta del proyecto de Godot (la que tiene `project.godot`) que
/// contiene `dir`.
fn godot_project_root(dir: &Path) -> Result<PathBuf, String> {
    let mut d = dir;
    loop {
        if d.join("project.godot").exists() {
            return Ok(d.to_path_buf());
        }
        match d.parent() {
            Some(parent) => d = parent,
            None => {
                return Err(format!(
                    "{} no está dentro de un proyecto de Godot (no encuentro project.godot)",
                    dir.display()
                ));
            }
        }
    }
}

/// `dir` relativa a la raíz del proyecto, con `/` (vacía si es la raíz).
fn res_path(root: &Path, dir: &Path) -> String {
    dir.strip_prefix(root)
        .map(|rel| {
            rel.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_default()
}

fn godot(a: &VecGodotArgs, out: &mut dyn Write) -> Result<(), String> {
    let res = load_scene(&a.file)?;
    if !res.errors().is_empty() {
        return issues(out, &res);
    }
    if res.rig.is_some() {
        godot_character(&res, a, out)
    } else {
        godot_scene_export(&res, a, out)
    }
}

/// Un personaje: el atlas de piezas y la escena con su esqueleto
/// (`GodotExport`).
fn godot_character(res: &VecResult, a: &VecGodotArgs, out: &mut dyn Write) -> Result<(), String> {
    let rig = res
        .rig
        .as_ref()
        .ok_or("el dibujo no tiene esqueleto: declara huesos con bone() y piezas con part()")?;
    if rig.parts.is_empty() {
        return Err(
            "el esqueleto no tiene piezas: dibuja cada parte con part(hueso, () => { ... })".into(),
        );
    }
    let scale = if a.scale > 0.0 {
        a.scale
    } else {
        auto_scale(rig)
    };
    let name = godot_name(&a.name);
    let dir = go_abs(&a.out)?;
    let root = godot_project_root(&dir)?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    let rel = res_path(&root, &dir);
    let res_dir = if rel.is_empty() {
        "res://".to_string()
    } else {
        format!("res://{rel}")
    };
    let cells = pack_atlas(&rig.parts, scale);
    let atlas = atlas_svg(rig, &cells, scale);
    let png = dir.join(format!("{name}_piezas.png"));
    save_png(
        &png.to_string_lossy(),
        &atlas.svg,
        atlas.width,
        atlas.height,
        &RasterOptions {
            width: Some(atlas.out_width).filter(|w| *w > 0),
            ..RasterOptions::default()
        },
    )?;
    let tex = format!(
        "{}/{name}_piezas.png",
        res_dir.strip_suffix('/').unwrap_or(&res_dir)
    );
    let tscn = dir.join(format!("{name}.tscn"));
    lume_store::write_bytes(
        &tscn,
        godot_scene(rig, &name, &tex, &cells, scale).as_bytes(),
    )
    .map_err(|e| e.message)?;
    let _ = writeln!(out, "{}", png.display());
    let _ = writeln!(out, "{}", tscn.display());
    let actions: Vec<&str> = rig.actions.iter().map(|x| x.name.as_str()).collect();
    let _ = writeln!(
        out,
        "{} huesos, {} piezas, acciones: {}",
        rig.bones.len(),
        rig.parts.len(),
        actions.join(", ")
    );
    Ok(())
}

/// Una pieza recortada: su caja y sus píxeles, o nada si no dibuja.
type Cut = Result<Option<(PixelBox, Vec<u8>)>, String>;

/// Una escena: un PNG por pieza en `texturas/`, recortado a lo que dibuja,
/// y la escena con un `Parallax2D` por capa (`GodotSceneExport`).
fn godot_scene_export(
    res: &VecResult,
    a: &VecGodotArgs,
    out: &mut dyn Write,
) -> Result<(), String> {
    let mut plan = plan_scene(res.scene.as_ref(), &a.name, a.scale)?;
    let dir = go_abs(&a.out)?;
    let root = godot_project_root(&dir)?;
    let tex_dir = dir.join("texturas");
    std::fs::create_dir_all(&tex_dir).map_err(|e| format!("mkdir {}: {e}", tex_dir.display()))?;
    let rel = res_path(&root, &dir);
    let res_dir = if rel.is_empty() {
        "res:/".to_string()
    } else {
        format!("res://{rel}")
    };
    // Cada pieza sobre el lienzo entero, recortada a sus píxeles visibles
    // (con margen), para que los desenfoques no pierdan nada.
    let (pw, ph) = plan.cell();
    let (cw, ch) = (plan.scene.width as f64, plan.scene.height as f64);
    let svgs: Vec<String> = plan.pieces.iter().map(|p| plan.piece_svg(p)).collect();
    let cut = |svg: &String| -> Cut {
        let image = ResvgRasterizer
            .rasterize(
                svg,
                cw,
                ch,
                &RasterOptions {
                    width: Some(pw.max(1) as u32),
                    ..RasterOptions::default()
                },
            )
            .map_err(port_message)?;
        let width = i64::from(image.width);
        let cell = PixelBox {
            x0: 0,
            y0: 0,
            x1: pw.min(width),
            y1: ph.min(i64::from(image.height)),
        };
        Ok(alpha_bounds(&image.rgba, width, cell).map(|b| {
            let b = padded(b, cell);
            (b, crop(&image.rgba, width, b))
        }))
    };
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = svgs.len().div_ceil(threads).max(1);
    let cuts: Vec<Cut> = std::thread::scope(|scope| {
        let handles: Vec<_> = svgs
            .chunks(chunk)
            .map(|part| scope.spawn(|| part.iter().map(cut).collect::<Vec<_>>()))
            .collect();
        handles
            .into_iter()
            .flat_map(|h| {
                h.join()
                    .unwrap_or_else(|_| vec![Err("no se pudo rasterizar una pieza".into())])
            })
            .collect()
    });
    let mut written = Vec::new();
    for (piece, cut) in plan.pieces.iter_mut().zip(cuts) {
        let Some((bbox, rgba)) = cut? else { continue };
        piece.bbox = Some(bbox);
        let path = tex_dir.join(&piece.file);
        lume_store::write_rgba_png(&path, bbox.width() as u32, bbox.height() as u32, &rgba)
            .map_err(|e| e.message)?;
        written.push(path);
    }
    let tscn = dir.join(format!("{}.tscn", plan.name));
    lume_store::write_bytes(
        &tscn,
        scene_tscn(&plan, &format!("{res_dir}/texturas/")).as_bytes(),
    )
    .map_err(|e| e.message)?;
    let _ = writeln!(out, "{}", tscn.display());
    let _ = writeln!(
        out,
        "{} texturas en texturas/ · {} capas · {} elementos",
        written.len(),
        plan.scene.layers.len(),
        plan.element_count()
    );
    let empty = plan.empty();
    if !empty.is_empty() {
        let _ = writeln!(
            out,
            "sin nada visible (no se exportan): {}",
            empty.join(", ")
        );
    }
    Ok(())
}

fn poses(a: &VecPosesArgs, out: &mut dyn Write) -> Result<(), String> {
    let res = load(&a.file)?;
    let poses = poses_svg(res.rig.as_ref(), &a.action, a.frames, a.width)?;
    save_png(
        &a.out,
        &poses.svg,
        poses.width,
        poses.height,
        &RasterOptions {
            width: Some(poses.out_width).filter(|w| *w > 0),
            ..RasterOptions::default()
        },
    )?;
    let _ = writeln!(
        out,
        "{} guardado: primera fila, la pose del dibujo; después una fila por acción",
        a.out
    );
    issues(out, &res)
}

/// `vec calco`: el calco de la referencia solo, sobre blanco, y sus colores
/// del más usado al menos.
fn calco(a: &VecCalcoArgs, out: &mut dyn Write) -> Result<(), String> {
    let path = reference_path(&a.reference)?;
    let res = load(&a.file)?;
    let (w, h) = canvas_size(&res);
    let job = TraceOptions {
        width: w,
        height: h,
        lines: a.mode == "lines",
        colors: a.colors.clamp(2, 64) as u32,
        paths: a.detail,
        corners: 50.0,
        noise: a.noise,
        threshold: a.threshold.clamp(0, 255) as u32,
        ..TraceOptions::default()
    };
    let kind =
        crate::vectorcraft::image_kind(Path::new(&path)).ok_or_else(|| not_an_image(&path))?;
    let raw = crate::vectorcraft::raw_trace(Path::new(&path), kind, &job)?;
    if !job.lines {
        let colors = trace_colors(&raw);
        let total: usize = colors.iter().map(|c| c.paths).sum();
        let _ = writeln!(
            out,
            "Colores del calco ({total} trazados), de más a menos usado:"
        );
        for c in &colors {
            let _ = writeln!(out, "  {}  {:5} trazados", c.hex, c.paths);
        }
    }
    let svg = trace_svg(&post_trace(&raw, &job), w, h);
    save_png(
        &a.out,
        &svg,
        f64::from(w),
        f64::from(h),
        &RasterOptions {
            width: out_width(a.width),
            background: Some(Rgba::new(255, 255, 255, 255)),
            ..RasterOptions::default()
        },
    )?;
    let base = Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.clone());
    let _ = writeln!(
        out,
        "{} guardado: el calco de {base} solo, en las coordenadas de dibujo.js",
        a.out
    );
    Ok(())
}

/// `vec export`: el dibujo en el formato de la extensión de `-o`. SVG y PDF
/// (también `.ai`, que es un PDF) con LumeArt; `.vectorcraft` y `.eps` con
/// VectorCraft.
fn export(a: &VecExportArgs, out: &mut dyn Write) -> Result<(), String> {
    let res = load(&a.file)?;
    let ext = Path::new(&a.out)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_default();
    if !matches!(
        ext.as_str(),
        ".svg" | ".vectorcraft" | ".ai" | ".pdf" | ".eps"
    ) {
        return Err(format!(
            "formato {} no soportado: usa .vectorcraft, .ai, .pdf, .eps o .svg",
            lume_domain::error::go_quote(&ext)
        ));
    }
    if res.clean.is_empty() {
        return Err("el dibujo está vacío".into());
    }
    let warnings = match ext.as_str() {
        ".svg" => {
            lume_store::write_bytes(Path::new(&a.out), format!("{}\n", res.clean).as_bytes())
                .map_err(|e| e.message)?;
            Vec::new()
        }
        ".pdf" | ".ai" => {
            let pdf = lume_raster::pdf::svg_to_pdf(&res.clean).map_err(port_message)?;
            lume_store::write_bytes(Path::new(&a.out), &pdf).map_err(|e| e.message)?;
            Vec::new()
        }
        _ => crate::vectorcraft::convert(&res.clean, &go_abs(&a.out)?)?,
    };
    let _ = writeln!(out, "{} guardado ({} formas)", a.out, res.count);
    for w in warnings {
        let _ = writeln!(out, "aviso de VectorCraft: {w}");
    }
    issues(out, &res)
}
