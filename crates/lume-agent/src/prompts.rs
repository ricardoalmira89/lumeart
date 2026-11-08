//! Instrucciones del agente. Son datos: viven en `crates/lume-agent/prompts/`
//! y se embeben con `include_str!`. Base: `internal/web/prompt.go` de pxcli,
//! adaptado a las herramientas de [`SessionTools`](crate::SessionTools).
//!
//! Los huecos se escriben `{nombre}` y se rellenan en una sola pasada: lo que
//! se mete (la petición del usuario, por ejemplo) no se vuelve a interpretar.

use lume_app::ports::{AgentTask, CANVAS_ATTACHMENT};
use lume_domain::{Palette, Rgba};

/// Instrucciones de sistema de pixel art: `{width}`, `{height}`, `{max_x}`,
/// `{max_y}`.
pub const PIXEL_SYSTEM: &str = include_str!("../prompts/pixel_system.md");
/// Petición y tamaño: `{request}`, `{width}`, `{height}`.
pub const PIXEL_REQUEST: &str = include_str!("../prompts/pixel_request.md");
/// Petición cuando solo hay imagen de referencia.
pub const PIXEL_REQUEST_REFERENCE: &str = include_str!("../prompts/pixel_request_reference.md");
/// Seguir sobre un dibujo que ya existe.
pub const PIXEL_CONTINUE: &str = include_str!("../prompts/pixel_continue.md");
/// Paleta fija: `{name}`, `{colors}`.
pub const PIXEL_PALETTE: &str = include_str!("../prompts/pixel_palette.md");
/// Hay imagen de referencia adjunta.
pub const PIXEL_REFERENCE: &str = include_str!("../prompts/pixel_reference.md");
/// El lienzo de la vuelta va adjunto.
pub const PIXEL_CANVAS: &str = include_str!("../prompts/pixel_canvas.md");

/// Instrucciones de sistema para Claude Code: `{width}`, `{height}`,
/// `{max_x}`, `{max_y}`.
pub const CLAUDE_CODE_SYSTEM: &str = include_str!("../prompts/claude_code_system.md");
/// Seguir sobre un dibujo que ya existe (Claude Code).
pub const CLAUDE_CODE_CONTINUE: &str = include_str!("../prompts/claude_code_continue.md");
/// Una imagen de referencia en la carpeta de trabajo: `{file}`.
pub const CLAUDE_CODE_REFERENCE: &str = include_str!("../prompts/claude_code_reference.md");
/// El lienzo de la vuelta en la carpeta de trabajo: `{file}`.
pub const CLAUDE_CODE_CANVAS: &str = include_str!("../prompts/claude_code_canvas.md");

/// Instrucciones de sistema de un encargo vectorial (sin herramientas: valen
/// para los dos runners).
pub const VEC_SYSTEM: &str = include_str!("../prompts/vec_system.md");
/// Seguir sobre el dibujo vectorial que ya hay (vale para los dos runners).
pub const VEC_CONTINUE: &str = include_str!("../prompts/vec_continue.md");
/// Cómo se cambia el dibujo vectorial con las herramientas de la `ToolBox`.
pub const VEC_API: &str = include_str!("../prompts/vec_api.md");
/// Cómo se cambia el dibujo vectorial con Claude Code (./dibujo.js y
/// `lumeart vec`).
pub const CLAUDE_CODE_VEC: &str = include_str!("../prompts/claude_code_vec.md");

/// Cómo se cambia la animación con las herramientas de la `ToolBox`.
pub const ANIM_API: &str = include_str!("../prompts/anim_api.md");
/// Cómo se cambia la animación con Claude Code (./anim.js y `lumeart anim`).
pub const CLAUDE_CODE_ANIM: &str = include_str!("../prompts/claude_code_anim.md");

/// Instrucciones del crítico (sin nombres de herramientas: valen para los
/// dos runners).
pub const CRITIC_SYSTEM: &str = include_str!("../prompts/critic_system.md");
/// Encargo del crítico: `{width}`, `{height}`, `{request}`.
pub const CRITIC_TASK: &str = include_str!("../prompts/critic_task.md");
/// Cómo mira el dibujo el crítico con las herramientas de la `ToolBox`.
pub const CRITIC_TASK_API: &str = include_str!("../prompts/critic_task_api.md");
/// Cómo mira el dibujo el crítico con Claude Code: `{file}`.
pub const CRITIC_TASK_CLAUDE_CODE: &str = include_str!("../prompts/critic_task_claude_code.md");

