//! Comprueba las reglas de dependencia entre crates (arquitectura hexagonal).
//!
//! Si este test falla, NO lo cambies para que pase: arregla la dependencia.
//! Solo se modifica cuando se toma una decisión de arquitectura nueva, y esa
//! decisión se anota en `docs/02-decisiones.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Crates internos que cada crate puede usar en `[dependencies]`.
fn allowed_internal() -> BTreeMap<&'static str, BTreeSet<&'static str>> {
    let adapter = ["lume-app", "lume-domain"];
    let mut m = BTreeMap::new();
    m.insert("lume-domain", BTreeSet::new());
    m.insert("lume-app", BTreeSet::from(["lume-domain"]));
    for c in [
        "lume-script",
        "lume-raster",
        "lume-store",
        "lume-agent",
        "lume-api",
        "lume-mcp",
        "lume-ui",
    ] {
        m.insert(c, BTreeSet::from(adapter));
    }
    m.insert("lume-cli", BTreeSet::from(["lume-domain"]));
    m.insert(
        "lumeart",
        BTreeSet::from([
            "lume-domain",
            "lume-app",
            "lume-script",
            "lume-raster",
            "lume-store",
            "lume-agent",
            "lume-api",
            "lume-mcp",
            "lume-ui",
            "lume-cli",
        ]),
    );
    m
}

/// Dependencias externas permitidas en el dominio (debe seguir siendo puro).
const DOMAIN_EXTERNAL_ALLOWED: &[&str] = &["serde", "thiserror"];

/// Nombres de las dependencias normales (no dev ni build) de un Cargo.toml.
fn normal_dependencies(manifest: &str) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    let mut in_deps = false;
    for raw in manifest.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_deps = line == "[dependencies]"
                || line.starts_with("[target.") && line.ends_with(".dependencies]");
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name: String = line
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !name.is_empty() {
            deps.insert(name);
        }
    }
    deps
}

#[test]
fn crates_only_depend_inwards() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let allowed = allowed_internal();
    let mut errors = Vec::new();

    for (krate, ok) in &allowed {
        let manifest_path = crates_dir.join(krate).join("Cargo.toml");
        let manifest = fs::read_to_string(&manifest_path)
            .unwrap_or_else(|e| panic!("no puedo leer {}: {e}", manifest_path.display()));
        for dep in normal_dependencies(&manifest) {
            let internal = dep.starts_with("lume-") || dep == "lumeart";
            if internal && !ok.contains(dep.as_str()) {
                errors.push(format!("{krate} no puede depender de {dep}"));
            }
            if *krate == "lume-domain"
                && !internal
                && !DOMAIN_EXTERNAL_ALLOWED.contains(&dep.as_str())
            {
                errors.push(format!(
                    "lume-domain debe ser puro: {dep} no está permitido (solo {DOMAIN_EXTERNAL_ALLOWED:?})"
                ));
            }
        }
    }

    assert!(
        errors.is_empty(),
        "reglas de arquitectura rotas:\n{}",
        errors.join("\n")
    );
}

#[test]
fn every_workspace_crate_is_checked() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let allowed = allowed_internal();
    for entry in fs::read_dir(&crates_dir).expect("carpeta crates") {
        let entry = entry.expect("entrada");
        if entry.path().join("Cargo.toml").exists() {
            let name = entry.file_name().to_string_lossy().to_string();
            assert!(
                allowed.contains_key(name.as_str()),
                "el crate {name} no tiene reglas en tests/arquitectura.rs"
            );
        }
    }
}

/// El workspace deniega `unsafe` y solo hay una excepción: la función de
/// Windows de `crates/lumeart/src/daemon.rs` que impide que la sesión herede
/// la salida de quien la lanza (`docs/02-decisiones.md`). Ninguna otra.
#[test]
fn unsafe_is_only_allowed_in_one_place() {
    fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(dir).expect("carpeta") {
            let path = entry.expect("entrada").path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let workspace = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(workspace.contains("unsafe_code = \"deny\""));

    let mut files = Vec::new();
    rust_files(&root.join("crates"), &mut files);
    let mut found = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        let count = text.matches("allow(unsafe_code)").count();
        let name = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        // Este mismo archivo nombra el atributo en el texto de arriba.
        if name == "crates/lumeart/tests/arquitectura.rs" {
            continue;
        }
        if count > 0 {
            found.push((name, count));
        }
    }
    assert_eq!(found, [("crates/lumeart/src/daemon.rs".to_string(), 1)]);
}
