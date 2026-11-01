//! `lumeart vec godot` frente a `pxcli vec godot` con los dibujos de
//! `reference/runs/` (`fixtures/vec_godot/`, generado por
//! `scripts/vec-golden/godot/`): un personaje con esqueleto da el mismo
//! `.tscn` y un atlas del mismo tamaño y casi igual; una escena, la misma
//! estructura (nodos, tipos, padres, recursos, propiedades y texturas). Los
//! números de una escena salen del recorte de cada pieza a sus píxeles
//! visibles, y la cola tenue de un desenfoque no acaba en el mismo píxel en
//! resvg que en Chrome: pueden moverse unos píxeles.

use std::path::{Path, PathBuf};

use serde_json::Value;

/// Diferencia media admitida en la rejilla de 8×8 colores medios de una
/// textura (de 0 a 255).
const MAX_GRID_DIFF: f64 = 3.0;
/// Píxeles que puede moverse el borde de un recorte de una escena.
const BOX_SLACK: f64 = 16.0;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vec_godot")
}

fn drawing(run: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../lume-script/tests/fixtures/vec")
        .join(run)
        .join("dibujo.js")
}

/// La rejilla 8×8 de colores medios premultiplicados (como el generador).
fn grid(path: &Path) -> (u32, u32, Vec<i64>) {
    let file = std::fs::File::open(path).unwrap();
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();
    assert_eq!(info.color_type, png::ColorType::Rgba, "{}", path.display());
    let (w, h) = (info.width as usize, info.height as usize);
    let mut out = Vec::new();
    for j in 0..8 {
        for i in 0..8 {
            let (x0, x1) = (i * w / 8, (i + 1) * w / 8);
            let (y0, y1) = (j * h / 8, (j + 1) * h / 8);
            let mut sum = [0i64; 4];
            let mut n = 0i64;
            for y in y0..y1 {
                for x in x0..x1 {
                    let o = (y * w + x) * 4;
                    let a = i64::from(buf[o + 3]);
                    for c in 0..3 {
                        sum[c] += i64::from(buf[o + c]) * a / 255;
                    }
                    sum[3] += a;
                    n += 1;
                }
            }
            for s in sum {
                out.push(if n > 0 { s / n } else { 0 });
            }
        }
    }
    (info.width, info.height, out)
}

/// Los números de una línea, y la línea con ellos cambiados por `#`.
fn numbers(line: &str) -> (String, Vec<f64>) {
    let mut shape = String::new();
    let mut nums = Vec::new();
    let mut current = String::new();
    let flush = |current: &mut String, shape: &mut String, nums: &mut Vec<f64>| {
        if !current.is_empty() {
            match current.parse::<f64>() {
                Ok(v) => {
                    nums.push(v);
                    shape.push('#');
                }
                Err(_) => shape.push_str(current),
            }
            current.clear();
        }
    };
    let mut prev = ' ';
    for c in line.chars() {
        let starts = c.is_ascii_digit() || (c == '-' && !prev.is_alphanumeric());
        let continues = !current.is_empty() && (c.is_ascii_digit() || c == '.');
        if continues || (current.is_empty() && starts && !prev.is_alphanumeric() && prev != '_') {
            current.push(c);
        } else {
            flush(&mut current, &mut shape, &mut nums);
            shape.push(c);
        }
        prev = c;
    }
    flush(&mut current, &mut shape, &mut nums);
    (shape, nums)
}

