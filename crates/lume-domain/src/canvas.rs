//! Lienzo de píxeles, compatible con `internal/canvas` de pxcli.
//!
//! Las coordenadas de entrada son `i64` (como `int` en Go) para que las
//! negativas den `out_of_bounds` con el mismo mensaje que pxcli. Las
//! operaciones que modifican el lienzo devuelven las celdas cambiadas, base del
//! `Patch` para deshacer.

use crate::color::Rgba;
use crate::error::DomainError;

/// Tamaño máximo de ancho y alto (el límite de `new` en pxcli).
pub const MAX_CANVAS_SIZE: u32 = 512;

/// Una celda cambiada: posición, color anterior y color nuevo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellChange {
    pub x: u32,
    pub y: u32,
    pub before: Rgba,
    pub after: Rgba,
}

/// Cambio de tamaño del lienzo: tamaños antes y después, y las celdas con
/// color que quedaron fuera (para poder deshacerlo).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SizeChange {
    pub before: (u32, u32),
    pub after: (u32, u32),
    pub lost: Vec<(u32, u32, Rgba)>,
}

/// Búfer RGBA de ancho × alto, transparente al crearse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<Rgba>,
}

impl Canvas {
    /// Lienzo transparente de `width` × `height` (entre 1 y 512).
    pub fn new(width: i64, height: i64) -> Result<Canvas, DomainError> {
        let (width, height) = validate_size(width, height)?;
        Ok(Canvas {
            width,
            height,
            pixels: vec![Rgba::TRANSPARENT; (width * height) as usize],
        })
    }

