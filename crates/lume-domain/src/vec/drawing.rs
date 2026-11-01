//! El dibujo vectorial de un documento: su `dibujo.js`. Se cambia con
//! [`VecEdit`], que es un paso de deshacer como cualquier otro.

use crate::error::DomainError;

/// El dibujo vectorial de un documento.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct VectorDrawing {
    /// El código (`dibujo.js`).
    pub source: String,
}

impl VectorDrawing {
    pub fn new(source: impl Into<String>) -> VectorDrawing {
        VectorDrawing {
            source: source.into(),
        }
    }
}

/// Un cambio del dibujo vectorial del documento.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VecEdit {
    /// Pone (o quita) el dibujo entero.
    Set(Option<VectorDrawing>),
    /// Cambia el código.
    Source(String),
}

impl VecEdit {
    /// Etiqueta para el historial.
    pub fn label(&self) -> String {
        match self {
            VecEdit::Set(Some(_)) => "dibujo vectorial".into(),
            VecEdit::Set(None) => "quitar dibujo vectorial".into(),
            VecEdit::Source(_) => "dibujo.js".into(),
        }
    }

    /// El dibujo tras el cambio.
    pub fn apply(
        &self,
        current: &Option<VectorDrawing>,
    ) -> Result<Option<VectorDrawing>, DomainError> {
        match self {
            VecEdit::Set(drawing) => Ok(drawing.clone()),
            VecEdit::Source(source) => {
                if current.is_none() {
                    return Err(DomainError::invalid_args(
                        "el documento no tiene dibujo vectorial",
                    ));
                }
                Ok(Some(VectorDrawing::new(source.clone())))
            }
        }
    }
}

/// Las líneas que cambian de `before` a `after` (de 1 en adelante,
/// incluidas), o nada si son iguales: desde la primera distinta por arriba
/// hasta la última distinta por abajo. Es lo que se resalta en el panel de
/// código cuando el agente edita.
pub fn changed_lines(before: &str, after: &str) -> Option<(usize, usize)> {
    if before == after {
        return None;
    }
    let a: Vec<&str> = before.split('\n').collect();
    let b: Vec<&str> = after.split('\n').collect();
    let head = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let tail = a
        .iter()
        .rev()
        .zip(b.iter().rev())
        .take_while(|(x, y)| x == y)
        .count()
        .min(a.len().min(b.len()) - head);
    let last = b.len() - tail;
    if last <= head {
        // Solo se borraron líneas: se señala dónde estaban.
        return Some((head + 1, head + 1));
    }
    Some((head + 1, last))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_apply_and_name_themselves() {
        let d = Some(VectorDrawing::new("canvas({})"));
        assert_eq!(
            VecEdit::Source("x".into()).apply(&d).unwrap(),
            Some(VectorDrawing::new("x"))
        );
        assert!(VecEdit::Source("x".into()).apply(&None).is_err());
        assert_eq!(VecEdit::Set(None).apply(&d).unwrap(), None);
        assert_eq!(VecEdit::Source(String::new()).label(), "dibujo.js");
    }

    #[test]
    fn changed_lines_span_the_edit() {
        let a = "a\nb\nc\nd\ne";
        assert_eq!(changed_lines(a, a), None);
        assert_eq!(changed_lines(a, "a\nB\nc\nd\ne"), Some((2, 2)));
        assert_eq!(changed_lines(a, "a\nb\nX\nY\nZ\nd\ne"), Some((3, 5)));
        assert_eq!(changed_lines(a, "a\nb\ne"), Some((3, 3)));
        assert_eq!(changed_lines(a, "a\nb\nc\nd\ne\nf"), Some((6, 6)));
    }
}
