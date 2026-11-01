//! `lumeart.toml`: leer, validar y escribir la guía de arte del juego.
//!
//! Al leer se comprueba todo y se informa de cada problema con su línea; una
//! clave que falta toma el valor por defecto, pero una clave desconocida es
//! un error (casi siempre es una errata).

use std::fmt;
use std::path::{Path, PathBuf};

use lume_app::ports::PortError;
use lume_app::{AssetFile, GuideFile, ProjectStore};
use lume_domain::Rgba;
use lume_domain::guide::{ArtGuide, GUIDE_FILE, GuideError, GuidePalette, Naming};
use serde::Deserialize;
use toml::{Spanned, Value};

/// Claves del archivo, en orden.
const KEYS: [&str; 5] = ["style", "size", "palette", "output", "naming"];

/// Un problema del archivo, con su línea si se sabe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub line: Option<usize>,
    pub message: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "línea {line}: {}", self.message),
            None => f.write_str(&self.message),
        }
    }
}

/// Lo que puede fallar con `lumeart.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuideFileError {
    /// No se pudo leer o escribir.
    Io { path: PathBuf, message: String },
    /// `init` sin `--force` y ya hay una guía.
    Exists(PathBuf),
    /// El archivo no vale (uno o más problemas).
    Invalid {
        path: PathBuf,
        problems: Vec<Problem>,
    },
}

impl fmt::Display for GuideFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GuideFileError::Io { path, message } => write!(f, "{}: {message}", path.display()),
            GuideFileError::Exists(path) => write!(
                f,
                "ya existe {}; usa --force para sobrescribirlo",
                path.display()
            ),
            GuideFileError::Invalid { path, problems } => {
                write!(f, "{} no es una guía de arte válida:", path.display())?;
                for problem in problems {
                    write!(f, "\n  {problem}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for GuideFileError {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGuide {
    style: Option<Spanned<Value>>,
    size: Option<Spanned<Value>>,
    palette: Option<Spanned<Value>>,
    output: Option<Spanned<Value>>,
    naming: Option<Spanned<Value>>,
}

/// Línea (desde 1) del byte `offset` de `text`.
fn line_of(text: &str, offset: usize) -> usize {
    let end = offset.min(text.len());
    text.as_bytes()[..end]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}

/// Lee una guía de un texto TOML. Devuelve todos los problemas que encuentre.
pub fn guide_from_toml(text: &str) -> Result<ArtGuide, Vec<Problem>> {
    let raw: RawGuide = toml::from_str(text).map_err(|e| {
        let line = e.span().map(|s| line_of(text, s.start));
        vec![Problem {
            line,
            message: toml_message(e.message()),
        }]
    })?;
    let mut problems = Vec::new();
    let p = &mut problems;
    let style = field(text, &raw.style, p, |v| {
        Ok(text_value("style", v)?.to_string())
    });
    let size = field(text, &raw.size, p, |v| {
        ArtGuide::parse_size(text_value("size", v)?)
    });
    let palette = field(text, &raw.palette, p, palette_value);
    let output = field(text, &raw.output, p, |v| {
        ArtGuide::parse_output(text_value("output", v)?)
    });
    let naming = field(text, &raw.naming, p, |v| {
        Naming::parse(text_value("naming", v)?)
    });
    if !problems.is_empty() {
        return Err(problems);
    }
    let d = ArtGuide::default();
    Ok(ArtGuide {
        style: style.unwrap_or(d.style),
        size: size.unwrap_or(d.size),
        palette: palette.unwrap_or(d.palette),
        output: output.unwrap_or(d.output),
        naming: naming.unwrap_or(d.naming),
    })
}

/// Interpreta un campo si está; si no vale, apunta el problema con su línea.
fn field<T>(
    text: &str,
    value: &Option<Spanned<Value>>,
    problems: &mut Vec<Problem>,
    parse: impl Fn(&Value) -> Result<T, GuideError>,
) -> Option<T> {
    let value = value.as_ref()?;
    match parse(value.get_ref()) {
        Ok(parsed) => Some(parsed),
        Err(e) => {
            problems.push(Problem {
                line: Some(line_of(text, value.span().start)),
                message: e.to_string(),
            });
            None
        }
    }
}

/// Los errores de `toml`/`serde` vienen en inglés; los habituales se dicen
/// en español.
fn toml_message(message: &str) -> String {
    if let Some(rest) = message.strip_prefix("unknown field `")
        && let Some((key, _)) = rest.split_once('`')
    {
        return format!(
            "clave desconocida `{key}`; las claves válidas son {}",
            KEYS.map(|k| format!("`{k}`")).join(", ")
        );
    }
    if message.trim() == "duplicate key" {
        return "clave repetida".into();
    }
    format!("no es TOML válido: {}", message.trim())
}

fn text_value<'a>(key: &'static str, value: &'a Value) -> Result<&'a str, GuideError> {
    value.as_str().ok_or_else(|| GuideError {
        key,
        message: format!(
            "tiene que ser un texto entre comillas, no {}",
            type_name(value)
        ),
    })
}

fn palette_value(value: &Value) -> Result<GuidePalette, GuideError> {
    match value {
        Value::String(name) => GuidePalette::parse_name(name),
        Value::Array(items) => {
            let mut colors = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                let Some(text) = item.as_str() else {
                    return Err(GuideError {
                        key: "palette",
                        message: format!(
                            "el color {} tiene que ser un texto entre comillas, como \"#1a1c2c\", no {}",
                            i + 1,
                            type_name(item)
                        ),
                    });
                };
                colors.push(text);
            }
            GuidePalette::parse_colors(&colors)
        }
        other => Err(GuideError {
            key: "palette",
            message: format!(
                "tiene que ser un nombre entre comillas o una lista de colores, no {}",
                type_name(other)
            ),
        }),
    }
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "un texto",
        Value::Integer(_) => "un número",
        Value::Float(_) => "un número decimal",
        Value::Boolean(_) => "un sí/no",
        Value::Datetime(_) => "una fecha",
        Value::Array(_) => "una lista",
        Value::Table(_) => "una tabla",
    }
}

