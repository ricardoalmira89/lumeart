//! `Document`: la raíz de lo que se edita. Une lienzo, historial (con su
//! receta), versiones, paleta y notas. Es puro: guardar y abrir archivos es
//! cosa de `lume-store`.

use crate::anim::animation::{AnimEdit, Animation};
use crate::batch::{BatchReport, ExportFn, run_batch};
use crate::canvas::{Canvas, CellChange, diff};
use crate::command::Command;
use crate::error::DomainError;
use crate::history::{History, HistoryEntry, Origin};
use crate::palette::Palette;
use crate::patch::{AnimChange, Patch, VecChange};
use crate::protocol::{Executed, Request, Response, execute, parse_request};
use crate::recipe::Recipe;
use crate::vec::drawing::{VecEdit, VectorDrawing};

/// Instantánea numerada del documento: «Versión N», con autor y mensaje.
/// Las vueltas del bucle de refinado también son versiones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub number: u32,
    pub author: Origin,
    pub message: String,
    pub canvas: Canvas,
    pub recipe: Recipe,
    /// El dibujo vectorial en esa versión, si el documento tenía.
    pub vector: Option<VectorDrawing>,
}

/// Un documento abierto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Nombre del documento (normalmente el del archivo, sin extensión).
    pub name: String,
    /// Paleta elegida (fija o propia, con nombres). Sin ella, la paleta es la
    /// del lienzo.
    pub palette: Option<Palette>,
    /// Notas del agente entre vueltas del bucle (`notas.md`).
    pub notes: String,
    /// La animación (`anim.js`, base y duraciones), si la tiene. Se cambia
    /// con [`Document::edit_animation`] para poder deshacerlo.
    pub animation: Option<Animation>,
    /// El dibujo vectorial (`dibujo.js`), si lo tiene. Se cambia con
    /// [`Document::edit_vector`] para poder deshacerlo.
    pub vector: Option<VectorDrawing>,
    canvas: Canvas,
    history: History,
    versions: Vec<Version>,
}

impl Document {
    /// Documento nuevo con un lienzo transparente.
    pub fn new(name: impl Into<String>, width: i64, height: i64) -> Result<Self, DomainError> {
        let canvas = Canvas::new(width, height)?;
        let history = History::with_recipe(Recipe::starting_at(canvas.width(), canvas.height()));
        Ok(Self {
            name: name.into(),
            palette: None,
            notes: String::new(),
            animation: None,
            vector: None,
            canvas,
            history,
            versions: Vec::new(),
        })
    }

