//! `Recipe`: los comandos que reconstruyen el lienzo desde uno en blanco.
//!
//! Es el `recipe.txt` de pxcli: una línea del protocolo por comando aplicado.
//! El `History` la mantiene al día (aplicar añade, deshacer quita, `new` la
//! vacía). Los `Batch` se guardan aplanados, comando a comando, para que la
//! receta y su texto sean intercambiables.

use std::fmt;

use crate::batch::normalize_batch_line;
use crate::canvas::Canvas;
use crate::command::Command;
use crate::error::DomainError;
use crate::protocol::{Request, parse_request};

/// Lista de comandos de dibujo, en el orden en que se aplicaron, y el tamaño
/// del lienzo en blanco del que parten (si se conoce).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Recipe {
    start: Option<(u32, u32)>,
    commands: Vec<Command>,
}

impl Recipe {
    pub fn new() -> Self {
        Self::default()
    }

    /// Receta vacía que parte de un lienzo en blanco de `width` × `height`.
    pub fn starting_at(width: u32, height: u32) -> Self {
        Self {
            start: Some((width, height)),
            commands: Vec::new(),
        }
    }

    /// Tamaño del lienzo en blanco del que parte la receta.
    pub fn start(&self) -> Option<(u32, u32)> {
        self.start
    }

    pub fn set_start(&mut self, start: Option<(u32, u32)>) {
        self.start = start;
    }

    /// Reconstruye el lienzo desde el tamaño de partida de la receta.
    pub fn rebuild(&self) -> Result<Canvas, DomainError> {
        let (width, height) = self
            .start
            .ok_or_else(|| DomainError::invalid_args("recipe has no start size"))?;
        self.replay(width, height)
    }

    /// Receta con estos comandos (los `Batch` se aplanan).
    pub fn from_commands(commands: impl IntoIterator<Item = Command>) -> Self {
        let mut recipe = Self::new();
        for command in commands {
            recipe.push(command);
        }
        recipe
    }

    /// Comandos simples (nunca contiene `Batch`).
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Añade un comando al final (un `Batch` se añade comando a comando).
    pub fn push(&mut self, command: Command) {
        match command {
            Command::Batch(commands) => commands.into_iter().for_each(|c| self.push(c)),
            command => self.commands.push(command),
        }
    }

    /// Quita los últimos `count` comandos simples.
    pub fn pop_last(&mut self, count: usize) {
        let keep = self.commands.len().saturating_sub(count);
        self.commands.truncate(keep);
    }

    /// Quita todos los comandos (conserva el tamaño de partida).
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Aplica la receta sobre un lienzo.
    pub fn replay_onto(&self, canvas: &mut Canvas) -> Result<(), DomainError> {
        for command in &self.commands {
            command.apply(canvas)?;
        }
        Ok(())
    }

    /// Reconstruye el lienzo partiendo de uno en blanco de `width` × `height`
    /// (el tamaño que tenía al empezar la receta).
    pub fn replay(&self, width: u32, height: u32) -> Result<Canvas, DomainError> {
        let mut canvas = Canvas::new(i64::from(width), i64::from(height))?;
        self.replay_onto(&mut canvas)?;
        Ok(canvas)
    }

    /// Lee una receta en el protocolo de texto. Acepta lo mismo que una línea
    /// de `batch` (comentarios, `;`, `pxcli`/`lumeart` delante, comillas),
    /// pero solo comandos de dibujo. Los errores llevan el número de línea.
    pub fn parse(text: &str) -> Result<Recipe, DomainError> {
        let mut recipe = Recipe::new();
        for line in crate::batch::split_batch(text) {
            let at_line = |err: DomainError| {
                DomainError::new(err.code, format!("line {}: {}", line.num, err.message))
            };
            let normalized =
                normalize_batch_line(&line.text, &|p: &str| p.to_string()).map_err(at_line)?;
            match parse_request(&normalized).map_err(at_line)? {
                Request::Apply(command) => recipe.push(command),
                other => {
                    let name = other.to_string();
                    let name = name.split(' ').next().unwrap_or_default().to_string();
                    return Err(at_line(DomainError::invalid_command(format!(
                        "{name} is not a drawing command"
                    ))));
                }
            }
        }
        Ok(recipe)
    }
}

