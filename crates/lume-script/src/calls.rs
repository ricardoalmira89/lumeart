//! Las llamadas de un script y la línea de su paréntesis.
//!
//! goja (el motor de pxcli) apunta cada llamada en curso en la línea de su
//! `(`; QuickJS, en la última expresión que evaluó dentro de ella (el nombre
//! de la función, un argumento…). Para que las líneas sean las de pxcli, se
//! recorre el código una vez buscando cada llamada (dónde empieza, su `(` y
//! su `)`), y una posición de QuickJS se traduce a la línea del `(` de la
//! llamada que la contiene.

/// Una posición: línea desde 1 y columna en bytes desde 1 (como las pilas
/// de QuickJS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    pub line: u32,
    pub col: u32,
}

/// Una llamada: desde el comienzo de lo llamado (con el objeto delante, en
/// `a.b(…)`) hasta su `)`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Call {
    start: Pos,
    open: Pos,
    close: Pos,
    /// El nombre justo antes del `(` (`b` en `a.b(…)`), si lo hay.
    name: Option<String>,
}

/// Lo último que se leyó: para saber si un `(` es una llamada, dónde empieza
/// lo llamado y si un `/` empieza una expresión regular.
#[derive(Debug, Clone, PartialEq)]
enum Prev {
    None,
    /// Un nombre, con dónde empieza su cadena (`a` en `a.b`) y si va justo
    /// después de `function` (entonces su `(` es de parámetros).
    Ident {
        text: String,
        chain: Pos,
        after_function: bool,
    },
    /// `)` o `]`, con dónde empieza lo que cierran (para `f(a)(b)`, `x[0](…)`).
    Close {
        chain: Pos,
    },
    /// `.` o `?.`, con dónde empieza lo de delante.
    Dot {
        chain: Pos,
    },
    /// Un número, una cadena, una plantilla o una expresión regular.
    Value,
    /// Cualquier otro signo.
    Punct,
}

impl Prev {
    /// Dónde empieza la expresión que acaba aquí (si es un nombre o un
    /// cierre).
    fn chain(&self) -> Option<Pos> {
        match self {
            Prev::Ident { chain, .. } | Prev::Close { chain } => Some(*chain),
            _ => None,
        }
    }
}

/// Palabras tras las que un `(` no es una llamada (y un `/` es una
/// expresión regular).
const KEYWORDS: &[&str] = &[
    "if",
    "for",
    "while",
    "switch",
    "catch",
    "function",
    "return",
    "typeof",
    "void",
    "delete",
    "in",
    "of",
    "instanceof",
    "else",
    "do",
    "case",
    "throw",
    "yield",
    "await",
    "with",
    "new",
];

/// Un grupo abierto: `(`, `[`, `{` o el `${` de una plantilla.
#[derive(Debug, Clone)]
struct Open {
    kind: u8,
    /// Si es el `(` de una llamada: su índice.
    call: Option<usize>,
    /// Dónde empieza la expresión que acaba en este grupo al cerrarse.
    chain: Pos,
}

struct Scanner<'a> {
    src: &'a [u8],
    i: usize,
    line: u32,
    line_start: usize,
}

impl Scanner<'_> {
    fn pos(&self) -> Pos {
        Pos {
            line: self.line,
            col: (self.i - self.line_start) as u32 + 1,
        }
    }

    fn peek(&self, ahead: usize) -> u8 {
        self.src.get(self.i + ahead).copied().unwrap_or(0)
    }

    fn done(&self) -> bool {
        self.i >= self.src.len()
    }

    /// Avanza un byte, llevando la cuenta de las líneas.
    fn bump(&mut self) {
        if self.peek(0) == b'\n' {
            self.line += 1;
            self.line_start = self.i + 1;
        }
        self.i += 1;
    }

    /// Salta una cadena (desde su comilla de apertura).
    fn string(&mut self, quote: u8) {
        self.bump();
        while !self.done() {
            match self.peek(0) {
                b'\\' => {
                    self.bump();
                    self.bump();
                }
                b'\n' => return,
                c if c == quote => {
                    self.bump();
                    return;
                }
                _ => self.bump(),
            }
        }
    }

    /// Salta una expresión regular (desde su `/` de apertura).
    fn regex(&mut self) {
        self.bump();
        let mut class = false;
        while !self.done() {
            match self.peek(0) {
                b'\\' => {
                    self.bump();
                    self.bump();
                }
                b'\n' => return,
                b'[' => {
                    class = true;
                    self.bump();
                }
                b']' => {
                    class = false;
                    self.bump();
                }
                b'/' if !class => {
                    self.bump();
                    while self.peek(0).is_ascii_alphabetic() {
                        self.bump();
                    }
                    return;
                }
                _ => self.bump(),
            }
        }
    }

    /// Salta el texto de una plantilla hasta su final o hasta un `${` (que
    /// deja leído). Devuelve si se abrió un `${`.
    fn template_text(&mut self) -> bool {
        while !self.done() {
            match self.peek(0) {
                b'\\' => {
                    self.bump();
                    self.bump();
                }
                b'`' => {
                    self.bump();
                    return false;
                }
                b'$' if self.peek(1) == b'{' => {
                    self.bump();
                    self.bump();
                    return true;
                }
                _ => self.bump(),
            }
        }
        false
    }
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$' || c >= 0x80
}

