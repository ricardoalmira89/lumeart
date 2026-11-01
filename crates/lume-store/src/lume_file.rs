//! Formato `.lume`: un ZIP con nombres legibles que se abre con cualquier
//! herramienta.
//!
//! ```text
//! manifest.json        formato, nombre, tamaño, paleta, acciones, versiones
//! canvas.png           lienzo actual (RGBA)
//! recipe.txt           receta en el protocolo de texto (como `batch`)
//! notas.md             notas del agente entre vueltas del bucle
//! versions/001.png     lienzo de cada versión
//! versions/001.txt     receta de cada versión
//! anim.js              código de la animación, si tiene
//! anim/base.png        su base, si no es el lienzo (RGBA tal cual)
//! dibujo.js            código del dibujo vectorial, si tiene
//! versions/001.js      dibujo.js de cada versión, si tenía
//! ```
//!
//! Las duraciones de los fotogramas van en `manifest.json` (`animation`).
//! `agent/` llegará con el agente. Al abrir, un `format` antiguo pasa por
//! [`migrate`]; uno más nuevo que [`FORMAT`] da un error que pide actualizar
//! LumeArt.

use std::io::{Cursor, Read, Write};
use std::path::Path;

use lume_domain::anim::animation::Animation;
use lume_domain::vec::drawing::VectorDrawing;
use lume_domain::{
    Canvas, Document, DomainError, ErrorCode, Origin, Palette, Recipe, Rgba, Version,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::io_error;
use crate::png_io::{decode_image, decode_png, encode_image_raw, encode_png};

/// Versión actual del formato.
pub const FORMAT: u64 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Manifest {
    format: u64,
    /// Programa que lo escribió (`lumeart 0.1.0`).
    app: String,
    name: String,
    width: u32,
    height: u32,
    /// Tamaño del lienzo en blanco del que parte `recipe.txt`.
    recipe_start: Option<[u32; 2]>,
    palette: Option<ManifestPalette>,
    /// Acciones de animación (llegarán con la fase 8).
    #[serde(default)]
    actions: Vec<Value>,
    #[serde(default)]
    versions: Vec<ManifestVersion>,
    /// La animación, si el documento tiene (`anim.js` aparte).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    animation: Option<ManifestAnimation>,
    /// Hay dibujo vectorial (`dibujo.js`).
    #[serde(default, skip_serializing_if = "is_false")]
    vector: bool,
}

fn is_false(b: &bool) -> bool {
    !b
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ManifestAnimation {
    /// Hay `anim/base.png`.
    #[serde(default)]
    base: bool,
    /// Milisegundos de cada fotograma, por acción.
    #[serde(default)]
    durations: std::collections::BTreeMap<String, Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ManifestPalette {
    name: Option<String>,
    colors: Vec<ManifestColor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ManifestColor {
    color: String,
    name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ManifestVersion {
    number: u32,
    /// `user`, `agent` o `cli`.
    author: String,
    /// Ejecución del agente, si el autor es `agent`.
    #[serde(default)]
    run: Option<u64>,
    message: String,
    recipe_start: Option<[u32; 2]>,
    /// Tiene `versions/NNN.js`.
    #[serde(default, skip_serializing_if = "is_false")]
    vector: bool,
}

fn invalid(message: impl Into<String>) -> DomainError {
    DomainError::new(
        ErrorCode::Io,
        format!("invalid .lume file: {}", message.into()),
    )
}

fn version_file(number: u32, ext: &str) -> String {
    format!("versions/{number:03}.{ext}")
}

fn author_name(origin: Origin) -> (String, Option<u64>) {
    match origin {
        Origin::User => ("user".into(), None),
        Origin::Agent { run } => ("agent".into(), Some(run)),
        Origin::Cli => ("cli".into(), None),
    }
}

fn author_origin(author: &str, run: Option<u64>) -> Result<Origin, DomainError> {
    match author {
        "user" => Ok(Origin::User),
        "cli" => Ok(Origin::Cli),
        "agent" => Ok(Origin::Agent {
            run: run.unwrap_or(0),
        }),
        other => Err(invalid(format!("unknown version author {other:?}"))),
    }
}

/// Bytes del `.lume` de un documento.
pub fn encode_lume(document: &Document) -> Result<Vec<u8>, DomainError> {
    let canvas = document.canvas();
    let palette = document.palette.as_ref().map(|p| ManifestPalette {
        name: p.name.clone(),
        colors: p
            .colors()
            .iter()
            .map(|c| ManifestColor {
                color: c.color.to_string(),
                name: c.name.clone(),
            })
            .collect(),
    });
    let manifest = Manifest {
        format: FORMAT,
        app: format!("lumeart {}", env!("CARGO_PKG_VERSION")),
        name: document.name.clone(),
        width: canvas.width(),
        height: canvas.height(),
        recipe_start: document.recipe().start().map(|(w, h)| [w, h]),
        palette,
        actions: Vec::new(),
        animation: document.animation.as_ref().map(|a| ManifestAnimation {
            base: a.base.is_some(),
            durations: a.durations.clone(),
        }),
        vector: document.vector.is_some(),
        versions: document
            .versions()
            .iter()
            .map(|v| {
                let (author, run) = author_name(v.author);
                ManifestVersion {
                    number: v.number,
                    author,
                    run,
                    message: v.message.clone(),
                    recipe_start: v.recipe.start().map(|(w, h)| [w, h]),
                    vector: v.vector.is_some(),
                }
            })
            .collect(),
    };

    let text = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    // Los PNG ya van comprimidos.
    let image = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let mut add =
        |name: &str, options: SimpleFileOptions, bytes: &[u8]| -> Result<(), DomainError> {
            zip.start_file(name, options).map_err(zip_error)?;
            zip.write_all(bytes).map_err(|e| zip_error(e.into()))
        };

    let mut json =
        serde_json::to_string_pretty(&manifest).expect("el manifiesto siempre se serializa");
    json.push('\n');
    add("manifest.json", text, json.as_bytes())?;
    add("canvas.png", image, &encode_png(canvas, 1)?)?;
    add("recipe.txt", text, document.recipe().to_string().as_bytes())?;
    add("notas.md", text, document.notes.as_bytes())?;
    if let Some(animation) = &document.animation {
        add("anim.js", text, animation.source.as_bytes())?;
        if let Some(base) = &animation.base {
            add("anim/base.png", image, &encode_image_raw(base)?)?;
        }
    }
    if let Some(vector) = &document.vector {
        add("dibujo.js", text, vector.source.as_bytes())?;
    }
    for v in document.versions() {
        if let Some(vector) = &v.vector {
            add(
                &version_file(v.number, "js"),
                text,
                vector.source.as_bytes(),
            )?;
        }
        add(
            &version_file(v.number, "png"),
            image,
            &encode_png(&v.canvas, 1)?,
        )?;
        add(
            &version_file(v.number, "txt"),
            text,
            v.recipe.to_string().as_bytes(),
        )?;
    }
    let cursor = zip.finish().map_err(zip_error)?;
    Ok(cursor.into_inner())
}

/// Documento a partir de los bytes de un `.lume`.
pub fn decode_lume(bytes: &[u8]) -> Result<Document, DomainError> {
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|e| invalid(e.to_string()))?;
    let raw: Value = serde_json::from_slice(&read_entry(&mut zip, "manifest.json")?)
        .map_err(|e| invalid(format!("manifest.json: {e}")))?;
    let manifest: Manifest = serde_json::from_value(migrate(raw)?)
        .map_err(|e| invalid(format!("manifest.json: {e}")))?;

    let canvas = decode_png(&read_entry(&mut zip, "canvas.png")?)?;
    if (canvas.width(), canvas.height()) != (manifest.width, manifest.height) {
        return Err(invalid(format!(
            "canvas.png is {}x{} but manifest.json says {}x{}",
            canvas.width(),
            canvas.height(),
            manifest.width,
            manifest.height
        )));
    }
    let recipe = read_recipe(&mut zip, "recipe.txt", manifest.recipe_start)?;

    let mut versions = Vec::new();
    for v in &manifest.versions {
        versions.push(Version {
            number: v.number,
            author: author_origin(&v.author, v.run)?,
            message: v.message.clone(),
            canvas: decode_png(&read_entry(&mut zip, &version_file(v.number, "png"))?)?,
            recipe: read_recipe(&mut zip, &version_file(v.number, "txt"), v.recipe_start)?,
            vector: if v.vector {
                Some(VectorDrawing::new(text_entry(
                    &mut zip,
                    &version_file(v.number, "js"),
                )?))
            } else {
                None
            },
        });
    }

    let mut document = Document::from_parts(manifest.name, canvas, recipe, versions);
    document.notes = if zip.index_for_name("notas.md").is_some() {
        text_entry(&mut zip, "notas.md")?
    } else {
        String::new()
    };
    document.palette = manifest.palette.map(palette_from_manifest).transpose()?;
    if let Some(anim) = manifest.animation {
        let base = if anim.base {
            Some(decode_image(&read_entry(&mut zip, "anim/base.png")?)?)
        } else {
            None
        };
        document.animation = Some(Animation {
            source: text_entry(&mut zip, "anim.js")?,
            base,
            durations: anim.durations,
        });
    }
    if manifest.vector {
        document.vector = Some(VectorDrawing::new(text_entry(&mut zip, "dibujo.js")?));
    }
    Ok(document)
}

fn palette_from_manifest(p: ManifestPalette) -> Result<Palette, DomainError> {
    let mut palette = Palette::new();
    palette.name = p.name;
    for c in p.colors {
        let color =
            Rgba::parse(&c.color).map_err(|e| invalid(format!("palette: {}", e.message)))?;
        palette.push(color, c.name);
    }
    Ok(palette)
}

fn read_recipe(
    zip: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
    start: Option<[u32; 2]>,
) -> Result<Recipe, DomainError> {
    let mut recipe = Recipe::parse(&text_entry(zip, name)?)
        .map_err(|e| invalid(format!("{name}: {}", e.message)))?;
    recipe.set_start(start.map(|[w, h]| (w, h)));
    Ok(recipe)
}

fn read_entry(zip: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<Vec<u8>, DomainError> {
    let mut file = zip
        .by_name(name)
        .map_err(|_| invalid(format!("missing {name}")))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| invalid(format!("{name}: {e}")))?;
    Ok(bytes)
}

fn text_entry(zip: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<String, DomainError> {
    String::from_utf8(read_entry(zip, name)?).map_err(|_| invalid(format!("{name} is not UTF-8")))
}

fn zip_error(err: zip::result::ZipError) -> DomainError {
    DomainError::new(ErrorCode::Io, format!("zip: {err}"))
}

/// Lleva un `manifest.json` de cualquier formato conocido al actual.
///
/// - `format` 0: borrador de octubre de 2026 (`docs/03-arquitectura.md`), con
///   el tamaño como texto (`"size": "32x32"`) y sin `app` ni `recipe_start`.
///   La receta partía del tamaño del lienzo.
pub fn migrate(mut manifest: Value) -> Result<Value, DomainError> {
    let format = manifest
        .get("format")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid("manifest.json has no format"))?;
    if format > FORMAT {
        return Err(DomainError::new(
            ErrorCode::Io,
            format!(
                "unsupported .lume format {format}: this LumeArt reads formats up to {FORMAT}, update LumeArt"
            ),
        ));
    }
    let mut current = format;
    while current < FORMAT {
        manifest = match current {
            0 => migrate_v0_to_v1(manifest)?,
            _ => unreachable!("falta la migración del formato {current}"),
        };
        current += 1;
    }
    Ok(manifest)
}

fn migrate_v0_to_v1(mut m: Value) -> Result<Value, DomainError> {
    let obj = m
        .as_object_mut()
        .ok_or_else(|| invalid("manifest.json is not an object"))?;
    let size = obj
        .remove("size")
        .and_then(|s| s.as_str().map(str::to_string))
        .ok_or_else(|| invalid("format 0 manifest has no size"))?;
    let (w, h) = size
        .split_once('x')
        .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)))
        .ok_or_else(|| invalid(format!("format 0 size {size:?} is not WxH")))?;
    obj.insert("format".into(), Value::from(1));
    obj.insert("app".into(), Value::from("lumeart (format 0)"));
    obj.insert("width".into(), Value::from(w));
    obj.insert("height".into(), Value::from(h));
    obj.insert("recipe_start".into(), serde_json::json!([w, h]));
    obj.entry("palette").or_insert(Value::Null);
    if let Some(Value::Array(versions)) = obj.get_mut("versions") {
        for v in versions {
            if let Some(v) = v.as_object_mut() {
                v.entry("recipe_start").or_insert(Value::Null);
            }
        }
    }
    Ok(m)
}

/// Guarda un documento en un archivo `.lume`.
pub fn write_lume(document: &Document, path: &Path) -> Result<(), DomainError> {
    let bytes = encode_lume(document)?;
    std::fs::write(path, bytes).map_err(|err| io_error("open", path, &err))
}

/// Abre un archivo `.lume`.
pub fn read_lume(path: &Path) -> Result<Document, DomainError> {
    let bytes = std::fs::read(path).map_err(|err| io_error("open", path, &err))?;
    decode_lume(&bytes)
}

/// Importa la carpeta de una ejecución de pxcli: `final.png` (obligatorio),
/// `recipe.txt` y `meta.json` (opcionales).
///
/// Si la receta, partiendo del tamaño de `meta.json` (o del de `final.png`),
/// reproduce `final.png` tal como lo escribía pxcli, el lienzo es el de la
/// receta (con los colores semitransparentes correctos). Si no, el lienzo es
/// `final.png` y la receta queda sin tamaño de partida (no lo reconstruye).
pub fn import_pxcli_run(dir: &Path) -> Result<Document, DomainError> {
    let final_png = crate::png_io::read_png(&dir.join("final.png"))?;
    let recipe_text = match std::fs::read_to_string(dir.join("recipe.txt")) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(io_error("open", &dir.join("recipe.txt"), &err)),
    };
    let meta: Option<Value> = std::fs::read(dir.join("meta.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());

    let size_from_meta = meta.as_ref().and_then(|m| {
        let w = u32::try_from(m.get("width")?.as_u64()?).ok()?;
        let h = u32::try_from(m.get("height")?.as_u64()?).ok()?;
        Some((w, h))
    });
    let start = size_from_meta.unwrap_or((final_png.width(), final_png.height()));

    let mut recipe =
        Recipe::parse(&recipe_text).map_err(|e| invalid(format!("recipe.txt: {}", e.message)))?;
    recipe.set_start(Some(start));
    let canvas = match recipe.rebuild() {
        Ok(rebuilt) if matches_pxcli_png(&rebuilt, &final_png) => rebuilt,
        _ => {
            recipe.set_start(None);
            final_png
        }
    };

    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "pxcli".into());
    let mut document = Document::from_parts(name, canvas, recipe, Vec::new());
    if let Some(prompt) = meta.as_ref().and_then(|m| m.get("prompt")?.as_str()) {
        document.notes = format!("# Encargo (importado de pxcli)\n\n{prompt}\n");
    }
    // Una animación de pxcli: su anim.js y su base.png, como los cargaba pxcli.
    if let Ok(source) = std::fs::read_to_string(dir.join("anim.js")) {
        let base = dir.join("base.png");
        let base = if base.is_file() {
            Some(crate::png_io::read_anim_image(&base)?)
        } else {
            None
        };
        document.animation = Some(Animation {
            source,
            base,
            durations: Default::default(),
        });
    }
    Ok(document)
}

/// ¿Escribiría pxcli este lienzo como esta imagen? (Ver `pxcli_png_color`.)
pub fn matches_pxcli_png(canvas: &Canvas, png: &Canvas) -> bool {
    (canvas.width(), canvas.height()) == (png.width(), png.height())
        && canvas
            .pixels()
            .iter()
            .zip(png.pixels())
            .all(|(c, p)| crate::png_io::pxcli_png_color(*c) == *p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_domain::Executed;

    fn line(doc: &mut Document, text: &str) {
        match doc.handle_line(text, Origin::Cli) {
            Executed::Reply(r) => assert!(r.is_ok(), "{text}: {r}"),
            other => panic!("{other:?}"),
        }
    }

    fn sample() -> Document {
        let mut doc = Document::new("poción morada", 8, 8).unwrap();
        line(&mut doc, "fill_rect 2 2 4 4 #8b3fe0");
        line(&mut doc, "set_pixel 3 3 #cfdde3aa");
        doc.create_version(Origin::User, "base");
        line(&mut doc, "line 0 7 7 0 #ff000080");
        doc.create_version(Origin::Agent { run: 4 }, "Vuelta 1: sombras");
        line(&mut doc, "new 4 6");
        line(&mut doc, "set_pixel 1 1 white");
        doc.create_version(Origin::Cli, "");
        line(&mut doc, "resize 5 6");
        doc.notes =
            "Vuelta 2: falta el brillo del cristal.\nNo usar más de 12 colores. ñ ✓\n".into();
        let mut palette = Palette::fixed("pico-8").unwrap();
        palette.rename(Rgba::rgb(0xff, 0x00, 0x4d), Some("líquido".into()));
        doc.palette = Some(palette);
        doc
    }

    /// El dibujo vectorial y el de cada versión sobreviven a guardar y abrir.
    #[test]
    fn the_vector_drawing_round_trips() {
        let mut doc = Document::new("faro", 4, 4).unwrap();
        doc.vector = Some(VectorDrawing::new("canvas({ width: 400, height: 300 })\n"));
        doc.create_version(Origin::Agent { run: 2 }, "Retoque de la zona 1");
        doc.vector = Some(VectorDrawing::new(
            "canvas({ width: 400, height: 300 })\ncircle(1, 2, 3)\n",
        ));
        let back = decode_lume(&encode_lume(&doc).unwrap()).unwrap();
        assert_eq!(back.vector, doc.vector);
        assert_eq!(back.versions(), doc.versions());
        assert_eq!(
            back.versions()[0].vector.as_ref().unwrap().source,
            "canvas({ width: 400, height: 300 })\n"
        );
        let plain = decode_lume(&encode_lume(&Document::new("a", 2, 2).unwrap()).unwrap()).unwrap();
        assert_eq!(plain.vector, None);
    }

    /// La animación (código, base y duraciones) sobrevive a guardar y abrir.
    #[test]
    fn the_animation_round_trips() {
        let mut doc = sample();
        let mut base = lume_domain::anim::Image::new(3, 2);
        base.set(1, 1, Rgba::new(0x29, 0x29, 0x29, 0x29));
        let mut animation = Animation::new("action('a', { frames: 2 }, () => { base(); });\n");
        animation.base = Some(base);
        animation.durations.insert("a".into(), vec![100, 250]);
        doc.animation = Some(animation);
        let back = decode_lume(&encode_lume(&doc).unwrap()).unwrap();
        assert_eq!(back.animation, doc.animation);
        assert_same(&back, &doc);
        // Sin animación, no hay anim.js.
        let plain = decode_lume(&encode_lume(&sample()).unwrap()).unwrap();
        assert_eq!(plain.animation, None);
    }

    fn assert_same(a: &Document, b: &Document) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.canvas(), b.canvas());
        assert_eq!(a.recipe(), b.recipe());
        assert_eq!(a.versions(), b.versions());
        assert_eq!(a.notes, b.notes);
        assert_eq!(a.palette, b.palette);
    }

    #[test]
    fn save_and_open_returns_the_same_document() {
        let doc = sample();
        let back = decode_lume(&encode_lume(&doc).unwrap()).unwrap();
        assert_same(&doc, &back);
        assert_eq!(back.recipe().rebuild().unwrap(), *back.canvas());
        for v in back.versions() {
            assert_eq!(v.recipe.rebuild().unwrap(), v.canvas);
        }
        // Sin paleta elegida.
        let mut plain = sample();
        plain.palette = None;
        assert_same(&plain, &decode_lume(&encode_lume(&plain).unwrap()).unwrap());
    }

    #[test]
    fn the_zip_is_readable() {
        let doc = sample();
        let bytes = encode_lume(&doc).unwrap();
        let mut zip = ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        let names: Vec<String> = zip.file_names().map(str::to_string).collect();
        assert_eq!(
            names,
            vec![
                "manifest.json",
                "canvas.png",
                "recipe.txt",
                "notas.md",
                "versions/001.png",
                "versions/001.txt",
                "versions/002.png",
                "versions/002.txt",
                "versions/003.png",
                "versions/003.txt",
            ]
        );
        let manifest: Value =
            serde_json::from_slice(&read_entry(&mut zip, "manifest.json").unwrap()).unwrap();
        assert_eq!(manifest["format"], 1);
        assert_eq!(manifest["name"], "poción morada");
        assert_eq!(manifest["width"], 5);
        assert_eq!(manifest["recipe_start"], serde_json::json!([4, 6]));
        assert_eq!(manifest["versions"][1]["author"], "agent");
        assert_eq!(manifest["versions"][1]["run"], 4);
        assert_eq!(manifest["palette"]["name"], "pico-8");
        let recipe = text_entry(&mut zip, "recipe.txt").unwrap();
        assert_eq!(recipe, "set_pixel 1 1 #ffffffff\nresize 5 6\n");
        assert_eq!(text_entry(&mut zip, "notas.md").unwrap(), doc.notes);
        let v1 = decode_png(&read_entry(&mut zip, "versions/001.png").unwrap()).unwrap();
        assert_eq!(v1, doc.version(1).unwrap().canvas);
    }

    /// Escribe un `.lume` a mano con este manifiesto.
    fn lume_with_manifest(manifest: Value, canvas: &Canvas, recipe: &str) -> Vec<u8> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let o = SimpleFileOptions::default();
        zip.start_file("manifest.json", o).unwrap();
        zip.write_all(manifest.to_string().as_bytes()).unwrap();
        zip.start_file("canvas.png", o).unwrap();
        zip.write_all(&encode_png(canvas, 1).unwrap()).unwrap();
        zip.start_file("recipe.txt", o).unwrap();
        zip.write_all(recipe.as_bytes()).unwrap();
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn format_0_goes_through_the_migration() {
        let mut canvas = Canvas::new(4, 3).unwrap();
        canvas.set(1, 1, Rgba::rgb(255, 0, 0)).unwrap();
        let manifest = serde_json::json!({
            "format": 0,
            "name": "antiguo",
            "size": "4x3",
            "versions": [],
        });
        let doc = decode_lume(&lume_with_manifest(
            manifest.clone(),
            &canvas,
            "set_pixel 1 1 red\n",
        ))
        .unwrap();
        assert_eq!(doc.name, "antiguo");
        assert_eq!(doc.canvas(), &canvas);
        assert_eq!(doc.recipe().start(), Some((4, 3)));
        assert_eq!(doc.recipe().rebuild().unwrap(), canvas);
        assert_eq!(doc.notes, "");

        let migrated = migrate(manifest).unwrap();
        assert_eq!(migrated["format"], 1);
        assert_eq!(migrated["width"], 4);
        assert!(migrated.get("size").is_none());
    }

    #[test]
    fn unknown_or_broken_formats_give_clear_errors() {
        let canvas = Canvas::new(1, 1).unwrap();
        let future = serde_json::json!({"format": 99, "name": "x", "width": 1, "height": 1});
        let err = decode_lume(&lume_with_manifest(future, &canvas, "")).unwrap_err();
        assert_eq!(err.code, ErrorCode::Io);
        assert_eq!(
            err.message,
            "unsupported .lume format 99: this LumeArt reads formats up to 1, update LumeArt"
        );

        let no_format = serde_json::json!({"name": "x"});
        let err = decode_lume(&lume_with_manifest(no_format, &canvas, "")).unwrap_err();
        assert_eq!(
            err.message,
            "invalid .lume file: manifest.json has no format"
        );

        let err = decode_lume(b"no es un zip").unwrap_err();
        assert_eq!(err.code, ErrorCode::Io);
        assert!(
            err.message.starts_with("invalid .lume file: "),
            "{}",
            err.message
        );

        let wrong_size = serde_json::json!({"format": 1, "app": "x", "name": "x", "width": 2, "height": 2, "recipe_start": null, "palette": null});
        let err = decode_lume(&lume_with_manifest(wrong_size, &canvas, "")).unwrap_err();
        assert_eq!(
            err.message,
            "invalid .lume file: canvas.png is 1x1 but manifest.json says 2x2"
        );

        let bad_recipe = serde_json::json!({"format": 1, "app": "x", "name": "x", "width": 1, "height": 1, "recipe_start": null, "palette": null});
        let err = decode_lume(&lume_with_manifest(
            bad_recipe,
            &canvas,
            "set_pixel 0 0 red\nundo\n",
        ))
        .unwrap_err();
        assert_eq!(
            err.message,
            "invalid .lume file: recipe.txt: line 2: undo is not a drawing command"
        );
    }

    #[test]
    fn import_a_pxcli_folder() {
        let dir = tempfile::tempdir().unwrap();
        let run = dir.path().join("20261028-125955-reloj");
        std::fs::create_dir(&run).unwrap();
        // Lienzo como lo dejaría pxcli, con un color semitransparente.
        let recipe = "fill_rect 0 0 3 3 #cfdde3aa\nset_pixel 1 1 red\n";
        let mut canvas = Canvas::new(4, 4).unwrap();
        canvas
            .fill_rect(0, 0, 3, 3, Rgba::new(0xcf, 0xdd, 0xe3, 0xaa))
            .unwrap();
        canvas.set(1, 1, Rgba::rgb(255, 0, 0)).unwrap();
        let mut as_pxcli = canvas.clone();
        for y in 0..4 {
            for x in 0..4 {
                let c = as_pxcli.get(x, y).unwrap();
                as_pxcli
                    .set(x, y, crate::png_io::pxcli_png_color(c))
                    .unwrap();
            }
        }
        std::fs::write(run.join("final.png"), encode_png(&as_pxcli, 1).unwrap()).unwrap();
        std::fs::write(run.join("recipe.txt"), recipe).unwrap();
        std::fs::write(
            run.join("meta.json"),
            r#"{"prompt": "un reloj de arena", "width": 4, "height": 4}"#,
        )
        .unwrap();

        let doc = import_pxcli_run(&run).unwrap();
        assert_eq!(doc.name, "20261028-125955-reloj");
        assert_eq!(
            doc.canvas(),
            &canvas,
            "el lienzo sale de la receta, con el color correcto"
        );
        assert_eq!(doc.recipe().len(), 2);
        assert_eq!(doc.recipe().start(), Some((4, 4)));
        assert!(doc.notes.contains("un reloj de arena"));

        // Si la receta no reproduce la imagen, manda la imagen.
        std::fs::write(run.join("recipe.txt"), "set_pixel 0 0 blue\n").unwrap();
        let doc = import_pxcli_run(&run).unwrap();
        assert_eq!(doc.canvas(), &as_pxcli);
        assert_eq!(doc.recipe().start(), None);

        // Sin final.png no hay nada que importar.
        let err = import_pxcli_run(dir.path()).unwrap_err();
        assert_eq!(err.code, ErrorCode::Io);
    }
}
