//! `ArtGuide`: la guía de arte de un juego (`lumeart.toml`).
//!
//! Dice cómo tiene que ser todo el arte del proyecto para que encaje: estilo,
//! tamaño por defecto, paleta, carpeta de salida y convención de nombres. Aquí
//! están el tipo, sus valores por defecto y la validación de cada campo; leer
//! y escribir el archivo es cosa de `lume-store`.

use thiserror::Error;

use crate::canvas::MAX_CANVAS_SIZE;
use crate::color::Rgba;
use crate::palette::{FIXED_PALETTES, Palette};

/// Nombre del archivo de la guía, en la carpeta del juego.
pub const GUIDE_FILE: &str = "lumeart.toml";

/// Un campo de la guía que no vale. `key` es la clave del archivo.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{key}: {message}")]
pub struct GuideError {
    pub key: &'static str,
    pub message: String,
}

impl GuideError {
    fn new(key: &'static str, message: impl Into<String>) -> Self {
        Self {
            key,
            message: message.into(),
        }
    }
}

/// La paleta del proyecto.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum GuidePalette {
    /// Sin límite de colores.
    #[default]
    Free,
    /// Una paleta fija por nombre (`pico-8`, `game-boy`).
    Fixed(String),
    /// Una lista de colores propia.
    Custom(Vec<Rgba>),
}

/// Valor de `palette` que significa «sin límite».
pub const FREE_PALETTE: &str = "libre";

impl GuidePalette {
    /// Lee un nombre: `libre` o una paleta fija.
    pub fn parse_name(name: &str) -> Result<GuidePalette, GuideError> {
        let name = name.trim();
        if name == FREE_PALETTE {
            return Ok(GuidePalette::Free);
        }
        if Palette::fixed(name).is_some() {
            return Ok(GuidePalette::Fixed(name.to_string()));
        }
        Err(GuideError::new(
            "palette",
            format!(
                "no hay ninguna paleta llamada {name:?}; usa \"{FREE_PALETTE}\", {} o una lista de colores como [\"#1a1c2c\", \"#f4f4f4\"]",
                FIXED_PALETTES
                    .iter()
                    .map(|n| format!("\"{n}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ))
    }

    /// Lee una lista de colores (`#rrggbb`, `#rrggbbaa` o un nombre de
    /// color). Sin repetidos y con al menos uno.
    pub fn parse_colors<S: AsRef<str>>(colors: &[S]) -> Result<GuidePalette, GuideError> {
        if colors.is_empty() {
            return Err(GuideError::new(
                "palette",
                format!(
                    "la lista de colores está vacía; para no limitar los colores usa \"{FREE_PALETTE}\""
                ),
            ));
        }
        let mut out: Vec<Rgba> = Vec::with_capacity(colors.len());
        for (i, raw) in colors.iter().enumerate() {
            let raw = raw.as_ref().trim();
            let color = Rgba::parse(raw).map_err(|_| {
                GuideError::new(
                    "palette",
                    format!(
                        "el color {} ({raw:?}) no es válido; usa #rrggbb, #rrggbbaa o un nombre como \"red\"",
                        i + 1
                    ),
                )
            })?;
            if let Some(first) = out.iter().position(|c| *c == color) {
                return Err(GuideError::new(
                    "palette",
                    format!("el color {} ({raw:?}) repite el color {}", i + 1, first + 1),
                ));
            }
            out.push(color);
        }
        Ok(GuidePalette::Custom(out))
    }

    /// La paleta que limita los colores, si la hay.
    pub fn palette(&self) -> Option<Palette> {
        match self {
            GuidePalette::Free => None,
            GuidePalette::Fixed(name) => Palette::fixed(name),
            GuidePalette::Custom(colors) => {
                let mut palette = Palette::new();
                for color in colors {
                    palette.push(*color, None);
                }
                Some(palette)
            }
        }
    }
}

/// Convención de nombres de los archivos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Naming {
    /// `cofre_de_madera`
    #[default]
    Snake,
    /// `cofre-de-madera`
    Kebab,
    /// `cofreDeMadera`
    Camel,
    /// `CofreDeMadera`
    Pascal,
}

impl Naming {
    pub const ALL: [Naming; 4] = [Naming::Snake, Naming::Kebab, Naming::Camel, Naming::Pascal];

    pub fn as_str(self) -> &'static str {
        match self {
            Naming::Snake => "snake_case",
            Naming::Kebab => "kebab-case",
            Naming::Camel => "camelCase",
            Naming::Pascal => "PascalCase",
        }
    }

    pub fn parse(raw: &str) -> Result<Naming, GuideError> {
        let raw = raw.trim();
        Naming::ALL
            .into_iter()
            .find(|n| n.as_str() == raw)
            .ok_or_else(|| {
                let names: Vec<&str> = Naming::ALL.iter().map(|n| n.as_str()).collect();
                GuideError::new(
                    "naming",
                    format!(
                        "{raw:?} no es una convención de nombres; usa {}",
                        quoted_list(&names)
                    ),
                )
            })
    }

    /// Nombre de archivo (sin extensión) para un nombre libre: palabras en
    /// ASCII, sin tildes, unidas según la convención. `None` si no queda
    /// ninguna letra ni cifra.
    pub fn apply(self, name: &str) -> Option<String> {
        let words = words(name);
        if words.is_empty() {
            return None;
        }
        let capital = |w: &str| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        };
        Some(match self {
            Naming::Snake => words.join("_"),
            Naming::Kebab => words.join("-"),
            Naming::Pascal => words.iter().map(|w| capital(w)).collect(),
            Naming::Camel => {
                let mut out = words[0].clone();
                out.extend(words[1..].iter().map(|w| capital(w)));
                out
            }
        })
    }
}

/// Palabras en minúsculas ASCII: corta en lo que no sea letra o cifra y en
/// los cambios de minúscula a mayúscula («CofreAbierto» → cofre, abierto).
fn words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut prev_lower = false;
    for c in name.chars() {
        let Some(c) = fold(c) else {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            prev_lower = false;
            continue;
        };
        if c.is_ascii_uppercase() && prev_lower && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
        prev_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        current.push(c.to_ascii_lowercase());
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// La letra o cifra ASCII que corresponde a `c` (quita tildes), o `None`.
fn fold(c: char) -> Option<char> {
    if c.is_ascii_alphanumeric() {
        return Some(c);
    }
    let plain = match c {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => 'a',
        'Á' | 'À' | 'Â' | 'Ä' | 'Ã' | 'Å' => 'A',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
        'Ó' | 'Ò' | 'Ô' | 'Ö' | 'Õ' => 'O',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
        'ñ' => 'n',
        'Ñ' => 'N',
        'ç' => 'c',
        'Ç' => 'C',
        _ => return None,
    };
    Some(plain)
}

fn quoted_list(items: &[&str]) -> String {
    let quoted: Vec<String> = items.iter().map(|s| format!("\"{s}\"")).collect();
    match quoted.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} o {last}", rest.join(", ")),
        _ => quoted.join(""),
    }
}

/// La guía de arte de un juego.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtGuide {
    /// Estilo en pocas palabras; lo recibe el agente con cada encargo.
    pub style: String,
    /// Tamaño por defecto de los recursos (ancho, alto).
    pub size: (i64, i64),
    pub palette: GuidePalette,
    /// Carpeta de salida, relativa a la carpeta de la guía, con `/`.
    pub output: String,
    pub naming: Naming,
}

