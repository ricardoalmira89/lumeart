//! La animación de un documento: su `anim.js`, su base (si no es el propio
//! lienzo) y la duración de cada fotograma. Se cambia con [`AnimEdit`], que
//! es un paso de deshacer como cualquier otro.

use std::collections::BTreeMap;

use super::{ActionSpec, Image};
use crate::error::DomainError;

/// Duración más corta y más larga de un fotograma, en milisegundos.
pub const MIN_DURATION_MS: u32 = 10;
pub const MAX_DURATION_MS: u32 = 10_000;

/// La animación de un documento.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Animation {
    /// El código (`anim.js`).
    pub source: String,
    /// La base, si no es el lienzo del documento (al importar de pxcli, su
    /// `base.png` tal como la cargaba pxcli).
    pub base: Option<Image>,
    /// Milisegundos de cada fotograma, por acción. Lo que no esté aquí dura
    /// lo que marcan sus fps.
    pub durations: BTreeMap<String, Vec<u32>>,
}

/// Lo que dura un fotograma a `fps` fotogramas por segundo.
pub fn default_duration(fps: u32) -> u32 {
    (1000.0 / f64::from(fps.max(1))).round() as u32
}

impl Animation {
    pub fn new(source: impl Into<String>) -> Animation {
        Animation {
            source: source.into(),
            ..Animation::default()
        }
    }

    /// La duración de cada fotograma de una acción (las que no se cambiaron,
    /// las de sus fps).
    pub fn durations_for(&self, spec: &ActionSpec) -> Vec<u32> {
        let default = default_duration(spec.fps);
        let stored = self.durations.get(&spec.name);
        (0..spec.frames as usize)
            .map(|f| stored.and_then(|d| d.get(f)).copied().unwrap_or(default))
            .collect()
    }
}

/// Un cambio de la animación del documento.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnimEdit {
    /// Pone (o quita) la animación entera.
    Set(Option<Animation>),
    /// Cambia el código y deja lo demás.
    Source(String),
    /// Cambia lo que dura un fotograma de una acción.
    FrameDuration {
        action: ActionSpec,
        frame: u32,
        ms: u32,
    },
}

impl AnimEdit {
    /// Etiqueta para el historial.
    pub fn label(&self) -> String {
        match self {
            AnimEdit::Set(Some(_)) => "animación".into(),
            AnimEdit::Set(None) => "quitar animación".into(),
            AnimEdit::Source(_) => "anim.js".into(),
            AnimEdit::FrameDuration { action, frame, ms } => {
                format!("duración · {} f{} · {ms} ms", action.name, frame + 1)
            }
        }
    }

