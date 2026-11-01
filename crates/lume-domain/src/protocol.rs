//! Protocolo de texto compatible con pxcli: una petición por línea y una
//! respuesta de una línea (`ok`, `ok <dato>` o `err <código> <mensaje>`).
//!
//! Base: `internal/protocol/protocol.go` y `internal/daemon/handler.go`. El
//! orden de validación es el de pxcli: número de argumentos, enteros, color y,
//! al aplicar, los límites del lienzo. `export` y `stop` necesitan E/S, así que
//! aquí solo se parsean y [`execute`] los devuelve como efectos para quien
//! llama (la `Session`).

use std::fmt;

use crate::canvas::{Canvas, MAX_CANVAS_SIZE};
use crate::color::Rgba;
use crate::command::Command;
use crate::error::{DomainError, go_quote};
use crate::history::{History, Origin};

/// Escala máxima de `export` (como pxcli).
pub const MAX_EXPORT_SCALE: u32 = 64;

/// Una petición del protocolo ya validada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Un comando de dibujo: `set_pixel`, `fill_rect`, `line`, `clear` o
    /// `resize` (este último solo existe en LumeArt).
    Apply(Command),
    /// `get_pixel x y` → `ok #rrggbbaa`
    GetPixel { x: i64, y: i64 },
    /// `export <archivo> [escala]`
    Export { path: String, scale: u32 },
    /// `undo`
    Undo,
    /// `redo`
    Redo,
    /// `new W H` → lienzo en blanco de ese tamaño y sin historial.
    New { w: u32, h: u32 },
    /// `stop`
    Stop,
}

/// ¿Es un espacio ASCII para el protocolo? (Como `fieldsASCII` de pxcli:
/// incluye el tabulador vertical, que `char::is_ascii_whitespace` no trata
/// como espacio.)
pub fn is_protocol_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{0b}' | '\u{0c}')
}

/// Divide en campos por espacios ASCII, como `fieldsASCII` de pxcli.
///
/// Extensión de LumeArt: un campo que empieza por `"` llega hasta la
/// siguiente `"` y puede tener espacios (rutas como `"C:\Users\Ana María\a.png"`);
/// dentro, `\"` es una comilla y `\\` una barra. Sin comilla de cierre, el
/// campo llega hasta el final de la línea.
pub fn fields(line: &str) -> Vec<String> {
    split_fields(line, is_protocol_space)
}

/// Divide en campos con el separador dado y las comillas de [`fields`].
pub fn split_fields(line: &str, is_space: impl Fn(char) -> bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = line.chars().peekable();
    loop {
        while chars.next_if(|c| is_space(*c)).is_some() {}
        let Some(&first) = chars.peek() else {
            return out;
        };
        let mut field = String::new();
        if first == '"' {
            chars.next();
            while let Some(c) = chars.next() {
                match c {
                    '"' => break,
                    '\\' if matches!(chars.peek(), Some('"' | '\\')) => {
                        field.push(chars.next().expect("se acaba de mirar"));
                    }
                    c => field.push(c),
                }
            }
        }
        while let Some(c) = chars.next_if(|c| !is_space(*c)) {
            field.push(c);
        }
        out.push(field);
    }
}

