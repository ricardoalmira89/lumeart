//! `batch`: varios comandos de una vez, con la salida de pxcli.
//!
//! Base: `internal/cli/batch.go`. En pxcli era la CLI la que enviaba cada línea
//! al daemon (cada una, un paso de deshacer). En LumeArt el `batch` se ejecuta
//! junto al documento dentro de una transacción del historial: todo el `batch`
//! es un solo paso de deshacer, pero un `undo` dentro del `batch` deshace solo
//! la línea anterior, como en pxcli.
//!
//! La salida es la misma: una línea por error o por respuesta con dato
//! (`line N: <petición> -> <respuesta>`) y al final `batch: N ok, M failed`.

use crate::canvas::Canvas;
use crate::error::DomainError;
use crate::history::{History, Origin};
use crate::protocol::{Executed, Response, handle_line, quote_field, split_fields};

/// Una línea útil del guion, con su número de línea original (desde 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchLine {
    pub num: usize,
    pub text: String,
}

/// Divide un guion en comandos: uno por línea o separados por `;`. Ignora las
/// líneas vacías y los comentarios (`#` o `//`). Como `splitBatch` de pxcli.
pub fn split_batch(script: &str) -> Vec<BatchLine> {
    let mut lines = Vec::new();
    for (i, raw) in script.split('\n').enumerate() {
        for part in raw.split(';') {
            let text = part.trim();
            if text.is_empty() || text.starts_with('#') || text.starts_with("//") {
                continue;
            }
            lines.push(BatchLine {
                num: i + 1,
                text: text.to_string(),
            });
        }
    }
    lines
}

/// Comandos que no tienen sentido dentro de un `batch` (como en pxcli).
const FORBIDDEN: &[&str] = &["start", "stop", "daemon", "batch", "studio"];

/// ¿Es el nombre del programa al principio de la línea (`pxcli`, `lumeart`,
/// `./pxcli`, `/usr/bin/lumeart`…)?
fn is_program_name(field: &str) -> bool {
    ["pxcli", "lumeart"]
        .iter()
        .any(|name| field == *name || field.ends_with(&format!("/{name}")))
}

/// Tolera lo que suelen escribir los agentes, como `normalizeBatchLine` de
/// pxcli: quita `pxcli`/`lumeart` al principio y las comillas de cada campo,
/// prohíbe `start`, `stop`, `daemon`, `batch` y `studio`, y convierte
/// `export [--scale N] <archivo>` en `export <ruta absoluta> [N]`.
///
/// `resolve_path` convierte una ruta relativa en absoluta respecto a la
/// carpeta de quien envía el `batch` (el dominio no conoce el sistema de
/// archivos).
pub fn normalize_batch_line(
    line: &str,
    resolve_path: &dyn Fn(&str) -> String,
) -> Result<String, DomainError> {
    let mut fields = split_fields(line, char::is_whitespace);
    if fields.first().is_some_and(|f| is_program_name(f)) {
        fields.remove(0);
    }
    if fields.is_empty() {
        return Err(DomainError::invalid_args("empty command"));
    }
    let fields: Vec<&str> = fields
        .iter()
        .map(|f| f.trim_matches(|c| c == '"' || c == '\''))
        .collect();
    if FORBIDDEN.contains(&fields[0]) {
        return Err(DomainError::invalid_args(format!(
            "{} is not allowed inside batch",
            fields[0]
        )));
    }
    if fields[0] != "export" {
        let quoted: Vec<String> = fields.iter().map(|f| quote_field(f)).collect();
        return Ok(quoted.join(" "));
    }

    let mut path = "";
    let mut scale = "";
    let mut i = 1;
    while i < fields.len() {
        let field = fields[i];
        if field == "--scale" && i + 1 < fields.len() {
            scale = fields[i + 1];
            i += 1;
        } else if let Some(value) = field.strip_prefix("--scale=") {
            scale = value;
        } else {
            path = field;
        }
        i += 1;
    }
    if path.is_empty() {
        return Err(DomainError::invalid_args("export needs a file name"));
    }
    let mut out = format!("export {}", quote_field(&resolve_path(path)));
    if !scale.is_empty() {
        out.push(' ');
        out.push_str(scale);
    }
    Ok(out)
}

/// Resultado de un `batch`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BatchReport {
    /// Líneas de salida antes del resumen (errores y respuestas con dato).
    pub lines: Vec<String>,
    pub ok: usize,
    pub failed: usize,
}

impl BatchReport {
    /// Número de comandos del guion.
    pub fn total(&self) -> usize {
        self.ok + self.failed
    }

