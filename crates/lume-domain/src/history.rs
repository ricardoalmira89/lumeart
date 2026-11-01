//! Historial de deshacer/rehacer con transacciones y autoría.
//!
//! Base: `internal/history/history.go` de pxcli, pero guardando `Patch`es (solo
//! las celdas cambiadas) en lugar de copias del lienzo. Como en pxcli, cada
//! comando aplicado con éxito es un paso de deshacer aunque no cambie nada, y
//! un comando que falla no deja rastro ni vacía la pila de rehacer.

use std::fmt;

use crate::canvas::Canvas;
use crate::command::Command;
use crate::error::DomainError;
use crate::patch::Patch;
use crate::recipe::Recipe;

/// Identificador de una ejecución (turno) del agente.
pub type RunId = u64;

/// Quién hizo un cambio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    /// El usuario, con las herramientas de la ventana.
    User,
    /// El agente interno, en una ejecución concreta.
    Agent { run: RunId },
    /// Un cliente externo: CLI, API local o MCP.
    Cli,
}

/// Nombre corto en español, como aparece en el historial de la interfaz.
impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Origin::User => "tú",
            Origin::Agent { .. } => "agente",
            Origin::Cli => "cli",
        })
    }
}

/// Un paso del historial: lo que cambió, quién lo hizo y con qué comandos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub patch: Patch,
    pub origin: Origin,
    pub label: String,
    pub commands: Vec<Command>,
    /// Si el paso sustituye la receta entera (volver a una versión), la
    /// receta de antes y la de después. Si no, el paso añade sus `commands`.
    pub recipe_swap: Option<Box<RecipeSwap>>,
}

/// Receta antes y después de un paso que la sustituye.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeSwap {
    pub before: Recipe,
    pub after: Recipe,
}

impl HistoryEntry {
    /// Número de comandos simples del paso.
    pub fn command_count(&self) -> usize {
        self.commands.iter().map(Command::count).sum()
    }
}

/// Transacción abierta: varios comandos que serán un solo paso al cerrarla.
///
/// Mientras está abierta, cada comando es un subpaso: `undo` deshace solo el
/// último subpaso (como hacía pxcli en un `batch`, donde cada línea era un
/// paso aparte) y `redo` lo vuelve a meter en la transacción.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Transaction {
    origin: Origin,
    label: String,
    steps: Vec<HistoryEntry>,
}