/// Escribe un campo para que [`fields`] lo lea igual: entre comillas si tiene
/// espacios o comillas.
pub fn quote_field(field: &str) -> String {
    if !field
        .chars()
        .any(|c| is_protocol_space(c) || c.is_whitespace() || c == '"')
    {
        return field.to_string();
    }
    let mut out = String::from('"');
    for c in field.chars() {
        if c == '"' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// Parsea una línea del protocolo.
pub fn parse_request(line: &str) -> Result<Request, DomainError> {
    let fields = fields(line);
    let Some((name, rest)) = fields.split_first() else {
        return Err(DomainError::invalid_command("command is required"));
    };
    let args: Vec<&str> = rest.iter().map(String::as_str).collect();
    let args = args.as_slice();
    match name.as_str() {
        "set_pixel" => {
            expect_args(args, 3)?;
            let x = int(args[0], "x")?;
            let y = int(args[1], "y")?;
            let color = Rgba::parse(args[2])?;
            Ok(Request::Apply(Command::SetPixel { x, y, color }))
        }
        "get_pixel" => {
            expect_args(args, 2)?;
            let x = int(args[0], "x")?;
            let y = int(args[1], "y")?;
            Ok(Request::GetPixel { x, y })
        }
        "fill_rect" => {
            expect_args(args, 5)?;
            let x = int(args[0], "x")?;
            let y = int(args[1], "y")?;
            let w = int(args[2], "w")?;
            let h = int(args[3], "h")?;
            let color = Rgba::parse(args[4])?;
            Ok(Request::Apply(Command::FillRect { x, y, w, h, color }))
        }
        "line" => {
            expect_args(args, 5)?;
            let x1 = int(args[0], "x1")?;
            let y1 = int(args[1], "y1")?;
            let x2 = int(args[2], "x2")?;
            let y2 = int(args[3], "y2")?;
            let color = Rgba::parse(args[4])?;
            Ok(Request::Apply(Command::Line {
                from: (x1, y1),
                to: (x2, y2),
                color,
            }))
        }
        "clear" => {
            if args.len() > 1 {
                return Err(arg_count(1, args.len()));
            }
            let color = args.first().map(|c| Rgba::parse(c)).transpose()?;
            Ok(Request::Apply(Command::Clear { color }))
        }
        "resize" => {
            expect_args(args, 2)?;
            let w = int(args[0], "width")?;
            let h = int(args[1], "height")?;
            Ok(Request::Apply(Command::Resize { w, h }))
        }
        "export" => {
            if args.len() != 1 && args.len() != 2 {
                return Err(DomainError::invalid_args(format!(
                    "expected 1 or 2 args, got {}",
                    args.len()
                )));
            }
            let scale = match args.get(1) {
                Some(raw) => int(raw, "scale")?,
                None => 1,
            };
            if !(1..=i64::from(MAX_EXPORT_SCALE)).contains(&scale) {
                return Err(DomainError::invalid_args(format!(
                    "scale must be between 1 and {MAX_EXPORT_SCALE}"
                )));
            }
            Ok(Request::Export {
                path: args[0].to_string(),
                scale: scale as u32,
            })
        }
        "new" => {
            expect_args(args, 2)?;
            let w = int(args[0], "width")?;
            let h = int(args[1], "height")?;
            let max = i64::from(MAX_CANVAS_SIZE);
            if w < 1 || h < 1 || w > max || h > max {
                return Err(DomainError::invalid_args(format!(
                    "width and height must be between 1 and {MAX_CANVAS_SIZE}"
                )));
            }
            Ok(Request::New {
                w: w as u32,
                h: h as u32,
            })
        }
        "undo" => {
            expect_args(args, 0)?;
            Ok(Request::Undo)
        }
        "redo" => {
            expect_args(args, 0)?;
            Ok(Request::Redo)
        }
        "stop" => {
            expect_args(args, 0)?;
            Ok(Request::Stop)
        }
        other => Err(DomainError::invalid_command(format!(
            "unknown command {}",
            go_quote(other)
        ))),
    }
}

fn expect_args(args: &[&str], expected: usize) -> Result<(), DomainError> {
    if args.len() != expected {
        return Err(arg_count(expected, args.len()));
    }
    Ok(())
}

fn arg_count(expected: usize, got: usize) -> DomainError {
    DomainError::invalid_args(format!("expected {expected} args, got {got}"))
}

/// Entero como `strconv.Atoi` de Go (signo opcional, sin espacios, 64 bits).
fn int(raw: &str, name: &str) -> Result<i64, DomainError> {
    raw.parse::<i64>()
        .map_err(|_| DomainError::invalid_args(format!("{name} must be an integer")))
}

/// `FormatOK` de pxcli: `ok` u `ok <dato>`.
pub fn format_ok(payload: &str) -> String {
    let payload = payload.trim();
    if payload.is_empty() {
        "ok".to_string()
    } else {
        format!("ok {payload}")
    }
}

/// `FormatError` de pxcli: `err <código> <mensaje>`.
pub fn format_error(code: &str, message: &str) -> String {
    let code = match code.trim() {
        "" => "error",
        c => c,
    };
    let message = match message.trim() {
        "" => "unknown error",
        m => m,
    };
    format!("err {code} {message}")
}

/// Respuesta de una línea del protocolo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// `ok` (con dato opcional).
    Ok(Option<String>),
    /// `err <código> <mensaje>`.
    Err(DomainError),
}

impl Response {
    pub fn ok() -> Self {
        Response::Ok(None)
    }

    pub fn ok_with(payload: impl Into<String>) -> Self {
        Response::Ok(Some(payload.into()))
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Response::Ok(_))
    }
}