fn is_ident_char(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

/// Las llamadas de un script.
#[derive(Debug, Clone, Default)]
pub struct CallMap {
    calls: Vec<Call>,
}

impl CallMap {
    /// Busca las llamadas de `source`.
    pub fn new(source: &str) -> Self {
        let mut s = Scanner {
            src: source.as_bytes(),
            i: 0,
            line: 1,
            line_start: 0,
        };
        let mut calls: Vec<Call> = Vec::new();
        let mut stack: Vec<Open> = Vec::new();
        let mut prev = Prev::None;
        while !s.done() {
            let c = s.peek(0);
            let at = s.pos();
            match c {
                b' ' | b'\t' | b'\r' | b'\n' => s.bump(),
                b'/' if s.peek(1) == b'/' => {
                    while !s.done() && s.peek(0) != b'\n' {
                        s.bump();
                    }
                }
                b'/' if s.peek(1) == b'*' => {
                    s.bump();
                    s.bump();
                    while !s.done() && (s.peek(0) != b'*' || s.peek(1) != b'/') {
                        s.bump();
                    }
                    s.bump();
                    s.bump();
                }
                b'/' => {
                    // Tras un valor, es una división; si no, una expresión
                    // regular.
                    let division = match &prev {
                        Prev::Ident { text, .. } => !KEYWORDS.contains(&text.as_str()),
                        Prev::Close { .. } | Prev::Value => true,
                        _ => false,
                    };
                    if division {
                        s.bump();
                        prev = Prev::Punct;
                    } else {
                        s.regex();
                        prev = Prev::Value;
                    }
                }
                b'"' | b'\'' => {
                    s.string(c);
                    prev = Prev::Value;
                }
                b'`' => {
                    s.bump();
                    if s.template_text() {
                        stack.push(Open {
                            kind: b'$',
                            call: None,
                            chain: at,
                        });
                        prev = Prev::Punct;
                    } else {
                        prev = Prev::Value;
                    }
                }
                b'0'..=b'9' => {
                    while is_ident_char(s.peek(0)) || s.peek(0) == b'.' {
                        s.bump();
                    }
                    prev = Prev::Value;
                }
                b'.' if s.peek(1) == b'.' && s.peek(2) == b'.' => {
                    s.bump();
                    s.bump();
                    s.bump();
                    prev = Prev::Punct;
                }
                b'.' if s.peek(1).is_ascii_digit() => {
                    s.bump();
                    while is_ident_char(s.peek(0)) {
                        s.bump();
                    }
                    prev = Prev::Value;
                }
                b'.' => {
                    s.bump();
                    prev = Prev::Dot {
                        chain: prev.chain().unwrap_or(at),
                    };
                }
                b'?' if s.peek(1) == b'.' && !s.peek(2).is_ascii_digit() => {
                    s.bump();
                    s.bump();
                    prev = Prev::Dot {
                        chain: prev.chain().unwrap_or(at),
                    };
                }
                c if is_ident_start(c) => {
                    let begin = s.i;
                    while !s.done() && is_ident_char(s.peek(0)) {
                        s.bump();
                    }
                    let text = String::from_utf8_lossy(&s.src[begin..s.i]).into_owned();
                    let after_function =
                        matches!(&prev, Prev::Ident { text, .. } if text == "function");
                    let chain = match &prev {
                        Prev::Dot { chain } => *chain,
                        _ => at,
                    };
                    prev = Prev::Ident {
                        text,
                        chain,
                        after_function,
                    };
                }
                b'(' => {
                    s.bump();
                    let call = match &prev {
                        Prev::Ident {
                            text,
                            chain,
                            after_function,
                        } if !after_function && !KEYWORDS.contains(&text.as_str()) => {
                            Some((Some(text.clone()), *chain))
                        }
                        Prev::Close { chain } => Some((None, *chain)),
                        _ => None,
                    };
                    let chain = call.as_ref().map_or(at, |(_, chain)| *chain);
                    let index = call.map(|(name, start)| {
                        calls.push(Call {
                            start,
                            open: at,
                            close: at,
                            name,
                        });
                        calls.len() - 1
                    });
                    stack.push(Open {
                        kind: b'(',
                        call: index,
                        chain,
                    });
                    prev = Prev::Punct;
                }
                b'[' => {
                    s.bump();
                    stack.push(Open {
                        kind: b'[',
                        call: None,
                        chain: prev.chain().unwrap_or(at),
                    });
                    prev = Prev::Punct;
                }
                b'{' => {
                    s.bump();
                    stack.push(Open {
                        kind: b'{',
                        call: None,
                        chain: at,
                    });
                    prev = Prev::Punct;
                }
                b')' | b']' => {
                    s.bump();
                    let open = stack.pop();
                    if let Some(index) = open.as_ref().and_then(|o| o.call) {
                        calls[index].close = at;
                    }
                    prev = Prev::Close {
                        chain: open.map_or(at, |o| o.chain),
                    };
                }
                b'}' => {
                    s.bump();
                    match stack.pop() {
                        // El final de un `${…}`: sigue la plantilla.
                        Some(Open { kind: b'$', .. }) => {
                            if s.template_text() {
                                stack.push(Open {
                                    kind: b'$',
                                    call: None,
                                    chain: at,
                                });
                                prev = Prev::Punct;
                            } else {
                                prev = Prev::Value;
                            }
                        }
                        _ => prev = Prev::Punct,
                    }
                }
                _ => {
                    s.bump();
                    prev = Prev::Punct;
                }
            }
        }
        CallMap { calls }
    }

    /// La línea del `(` de la llamada en curso en `at` (una posición de
    /// QuickJS), a la función `callee` si se sabe cuál es. Entre las
    /// llamadas que contienen `at`, la más interior con ese nombre o, si
    /// ninguna lo tiene, la más interior. Sin ninguna, la línea de `at`.
    pub fn paren_line(&self, at: Pos, callee: Option<&str>) -> u32 {
        let containing = || {
            self.calls
                .iter()
                .filter(move |c| c.start <= at && at <= c.close)
        };
        fn innermost<'a>(calls: &mut dyn Iterator<Item = &'a Call>) -> Option<&'a Call> {
            calls.max_by(|a, b| a.start.cmp(&b.start).then(b.close.cmp(&a.close)))
        }
        callee
            .and_then(|name| {
                innermost(&mut containing().filter(|c| c.name.as_deref() == Some(name)))
            })
            .or_else(|| innermost(&mut containing()))
            .map_or(at.line, |c| c.open.line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(line: u32, col: u32) -> Pos {
        Pos { line, col }
    }

    /// Los casos medidos con goja y QuickJS: QuickJS da la posición de la
    /// izquierda; goja, la línea del `(`.
    #[test]
    fn quickjs_positions_become_pxcli_lines() {
        let src = "function f() { pila() }
f(1,
  2,
  3)
var x = f(
  [1, 2],
  [3, 4]
)
function g(cb) { cb() }
g(function () {
  pila()
  f()
})
obj = { m: f }
obj
  .m(1,
     2)
f(1, 2); f(3,
  4)
var y = f
  (5)
";
        let map = CallMap::new(src);
        assert_eq!(map.paren_line(at(2, 1), Some("f")), 2);
        assert_eq!(map.paren_line(at(6, 3), Some("f")), 5);
        assert_eq!(map.paren_line(at(10, 3), Some("g")), 10);
        assert_eq!(map.paren_line(at(12, 3), Some("f")), 12);
        assert_eq!(map.paren_line(at(16, 4), Some("f")), 16);
        assert_eq!(map.paren_line(at(18, 10), Some("f")), 18);
        assert_eq!(map.paren_line(at(21, 3), Some("f")), 21);
        // La declaración de una función no es una llamada.
        assert_eq!(map.paren_line(at(1, 12), None), 1);
    }

    #[test]
    fn strings_comments_templates_and_regexes_are_skipped() {
        let src = "var a = \"f(\" + 'g(' // h(
/* i( */ var r = /\\(x/g; var t = `j( ${k(1,
2)} l(`
freeform(iris, [
  [1, 2, \"#fff\"],
  [3, 4, P.dark],
])
";
        let map = CallMap::new(src);
        let names: Vec<_> = map.calls.iter().map(|c| c.name.clone().unwrap()).collect();
        assert_eq!(names, ["k", "freeform"]);
        assert_eq!(map.paren_line(at(3, 1), Some("k")), 2);
        // QuickJS señala el último argumento; goja, el `(`.
        assert_eq!(map.paren_line(at(6, 3), Some("freeform")), 4);
    }

    #[test]
    fn the_callee_picks_among_nested_calls() {
        let src = "circle(cx(1),
  cy(2), 5)
";
        let map = CallMap::new(src);
        // En `cy`, la de `cy`; ya en `circle`, la de `circle`.
        assert_eq!(map.paren_line(at(2, 3), Some("cy")), 2);
        assert_eq!(map.paren_line(at(2, 3), Some("circle")), 1);
        assert_eq!(map.paren_line(at(2, 3), None), 2);
        assert_eq!(map.paren_line(at(9, 1), None), 9);
    }
}
