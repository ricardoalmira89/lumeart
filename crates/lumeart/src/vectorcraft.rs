//! Las ayudas de `dibujo.js` que viven fuera del motor: el calco de la
//! referencia (propio, con el motor de vtracer) y VectorCraft, un programa
//! externo y opcional (`vectorcraft-cli`) para lo que el SVG solo no puede
//! hacer (sólidos 3D, degradados libres, sus efectos) y para guardar en
//! `.vectorcraft` o `.eps`. Como `vectorcraft.go` y `trace.go` de pxcli.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use lume_app::ports::VecHelper;
use lume_domain::vec::FILE_NAME;
use lume_domain::vec::reference::{REFERENCE_PREFIX, sniff_image};
use lume_domain::vec::trace::{TraceOptions, post_trace, svg_fragment};
use serde_json::Value;

/// Lo que puede tardar VectorCraft en un trabajo.
const VC_TIMEOUT: Duration = Duration::from_secs(40);
/// Lo que puede tardar en convertir un dibujo para exportarlo.
const CONVERT_TIMEOUT: Duration = Duration::from_secs(120);
/// Cambia para invalidar los fragmentos guardados si cambia el retoque.
const CACHE_VERSION: &str = "1";

/// `vectorcraft-cli`: `LUMEART_VECTORCRAFT` (o `PXCLI_VECTORCRAFT`), el PATH
/// o las carpetas de siempre. `LUMEART_VECTORCRAFT=no` lo desactiva.
pub fn find() -> Result<PathBuf, String> {
    let missing = "hace falta VectorCraft: instala vectorcraft-cli (viene con VectorCraft) o indica su ruta en LUMEART_VECTORCRAFT";
    for var in ["LUMEART_VECTORCRAFT", "PXCLI_VECTORCRAFT"] {
        if let Some(value) = std::env::var_os(var).filter(|v| !v.is_empty()) {
            if value == "no" {
                return Err(missing.into());
            }
            return Ok(PathBuf::from(value));
        }
    }
    let mut candidates: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| {
            std::env::split_paths(&p)
                .map(|d| d.join("vectorcraft-cli"))
                .collect()
        })
        .unwrap_or_default();
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(Path::new(&home).join(".local/bin/vectorcraft-cli"));
    }
    candidates.push("/usr/bin/vectorcraft-cli".into());
    candidates.push("/usr/local/bin/vectorcraft-cli".into());
    candidates
        .into_iter()
        .find(|p| p.is_file())
        .ok_or_else(|| missing.into())
}

/// FNV-1a de 64 bits (para nombrar lo guardado en la caché).
fn fnv(parts: &[&[u8]]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for b in *part {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h ^= 0xff;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// La carpeta de caché de `kind` (si hay una).
fn cache_dir(kind: &str) -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "lumeart").map(|d| d.cache_dir().join(kind))
}

fn cached(kind: &str, key: &str) -> Option<String> {
    std::fs::read_to_string(cache_dir(kind)?.join(format!("{key}.svg"))).ok()
}

fn store(kind: &str, key: &str, fragment: &str) {
    if let Some(dir) = cache_dir(kind)
        && std::fs::create_dir_all(&dir).is_ok()
    {
        let _ = std::fs::write(dir.join(format!("{key}.svg")), fragment);
    }
}

/// Ejecuta `command` con un tiempo máximo y devuelve su salida (las dos).
fn run_with_timeout(mut command: Command, limit: Duration) -> Result<(bool, String), String> {
    let mut child = command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let start = Instant::now();
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(_) => break,
            None if start.elapsed() > limit => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("signal: killed".into());
            }
            None => std::thread::sleep(Duration::from_millis(30)),
        }
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok((out.status.success(), text))
}

/// El paso que VectorCraft no aceptó, de sus líneas JSON (`vcProblem`).
fn problem(output: &str, error: Option<&str>) -> String {
    for line in output.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(line)
            && let Some(err) = v["error"].as_str().filter(|e| !e.is_empty())
        {
            return match v["command"].as_str().filter(|c| !c.is_empty()) {
                Some(c) => format!("{c}: {err}"),
                None => err.to_string(),
            };
        }
    }
    let last = output.trim().lines().last().unwrap_or_default().trim();
    match error {
        Some(e) => format!("{e} {last}"),
        None => last.to_string(),
    }
}