/// Carpeta de salida por defecto.
pub const DEFAULT_OUTPUT: &str = "assets/art";

impl Default for ArtGuide {
    fn default() -> Self {
        Self {
            style: String::new(),
            size: (32, 32),
            palette: GuidePalette::Free,
            output: DEFAULT_OUTPUT.to_string(),
            naming: Naming::Snake,
        }
    }
}

impl ArtGuide {
    /// Lee el tamaño `ANCHOxALTO` (de 1 a 512 por lado).
    pub fn parse_size(raw: &str) -> Result<(i64, i64), GuideError> {
        let bad = || {
            GuideError::new(
                "size",
                format!("{raw:?} no es un tamaño; usa ANCHOxALTO, por ejemplo \"32x32\""),
            )
        };
        let lower = raw.trim().to_ascii_lowercase();
        let (w, h) = lower.split_once('x').ok_or_else(bad)?;
        let w: i64 = w.trim().parse().map_err(|_| bad())?;
        let h: i64 = h.trim().parse().map_err(|_| bad())?;
        let range = 1..=i64::from(MAX_CANVAS_SIZE);
        if !range.contains(&w) || !range.contains(&h) {
            return Err(GuideError::new(
                "size",
                format!("{raw:?}: el ancho y el alto van de 1 a {MAX_CANVAS_SIZE}"),
            ));
        }
        Ok((w, h))
    }