/// Los prompts de Claude Code, con su archivo (para los tests).
pub const CLAUDE_CODE_ALL: [(&str, &str); 10] = [
    ("claude_code_system.md", CLAUDE_CODE_SYSTEM),
    ("claude_code_vec.md", CLAUDE_CODE_VEC),
    ("vec_continue.md", VEC_CONTINUE),
    ("claude_code_anim.md", CLAUDE_CODE_ANIM),
    ("claude_code_canvas.md", CLAUDE_CODE_CANVAS),
    ("claude_code_continue.md", CLAUDE_CODE_CONTINUE),
    ("claude_code_reference.md", CLAUDE_CODE_REFERENCE),
    ("critic_system.md", CRITIC_SYSTEM),
    ("critic_task.md", CRITIC_TASK),
    ("critic_task_claude_code.md", CRITIC_TASK_CLAUDE_CODE),
];

/// Los prompts del `ApiRunner`, con su archivo (para los tests).
pub const ALL: [(&str, &str); 13] = [
    ("pixel_canvas.md", PIXEL_CANVAS),
    ("vec_api.md", VEC_API),
    ("vec_continue.md", VEC_CONTINUE),
    ("anim_api.md", ANIM_API),
    ("critic_system.md", CRITIC_SYSTEM),
    ("critic_task.md", CRITIC_TASK),
    ("critic_task_api.md", CRITIC_TASK_API),
    ("pixel_system.md", PIXEL_SYSTEM),
    ("pixel_request.md", PIXEL_REQUEST),
    ("pixel_request_reference.md", PIXEL_REQUEST_REFERENCE),
    ("pixel_continue.md", PIXEL_CONTINUE),
    ("pixel_palette.md", PIXEL_PALETTE),
    ("pixel_reference.md", PIXEL_REFERENCE),
];

