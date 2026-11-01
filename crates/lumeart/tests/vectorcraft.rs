//! VectorCraft como complemento: si está instalado, `dibujo.js` puede usar
//! `freeform`, `revolve`… y `vec export` guarda `.eps` y `.vectorcraft`.
//! Necesita `vectorcraft-cli` (`LUMEART_VECTORCRAFT` o en el PATH):
//! `cargo test -p lumeart --test vectorcraft -- --ignored`.

use std::path::Path;

fn lumeart(dir: &Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
        .args(args)
        .current_dir(dir)
        .env("XDG_CACHE_HOME", dir.join(".cache"))
        .output()
        .unwrap()
}

#[test]
#[ignore = "necesita VectorCraft instalado"]
fn freeform_and_eps_work_with_vectorcraft() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    // El vigía de reference/runs usa freeform (un degradado libre).
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../lume-script/tests/fixtures/vec/20261030-153943-vec-vigia/dibujo.js"),
        dir.join("dibujo.js"),
    )
    .unwrap();
    let out = lumeart(dir, &["vec", "check"]);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "1024x1024 · 40 formas · SVG de 44 KB\n",
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = lumeart(dir, &["vec", "export", "-o", "vigia.eps"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let eps = std::fs::read(dir.join("vigia.eps")).unwrap();
    // EPS, a secas o con la cabecera binaria de DOS (con vista previa).
    let dos = eps.starts_with(&[0xC5, 0xD0, 0xD3, 0xC6]);
    assert!(eps.starts_with(b"%!PS") || dos, "no es EPS");
    assert!(String::from_utf8_lossy(&eps).contains("%!PS-Adobe-3.0 EPSF-3.0"));
    let out = lumeart(dir, &["vec", "export", "-o", "vigia.vectorcraft"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