    /// Lienzo a partir de bytes RGBA (4 por píxel, por filas), como los de un
    /// PNG decodificado. El tamaño debe estar entre 1 y 512.
    pub fn from_rgba_bytes(width: i64, height: i64, bytes: &[u8]) -> Result<Canvas, DomainError> {
        let (w, h) = validate_size(width, height)?;
        let expected = w as usize * h as usize * 4;
        if bytes.len() != expected {
            return Err(DomainError::invalid_args(format!(
                "expected {expected} bytes for a {w}x{h} canvas, got {}",
                bytes.len()
            )));
        }
        let pixels = bytes
            .chunks_exact(4)
            .map(|p| Rgba::new(p[0], p[1], p[2], p[3]))
            .collect();
        Ok(Canvas {
            width: w,
            height: h,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Píxeles por filas, de arriba abajo y de izquierda a derecha.
    pub fn pixels(&self) -> &[Rgba] {
        &self.pixels
    }

    /// Copia del lienzo como bytes RGBA (4 por píxel), para PNG o texturas.
    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|c| [c.r, c.g, c.b, c.a])
            .collect()
    }

    /// Bytes RGBA del lienzo ampliado `scale` veces (cada píxel pasa a ser un
    /// bloque de `scale` × `scale`, vecino más cercano). Devuelve también el
    /// ancho y el alto resultantes.
    pub fn scaled_rgba_bytes(&self, scale: u32) -> (u32, u32, Vec<u8>) {
        let scale = scale.max(1);
        let (w, h) = (self.width * scale, self.height * scale);
        let mut bytes = Vec::with_capacity(w as usize * h as usize * 4);
        for y in 0..h {
            let row = (y / scale) as usize * self.width as usize;
            for x in 0..w {
                let c = self.pixels[row + (x / scale) as usize];
                bytes.extend_from_slice(&[c.r, c.g, c.b, c.a]);
            }
        }
        (w, h, bytes)
    }

    /// Color del píxel en (`x`, `y`).
    pub fn get(&self, x: i64, y: i64) -> Result<Rgba, DomainError> {
        let idx = self.index(x, y)?;
        Ok(self.pixels[idx])
    }

    /// Pinta un píxel. Devuelve la celda cambiada, o nada si ya tenía ese color.
    pub fn set(&mut self, x: i64, y: i64, color: Rgba) -> Result<Vec<CellChange>, DomainError> {
        let idx = self.index(x, y)?;
        Ok(self.write(idx, color).into_iter().collect())
    }

    /// Pinta todo el lienzo de un color. Devuelve las celdas que cambiaron.
    pub fn clear(&mut self, color: Rgba) -> Vec<CellChange> {
        (0..self.pixels.len())
            .filter_map(|idx| self.write(idx, color))
            .collect()
    }

    /// Rellena un rectángulo de `w` × `h` con la esquina en (`x`, `y`).
    ///
    /// Como pxcli: primero `w`/`h` ≤ 0 → `invalid_args`; después, si no cabe
    /// entero en el lienzo → `out_of_bounds`. Devuelve las celdas cambiadas.
    pub fn fill_rect(
        &mut self,
        x: i64,
        y: i64,
        w: i64,
        h: i64,
        color: Rgba,
    ) -> Result<Vec<CellChange>, DomainError> {
        if w <= 0 || h <= 0 {
            return Err(DomainError::invalid_args(
                "rect width and height must be positive",
            ));
        }
        // En i128 para que x + w no desborde con valores enormes.
        let fits = x >= 0
            && y >= 0
            && i128::from(x) + i128::from(w) <= i128::from(self.width)
            && i128::from(y) + i128::from(h) <= i128::from(self.height);
        if !fits {
            return Err(DomainError::out_of_bounds(format!(
                "rect ({x},{y}) size {w}x{h} outside canvas"
            )));
        }
        let width = self.width as usize;
        let mut changes = Vec::new();
        for row in y as usize..(y + h) as usize {
            let start = row * width + x as usize;
            for idx in start..start + w as usize {
                changes.extend(self.write(idx, color));
            }
        }
        Ok(changes)
    }

    /// Línea de (`x1`, `y1`) a (`x2`, `y2`) con Bresenham, extremos incluidos.
    ///
    /// Valida el primer extremo y luego el segundo, como pxcli. Devuelve las
    /// celdas cambiadas en el orden en que se recorren.
    pub fn line(
        &mut self,
        x1: i64,
        y1: i64,
        x2: i64,
        y2: i64,
        color: Rgba,
    ) -> Result<Vec<CellChange>, DomainError> {
        self.index(x1, y1)?;
        self.index(x2, y2)?;
        Ok(bresenham(x1, y1, x2, y2)
            .into_iter()
            .filter_map(|(x, y)| {
                let idx = y as usize * self.width as usize + x as usize;
                self.write(idx, color)
            })
            .collect())
    }

    /// Cambia el tamaño conservando el contenido anclado arriba a la izquierda:
    /// lo que sobra se recorta y lo nuevo queda transparente. Devuelve `None`
    /// si el tamaño no cambia.
    pub fn resize(&mut self, width: i64, height: i64) -> Result<Option<SizeChange>, DomainError> {
        let (width, height) = validate_size(width, height)?;
        if (width, height) == (self.width, self.height) {
            return Ok(None);
        }
        let mut lost = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.pixels[self.offset(x, y)];
                if (x >= width || y >= height) && color != Rgba::TRANSPARENT {
                    lost.push((x, y, color));
                }
            }
        }
        let before = (self.width, self.height);
        self.set_size(width, height);
        Ok(Some(SizeChange {
            before,
            after: (width, height),
            lost,
        }))
    }

    /// Cambia el tamaño sin validar (el tamaño ya viene validado).
    pub(crate) fn set_size(&mut self, width: u32, height: u32) {
        let mut pixels = vec![Rgba::TRANSPARENT; width as usize * height as usize];
        for y in 0..height.min(self.height) {
            for x in 0..width.min(self.width) {
                pixels[y as usize * width as usize + x as usize] = self.pixels[self.offset(x, y)];
            }
        }
        self.width = width;
        self.height = height;
        self.pixels = pixels;
    }

    /// Escribe en una celda que se sabe dentro del lienzo.
    pub(crate) fn put(&mut self, x: u32, y: u32, color: Rgba) {
        let idx = self.offset(x, y);
        self.pixels[idx] = color;
    }

    /// ¿Está (`x`, `y`) dentro del lienzo?
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    fn offset(&self, x: u32, y: u32) -> usize {
        y as usize * self.width as usize + x as usize
    }

    /// Índice de (`x`, `y`) o `out_of_bounds` con el mensaje de pxcli.
    pub(crate) fn index(&self, x: i64, y: i64) -> Result<usize, DomainError> {
        if x < 0 || x >= i64::from(self.width) || y < 0 || y >= i64::from(self.height) {
            return Err(DomainError::out_of_bounds(format!(
                "pixel ({x},{y}) outside canvas"
            )));
        }
        Ok(y as usize * self.width as usize + x as usize)
    }

    /// Escribe un color en un índice válido y devuelve el cambio, si lo hay.
    pub(crate) fn write(&mut self, idx: usize, color: Rgba) -> Option<CellChange> {
        let before = self.pixels[idx];
        if before == color {
            return None;
        }
        self.pixels[idx] = color;
        let width = self.width as usize;
        Some(CellChange {
            x: (idx % width) as u32,
            y: (idx / width) as u32,
            before,
            after: color,
        })
    }
}

