//! Lo que hace falta para publicar: los iconos de cada plataforma, el
//! lanzador de Linux, el paquete .deb y el workflow de publicación.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(root().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn png_size(png: &[u8]) -> (u32, u32) {
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "no es un PNG");
    (
        u32::from_be_bytes(png[16..20].try_into().unwrap()),
        u32::from_be_bytes(png[20..24].try_into().unwrap()),
    )
}

/// El `.icns` de macOS: cabecera, longitud y una entrada PNG por tamaño, del
/// 16 al 1024 (con los @2x).
#[test]
fn the_macos_icon_has_every_size() {
    let icns = read("assets/brand/icon.icns");
    assert_eq!(&icns[..4], b"icns");
    assert_eq!(
        u32::from_be_bytes(icns[4..8].try_into().unwrap()) as usize,
        icns.len()
    );
    let mut entries = Vec::new();
    let mut at = 8;
    while at < icns.len() {
        let kind = String::from_utf8(icns[at..at + 4].to_vec()).unwrap();
        let len = u32::from_be_bytes(icns[at + 4..at + 8].try_into().unwrap()) as usize;
        let (w, h) = png_size(&icns[at + 8..at + len]);
        assert_eq!(w, h);
        entries.push((kind, w));
        at += len;
    }
    assert_eq!(at, icns.len());
    let expected = [
        ("icp4", 16),
        ("icp5", 32),
        ("ic11", 32),
        ("icp6", 64),
        ("ic12", 64),
        ("ic07", 128),
        ("ic08", 256),
        ("ic13", 256),
        ("ic09", 512),
        ("ic14", 512),
        ("ic10", 1024),
    ];
    let got: Vec<(&str, u32)> = entries.iter().map(|(k, w)| (k.as_str(), *w)).collect();
    assert_eq!(got, expected);
}

/// El `.ico` de Windows (el que lleva el `.exe`): del 16 al 256.
#[test]
fn the_windows_icon_has_every_size() {
    let ico = read("assets/brand/icon.ico");
    assert_eq!(&ico[..4], &[0, 0, 1, 0], "cabecera ICO");
    let count = u16::from_le_bytes([ico[4], ico[5]]) as usize;
    let sizes: Vec<u32> = (0..count)
        .map(|i| {
            let w = ico[6 + i * 16];
            if w == 0 { 256 } else { u32::from(w) }
        })
        .collect();
    for size in [16, 24, 32, 48, 64, 128, 256] {
        assert!(sizes.contains(&size), "falta {size}: {sizes:?}");
    }
    let build = std::fs::read_to_string(root().join("crates/lumeart/build.rs")).unwrap();
    assert!(build.contains("../../assets/brand/icon.ico"));
}

/// El lanzador de Linux abre `lumeart` con su icono y casa con la ventana
/// (`app_id` y `StartupWMClass` iguales).
#[test]
fn the_linux_launcher_matches_the_window() {
    let desktop = String::from_utf8(read("assets/linux/lumeart.desktop")).unwrap();
    for line in [
        "[Desktop Entry]",
        "Type=Application",
        "Name=LumeArt",
        "Exec=lumeart",
        "Icon=lumeart",
        "StartupWMClass=lumeart",
    ] {
        assert!(desktop.lines().any(|l| l == line), "falta {line}");
    }
    let ui = std::fs::read_to_string(root().join("crates/lume-ui/src/lib.rs")).unwrap();
    assert!(ui.contains(".with_app_id(\"lumeart\")"));
    assert_eq!(png_size(&read("assets/brand/icon-256.png")), (256, 256));
}

/// Al subir un tag `v*`, `release.yml` compila para Linux (.deb y .tar.gz),
/// Windows (.zip) y macOS (Apple Silicon e Intel, .app) y publica la versión;
/// el README nombra esos mismos paquetes.
#[test]
fn the_release_builds_every_platform_on_a_tag() {
    let workflow = String::from_utf8(read(".github/workflows/release.yml")).unwrap();
    assert!(
        workflow.contains("tags: [\"v*\"]"),
        "se lanza con un tag v*"
    );
    for job in ["linux:", "windows:", "macos:", "publish:"] {
        assert!(
            workflow.contains(&format!("\n  {job}")),
            "falta el trabajo {job}"
        );
    }
    for target in ["aarch64-apple-darwin", "x86_64-apple-darwin"] {
        assert!(workflow.contains(target), "falta {target}");
    }
    // Las librerías de la ventana para compilar en Linux.
    for lib in ["libxkbcommon-dev", "libwayland-dev", "libgl1-mesa-dev"] {
        assert!(workflow.contains(lib), "falta {lib}");
    }
    // Lo que se copia en los paquetes existe.
    for path in [
        "assets/linux/lumeart.desktop",
        "assets/brand/icon-256.png",
        "assets/brand/icon.icns",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "README.md",
    ] {
        assert!(workflow.contains(path), "el workflow no usa {path}");
        assert!(root().join(path).is_file(), "falta {path}");
    }

    // El .deb (cargo deb): sus archivos existen.
    let manifest: toml::Value =
        toml::from_str(&String::from_utf8(read("crates/lumeart/Cargo.toml")).unwrap()).unwrap();
    let deb = &manifest["package"]["metadata"]["deb"];
    assert_eq!(deb["name"].as_str(), Some("lumeart"));
    for asset in deb["assets"].as_array().unwrap() {
        let source = asset[0].as_str().unwrap();
        if source.starts_with("target/") {
            continue;
        }
        let path = root().join("crates/lumeart").join(source);
        assert!(path.is_file(), "{}", path.display());
    }

    // El README dice cómo instalar cada paquete que se publica.
    let readme = String::from_utf8(read("README.md")).unwrap();
    for package in [
        "lumeart_<versión>_amd64.deb",
        "lumeart-<versión>-linux-x86_64.tar.gz",
        "lumeart-<versión>-windows-x86_64.zip",
        "lumeart-<versión>-macos-aarch64.zip",
        "lumeart-<versión>-macos-x86_64.zip",
    ] {
        assert!(readme.contains(package), "el README no nombra {package}");
    }
    for made in [
        "lumeart_${version}_amd64.deb",
        "lumeart-${version}-linux-x86_64",
        "lumeart-$version-windows-x86_64",
        "lumeart-${version}-macos-${{ matrix.arch }}.zip",
    ] {
        assert!(workflow.contains(made), "el workflow no hace {made}");
    }
}
