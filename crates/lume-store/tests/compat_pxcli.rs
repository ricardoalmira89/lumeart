//! Banco de compatibilidad con pxcli: misma `recipe.txt` → mismo `final.png`.
//!
//! - `fixtures_reproduce_final_png`: 4 ejecuciones pequeñas copiadas en
//!   `tests/fixtures/pxcli-runs/` (corre siempre, también en CI).
//! - `all_reference_runs`: todas las ejecuciones de pixel art con receta de
//!   `reference/runs/` (la crea `scripts/setup-reference.sh`); sin esa
//!   carpeta, se salta con un mensaje.

use std::path::Path;

use lume_store::compat::{compare_pxcli_run, pixel_art_runs, report};

#[test]
fn fixtures_reproduce_final_png() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pxcli-runs");
    let runs = pixel_art_runs(&root);
    assert!(runs.len() >= 3, "faltan fixtures en {}", root.display());
    let results: Vec<_> = runs.iter().map(|d| compare_pxcli_run(d).unwrap()).collect();
    eprint!("{}", report(&results));
    for r in &results {
        assert!(
            r.is_exact(),
            "{} no reproduce final.png: {:.3} %",
            r.name,
            r.percent()
        );
    }
}

#[test]
fn all_reference_runs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reference/runs");
    if !root.is_dir() {
        eprintln!(
            "se salta: no existe reference/runs (ejecuta scripts/setup-reference.sh para probar todas las ejecuciones de pxcli)"
        );
        return;
    }
    let runs = pixel_art_runs(&root);
    let results: Vec<_> = runs
        .iter()
        .map(|d| compare_pxcli_run(d).unwrap_or_else(|e| panic!("{}: {e}", d.display())))
        .collect();
    eprintln!(
        "Banco de compatibilidad con pxcli ({} ejecuciones de pixel art con receta):",
        results.len()
    );
    eprint!("{}", report(&results));
    assert!(
        !results.is_empty(),
        "reference/runs no tiene ejecuciones con recipe.txt y final.png"
    );
    let failing: Vec<_> = results
        .iter()
        .filter(|r| !r.is_exact())
        .map(|r| r.name.as_str())
        .collect();
    assert!(failing.is_empty(), "no reproducen final.png: {failing:?}");
}

/// La comparación detecta diferencias (para que el banco no pase siempre).
#[test]
fn a_wrong_recipe_is_detected() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pxcli-runs/20261028-144533-dibujo");
    let dir = tempfile::tempdir().unwrap();
    for f in ["final.png", "meta.json"] {
        std::fs::copy(src.join(f), dir.path().join(f)).unwrap();
    }
    let mut recipe = std::fs::read_to_string(src.join("recipe.txt")).unwrap();
    recipe.push_str("set_pixel 0 0 #123456\n");
    std::fs::write(dir.path().join("recipe.txt"), recipe).unwrap();
    let r = compare_pxcli_run(dir.path()).unwrap();
    assert!(!r.is_exact());
    assert_eq!(r.total - r.matching, 1);

    // Tamaño distinto: no coincide ningún píxel.
    std::fs::write(
        dir.path().join("meta.json"),
        r#"{"width": 16, "height": 16}"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("recipe.txt"), "clear\n").unwrap();
    let r = compare_pxcli_run(dir.path()).unwrap();
    assert_eq!(r.matching, 0);
}