/// Rellena los huecos `{nombre}` de `template` con `values`. Un hueco sin
/// valor se deja tal cual (lo detectan los tests).
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let key_end = after.find('}');
        let value = key_end.and_then(|end| {
            let key = &after[..end];
            values
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v, end))
        });
        match value {
            Some((value, end)) => {
                out.push_str(value);
                rest = &after[end + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Instrucciones de sistema para un encargo de pixel art.
pub fn pixel_system(task: &AgentTask) -> String {
    if task.vector.is_some() {
        return VEC_SYSTEM.trim_end().to_string();
    }
    let (w, h) = task.canvas;
    let values = [
        ("width", w.to_string()),
        ("height", h.to_string()),
        ("max_x", w.saturating_sub(1).to_string()),
        ("max_y", h.saturating_sub(1).to_string()),
    ];
    let refs: Vec<(&str, &str)> = values.iter().map(|(k, v)| (*k, v.as_str())).collect();
    fill(PIXEL_SYSTEM, &refs).trim_end().to_string()
}

/// Primer mensaje del encargo: seguir (si toca), petición y tamaño, paleta
/// fija, aviso de la referencia y el estado heredado (`context`).
pub fn pixel_task(task: &AgentTask) -> String {
    let mut lines = Vec::new();
    if has_references(task) {
        lines.push(PIXEL_REFERENCE.trim().to_string());
    }
    if has_canvas(task) {
        lines.push(PIXEL_CANVAS.trim().to_string());
    }
    if task.anim.is_some() {
        lines.push(ANIM_API.trim().to_string());
    }
    if task.vector.is_some() {
        lines.push(VEC_API.trim().to_string());
    }
    task_text(task, PIXEL_CONTINUE, lines)
}

/// ¿Lleva el encargo imágenes de referencia (además del lienzo)?
pub fn has_references(task: &AgentTask) -> bool {
    task.attachments
        .iter()
        .any(|a| a.label != CANVAS_ATTACHMENT)
}

/// ¿Lleva el encargo el lienzo de la vuelta?
pub fn has_canvas(task: &AgentTask) -> bool {
    task.attachments
        .iter()
        .any(|a| a.label == CANVAS_ATTACHMENT)
}

/// Se añade a las instrucciones de Claude Code cuando tiene acceso completo
/// (Ajustes › Claude Code): las de siempre dicen que solo use `lumeart`.
pub const FULL_ACCESS_NOTE: &str = "\n\n## Acceso completo\nEl usuario te ha dado acceso completo: puedes usar cualquier herramienta, programa o archivo del sistema sin pedir permiso, además de lumeart. Tu carpeta de trabajo sigue siendo la actual y el dibujo sigue siendo el de este encargo; úsalo para lo que lumeart no cubra (buscar un archivo, mirar una imagen de fuera, procesar algo con otra herramienta).\n";

/// Instrucciones de sistema para Claude Code (`claude -p`), que dibuja con
/// `lumeart batch` y revisa con `lumeart export` y `Read`.
pub fn claude_code_system(task: &AgentTask) -> String {
    if task.vector.is_some() {
        return VEC_SYSTEM.trim_end().to_string();
    }
    let (w, h) = task.canvas;
    let values = [
        ("width", w.to_string()),
        ("height", h.to_string()),
        ("max_x", w.saturating_sub(1).to_string()),
        ("max_y", h.saturating_sub(1).to_string()),
    ];
    let refs: Vec<(&str, &str)> = values.iter().map(|(k, v)| (*k, v.as_str())).collect();
    fill(CLAUDE_CODE_SYSTEM, &refs).trim_end().to_string()
}

/// Encargo para Claude Code. `references` son los nombres de las imágenes
/// de referencia y `canvas` el del lienzo de la vuelta, ya guardados en su
/// carpeta de trabajo.
pub fn claude_code_task(task: &AgentTask, references: &[String], canvas: Option<&str>) -> String {
    let mut lines: Vec<String> = references
        .iter()
        .map(|file| {
            fill(CLAUDE_CODE_REFERENCE, &[("file", file)])
                .trim()
                .to_string()
        })
        .collect();
    if let Some(file) = canvas {
        lines.push(
            fill(CLAUDE_CODE_CANVAS, &[("file", file)])
                .trim()
                .to_string(),
        );
    }
    if task.anim.is_some() {
        lines.push(CLAUDE_CODE_ANIM.trim().to_string());
    }
    if task.vector.is_some() {
        lines.push(CLAUDE_CODE_VEC.trim().to_string());
    }
    task_text(task, CLAUDE_CODE_CONTINUE, lines)
}

/// Instrucciones de sistema del crítico.
pub fn critic_system() -> String {
    CRITIC_SYSTEM.trim_end().to_string()
}

/// Encargo del crítico con la `ToolBox` (mira con `view_canvas`).
pub fn critic_task_api(task: &AgentTask) -> String {
    let reference = has_references(task).then(|| PIXEL_REFERENCE.trim().to_string());
    critic_text(
        task,
        CRITIC_TASK_API.trim().to_string(),
        reference.into_iter().collect(),
    )
}

/// Encargo del crítico con Claude Code: el dibujo en `drawing` y las
/// referencias en `references`, en su carpeta de trabajo.
pub fn critic_task_claude_code(task: &AgentTask, drawing: &str, references: &[String]) -> String {
    let look = fill(CRITIC_TASK_CLAUDE_CODE, &[("file", drawing)])
        .trim()
        .to_string();
    let refs = references
        .iter()
        .map(|file| {
            fill(CLAUDE_CODE_REFERENCE, &[("file", file)])
                .trim()
                .to_string()
        })
        .collect();
    critic_text(task, look, refs)
}

fn critic_text(task: &AgentTask, look: String, references: Vec<String>) -> String {
    let (w, h) = task.canvas;
    let mut parts = vec![
        fill(
            CRITIC_TASK,
            &[
                ("width", &w.to_string()),
                ("height", &h.to_string()),
                ("request", task.goal.trim()),
            ],
        )
        .trim()
        .to_string(),
        look,
    ];
    if let Some(palette) = task.palette.as_ref().filter(|p| !p.is_empty()) {
        parts.push(palette_line(palette));
    }
    parts.extend(references);
    if !task.context.trim().is_empty() {
        parts.push(task.context.trim().to_string());
    }
    parts.join("\n\n")
}

/// Seguir (si toca), petición y tamaño, paleta fija, referencias y el estado
/// heredado (`context`).
fn task_text(task: &AgentTask, continue_text: &str, references: Vec<String>) -> String {
    let (w, h) = task.canvas;
    let goal = task.goal.trim();
    let request = if goal.is_empty() && has_references(task) {
        PIXEL_REQUEST_REFERENCE.trim().to_string()
    } else {
        goal.to_string()
    };
    let mut parts = Vec::new();
    if task.continuing {
        // En vector, el dibujo es dibujo.js (no el lienzo de píxeles).
        let text = if task.vector.is_some() {
            VEC_CONTINUE
        } else {
            continue_text
        };
        parts.push(text.trim().to_string());
    }
    if task.vector.is_some() {
        // El tamaño del dibujo vectorial lo dice su canvas().
        parts.push(format!("Petición: {request}"));
    } else {
        parts.push(
            fill(
                PIXEL_REQUEST,
                &[
                    ("request", &request),
                    ("width", &w.to_string()),
                    ("height", &h.to_string()),
                ],
            )
            .trim()
            .to_string(),
        );
    }
    if let Some(palette) = task.palette.as_ref().filter(|p| !p.is_empty()) {
        parts.push(palette_line(palette));
    }
    parts.extend(references);
    if !task.context.trim().is_empty() {
        parts.push(task.context.trim().to_string());
    }
    parts.join("\n\n")
}

fn palette_line(palette: &Palette) -> String {
    let colors: Vec<String> = palette.colors().iter().map(|c| hex(c.color)).collect();
    let name = palette.name.as_deref().unwrap_or("del dibujo");
    fill(
        PIXEL_PALETTE,
        &[("name", name), ("colors", &colors.join(", "))],
    )
    .trim()
    .to_string()
}

fn hex(c: Rgba) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        c.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_app::ports::{AgentLimits, Attachment};

    fn task() -> AgentTask {
        AgentTask {
            run: 1,
            goal: "un cofre del tesoro".into(),
            context: String::new(),
            attachments: vec![],
            model: "m".into(),
            limits: AgentLimits::default(),
            canvas: (32, 24),
            palette: None,
            continuing: false,
            resume: None,
            anim: None,
            vector: None,
        }
    }

    /// Palabras entre comillas invertidas que parecen nombres de herramienta.
    fn backticked(text: &str) -> Vec<String> {
        // Las vallas de código (```json) no son nombres.
        let text = text.replace("```", "");
        text.split('`')
            .skip(1)
            .step_by(2)
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
            .map(str::to_string)
            .collect()
    }

    /// Cada prompt nombra solo herramientas que existen en la `ToolBox`, y
    /// ninguna de Claude Code ni de la CLI.
    #[test]
    fn prompts_only_mention_existing_tools() {
        let tools: Vec<String> = crate::tools::specs()
            .into_iter()
            .chain(crate::tools::anim_specs())
            .chain(crate::tools::vec_specs())
            .map(|s| s.name)
            .collect();
        for (file, text) in ALL {
            for name in backticked(text) {
                assert!(
                    tools.contains(&name),
                    "{file} nombra `{name}`, que no es una herramienta ({tools:?})"
                );
            }
            let words: Vec<&str> = text
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .collect();
            for foreign in [
                "Bash", "Read", "Edit", "Write", "Glob", "Grep", "pxcli", "lumeart", "export",
            ] {
                assert!(
                    !words.contains(&foreign),
                    "{file} menciona «{foreign}», que el agente no tiene"
                );
            }
        }
    }

    #[test]
    fn the_system_prompt_presents_every_tool() {
        for tool in crate::tools::specs() {
            assert!(
                PIXEL_SYSTEM.contains(&format!("`{}`", tool.name)),
                "pixel_system.md no explica `{}`",
                tool.name
            );
        }
    }

    #[test]
    fn the_system_prompt_is_filled_for_the_canvas() {
        let system = pixel_system(&task());
        assert!(system.contains("Tamaño 32x24."), "{system}");
        assert!(system.contains("x de 0 a 31, y de 0 a 23"), "{system}");
        assert!(!system.contains('{') && !system.contains('}'), "{system}");
        assert!(system.contains("español neutro, tuteando"));
    }

    #[test]
    fn task_messages_cover_each_case() {
        assert_eq!(
            pixel_task(&task()),
            "Petición: un cofre del tesoro\n\nLienzo: 32x24 píxeles."
        );

        let mut t = task();
        t.goal = String::new();
        t.attachments = vec![Attachment {
            label: "Imagen de referencia".into(),
            media_type: "image/png".into(),
            data: vec![1, 2, 3],
        }];
        t.continuing = true;
        t.palette = Palette::fixed("game-boy");
        t.context = "Notas de la vuelta anterior: falta el contorno.".into();
        let text = pixel_task(&t);
        let expected_start = "Sigue trabajando sobre el dibujo actual";
        assert!(text.starts_with(expected_start), "{text}");
        assert!(
            text.contains(
                "Petición: dibuja en pixel art lo que aparece en la imagen de referencia."
            ),
            "{text}"
        );
        assert!(
            text.contains("Paleta fija «game-boy»: usa solo estos colores: #"),
            "{text}"
        );
        assert!(text.contains("Imagen de referencia: va adjunta"), "{text}");
        assert!(
            text.ends_with("Notas de la vuelta anterior: falta el contorno."),
            "{text}"
        );
        assert!(!text.contains("{"), "{text}");
    }

    /// Los prompts de Claude Code solo nombran sus herramientas (Bash y
    /// Read; con animación o dibujo vectorial, también Edit y Write, que
    /// entonces tiene) y comandos de lumeart que existen; nada de la
    /// `ToolBox` ni de pxcli.
    #[test]
    fn claude_code_prompts_only_mention_its_tools() {
        const COMMANDS: [&str; 14] = [
            "anim",
            "vec",
            "batch",
            "export",
            "get_pixel",
            "set_pixel",
            "fill_rect",
            "line",
            "clear",
            "undo",
            "redo",
            "start",
            "stop",
            "new",
        ];
        let toolbox: Vec<String> = crate::tools::specs()
            .into_iter()
            .chain(crate::tools::anim_specs())
            .chain(crate::tools::vec_specs())
            .map(|s| s.name)
            .collect();
        for (file, text) in CLAUDE_CODE_ALL {
            let words: Vec<&str> = text
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .filter(|w| !w.is_empty())
                .collect();
            // Con anim.js o dibujo.js, Claude Code edita ese archivo
            // (`--tools Bash,Read,Write,Edit`).
            let edits_files = matches!(file, "claude_code_vec.md" | "claude_code_anim.md");
            for foreign in [
                "pxcli",
                "Edit",
                "Write",
                "Glob",
                "Grep",
                "WebFetch",
                "WebSearch",
                "Task",
            ] {
                if edits_files && matches!(foreign, "Edit" | "Write") {
                    continue;
                }
                assert!(!words.contains(&foreign), "{file} menciona «{foreign}»");
            }
            for name in &toolbox {
                assert!(
                    !text.contains(&format!("`{name}`")),
                    "{file} nombra `{name}`"
                );
            }
            // Cada «lumeart <comando>» del texto es un comando que existe.
            for (i, _) in text.match_indices("lumeart ") {
                let next: String = text[i + "lumeart ".len()..]
                    .chars()
                    .take_while(|c| c.is_ascii_lowercase() || *c == '_')
                    .collect();
                assert!(
                    COMMANDS.contains(&next.as_str()),
                    "{file}: «lumeart {next}» no es un comando"
                );
            }
        }
        assert!(CLAUDE_CODE_SYSTEM.contains("lumeart batch"));
        assert!(CLAUDE_CODE_SYSTEM.contains("con Read"));
    }

    #[test]
    fn claude_code_task_lists_the_reference_files() {
        let mut t = task();
        t.goal = String::new();
        t.attachments = vec![Attachment {
            label: "Imagen de referencia".into(),
            media_type: "image/png".into(),
            data: vec![1],
        }];
        t.continuing = true;
        let text = claude_code_task(&t, &["referencia-1.png".to_string()], None);
        assert!(
            text.starts_with("Sigue trabajando sobre el dibujo actual"),
            "{text}"
        );
        assert!(
            text.contains("lumeart export --scale 10 actual.png"),
            "{text}"
        );
        assert!(text.contains("dibuja en pixel art lo que aparece en la imagen de referencia."));
        assert!(
            text.ends_with(
                "Imagen de referencia: ./referencia-1.png (ábrela con Read antes de empezar)."
            ),
            "{text}"
        );
        let system = claude_code_system(&t);
        assert!(
            system.contains("Tamaño 32x24.") && system.contains("x de 0 a 31"),
            "{system}"
        );
        assert!(!system.contains('{'));
    }

    /// Un encargo de animación dice cómo cambiarla con las herramientas de
    /// cada runner; el encargo en sí no nombra ninguna.
    #[test]
    fn animation_tasks_say_how_to_change_anim_js() {
        let mut t = task();
        t.goal = lume_app::anim_goal("más rebote");
        t.anim = Some(lume_app::ports::AnimTask {
            source: String::new(),
            base_png: Vec::new(),
        });
        assert!(!t.goal.contains('`'), "{}", t.goal);
        let api = pixel_task(&t);
        assert!(
            api.contains("`write_anim`") && api.contains("`check_anim`"),
            "{api}"
        );
        assert!(!api.contains("lumeart"), "{api}");
        let code = claude_code_task(&t, &[], None);
        assert!(code.contains("`lumeart anim check anim.js`"), "{code}");
        assert!(!code.contains("write_anim"), "{code}");
        // Sin animación, nada de esto.
        assert!(!pixel_task(&task()).contains("anim"));
        assert!(!claude_code_task(&task(), &[], None).contains("anim"));
    }

    /// Un encargo vectorial usa las instrucciones de ilustración vectorial
    /// (sin herramientas) y cada runner dice cómo cambiar dibujo.js.
    #[test]
    fn vector_tasks_get_the_vector_prompts() {
        let mut t = task();
        t.vector = Some(lume_app::ports::VecTask {
            source: "canvas({ width: 400, height: 300 })\n".into(),
        });
        let system = pixel_system(&t);
        assert!(
            system.starts_with("Eres un ilustrador vectorial"),
            "{system}"
        );
        assert_eq!(claude_code_system(&t), system);
        for word in [
            "pxcli",
            "lumeart",
            "Bash",
            "Read",
            "Edit",
            "Write",
            "read_vec",
            "write_vec",
        ] {
            assert!(!system.contains(word), "vec_system.md nombra {word}");
        }
        let api = pixel_task(&t);
        assert!(api.starts_with("Petición: un cofre del tesoro"), "{api}");
        assert!(!api.contains("Lienzo: 32x24"), "{api}");
        assert!(api.contains("`write_vec`"), "{api}");
        let code = claude_code_task(&t, &[], None);
        assert!(
            code.contains("`lumeart vec render dibujo.js -o revision.png`"),
            "{code}"
        );
        assert!(!code.contains("write_vec"), "{code}");
        assert!(!pixel_task(&task()).contains("dibujo.js"));
        // Siguiendo sobre el dibujo: el texto vectorial, no el del lienzo.
        t.continuing = true;
        for text in [pixel_task(&t), claude_code_task(&t, &[], None)] {
            assert!(text.contains(VEC_CONTINUE.trim()), "{text}");
            assert!(!text.contains("lumeart export"), "{text}");
            assert!(!text.contains("view_canvas"), "{text}");
        }
    }

    /// Lo que escribe el usuario no se toma por un hueco.
    #[test]
    fn user_text_is_not_reinterpreted() {
        let mut t = task();
        t.goal = "un cartel que diga {width} y {request}".into();
        assert_eq!(
            pixel_task(&t),
            "Petición: un cartel que diga {width} y {request}\n\nLienzo: 32x24 píxeles."
        );
        assert_eq!(fill("a {x} {y} {", &[("x", "1")]), "a 1 {y} {");
    }
}
