//! Ejecuta guiones de `batch` y compara la salida con la de la CLI real de
//! pxcli, capturada con `scripts/pxcli-batch.sh <guion> 4x4`.

use lume_domain::batch::run_batch;
use lume_domain::{Canvas, History, Origin};

/// Igual que `scripts/pxcli-batch.sh`: las rutas relativas se resuelven en la
/// carpeta de trabajo, que en la salida aparece como `/BATCHDIR`.
fn resolve(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/BATCHDIR/{path}")
    }
}

fn check(script: &str, expected: &str) {
    let (stdout, rest) = expected
        .split_once("--- stderr ---\n")
        .expect("falta --- stderr ---");
    let (stderr, exit) = rest.split_once("--- exit ").expect("falta --- exit");

    let mut canvas = Canvas::new(4, 4).unwrap();
    let mut history = History::new();
    let report = run_batch(
        script,
        &mut canvas,
        &mut history,
        Origin::Cli,
        &resolve,
        &mut |_, _, _| Ok(()),
    )
    .expect("el guion no está vacío");

    assert_eq!(report.output(), stdout);
    match report.error() {
        Some(err) => {
            assert_eq!(format!("Error: {err}\n"), stderr);
            assert!(exit.starts_with('1'));
        }
        None => {
            assert_eq!(stderr, "");
            assert!(exit.starts_with('0'));
        }
    }
}

#[test]
fn batch_1_matches_pxcli() {
    check(
        include_str!("fixtures/batch-1.txt"),
        include_str!("fixtures/batch-1.pxcli.txt"),
    );
}

#[test]
fn batch_2_matches_pxcli() {
    check(
        include_str!("fixtures/batch-2.txt"),
        include_str!("fixtures/batch-2.pxcli.txt"),
    );
}