    /// Documento a partir de lo guardado (lienzo, receta y versiones), con el
    /// historial vacío.
    pub fn from_parts(
        name: impl Into<String>,
        canvas: Canvas,
        recipe: Recipe,
        versions: Vec<Version>,
    ) -> Self {
        Self {
            name: name.into(),
            palette: None,
            notes: String::new(),
            animation: None,
            vector: None,
            canvas,
            history: History::with_recipe(recipe),
            versions,
        }
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    /// Comandos que reconstruyen el lienzo actual.
    pub fn recipe(&self) -> &Recipe {
        self.history.recipe()
    }

    /// Paleta elegida o, si no hay, la del lienzo.
    pub fn palette(&self) -> Palette {
        self.palette
            .clone()
            .unwrap_or_else(|| Palette::from_canvas(&self.canvas))
    }

    // --- Edición ---

    /// Aplica un comando y lo registra en el historial.
    pub fn apply(&mut self, command: Command, origin: Origin) -> Result<Patch, DomainError> {
        self.history.apply(&mut self.canvas, command, origin)
    }

    pub fn undo(&mut self) -> Result<&HistoryEntry, DomainError> {
        let patch = &self.history.undo(&mut self.canvas)?.patch;
        let (animation, vector) = (
            patch.animation_before().cloned(),
            patch.vector_before().cloned(),
        );
        if let Some(animation) = animation {
            self.animation = animation;
        }
        if let Some(vector) = vector {
            self.vector = vector;
        }
        Ok(self
            .history
            .redo_entries()
            .last()
            .expect("se acaba de deshacer"))
    }

    pub fn redo(&mut self) -> Result<&HistoryEntry, DomainError> {
        let patch = &self.history.redo(&mut self.canvas)?.patch;
        let (animation, vector) = (
            patch.animation_after().cloned(),
            patch.vector_after().cloned(),
        );
        if let Some(animation) = animation {
            self.animation = animation;
        }
        if let Some(vector) = vector {
            self.vector = vector;
        }
        let history = &self.history;
        Ok(match history.in_transaction() {
            true => history.open_last().expect("se acaba de rehacer"),
            false => history.entries().last().expect("se acaba de rehacer"),
        })
    }

    /// Deshace los últimos pasos seguidos de una ejecución del agente.
    pub fn undo_run(&mut self, run: crate::history::RunId) -> Result<usize, DomainError> {
        self.history.commit();
        let mut undone = 0;
        while self
            .history
            .entries()
            .last()
            .is_some_and(|e| e.origin == Origin::Agent { run })
        {
            self.undo()?;
            undone += 1;
        }
        if undone == 0 {
            return Err(DomainError::no_history(format!(
                "nothing to undo for run {run}"
            )));
        }
        Ok(undone)
    }

    /// Cambia la animación como un paso de deshacer. Si no cambia nada, no
    /// deja paso.
    pub fn edit_animation(&mut self, edit: AnimEdit, origin: Origin) -> Result<bool, DomainError> {
        let after = edit.apply(&self.animation)?;
        if after == self.animation {
            return Ok(false);
        }
        let before = std::mem::replace(&mut self.animation, after.clone());
        self.history
            .record_animation(AnimChange { before, after }, origin, edit.label());
        Ok(true)
    }

    /// Cambia el dibujo vectorial (un paso de deshacer). `Ok(false)` si no
    /// cambia nada.
    pub fn edit_vector(&mut self, edit: VecEdit, origin: Origin) -> Result<bool, DomainError> {
        let after = edit.apply(&self.vector)?;
        if after == self.vector {
            return Ok(false);
        }
        let before = std::mem::replace(&mut self.vector, after.clone());
        self.history
            .record_vector(VecChange { before, after }, origin, edit.label());
        Ok(true)
    }

    /// Abre una transacción (un trazo del lápiz, un turno del agente…).
    pub fn begin(&mut self, origin: Origin, label: impl Into<String>) -> Result<(), DomainError> {
        self.history.begin(origin, label)
    }

    pub fn commit(&mut self) -> Option<&HistoryEntry> {
        self.history.commit()
    }

    pub fn rollback(&mut self) -> Result<(), DomainError> {
        let steps = self.history.open_steps();
        let before = steps
            .iter()
            .find_map(|step| step.patch.animation_before().cloned());
        let vector = steps
            .iter()
            .find_map(|step| step.patch.vector_before().cloned());
        self.history.rollback(&mut self.canvas)?;
        if let Some(animation) = before {
            self.animation = animation;
        }
        if let Some(drawing) = vector {
            self.vector = drawing;
        }
        Ok(())
    }

    /// Ejecuta una petición del protocolo.
    pub fn execute(&mut self, request: Request, origin: Origin) -> Executed {
        // Deshacer y rehacer pasan por el documento: también devuelven la
        // animación.
        match request {
            Request::Undo => Executed::Reply(self.undo().map(|_| None).into()),
            Request::Redo => Executed::Reply(self.redo().map(|_| None).into()),
            other => execute(other, &mut self.canvas, &mut self.history, origin),
        }
    }

    /// Parsea y ejecuta una línea del protocolo.
    pub fn handle_line(&mut self, line: &str, origin: Origin) -> Executed {
        match parse_request(line) {
            Ok(request) => self.execute(request, origin),
            Err(err) => Executed::Reply(Response::Err(err)),
        }
    }

    /// Ejecuta un `batch` como un solo paso de deshacer.
    pub fn run_batch(
        &mut self,
        script: &str,
        origin: Origin,
        resolve_path: &dyn Fn(&str) -> String,
        export: &mut ExportFn<'_>,
    ) -> Result<BatchReport, DomainError> {
        run_batch(
            script,
            &mut self.canvas,
            &mut self.history,
            origin,
            resolve_path,
            export,
        )
    }

    // --- Versiones ---

    pub fn versions(&self) -> &[Version] {
        &self.versions
    }

    pub fn version(&self, number: u32) -> Result<&Version, DomainError> {
        self.versions
            .iter()
            .find(|v| v.number == number)
            .ok_or_else(|| DomainError::invalid_args(format!("version {number} not found")))
    }

    /// Guarda una versión con una copia del lienzo y de la receta. Los
    /// números empiezan en 1 y siempre crecen.
    pub fn create_version(&mut self, author: Origin, message: impl Into<String>) -> &Version {
        let number = self.versions.iter().map(|v| v.number).max().unwrap_or(0) + 1;
        self.versions.push(Version {
            number,
            author,
            message: message.into(),
            canvas: self.canvas.clone(),
            recipe: self.history.recipe().clone(),
            vector: self.vector.clone(),
        });
        self.versions.last().expect("se acaba de añadir")
    }

    /// Vuelve a una versión: restaura su lienzo y su receta como un paso de
    /// deshacer («volver a la versión N»). Devuelve el `Patch` aplicado.
    pub fn restore_version(&mut self, number: u32, origin: Origin) -> Result<Patch, DomainError> {
        let version = self.version(number)?;
        let mut patch = Patch::between(&self.canvas, &version.canvas);
        let recipe = version.recipe.clone();
        // El dibujo vectorial vuelve en el mismo paso de deshacer.
        let vector = version.vector.clone();
        if version.vector.is_some() && vector != self.vector {
            patch.extend(Patch::from_vector(VecChange {
                before: self.vector.clone(),
                after: vector.clone(),
            }));
        }
        let entry = self.history.replace(
            &mut self.canvas,
            patch,
            recipe,
            origin,
            format!("volver a la versión {number}"),
        )?;
        let patch = entry.patch.clone();
        if let Some(after) = patch.vector_after() {
            self.vector = after.clone();
        }
        Ok(patch)
    }

    /// Celdas distintas entre dos versiones (`before` = versión `a`, `after`
    /// = versión `b`).
    pub fn compare_versions(&self, a: u32, b: u32) -> Result<Vec<CellChange>, DomainError> {
        Ok(diff(&self.version(a)?.canvas, &self.version(b)?.canvas))
    }

    /// Celdas distintas entre una versión y el lienzo actual.
    pub fn compare_with_current(&self, number: u32) -> Result<Vec<CellChange>, DomainError> {
        Ok(diff(&self.version(number)?.canvas, &self.canvas))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgba;
    use crate::error::ErrorCode;
    use crate::protocol::Executed;

    const RED: Rgba = Rgba::rgb(255, 0, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    fn line(doc: &mut Document, text: &str) -> String {
        match doc.handle_line(text, Origin::Cli) {
            Executed::Reply(r) => r.to_string(),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn new_document() {
        let doc = Document::new("pocion", 32, 32).unwrap();
        assert_eq!(doc.name, "pocion");
        assert_eq!((doc.canvas().width(), doc.canvas().height()), (32, 32));
        assert_eq!(doc.recipe().start(), Some((32, 32)));
        assert!(doc.versions().is_empty());
        assert!(doc.palette().is_empty());
        assert_eq!(
            Document::new("x", 0, 4).unwrap_err().code,
            ErrorCode::InvalidArgs
        );
    }

    #[test]
    fn versions_copy_canvas_and_recipe_with_growing_numbers() {
        let mut doc = Document::new("d", 4, 4).unwrap();
        line(&mut doc, "set_pixel 0 0 red");
        let v1 = doc.create_version(Origin::User, "primera").clone();
        line(&mut doc, "fill_rect 1 1 2 2 blue");
        let v2 = doc
            .create_version(Origin::Agent { run: 3 }, "Vuelta 1")
            .clone();
        line(&mut doc, "clear");
        let v3 = doc.create_version(Origin::Cli, "").clone();

        assert_eq!((v1.number, v2.number, v3.number), (1, 2, 3));
        assert_eq!(v1.author, Origin::User);
        assert_eq!(v2.message, "Vuelta 1");
        assert_eq!(v1.canvas.get(0, 0).unwrap(), RED);
        assert_eq!(v1.canvas.get(1, 1).unwrap(), Rgba::TRANSPARENT);
        assert_eq!(v1.recipe.len(), 1);
        assert_eq!(v2.recipe.len(), 2);
        assert_eq!(v2.recipe.rebuild().unwrap(), v2.canvas);
        // La copia no cambia al seguir dibujando.
        assert_eq!(doc.version(1).unwrap().canvas.get(0, 0).unwrap(), RED);
        assert_eq!(doc.version(9).unwrap_err().message, "version 9 not found");
    }

    #[test]
    fn restore_is_one_undo_step() {
        let mut doc = Document::new("d", 4, 4).unwrap();
        line(&mut doc, "set_pixel 0 0 red");
        doc.create_version(Origin::User, "v1");
        line(&mut doc, "fill_rect 0 0 4 4 blue");
        line(&mut doc, "resize 6 3");
        line(&mut doc, "set_pixel 5 2 red");
        let before = doc.canvas().clone();
        let recipe_before = doc.recipe().clone();

        doc.restore_version(1, Origin::User).unwrap();
        assert_eq!(doc.canvas(), &doc.version(1).unwrap().canvas.clone());
        assert_eq!(doc.recipe(), &doc.version(1).unwrap().recipe.clone());
        assert_eq!(doc.recipe().rebuild().unwrap(), *doc.canvas());
        let entry = doc.history().entries().last().unwrap();
        assert_eq!(entry.label, "volver a la versión 1");
        assert_eq!(entry.origin, Origin::User);

        doc.undo().unwrap();
        assert_eq!(doc.canvas(), &before);
        assert_eq!(doc.recipe(), &recipe_before);
        assert_eq!(doc.recipe().rebuild().unwrap(), before);

        doc.redo().unwrap();
        assert_eq!(doc.canvas(), &doc.version(1).unwrap().canvas.clone());
        assert_eq!(doc.recipe().len(), 1);

        // Seguir dibujando después de volver añade a la receta restaurada.
        line(&mut doc, "set_pixel 3 3 blue");
        assert_eq!(doc.recipe().len(), 2);
        assert_eq!(doc.recipe().rebuild().unwrap(), *doc.canvas());
    }

    #[test]
    fn restore_across_new() {
        let mut doc = Document::new("d", 4, 4).unwrap();
        line(&mut doc, "set_pixel 1 1 red");
        doc.create_version(Origin::User, "pequeño");
        assert_eq!(line(&mut doc, "new 8 6"), "ok 8x6");
        assert_eq!(doc.recipe().start(), Some((8, 6)));
        line(&mut doc, "set_pixel 7 5 blue");
        doc.create_version(Origin::User, "grande");

        doc.restore_version(1, Origin::User).unwrap();
        assert_eq!((doc.canvas().width(), doc.canvas().height()), (4, 4));
        assert_eq!(doc.recipe().start(), Some((4, 4)));
        assert_eq!(doc.recipe().rebuild().unwrap(), *doc.canvas());

        doc.undo().unwrap();
        assert_eq!((doc.canvas().width(), doc.canvas().height()), (8, 6));
        assert_eq!(doc.canvas().get(7, 5).unwrap(), BLUE);
        assert_eq!(doc.recipe().rebuild().unwrap(), *doc.canvas());
    }

    #[test]
    fn restore_closes_an_open_transaction_and_clears_redo() {
        let mut doc = Document::new("d", 2, 2).unwrap();
        doc.create_version(Origin::User, "vacío");
        doc.apply(
            Command::SetPixel {
                x: 0,
                y: 0,
                color: RED,
            },
            Origin::User,
        )
        .unwrap();
        doc.undo().unwrap();
        doc.begin(Origin::User, "trazo").unwrap();
        doc.apply(
            Command::SetPixel {
                x: 1,
                y: 1,
                color: RED,
            },
            Origin::User,
        )
        .unwrap();
        doc.restore_version(1, Origin::User).unwrap();
        assert!(!doc.history().in_transaction());
        assert_eq!(doc.history().entries().len(), 2);
        assert!(!doc.history().can_redo());
        assert_eq!(doc.canvas(), &Canvas::new(2, 2).unwrap());
    }

    #[test]
    fn compare_versions_returns_different_cells() {
        let mut doc = Document::new("d", 3, 3).unwrap();
        line(&mut doc, "set_pixel 0 0 red");
        doc.create_version(Origin::User, "a");
        line(&mut doc, "set_pixel 0 0 blue");
        line(&mut doc, "set_pixel 2 2 red");
        doc.create_version(Origin::User, "b");

        let cells = doc.compare_versions(1, 2).unwrap();
        assert_eq!(
            cells,
            vec![
                CellChange {
                    x: 0,
                    y: 0,
                    before: RED,
                    after: BLUE
                },
                CellChange {
                    x: 2,
                    y: 2,
                    before: Rgba::TRANSPARENT,
                    after: RED
                },
            ]
        );
        assert!(doc.compare_versions(2, 2).unwrap().is_empty());
        assert!(doc.compare_with_current(2).unwrap().is_empty());
        assert_eq!(
            doc.compare_versions(1, 7).unwrap_err().code,
            ErrorCode::InvalidArgs
        );
    }

    #[test]
    fn compare_versions_of_different_sizes() {
        let mut doc = Document::new("d", 2, 2).unwrap();
        doc.create_version(Origin::User, "2x2");
        line(&mut doc, "resize 3 1");
        line(&mut doc, "set_pixel 2 0 red");
        doc.create_version(Origin::User, "3x1");
        let cells = doc.compare_versions(1, 2).unwrap();
        assert_eq!(
            cells,
            vec![CellChange {
                x: 2,
                y: 0,
                before: Rgba::TRANSPARENT,
                after: RED
            }]
        );
    }

    #[test]
    fn palette_is_fixed_or_from_the_canvas() {
        let mut doc = Document::new("d", 2, 2).unwrap();
        line(&mut doc, "set_pixel 1 1 red");
        assert_eq!(doc.palette().colors()[0].color, RED);
        doc.palette = Palette::fixed("game-boy");
        assert_eq!(doc.palette().len(), 4);
    }

    #[test]
    fn from_parts_keeps_recipe_and_versions() {
        let mut doc = Document::new("d", 2, 2).unwrap();
        line(&mut doc, "set_pixel 1 1 red");
        doc.create_version(Origin::User, "v1");
        let loaded = Document::from_parts(
            "d",
            doc.canvas().clone(),
            doc.recipe().clone(),
            doc.versions().to_vec(),
        );
        assert_eq!(loaded.recipe().len(), 1);
        assert_eq!(loaded.versions().len(), 1);
        assert!(!loaded.history().can_undo());
    }

    mod properties {
        use super::*;
        use proptest::prelude::*;

        #[derive(Debug, Clone)]
        enum Step {
            Line(String),
            Undo,
            Redo,
            Version,
            Restore(u32),
        }

        fn step() -> impl Strategy<Value = Step> {
            let c = -1i64..9;
            let color = prop_oneof![
                Just("red"),
                Just("blue"),
                Just("#00ff0080"),
                Just("transparent")
            ];
            prop_oneof![
                4 => (c.clone(), c.clone(), color.clone()).prop_map(|(x, y, col)| Step::Line(format!("set_pixel {x} {y} {col}"))),
                2 => (c.clone(), c.clone(), 0i64..6, 0i64..6, color.clone())
                    .prop_map(|(x, y, w, h, col)| Step::Line(format!("fill_rect {x} {y} {w} {h} {col}"))),
                1 => (1i64..9, 1i64..9).prop_map(|(w, h)| Step::Line(format!("resize {w} {h}"))),
                1 => (1i64..9, 1i64..9).prop_map(|(w, h)| Step::Line(format!("new {w} {h}"))),
                1 => (c, color).prop_map(|(x, col)| Step::Line(format!("set_pixel 0 0 {col}; line 0 0 {x} 0 {col}; undo"))),
                2 => Just(Step::Undo),
                1 => Just(Step::Redo),
                2 => Just(Step::Version),
                2 => (1u32..6).prop_map(Step::Restore),
            ]
        }

        proptest! {
            #[test]
            fn recipe_rebuilds_the_canvas_after_any_steps(steps in proptest::collection::vec(step(), 0..30)) {
                let mut doc = Document::new("p", 6, 6).unwrap();
                for step in steps {
                    match step {
                        Step::Line(text) if text.contains(';') => {
                            doc.run_batch(&text, Origin::Cli, &|p: &str| p.to_string(), &mut |_, _, _| Ok(())).unwrap();
                        }
                        Step::Line(text) => { let _ = doc.handle_line(&text, Origin::Cli); }
                        Step::Undo => { let _ = doc.undo(); }
                        Step::Redo => { let _ = doc.redo(); }
                        Step::Version => { doc.create_version(Origin::User, "v"); }
                        Step::Restore(n) => { let _ = doc.restore_version(n, Origin::User); }
                    }
                    prop_assert_eq!(&doc.recipe().rebuild().unwrap(), doc.canvas());
                    for v in doc.versions() {
                        prop_assert_eq!(&v.recipe.rebuild().unwrap(), &v.canvas);
                    }
                }
            }
        }
    }

    /// Cada cambio del dibujo vectorial se deshace y se rehace; los del
    /// agente se deshacen juntos; una versión guarda su dibujo.js y volver a
    /// ella es un solo paso.
    #[test]
    fn vector_edits_undo_redo_and_versions() {
        let mut doc = Document::new("faro", 4, 4).unwrap();
        let first = VectorDrawing::new("canvas({ width: 400, height: 300 })\n");
        assert!(
            doc.edit_vector(VecEdit::Source("x".into()), Origin::User)
                .is_err()
        );
        assert!(
            doc.edit_vector(VecEdit::Set(Some(first.clone())), Origin::User)
                .unwrap()
        );
        assert!(
            !doc.edit_vector(VecEdit::Set(Some(first.clone())), Origin::User)
                .unwrap()
        );
        doc.create_version(Origin::User, "base");
        let run = Origin::Agent { run: 7 };
        doc.edit_vector(VecEdit::Source("a".into()), run).unwrap();
        doc.edit_vector(VecEdit::Source("b".into()), run).unwrap();
        doc.create_version(run, "Retoque de la zona 1");
        assert_eq!(doc.versions()[1].vector, Some(VectorDrawing::new("b")));

        doc.undo().unwrap();
        assert_eq!(doc.vector, Some(VectorDrawing::new("a")));
        doc.redo().unwrap();
        assert_eq!(doc.vector, Some(VectorDrawing::new("b")));
        assert_eq!(doc.undo_run(7).unwrap(), 2);
        assert_eq!(doc.vector, Some(first.clone()));

        doc.restore_version(2, Origin::User).unwrap();
        assert_eq!(doc.vector, Some(VectorDrawing::new("b")));
        doc.undo().unwrap();
        assert_eq!(doc.vector, Some(first));
    }
}