/// Celdas en las que difieren dos lienzos, por filas. Si tienen distinto
/// tamaño se compara el área que cubren entre los dos y lo que queda fuera de
/// uno cuenta como transparente. `before` es el color en `a` y `after` en `b`.
pub fn diff(a: &Canvas, b: &Canvas) -> Vec<CellChange> {
    let width = a.width.max(b.width);
    let height = a.height.max(b.height);
    let color_at = |c: &Canvas, x: u32, y: u32| {
        if c.contains(x, y) {
            c.pixels[c.offset(x, y)]
        } else {
            Rgba::TRANSPARENT
        }
    };
    let mut cells = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let before = color_at(a, x, y);
            let after = color_at(b, x, y);
            if before != after {
                cells.push(CellChange {
                    x,
                    y,
                    before,
                    after,
                });
            }
        }
    }
    cells
}

/// Puntos de la línea de Bresenham, en el mismo orden que pxcli.
pub fn bresenham(mut x1: i64, mut y1: i64, x2: i64, y2: i64) -> Vec<(i64, i64)> {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut points = Vec::with_capacity((dx.max(dy) + 1) as usize);
    loop {
        points.push((x1, y1));
        if x1 == x2 && y1 == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x1 += sx;
        }
        if e2 < dx {
            err += dx;
            y1 += sy;
        }
    }
    points
}

