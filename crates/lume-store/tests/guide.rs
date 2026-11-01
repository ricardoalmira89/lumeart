//! `lumeart.toml`: se escribe comentado, se lee igual, se valida con errores
//! claros (con su línea) y `init_guide` no pisa una guía sin `force`.

use lume_domain::{ArtGuide, GUIDE_FILE, GuidePalette, Naming, Rgba};
use lume_store::guide::{
    GuideFileError, Problem, find_guide, guide_from_toml, guide_to_toml, init_guide, load_guide,
};

fn custom() -> ArtGuide {
    ArtGuide {
        style: "pixel art de 16 bits, contorno oscuro".into(),
        size: (16, 24),
        palette: GuidePalette::Custom(vec![
            Rgba::rgb(0x1a, 0x1c, 0x2c),
            Rgba::new(0xf4, 0xf4, 0xf4, 0x80),
        ]),
        output: "res/sprites".into(),
        naming: Naming::Kebab,
    }
}

#[test]
fn the_guide_round_trips() {
    for guide in [
        ArtGuide::default(),
        custom(),
        ArtGuide {
            palette: GuidePalette::Fixed("pico-8".into()),
            style: "comillas \" y\nsaltos".into(),
            ..ArtGuide::default()
        },
    ] {
        let text = guide_to_toml(&guide);
        assert_eq!(guide_from_toml(&text), Ok(guide.clone()), "{text}");
    }
}

#[test]
fn the_default_file_explains_each_key() {
    let text = guide_to_toml(&ArtGuide::default());
    for line in [
        "style = \"\"",
        "size = \"32x32\"",
        "palette = \"libre\"",
        "output = \"assets/art\"",
        "naming = \"snake_case\"",
        "# Paleta: \"libre\" (sin límite), una fija (\"pico-8\", \"game-boy\")",
        "# Nombres de archivo: \"snake_case\", \"kebab-case\", \"camelCase\", \"PascalCase\".",
    ] {
        assert!(text.contains(line), "falta {line:?} en:\n{text}");
    }
    assert!(
        guide_to_toml(&custom()).contains("palette = [\"#1a1c2c\", \"#f4f4f480\"]"),
        "{}",
        guide_to_toml(&custom())
    );
}

#[test]
fn missing_keys_take_the_defaults() {
    assert_eq!(guide_from_toml(""), Ok(ArtGuide::default()));
    let guide = guide_from_toml("size = \"8x8\"\n").unwrap();
    assert_eq!(guide.size, (8, 8));
    assert_eq!(guide.output, "assets/art");
}

fn problems(text: &str) -> Vec<String> {
    guide_from_toml(text)
        .unwrap_err()
        .iter()
        .map(Problem::to_string)
        .collect()
}

#[test]
fn every_wrong_value_is_reported_with_its_line() {
    let text = "# guía\nstyle = 3\nsize = \"32\"\npalette = [\"#000000\", \"#nope\"]\noutput = \"../fuera\"\nnaming = \"snake\"\n";
    assert_eq!(
        problems(text),
        [
            "línea 2: style: tiene que ser un texto entre comillas, no un número",
            "línea 3: size: \"32\" no es un tamaño; usa ANCHOxALTO, por ejemplo \"32x32\"",
            "línea 4: palette: el color 2 (\"#nope\") no es válido; usa #rrggbb, #rrggbbaa o un nombre como \"red\"",
            "línea 5: output: \"../fuera\" sale de la carpeta del proyecto; usa una carpeta dentro de ella",
            "línea 6: naming: \"snake\" no es una convención de nombres; usa \"snake_case\", \"kebab-case\", \"camelCase\" o \"PascalCase\"",
        ]
    );
    assert_eq!(
        problems("palette = \"nes\""),
        [
            "línea 1: palette: no hay ninguna paleta llamada \"nes\"; usa \"libre\", \"pico-8\", \"game-boy\" o una lista de colores como [\"#1a1c2c\", \"#f4f4f4\"]"
        ]
    );
    assert_eq!(
        problems("palette = 4"),
        [
            "línea 1: palette: tiene que ser un nombre entre comillas o una lista de colores, no un número"
        ]
    );
    assert_eq!(
        problems("palette = [\"#000000\", 7]"),
        [
            "línea 1: palette: el color 2 tiene que ser un texto entre comillas, como \"#1a1c2c\", no un número"
        ]
    );
    assert_eq!(
        problems("size = \"600x8\""),
        ["línea 1: size: \"600x8\": el ancho y el alto van de 1 a 512"]
    );
}

#[test]
fn unknown_keys_and_bad_toml_are_errors() {
    assert_eq!(
        problems("size = \"8x8\"\npallete = \"pico-8\"\n"),
        [
            "línea 2: clave desconocida `pallete`; las claves válidas son `style`, `size`, `palette`, `output`, `naming`"
        ]
    );
    assert_eq!(
        problems("size = \"8x8\"\nsize = \"9x9\"\n"),
        ["línea 2: clave repetida"]
    );
    let bad = problems("size = \"8x8\"\nstyle = \n");
    assert_eq!(bad.len(), 1);
    assert!(
        bad[0].starts_with("línea 2: no es TOML válido: "),
        "{bad:?}"
    );
}

