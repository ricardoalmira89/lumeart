//! Modelo de animación (`anim.js`), compatible con `internal/anim` de pxcli.
//!
//! Aquí está lo que no depende del motor de JavaScript: la imagen RGBA de
//! cada fotograma ([`Image`]), el resultado de ejecutar `anim.js`
//! ([`AnimResult`]), la receta de comandos ([`recipe`]), las comprobaciones
//! ([`check`]), las hojas ([`sheet`]) y el esqueleto inicial ([`skeleton`]).
//! El motor (QuickJS) está en `lume-script`.

pub mod animation;
pub mod check;
pub mod export;
pub mod project;
pub mod recipe;
pub mod sheet;
pub mod skeleton;

use serde::{Deserialize, Serialize};

use crate::canvas::Canvas;
use crate::color::Rgba;

/// Lado máximo de una celda.
pub const MAX_CELL: u32 = 256;
/// Fotogramas por acción, como mucho.
pub const MAX_FRAMES: u32 = 64;
/// Acciones por animación, como mucho.
pub const MAX_ACTIONS: usize = 16;
/// Celda por defecto (sin base ni `config({ cell })`).
pub const DEFAULT_CELL: (u32, u32) = (32, 32);

/// Una imagen RGBA sin límite de tamaño (las hojas pueden pasar de 512).
/// Guarda los colores tal como los guarda `image.RGBA` de Go en pxcli.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<Rgba>,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Image({}x{})", self.width, self.height)
    }
}

impl Image {
    /// Imagen transparente.
    pub fn new(width: u32, height: u32) -> Image {
        Image {
            width,
            height,
            pixels: vec![Rgba::TRANSPARENT; width as usize * height as usize],
        }
    }

    /// Imagen con estos píxeles (por filas). `None` si no cuadra el tamaño.
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<Rgba>) -> Option<Image> {
        (pixels.len() == width as usize * height as usize).then_some(Image {
            width,
            height,
            pixels,
        })
    }

    /// Los píxeles de un lienzo.
    pub fn from_canvas(canvas: &Canvas) -> Image {
        Image {
            width: canvas.width(),
            height: canvas.height(),
            pixels: canvas.pixels().to_vec(),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// `(ancho, alto)`.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn pixels(&self) -> &[Rgba] {
        &self.pixels
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        (x >= 0 && y >= 0 && x < i64::from(self.width) && y < i64::from(self.height))
            .then(|| y as usize * self.width as usize + x as usize)
    }

    /// El píxel en (x, y); transparente fuera de la imagen.
    pub fn get(&self, x: i64, y: i64) -> Rgba {
        self.index(x, y)
            .map(|i| self.pixels[i])
            .unwrap_or(Rgba::TRANSPARENT)
    }

    /// Pinta un píxel; fuera de la imagen no hace nada.
    pub fn set(&mut self, x: i64, y: i64, color: Rgba) {
        if let Some(i) = self.index(x, y) {
            self.pixels[i] = color;
        }
    }

    /// Llena toda la imagen.
    pub fn fill(&mut self, color: Rgba) {
        self.pixels.fill(color);
    }

    /// Convierte colores sin premultiplicar (los de un PNG) en los que
    /// guarda `image.RGBA` de Go al cargar una base en pxcli: el alfa
    /// premultiplicado, con el mismo redondeo.
    pub fn premultiplied(mut self) -> Image {
        for p in &mut self.pixels {
            let a = u32::from(p.a);
            let channel = |c: u8| {
                let mut v = u32::from(c);
                v |= v << 8;
                v *= a;
                v /= 0xff;
                (v >> 8) as u8
            };
            *p = Rgba::new(channel(p.r), channel(p.g), channel(p.b), p.a);
        }
        self
    }
}

/// Los colores visibles de una imagen, sin repetir, por filas.
pub fn palette(img: &Image) -> Vec<Rgba> {
    let mut out: Vec<Rgba> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for &c in img.pixels() {
        if c.a > 0 && seen.insert(c) {
            out.push(c);
        }
    }
    out
}

/// ¿Mismos píxeles?
pub fn same_image(a: &Image, b: &Image) -> bool {
    diff_count(a, b) == 0
}

/// Píxeles distintos; de distinto tamaño, todos los de `a`.
pub fn diff_count(a: &Image, b: &Image) -> usize {
    if a.size() != b.size() {
        return a.pixels.len();
    }
    a.pixels
        .iter()
        .zip(&b.pixels)
        .filter(|(x, y)| x != y)
        .count()
}

/// Un color en corto: `transparent`, `#rrggbb` o `#rrggbbaa` (`Hex` de pxcli).
pub fn hex(c: Rgba) -> String {
    match c.a {
        0 => "transparent".into(),
        255 => format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b),
        _ => format!("#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, c.a),
    }
}

/// Gravedad de un problema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warning,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// Un problema al ejecutar o comprobar `anim.js` (el JSON de pxcli).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Issue {
    pub level: Level,
    /// Línea de `anim.js` (0: sin línea).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub line: u32,
    /// Dónde: «reposo», el nombre de una acción, «andar f3»…
    #[serde(default, rename = "where", skip_serializing_if = "String::is_empty")]
    pub place: String,
    pub msg: String,
}

/// Una fila de la hoja tal como se pide: nombre, fotogramas, fps y bucle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActionSpec {
    pub name: String,
    pub frames: u32,
    pub fps: u32,
    #[serde(rename = "loop")]
    pub looping: bool,
}

