//! El crítico: otra ejecución del agente, con sus propias instrucciones y
//! herramientas de solo lectura, que mira el dibujo y devuelve un veredicto
//! JSON con el esquema de pxcli (`internal/web/critic.go`):
//!
//! ```json
//! {"aprobado": false, "problemas": [{"gravedad": "alta", "zona": [x, y, w, h],
//!   "problema": "…", "arreglo": "…"}]}
//! ```
//!
//! Si no hay JSON o no se entiende, el resultado es «sin veredicto»: nunca un
//! error que pare el bucle de refinado.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::ports::{
    AgentTask, AgentTools, BoxFuture, CancellationToken, EventSink, PreparedAgent, ToolBox,
    ToolOutput, ToolSpec,
};

/// Herramientas que puede usar el crítico (ninguna cambia el dibujo).
pub const CRITIC_TOOLS: [&str; 3] = ["view_canvas", "get_pixel", "palette"];

/// Un problema que encuentra el crítico.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    /// `alta`, `media` o `baja`.
    #[serde(rename = "gravedad", default)]
    pub severity: String,
    /// Zona en el lienzo: `[x, y, ancho, alto]`.
    #[serde(rename = "zona", default)]
    pub zone: Vec<f64>,
    #[serde(rename = "problema", default)]
    pub problem: String,
    #[serde(rename = "arreglo", default)]
    pub fix: String,
}

impl Issue {
    /// ¿Merece otra pasada del agente? (gravedad alta o media)
    pub fn is_serious(&self) -> bool {
        matches!(self.severity.to_lowercase().as_str(), "alta" | "media")
    }
}

/// El veredicto del crítico.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Critique {
    #[serde(rename = "aprobado", default)]
    pub approved: bool,
    #[serde(rename = "problemas", default)]
    pub issues: Vec<Issue>,
}

impl Critique {
    /// Problemas graves (altos y medios).
    pub fn serious(&self) -> Vec<&Issue> {
        self.issues.iter().filter(|i| i.is_serious()).collect()
    }

    /// Los problemas como lista numerada, como el `fixPrompt` de pxcli:
    /// «1. [alta] en la zona x 1, y 2, 3×4: … Arreglo propuesto: …».
    pub fn issues_text(issues: &[&Issue]) -> String {
        let mut out = String::new();
        for (i, issue) in issues.iter().enumerate() {
            let zone = match issue.zone.as_slice() {
                [x, y, w, h] => format!(" en la zona x {x:.0}, y {y:.0}, {w:.0}×{h:.0}"),
                _ => String::new(),
            };
            out.push_str(&format!(
                "{}. [{}]{zone}: {}",
                i + 1,
                issue.severity,
                issue.problem
            ));
            if !issue.fix.is_empty() {
                out.push_str(&format!(" Arreglo propuesto: {}", issue.fix));
            }
            out.push('\n');
        }
        out
    }
}

/// Lee el veredicto: el último bloque ```json que se entienda o, si no, el
/// último `{"aprobado"…}` del texto. `None` si no hay ninguno.
pub fn parse_critique(text: &str) -> Option<Critique> {
    let mut candidates: Vec<&str> = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find("```") {
        let after = &rest[open + 3..];
        let body_start = after.strip_prefix("json").map(|_| 4).unwrap_or(0);
        let Some(close) = after[body_start..].find("```") else {
            break;
        };
        let body = after[body_start..body_start + close].trim();
        if body.starts_with('{') && body.ends_with('}') {
            candidates.push(body);
        }
        rest = &after[body_start + close + 3..];
    }
    if let (Some(i), Some(j)) = (text.rfind("{\"aprobado\""), text.rfind('}'))
        && j > i
    {
        candidates.push(&text[i..=j]);
    }
    candidates
        .iter()
        .rev()
        .find_map(|c| serde_json::from_str::<Critique>(c).ok())
}

/// Resultado de una revisión.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Ready(Critique),
    /// El crítico no dio un veredicto que se entienda (o falló): el bucle
    /// sigue como si no hubiera crítico en esa vuelta.
    None {
        reason: String,
    },
}

/// Herramientas del crítico: las de solo lectura de otra `ToolBox`. Pedir
/// cualquier otra (`draw`, `undo`…) es un error.
pub struct ReadOnlyTools {
    inner: ToolBox,
}

impl ReadOnlyTools {
    pub fn new(inner: ToolBox) -> Self {
        Self { inner }
    }
}

