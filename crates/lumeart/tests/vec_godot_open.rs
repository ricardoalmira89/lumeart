//! Lo que exporta `vec godot` se abre en Godot 4: un personaje (el maniquí
//! de `reference/runs/`) y una escena (la del videojuego) dentro de un
//! proyecto mínimo; Godot, sin ventana, los carga, los añade al árbol,
//! reproduce una acción y cuenta huesos, piezas, capas y texturas. Necesita
//! Godot instalado (`GODOT` o `godot` en el PATH):
//! `cargo test -p lumeart --test vec_godot_open -- --ignored`.

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

func count(node, type):
	var n = 1 if node.is_class(type) else 0
	for c in node.get_children():
		n += count(c, type)
	return n

func missing_textures(node):
	var n = 0
	if node is Sprite2D and node.texture == null:
		n += 1
	for c in node.get_children():
		n += missing_textures(c)
	return n

func _init():
	var scene = load("res://personajes/personaje.tscn")
	if scene == null:
		print("NO CARGA personaje")
		quit(1)
		return
	var pj = scene.instantiate()
	root.add_child(pj)
	var player: AnimationPlayer = pj.get_node("AnimationPlayer")
	player.play("andar")
	player.advance(0.3)
	print("personaje huesos=", count(pj, "Bone2D"), " piezas=", count(pj, "Sprite2D"), " sin_textura=", missing_textures(pj), " acciones=", player.get_animation_list(), " alto=", pj.get_meta("alto") > 0)
	scene = load("res://niveles/escena.tscn")
	if scene == null:
		print("NO CARGA escena")
		quit(1)
		return
	var es = scene.instantiate()
	root.add_child(es)
	print("escena capas=", count(es, "Parallax2D"), " sprites=", count(es, "Sprite2D"), " sin_textura=", missing_textures(es), " ancho=", es.get_meta("ancho"))
	quit(0)
"#;

fn fixture(run: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../lume-script/tests/fixtures/vec")
        .join(run)
        .join("dibujo.js")
}

#[test]
#[ignore = "necesita Godot 4 instalado"]
fn the_exported_scenes_open_in_godot() {
    let godot = godot().expect("Godot no está instalado (usa GODOT=/ruta/godot)");
    let project = tempfile::tempdir().unwrap();
    let p = project.path();
    std::fs::write(
        p.join("project.godot"),
        "config_version=5\n\n[application]\nconfig/name=\"lumeart\"\n",
    )
    .unwrap();
    std::fs::write(p.join("comprobar.gd"), SCRIPT).unwrap();
    std::fs::copy(
        fixture("20261029-120000-vec-maniqui-de-prueba"),
        p.join("maniqui.js"),
    )
    .unwrap();
    std::fs::copy(
        fixture("20261029-130935-vec-escena-de-un-videojuego-2d-d"),
        p.join("nivel.js"),
    )
    .unwrap();
    for (file, out) in [("maniqui.js", "personajes"), ("nivel.js", "niveles")] {
        let export = std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
            .args(["vec", "godot", file, "-o", out])
            .current_dir(p)
            .env("LUMEART_VECTORCRAFT", "no")
            .env("XDG_CACHE_HOME", p.join(".cache"))
            .output()
            .unwrap();
        assert!(
            export.status.success(),
            "{}",
            String::from_utf8_lossy(&export.stderr)
        );
    }
    let run = |args: &[&str]| {
        std::process::Command::new(&godot)
            .args(["--headless", "--path"])
            .arg(p)
            .args(args)
            .output()
            .unwrap()
    };
    run(&["--import"]);
    let out = run(&["--script", "res://comprobar.gd"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}");
    assert!(
        text.contains(
            "personaje huesos=15 piezas=15 sin_textura=0 acciones=[\"RESET\", \"andar\", \"reposo\"] alto=true"
        ),
        "{text}"
    );
    // 13 capas de dibujo más los Parallax2D de los elementos con parallax
    // propio y el del fondo liso; 64 texturas, todas cargadas.
    assert!(
        text.contains("escena capas=20 sprites=64 sin_textura=0 ancho=1920"),
        "{text}"
    );
}