/// Valida un tamaño de lienzo con los mensajes de pxcli.
pub fn validate_size(width: i64, height: i64) -> Result<(u32, u32), DomainError> {
    if width <= 0 || height <= 0 {
        return Err(DomainError::invalid_args(
            "canvas dimensions must be positive",
        ));
    }
    let max = i64::from(MAX_CANVAS_SIZE);
    if width > max || height > max {
        return Err(DomainError::invalid_args(format!(
            "width and height must be between 1 and {MAX_CANVAS_SIZE}"
        )));
    }
    Ok((width as u32, height as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;

    const RED: Rgba = Rgba::rgb(255, 0, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    #[test]
    fn new_canvas_is_transparent() {
        let c = Canvas::new(32, 32).unwrap();
        assert_eq!((c.width(), c.height()), (32, 32));
        assert_eq!(c.pixels().len(), 32 * 32);
        for y in 0..32 {
            for x in 0..32 {
                assert_eq!(c.get(x, y).unwrap().to_string(), "#00000000");
            }
        }
    }

    // Portado de TestCanvasSetGetPixel.
    #[test]
    fn set_get_pixel() {
        let mut c = Canvas::new(4, 4).unwrap();
        c.set(1, 2, RED).unwrap();
        assert_eq!(c.get(1, 2).unwrap(), RED);
        assert_eq!(c.get(2, 1).unwrap(), Rgba::TRANSPARENT);
        assert_eq!(c.pixels()[2 * 4 + 1], RED);
    }

    // Portado de TestCanvasBounds.
    #[test]
    fn bounds() {
        let mut c = Canvas::new(4, 4).unwrap();
        for (x, y) in [(-1, 0), (0, -1), (4, 0), (0, 4)] {
            let err = c.set(x, y, Rgba::TRANSPARENT).unwrap_err();
            assert_eq!(err.code, ErrorCode::OutOfBounds, "set ({x},{y})");
            let err = c.get(x, y).unwrap_err();
            assert_eq!(err.code, ErrorCode::OutOfBounds, "get ({x},{y})");
        }
    }

    #[test]
    fn out_of_bounds_message_matches_pxcli() {
        let mut c = Canvas::new(32, 32).unwrap();
        let err = c.set(-1, 10, RED).unwrap_err();
        assert_eq!(err.message, "pixel (-1,10) outside canvas");
        let err = c.get(32, 0).unwrap_err();
        assert_eq!(err.message, "pixel (32,0) outside canvas");
        let err = c.get(i64::MAX, i64::MIN).unwrap_err();
        assert_eq!(
            err.message,
            format!("pixel ({},{}) outside canvas", i64::MAX, i64::MIN)
        );
    }

    // Portado de TestCanvasClear.
    #[test]
    fn clear_paints_everything() {
        let mut c = Canvas::new(3, 2).unwrap();
        let changes = c.clear(BLUE);
        assert_eq!(changes.len(), 6);
        assert!(c.pixels().iter().all(|p| *p == BLUE));
    }

    #[test]
    fn set_returns_only_changed_cells() {
        let mut c = Canvas::new(4, 4).unwrap();
        let changes = c.set(3, 1, RED).unwrap();
        assert_eq!(
            changes,
            vec![CellChange {
                x: 3,
                y: 1,
                before: Rgba::TRANSPARENT,
                after: RED
            }]
        );
        assert!(c.set(3, 1, RED).unwrap().is_empty());
    }

    #[test]
    fn clear_returns_only_changed_cells() {
        let mut c = Canvas::new(3, 3).unwrap();
        c.set(1, 1, BLUE).unwrap();
        let changes = c.clear(BLUE);
        assert_eq!(changes.len(), 8);
        assert!(!changes.iter().any(|ch| ch.x == 1 && ch.y == 1));
        assert!(changes.iter().all(|ch| ch.before == Rgba::TRANSPARENT));
        assert!(c.clear(BLUE).is_empty());
    }

    #[test]
    fn invalid_sizes_are_rejected() {
        for (w, h) in [(0, 4), (4, 0), (-1, 4), (4, -3), (0, 0)] {
            let err = Canvas::new(w, h).unwrap_err();
            assert_eq!(err.code, ErrorCode::InvalidArgs, "{w}x{h}");
            assert_eq!(err.message, "canvas dimensions must be positive");
        }
        for (w, h) in [(513, 4), (4, 513), (100_000, 100_000)] {
            let err = Canvas::new(w, h).unwrap_err();
            assert_eq!(err.code, ErrorCode::InvalidArgs, "{w}x{h}");
            assert_eq!(err.message, "width and height must be between 1 and 512");
        }
        assert!(Canvas::new(512, 512).is_ok());
        assert!(Canvas::new(1, 1).is_ok());
    }

    #[test]
    fn rgba_bytes_follow_row_order() {
        let mut c = Canvas::new(2, 2).unwrap();
        c.set(1, 0, Rgba::new(1, 2, 3, 4)).unwrap();
        c.set(0, 1, Rgba::new(5, 6, 7, 8)).unwrap();
        assert_eq!(
            c.to_rgba_bytes(),
            vec![0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0]
        );
    }

    /// Celdas de un color, ordenadas por filas, para comparar con pxcli.
    fn cells_of(c: &Canvas, color: Rgba) -> Vec<(i64, i64)> {
        let mut out = Vec::new();
        for y in 0..i64::from(c.height()) {
            for x in 0..i64::from(c.width()) {
                if c.get(x, y).unwrap() == color {
                    out.push((x, y));
                }
            }
        }
        out
    }

    // Portado de TestCanvasFillRect.
    #[test]
    fn fill_rect_fills_only_the_rect() {
        let mut c = Canvas::new(8, 8).unwrap();
        let changes = c.fill_rect(2, 3, 3, 2, BLUE).unwrap();
        assert_eq!(changes.len(), 6);
        for y in 0..8 {
            for x in 0..8 {
                let in_rect = (2..=4).contains(&x) && (3..=4).contains(&y);
                let want = if in_rect { BLUE } else { Rgba::TRANSPARENT };
                assert_eq!(c.get(x, y).unwrap(), want, "({x},{y})");
            }
        }
    }

    // Portado de TestCanvasFillRectErrors.
    #[test]
    fn fill_rect_errors() {
        let mut c = Canvas::new(4, 4).unwrap();
        for (x, y, w, h) in [(0, 0, 0, 1), (0, 0, 1, 0), (0, 0, -1, 1), (0, 0, 1, -1)] {
            let err = c.fill_rect(x, y, w, h, RED).unwrap_err();
            assert_eq!(err.code, ErrorCode::InvalidArgs, "{x},{y},{w},{h}");
            assert_eq!(err.message, "rect width and height must be positive");
        }
        for (x, y, w, h) in [
            (-1, 0, 1, 1),
            (0, -1, 1, 1),
            (0, 0, 5, 1),
            (0, 0, 1, 5),
            (3, 3, 2, 1),
        ] {
            let err = c.fill_rect(x, y, w, h, RED).unwrap_err();
            assert_eq!(err.code, ErrorCode::OutOfBounds, "{x},{y},{w},{h}");
        }
        assert!(c.pixels().iter().all(|p| *p == Rgba::TRANSPARENT));
    }

    #[test]
    fn fill_rect_messages_match_pxcli() {
        let mut c = Canvas::new(4, 4).unwrap();
        let cases = [
            ((-1, 0, 1, 1), "rect (-1,0) size 1x1 outside canvas"),
            ((0, 0, 5, 1), "rect (0,0) size 5x1 outside canvas"),
            ((3, 3, 2, 1), "rect (3,3) size 2x1 outside canvas"),
        ];
        for ((x, y, w, h), msg) in cases {
            assert_eq!(c.fill_rect(x, y, w, h, RED).unwrap_err().message, msg);
        }
        // w negativo gana a la x negativa, como en pxcli.
        let err = c.fill_rect(-5, -5, -1, 2, RED).unwrap_err();
        assert_eq!(err.message, "rect width and height must be positive");
        // Sin desbordamiento con valores enormes.
        let err = c.fill_rect(1, 0, i64::MAX, 1, RED).unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfBounds);
    }

    #[test]
    fn fill_rect_reports_changed_cells() {
        let mut c = Canvas::new(4, 4).unwrap();
        c.set(1, 1, RED).unwrap();
        let changes = c.fill_rect(0, 0, 2, 2, RED).unwrap();
        assert_eq!(changes.len(), 3);
        assert!(
            changes
                .iter()
                .all(|ch| ch.before == Rgba::TRANSPARENT && ch.after == RED)
        );
        assert!(c.fill_rect(0, 0, 2, 2, RED).unwrap().is_empty());
    }

    // Portado de TestCanvasLineHorizontal.
    #[test]
    fn line_horizontal() {
        let mut c = Canvas::new(8, 8).unwrap();
        c.line(0, 0, 3, 0, RED).unwrap();
        assert_eq!(cells_of(&c, RED), vec![(0, 0), (1, 0), (2, 0), (3, 0)]);
    }

    // Portado de TestCanvasLineVertical.
    #[test]
    fn line_vertical() {
        let green = Rgba::rgb(0, 255, 0);
        let mut c = Canvas::new(6, 6).unwrap();
        c.line(2, 1, 2, 4, green).unwrap();
        assert_eq!(cells_of(&c, green), vec![(2, 1), (2, 2), (2, 3), (2, 4)]);
    }

    // Portado de TestCanvasLineDiagonal.
    #[test]
    fn line_diagonal() {
        let mut c = Canvas::new(5, 5).unwrap();
        c.line(0, 0, 3, 3, BLUE).unwrap();
        assert_eq!(cells_of(&c, BLUE), vec![(0, 0), (1, 1), (2, 2), (3, 3)]);
    }

    /// Salidas del `canvas.go` original sobre un lienzo de 8×8.
    #[test]
    fn line_slopes_match_pxcli() {
        type Segment = (i64, i64, i64, i64);
        type Cells = &'static [(i64, i64)];
        let cases: [(Segment, Cells); 8] = [
            (
                (0, 0, 7, 3),
                &[
                    (0, 0),
                    (1, 0),
                    (2, 1),
                    (3, 1),
                    (4, 2),
                    (5, 2),
                    (6, 3),
                    (7, 3),
                ],
            ),
            (
                (0, 0, 3, 7),
                &[
                    (0, 0),
                    (0, 1),
                    (1, 2),
                    (1, 3),
                    (2, 4),
                    (2, 5),
                    (3, 6),
                    (3, 7),
                ],
            ),
            (
                (7, 0, 0, 5),
                &[
                    (7, 0),
                    (5, 1),
                    (6, 1),
                    (4, 2),
                    (3, 3),
                    (1, 4),
                    (2, 4),
                    (0, 5),
                ],
            ),
            (
                (5, 6, 1, 1),
                &[(1, 1), (2, 2), (3, 3), (3, 4), (4, 5), (5, 6)],
            ),
            (
                (2, 7, 6, 0),
                &[
                    (6, 0),
                    (5, 1),
                    (5, 2),
                    (4, 3),
                    (4, 4),
                    (3, 5),
                    (3, 6),
                    (2, 7),
                ],
            ),
            (
                (7, 3, 0, 0),
                &[
                    (0, 0),
                    (1, 0),
                    (2, 1),
                    (3, 1),
                    (4, 2),
                    (5, 2),
                    (6, 3),
                    (7, 3),
                ],
            ),
            (
                (1, 6, 6, 4),
                &[(5, 4), (6, 4), (3, 5), (4, 5), (1, 6), (2, 6)],
            ),
            ((3, 3, 3, 3), &[(3, 3)]),
        ];
        for ((x1, y1, x2, y2), want) in cases {
            let mut c = Canvas::new(8, 8).unwrap();
            c.line(x1, y1, x2, y2, RED).unwrap();
            assert_eq!(cells_of(&c, RED), want, "({x1},{y1})-({x2},{y2})");
        }
    }

    // Portado de TestCanvasLineErrors, con los mensajes de pxcli.
    #[test]
    fn line_errors() {
        let mut c = Canvas::new(4, 4).unwrap();
        let cases = [
            ((0, 0, 4, 0), "pixel (4,0) outside canvas"),
            ((0, 0, 0, -1), "pixel (0,-1) outside canvas"),
            ((-1, 0, 2, 2), "pixel (-1,0) outside canvas"),
            ((9, 9, -1, -1), "pixel (9,9) outside canvas"),
        ];
        for ((x1, y1, x2, y2), msg) in cases {
            let err = c.line(x1, y1, x2, y2, RED).unwrap_err();
            assert_eq!(err.code, ErrorCode::OutOfBounds);
            assert_eq!(err.message, msg);
        }
        assert!(c.pixels().iter().all(|p| *p == Rgba::TRANSPARENT));
    }

    #[test]
    fn line_reports_changed_cells_in_order() {
        let mut c = Canvas::new(4, 4).unwrap();
        c.set(1, 0, RED).unwrap();
        let changes = c.line(0, 0, 3, 0, RED).unwrap();
        let cells: Vec<_> = changes.iter().map(|ch| (ch.x, ch.y)).collect();
        assert_eq!(cells, vec![(0, 0), (2, 0), (3, 0)]);
        assert!(
            changes
                .iter()
                .all(|ch| ch.before == Rgba::TRANSPARENT && ch.after == RED)
        );
    }

    #[test]
    fn resize_keeps_content_and_reports_lost_cells() {
        let mut c = Canvas::new(4, 3).unwrap();
        c.set(0, 0, RED).unwrap();
        c.set(3, 2, BLUE).unwrap();
        c.set(1, 2, BLUE).unwrap();
        let change = c.resize(2, 2).unwrap().unwrap();
        assert_eq!(change.before, (4, 3));
        assert_eq!(change.after, (2, 2));
        assert_eq!(change.lost, vec![(1, 2, BLUE), (3, 2, BLUE)]);
        assert_eq!((c.width(), c.height()), (2, 2));
        assert_eq!(c.get(0, 0).unwrap(), RED);

        c.resize(5, 5).unwrap();
        assert_eq!(c.get(0, 0).unwrap(), RED);
        assert_eq!(c.get(4, 4).unwrap(), Rgba::TRANSPARENT);
        assert!(c.resize(5, 5).unwrap().is_none());
    }

    #[test]
    fn resize_rejects_invalid_sizes() {
        let mut c = Canvas::new(4, 4).unwrap();
        assert_eq!(c.resize(0, 4).unwrap_err().code, ErrorCode::InvalidArgs);
        assert_eq!(c.resize(4, 600).unwrap_err().code, ErrorCode::InvalidArgs);
        assert_eq!((c.width(), c.height()), (4, 4));
    }

    #[test]
    fn from_rgba_bytes_round_trip() {
        let mut c = Canvas::new(3, 2).unwrap();
        c.set(2, 1, Rgba::new(1, 2, 3, 4)).unwrap();
        let back = Canvas::from_rgba_bytes(3, 2, &c.to_rgba_bytes()).unwrap();
        assert_eq!(back, c);
        let err = Canvas::from_rgba_bytes(3, 2, &[0; 5]).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidArgs);
        assert_eq!(err.message, "expected 24 bytes for a 3x2 canvas, got 5");
        assert_eq!(
            Canvas::from_rgba_bytes(600, 1, &[]).unwrap_err().code,
            ErrorCode::InvalidArgs
        );
    }

    #[test]
    fn scaled_bytes_use_nearest_neighbour() {
        let mut c = Canvas::new(2, 1).unwrap();
        c.set(1, 0, RED).unwrap();
        let (w, h, bytes) = c.scaled_rgba_bytes(3);
        assert_eq!((w, h), (6, 3));
        for y in 0..3 {
            for x in 0..6 {
                let i = (y * 6 + x) * 4;
                let want = if x >= 3 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 0, 0]
                };
                assert_eq!(&bytes[i..i + 4], &want, "({x},{y})");
            }
        }
        assert_eq!(c.scaled_rgba_bytes(1).2, c.to_rgba_bytes());
    }
}