/// Una llamada a `draw(pieza, params)` en un fotograma: la pieza y una
/// firma de cómo se dibujó (parámetros y movimientos en curso).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartCall {
    pub part: String,
    pub key: String,
}

/// Una acción dibujada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub spec: ActionSpec,
    /// Línea de `action(...)` en `anim.js`.
    pub line: u32,
    /// Un fotograma por cada `f`.
    pub images: Vec<Image>,
    /// Píxeles que se salieron de la celda en cada fotograma.
    pub out_of_bounds: Vec<usize>,
    /// Las piezas que dibujó cada fotograma, en orden.
    pub part_calls: Vec<Vec<PartCall>>,
}

/// La pista de una pieza en una acción: en qué fotogramas se dibuja y en
/// cuáles cambia (un rombo en la línea de tiempo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub part: String,
    pub drawn: Vec<bool>,
    pub keys: Vec<bool>,
}

/// Las pistas de las piezas que usa una acción, en el orden en que se
/// declararon. Un fotograma es clave para una pieza si la dibuja y es el
/// primero, o la dibuja de otra forma (otros parámetros u otra posición) que
/// en el anterior.
pub fn tracks(res: &AnimResult, action: &Action) -> Vec<Track> {
    let frames = action.part_calls.len();
    let calls = |f: usize, part: &str| -> Vec<&str> {
        action.part_calls[f]
            .iter()
            .filter(|c| c.part == part)
            .map(|c| c.key.as_str())
            .collect()
    };
    let mut out = Vec::new();
    for part in &res.parts {
        let drawn: Vec<bool> = (0..frames).map(|f| !calls(f, part).is_empty()).collect();
        if !drawn.iter().any(|d| *d) {
            continue;
        }
        let keys = (0..frames)
            .map(|f| drawn[f] && (f == 0 || !drawn[f - 1] || calls(f, part) != calls(f - 1, part)))
            .collect();
        out.push(Track {
            part: part.clone(),
            drawn,
            keys,
        });
    }
    out
}

/// Todo lo que sale de ejecutar `anim.js`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimResult {
    pub name: String,
    pub cell: (u32, u32),
    /// Dónde cae el (0, 0) de la base dentro de cada celda.
    pub origin: (i64, i64),
    /// Colores de más declarados en `config({ colors })`.
    pub colors: Vec<Rgba>,
    pub base: Option<Image>,
    /// La pose de reposo, si hay `rest()`.
    pub rest: Option<Image>,
    pub actions: Vec<Action>,
    pub issues: Vec<Issue>,
    /// Las piezas (`part`), en el orden en que se declararon.
    pub parts: Vec<String>,
}

impl Default for AnimResult {
    fn default() -> Self {
        Self {
            name: String::new(),
            cell: DEFAULT_CELL,
            origin: (0, 0),
            colors: Vec::new(),
            base: None,
            rest: None,
            actions: Vec::new(),
            issues: Vec::new(),
            parts: Vec::new(),
        }
    }
}

impl AnimResult {
    /// Solo los errores.
    pub fn errors(&self) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|i| i.level == Level::Error)
            .collect()
    }

    /// Añade un problema.
    pub fn issue(
        &mut self,
        level: Level,
        line: u32,
        place: impl Into<String>,
        msg: impl Into<String>,
    ) {
        self.issues.push(Issue {
            level,
            line,
            place: place.into(),
            msg: msg.into(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premultiplied_like_go() {
        // Blanco al 16 %: Go guarda 0x29 en cada canal.
        let img = Image::from_pixels(
            2,
            1,
            vec![Rgba::new(255, 255, 255, 0x29), Rgba::new(200, 100, 0, 255)],
        )
        .unwrap()
        .premultiplied();
        assert_eq!(img.get(0, 0), Rgba::new(0x29, 0x29, 0x29, 0x29));
        assert_eq!(img.get(1, 0), Rgba::new(200, 100, 0, 255));
        // 0x80 de 0xff al 50 %: (0x8080 * 0x80 / 0xff) >> 8 = 0x40.
        let half = Image::from_pixels(1, 1, vec![Rgba::new(0x80, 0, 0, 0x80)])
            .unwrap()
            .premultiplied();
        assert_eq!(half.get(0, 0).r, 0x40);
    }

    #[test]
    fn hex_palette_and_diffs() {
        assert_eq!(hex(Rgba::TRANSPARENT), "transparent");
        assert_eq!(hex(Rgba::rgb(1, 2, 3)), "#010203");
        assert_eq!(hex(Rgba::new(1, 2, 3, 4)), "#01020304");
        let mut a = Image::new(3, 2);
        a.set(2, 1, Rgba::rgb(9, 9, 9));
        a.set(0, 0, Rgba::rgb(1, 1, 1));
        a.set(1, 0, Rgba::new(5, 5, 5, 0));
        a.set(9, 9, Rgba::rgb(7, 7, 7));
        assert_eq!(palette(&a), [Rgba::rgb(1, 1, 1), Rgba::rgb(9, 9, 9)]);
        let mut b = a.clone();
        assert!(same_image(&a, &b));
        b.set(0, 1, Rgba::rgb(2, 2, 2));
        assert_eq!(diff_count(&a, &b), 1);
        assert_eq!(diff_count(&a, &Image::new(2, 2)), 6);
    }

    #[test]
    fn issues_serialize_like_pxcli() {
        let issue = Issue {
            level: Level::Warning,
            line: 0,
            place: "reposo".into(),
            msg: "x".into(),
        };
        assert_eq!(
            serde_json::to_string(&issue).unwrap(),
            r#"{"level":"warning","where":"reposo","msg":"x"}"#
        );
    }
}