impl AgentTools for ReadOnlyTools {
    fn specs(&self) -> Vec<ToolSpec> {
        self.inner
            .specs()
            .into_iter()
            .filter(|s| CRITIC_TOOLS.contains(&s.name.as_str()))
            .collect()
    }

    fn call<'a>(&'a self, name: &'a str, input_json: &'a str) -> BoxFuture<'a, ToolOutput> {
        if CRITIC_TOOLS.contains(&name) {
            self.inner.call(name, input_json)
        } else {
            Box::pin(async move {
                ToolOutput::error(format!(
                    "el crítico solo puede mirar el dibujo ({}); `{name}` no está disponible",
                    CRITIC_TOOLS.join(", ")
                ))
            })
        }
    }
}

/// Encargo del crítico para lo que se le pidió al artista: la misma petición,
/// lienzo, paleta y referencias, sin estado heredado y con límites cortos.
pub fn critic_task(artist: &AgentTask) -> AgentTask {
    let references = artist
        .attachments
        .iter()
        .filter(|a| a.label != crate::ports::CANVAS_ATTACHMENT)
        .cloned()
        .collect();
    AgentTask {
        attachments: references,
        resume: None,
        anim: None,
        vector: None,
        context: String::new(),
        continuing: false,
        limits: crate::ports::AgentLimits {
            max_steps: 8,
            max_tokens: 150_000,
        },
        ..artist.clone()
    }
}

/// Pasa la revisión: lanza el crítico ya preparado y lee su veredicto.
/// Nunca falla: un error del crítico o un JSON que no se entiende dan
/// [`Verdict::None`].
pub async fn review(
    critic: PreparedAgent,
    task: AgentTask,
    events: EventSink,
    cancel: CancellationToken,
) -> Verdict {
    let tools: ToolBox = Arc::new(ReadOnlyTools::new(critic.tools));
    match critic.runner.run(task, tools, events, cancel).await {
        Ok(summary) => match parse_critique(&summary.final_text) {
            Some(critique) => Verdict::Ready(critique),
            None => Verdict::None {
                reason: "el crítico no devolvió su veredicto en JSON".into(),
            },
        },
        Err(err) => Verdict::None {
            reason: format!("el crítico falló: {err}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El caso de `TestParseCritique` de pxcli.
    #[test]
    fn verdicts_are_read_like_pxcli() {
        let text = "Mira, el tejado...\n```json\n{\"aprobado\": false, \"problemas\": [{\"gravedad\": \"alta\", \"zona\": [1, 2, 3, 4], \"problema\": \"p\", \"arreglo\": \"a\"}, {\"gravedad\": \"baja\", \"problema\": \"q\"}]}\n```";
        let c = parse_critique(text).unwrap();
        assert!(!c.approved);
        assert_eq!(c.issues.len(), 2);
        let serious = c.serious();
        assert_eq!(serious.len(), 1);
        assert_eq!(serious[0].zone[3], 4.0);
        assert_eq!(
            Critique::issues_text(&serious),
            "1. [alta] en la zona x 1, y 2, 3×4: p Arreglo propuesto: a\n"
        );
        assert_eq!(parse_critique("sin veredicto"), None);
    }

    #[test]
    fn the_last_readable_verdict_wins() {
        // Dos bloques: vale el último; uno roto se salta.
        let text = "```json\n{\"aprobado\": false, \"problemas\": []}\n```\nY al final:\n```json\n{\"aprobado\": true, \"problemas\": []}\n```";
        assert!(parse_critique(text).unwrap().approved);
        let text = "```json\n{\"aprobado\": true, \"problemas\": []}\n```\n```json\n{roto}\n```";
        assert!(parse_critique(text).unwrap().approved);
        // Sin bloque: el último {"aprobado"…} del texto.
        let text = "Veredicto: {\"aprobado\": true, \"problemas\": []} y nada más.";
        assert!(parse_critique(text).unwrap().approved);
        // Tipos equivocados: no es un veredicto.
        assert_eq!(parse_critique("{\"aprobado\": \"sí\"}"), None);
        assert_eq!(parse_critique("```json\n{\"problemas\": 3}\n```"), None);
        // Gravedad sin distinguir mayúsculas.
        let c = parse_critique("{\"aprobado\": false, \"problemas\": [{\"gravedad\": \"Media\", \"problema\": \"x\"}]}").unwrap();
        assert_eq!(c.serious().len(), 1);
    }
}
