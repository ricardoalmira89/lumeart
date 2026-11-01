//! El `SpriteFrames` exportado se abre en Godot 4: se exporta el reloj
//! dentro de un proyecto mínimo y Godot, sin ventana, lo carga y cuenta sus
//! animaciones. Necesita Godot instalado (`GODOT` o `godot` en el PATH):
//! `cargo test -p lumeart --test anim_godot -- --ignored`.

use std::path::{Path, PathBuf};

fn godot() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("GODOT") {
        return Some(PathBuf::from(path));
    }
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|dir| dir.join("godot"))
        .find(|p| p.is_file())
}

const SCRIPT: &str = r#"extends SceneTree

func _init():
	var frames: SpriteFrames = load("res://arte/reloj-de-arena_12x12.tres")
	if frames == null:
		print("NO CARGA")
		quit(1)
		return
	for anim in frames.get_animation_names():
		var tex: AtlasTexture = frames.get_frame_texture(anim, 0)
		print(anim, " ", frames.get_frame_count(anim), " ", frames.get_animation_speed(anim), " ", frames.get_animation_loop(anim), " ", tex.atlas.get_size())
	quit(0)
"#;

#[test]
#[ignore = "necesita Godot 4 instalado"]
fn the_sprite_frames_open_in_godot() {
    let godot = godot().expect("Godot no está instalado (usa GODOT=/ruta/godot)");
    let project = tempfile::tempdir().unwrap();
    let art = project.path().join("arte");
    std::fs::create_dir_all(&art).unwrap();
    std::fs::write(
        project.path().join("project.godot"),
        "config_version=5\n\n[application]\nconfig/name=\"lumeart\"\n",
    )
    .unwrap();
    std::fs::write(project.path().join("comprobar.gd"), SCRIPT).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/anim_cli/reloj");
    for file in ["anim.js", "base.png"] {
        std::fs::copy(fixture.join(file), art.join(file)).unwrap();
    }
    let export = std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
        .args(["anim", "export", "--godot"])
        .current_dir(&art)
        .output()
        .unwrap();
    assert!(export.status.success());
    let tres = std::fs::read_to_string(art.join("reloj-de-arena_12x12.tres")).unwrap();
    assert!(tres.contains("path=\"res://arte/reloj-de-arena_12x12.png\""));

    let run = |args: &[&str]| {
        std::process::Command::new(&godot)
            .args(["--headless", "--path"])
            .arg(project.path())
            .args(args)
            .output()
            .unwrap()
    };
    run(&["--import"]);
    let out = run(&["--script", "res://comprobar.gd"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}");
    assert!(text.contains("caer 4 8.0 true (48.0, 36.0)"), "{text}");
    assert!(text.contains("salto 3 8.0 false"), "{text}");
    assert!(text.contains("espejo 2 8.0 true"), "{text}");
}
