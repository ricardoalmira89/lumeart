//! `lumeart eval` con el binario real: los errores sin evaluación, sin
//! plugin o con encargos que no valen, y (con `#[ignore]`, porque usa Claude
//! Code de verdad) una evaluación pequeña con el plugin oficial.

use std::path::Path;
use std::process::Output;

/// El binario con ajustes, datos y caché en la carpeta del test.
fn lumeart(dir: &Path, args: &[&str], stdin: &str) -> Output {
    use std::io::Write;
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
        .args(args)
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", dir.join(".config"))
        .env("XDG_DATA_HOME", dir.join(".data"))
        .env("XDG_CACHE_HOME", dir.join(".cache"))
        .env("CLAUDE_CONFIG_DIR", dir.join(".claude-config"))
        .env("LUMEART_VECTORCRAFT", "no")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn eval_explains_what_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let help = lumeart(dir.path(), &["eval", "--help"], "");
    let help = text(&help.stdout);
    for sub in ["run", "compare", "report"] {
        assert!(help.contains(sub), "{help}");
    }

    // Sin evaluación hecha.
    for args in [["eval", "compare", "nada"], ["eval", "report", "nada"]] {
        let out = lumeart(dir.path(), &args, "");
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(
            text(&out.stderr),
            "Error: nada no es una evaluación: hazla antes con lumeart eval run -o nada\n"
        );
    }

    // Con A y sin el plugin instalado.
    let out = lumeart(dir.path(), &["eval", "run", "--limit", "1"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("no encuentro el plugin oficial ralph-loop"),
        "{}",
        text(&out.stderr)
    );

    // Variantes y encargos que no valen.
    let out = lumeart(dir.path(), &["eval", "run", "--variants", "D"], "");
    assert_eq!(
        text(&out.stderr),
        "Error: variante desconocida: D (son A, B y C)\n"
    );
    std::fs::write(dir.path().join("malos.json"), "[]").unwrap();
    let out = lumeart(
        dir.path(),
        &["eval", "run", "--tasks", "malos.json", "--variants", "B"],
        "",
    );
    assert_eq!(text(&out.stderr), "Error: el conjunto no tiene encargos\n");
}

/// Una evaluación de verdad: un encargo con las tres variantes (haiku, dos
/// vueltas), con el plugin instalado en Claude Code. Comprueba que el Stop
/// hook del plugin funciona con `claude -p` (la variante A da sus vueltas) y
/// que la comparación ciega revela las variantes solo al final.
#[test]
#[ignore = "usa Claude Code de verdad, con el plugin ralph-loop instalado"]
fn a_real_evaluation_with_the_official_plugin() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join(".config/lumeart");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(
        config.join("config.toml"),
        "[agent]\nprovider = \"claude-code\"\n[models]\nclaude_code = \"haiku\"\n",
    )
    .unwrap();
    // El plugin es el instalado en Claude Code (`~/.claude`); `claude` usa
    // su configuración de siempre.
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"));
    let out = command
        .args([
            "eval", "run", "-o", "ev", "--limit", "1", "--turns", "2", "--seed", "5",
        ])
        .current_dir(dir.path())
        .env("XDG_CONFIG_HOME", dir.path().join(".config"))
        .env("XDG_DATA_HOME", dir.path().join(".data"))
        .env("XDG_CACHE_HOME", dir.path().join(".cache"))
        .output()
        .unwrap();
    let stdout = text(&out.stdout);
    assert!(out.status.success(), "{stdout}{}", text(&out.stderr));
    assert!(stdout.contains("3 de 3 hechas"), "{stdout}");
    for word in ["plugin", "contexto vacío", "misma conversación"] {
        assert!(!stdout.contains(word), "{stdout}");
    }
    let results: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("ev/.datos/resultados.json")).unwrap(),
    )
    .unwrap();
    let a = results
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["variant"] == "A")
        .unwrap();
    assert!(a["error"].is_null(), "{a}");
    assert!(a["turns"].as_u64().unwrap() >= 1, "{a}");

    let out = lumeart(dir.path(), &["eval", "compare", "ev"], "1\n2\n=\n");
    let shown = text(&out.stdout);
    let (before, after) = shown.split_once("Ya están todos").unwrap();
    assert!(
        !before.contains("A (") && !before.contains("= A"),
        "{before}"
    );
    assert!(after.contains("A (plugin oficial)"), "{after}");
    assert!(dir.path().join("ev/informe.md").is_file());
}