/// `#rrggbb` si es opaco; si no, `#rrggbbaa`.
fn hex(color: Rgba) -> String {
    let full = color.to_string();
    if color.a == 255 {
        full[..7].to_string()
    } else {
        full
    }
}

/// Escribe la guía como TOML comentado.
pub fn guide_to_toml(guide: &ArtGuide) -> String {
    let q = |text: &str| Value::String(text.to_string()).to_string();
    let palette = match &guide.palette {
        GuidePalette::Free => q(lume_domain::guide::FREE_PALETTE),
        GuidePalette::Fixed(name) => q(name),
        GuidePalette::Custom(colors) => {
            let items: Vec<String> = colors.iter().map(|c| q(&hex(*c))).collect();
            format!("[{}]", items.join(", "))
        }
    };
    let fixed: Vec<String> = lume_domain::palette::FIXED_PALETTES
        .iter()
        .map(|n| q(n))
        .collect();
    let namings: Vec<String> = Naming::ALL.iter().map(|n| q(n.as_str())).collect();
    format!(
        "# Guía de arte de LumeArt para este juego. La leen el studio, `lumeart mcp`
# y los agentes antes de encargar un recurso, para que todo el arte encaje.

# Estilo en pocas palabras; el agente lo recibe con cada encargo. Por
# ejemplo: \"pixel art de 16 bits, contorno oscuro de 1 px, luz desde arriba
# a la izquierda\".
style = {style}

# Tamaño por defecto de cada recurso: ANCHOxALTO, de 1 a 512.
size = {size}

# Paleta: \"libre\" (sin límite), una fija ({fixed}) o una lista de
# colores, como [\"#1a1c2c\", \"#5d275d\", \"#f4f4f4\"].
palette = {palette}

# Carpeta de los archivos (.png y .lume), relativa a este archivo.
output = {output}

# Nombres de archivo: {namings}.
naming = {naming}
",
        style = q(&guide.style),
        size = q(&format!("{}x{}", guide.size.0, guide.size.1)),
        fixed = fixed.join(", "),
        output = q(&guide.output),
        namings = namings.join(", "),
        naming = q(guide.naming.as_str()),
    )
}

