//! Los `anim.js` de las ejecuciones de pxcli (`tests/fixtures/anim/`) dan
//! los mismos fotogramas que el motor de pxcli: mismo reposo, mismas
//! imágenes por acción (hash FNV-1a de sus bytes RGBA), mismas líneas, mismos
//! píxeles fuera de la celda y mismos avisos, antes y después de comprobar.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use lume_app::ports::BaseLoader;
use lume_domain::anim::check::check;
use lume_domain::anim::{Image, Issue, hex};
use lume_script::anim::render;
use serde_json::Value;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/anim")
}

/// La base como la carga pxcli: el PNG con el alfa premultiplicado.
pub fn loader(dir: PathBuf) -> BaseLoader {
    Arc::new(move |name: &str| {
        let bytes = std::fs::read(dir.join(name)).map_err(|e| e.to_string())?;
        let canvas = lume_store::decode_png(&bytes).map_err(|e| e.message)?;
        Ok(Image::from_canvas(&canvas).premultiplied())
    })
}

/// FNV-1a de 64 bits de los bytes RGBA, por filas (como el generador).
fn hash(img: &Image) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in img.pixels() {
        for b in [p.r, p.g, p.b, p.a] {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

fn issues(value: &Value) -> Vec<Issue> {
    serde_json::from_value(value.clone()).unwrap()
}

#[test]
fn every_reference_animation_draws_the_same_frames() {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    assert_eq!(dirs.len(), 19, "las 19 animaciones de reference/runs");
    let mut frames = 0;
    for dir in dirs {
        let run = dir.file_name().unwrap().to_string_lossy().into_owned();
        let source = std::fs::read_to_string(dir.join("anim.js")).unwrap();
        let want: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("pxcli.json")).unwrap())
                .unwrap();
        let mut res = render(&source, loader(dir.clone()));

        assert_eq!(issues(&want["issues"]), res.issues, "{run}: avisos");
        assert_eq!(want["name"], res.name.as_str(), "{run}");
        assert_eq!(
            (
                want["cell"][0].as_u64().unwrap(),
                want["cell"][1].as_u64().unwrap()
            ),
            (u64::from(res.cell.0), u64::from(res.cell.1)),
            "{run}: celda"
        );
        assert_eq!(
            (
                want["origin"][0].as_i64().unwrap(),
                want["origin"][1].as_i64().unwrap()
            ),
            res.origin,
            "{run}: origen"
        );
        let colors: Vec<String> = res.colors.iter().map(|c| hex(*c)).collect();
        assert_eq!(want["colors"], serde_json::json!(colors), "{run}: colores");
        assert_eq!(
            want["base"],
            res.base.as_ref().map(hash).unwrap_or_default().as_str(),
            "{run}: base"
        );
        assert_eq!(
            want["rest"],
            res.rest.as_ref().map(hash).unwrap_or_default().as_str(),
            "{run}: reposo"
        );
        let actions = want["actions"].as_array().unwrap();
        assert_eq!(actions.len(), res.actions.len(), "{run}: acciones");
        for (w, a) in actions.iter().zip(&res.actions) {
            let name = &a.spec.name;
            assert_eq!(w["name"], name.as_str(), "{run}");
            assert_eq!(w["frames"], a.spec.frames, "{run} {name}: frames");
            assert_eq!(w["fps"], a.spec.fps, "{run} {name}: fps");
            assert_eq!(w["loop"], a.spec.looping, "{run} {name}: loop");
            assert_eq!(w["line"], a.line, "{run} {name}: línea de action()");
            assert_eq!(
                w["out_of_bounds"],
                serde_json::json!(a.out_of_bounds),
                "{run} {name}: fuera de la celda"
            );
            let hashes: Vec<String> = a.images.iter().map(hash).collect();
            assert_eq!(
                w["images"],
                serde_json::json!(hashes),
                "{run} {name}: fotogramas"
            );
            frames += hashes.len();
        }

        check(&mut res);
        assert_eq!(
            issues(&want["checked"]),
            res.issues,
            "{run}: comprobaciones"
        );
    }
    assert!(frames > 50, "solo {frames} fotogramas");
}