impl From<Result<Option<String>, DomainError>> for Response {
    fn from(result: Result<Option<String>, DomainError>) -> Self {
        match result {
            Ok(payload) => Response::Ok(payload),
            Err(err) => Response::Err(err),
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Ok(payload) => f.write_str(&format_ok(payload.as_deref().unwrap_or(""))),
            Response::Err(err) => f.write_str(&format_error(err.code.as_str(), &err.message)),
        }
    }
}

/// Resultado de ejecutar una petición.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Executed {
    /// Respuesta lista para enviar.
    Reply(Response),
    /// Hay que exportar el lienzo a PNG (lo hace quien llama) y responder.
    Export { path: String, scale: u32 },
    /// Hay que parar el servidor (y responder `ok`).
    Stop,
}

/// Ejecuta una petición sobre el lienzo y su historial.
pub fn execute(
    request: Request,
    canvas: &mut Canvas,
    history: &mut History,
    origin: Origin,
) -> Executed {
    let reply = |result: Result<Option<String>, DomainError>| Executed::Reply(result.into());
    match request {
        Request::Apply(command) => reply(history.apply(canvas, command, origin).map(|_| None)),
        Request::GetPixel { x, y } => reply(canvas.get(x, y).map(|c| Some(c.to_string()))),
        Request::Undo => reply(history.undo(canvas).map(|_| None)),
        Request::Redo => reply(history.redo(canvas).map(|_| None)),
        Request::New { w, h } => reply(Canvas::new(i64::from(w), i64::from(h)).map(|blank| {
            *canvas = blank;
            history.reset(w, h);
            Some(format!("{w}x{h}"))
        })),
        Request::Export { path, scale } => Executed::Export { path, scale },
        Request::Stop => Executed::Stop,
    }
}

/// Parsea y ejecuta una línea; los errores de parseo se devuelven como `err`.
pub fn handle_line(
    line: &str,
    canvas: &mut Canvas,
    history: &mut History,
    origin: Origin,
) -> Executed {
    match parse_request(line) {
        Ok(request) => execute(request, canvas, history, origin),
        Err(err) => Executed::Reply(Response::Err(err)),
    }
}

/// Formato del protocolo: una línea por comando simple (un `Batch` se aplana
/// en varias líneas). Parsear la línea de un comando simple devuelve el mismo
/// comando.
impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::SetPixel { x, y, color } => write!(f, "set_pixel {x} {y} {color}"),
            Command::FillRect { x, y, w, h, color } => {
                write!(f, "fill_rect {x} {y} {w} {h} {color}")
            }
            Command::Line { from, to, color } => {
                write!(f, "line {} {} {} {} {color}", from.0, from.1, to.0, to.1)
            }
            Command::Clear { color: None } => f.write_str("clear"),
            Command::Clear { color: Some(c) } => write!(f, "clear {c}"),
            Command::Resize { w, h } => write!(f, "resize {w} {h}"),
            Command::Batch(commands) => {
                for (i, command) in commands.iter().enumerate() {
                    if i > 0 {
                        f.write_str("\n")?;
                    }
                    write!(f, "{command}")?;
                }
                Ok(())
            }
        }
    }
}

