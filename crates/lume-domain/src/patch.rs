//! `Patch`: lo que cambió al aplicar un comando, para deshacerlo y rehacerlo.
//!
//! Guarda solo las celdas cambiadas (color anterior y nuevo), nunca el lienzo
//! entero. Un cambio de tamaño se guarda como un paso aparte, con las celdas
//! que se recortaron, porque cambia las dimensiones del lienzo.

use std::collections::HashMap;

use crate::anim::animation::Animation;
use crate::canvas::{Canvas, CellChange, SizeChange};
use crate::error::DomainError;
use crate::vec::drawing::VectorDrawing;

/// Cambio de la animación del documento: cómo estaba y cómo queda.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnimChange {
    pub before: Option<Animation>,
    pub after: Option<Animation>,
}

/// Cambio del dibujo vectorial del documento: cómo estaba y cómo queda.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VecChange {
    pub before: Option<VectorDrawing>,
    pub after: Option<VectorDrawing>,
}

/// Un paso de un `Patch`, en el orden en que se aplicó.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatchStep {
    /// Celdas cambiadas, con su color anterior y el nuevo.
    Cells(Vec<CellChange>),
    /// Cambio de tamaño del lienzo.
    Resize(SizeChange),
    /// Cambio de la animación (no toca el lienzo: lo aplica el documento).
    Animation(Box<AnimChange>),
    /// Cambio del dibujo vectorial (tampoco toca el lienzo).
    Vector(Box<VecChange>),
}

/// Cambios de un comando (o de varios agrupados). Vacío si no cambió nada.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Patch {
    steps: Vec<PatchStep>,
}