/// Un trabajo de VectorCraft (`{ w, h, steps }`) hecho fragmento SVG, de la
/// caché si se puede (`runVectorCraft`).
fn run_job(cli: &Path, job: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(job).map_err(|e| format!("trabajo no válido: {e}"))?;
    let key = fnv(&[CACHE_VERSION.as_bytes(), job.as_bytes()]);
    if let Some(fragment) = cached("vectorcraft", &key) {
        return Ok(fragment);
    }
    let tmp = std::env::temp_dir().join(format!("lumeart-vc-{}-{key}", std::process::id()));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let out = tmp.join("out.svg");
    let mut steps = vec![serde_json::json!({
        "command": "file.new",
        "params": {"width": value["w"], "height": value["h"], "units": "Pixels", "rasterEffectsPpi": 150}
    })];
    steps.extend(value["steps"].as_array().cloned().unwrap_or_default());
    steps.push(serde_json::json!({
        "command": "file.export",
        "params": {"path": out.to_string_lossy(), "svg": {"decimals": 2}}
    }));
    let steps_file = tmp.join("steps.json");
    let result = std::fs::write(&steps_file, Value::Array(steps).to_string())
        .map_err(|e| e.to_string())
        .and_then(|()| {
            let mut command = Command::new(cli);
            command
                .arg("run")
                .arg("--commands")
                .arg(&steps_file)
                .current_dir(&tmp)
                .env("VECTORCRAFT_NO_PREFS", "1");
            run_with_timeout(command, VC_TIMEOUT)
        });
    let svg = std::fs::read_to_string(&out);
    let _ = std::fs::remove_dir_all(&tmp);
    let (ok, output) = result.map_err(|e| format!("VectorCraft falló: {e}"))?;
    let svg = match svg {
        Ok(svg) if ok => svg,
        Ok(_) => {
            return Err(format!(
                "VectorCraft falló: {}",
                problem(&output, Some("exit status 1"))
            ));
        }
        Err(e) => {
            return Err(format!(
                "VectorCraft falló: {}",
                problem(&output, Some(&e.to_string()))
            ));
        }
    };
    let fragment = svg_fragment(&svg, &format!("vc{}-", &key[..8]));
    store("vectorcraft", &key, &fragment);
    Ok(fragment)
}

/// VectorCraft como ayuda de `dibujo.js`, si está instalado.
pub fn helper() -> Option<VecHelper> {
    let cli = find().ok()?;
    Some(Arc::new(move |job: &str| run_job(&cli, job)))
}

/// Guarda el SVG limpio con VectorCraft en el formato de `out`
/// (`.vectorcraft`, `.eps`…). Devuelve sus avisos (lo que no pasó).
pub fn convert(clean: &str, out: &Path) -> Result<Vec<String>, String> {
    let cli = find()?;
    let tmp = std::env::temp_dir().join(format!("lumeart-export-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let input = tmp.join("dibujo.svg");
    let result = std::fs::write(&input, clean)
        .map_err(|e| e.to_string())
        .and_then(|()| {
            let mut command = Command::new(&cli);
            command
                .arg("convert")
                .arg(&input)
                .arg(out)
                .env("VECTORCRAFT_NO_PREFS", "1");
            run_with_timeout(command, CONVERT_TIMEOUT)
        });
    let _ = std::fs::remove_dir_all(&tmp);
    let (ok, output) =
        result.map_err(|e| format!("VectorCraft no pudo convertir el dibujo: {e}"))?;
    if !ok {
        return Err(format!(
            "VectorCraft no pudo convertir el dibujo: {}",
            problem(&output, Some("exit status 1"))
        ));
    }
    let mut warnings = Vec::new();
    for line in output.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            for key in ["importWarnings", "warnings"] {
                for w in v[key].as_array().into_iter().flatten() {
                    if let Some(w) = w.as_str() {
                        warnings.push(w.to_string());
                    }
                }
            }
        }
    }
    Ok(warnings)
}

/// La `referencia-*` más nueva de `dir` que sea una imagen.
pub fn find_reference(dir: &Path) -> Option<PathBuf> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(REFERENCE_PREFIX))
        .collect();
    names.sort();
    names
        .into_iter()
        .rev()
        .map(|n| dir.join(n))
        .find(|p| image_kind(p).is_some())
}

/// El tipo de imagen de un archivo (PNG, JPEG, GIF o WebP), si lo es.
pub fn image_kind(path: &Path) -> Option<&'static str> {
    let bytes = std::fs::read(path).ok()?;
    sniff_image(&bytes[..bytes.len().min(512)])
}

