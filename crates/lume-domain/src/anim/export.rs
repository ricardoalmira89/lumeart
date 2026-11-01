//! Lo que acompaña a la hoja exportada: sus datos en JSON y un recurso
//! `SpriteFrames` de Godot 4 (`.tres`) listo para `AnimatedSprite2D`.

use std::fmt::Write;

use serde::Serialize;

use super::animation::default_duration;
use super::project::safe_name;
use super::sheet::{sheet, strip};
use super::{Action, AnimResult, Image};

/// Una fila de la hoja.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SheetAction {
    pub name: String,
    /// Fila de la hoja (desde 0).
    pub row: usize,
    pub frames: u32,
    pub fps: u32,
    #[serde(rename = "loop")]
    pub looping: bool,
    /// Milisegundos de cada fotograma.
    pub durations: Vec<u32>,
}

/// Los datos de una hoja (el `.json` junto al PNG).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SheetMeta {
    pub name: String,
    /// El PNG de la hoja (nombre de archivo).
    pub image: String,
    /// Tamaño de cada fotograma en la hoja (ya ampliado).
    pub frame_width: u32,
    pub frame_height: u32,
    pub scale: u32,
    /// Dónde cae el (0, 0) de la base en cada celda (sin ampliar).
    pub origin: [i64; 2],
    pub actions: Vec<SheetAction>,
}

/// Los datos de la hoja de `res` ampliada `scale` veces. `durations`: los
/// milisegundos de cada fotograma por acción (si falta una, los de sus fps).
pub fn sheet_meta(res: &AnimResult, image: &str, scale: u32, durations: &[Vec<u32>]) -> SheetMeta {
    let scale = scale.max(1);
    SheetMeta {
        name: res.name.clone(),
        image: image.to_string(),
        frame_width: res.cell.0 * scale,
        frame_height: res.cell.1 * scale,
        scale,
        origin: [res.origin.0, res.origin.1],
        actions: res
            .actions
            .iter()
            .enumerate()
            .map(|(row, a)| SheetAction {
                name: a.spec.name.clone(),
                row,
                frames: a.spec.frames,
                fps: a.spec.fps,
                looping: a.spec.looping,
                durations: durations_of(a, durations.get(row)),
            })
            .collect(),
    }
}

/// Las duraciones de una acción: las dadas o las de sus fps.
fn durations_of(action: &Action, given: Option<&Vec<u32>>) -> Vec<u32> {
    match given {
        Some(d) if d.len() == action.spec.frames as usize => d.clone(),
        _ => vec![default_duration(action.spec.fps); action.spec.frames as usize],
    }
}

/// Qué se exporta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    /// La hoja de sprites: todas las acciones en una imagen.
    Sheet,
    /// Un PNG por acción.
    Strips,
    /// Un GIF animado por acción.
    Gif,
    /// La hoja y un `SpriteFrames` de Godot 4 (`.tres`).
    Godot,
}

impl ExportFormat {
    pub const ALL: [ExportFormat; 4] = [
        ExportFormat::Sheet,
        ExportFormat::Strips,
        ExportFormat::Gif,
        ExportFormat::Godot,
    ];
}

/// Un archivo que hay que escribir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportFile {
    Png(Image),
    Gif {
        frames: Vec<Image>,
        durations: Vec<u32>,
        looping: bool,
    },
    Text(String),
}

/// Una imagen ampliada `scale` veces.
pub fn scaled(img: &Image, scale: u32) -> Image {
    let one = Action {
        spec: super::ActionSpec {
            name: String::new(),
            frames: 1,
            fps: 1,
            looping: false,
        },
        line: 0,
        images: vec![img.clone()],
        out_of_bounds: vec![0],
        part_calls: vec![Vec::new()],
    };
    strip(&one, img.size(), scale)
}

/// Nombre por defecto de la hoja: `<nombre>_<W>x<H>.png` (con la celda ya
/// ampliada), como `anim export`.
pub fn sheet_file_name(res: &AnimResult, scale: u32) -> String {
    let scale = scale.max(1);
    format!(
        "{}_{}x{}.png",
        safe_name(&res.name),
        res.cell.0 * scale,
        res.cell.1 * scale
    )
}