fn compare_textures(run: &str, project: &Path, want: &Value, slack: u32) -> Vec<String> {
    let mut wrong = Vec::new();
    let mut ours: Vec<String> = Vec::new();
    let arte = project.join("arte");
    let mut walk = vec![arte.clone()];
    while let Some(dir) = walk.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk.push(path);
            } else if path.extension().is_some_and(|e| e == "png") {
                let rel = path.strip_prefix(project).unwrap();
                ours.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    ours.sort();
    let theirs: Vec<String> = want["textures"]
        .as_array()
        .map(|t| {
            t.iter()
                .map(|x| x["file"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default();
    if ours != theirs {
        wrong.push(format!("{run}: texturas {ours:?}, pxcli {theirs:?}"));
        return wrong;
    }
    for t in want["textures"].as_array().into_iter().flatten() {
        let file = t["file"].as_str().unwrap();
        let (w, h, g) = grid(&project.join(file));
        let (tw, th) = (
            t["width"].as_u64().unwrap() as u32,
            t["height"].as_u64().unwrap() as u32,
        );
        if w.abs_diff(tw) > slack || h.abs_diff(th) > slack {
            wrong.push(format!("{run}: {file} mide {w}x{h}, pxcli {tw}x{th}"));
            continue;
        }
        // Con otro recorte, la rejilla cae en otro sitio: solo se compara
        // el contenido de las que miden lo mismo.
        if (w, h) != (tw, th) {
            continue;
        }
        let theirs: Vec<i64> = t["grid"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect();
        let diff = g
            .iter()
            .zip(&theirs)
            .map(|(a, b)| (a - b).abs())
            .sum::<i64>() as f64
            / g.len() as f64;
        if diff > MAX_GRID_DIFF {
            wrong.push(format!("{run}: {file} difiere {diff:.2} de media"));
        }
    }
    wrong
}

#[test]
fn godot_exports_match_pxcli() {
    let mut runs: Vec<PathBuf> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    runs.sort();
    assert_eq!(runs.len(), 32, "25 personajes y 7 escenas");
    let mut wrong = Vec::new();
    let (mut characters, mut scenes) = (0, 0);
    for dir in &runs {
        let run = dir.file_name().unwrap().to_string_lossy().into_owned();
        let want: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("pxcli.json")).unwrap())
                .unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        std::fs::write(project.join("project.godot"), "config_version=5\n").unwrap();
        std::fs::copy(drawing(&run), project.join("dibujo.js")).unwrap();
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
            .args(["vec", "godot", "-o", "arte"])
            .current_dir(project)
            .env("LUMEART_VECTORCRAFT", "no")
            .env("XDG_CACHE_HOME", project.join(".cache"))
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        if let Some(error) = want["error"].as_str() {
            if stderr != format!("Error: {error}\n") {
                wrong.push(format!("{run}: error {stderr:?}, pxcli {error:?}"));
            }
            continue;
        }
        if !out.status.success() {
            wrong.push(format!("{run}: falló: {stderr}"));
            continue;
        }
        let summary = want["summary"].as_str().unwrap();
        if !stdout.ends_with(&format!("{summary}\n")) {
            wrong.push(format!(
                "{run}: salida {stdout:?}, pxcli termina en {summary:?}"
            ));
        }
        let kind = want["kind"].as_str().unwrap();
        let tscn_name = if kind == "personaje" {
            "personaje"
        } else {
            "escena"
        };
        let ours = std::fs::read_to_string(project.join("arte").join(format!("{tscn_name}.tscn")))
            .unwrap_or_default();
        let theirs = want["tscn"].as_str().unwrap();
        if kind == "personaje" {
            characters += 1;
            if ours != theirs {
                let first = ours
                    .lines()
                    .zip(theirs.lines())
                    .position(|(a, b)| a != b)
                    .unwrap_or(0);
                wrong.push(format!(
                    "{run}: .tscn distinto en la línea {}: {:?} / pxcli {:?}",
                    first + 1,
                    ours.lines().nth(first),
                    theirs.lines().nth(first)
                ));
            }
            wrong.extend(compare_textures(&run, project, &want, 0));
        } else {
            scenes += 1;
            // Mismas líneas; los números, con el margen del recorte.
            let (a, b): (Vec<&str>, Vec<&str>) = (ours.lines().collect(), theirs.lines().collect());
            if a.len() != b.len() {
                wrong.push(format!(
                    "{run}: .tscn de {} líneas, pxcli {}",
                    a.len(),
                    b.len()
                ));
            } else {
                for (i, (x, y)) in a.iter().zip(&b).enumerate() {
                    let ((sx, nx), (sy, ny)) = (numbers(x), numbers(y));
                    let close = nx.len() == ny.len()
                        && nx.iter().zip(&ny).all(|(p, q)| (p - q).abs() <= BOX_SLACK);
                    if sx != sy || !close {
                        wrong.push(format!("{run}: línea {}: {x:?} / pxcli {y:?}", i + 1));
                        break;
                    }
                }
            }
            wrong.extend(compare_textures(&run, project, &want, BOX_SLACK as u32 * 2));
        }
    }
    assert_eq!((characters, scenes), (22, 7));
    assert!(
        wrong.is_empty(),
        "{} diferencias:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