#[test]
fn load_says_which_file_is_wrong() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(GUIDE_FILE);
    std::fs::write(&path, "size = \"grande\"\nnaming = 1\n").unwrap();
    let err = load_guide(&path).unwrap_err();
    assert_eq!(
        err.to_string(),
        format!(
            "{} no es una guía de arte válida:\n  línea 1: size: \"grande\" no es un tamaño; usa ANCHOxALTO, por ejemplo \"32x32\"\n  línea 2: naming: tiene que ser un texto entre comillas, no un número",
            path.display()
        )
    );
    let missing = load_guide(&dir.path().join("no-existe.toml")).unwrap_err();
    assert!(matches!(missing, GuideFileError::Io { .. }), "{missing:?}");
}

#[test]
fn init_does_not_overwrite_without_force() {
    let dir = tempfile::tempdir().unwrap();
    let path = init_guide(dir.path(), &ArtGuide::default(), false).unwrap();
    assert_eq!(path, dir.path().join("lumeart.toml"));
    assert_eq!(load_guide(&path), Ok(ArtGuide::default()));

    // Lo que haya escrito el usuario se queda como estaba.
    std::fs::write(&path, "style = \"mío\"\n").unwrap();
    let err = init_guide(dir.path(), &custom(), false).unwrap_err();
    assert_eq!(err, GuideFileError::Exists(path.clone()));
    assert_eq!(
        err.to_string(),
        format!(
            "ya existe {}; usa --force para sobrescribirlo",
            path.display()
        )
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "style = \"mío\"\n");

    // Con `force`, sí.
    init_guide(dir.path(), &custom(), true).unwrap();
    assert_eq!(load_guide(&path), Ok(custom()));
    let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert_eq!(leftovers.len(), 1, "sin archivos temporales");
}

#[test]
fn the_guide_is_found_from_a_subfolder() {
    let dir = tempfile::tempdir().unwrap();
    let deep = dir.path().join("src/escenas/nivel1");
    std::fs::create_dir_all(&deep).unwrap();
    assert_eq!(find_guide(&deep), None);
    let path = init_guide(dir.path(), &ArtGuide::default(), false).unwrap();
    assert_eq!(find_guide(&deep), Some(path.clone()));
    assert_eq!(find_guide(dir.path()), Some(path));
}

#[test]
fn the_project_store_finds_the_guide_and_lists_the_assets() {
    use lume_app::ProjectStore;
    use lume_app::ports::PortError;
    use lume_store::FsProject;

    let dir = tempfile::tempdir().unwrap();
    let deep = dir.path().join("src");
    std::fs::create_dir_all(&deep).unwrap();
    assert_eq!(FsProject.find_guide(&deep), Ok(None));

    std::fs::write(dir.path().join(GUIDE_FILE), "size = \"16x16\"\n").unwrap();
    let found = FsProject.find_guide(&deep).unwrap().unwrap();
    assert_eq!(found.path, dir.path().join(GUIDE_FILE));
    assert_eq!(found.project(), dir.path());
    assert_eq!(found.text, "size = \"16x16\"\n");
    assert_eq!(found.guide.size, (16, 16));

    std::fs::write(dir.path().join(GUIDE_FILE), "size = 16\n").unwrap();
    let Err(PortError::Invalid(message)) = FsProject.find_guide(&deep) else {
        panic!("una guía mal escrita es un error");
    };
    assert!(
        message.contains("línea 1: size: tiene que ser un texto"),
        "{message}"
    );

    let out = dir.path().join("assets/art");
    assert_eq!(FsProject.list_assets(&out), Ok(vec![]));
    FsProject.prepare_output(&out).unwrap();
    assert!(out.is_dir());
    let mut document = lume_domain::Document::new("cofre", 4, 4).unwrap();
    document.create_version(lume_domain::Origin::Agent { run: 1 }, "Vuelta 1 de 1");
    std::fs::write(
        out.join("cofre.lume"),
        lume_store::encode_lume(&document).unwrap(),
    )
    .unwrap();
    std::fs::write(out.join("cofre.png"), b"png").unwrap();
    std::fs::write(out.join("llave.png"), b"png").unwrap();
    std::fs::write(out.join("notas.txt"), b"x").unwrap();
    let assets = FsProject.list_assets(&out).unwrap();
    let names: Vec<&str> = assets.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["cofre", "llave"]);
    assert_eq!(assets[0].versions, Some(1));
    assert_eq!(
        assets[0].png.as_deref(),
        Some(out.join("cofre.png").as_path())
    );
    assert_eq!(assets[1].lume, None);
    assert_eq!(assets[1].versions, None);
}
