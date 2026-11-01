//! Resaltado de sintaxis de `dibujo.js` y `anim.js` (JavaScript) con los
//! colores del tema: palabras clave en verde menta, funciones en azul,
//! cadenas en ámbar, números en coral y comentarios apagados.

use eframe::egui::{Color32, FontId, TextFormat, text::LayoutJob};

use crate::theme;

pub const KEYWORD: Color32 = Color32::from_rgb(0x7F, 0xD1, 0xAE);
pub const FUNCTION: Color32 = Color32::from_rgb(0x6C, 0xB6, 0xFF);
pub const STRING: Color32 = Color32::from_rgb(0xE5, 0xC0, 0x7B);
pub const NUMBER: Color32 = Color32::from_rgb(0xF2, 0x9E, 0x74);
pub const PROPERTY: Color32 = Color32::from_rgb(0xB4, 0xBC, 0xC8);
pub const PUNCT: Color32 = Color32::from_rgb(0x7A, 0x84, 0x93);

const KEYWORDS: [&str; 22] = [
    "const",
    "let",
    "var",
    "for",
    "of",
    "in",
    "if",
    "else",
    "return",
    "function",
    "true",
    "false",
    "null",
    "undefined",
    "new",
    "while",
    "break",
    "continue",
    "Math",
    "this",
    "typeof",
    "do",
];

/// Una línea de JavaScript como texto con colores. `base` es el color del
/// resto (los nombres).
pub fn highlight(line: &str, font: FontId, base: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    let chars: Vec<char> = line.chars().collect();
    let push = |job: &mut LayoutJob, text: &str, color: Color32, italic: bool| {
        job.append(
            text,
            0.0,
            TextFormat {
                font_id: font.clone(),
                color,
                italics: italic,
                ..Default::default()
            },
        );
    };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // Comentario hasta el final.
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            let rest: String = chars[i..].iter().collect();
            push(&mut job, &rest, theme::TEXT_4, true);
            break;
        }
        // Cadena.
        if c == '"' || c == '\'' || c == '`' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != c {
                if chars[j] == '\\' {
                    j += 1;
                }
                j += 1;
            }
            let end = (j + 1).min(chars.len());
            let text: String = chars[i..end].iter().collect();
            push(&mut job, &text, STRING, false);
            i = end;
            continue;
        }
        // Número.
        if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).is_some_and(char::is_ascii_digit)) {
            let mut j = i;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '.') {
                j += 1;
            }
            let text: String = chars[i..j].iter().collect();
            push(&mut job, &text, NUMBER, false);
            i = j;
            continue;
        }
        // Nombre: palabra clave, llamada, propiedad o variable.
        if c.is_alphabetic() || c == '_' || c == '$' {
            let mut j = i;
            while j < chars.len()
                && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '$')
            {
                j += 1;
            }
            let word: String = chars[i..j].iter().collect();
            let next = chars[j..].iter().find(|c| !c.is_whitespace()).copied();
            let color = if KEYWORDS.contains(&word.as_str()) {
                KEYWORD
            } else if next == Some('(') {
                FUNCTION
            } else if next == Some(':') {
                PROPERTY
            } else {
                base
            };
            push(&mut job, &word, color, false);
            i = j;
            continue;
        }
        // Espacios y signos.
        let mut j = i;
        while j < chars.len() {
            let d = chars[j];
            let starts = d.is_alphanumeric()
                || d == '_'
                || d == '$'
                || d == '"'
                || d == '\''
                || d == '`'
                || (d == '/' && chars.get(j + 1) == Some(&'/'));
            if starts && j > i {
                break;
            }
            if starts {
                break;
            }
            j += 1;
        }
        let j = j.max(i + 1);
        let text: String = chars[i..j].iter().collect();
        let color = if text.trim().is_empty() { base } else { PUNCT };
        push(&mut job, &text, color, false);
        i = j;
    }
    job
}
