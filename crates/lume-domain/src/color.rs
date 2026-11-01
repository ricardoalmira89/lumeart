//! Color RGBA de 8 bits por canal, compatible con `internal/color` de pxcli.
//!
//! Acepta `#rgb`, `#rrggbb`, `#rrggbbaa` y nombres; el formato canónico de
//! salida es `#rrggbbaa` en minúsculas.

use std::fmt;
use std::str::FromStr;

use crate::error::{DomainError, go_quote};

/// Color con 8 bits por canal (rojo, verde, azul y opacidad).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Colores con nombre, con los mismos valores que pxcli.
pub const NAMED_COLORS: &[(&str, Rgba)] = &[
    ("black", Rgba::rgb(0, 0, 0)),
    ("white", Rgba::rgb(255, 255, 255)),
    ("red", Rgba::rgb(255, 0, 0)),
    ("green", Rgba::rgb(0, 128, 0)),
    ("blue", Rgba::rgb(0, 0, 255)),
    ("yellow", Rgba::rgb(255, 255, 0)),
    ("orange", Rgba::rgb(255, 165, 0)),
    ("purple", Rgba::rgb(128, 0, 128)),
    ("cyan", Rgba::rgb(0, 255, 255)),
    ("magenta", Rgba::rgb(255, 0, 255)),
    ("gray", Rgba::rgb(128, 128, 128)),
    ("grey", Rgba::rgb(128, 128, 128)),
    ("transparent", Rgba::TRANSPARENT),
];

impl Rgba {
    /// Transparente: `#00000000`. Es el color de un lienzo nuevo.
    pub const TRANSPARENT: Rgba = Rgba::new(0, 0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Color opaco.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Busca un color por nombre (en minúsculas).
    pub fn named(name: &str) -> Option<Rgba> {
        NAMED_COLORS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, c)| *c)
    }

    /// Parsea un color como `Parse` de pxcli, con sus mismos errores.
    pub fn parse(input: &str) -> Result<Rgba, DomainError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(DomainError::invalid_color("color is required"));
        }

        let lower = trimmed.to_lowercase();
        if let Some(named) = Rgba::named(&lower) {
            return Ok(named);
        }

        let Some(hex) = lower.strip_prefix('#') else {
            return Err(DomainError::invalid_color(
                "expected hex color or named color",
            ));
        };

        let invalid =
            || DomainError::invalid_color(format!("invalid hex color {}", go_quote(trimmed)));
        let bytes = hex.as_bytes();
        match bytes.len() {
            3 => {
                let r = hex_nibble(bytes[0]).ok_or_else(invalid)?;
                let g = hex_nibble(bytes[1]).ok_or_else(invalid)?;
                let b = hex_nibble(bytes[2]).ok_or_else(invalid)?;
                Ok(Rgba::rgb(r * 17, g * 17, b * 17))
            }
            6 => {
                let r = hex_byte(&bytes[0..2]).ok_or_else(invalid)?;
                let g = hex_byte(&bytes[2..4]).ok_or_else(invalid)?;
                let b = hex_byte(&bytes[4..6]).ok_or_else(invalid)?;
                Ok(Rgba::rgb(r, g, b))
            }
            8 => {
                let r = hex_byte(&bytes[0..2]).ok_or_else(invalid)?;
                let g = hex_byte(&bytes[2..4]).ok_or_else(invalid)?;
                let b = hex_byte(&bytes[4..6]).ok_or_else(invalid)?;
                let a = hex_byte(&bytes[6..8]).ok_or_else(invalid)?;
                Ok(Rgba::new(r, g, b, a))
            }
            n => Err(DomainError::invalid_color(format!(
                "invalid hex length {n}"
            ))),
        }
    }
}

impl FromStr for Rgba {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Rgba::parse(s)
    }
}

/// Formato canónico `#rrggbbaa` en minúsculas.
impl fmt::Display for Rgba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

