//! `lumeart anim` contra `pxcli anim`: los mismos pasos (`steps.json`) sobre
//! los mismos archivos dan la misma salida, el mismo error y los mismos
//! archivos (PNG: tamaño y hash de sus píxeles; el resto, su texto). Las
//! referencias (`pxcli_cli.json`) las generó la CLI de pxcli.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/anim_cli")
}

/// Como `signature` del generador: PNG → «png WxH hash»; si no, el texto.
fn signature(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    if path.extension().is_some_and(|e| e == "png") {
        let Ok(img) = lume_store::decode_image(&bytes) else {
            return "png roto".into();
        };
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for p in img.pixels() {
            for b in [p.r, p.g, p.b, p.a] {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        return format!("png {}x{} {h:016x}", img.width(), img.height());
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn snapshot(dir: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                // Con «/» en todos los sistemas.
                let rel = path
                    .strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, signature(&path));
            }
        }
    }
    out
}

fn copy_case(src: &Path, dst: &Path) {
    for entry in std::fs::read_dir(src).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name == "steps.json" || name == "pxcli_cli.json" {
            continue;
        }
        std::fs::copy(&path, dst.join(&name)).unwrap();
    }
}

#[test]
fn anim_commands_behave_like_pxcli() {
    let mut cases: Vec<PathBuf> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    cases.sort();
    assert_eq!(cases.len(), 6);
    let mut checked = 0;
    for case in cases {
        let name = case.file_name().unwrap().to_string_lossy().into_owned();
        let golden: Value =
            serde_json::from_str(&std::fs::read_to_string(case.join("pxcli_cli.json")).unwrap())
                .unwrap();
        let tmp = tempfile::tempdir().unwrap();
        copy_case(&case, tmp.path());
        let mut before = snapshot(tmp.path());
        for step in golden["steps"].as_array().unwrap() {
            let args: Vec<String> = step["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a.as_str().unwrap().to_string())
                .collect();
            let out = std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
                .arg("anim")
                .args(&args)
                .current_dir(tmp.path())
                .output()
                .unwrap();
            let what = format!("{name}: anim {}", args.join(" "));
            // La salida de pxcli se capturó en Linux; en Windows las rutas
            // llevan `\`.
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let (stdout, stderr) = if cfg!(windows) {
                (stdout.replace('\\', "/"), stderr.replace('\\', "/"))
            } else {
                (stdout.into_owned(), stderr.into_owned())
            };
            assert_eq!(stdout, step["stdout"].as_str().unwrap(), "{what}: salida");
            let error = step["error"].as_str().unwrap();
            if error.is_empty() {
                assert!(out.status.success(), "{what}: {stderr}");
                assert_eq!(stderr, "", "{what}");
            } else {
                assert_eq!(out.status.code(), Some(1), "{what}");
                assert_eq!(stderr, format!("Error: {error}\n"), "{what}: error");
            }
            let after = snapshot(tmp.path());
            let changed: BTreeMap<String, String> = after
                .iter()
                .filter(|(k, v)| before.get(*k) != Some(*v))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let want: BTreeMap<String, String> =
                serde_json::from_value(step["files"].clone()).unwrap();
            assert_eq!(changed, want, "{what}: archivos");
            before = after;
            checked += 1;
        }
    }
    assert_eq!(checked, 49, "todos los pasos");
}

fn lumeart(dir: &Path, args: &[&str]) -> std::process::Output {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

/// La hoja: una fila por acción y una columna por fotograma, sin huecos;
/// cada celda es el fotograma (como `anim frame`) y lo que sobra de las
/// filas cortas queda transparente. El `.json` dice lo mismo.
#[test]
fn the_sheet_has_one_row_per_action_and_one_column_per_frame() {
    let tmp = tempfile::tempdir().unwrap();
    copy_case(&fixtures().join("reloj"), tmp.path());
    let out = lumeart(tmp.path(), &["anim", "export", "--scale", "2", "--json"]);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "reloj-de-arena_24x24.png\nreloj-de-arena_24x24.json\n"
    );
    let sheet = lume_store::read_image(&tmp.path().join("reloj-de-arena_24x24.png")).unwrap();
    let meta: Value = serde_json::from_str(
        &std::fs::read_to_string(tmp.path().join("reloj-de-arena_24x24.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(meta["image"], "reloj-de-arena_24x24.png");
    assert_eq!(
        (meta["frame_width"].as_u64(), meta["frame_height"].as_u64()),
        (Some(24), Some(24))
    );
    let actions = meta["actions"].as_array().unwrap();
    let frames: Vec<u64> = actions
        .iter()
        .map(|a| a["frames"].as_u64().unwrap())
        .collect();
    assert_eq!(frames, [4, 3, 2]);
    // 4 columnas (la acción más larga) × 3 filas, sin huecos.
    assert_eq!(sheet.size(), (4 * 24, 3 * 24));
    for (row, action) in actions.iter().enumerate() {
        assert_eq!(action["row"], row);
        let name = action["name"].as_str().unwrap();
        for f in 0..4u32 {
            let cell =
                |x: u32, y: u32| sheet.get(i64::from(f * 24 + x), i64::from(row as u32 * 24 + y));
            if u64::from(f) < frames[row] {
                lumeart(
                    tmp.path(),
                    &[
                        "anim",
                        "frame",
                        "--action",
                        name,
                        "--frame",
                        &f.to_string(),
                        "--scale",
                        "2",
                        "-o",
                        "f.png",
                    ],
                );
                let frame = lume_store::read_image(&tmp.path().join("f.png")).unwrap();
                assert_eq!(frame.size(), (24, 24));
                for y in 0..24 {
                    for x in 0..24 {
                        assert_eq!(
                            cell(x, y),
                            frame.get(i64::from(x), i64::from(y)),
                            "{name} f{f} ({x},{y})"
                        );
                    }
                }
            } else {
                for y in 0..24 {
                    for x in 0..24 {
                        assert_eq!(
                            cell(x, y).a,
                            0,
                            "{name}: la celda {f} sobra y es transparente"
                        );
                    }
                }
            }
        }
    }
}