/// Pila de deshacer/rehacer. Mantiene también la `Recipe`: aplicar añade
/// los comandos, deshacer los quita, rehacer los vuelve a poner y `clear`
/// (lo que hace `new`) la vacía.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct History {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    open: Option<Transaction>,
    recipe: Recipe,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    /// Historial vacío que parte de una receta ya hecha (un documento abierto
    /// desde archivo).
    pub fn with_recipe(recipe: Recipe) -> Self {
        Self {
            recipe,
            ..Self::default()
        }
    }

    /// Comandos que reconstruyen el lienzo actual.
    pub fn recipe(&self) -> &Recipe {
        &self.recipe
    }

    /// Aplica un comando y lo registra. Si hay una transacción abierta, se
    /// suma a ella. Devuelve el `Patch` del comando.
    pub fn apply(
        &mut self,
        canvas: &mut Canvas,
        command: Command,
        origin: Origin,
    ) -> Result<Patch, DomainError> {
        let patch = command.apply(canvas)?;
        self.redo.clear();
        self.recipe.push(command.clone());
        let entry = HistoryEntry {
            patch: patch.clone(),
            origin,
            label: command.name().to_string(),
            commands: vec![command],
            recipe_swap: None,
        };
        match &mut self.open {
            Some(tx) => tx.steps.push(entry),
            None => self.undo.push(entry),
        }
        Ok(patch)
    }

    /// Abre una transacción: los comandos que se apliquen hasta `commit`
    /// serán un solo paso de deshacer (un `batch`, un trazo del lápiz…).
    pub fn begin(&mut self, origin: Origin, label: impl Into<String>) -> Result<(), DomainError> {
        if self.open.is_some() {
            return Err(DomainError::invalid_command("transaction already open"));
        }
        self.open = Some(Transaction {
            origin,
            label: label.into(),
            steps: Vec::new(),
        });
        Ok(())
    }

    /// ¿Hay una transacción abierta?
    pub fn in_transaction(&self) -> bool {
        self.open.is_some()
    }

    /// Los pasos de la transacción abierta (vacío si no hay).
    pub fn open_steps(&self) -> &[HistoryEntry] {
        self.open
            .as_ref()
            .map(|tx| tx.steps.as_slice())
            .unwrap_or(&[])
    }

    /// El último paso de la transacción abierta.
    pub fn open_last(&self) -> Option<&HistoryEntry> {
        self.open_steps().last()
    }

    /// Cierra la transacción. Si le queda al menos un comando, todos juntos
    /// pasan a ser un paso del historial con el autor y la etiqueta de la
    /// transacción; si no, no deja nada. Devuelve el paso creado.
    pub fn commit(&mut self) -> Option<&HistoryEntry> {
        let tx = self.open.take()?;
        if tx.steps.is_empty() {
            return None;
        }
        let mut patch = Patch::new();
        let mut commands = Vec::new();
        for step in tx.steps {
            patch.extend(step.patch);
            commands.extend(step.commands);
        }
        patch.compact();
        self.undo.push(HistoryEntry {
            patch,
            origin: tx.origin,
            label: tx.label,
            commands,
            recipe_swap: None,
        });
        self.undo.last()
    }

    /// Descarta la transacción y deshace en el lienzo los comandos que le
    /// quedaban.
    pub fn rollback(&mut self, canvas: &mut Canvas) -> Result<(), DomainError> {
        if let Some(mut tx) = self.open.take() {
            while let Some(step) = tx.steps.pop() {
                if let Err(err) = step.patch.revert(canvas) {
                    tx.steps.push(step);
                    self.open = Some(tx);
                    return Err(err);
                }
                self.recipe.pop_last(step.command_count());
            }
        }
        Ok(())
    }

    /// Deshace el último paso (dentro de una transacción, su último comando).
    pub fn undo(&mut self, canvas: &mut Canvas) -> Result<&HistoryEntry, DomainError> {
        let in_tx = self.open.as_ref().is_some_and(|tx| !tx.steps.is_empty());
        let stack = match (&mut self.open, in_tx) {
            (Some(tx), true) => &mut tx.steps,
            _ => &mut self.undo,
        };
        let entry = stack
            .pop()
            .ok_or_else(|| DomainError::no_history("nothing to undo"))?;
        if let Err(err) = entry.patch.revert(canvas) {
            stack.push(entry);
            return Err(err);
        }
        match &entry.recipe_swap {
            Some(swap) => self.recipe = swap.before.clone(),
            None => self.recipe.pop_last(entry.command_count()),
        }
        self.redo.push(entry);
        Ok(self.redo.last().expect("se acaba de añadir"))
    }

    /// Rehace el último paso deshecho (dentro de una transacción, vuelve a
    /// formar parte de ella).
    pub fn redo(&mut self, canvas: &mut Canvas) -> Result<&HistoryEntry, DomainError> {
        let entry = self
            .redo
            .pop()
            .ok_or_else(|| DomainError::no_history("nothing to redo"))?;
        if let Err(err) = entry.patch.reapply(canvas) {
            self.redo.push(entry);
            return Err(err);
        }
        match &entry.recipe_swap {
            Some(swap) => self.recipe = swap.after.clone(),
            None => entry
                .commands
                .iter()
                .for_each(|c| self.recipe.push(c.clone())),
        }
        let stack = match &mut self.open {
            Some(tx) => &mut tx.steps,
            None => &mut self.undo,
        };
        stack.push(entry);
        Ok(stack.last().expect("se acaba de añadir"))
    }

    /// Deshace de una vez los últimos pasos seguidos de una ejecución del
    /// agente. Cierra antes la transacción abierta. Devuelve cuántos pasos
    /// deshizo.
    pub fn undo_run(&mut self, canvas: &mut Canvas, run: RunId) -> Result<usize, DomainError> {
        self.commit();
        let mut undone = 0;
        while self
            .undo
            .last()
            .is_some_and(|e| e.origin == Origin::Agent { run })
        {
            self.undo(canvas)?;
            undone += 1;
        }
        if undone == 0 {
            return Err(DomainError::no_history(format!(
                "nothing to undo for run {run}"
            )));
        }
        Ok(undone)
    }

    /// Borra todo el historial, la transacción abierta y los comandos de la
    /// receta.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.open = None;
        self.recipe.clear();
    }

    /// Lo que hace `new`: borra todo y la receta pasa a partir de un lienzo en
    /// blanco de `width` × `height`.
    pub fn reset(&mut self, width: u32, height: u32) {
        self.clear();
        self.recipe = Recipe::starting_at(width, height);
    }

    /// Registra un paso que sustituye el lienzo y la receta enteros (volver a
    /// una versión): aplica `patch` (que debe llevar el lienzo actual al
    /// nuevo, ver `Patch::between`) y pone `recipe`. Cierra antes la
    /// transacción abierta. Se deshace en un paso.
    pub fn replace(
        &mut self,
        canvas: &mut Canvas,
        patch: Patch,
        recipe: Recipe,
        origin: Origin,
        label: impl Into<String>,
    ) -> Result<&HistoryEntry, DomainError> {
        self.commit();
        patch.reapply(canvas)?;
        self.redo.clear();
        let before = std::mem::replace(&mut self.recipe, recipe.clone());
        self.undo.push(HistoryEntry {
            patch,
            origin,
            label: label.into(),
            commands: Vec::new(),
            recipe_swap: Some(Box::new(RecipeSwap {
                before,
                after: recipe,
            })),
        });
        Ok(self.undo.last().expect("se acaba de añadir"))
    }

    /// Registra un cambio de la animación (no toca el lienzo ni la receta).
    /// Si hay una transacción abierta, se suma a ella.
    pub fn record_animation(
        &mut self,
        change: crate::patch::AnimChange,
        origin: Origin,
        label: impl Into<String>,
    ) {
        self.record_patch(Patch::from_animation(change), origin, label);
    }

    /// Registra un cambio del dibujo vectorial (no toca el lienzo ni la
    /// receta). Si hay una transacción abierta, se suma a ella.
    pub fn record_vector(
        &mut self,
        change: crate::patch::VecChange,
        origin: Origin,
        label: impl Into<String>,
    ) {
        self.record_patch(Patch::from_vector(change), origin, label);
    }

    /// Registra un patch que no toca el lienzo.
    fn record_patch(&mut self, patch: Patch, origin: Origin, label: impl Into<String>) {
        self.redo.clear();
        let entry = HistoryEntry {
            patch,
            origin,
            label: label.into(),
            commands: Vec::new(),
            recipe_swap: None,
        };
        match &mut self.open {
            Some(tx) => tx.steps.push(entry),
            None => self.undo.push(entry),
        }
    }

    /// Pasos que se pueden deshacer, del más antiguo al más reciente (sin
    /// contar la transacción abierta).
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.undo
    }

    /// Pasos deshechos que se pueden rehacer (el último es el próximo).
    pub fn redo_entries(&self) -> &[HistoryEntry] {
        &self.redo
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty() || self.open.as_ref().is_some_and(|tx| !tx.steps.is_empty())
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgba;
    use crate::error::ErrorCode;

    const RED: Rgba = Rgba::rgb(255, 0, 0);
    const GREEN: Rgba = Rgba::rgb(0, 255, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    fn set(x: i64, y: i64, color: Rgba) -> Command {
        Command::SetPixel { x, y, color }
    }

    // Portado de TestHistoryUndoRedoRestoresState.
    #[test]
    fn undo_redo_restores_state() {
        let mut c = Canvas::new(2, 2).unwrap();
        let mut h = History::new();
        h.apply(&mut c, set(1, 1, RED), Origin::Cli).unwrap();
        assert_eq!(c.get(1, 1).unwrap(), RED);
        h.undo(&mut c).unwrap();
        assert_eq!(c.get(1, 1).unwrap(), Rgba::TRANSPARENT);
        h.redo(&mut c).unwrap();
        assert_eq!(c.get(1, 1).unwrap(), RED);
    }

    // Portado de TestHistoryNoHistoryErrors.
    #[test]
    fn no_history_errors() {
        let mut c = Canvas::new(1, 1).unwrap();
        let mut h = History::new();
        let err = h.undo(&mut c).unwrap_err();
        assert_eq!(err.code, ErrorCode::NoHistory);
        assert_eq!(err.message, "nothing to undo");
        let err = h.redo(&mut c).unwrap_err();
        assert_eq!(err.code, ErrorCode::NoHistory);
        assert_eq!(err.message, "nothing to redo");
    }

    // Portado de TestHistoryRedoClearedOnNewMutation.
    #[test]
    fn redo_cleared_on_new_mutation() {
        let mut c = Canvas::new(2, 1).unwrap();
        let mut h = History::new();
        h.apply(&mut c, set(0, 0, RED), Origin::Cli).unwrap();
        h.undo(&mut c).unwrap();
        assert!(h.can_redo());
        h.apply(&mut c, set(1, 0, GREEN), Origin::Cli).unwrap();
        assert_eq!(h.redo(&mut c).unwrap_err().code, ErrorCode::NoHistory);
        assert_eq!(c.get(1, 0).unwrap(), GREEN);
        assert_eq!(c.get(0, 0).unwrap(), Rgba::TRANSPARENT);
    }

    #[test]
    fn failed_command_leaves_no_trace() {
        let mut c = Canvas::new(2, 2).unwrap();
        let mut h = History::new();
        h.apply(&mut c, set(0, 0, RED), Origin::Cli).unwrap();
        h.undo(&mut c).unwrap();
        let err = h.apply(&mut c, set(5, 5, RED), Origin::Cli).unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfBounds);
        assert!(h.entries().is_empty());
        assert!(
            h.can_redo(),
            "un error no vacía la pila de rehacer (como pxcli)"
        );
    }

    /// Como pxcli: un comando sin cambios visibles también es un paso.
    #[test]
    fn no_op_command_is_still_a_step() {
        let mut c = Canvas::new(2, 2).unwrap();
        let mut h = History::new();
        h.apply(&mut c, set(0, 0, RED), Origin::Cli).unwrap();
        h.apply(&mut c, set(0, 0, RED), Origin::Cli).unwrap();
        assert_eq!(h.entries().len(), 2);
        h.undo(&mut c).unwrap();
        assert_eq!(c.get(0, 0).unwrap(), RED);
        h.undo(&mut c).unwrap();
        assert_eq!(c.get(0, 0).unwrap(), Rgba::TRANSPARENT);
    }

    #[test]
    fn transaction_is_one_undo_step() {
        let mut c = Canvas::new(4, 4).unwrap();
        let mut h = History::new();
        h.apply(&mut c, set(3, 3, BLUE), Origin::User).unwrap();
        let before = c.clone();

        h.begin(Origin::Agent { run: 7 }, "batch").unwrap();
        h.apply(&mut c, set(0, 0, RED), Origin::Agent { run: 7 })
            .unwrap();
        h.apply(
            &mut c,
            Command::FillRect {
                x: 1,
                y: 1,
                w: 2,
                h: 2,
                color: GREEN,
            },
            Origin::Agent { run: 7 },
        )
        .unwrap();
        assert!(
            h.apply(&mut c, set(9, 9, RED), Origin::Agent { run: 7 })
                .is_err()
        );
        h.apply(
            &mut c,
            Command::Line {
                from: (0, 3),
                to: (3, 0),
                color: RED,
            },
            Origin::Agent { run: 7 },
        )
        .unwrap();
        let entry = h.commit().unwrap();
        assert_eq!(entry.label, "batch");
        assert_eq!(entry.origin, Origin::Agent { run: 7 });
        assert_eq!(entry.command_count(), 3);
        assert_eq!(h.entries().len(), 2);

        let after = c.clone();
        h.undo(&mut c).unwrap();
        assert_eq!(c, before);
        h.redo(&mut c).unwrap();
        assert_eq!(c, after);
    }

    #[test]
    fn empty_transaction_leaves_nothing() {
        let mut c = Canvas::new(2, 2).unwrap();
        let mut h = History::new();
        h.begin(Origin::User, "trazo").unwrap();
        assert!(h.apply(&mut c, set(9, 9, RED), Origin::User).is_err());
        assert!(h.commit().is_none());
        assert!(h.entries().is_empty());
    }

    #[test]
    fn nested_begin_is_an_error() {
        let mut h = History::new();
        h.begin(Origin::User, "a").unwrap();
        assert_eq!(
            h.begin(Origin::User, "b").unwrap_err().code,
            ErrorCode::InvalidCommand
        );
    }

    #[test]
    fn rollback_reverts_the_open_transaction() {
        let mut c = Canvas::new(2, 2).unwrap();
        let mut h = History::new();
        h.begin(Origin::User, "trazo").unwrap();
        h.apply(&mut c, set(0, 0, RED), Origin::User).unwrap();
        h.apply(&mut c, set(1, 1, RED), Origin::User).unwrap();
        h.rollback(&mut c).unwrap();
        assert_eq!(c, Canvas::new(2, 2).unwrap());
        assert!(!h.in_transaction());
        assert!(h.entries().is_empty());
    }

    /// Como un `batch` de pxcli (cada línea era un paso): dentro de una
    /// transacción, `undo` deshace solo el último comando.
    #[test]
    fn undo_inside_a_transaction_undoes_the_last_command() {
        let mut c = Canvas::new(2, 2).unwrap();
        let mut h = History::new();
        h.apply(&mut c, set(1, 1, BLUE), Origin::Cli).unwrap();
        h.begin(Origin::Cli, "batch").unwrap();
        h.apply(&mut c, set(0, 0, RED), Origin::Cli).unwrap();
        h.apply(&mut c, set(1, 0, RED), Origin::Cli).unwrap();
        h.undo(&mut c).unwrap();
        assert_eq!(c.get(0, 0).unwrap(), RED);
        assert_eq!(c.get(1, 0).unwrap(), Rgba::TRANSPARENT);
        h.redo(&mut c).unwrap();
        assert_eq!(c.get(1, 0).unwrap(), RED);
        h.undo(&mut c).unwrap();
        h.undo(&mut c).unwrap();
        // Sin comandos en la transacción, deshace pasos anteriores.
        h.undo(&mut c).unwrap();
        assert_eq!(c, Canvas::new(2, 2).unwrap());
        assert!(h.commit().is_none());
        assert_eq!(h.redo_entries().len(), 3);
    }

    #[test]
    fn redo_inside_a_transaction_joins_it() {
        let mut c = Canvas::new(2, 2).unwrap();
        let mut h = History::new();
        h.apply(&mut c, set(0, 0, RED), Origin::Cli).unwrap();
        h.undo(&mut c).unwrap();
        h.begin(Origin::Cli, "batch").unwrap();
        h.redo(&mut c).unwrap();
        h.apply(&mut c, set(1, 1, RED), Origin::Cli).unwrap();
        let entry = h.commit().unwrap();
        assert_eq!(entry.command_count(), 2);
        assert_eq!(h.entries().len(), 1);
        h.undo(&mut c).unwrap();
        assert_eq!(c, Canvas::new(2, 2).unwrap());
    }

    #[test]
    fn undo_a_whole_agent_run() {
        let mut c = Canvas::new(4, 4).unwrap();
        let mut h = History::new();
        h.apply(&mut c, set(0, 0, BLUE), Origin::User).unwrap();
        let before_run = c.clone();
        for i in 0..4 {
            h.apply(&mut c, set(i, 1, RED), Origin::Agent { run: 2 })
                .unwrap();
        }
        h.begin(Origin::Agent { run: 2 }, "batch").unwrap();
        h.apply(&mut c, set(3, 3, GREEN), Origin::Agent { run: 2 })
            .unwrap();
        h.commit();

        assert_eq!(h.undo_run(&mut c, 2).unwrap(), 5);
        assert_eq!(c, before_run);
        assert_eq!(h.entries().len(), 1);
        assert_eq!(h.entries()[0].origin, Origin::User);

        let err = h.undo_run(&mut c, 2).unwrap_err();
        assert_eq!(err.code, ErrorCode::NoHistory);
        assert_eq!(
            h.undo_run(&mut c, 3).unwrap_err().code,
            ErrorCode::NoHistory
        );
    }

    #[test]
    fn origin_and_labels() {
        let mut c = Canvas::new(2, 2).unwrap();
        let mut h = History::new();
        h.apply(&mut c, set(0, 0, RED), Origin::User).unwrap();
        h.apply(&mut c, Command::Clear { color: None }, Origin::Cli)
            .unwrap();
        let labels: Vec<_> = h
            .entries()
            .iter()
            .map(|e| (e.label.as_str(), e.origin.to_string()))
            .collect();
        assert_eq!(
            labels,
            vec![
                ("set_pixel", "tú".to_string()),
                ("clear", "cli".to_string())
            ]
        );
        assert_eq!(Origin::Agent { run: 1 }.to_string(), "agente");
    }

    #[test]
    fn clear_drops_everything() {
        let mut c = Canvas::new(2, 2).unwrap();
        let mut h = History::new();
        h.apply(&mut c, set(0, 0, RED), Origin::Cli).unwrap();
        h.apply(&mut c, set(1, 0, RED), Origin::Cli).unwrap();
        h.undo(&mut c).unwrap();
        h.begin(Origin::User, "x").unwrap();
        h.clear();
        assert!(!h.can_undo() && !h.can_redo() && !h.in_transaction());
    }
}