/// Formato de la petición en el protocolo (lo que envía la CLI).
impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Request::Apply(command) => write!(f, "{command}"),
            Request::GetPixel { x, y } => write!(f, "get_pixel {x} {y}"),
            Request::Export { path, scale: 1 } => write!(f, "export {}", quote_field(path)),
            Request::Export { path, scale } => {
                write!(f, "export {} {scale}", quote_field(path))
            }
            Request::Undo => f.write_str("undo"),
            Request::Redo => f.write_str("redo"),
            Request::New { w, h } => write!(f, "new {w} {h}"),
            Request::Stop => f.write_str("stop"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;
    use proptest::prelude::*;

    // --- Portados de internal/protocol/protocol_test.go ---

    #[test]
    fn parse_line() {
        assert_eq!(
            fields("set_pixel 10 10 #ff0000"),
            vec!["set_pixel", "10", "10", "#ff0000"]
        );
        assert_eq!(
            parse_request("set_pixel 10 10 #ff0000").unwrap(),
            Request::Apply(Command::SetPixel {
                x: 10,
                y: 10,
                color: Rgba::rgb(255, 0, 0)
            })
        );
    }

    #[test]
    fn parse_line_whitespace() {
        assert_eq!(fields("  get_pixel   1\t2  "), vec!["get_pixel", "1", "2"]);
        assert_eq!(
            parse_request("  get_pixel   1\t2  ").unwrap(),
            Request::GetPixel { x: 1, y: 2 }
        );
        assert_eq!(fields("a\u{0b}b\u{0c}c\rd"), vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn quoted_fields_are_a_lumeart_extension() {
        assert_eq!(
            fields(r#"export "/home/ana/mi juego/a.png" 4"#),
            vec!["export", "/home/ana/mi juego/a.png", "4"]
        );
        assert_eq!(
            fields(r#"a "b \"c\" \\ d" e"#),
            vec!["a", r#"b "c" \ d"#, "e"]
        );
        assert_eq!(
            fields(r#""C:\Users\Ana María\x.png""#),
            vec![r#"C:\Users\Ana María\x.png"#]
        );
        assert_eq!(fields(r#""sin cierre"#), vec!["sin cierre"]);
        assert_eq!(fields(r#"x"y z"#), vec![r#"x"y"#, "z"]);
        for raw in [
            "/a b/c.png",
            r#"comilla " y \ barra"#,
            "plano",
            "tab\tdentro",
            r#"C:\dir\"#,
        ] {
            let line = format!("export {} 2", quote_field(raw));
            assert_eq!(
                parse_request(&line).unwrap(),
                Request::Export {
                    path: raw.to_string(),
                    scale: 2
                },
                "{line}"
            );
        }
        assert_eq!(quote_field("sin-espacios"), "sin-espacios");
        let req = Request::Export {
            path: "/mi juego/a.png".into(),
            scale: 1,
        };
        assert_eq!(req.to_string(), r#"export "/mi juego/a.png""#);
        assert_eq!(parse_request(&req.to_string()).unwrap(), req);
        // Un color entre comillas también vale (pxcli daba invalid_color).
        assert!(parse_request(r##"set_pixel 0 0 "#ff0000""##).is_ok());
    }

    #[test]
    fn parse_line_empty() {
        let err = parse_request("\t  \n").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidCommand);
        assert_eq!(err.message, "command is required");
    }

    #[test]
    fn format_ok_matches_pxcli() {
        assert_eq!(format_ok(""), "ok");
        assert_eq!(format_ok("#ff0000ff"), "ok #ff0000ff");
        assert_eq!(format_ok("  16x8 "), "ok 16x8");
    }

    #[test]
    fn format_error_matches_pxcli() {
        assert_eq!(
            format_error("invalid_command", "bad request"),
            "err invalid_command bad request"
        );
        assert_eq!(format_error(" ", ""), "err error unknown error");
    }

    // --- Portados de internal/daemon/handler_test.go y handler_new_test.go ---

    struct Fixture {
        canvas: Canvas,
        history: History,
    }

    impl Fixture {
        fn new(w: i64, h: i64) -> Self {
            Self {
                canvas: Canvas::new(w, h).unwrap(),
                history: History::new(),
            }
        }

        fn run(&mut self, line: &str) -> Executed {
            handle_line(line, &mut self.canvas, &mut self.history, Origin::Cli)
        }

        fn reply(&mut self, line: &str) -> String {
            match self.run(line) {
                Executed::Reply(r) => r.to_string(),
                other => panic!("se esperaba una respuesta y llegó {other:?}"),
            }
        }
    }

    #[test]
    fn handler_get_pixel() {
        let mut fx = Fixture::new(4, 4);
        fx.canvas.set(1, 2, Rgba::new(1, 2, 3, 4)).unwrap();
        assert_eq!(fx.reply("get_pixel 1 2"), "ok #01020304");
    }

    #[test]
    fn handler_set_pixel_arg_count_error() {
        let mut fx = Fixture::new(2, 2);
        assert_eq!(
            fx.reply("set_pixel 1 2"),
            "err invalid_args expected 3 args, got 2"
        );
    }

    #[test]
    fn handler_unknown_command() {
        let mut fx = Fixture::new(2, 2);
        assert_eq!(
            fx.reply("nope"),
            "err invalid_command unknown command \"nope\""
        );
        assert_eq!(
            fx.reply("SET_PIXEL 1 1 red"),
            "err invalid_command unknown command \"SET_PIXEL\""
        );
    }

    #[test]
    fn handler_mutating_command_returns_ok() {
        let mut fx = Fixture::new(2, 2);
        assert_eq!(fx.reply("set_pixel 0 1 red"), "ok");
        assert_eq!(fx.canvas.get(0, 1).unwrap(), Rgba::rgb(255, 0, 0));
    }

    #[test]
    fn handler_clear_default_transparent() {
        let mut fx = Fixture::new(2, 2);
        fx.canvas.set(0, 0, Rgba::rgb(10, 20, 30)).unwrap();
        assert_eq!(fx.reply("clear"), "ok");
        assert_eq!(fx.canvas.get(0, 0).unwrap(), Rgba::TRANSPARENT);
    }

    #[test]
    fn handler_stop() {
        let mut fx = Fixture::new(2, 2);
        assert_eq!(fx.run("stop"), Executed::Stop);
        assert_eq!(
            fx.reply("stop now"),
            "err invalid_args expected 0 args, got 1"
        );
    }

    #[test]
    fn handle_new_and_scaled_export() {
        let mut fx = Fixture::new(4, 4);
        assert_eq!(fx.reply("set_pixel 0 0 #fff"), "ok");
        assert_eq!(fx.reply("new 16 8"), "ok 16x8");
        assert_eq!((fx.canvas.width(), fx.canvas.height()), (16, 8));
        assert_eq!(fx.canvas.get(0, 0).unwrap(), Rgba::TRANSPARENT);
        assert_eq!(fx.reply("undo"), "err no_history nothing to undo");
        assert_eq!(
            fx.reply("new 0 8"),
            "err invalid_args width and height must be between 1 and 512"
        );
        assert_eq!(
            fx.reply("new 513 8"),
            "err invalid_args width and height must be between 1 and 512"
        );
        assert_eq!(
            fx.run("export /tmp/a.png 3"),
            Executed::Export {
                path: "/tmp/a.png".into(),
                scale: 3
            }
        );
        assert_eq!(
            fx.run("export /tmp/a.png"),
            Executed::Export {
                path: "/tmp/a.png".into(),
                scale: 1
            }
        );
        assert_eq!(
            fx.reply("export /tmp/a.png x"),
            "err invalid_args scale must be an integer"
        );
        assert_eq!(
            fx.reply("export /tmp/a.png 0"),
            "err invalid_args scale must be between 1 and 64"
        );
        assert_eq!(
            fx.reply("export /tmp/a.png 65"),
            "err invalid_args scale must be between 1 and 64"
        );
        assert_eq!(
            fx.reply("export"),
            "err invalid_args expected 1 or 2 args, got 0"
        );
    }

    // --- Ejemplos de docs/06-protocolo.md y salidas exactas de handler.go ---

    #[test]
    fn documented_examples() {
        let mut fx = Fixture::new(32, 32);
        assert_eq!(fx.reply("set_pixel 10 10 #ff0000"), "ok");
        assert_eq!(fx.reply("get_pixel 10 10"), "ok #ff0000ff");
        assert_eq!(
            fx.reply("set_pixel -1 10 #ff0000"),
            "err out_of_bounds pixel (-1,10) outside canvas"
        );
    }

    #[test]
    fn error_messages_match_handler() {
        let mut fx = Fixture::new(8, 8);
        let cases = [
            ("set_pixel a 1 red", "err invalid_args x must be an integer"),
            ("set_pixel 1 b red", "err invalid_args y must be an integer"),
            (
                "set_pixel 1 1 nocolor",
                "err invalid_color expected hex color or named color",
            ),
            (
                "set_pixel 1.5 1 red",
                "err invalid_args x must be an integer",
            ),
            (
                "set_pixel 99 1 bad",
                "err invalid_color expected hex color or named color",
            ),
            ("get_pixel 1", "err invalid_args expected 2 args, got 1"),
            ("get_pixel 1 z", "err invalid_args y must be an integer"),
            (
                "get_pixel 8 0",
                "err out_of_bounds pixel (8,0) outside canvas",
            ),
            ("fill_rect 0 0 2", "err invalid_args expected 5 args, got 3"),
            (
                "fill_rect 0 0 w 2 red",
                "err invalid_args w must be an integer",
            ),
            (
                "fill_rect 0 0 2 h red",
                "err invalid_args h must be an integer",
            ),
            (
                "fill_rect 0 0 0 2 red",
                "err invalid_args rect width and height must be positive",
            ),
            (
                "fill_rect 7 7 2 2 red",
                "err out_of_bounds rect (7,7) size 2x2 outside canvas",
            ),
            ("line 0 0 1", "err invalid_args expected 5 args, got 3"),
            ("line a 0 1 1 red", "err invalid_args x1 must be an integer"),
            ("line 0 a 1 1 red", "err invalid_args y1 must be an integer"),
            ("line 0 0 a 1 red", "err invalid_args x2 must be an integer"),
            ("line 0 0 1 a red", "err invalid_args y2 must be an integer"),
            (
                "line 0 0 8 0 red",
                "err out_of_bounds pixel (8,0) outside canvas",
            ),
            ("clear red blue", "err invalid_args expected 1 args, got 2"),
            ("clear #12", "err invalid_color invalid hex length 2"),
            ("new 4", "err invalid_args expected 2 args, got 1"),
            ("new x 4", "err invalid_args width must be an integer"),
            ("new 4 y", "err invalid_args height must be an integer"),
            ("undo now", "err invalid_args expected 0 args, got 1"),
            ("redo", "err no_history nothing to redo"),
            ("", "err invalid_command command is required"),
        ];
        for (line, want) in cases {
            assert_eq!(fx.reply(line), want, "{line:?}");
        }
        assert!(fx.canvas.pixels().iter().all(|p| *p == Rgba::TRANSPARENT));
    }

    #[test]
    fn integers_parse_like_go_atoi() {
        let mut fx = Fixture::new(8, 8);
        assert_eq!(fx.reply("set_pixel +1 007 red"), "ok");
        assert_eq!(fx.canvas.get(1, 7).unwrap(), Rgba::rgb(255, 0, 0));
        assert_eq!(
            fx.reply("set_pixel 0x1 0 red"),
            "err invalid_args x must be an integer"
        );
        assert_eq!(
            fx.reply("set_pixel 1_0 0 red"),
            "err invalid_args x must be an integer"
        );
        assert_eq!(
            fx.reply("set_pixel 9223372036854775807 0 red"),
            "err out_of_bounds pixel (9223372036854775807,0) outside canvas"
        );
        assert_eq!(
            fx.reply("set_pixel 9223372036854775808 0 red"),
            "err invalid_args x must be an integer"
        );
    }

    #[test]
    fn undo_redo_through_the_protocol() {
        let mut fx = Fixture::new(4, 4);
        assert_eq!(fx.reply("fill_rect 0 0 2 2 red"), "ok");
        assert_eq!(fx.reply("undo"), "ok");
        assert_eq!(fx.reply("get_pixel 0 0"), "ok #00000000");
        assert_eq!(fx.reply("redo"), "ok");
        assert_eq!(fx.reply("get_pixel 1 1"), "ok #ff0000ff");
    }

    #[test]
    fn resize_is_a_lumeart_extension() {
        let mut fx = Fixture::new(4, 4);
        assert_eq!(fx.reply("set_pixel 3 3 red"), "ok");
        assert_eq!(fx.reply("resize 8 2"), "ok");
        assert_eq!((fx.canvas.width(), fx.canvas.height()), (8, 2));
        assert_eq!(fx.reply("undo"), "ok");
        assert_eq!(fx.reply("get_pixel 3 3"), "ok #ff0000ff");
        assert_eq!(
            fx.reply("resize 0 2"),
            "err invalid_args canvas dimensions must be positive"
        );
    }

    #[test]
    fn requests_format_back_to_the_wire() {
        for line in [
            "set_pixel 1 2 #ff0000ff",
            "get_pixel 3 4",
            "fill_rect 0 0 2 2 #00000000",
            "line 0 0 3 3 #12345678",
            "clear",
            "clear #ffffffff",
            "resize 8 8",
            "export /tmp/x.png",
            "export /tmp/x.png 4",
            "undo",
            "redo",
            "new 16 8",
            "stop",
        ] {
            assert_eq!(parse_request(line).unwrap().to_string(), line);
        }
    }

    #[test]
    fn batch_formats_as_lines() {
        let batch = Command::Batch(vec![
            Command::SetPixel {
                x: 0,
                y: 0,
                color: Rgba::rgb(255, 0, 0),
            },
            Command::Clear { color: None },
        ]);
        assert_eq!(batch.to_string(), "set_pixel 0 0 #ff0000ff\nclear");
    }

    fn simple_command() -> impl Strategy<Value = Command> {
        let n = any::<i64>();
        let color = any::<(u8, u8, u8, u8)>().prop_map(|(r, g, b, a)| Rgba::new(r, g, b, a));
        prop_oneof![
            (n, n, color.clone()).prop_map(|(x, y, color)| Command::SetPixel { x, y, color }),
            (n, n, n, n, color.clone()).prop_map(|(x, y, w, h, color)| Command::FillRect {
                x,
                y,
                w,
                h,
                color
            }),
            (n, n, n, n, color.clone()).prop_map(|(a, b, c, d, color)| Command::Line {
                from: (a, b),
                to: (c, d),
                color
            }),
            proptest::option::of(color).prop_map(|color| Command::Clear { color }),
            (n, n).prop_map(|(w, h)| Command::Resize { w, h }),
        ]
    }

    proptest! {
        #[test]
        fn command_round_trip(cmd in simple_command()) {
            let line = cmd.to_string();
            prop_assert_eq!(parse_request(&line).unwrap(), Request::Apply(cmd));
        }
    }
}
