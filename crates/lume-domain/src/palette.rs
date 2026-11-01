//! `Palette`: colores ordenados con nombre opcional.
//!
//! Se deriva del lienzo (los colores visibles en orden de primera aparición,
//! como `palette()` en `internal/anim/check.go` de pxcli) o es fija
//! (PICO-8, Game Boy) para limitar los colores de un dibujo.

use crate::canvas::Canvas;
use crate::color::Rgba;

/// Un color de la paleta, con su nombre si lo tiene («líquido», «contorno»…).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaletteColor {
    pub color: Rgba,
    pub name: Option<String>,
}

/// Colores ordenados y sin repetir.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Palette {
    /// Nombre de la paleta fija (`pico-8`, `game-boy`), si lo es.
    pub name: Option<String>,
    colors: Vec<PaletteColor>,
}

/// PICO-8: los 16 colores oficiales.
const PICO8: [(&str, Rgba); 16] = [
    ("negro", Rgba::rgb(0x00, 0x00, 0x00)),
    ("azul oscuro", Rgba::rgb(0x1d, 0x2b, 0x53)),
    ("morado oscuro", Rgba::rgb(0x7e, 0x25, 0x53)),
    ("verde oscuro", Rgba::rgb(0x00, 0x87, 0x51)),
    ("marrón", Rgba::rgb(0xab, 0x52, 0x36)),
    ("gris oscuro", Rgba::rgb(0x5f, 0x57, 0x4f)),
    ("gris claro", Rgba::rgb(0xc2, 0xc3, 0xc7)),
    ("blanco", Rgba::rgb(0xff, 0xf1, 0xe8)),
    ("rojo", Rgba::rgb(0xff, 0x00, 0x4d)),
    ("naranja", Rgba::rgb(0xff, 0xa3, 0x00)),
    ("amarillo", Rgba::rgb(0xff, 0xec, 0x27)),
    ("verde", Rgba::rgb(0x00, 0xe4, 0x36)),
    ("azul", Rgba::rgb(0x29, 0xad, 0xff)),
    ("lavanda", Rgba::rgb(0x83, 0x76, 0x9c)),
    ("rosa", Rgba::rgb(0xff, 0x77, 0xa8)),
    ("melocotón", Rgba::rgb(0xff, 0xcc, 0xaa)),
];

/// Game Boy (DMG): cuatro verdes.
const GAME_BOY: [(&str, Rgba); 4] = [
    ("más oscuro", Rgba::rgb(0x0f, 0x38, 0x0f)),
    ("oscuro", Rgba::rgb(0x30, 0x62, 0x30)),
    ("claro", Rgba::rgb(0x8b, 0xac, 0x0f)),
    ("más claro", Rgba::rgb(0x9b, 0xbc, 0x0f)),
];

/// Nombres de las paletas fijas disponibles.
pub const FIXED_PALETTES: &[&str] = &["pico-8", "game-boy"];

impl Palette {
    pub fn new() -> Self {
        Self::default()
    }

    /// Colores visibles (opacidad > 0) del lienzo, sin repetir, en orden de
    /// primera aparición (por filas, de arriba abajo).
    pub fn from_canvas(canvas: &Canvas) -> Palette {
        let mut palette = Palette::new();
        for &color in canvas.pixels() {
            if color.a > 0 {
                palette.push(color, None);
            }
        }
        palette
    }

    /// Una paleta fija por nombre (`pico-8`, `game-boy`).
    pub fn fixed(name: &str) -> Option<Palette> {
        let colors: &[(&str, Rgba)] = match name {
            "pico-8" => &PICO8,
            "game-boy" => &GAME_BOY,
            _ => return None,
        };
        let mut palette = Palette {
            name: Some(name.to_string()),
            colors: Vec::new(),
        };
        for (color_name, color) in colors {
            palette.push(*color, Some(color_name.to_string()));
        }
        Some(palette)
    }