    /// `batch: N ok, M failed`
    pub fn summary(&self) -> String {
        format!("batch: {} ok, {} failed", self.ok, self.failed)
    }

    /// Salida completa (líneas y resumen), cada una terminada en `\n`.
    pub fn output(&self) -> String {
        let mut out = String::new();
        for line in &self.lines {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str(&self.summary());
        out.push('\n');
        out
    }

    /// Si algo falló, el error final de pxcli:
    /// `err batch M of T commands failed`.
    pub fn error(&self) -> Option<String> {
        (self.failed > 0).then(|| {
            format!(
                "err batch {} of {} commands failed",
                self.failed,
                self.total()
            )
        })
    }
}

/// Exporta el lienzo a PNG (lo implementa quien tiene acceso a archivos).
pub type ExportFn<'a> = dyn FnMut(&Canvas, &str, u32) -> Result<(), DomainError> + 'a;

/// Ejecuta un guion como un solo paso de deshacer.
///
/// Error si el guion no tiene ningún comando (`batch is empty`, como pxcli) o
/// si ya había una transacción abierta.
pub fn run_batch(
    script: &str,
    canvas: &mut Canvas,
    history: &mut History,
    origin: Origin,
    resolve_path: &dyn Fn(&str) -> String,
    export: &mut ExportFn<'_>,
) -> Result<BatchReport, DomainError> {
    let lines = split_batch(script);
    if lines.is_empty() {
        return Err(DomainError::invalid_args("batch is empty"));
    }
    history.begin(origin, "batch")?;

    let mut report = BatchReport::default();
    for line in lines {
        let request = match normalize_batch_line(&line.text, resolve_path) {
            Ok(request) => request,
            Err(err) => {
                report.lines.push(format!(
                    "line {}: {} -> {}",
                    line.num,
                    line.text,
                    Response::Err(err)
                ));
                report.failed += 1;
                continue;
            }
        };
        // `new` borra el historial y con él la transacción: se abre otra.
        if !history.in_transaction() {
            history.begin(origin, "batch")?;
        }
        let response = match handle_line(&request, canvas, history, origin) {
            Executed::Reply(response) => response,
            Executed::Export { path, scale } => match export(canvas, &path, scale) {
                Ok(()) => Response::ok(),
                Err(err) => Response::Err(err),
            },
            // `stop` está prohibido dentro de un batch.
            Executed::Stop => Response::ok(),
        };
        match &response {
            Response::Ok(None) => report.ok += 1,
            Response::Ok(Some(_)) => {
                report
                    .lines
                    .push(format!("line {}: {} -> {}", line.num, request, response));
                report.ok += 1;
            }
            Response::Err(_) => {
                report
                    .lines
                    .push(format!("line {}: {} -> {}", line.num, request, response));
                report.failed += 1;
            }
        }
    }
    history.commit();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgba;
    use crate::error::ErrorCode;

    fn abs(path: &str) -> String {
        if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/work/{path}")
        }
    }

