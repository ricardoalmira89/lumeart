//! Errores del dominio con los códigos del protocolo de pxcli.
//!
//! En pxcli cada paquete tenía su `Error { Code, Message }` y el daemon los
//! convertía en `err <código> <mensaje>`. Aquí hay un solo tipo para todo el
//! dominio con el mismo par código + mensaje.

use std::fmt;

/// Código de error del protocolo (`err <código> <mensaje>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Comando desconocido o línea vacía.
    InvalidCommand,
    /// Número o tipo de argumentos incorrecto.
    InvalidArgs,
    /// Color no reconocido.
    InvalidColor,
    /// Coordenada fuera del lienzo.
    OutOfBounds,
    /// Deshacer o rehacer sin historial.
    NoHistory,
    /// Error al leer o escribir un archivo.
    Io,
    /// La ventana no está disponible.
    RendererUnavailable,
}

impl ErrorCode {
    /// Texto del código tal como aparece en el protocolo.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::InvalidCommand => "invalid_command",
            ErrorCode::InvalidArgs => "invalid_args",
            ErrorCode::InvalidColor => "invalid_color",
            ErrorCode::OutOfBounds => "out_of_bounds",
            ErrorCode::NoHistory => "no_history",
            ErrorCode::Io => "io",
            ErrorCode::RendererUnavailable => "renderer_unavailable",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error del dominio: un código del protocolo y un mensaje igual al de pxcli.
///
/// Se muestra igual que `Error()` en pxcli: `código: mensaje`, o solo el código.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}", display(.code, .message))]
pub struct DomainError {
    pub code: ErrorCode,
    pub message: String,
}

impl DomainError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn invalid_color(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidColor, message)
    }

    pub fn invalid_args(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidArgs, message)
    }

    pub fn invalid_command(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidCommand, message)
    }

    pub fn out_of_bounds(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::OutOfBounds, message)
    }

    pub fn no_history(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NoHistory, message)
    }
}

fn display(code: &ErrorCode, message: &str) -> String {
    if message.is_empty() {
        code.to_string()
    } else {
        format!("{code}: {message}")
    }
}

/// Entrecomilla como `%q` de Go para cadenas imprimibles: comillas dobles y
/// escapes de `\`, `"` y caracteres de control.
pub fn go_quote(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 2);
    out.push('"');
    for c in input.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{07}' => out.push_str("\\a"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0b}' => out.push_str("\\v"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_pxcli() {
        let e = DomainError::invalid_color("color is required");
        assert_eq!(e.to_string(), "invalid_color: color is required");
        let e = DomainError::new(ErrorCode::NoHistory, "");
        assert_eq!(e.to_string(), "no_history");
    }

    #[test]
    fn codes_have_protocol_names() {
        let all = [
            (ErrorCode::InvalidCommand, "invalid_command"),
            (ErrorCode::InvalidArgs, "invalid_args"),
            (ErrorCode::InvalidColor, "invalid_color"),
            (ErrorCode::OutOfBounds, "out_of_bounds"),
            (ErrorCode::NoHistory, "no_history"),
            (ErrorCode::Io, "io"),
            (ErrorCode::RendererUnavailable, "renderer_unavailable"),
        ];
        for (code, name) in all {
            assert_eq!(code.as_str(), name);
        }
    }

    #[test]
    fn go_quote_escapes_like_go() {
        assert_eq!(go_quote("#12"), "\"#12\"");
        assert_eq!(go_quote("a\"b\\c"), "\"a\\\"b\\\\c\"");
        assert_eq!(go_quote("x\ty"), "\"x\\ty\"");
        assert_eq!(go_quote("\u{01}"), "\"\\x01\"");
        assert_eq!(go_quote("ñ"), "\"ñ\"");
    }
}
