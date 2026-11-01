//! Importa las ejecuciones reales de pxcli de `reference/runs/` (si existe;
//! la crea `scripts/setup-reference.sh`). Sin `reference/`, se salta.

use std::path::Path;

use lume_store::lume_file::{decode_lume, encode_lume, import_pxcli_run};
use lume_store::png_io::read_png;

#[test]
fn import_reference_runs() {
    let runs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reference/runs");
    if !runs.is_dir() {
        eprintln!("sin reference/runs: se salta (ejecuta scripts/setup-reference.sh)");
        return;
    }
    let mut imported = 0;
    let mut rebuilt = 0;
    let mut entries: Vec<_> = std::fs::read_dir(&runs)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for dir in entries {
        if !dir.join("final.png").is_file() || !dir.join("recipe.txt").is_file() {
            continue;
        }
        let doc = import_pxcli_run(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let png = read_png(&dir.join("final.png")).unwrap();
        assert_eq!(
            (doc.canvas().width(), doc.canvas().height()),
            (png.width(), png.height())
        );
        assert!(!doc.recipe().is_empty(), "{}", dir.display());
        if doc.recipe().start().is_some() {
            assert_eq!(&doc.recipe().rebuild().unwrap(), doc.canvas());
            rebuilt += 1;
        }
        // Y se puede guardar como .lume y volver a abrir igual.
        let back = decode_lume(&encode_lume(&doc).unwrap()).unwrap();
        assert_eq!(back.canvas(), doc.canvas());
        assert_eq!(back.recipe(), doc.recipe());
        imported += 1;
    }
    eprintln!(
        "importadas {imported} ejecuciones de pxcli; la receta reproduce final.png en {rebuilt}"
    );
    assert!(
        imported > 0,
        "no había ninguna ejecución con final.png y recipe.txt"
    );
}