/// Texto de la receta: una línea del protocolo por comando, cada una terminada
/// en salto de línea (como el `recipe.txt` de pxcli). El tamaño de partida no
/// va en el texto (en un `.lume` va en `manifest.json`).
impl fmt::Display for Recipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for command in &self.commands {
            writeln!(f, "{command}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgba;
    use crate::error::ErrorCode;
    use crate::history::{History, Origin};
    use proptest::prelude::*;

    const RED: Rgba = Rgba::rgb(255, 0, 0);

    #[test]
    fn batch_is_flattened() {
        let r = Recipe::from_commands([
            Command::Clear { color: None },
            Command::Batch(vec![
                Command::SetPixel {
                    x: 0,
                    y: 0,
                    color: RED,
                },
                Command::Batch(vec![Command::Resize { w: 4, h: 4 }]),
            ]),
        ]);
        assert_eq!(r.len(), 3);
        assert_eq!(
            r.to_string(),
            "clear\nset_pixel 0 0 #ff0000ff\nresize 4 4\n"
        );
    }

    #[test]
    fn parse_accepts_pxcli_recipes() {
        let text = "# receta\npxcli fill_rect 9 5 14 5 #cfdde3aa\n\nline 10 10 21 10 \"#cfdde3aa\"; set_pixel 1 1 red\n// fin\nclear\n";
        let r = Recipe::parse(text).unwrap();
        assert_eq!(
            r.commands(),
            &[
                Command::FillRect {
                    x: 9,
                    y: 5,
                    w: 14,
                    h: 5,
                    color: Rgba::new(0xcf, 0xdd, 0xe3, 0xaa)
                },
                Command::Line {
                    from: (10, 10),
                    to: (21, 10),
                    color: Rgba::new(0xcf, 0xdd, 0xe3, 0xaa)
                },
                Command::SetPixel {
                    x: 1,
                    y: 1,
                    color: RED
                },
                Command::Clear { color: None },
            ]
        );
    }

    #[test]
    fn parse_errors_carry_the_line_number() {
        let err = Recipe::parse("set_pixel 0 0 red\n\nset_pixel x 0 red").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidArgs);
        assert_eq!(err.message, "line 3: x must be an integer");
        let err = Recipe::parse("set_pixel 0 0 red\nundo").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidCommand);
        assert_eq!(err.message, "line 2: undo is not a drawing command");
        let err = Recipe::parse("get_pixel 0 0").unwrap_err();
        assert_eq!(err.message, "line 1: get_pixel is not a drawing command");
        let err = Recipe::parse("stop").unwrap_err();
        assert_eq!(err.message, "line 1: stop is not allowed inside batch");
        assert!(Recipe::parse("").unwrap().is_empty());
    }

    #[test]
    fn replay_rebuilds_the_canvas() {
        let mut c = Canvas::new(8, 8).unwrap();
        let mut h = History::new();
        for line in [
            "fill_rect 0 0 8 8 #102030",
            "line 0 0 7 7 red",
            "resize 10 6",
            "set_pixel 9 5 blue",
        ] {
            let crate::protocol::Request::Apply(cmd) = parse_request(line).unwrap() else {
                unreachable!()
            };
            h.apply(&mut c, cmd, Origin::Cli).unwrap();
        }
        assert_eq!(h.recipe().replay(8, 8).unwrap(), c);
    }

    #[test]
    fn undo_removes_commands_from_the_recipe() {
        let mut c = Canvas::new(4, 4).unwrap();
        let mut h = History::new();
        h.apply(
            &mut c,
            Command::SetPixel {
                x: 0,
                y: 0,
                color: RED,
            },
            Origin::Cli,
        )
        .unwrap();
        h.begin(Origin::Agent { run: 1 }, "batch").unwrap();
        h.apply(
            &mut c,
            Command::SetPixel {
                x: 1,
                y: 0,
                color: RED,
            },
            Origin::Agent { run: 1 },
        )
        .unwrap();
        h.apply(
            &mut c,
            Command::SetPixel {
                x: 2,
                y: 0,
                color: RED,
            },
            Origin::Agent { run: 1 },
        )
        .unwrap();
        h.commit();
        assert_eq!(h.recipe().len(), 3);

        h.undo(&mut c).unwrap();
        assert_eq!(h.recipe().to_string(), "set_pixel 0 0 #ff0000ff\n");
        h.redo(&mut c).unwrap();
        assert_eq!(h.recipe().len(), 3);
        h.undo(&mut c).unwrap();
        h.undo(&mut c).unwrap();
        assert!(h.recipe().is_empty());
        h.clear();
        assert!(h.recipe().is_empty());
    }

    // --- Propiedades ---

    fn color() -> impl Strategy<Value = Rgba> {
        any::<(u8, u8, u8, u8)>().prop_map(|(r, g, b, a)| Rgba::new(r, g, b, a))
    }

    fn command() -> impl Strategy<Value = Command> {
        let c = -1i64..12;
        prop_oneof![
            (c.clone(), c.clone(), color()).prop_map(|(x, y, color)| Command::SetPixel {
                x,
                y,
                color
            }),
            (c.clone(), c.clone(), 0i64..8, 0i64..8, color())
                .prop_map(|(x, y, w, h, color)| Command::FillRect { x, y, w, h, color }),
            (c.clone(), c.clone(), c.clone(), c, color()).prop_map(|(a, b, x, y, color)| {
                Command::Line {
                    from: (a, b),
                    to: (x, y),
                    color,
                }
            }),
            proptest::option::of(color()).prop_map(|color| Command::Clear { color }),
            (1i64..12, 1i64..12).prop_map(|(w, h)| Command::Resize { w, h }),
        ]
    }

    #[derive(Debug, Clone)]
    enum Step {
        Apply(Command),
        Batch(Vec<Command>),
        Undo,
        Redo,
    }

    fn step() -> impl Strategy<Value = Step> {
        prop_oneof![
            6 => command().prop_map(Step::Apply),
            2 => proptest::collection::vec(command(), 0..5).prop_map(Step::Batch),
            2 => Just(Step::Undo),
            1 => Just(Step::Redo),
        ]
    }

    proptest! {
            /// Con aplicar, transacciones, deshacer y rehacer al azar, la receta
            /// siempre reconstruye el lienzo actual.
            #[test]
            fn recipe_always_rebuilds_the_canvas(
                (w, h) in (1u32..10, 1u32..10),
                steps in proptest::collection::vec(step(), 0..25),
            ) {
                let mut c = Canvas::new(i64::from(w), i64::from(h)).unwrap();
                let mut hist = History::new();
                for step in steps {
                    match step {
                        Step::Apply(cmd) => { let _ = hist.apply(&mut c, cmd, Origin::Cli); }
                        Step::Batch(cmds) => {
                            hist.begin(Origin::Cli, "batch").unwrap();
                            for cmd in cmds { let _ = hist.apply(&mut c, cmd, Origin::Cli); }
                            hist.commit();
                        }
                        Step::Undo => { let _ = hist.undo(&mut c); }
                        Step::Redo => { let _ = hist.redo(&mut c); }
                    }
                    prop_assert_eq!(&hist.recipe().replay(w, h).unwrap(), &c);
                }
            }

            #[test]
            fn text_round_trip(commands in proptest::collection::vec(command(), 0..20)) {
                let recipe = Recipe::from_commands(commands);
                prop_assert_eq!(Recipe::parse(&recipe.to_string()).unwrap(), recipe);
            }

            /// Guiones de `batch` al azar (con `undo`, `redo` y `new` dentro):
            /// la receta reconstruye el lienzo desde el último `new`.
            #[test]
            fn recipe_follows_batches(
                scripts in proptest::collection::vec(
                    proptest::collection::vec(
                        prop_oneof![
                            6 => command().prop_map(|c| c.to_string()),
                            2 => Just("undo".to_string()),
                            1 => Just("redo".to_string()),
                            1 => (1u32..8, 1u32..8).prop_map(|(w, h)| format!("new {w} {h}")),
                        ],
                        1..10,
                    ),
                    1..5,
                ),
            ) {
                let mut c = Canvas::new(6, 6).unwrap();
                let mut hist = History::new();
                let mut start = (6u32, 6u32);
                for lines in scripts {
                    for line in &lines {
                        if let Some(size) = line.strip_prefix("new ") {
                            let (w, h) = size.split_once(' ').unwrap();
                            start = (w.parse().unwrap(), h.parse().unwrap());
                        }
                    }
                    let script = lines.join("\n");
                    crate::batch::run_batch(&script, &mut c, &mut hist, Origin::Cli, &|p: &str| p.to_string(), &mut |_, _, _| Ok(()))
                        .unwrap();
                    prop_assert_eq!(&hist.recipe().replay(start.0, start.1).unwrap(), &c);
                    let _ = hist.undo(&mut c);
                    prop_assert_eq!(&hist.recipe().replay(start.0, start.1).unwrap(), &c);
                }
            }
    }
}
