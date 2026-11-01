//! `lumeart init` con el binario real: crea `lumeart.toml` en la carpeta
//! actual, no pisa uno que ya exista sin `--force` y valida sus opciones.

use std::path::Path;
use std::process::Output;

use lume_domain::{ArtGuide, GuidePalette, Naming, Rgba};
use lume_store::guide::load_guide;

fn lumeart(cwd: &Path, args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}

/// La carpeta como la ve el programa desde dentro: en macOS, la temporal
/// está en /var, que es un enlace a /private/var, y el programa ve la real.
fn seen(dir: &Path) -> std::path::PathBuf {
    if cfg!(target_os = "macos") {
        std::fs::canonicalize(dir).unwrap()
    } else {
        dir.to_path_buf()
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn init_creates_the_guide_and_does_not_overwrite_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = seen(dir.path()).join("lumeart.toml");

    let out = lumeart(dir.path(), &["init"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        format!(
            "Guía de arte creada en {}\nLos recursos irán a assets/art/. Edita el archivo para cambiar el estilo, el tamaño o la paleta.\n",
            path.display()
        )
    );
    assert_eq!(load_guide(&path), Ok(ArtGuide::default()));

    // El usuario lo cambia; otro `init` no lo toca.
    std::fs::write(&path, "style = \"acuarela\"\n").unwrap();
    let out = lumeart(dir.path(), &["init", "--size", "8x8"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        text(&out.stderr),
        format!(
            "Error: ya existe {}; usa --force para sobrescribirlo\n",
            path.display()
        )
    );
    assert!(out.stdout.is_empty());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "style = \"acuarela\"\n"
    );

    // Con `--force`, sí, y con los valores pedidos.
    let out = lumeart(
        dir.path(),
        &[
            "init",
            "--force",
            "--style",
            "pixel art de Game Boy",
            "--size",
            "16x16",
            "--palette",
            "#0f380f, #306230,#8bac0f",
            "--output",
            "./res/sprites/",
            "--naming",
            "PascalCase",
        ],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("Los recursos irán a res/sprites/."));
    assert_eq!(
        load_guide(&path),
        Ok(ArtGuide {
            style: "pixel art de Game Boy".into(),
            size: (16, 16),
            palette: GuidePalette::Custom(vec![
                Rgba::rgb(0x0f, 0x38, 0x0f),
                Rgba::rgb(0x30, 0x62, 0x30),
                Rgba::rgb(0x8b, 0xac, 0x0f),
            ]),
            output: "res/sprites".into(),
            naming: Naming::Pascal,
        })
    );

    let out = lumeart(dir.path(), &["init", "--force", "--palette", "pico-8"]);
    assert!(out.status.success());
    assert_eq!(
        load_guide(&path).unwrap().palette,
        GuidePalette::Fixed("pico-8".into())
    );
}

#[test]
fn init_rejects_bad_options_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    for (args, error) in [
        (
            &["init", "--size", "grande"][..],
            "Error: --size: \"grande\" no es un tamaño; usa ANCHOxALTO, por ejemplo \"32x32\"\n",
        ),
        (
            &["init", "--palette", "nes"],
            "Error: --palette: no hay ninguna paleta llamada \"nes\"; usa \"libre\", \"pico-8\", \"game-boy\" o una lista de colores como [\"#1a1c2c\", \"#f4f4f4\"]\n",
        ),
        (
            &["init", "--output", "/tmp/arte"],
            "Error: --output: \"/tmp/arte\" es una ruta absoluta; escribe la carpeta relativa a lumeart.toml, por ejemplo \"assets/art\"\n",
        ),
        (
            &["init", "--naming", "UPPER"],
            "Error: --naming: \"UPPER\" no es una convención de nombres; usa \"snake_case\", \"kebab-case\", \"camelCase\" o \"PascalCase\"\n",
        ),
    ] {
        let out = lumeart(dir.path(), args);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert_eq!(text(&out.stderr), error);
    }
    assert!(!dir.path().join("lumeart.toml").exists());
}
