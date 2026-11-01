//! Los `dibujo.js` de las ejecuciones de pxcli (`tests/fixtures/vec/`) dan
//! el mismo SVG que el motor de pxcli (goja): mismo SVG del studio y mismo
//! SVG limpio (hash FNV-1a de 64 bits y longitud), mismo tamaño, mismas
//! formas y mismos avisos. Las referencias las genera
//! `scripts/vec-golden/dibujos/`.

use std::path::{Path, PathBuf};

use lume_domain::vec::Issue;
use lume_script::vec::render;
use serde_json::Value;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vec")
}

/// FNV-1a de 64 bits de los bytes (como el generador).
fn hash(text: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

#[test]
fn every_reference_drawing_gives_pxclis_svg() {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    assert_eq!(dirs.len(), 94, "los 94 dibujo.js de reference/runs");
    let mut failures = Vec::new();
    for dir in &dirs {
        let run = dir.file_name().unwrap().to_string_lossy().into_owned();
        let source = std::fs::read_to_string(dir.join("dibujo.js")).unwrap();
        let want: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("pxcli.json")).unwrap())
                .unwrap();
        let res = render(&source);
        let issues: Vec<Issue> = serde_json::from_value(want["issues"].clone()).unwrap();
        let mut wrong = Vec::new();
        if (res.width, res.height, res.count)
            != (
                want["width"].as_i64().unwrap(),
                want["height"].as_i64().unwrap(),
                want["count"].as_i64().unwrap(),
            )
        {
            wrong.push(format!(
                "tamaño/formas {}x{} {} (pxcli {}x{} {})",
                res.width, res.height, res.count, want["width"], want["height"], want["count"]
            ));
        }
        if res.issues != issues {
            wrong.push(format!("avisos {:?} (pxcli {:?})", res.issues, issues));
        }
        if (hash(&res.svg), res.svg.len() as u64)
            != (
                want["svg_hash"].as_str().unwrap().to_string(),
                want["svg_len"].as_u64().unwrap(),
            )
        {
            wrong.push(format!(
                "SVG del studio ({} bytes, pxcli {})",
                res.svg.len(),
                want["svg_len"]
            ));
        }
        if (hash(&res.clean), res.clean.len() as u64)
            != (
                want["clean_hash"].as_str().unwrap().to_string(),
                want["clean_len"].as_u64().unwrap(),
            )
        {
            wrong.push(format!(
                "SVG limpio ({} bytes, pxcli {})",
                res.clean.len(),
                want["clean_len"]
            ));
        }
        if !wrong.is_empty() {
            failures.push(format!("{run}: {}", wrong.join("; ")));
        }
    }
    assert!(
        failures.is_empty(),
        "{} de {} distintos:\n{}",
        failures.len(),
        dirs.len(),
        failures.join("\n")
    );
}