    /// Normaliza la carpeta de salida: relativa, dentro del proyecto (sin
    /// `..`), con `/` y sin `.` ni barras sobrantes.
    pub fn parse_output(raw: &str) -> Result<String, GuideError> {
        let trimmed = raw.trim();
        let absolute = trimmed.starts_with('/')
            || trimmed.starts_with('\\')
            || trimmed.chars().nth(1) == Some(':');
        if absolute {
            return Err(GuideError::new(
                "output",
                format!(
                    "{raw:?} es una ruta absoluta; escribe la carpeta relativa a lumeart.toml, por ejemplo \"{DEFAULT_OUTPUT}\""
                ),
            ));
        }
        let mut parts = Vec::new();
        for part in trimmed.split(['/', '\\']) {
            match part {
                "" | "." => {}
                ".." => {
                    return Err(GuideError::new(
                        "output",
                        format!(
                            "{raw:?} sale de la carpeta del proyecto; usa una carpeta dentro de ella"
                        ),
                    ));
                }
                other => parts.push(other),
            }
        }
        if parts.is_empty() {
            return Err(GuideError::new(
                "output",
                format!("la carpeta de salida está vacía; por ejemplo \"{DEFAULT_OUTPUT}\""),
            ));
        }
        Ok(parts.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let g = ArtGuide::default();
        assert_eq!(g.size, (32, 32));
        assert_eq!(g.output, "assets/art");
        assert_eq!(g.palette, GuidePalette::Free);
        assert_eq!(g.naming, Naming::Snake);
        assert!(g.palette.palette().is_none());
    }

    #[test]
    fn sizes() {
        assert_eq!(ArtGuide::parse_size("16x24"), Ok((16, 24)));
        assert_eq!(ArtGuide::parse_size(" 64X64 "), Ok((64, 64)));
        assert_eq!(ArtGuide::parse_size("512x1"), Ok((512, 1)));
        let e = ArtGuide::parse_size("32").unwrap_err();
        assert_eq!(e.key, "size");
        assert_eq!(
            e.to_string(),
            "size: \"32\" no es un tamaño; usa ANCHOxALTO, por ejemplo \"32x32\""
        );
        assert_eq!(
            ArtGuide::parse_size("0x8").unwrap_err().message,
            "\"0x8\": el ancho y el alto van de 1 a 512"
        );
        assert!(ArtGuide::parse_size("600x8").is_err());
        assert!(ArtGuide::parse_size("axb").is_err());
    }

    #[test]
    fn outputs() {
        assert_eq!(
            ArtGuide::parse_output("assets/art/"),
            Ok("assets/art".into())
        );
        assert_eq!(
            ArtGuide::parse_output("./res\\sprites//"),
            Ok("res/sprites".into())
        );
        for bad in ["/home/x", "C:\\juego", "\\\\red\\x"] {
            let e = ArtGuide::parse_output(bad).unwrap_err();
            assert!(e.message.contains("ruta absoluta"), "{bad}: {e}");
        }
        assert!(
            ArtGuide::parse_output("../otro")
                .unwrap_err()
                .message
                .contains("sale de la carpeta")
        );
        assert!(
            ArtGuide::parse_output(" ./ ")
                .unwrap_err()
                .message
                .contains("vacía")
        );
    }

    #[test]
    fn palettes() {
        assert_eq!(GuidePalette::parse_name("libre"), Ok(GuidePalette::Free));
        let gb = GuidePalette::parse_name("game-boy").unwrap();
        assert_eq!(gb.palette().unwrap().len(), 4);
        assert_eq!(
            GuidePalette::parse_name("nes").unwrap_err().message,
            "no hay ninguna paleta llamada \"nes\"; usa \"libre\", \"pico-8\", \"game-boy\" o una lista de colores como [\"#1a1c2c\", \"#f4f4f4\"]"
        );
        let custom = GuidePalette::parse_colors(&["#000000", "white", "#ff000080"]).unwrap();
        let palette = custom.palette().unwrap();
        assert_eq!(palette.len(), 3);
        assert_eq!(palette.colors()[1].color, Rgba::rgb(255, 255, 255));
        assert_eq!(
            GuidePalette::parse_colors(&["#000", "#zz0000"])
                .unwrap_err()
                .message,
            "el color 2 (\"#zz0000\") no es válido; usa #rrggbb, #rrggbbaa o un nombre como \"red\""
        );
        assert_eq!(
            GuidePalette::parse_colors(&["#ffffff", "black", "white"])
                .unwrap_err()
                .message,
            "el color 3 (\"white\") repite el color 1"
        );
        assert!(
            GuidePalette::parse_colors::<&str>(&[])
                .unwrap_err()
                .message
                .contains("vacía")
        );
    }

    #[test]
    fn naming() {
        assert_eq!(Naming::parse("kebab-case"), Ok(Naming::Kebab));
        assert_eq!(
            Naming::parse("snake").unwrap_err().to_string(),
            "naming: \"snake\" no es una convención de nombres; usa \"snake_case\", \"kebab-case\", \"camelCase\" o \"PascalCase\""
        );
        let name = "Poción de maná (grande) 2";
        assert_eq!(
            Naming::Snake.apply(name).unwrap(),
            "pocion_de_mana_grande_2"
        );
        assert_eq!(
            Naming::Kebab.apply(name).unwrap(),
            "pocion-de-mana-grande-2"
        );
        assert_eq!(Naming::Camel.apply(name).unwrap(), "pocionDeManaGrande2");
        assert_eq!(Naming::Pascal.apply(name).unwrap(), "PocionDeManaGrande2");
        assert_eq!(
            Naming::Snake.apply("CofreAbierto").unwrap(),
            "cofre_abierto"
        );
        assert_eq!(
            Naming::Snake.apply("cofre_de-madera").unwrap(),
            "cofre_de_madera"
        );
        assert_eq!(Naming::Snake.apply("¿?…"), None);
        for n in Naming::ALL {
            assert_eq!(Naming::parse(n.as_str()), Ok(n));
        }
    }
}