/// Las opciones de un trabajo de `trace()` (el JSON que manda `lib.js`).
pub fn trace_options(job: &str) -> Result<TraceOptions, String> {
    let v: Value =
        serde_json::from_str(job).map_err(|e| format!("trace: trabajo no válido: {e}"))?;
    let list = |key: &str| -> Vec<String> {
        v[key]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let defaults = TraceOptions::default();
    Ok(TraceOptions {
        width: v["w"].as_u64().map_or(defaults.width, |w| w as u32),
        height: v["h"].as_u64().map_or(defaults.height, |h| h as u32),
        reference: v["ref"].as_str().unwrap_or_default().to_string(),
        lines: v["mode"].as_str() == Some("lines"),
        colors: v["colors"].as_u64().map_or(defaults.colors, |c| c as u32),
        paths: v["paths"].as_f64().unwrap_or(defaults.paths),
        corners: v["corners"].as_f64().unwrap_or(defaults.corners),
        noise: v["noise"].as_f64().unwrap_or(defaults.noise),
        threshold: v["threshold"]
            .as_u64()
            .map_or(defaults.threshold, |t| t as u32),
        drop: list("drop"),
        tolerance: v["tolerance"].as_f64().unwrap_or(defaults.tolerance),
        palette: list("palette"),
        ink: v["ink"].as_str().unwrap_or_default().to_string(),
    })
}

/// La referencia de un trabajo de `trace()`, resuelta junto a `dibujo.js`.
fn resolve_reference(name: &str, dir: &Path) -> Result<(PathBuf, &'static str), String> {
    if name.is_empty() {
        let path = find_reference(dir).ok_or_else(|| {
            format!("trace: no hay imagen de referencia (referencia-*.png) junto a {FILE_NAME}")
        })?;
        let kind = image_kind(&path).unwrap_or("image/png");
        return Ok((path, kind));
    }
    let path = if Path::new(name).is_absolute() {
        PathBuf::from(name)
    } else {
        dir.join(name)
    };
    let kind = image_kind(&path).ok_or_else(|| {
        let base = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        format!("trace: {base} no es una imagen PNG, JPG, GIF o WebP")
    })?;
    Ok((path, kind))
}

/// Los trazos sin retocar del calco de `path` con `job` (de la caché si se
/// puede).
pub fn raw_trace(path: &Path, kind: &str, job: &TraceOptions) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let settings = lume_domain::vec::trace::describe(job);
    let key = fnv(&[CACHE_VERSION.as_bytes(), settings.as_bytes(), &bytes]);
    if let Some(fragment) = cached("calco", &key) {
        return Ok(fragment);
    }
    let fragment = lume_raster::trace::trace_reference(&bytes, kind, job).map_err(|e| match e {
        lume_app::PortError::Invalid(m) => format!("trace: {m}"),
        other => format!("trace: {other}"),
    })?;
    store("calco", &key, &fragment);
    Ok(fragment)
}

/// El calco como ayuda de `dibujo.js` (`trace()`), con la referencia
/// resuelta junto al dibujo en `dir`.
pub fn tracer(dir: PathBuf) -> VecHelper {
    Arc::new(move |job: &str| {
        let options = trace_options(job)?;
        let (path, kind) = resolve_reference(&options.reference, &dir)?;
        let raw = raw_trace(&path, kind, &options)?;
        Ok(post_trace(&raw, &options))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_trace_job_is_read_like_lib_js_writes_it() {
        let job = trace_options(
            r##"{"w":300,"h":200,"ref":"","mode":"lines","colors":8,"paths":40,"corners":50,"noise":3,"threshold":90,"drop":["#ffffff"],"tolerance":30,"palette":[],"ink":"#101010"}"##,
        )
        .unwrap();
        assert_eq!(
            (job.width, job.height, job.colors, job.threshold),
            (300, 200, 8, 90)
        );
        assert!(job.lines);
        assert_eq!(job.drop, ["#ffffff"]);
        assert_eq!(job.ink, "#101010");
        assert!(
            trace_options("{")
                .unwrap_err()
                .starts_with("trace: trabajo no válido")
        );
        assert_eq!(
            problem(
                "{\"command\":\"imageTrace.make\",\"error\":\"sin imagen\"}\n",
                None
            ),
            "imageTrace.make: sin imagen"
        );
    }

    #[test]
    fn missing_references_say_where_they_looked() {
        let tmp = std::env::temp_dir().join(format!("lumeart-ref-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        assert_eq!(
            resolve_reference("", &tmp).unwrap_err(),
            "trace: no hay imagen de referencia (referencia-*.png) junto a dibujo.js"
        );
        std::fs::write(tmp.join("foto.txt"), "hola").unwrap();
        assert_eq!(
            resolve_reference("foto.txt", &tmp).unwrap_err(),
            "trace: foto.txt no es una imagen PNG, JPG, GIF o WebP"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