/// Los archivos de una exportación, con su nombre (en la misma carpeta).
/// `sheet_name` es el de la hoja (también da nombre al `.json` y al `.tres`);
/// siempre va el `.json` con las acciones, fps y duraciones.
pub fn export_files(
    res: &AnimResult,
    durations: &[Vec<u32>],
    format: ExportFormat,
    scale: u32,
    sheet_name: &str,
) -> Vec<(String, ExportFile)> {
    let scale = scale.max(1);
    let name = safe_name(&res.name);
    let (cw, ch) = (res.cell.0 * scale, res.cell.1 * scale);
    let stem = sheet_name
        .strip_suffix(".png")
        .unwrap_or(sheet_name)
        .to_string();
    let sheet_name = format!("{stem}.png");
    let mut files = Vec::new();
    match format {
        ExportFormat::Sheet | ExportFormat::Godot => {
            files.push((sheet_name.clone(), ExportFile::Png(sheet(res, scale))));
        }
        ExportFormat::Strips => {
            for a in &res.actions {
                files.push((
                    format!("{name}_{}_{cw}x{ch}.png", safe_name(&a.spec.name)),
                    ExportFile::Png(strip(a, res.cell, scale)),
                ));
            }
        }
        ExportFormat::Gif => {
            for (row, a) in res.actions.iter().enumerate() {
                files.push((
                    format!("{name}_{}.gif", safe_name(&a.spec.name)),
                    ExportFile::Gif {
                        frames: a.images.iter().map(|img| scaled(img, scale)).collect(),
                        durations: durations_of(a, durations.get(row)),
                        looping: a.spec.looping,
                    },
                ));
            }
        }
    }
    if format == ExportFormat::Godot {
        files.push((
            format!("{stem}.tres"),
            ExportFile::Text(sprite_frames_tres(res, &sheet_name, scale)),
        ));
    }
    let meta = sheet_meta(res, &sheet_name, scale, durations);
    files.push((format!("{stem}.json"), ExportFile::Text(json(&meta))));
    files
}

/// JSON con sangría de los datos de la hoja (sin `serde_json`: el dominio
/// no lo usa).
fn json(meta: &SheetMeta) -> String {
    let q = |s: &str| crate::error::go_quote(s);
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"name\": {},\n", q(&meta.name)));
    out.push_str(&format!("  \"image\": {},\n", q(&meta.image)));
    out.push_str(&format!("  \"frame_width\": {},\n", meta.frame_width));
    out.push_str(&format!("  \"frame_height\": {},\n", meta.frame_height));
    out.push_str(&format!("  \"scale\": {},\n", meta.scale));
    out.push_str(&format!(
        "  \"origin\": [{}, {}],\n",
        meta.origin[0], meta.origin[1]
    ));
    out.push_str("  \"actions\": [");
    for (i, a) in meta.actions.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let durations: Vec<String> = a.durations.iter().map(u32::to_string).collect();
        out.push_str(&format!(
            "\n    {{\"name\": {}, \"row\": {}, \"frames\": {}, \"fps\": {}, \"loop\": {}, \"durations\": [{}]}}",
            q(&a.name),
            a.row,
            a.frames,
            a.fps,
            a.looping,
            durations.join(", ")
        ));
    }
    out.push_str(if meta.actions.is_empty() {
        "]\n}\n"
    } else {
        "\n  ]\n}\n"
    });
    out
}