    pub fn colors(&self) -> &[PaletteColor] {
        &self.colors
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    pub fn contains(&self, color: Rgba) -> bool {
        self.colors.iter().any(|c| c.color == color)
    }

    /// Añade un color al final si no está. Si ya está y se da un nombre, se
    /// lo pone. Devuelve su posición.
    pub fn push(&mut self, color: Rgba, name: Option<String>) -> usize {
        if let Some(i) = self.colors.iter().position(|c| c.color == color) {
            if name.is_some() {
                self.colors[i].name = name;
            }
            return i;
        }
        self.colors.push(PaletteColor { color, name });
        self.colors.len() - 1
    }

    /// Pone o quita el nombre de un color.
    pub fn rename(&mut self, color: Rgba, name: Option<String>) -> bool {
        match self.colors.iter_mut().find(|c| c.color == color) {
            Some(entry) => {
                entry.name = name;
                true
            }
            None => false,
        }
    }

    /// Colores visibles del lienzo que no están en esta paleta, en orden de
    /// aparición (para comprobar una paleta fija).
    pub fn colors_outside(&self, canvas: &Canvas) -> Vec<Rgba> {
        Palette::from_canvas(canvas)
            .colors
            .into_iter()
            .map(|c| c.color)
            .filter(|c| !self.contains(*c))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgba = Rgba::rgb(255, 0, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    #[test]
    fn from_canvas_unique_in_order_without_transparent() {
        let mut c = Canvas::new(4, 3).unwrap();
        c.set(3, 0, BLUE).unwrap();
        c.set(0, 1, RED).unwrap();
        c.set(1, 1, BLUE).unwrap();
        c.set(2, 2, Rgba::new(10, 20, 30, 0)).unwrap(); // invisible
        c.set(3, 2, Rgba::new(10, 20, 30, 128)).unwrap();
        let p = Palette::from_canvas(&c);
        let colors: Vec<_> = p.colors().iter().map(|c| c.color).collect();
        assert_eq!(colors, vec![BLUE, RED, Rgba::new(10, 20, 30, 128)]);
        assert!(p.colors().iter().all(|c| c.name.is_none()));
        assert!(Palette::from_canvas(&Canvas::new(2, 2).unwrap()).is_empty());
    }

    #[test]
    fn fixed_palettes() {
        let pico = Palette::fixed("pico-8").unwrap();
        assert_eq!(pico.len(), 16);
        assert_eq!(pico.name.as_deref(), Some("pico-8"));
        assert_eq!(pico.colors()[8].color.to_string(), "#ff004dff");
        assert_eq!(pico.colors()[8].name.as_deref(), Some("rojo"));
        let gb = Palette::fixed("game-boy").unwrap();
        assert_eq!(gb.len(), 4);
        assert_eq!(gb.colors()[0].color.to_string(), "#0f380fff");
        assert!(Palette::fixed("nes").is_none());
        for name in FIXED_PALETTES {
            assert!(Palette::fixed(name).is_some());
        }
    }

    #[test]
    fn push_and_rename() {
        let mut p = Palette::new();
        assert_eq!(p.push(RED, None), 0);
        assert_eq!(p.push(BLUE, Some("cielo".into())), 1);
        assert_eq!(p.push(RED, Some("líquido".into())), 0);
        assert_eq!(p.len(), 2);
        assert_eq!(p.colors()[0].name.as_deref(), Some("líquido"));
        assert!(p.rename(BLUE, None));
        assert!(!p.rename(Rgba::rgb(1, 1, 1), None));
    }

    #[test]
    fn colors_outside_a_fixed_palette() {
        let gb = Palette::fixed("game-boy").unwrap();
        let mut c = Canvas::new(3, 1).unwrap();
        c.set(0, 0, Rgba::rgb(0x0f, 0x38, 0x0f)).unwrap();
        c.set(1, 0, RED).unwrap();
        assert_eq!(gb.colors_outside(&c), vec![RED]);
    }
}