/// Busca `lumeart.toml` en `start` y en sus carpetas superiores.
pub fn find_guide(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|dir| dir.join(GUIDE_FILE))
        .find(|path| path.is_file())
}

/// Lee y valida una guía.
pub fn load_guide(path: &Path) -> Result<ArtGuide, GuideFileError> {
    let text = std::fs::read_to_string(path).map_err(|e| GuideFileError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    guide_from_toml(&text).map_err(|problems| GuideFileError::Invalid {
        path: path.to_path_buf(),
        problems,
    })
}

/// Crea `lumeart.toml` en `dir`. Sin `force`, no toca uno que ya exista.
pub fn init_guide(dir: &Path, guide: &ArtGuide, force: bool) -> Result<PathBuf, GuideFileError> {
    use std::io::Write;
    let path = dir.join(GUIDE_FILE);
    let io = |e: std::io::Error| GuideFileError::Io {
        path: path.clone(),
        message: e.to_string(),
    };
    let text = guide_to_toml(guide);
    if force {
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, &text).map_err(io)?;
        std::fs::rename(&tmp, &path).map_err(io)?;
    } else {
        // `create_new` no pisa nada aunque aparezca entre medias.
        let mut file = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(GuideFileError::Exists(path));
            }
            Err(e) => return Err(io(e)),
        };
        file.write_all(text.as_bytes()).map_err(io)?;
    }
    Ok(path)
}

/// `ProjectStore` sobre el disco: la guía y los recursos del juego.
#[derive(Debug, Clone, Copy, Default)]
pub struct FsProject;

impl ProjectStore for FsProject {
    fn find_guide(&self, dir: &Path) -> Result<Option<GuideFile>, PortError> {
        let Some(path) = find_guide(dir) else {
            return Ok(None);
        };
        let text = std::fs::read_to_string(&path)
            .map_err(|e| PortError::Io(format!("{}: {e}", path.display())))?;
        let guide = guide_from_toml(&text).map_err(|problems| {
            PortError::Invalid(
                GuideFileError::Invalid {
                    path: path.clone(),
                    problems,
                }
                .to_string(),
            )
        })?;
        Ok(Some(GuideFile { path, text, guide }))
    }

    fn list_assets(&self, dir: &Path) -> Result<Vec<AssetFile>, PortError> {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(PortError::Io(format!("{}: {e}", dir.display()))),
        };
        let mut assets: std::collections::BTreeMap<String, AssetFile> = Default::default();
        for entry in entries.flatten() {
            let path = entry.path();
            let (Some(stem), Some(ext)) = (
                path.file_stem().and_then(|s| s.to_str()),
                path.extension().and_then(|s| s.to_str()),
            ) else {
                continue;
            };
            let ext = ext.to_ascii_lowercase();
            if ext != "png" && ext != "lume" {
                continue;
            }
            let asset = assets.entry(stem.to_string()).or_insert_with(|| AssetFile {
                name: stem.to_string(),
                png: None,
                lume: None,
                versions: None,
            });
            if ext == "png" {
                asset.png = Some(path);
            } else {
                asset.versions = std::fs::read(&path)
                    .ok()
                    .and_then(|bytes| crate::decode_lume(&bytes).ok())
                    .map(|doc| doc.versions().len());
                asset.lume = Some(path);
            }
        }
        Ok(assets.into_values().collect())
    }

    fn prepare_output(&self, dir: &Path) -> Result<(), PortError> {
        std::fs::create_dir_all(dir).map_err(|e| PortError::Io(format!("{}: {e}", dir.display())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_numbers() {
        assert_eq!(line_of("a\nb\nc", 0), 1);
        assert_eq!(line_of("a\nb\nc", 2), 2);
        assert_eq!(line_of("a\nb\nc", 99), 3);
    }

    #[test]
    fn hex_drops_opaque_alpha() {
        assert_eq!(hex(Rgba::rgb(0x1a, 0x1c, 0x2c)), "#1a1c2c");
        assert_eq!(hex(Rgba::new(1, 2, 3, 4)), "#01020304");
    }
}
