//! La receta de un dibujo: los comandos de pxcli que lo pintan
//! (`internal/anim/recipe.go`).

use crate::canvas::bresenham;
use crate::color::Rgba;
use crate::error::go_quote;

use super::Image;

/// Comando de dibujo de una receta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    SetPixel,
    FillRect,
    Line,
    Clear,
}

impl Op {
    pub fn as_str(self) -> &'static str {
        match self {
            Op::SetPixel => "set_pixel",
            Op::FillRect => "fill_rect",
            Op::Line => "line",
            Op::Clear => "clear",
        }
    }
}

/// Un comando de la receta.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cmd {
    pub op: Op,
    pub args: Vec<i64>,
    pub color: Rgba,
}

/// Lee líneas de comandos de pxcli. Las vacías, los comentarios (`#`, `//`)
/// y un `pxcli` delante se ignoran; lo que no sea de dibujo es un error.
pub fn parse_recipe<S: AsRef<str>>(lines: &[S]) -> Result<Vec<Cmd>, String> {
    let mut cmds = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let n = i + 1;
        let line = raw.as_ref().trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        let mut fields: Vec<&str> = line.split_whitespace().collect();
        if fields[0] == "pxcli" || fields[0].ends_with("/pxcli") {
            fields.remove(0);
        }
        if fields.is_empty() {
            continue;
        }
        let (op, args) = (fields[0], &fields[1..]);
        let (op, want) = match op {
            "set_pixel" => (Op::SetPixel, 2),
            "fill_rect" => (Op::FillRect, 4),
            "line" => (Op::Line, 4),
            "clear" => (Op::Clear, 0),
            other => return Err(format!("line {n}: {other} is not a drawing command")),
        };
        let parse_color = |text: &str| {
            Rgba::parse(text.trim_matches(|c| c == '"' || c == '\''))
                .map_err(|e| format!("line {n}: {e}"))
        };
        if op == Op::Clear {
            let color = match args.first() {
                Some(text) => parse_color(text)?,
                None => Rgba::TRANSPARENT,
            };
            cmds.push(Cmd {
                op,
                args: Vec::new(),
                color,
            });
            continue;
        }
        if args.len() != want + 1 {
            return Err(format!(
                "line {n}: {} needs {} arguments",
                op.as_str(),
                want + 1
            ));
        }
        let mut numbers = Vec::with_capacity(want);
        for a in &args[..want] {
            numbers.push(
                a.parse::<i64>()
                    .map_err(|_| format!("line {n}: {} is not an integer", go_quote(a)))?,
            );
        }
        cmds.push(Cmd {
            op,
            args: numbers,
            color: parse_color(args[want])?,
        });
    }
    Ok(cmds)
}

/// Pinta los comandos en `img` (lo que se sale, se recorta). Con `owner`,
/// apunta en cada píxel el índice del último comando que lo pintó.
pub fn paint(img: &mut Image, cmds: &[Cmd], mut owner: Option<&mut [i64]>) {
    let (w, h) = (i64::from(img.width()), i64::from(img.height()));
    let mut set = |img: &mut Image, i: usize, x: i64, y: i64, c: Rgba| {
        if x < 0 || y < 0 || x >= w || y >= h {
            return;
        }
        img.set(x, y, c);
        if let Some(owner) = owner.as_deref_mut() {
            owner[(y * w + x) as usize] = i as i64;
        }
    };
    for (i, c) in cmds.iter().enumerate() {
        match c.op {
            Op::SetPixel => set(img, i, c.args[0], c.args[1], c.color),
            Op::FillRect => {
                for y in c.args[1]..c.args[1] + c.args[3] {
                    for x in c.args[0]..c.args[0] + c.args[2] {
                        set(img, i, x, y, c.color);
                    }
                }
            }
            Op::Line => {
                for (x, y) in bresenham(c.args[0], c.args[1], c.args[2], c.args[3]) {
                    set(img, i, x, y, c.color);
                }
            }
            Op::Clear => {
                for y in 0..h {
                    for x in 0..w {
                        set(img, i, x, y, c.color);
                    }
                }
            }
        }
    }
}

/// Quita los comandos cuyos píxeles se pintaron todos encima después; la
/// imagen final queda igual.
pub fn clean_recipe(cmds: &[Cmd], w: u32, h: u32) -> Vec<Cmd> {
    let mut owner = vec![-1i64; w as usize * h as usize];
    paint(&mut Image::new(w, h), cmds, Some(&mut owner));
    let alive: std::collections::HashSet<i64> = owner.into_iter().filter(|o| *o >= 0).collect();
    let mut out: Vec<Cmd> = cmds
        .iter()
        .enumerate()
        .filter(|(i, _)| alive.contains(&(*i as i64)))
        .map(|(_, c)| c.clone())
        .collect();
    // Un `clear` transparente al principio no cambia un lienzo nuevo.
    while out
        .first()
        .is_some_and(|c| c.op == Op::Clear && c.color.a == 0)
    {
        out.remove(0);
    }
    out
}

/// Rehace una receta solo con los píxeles (para PNG dibujados fuera):
/// rectángulos del mismo color, de arriba abajo.
pub fn recipe_from_image(img: &Image) -> Vec<Cmd> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let at = |x: usize, y: usize| img.pixels()[y * w + x];
    let mut done = vec![false; w * h];
    let mut cmds = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let c = at(x, y);
            if done[y * w + x] || c.a == 0 {
                continue;
            }
            let mut rw = 1;
            while x + rw < w && !done[y * w + x + rw] && at(x + rw, y) == c {
                rw += 1;
            }
            let mut rh = 1;
            'grow: while y + rh < h {
                for i in 0..rw {
                    if done[(y + rh) * w + x + i] || at(x + i, y + rh) != c {
                        break 'grow;
                    }
                }
                rh += 1;
            }
            for yy in y..y + rh {
                for xx in x..x + rw {
                    done[yy * w + xx] = true;
                }
            }
            let (x, y) = (x as i64, y as i64);
            cmds.push(if rw == 1 && rh == 1 {
                Cmd {
                    op: Op::SetPixel,
                    args: vec![x, y],
                    color: c,
                }
            } else {
                Cmd {
                    op: Op::FillRect,
                    args: vec![x, y, rw as i64, rh as i64],
                    color: c,
                }
            });
        }
    }
    cmds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_errors_like_pxcli() {
        assert_eq!(
            parse_recipe(&["undo"]).unwrap_err(),
            "line 1: undo is not a drawing command"
        );
        assert_eq!(
            parse_recipe(&["", "fill_rect 1 2 3 #fff"]).unwrap_err(),
            "line 2: fill_rect needs 5 arguments"
        );
        assert_eq!(
            parse_recipe(&["set_pixel a 2 red"]).unwrap_err(),
            "line 1: \"a\" is not an integer"
        );
        assert!(
            parse_recipe(&["set_pixel 1 2 nope"])
                .unwrap_err()
                .starts_with("line 1: invalid_color")
        );
        let cmds =
            parse_recipe(&["# c", "pxcli clear", "/usr/bin/pxcli set_pixel 1 2 'red'"]).unwrap();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].color, Rgba::TRANSPARENT);
        assert_eq!(cmds[1].color, Rgba::rgb(255, 0, 0));
    }
}