    /// La animación tras el cambio.
    pub fn apply(&self, current: &Option<Animation>) -> Result<Option<Animation>, DomainError> {
        match self {
            AnimEdit::Set(animation) => Ok(animation.clone()),
            AnimEdit::Source(source) => {
                let mut animation = current.clone().unwrap_or_default();
                animation.source = source.clone();
                Ok(Some(animation))
            }
            AnimEdit::FrameDuration { action, frame, ms } => {
                let Some(current) = current else {
                    return Err(DomainError::invalid_args("el documento no tiene animación"));
                };
                if *frame >= action.frames {
                    return Err(DomainError::invalid_args(format!(
                        "{} tiene {} fotogramas",
                        action.name, action.frames
                    )));
                }
                if !(MIN_DURATION_MS..=MAX_DURATION_MS).contains(ms) {
                    return Err(DomainError::invalid_args(format!(
                        "la duración va de {MIN_DURATION_MS} a {MAX_DURATION_MS} ms"
                    )));
                }
                let mut next = current.clone();
                let mut durations = next.durations_for(action);
                durations[*frame as usize] = *ms;
                next.durations.insert(action.name.clone(), durations);
                Ok(Some(next))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn andar() -> ActionSpec {
        ActionSpec {
            name: "andar".into(),
            frames: 3,
            fps: 10,
            looping: true,
        }
    }

    #[test]
    fn durations_default_to_the_fps() {
        let anim = Animation::new("x");
        assert_eq!(anim.durations_for(&andar()), [100, 100, 100]);
        assert_eq!(default_duration(8), 125);
        assert_eq!(default_duration(3), 333);
    }

    #[test]
    fn a_frame_duration_changes_only_that_frame() {
        let before = Some(Animation::new("x"));
        let edit = AnimEdit::FrameDuration {
            action: andar(),
            frame: 1,
            ms: 150,
        };
        let after = edit.apply(&before).unwrap().unwrap();
        assert_eq!(after.durations_for(&andar()), [100, 150, 100]);
        assert_eq!(edit.label(), "duración · andar f2 · 150 ms");
        assert!(
            AnimEdit::FrameDuration {
                action: andar(),
                frame: 3,
                ms: 100
            }
            .apply(&before)
            .is_err()
        );
        assert!(
            AnimEdit::FrameDuration {
                action: andar(),
                frame: 0,
                ms: 5
            }
            .apply(&before)
            .is_err()
        );
        assert!(edit.apply(&None).is_err());
        // Si la acción tiene ahora más fotogramas, los nuevos duran lo de sus fps.
        let longer = ActionSpec {
            frames: 4,
            ..andar()
        };
        assert_eq!(after.durations_for(&longer), [100, 150, 100, 100]);
    }

    /// En el documento, cambiar una duración es un paso de deshacer.
    #[test]
    fn duration_changes_undo_and_redo_in_the_document() {
        use crate::color::Rgba;
        use crate::command::Command;
        use crate::document::Document;
        use crate::history::Origin;

        let mut doc = Document::new("seta", 4, 4).unwrap();
        doc.apply(
            Command::SetPixel {
                x: 0,
                y: 0,
                color: Rgba::rgb(255, 0, 0),
            },
            Origin::User,
        )
        .unwrap();
        assert!(
            doc.edit_animation(
                AnimEdit::Set(Some(Animation::new("action('andar', {}, () => {});"))),
                Origin::User
            )
            .unwrap()
        );
        let edit = |ms| AnimEdit::FrameDuration {
            action: andar(),
            frame: 2,
            ms,
        };
        assert!(doc.edit_animation(edit(250), Origin::User).unwrap());
        assert!(
            !doc.edit_animation(edit(250), Origin::User).unwrap(),
            "sin cambio no hay paso"
        );
        let durations = |doc: &Document| doc.animation.as_ref().map(|a| a.durations_for(&andar()));
        assert_eq!(durations(&doc), Some(vec![100, 100, 250]));
        assert_eq!(
            doc.history().entries().last().unwrap().label,
            "duración · andar f3 · 250 ms"
        );

        doc.undo().unwrap();
        assert_eq!(durations(&doc), Some(vec![100, 100, 100]));
        doc.undo().unwrap();
        assert_eq!(doc.animation, None);
        assert_eq!(
            doc.canvas().get(0, 0).unwrap(),
            Rgba::rgb(255, 0, 0),
            "el lienzo no se toca"
        );
        doc.redo().unwrap();
        doc.redo().unwrap();
        assert_eq!(durations(&doc), Some(vec![100, 100, 250]));

        // Deshacer una ejecución del agente también devuelve la animación.
        let run = Origin::Agent { run: 7 };
        doc.edit_animation(edit(400), run).unwrap();
        doc.apply(
            Command::SetPixel {
                x: 1,
                y: 1,
                color: Rgba::rgb(0, 0, 255),
            },
            run,
        )
        .unwrap();
        assert_eq!(doc.undo_run(7).unwrap(), 2);
        assert_eq!(durations(&doc), Some(vec![100, 100, 250]));

        // Y descartar una transacción, también.
        doc.begin(Origin::User, "prueba").unwrap();
        doc.edit_animation(edit(600), Origin::User).unwrap();
        doc.rollback().unwrap();
        assert_eq!(durations(&doc), Some(vec![100, 100, 250]));
    }
}