/// Una cadena de Godot entre comillas (escapes de C).
fn godot_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Recurso `SpriteFrames` (Godot 4, `format=3`) con una animación por acción;
/// cada fotograma es un `AtlasTexture` de la hoja en `texture` (una ruta
/// `res://…` o relativa al `.tres`). Velocidad = fps; duración 1 por fotograma.
pub fn sprite_frames_tres(res: &AnimResult, texture: &str, scale: u32) -> String {
    let scale = scale.max(1);
    let (fw, fh) = (res.cell.0 * scale, res.cell.1 * scale);
    let frames: usize = res.actions.iter().map(|a| a.spec.frames as usize).sum();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "[gd_resource type=\"SpriteFrames\" load_steps={} format=3]\n",
        frames + 2
    );
    let _ = writeln!(
        out,
        "[ext_resource type=\"Texture2D\" path={} id=\"1_hoja\"]\n",
        godot_string(texture)
    );
    for (row, a) in res.actions.iter().enumerate() {
        for f in 0..a.spec.frames {
            let _ = writeln!(
                out,
                "[sub_resource type=\"AtlasTexture\" id=\"AtlasTexture_{row}_{f}\"]\natlas = ExtResource(\"1_hoja\")\nregion = Rect2({}, {}, {fw}, {fh})\n",
                f * fw,
                row as u32 * fh
            );
        }
    }
    out.push_str("[resource]\nanimations = [");
    for (row, a) in res.actions.iter().enumerate() {
        if row > 0 {
            out.push_str(", ");
        }
        out.push_str("{\n\"frames\": [");
        for f in 0..a.spec.frames {
            if f > 0 {
                out.push_str(", ");
            }
            let _ = write!(
                out,
                "{{\n\"duration\": 1.0,\n\"texture\": SubResource(\"AtlasTexture_{row}_{f}\")\n}}"
            );
        }
        let _ = write!(
            out,
            "],\n\"loop\": {},\n\"name\": &{},\n\"speed\": {:.1}\n}}",
            a.spec.looping,
            godot_string(&a.spec.name),
            f64::from(a.spec.fps)
        );
    }
    out.push_str("]\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::{Action, ActionSpec, Image};

    fn result() -> AnimResult {
        let action = |name: &str, frames: u32, looping: bool| Action {
            spec: ActionSpec {
                name: name.into(),
                frames,
                fps: 10,
                looping,
            },
            line: 1,
            images: vec![Image::new(8, 6); frames as usize],
            out_of_bounds: vec![0; frames as usize],
            part_calls: vec![Vec::new(); frames as usize],
        };
        AnimResult {
            name: "reloj".into(),
            cell: (8, 6),
            origin: (1, 2),
            actions: vec![action("caer", 2, true), action("di \"hola\"", 1, false)],
            ..AnimResult::default()
        }
    }

    #[test]
    fn meta_has_one_row_per_action() {
        let meta = sheet_meta(&result(), "reloj_16x12.png", 2, &[vec![50, 150]]);
        assert_eq!(meta.actions[0].durations, [50, 150]);
        assert_eq!(
            meta.actions[1].durations,
            [100],
            "sin dadas, las de sus fps"
        );
        assert_eq!((meta.frame_width, meta.frame_height), (16, 12));
        assert_eq!(meta.actions[1].row, 1);
        assert!(!meta.actions[1].looping);
    }

    #[test]
    fn each_format_writes_its_files() {
        let res = result();
        let names = |format| -> Vec<String> {
            export_files(&res, &[], format, 2, "reloj_16x12.png")
                .into_iter()
                .map(|(n, _)| n)
                .collect()
        };
        assert_eq!(
            names(ExportFormat::Sheet),
            ["reloj_16x12.png", "reloj_16x12.json"]
        );
        assert_eq!(
            names(ExportFormat::Strips),
            [
                "reloj_caer_16x12.png",
                "reloj_di-hola_16x12.png",
                "reloj_16x12.json"
            ]
        );
        assert_eq!(
            names(ExportFormat::Gif),
            ["reloj_caer.gif", "reloj_di-hola.gif", "reloj_16x12.json"]
        );
        assert_eq!(
            names(ExportFormat::Godot),
            ["reloj_16x12.png", "reloj_16x12.tres", "reloj_16x12.json"]
        );
        let files = export_files(&res, &[vec![60, 40]], ExportFormat::Gif, 3, "hoja");
        let ExportFile::Gif {
            frames,
            durations,
            looping,
        } = &files[0].1
        else {
            panic!()
        };
        assert_eq!((frames.len(), frames[0].size()), (2, (24, 18)));
        assert_eq!(durations, &[60, 40]);
        assert!(looping);
        let ExportFile::Text(json) = &files[2].1 else {
            panic!()
        };
        assert_eq!(files[2].0, "hoja.json");
        assert!(json.contains("\"image\": \"hoja.png\""), "{json}");
        assert!(json.contains("\"durations\": [60, 40]"), "{json}");
        let ExportFile::Png(sheet) = &export_files(&res, &[], ExportFormat::Sheet, 4, "x.png")[0].1
        else {
            panic!()
        };
        assert_eq!(sheet.size(), (2 * 8 * 4, 2 * 6 * 4));
    }

    #[test]
    fn sprite_frames_regions_follow_the_sheet() {
        let tres = sprite_frames_tres(&result(), "res://arte/reloj_16x12.png", 2);
        assert!(tres.starts_with("[gd_resource type=\"SpriteFrames\" load_steps=5 format=3]\n\n"));
        assert!(tres.contains(
            "[ext_resource type=\"Texture2D\" path=\"res://arte/reloj_16x12.png\" id=\"1_hoja\"]"
        ));
        assert!(tres.contains("[sub_resource type=\"AtlasTexture\" id=\"AtlasTexture_0_1\"]\natlas = ExtResource(\"1_hoja\")\nregion = Rect2(16, 0, 16, 12)"));
        assert!(tres.contains("region = Rect2(0, 12, 16, 12)"));
        assert!(tres.contains("\"loop\": true,\n\"name\": &\"caer\",\n\"speed\": 10.0"));
        assert!(tres.contains("\"name\": &\"di \\\"hola\\\"\""));
        assert!(tres.ends_with("}]\n"));
    }
}