    // Portado de TestSplitBatchSkipsCommentsAndSplitsSemicolons.
    #[test]
    fn split_skips_comments_and_splits_semicolons() {
        let lines = split_batch(
            "\n# comment\nset_pixel 1 1 #fff; fill_rect 0 0 2 2 #000\n\n// note\nundo\n",
        );
        let texts: Vec<_> = lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["set_pixel 1 1 #fff", "fill_rect 0 0 2 2 #000", "undo"]
        );
        assert_eq!(lines[0].num, 3);
        assert_eq!(lines[2].num, 6);
    }

    // Portado de TestNormalizeBatchLine.
    #[test]
    fn normalize_batch_line_cases() {
        let cases = [
            (
                r##"pxcli set_pixel 1 2 "#ff0000""##,
                "set_pixel 1 2 #ff0000",
            ),
            ("./pxcli fill_rect 0 0 4 4 '#abc'", "fill_rect 0 0 4 4 #abc"),
            ("export --scale 10 out.png", "export /work/out.png 10"),
            ("export --scale=4 out.png", "export /work/out.png 4"),
            ("export out.png", "export /work/out.png"),
            ("clear transparent", "clear transparent"),
            ("lumeart set_pixel 0 0 red", "set_pixel 0 0 red"),
            ("/usr/local/bin/lumeart  undo", "undo"),
            ("export /abs/x.png", "export /abs/x.png"),
        ];
        for (input, want) in cases {
            assert_eq!(
                normalize_batch_line(input, &abs).unwrap(),
                want,
                "{input:?}"
            );
        }
        for bad in [
            "stop",
            "pxcli start",
            "batch 'x'",
            "export --scale 2",
            "pxcli",
            "lumeart studio",
        ] {
            let err = normalize_batch_line(bad, &abs).unwrap_err();
            assert_eq!(err.code, ErrorCode::InvalidArgs, "{bad:?}");
        }
        assert_eq!(
            normalize_batch_line("pxcli", &abs).unwrap_err().message,
            "empty command"
        );
        assert_eq!(
            normalize_batch_line("stop", &abs).unwrap_err().message,
            "stop is not allowed inside batch"
        );
    }

    #[test]
    fn quoted_paths_with_spaces() {
        let resolve = |p: &str| {
            if p.starts_with('/') {
                p.to_string()
            } else {
                format!("/mi juego/{p}")
            }
        };
        assert_eq!(
            normalize_batch_line(r#"export --scale 2 "arte final/a.png""#, &resolve).unwrap(),
            r#"export "/mi juego/arte final/a.png" 2"#
        );
        assert_eq!(
            normalize_batch_line("export a.png", &resolve).unwrap(),
            r#"export "/mi juego/a.png""#
        );
    }

    fn run(script: &str, canvas: &mut Canvas, history: &mut History) -> BatchReport {
        run_batch(
            script,
            canvas,
            history,
            Origin::Cli,
            &abs,
            &mut |_, _, _| Ok(()),
        )
        .unwrap()
    }

    #[test]
    fn errors_are_reported_and_valid_commands_applied() {
        let mut c = Canvas::new(4, 4).unwrap();
        let mut h = History::new();
        let report = run(
            "set_pixel 0 0 red\nset_pixel 9 9 red\nfill_rect 1 1 2 2 blue",
            &mut c,
            &mut h,
        );
        assert_eq!(
            report.lines,
            vec!["line 2: set_pixel 9 9 red -> err out_of_bounds pixel (9,9) outside canvas"]
        );
        assert_eq!(report.summary(), "batch: 2 ok, 1 failed");
        assert_eq!(report.error().unwrap(), "err batch 1 of 3 commands failed");
        assert_eq!(c.get(0, 0).unwrap(), Rgba::rgb(255, 0, 0));
        assert_eq!(c.get(2, 2).unwrap(), Rgba::rgb(0, 0, 255));
    }

    #[test]
    fn undo_after_batch_reverts_the_whole_batch() {
        let mut c = Canvas::new(4, 4).unwrap();
        let mut h = History::new();
        h.apply(
            &mut c,
            crate::Command::SetPixel {
                x: 3,
                y: 3,
                color: Rgba::rgb(1, 2, 3),
            },
            Origin::User,
        )
        .unwrap();
        let before = c.clone();
        let report = run(
            "fill_rect 0 0 2 2 red; line 0 3 3 0 blue; set_pixel 9 9 red; clear #00ff00",
            &mut c,
            &mut h,
        );
        assert_eq!((report.ok, report.failed), (3, 1));
        assert_eq!(h.entries().len(), 2);
        let entry = h.entries().last().unwrap();
        assert_eq!(entry.label, "batch");
        assert_eq!(entry.command_count(), 3);
        h.undo(&mut c).unwrap();
        assert_eq!(c, before);
    }

    #[test]
    fn empty_batch_is_an_error() {
        let mut c = Canvas::new(4, 4).unwrap();
        let mut h = History::new();
        let err = run_batch(
            "# nada\n ; \n",
            &mut c,
            &mut h,
            Origin::Cli,
            &abs,
            &mut |_, _, _| Ok(()),
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidArgs);
        assert_eq!(err.message, "batch is empty");
        assert!(!h.in_transaction());
    }

    #[test]
    fn export_goes_through_the_callback() {
        let mut c = Canvas::new(4, 4).unwrap();
        let mut h = History::new();
        let mut calls = Vec::new();
        let report = run_batch(
            "set_pixel 0 0 red\nexport --scale 3 a.png\nexport b.png",
            &mut c,
            &mut h,
            Origin::Cli,
            &abs,
            &mut |canvas, path, scale| {
                calls.push((canvas.get(0, 0).unwrap(), path.to_string(), scale));
                if path.ends_with("b.png") {
                    Err(DomainError::new(crate::ErrorCode::Io, "permission denied"))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap();
        let red = Rgba::rgb(255, 0, 0);
        assert_eq!(
            calls,
            vec![
                (red, "/work/a.png".into(), 3),
                (red, "/work/b.png".into(), 1)
            ]
        );
        assert_eq!(
            report.lines,
            vec!["line 3: export /work/b.png -> err io permission denied"]
        );
    }
}