fn hex_byte(pair: &[u8]) -> Option<u8> {
    let hi = hex_nibble(pair[0])?;
    let lo = hex_nibble(pair[1])?;
    Some(hi << 4 | lo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;

    // --- Portados de internal/color/color_test.go ---

    #[test]
    fn parse_short_hex() {
        assert_eq!(Rgba::parse("#f00").unwrap(), Rgba::new(255, 0, 0, 255));
    }

    #[test]
    fn parse_invalid_color() {
        let err = Rgba::parse("#12").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidColor);
        let err = Rgba::parse("not-a-color").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidColor);
    }

    #[test]
    fn format_transparent() {
        assert_eq!(Rgba::new(0, 0, 0, 0).to_string(), "#00000000");
    }

    // --- Casos adicionales para fijar la conducta de pxcli ---

    #[test]
    fn parse_hex_lengths() {
        assert_eq!(Rgba::parse("#abc").unwrap(), Rgba::rgb(0xaa, 0xbb, 0xcc));
        assert_eq!(Rgba::parse("#123456").unwrap(), Rgba::rgb(0x12, 0x34, 0x56));
        assert_eq!(
            Rgba::parse("#12345678").unwrap(),
            Rgba::new(0x12, 0x34, 0x56, 0x78)
        );
    }

    #[test]
    fn parse_is_case_insensitive_and_trims() {
        assert_eq!(Rgba::parse("#FF0000").unwrap(), Rgba::rgb(255, 0, 0));
        assert_eq!(
            Rgba::parse("#AbCdEf80").unwrap(),
            Rgba::new(0xab, 0xcd, 0xef, 0x80)
        );
        assert_eq!(Rgba::parse("RED").unwrap(), Rgba::rgb(255, 0, 0));
        assert_eq!(Rgba::parse("  Blue \t").unwrap(), Rgba::rgb(0, 0, 255));
    }

    #[test]
    fn named_colors_match_pxcli() {
        let expected = [
            ("black", "#000000ff"),
            ("white", "#ffffffff"),
            ("red", "#ff0000ff"),
            ("green", "#008000ff"),
            ("blue", "#0000ffff"),
            ("yellow", "#ffff00ff"),
            ("orange", "#ffa500ff"),
            ("purple", "#800080ff"),
            ("cyan", "#00ffffff"),
            ("magenta", "#ff00ffff"),
            ("gray", "#808080ff"),
            ("grey", "#808080ff"),
            ("transparent", "#00000000"),
        ];
        assert_eq!(NAMED_COLORS.len(), expected.len());
        for (name, hex) in expected {
            assert_eq!(Rgba::parse(name).unwrap().to_string(), hex, "{name}");
        }
    }

    #[test]
    fn error_messages_match_pxcli() {
        let cases = [
            ("", "color is required"),
            ("   ", "color is required"),
            ("not-a-color", "expected hex color or named color"),
            ("ff0000", "expected hex color or named color"),
            ("#12", "invalid hex length 2"),
            ("#", "invalid hex length 0"),
            ("#1234567", "invalid hex length 7"),
            ("#ggg", "invalid hex color \"#ggg\""),
            ("#12345g", "invalid hex color \"#12345g\""),
            ("#GG0000", "invalid hex color \"#GG0000\""),
            ("#1234567z", "invalid hex color \"#1234567z\""),
        ];
        for (input, message) in cases {
            let err = Rgba::parse(input).unwrap_err();
            assert_eq!(err.code, ErrorCode::InvalidColor, "{input:?}");
            assert_eq!(err.message, message, "{input:?}");
        }
    }

    #[test]
    fn display_is_lowercase_rrggbbaa() {
        assert_eq!(Rgba::new(0xAB, 0xCD, 0xEF, 0x01).to_string(), "#abcdef01");
        assert_eq!(Rgba::rgb(255, 0, 0).to_string(), "#ff0000ff");
    }

    #[test]
    fn from_str_and_round_trip() {
        for value in [0u8, 1, 15, 16, 127, 128, 254, 255] {
            let c = Rgba::new(value, 255 - value, value / 2, value);
            let parsed: Rgba = c.to_string().parse().unwrap();
            assert_eq!(parsed, c);
        }
    }
}