impl Patch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Patch con un solo paso de celdas.
    pub fn from_cells(cells: Vec<CellChange>) -> Self {
        let mut patch = Self::new();
        patch.push_cells(cells);
        patch
    }

    /// Patch que convierte el lienzo `from` en `to` (tamaño y celdas): al
    /// aplicarlo con `reapply` sobre `from` queda `to`, y `revert` lo deshace.
    pub fn between(from: &Canvas, to: &Canvas) -> Patch {
        let mut work = from.clone();
        let mut patch = Patch::new();
        let resized = work
            .resize(i64::from(to.width()), i64::from(to.height()))
            .expect("el tamaño de un lienzo existente es válido");
        if let Some(change) = resized {
            patch.push_resize(change);
        }
        patch.push_cells(crate::canvas::diff(&work, to));
        patch
    }

    /// Patch con un cambio de tamaño.
    /// Patch con un cambio de la animación.
    pub fn from_animation(change: AnimChange) -> Self {
        Self {
            steps: vec![PatchStep::Animation(Box::new(change))],
        }
    }

    /// Patch con un cambio del dibujo vectorial.
    pub fn from_vector(change: VecChange) -> Self {
        Self {
            steps: vec![PatchStep::Vector(Box::new(change))],
        }
    }

    pub fn from_resize(change: SizeChange) -> Self {
        Self {
            steps: vec![PatchStep::Resize(change)],
        }
    }

    pub fn steps(&self) -> &[PatchStep] {
        &self.steps
    }

    /// ¿No cambió nada?
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Todas las celdas cambiadas, en orden (sin contar los cambios de tamaño).
    pub fn cells(&self) -> impl Iterator<Item = &CellChange> {
        self.steps.iter().flat_map(|step| match step {
            PatchStep::Cells(cells) => cells.as_slice(),
            PatchStep::Resize(_) | PatchStep::Animation(_) | PatchStep::Vector(_) => &[],
        })
    }

    /// Número de celdas cambiadas.
    pub fn cell_count(&self) -> usize {
        self.cells().count()
    }

    /// Añade celdas cambiadas al final (se juntan con el último paso de celdas).
    pub fn push_cells(&mut self, cells: Vec<CellChange>) {
        if cells.is_empty() {
            return;
        }
        match self.steps.last_mut() {
            Some(PatchStep::Cells(last)) => last.extend(cells),
            _ => self.steps.push(PatchStep::Cells(cells)),
        }
    }

    /// Añade un cambio de tamaño al final.
    pub fn push_resize(&mut self, change: SizeChange) {
        self.steps.push(PatchStep::Resize(change));
    }

    /// Añade otro patch aplicado después de este.
    pub fn extend(&mut self, other: Patch) {
        for step in other.steps {
            match step {
                PatchStep::Cells(cells) => self.push_cells(cells),
                PatchStep::Resize(change) => self.push_resize(change),
                PatchStep::Animation(change) => self.steps.push(PatchStep::Animation(change)),
                PatchStep::Vector(change) => self.steps.push(PatchStep::Vector(change)),
            }
        }
    }

    /// Deja una sola entrada por celda en cada paso de celdas (el color de
    /// antes del primer cambio y el de después del último) y quita las que
    /// acabaron con el mismo color.
    pub fn compact(&mut self) {
        for step in &mut self.steps {
            if let PatchStep::Cells(cells) = step {
                let mut merged: Vec<CellChange> = Vec::with_capacity(cells.len());
                let mut position: HashMap<(u32, u32), usize> = HashMap::new();
                for change in cells.drain(..) {
                    match position.get(&(change.x, change.y)) {
                        Some(&i) => merged[i].after = change.after,
                        None => {
                            position.insert((change.x, change.y), merged.len());
                            merged.push(change);
                        }
                    }
                }
                merged.retain(|c| c.before != c.after);
                *cells = merged;
            }
        }
        self.steps
            .retain(|step| !matches!(step, PatchStep::Cells(cells) if cells.is_empty()));
    }

    /// Deshace el patch: deja el lienzo como estaba antes de aplicarlo.
    ///
    /// El lienzo debe estar justo como quedó tras aplicarlo; si no encaja, no
    /// se toca y se devuelve `invalid_args`.
    pub fn revert(&self, canvas: &mut Canvas) -> Result<(), DomainError> {
        self.check(canvas, Direction::Back)?;
        for step in self.steps.iter().rev() {
            match step {
                PatchStep::Cells(cells) => {
                    for c in cells.iter().rev() {
                        canvas.put(c.x, c.y, c.before);
                    }
                }
                PatchStep::Resize(change) => {
                    canvas.set_size(change.before.0, change.before.1);
                    for &(x, y, color) in &change.lost {
                        canvas.put(x, y, color);
                    }
                }
                PatchStep::Animation(_) | PatchStep::Vector(_) => {}
            }
        }
        Ok(())
    }

    /// Rehace el patch sobre el lienzo tal como estaba antes de aplicarlo.
    pub fn reapply(&self, canvas: &mut Canvas) -> Result<(), DomainError> {
        self.check(canvas, Direction::Forward)?;
        for step in &self.steps {
            match step {
                PatchStep::Cells(cells) => {
                    for c in cells {
                        canvas.put(c.x, c.y, c.after);
                    }
                }
                PatchStep::Resize(change) => canvas.set_size(change.after.0, change.after.1),
                PatchStep::Animation(_) | PatchStep::Vector(_) => {}
            }
        }
        Ok(())
    }

    /// Comprueba, sin tocar el lienzo, que todos los pasos encajan con sus
    /// dimensiones, para no dejarlo a medias.
    fn check(&self, canvas: &Canvas, direction: Direction) -> Result<(), DomainError> {
        let mismatch = || DomainError::invalid_args("patch does not match canvas");
        let mut size = (canvas.width(), canvas.height());
        let inside = |size: (u32, u32), x: u32, y: u32| x < size.0 && y < size.1;
        let steps: Box<dyn Iterator<Item = &PatchStep>> = match direction {
            Direction::Back => Box::new(self.steps.iter().rev()),
            Direction::Forward => Box::new(self.steps.iter()),
        };
        for step in steps {
            match step {
                PatchStep::Cells(cells) => {
                    if !cells.iter().all(|c| inside(size, c.x, c.y)) {
                        return Err(mismatch());
                    }
                }
                PatchStep::Resize(change) => {
                    let (from, to) = match direction {
                        Direction::Back => (change.after, change.before),
                        Direction::Forward => (change.before, change.after),
                    };
                    if size != from {
                        return Err(mismatch());
                    }
                    size = to;
                }
                PatchStep::Animation(_) | PatchStep::Vector(_) => {}
            }
        }
        Ok(())
    }

    /// La animación antes del patch, si la cambia (la del primer paso).
    pub fn animation_before(&self) -> Option<&Option<Animation>> {
        self.steps.iter().find_map(|s| match s {
            PatchStep::Animation(change) => Some(&change.before),
            _ => None,
        })
    }

    /// El dibujo vectorial antes del patch, si lo cambia (el del primer paso).
    pub fn vector_before(&self) -> Option<&Option<VectorDrawing>> {
        self.steps.iter().find_map(|s| match s {
            PatchStep::Vector(change) => Some(&change.before),
            _ => None,
        })
    }

    /// El dibujo vectorial después del patch, si lo cambia (el del último
    /// paso).
    pub fn vector_after(&self) -> Option<&Option<VectorDrawing>> {
        self.steps.iter().rev().find_map(|s| match s {
            PatchStep::Vector(change) => Some(&change.after),
            _ => None,
        })
    }

    /// La animación después del patch, si la cambia (la del último paso).
    pub fn animation_after(&self) -> Option<&Option<Animation>> {
        self.steps.iter().rev().find_map(|s| match s {
            PatchStep::Animation(change) => Some(&change.after),
            _ => None,
        })
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Back,
    Forward,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgba;
    use crate::error::ErrorCode;

    const RED: Rgba = Rgba::rgb(255, 0, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    fn change(x: u32, y: u32, before: Rgba, after: Rgba) -> CellChange {
        CellChange {
            x,
            y,
            before,
            after,
        }
    }

    #[test]
    fn empty_patch() {
        let mut p = Patch::from_cells(vec![]);
        assert!(p.is_empty());
        p.compact();
        assert!(p.is_empty());
        let mut c = Canvas::new(2, 2).unwrap();
        p.revert(&mut c).unwrap();
        p.reapply(&mut c).unwrap();
        assert_eq!(c, Canvas::new(2, 2).unwrap());
    }

    #[test]
    fn compact_merges_repeated_cells() {
        let t = Rgba::TRANSPARENT;
        let mut p = Patch::from_cells(vec![
            change(0, 0, t, RED),
            change(1, 0, t, RED),
            change(0, 0, RED, BLUE),
            change(1, 0, RED, t),
        ]);
        p.compact();
        assert_eq!(p.steps(), &[PatchStep::Cells(vec![change(0, 0, t, BLUE)])]);
    }

    #[test]
    fn revert_rejects_a_mismatched_canvas() {
        let p = Patch::from_cells(vec![change(5, 5, Rgba::TRANSPARENT, RED)]);
        let mut c = Canvas::new(2, 2).unwrap();
        let err = p.revert(&mut c).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidArgs);
        assert_eq!(c, Canvas::new(2, 2).unwrap());
    }

    #[test]
    fn resize_step_round_trip() {
        let mut c = Canvas::new(4, 4).unwrap();
        c.set(3, 3, RED).unwrap();
        let original = c.clone();
        let p = Patch::from_resize(c.resize(2, 2).unwrap().unwrap());
        let resized = c.clone();
        p.revert(&mut c).unwrap();
        assert_eq!(c, original);
        p.reapply(&mut c).unwrap();
        assert_eq!(c, resized);
        // Revertir otra vez sobre un lienzo que ya no encaja falla sin tocarlo.
        let mut wrong = Canvas::new(3, 3).unwrap();
        assert!(p.revert(&mut wrong).is_err());
        assert_eq!(wrong, Canvas::new(3, 3).unwrap());
    }

    #[test]
    fn between_turns_one_canvas_into_another() {
        let mut a = Canvas::new(4, 3).unwrap();
        a.fill_rect(0, 0, 4, 3, BLUE).unwrap();
        let mut b = Canvas::new(2, 5).unwrap();
        b.set(1, 4, RED).unwrap();
        b.set(0, 0, BLUE).unwrap();
        let p = Patch::between(&a, &b);
        let mut c = a.clone();
        p.reapply(&mut c).unwrap();
        assert_eq!(c, b);
        p.revert(&mut c).unwrap();
        assert_eq!(c, a);
        assert!(Patch::between(&a, &a).is_empty());
    }
}
