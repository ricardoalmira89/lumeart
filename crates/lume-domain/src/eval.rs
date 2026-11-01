//! `lumeart eval`: lo que no hace E/S. Las tres variantes del bucle, las
//! métricas de cada ejecución, el resumen por variante, la nota del crítico,
//! el informe y la comparación ciega por parejas.
//!
//! La comparación ciega no dice nunca qué variante hay detrás de cada
//! imagen: las ejecuciones se nombran con una clave neutra (`k3f9a2`), los
//! pares van barajados y cada imagen cae a un lado al azar. Solo al final,
//! con todas las respuestas, [`reveal`] cuenta qué era cada cosa.

use serde::{Deserialize, Serialize};

use crate::canvas::Canvas;
use crate::color::Rgba;

/// Una forma de llevar el bucle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Variant {
    /// El plugin oficial de Claude Code (`ralph-loop`): un solo `claude -p`
    /// cuyo Stop hook le devuelve el mismo encargo hasta la promesa.
    A,
    /// El `RefineLoop` de LumeArt con el contexto vacío en cada vuelta.
    B,
    /// El `RefineLoop` retomando la misma conversación.
    C,
}

impl Variant {
    pub const ALL: [Variant; 3] = [Variant::A, Variant::B, Variant::C];

    pub fn letter(self) -> char {
        match self {
            Variant::A => 'A',
            Variant::B => 'B',
            Variant::C => 'C',
        }
    }

    /// «plugin oficial», «contexto vacío», «misma conversación».
    pub fn name(self) -> &'static str {
        match self {
            Variant::A => "plugin oficial",
            Variant::B => "contexto vacío",
            Variant::C => "misma conversación",
        }
    }

    /// «A (plugin oficial)».
    pub fn label(self) -> String {
        format!("{} ({})", self.letter(), self.name())
    }

    /// `A,B,C` o `ab`: las variantes, sin repetir y en orden.
    pub fn parse_list(raw: &str) -> Result<Vec<Variant>, String> {
        let mut out = Vec::new();
        for c in raw.chars().filter(|c| !matches!(c, ',' | ' ')) {
            let v = match c.to_ascii_uppercase() {
                'A' => Variant::A,
                'B' => Variant::B,
                'C' => Variant::C,
                _ => return Err(format!("variante desconocida: {c} (son A, B y C)")),
            };
            if !out.contains(&v) {
                out.push(v);
            }
        }
        if out.is_empty() {
            return Err("no hay variantes: usa A, B, C o una lista como A,B".into());
        }
        out.sort();
        Ok(out)
    }
}

/// Un encargo del conjunto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalTask {
    /// Nombre corto y único (`seta`, `brujula`).
    pub id: String,
    pub prompt: String,
    pub width: u32,
    pub height: u32,
    /// De qué ejecución de `reference/runs/` sale.
    #[serde(default)]
    pub origin: String,
}

/// Comprueba un conjunto de encargos: no vacío, ids únicos y tamaños
/// válidos.
pub fn check_tasks(tasks: &[EvalTask]) -> Result<(), String> {
    if tasks.is_empty() {
        return Err("el conjunto no tiene encargos".into());
    }
    for (i, t) in tasks.iter().enumerate() {
        if t.id.trim().is_empty() || t.prompt.trim().is_empty() {
            return Err(format!("el encargo {} no tiene id o petición", i + 1));
        }
        if !(1..=crate::MAX_CANVAS_SIZE).contains(&t.width)
            || !(1..=crate::MAX_CANVAS_SIZE).contains(&t.height)
        {
            return Err(format!(
                "el encargo {} tiene un tamaño no válido: {}x{}",
                t.id, t.width, t.height
            ));
        }
        if tasks[..i].iter().any(|o| o.id == t.id) {
            return Err(format!("el encargo {} está repetido", t.id));
        }
    }
    Ok(())
}

/// Lo que dio una ejecución: un encargo hecho por una variante.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunMetrics {
    pub task: String,
    pub variant: Variant,
    /// Clave neutra de la ejecución (nombra su imagen en la comparación).
    pub key: String,
    /// Nota del crítico sobre el resultado (de 0 a 10), si dio veredicto.
    pub critic: Option<f64>,
    /// Lo aprobó el crítico.
    pub approved: Option<bool>,
    /// Problemas que vio el crítico, por gravedad: alta, media y baja.
    #[serde(default)]
    pub issues: [u32; 3],
    /// El agente lo dio por bueno y pasan las comprobaciones objetivas.
    pub passed: bool,
    /// Vueltas del bucle.
    pub turns: u32,
    /// Tokens de entrada y salida (sin la caché).
    pub tokens: u64,
    /// Tokens leídos de la caché.
    #[serde(default)]
    pub cached_tokens: u64,
    /// Segundos que tardó (sin contar al crítico).
    pub seconds: f64,
    /// Cómo terminó, en una frase.
    pub end: String,
    /// Si falló antes de dibujar nada que mirar.
    #[serde(default)]
    pub error: Option<String>,
}

/// Nota del crítico: 10 si no ve nada, y resta 3 por cada problema alto,
/// 1,5 por cada medio y 0,5 por cada bajo (nunca menos de 0).
pub fn critic_score(issues: [u32; 3]) -> f64 {
    let [high, medium, low] = issues.map(f64::from);
    (10.0 - 3.0 * high - 1.5 * medium - 0.5 * low).max(0.0)
}

/// Cuenta las gravedades (`alta`, `media`, `baja`; lo demás cuenta como
/// baja).
pub fn count_issues<'a>(severities: impl IntoIterator<Item = &'a str>) -> [u32; 3] {
    let mut out = [0u32; 3];
    for s in severities {
        match s.trim().to_lowercase().as_str() {
            "alta" => out[0] += 1,
            "media" => out[1] += 1,
            _ => out[2] += 1,
        }
    }
    out
}

/// Las métricas de una variante sobre todos sus encargos.
#[derive(Debug, Clone, PartialEq)]
pub struct VariantSummary {
    pub variant: Variant,
    pub runs: usize,
    /// Media de las notas del crítico (de las ejecuciones que la tienen).
    pub critic: Option<f64>,
    /// Ejecuciones con nota.
    pub critic_runs: usize,
    /// Porcentaje que pasa las comprobaciones.
    pub passed_pct: f64,
    pub turns: f64,
    pub tokens: f64,
    pub seconds: f64,
    /// Ejecuciones que fallaron.
    pub failed: usize,
}

fn mean(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let (sum, n) = values
        .into_iter()
        .fold((0.0, 0usize), |(s, n), v| (s + v, n + 1));
    (n > 0).then(|| sum / n as f64)
}

/// Resumen por variante, en el orden A, B, C (solo las que hay).
pub fn summarize(runs: &[RunMetrics]) -> Vec<VariantSummary> {
    Variant::ALL
        .into_iter()
        .filter_map(|variant| {
            let mine: Vec<&RunMetrics> = runs.iter().filter(|r| r.variant == variant).collect();
            if mine.is_empty() {
                return None;
            }
            let n = mine.len();
            Some(VariantSummary {
                variant,
                runs: n,
                critic: mean(mine.iter().filter_map(|r| r.critic)),
                critic_runs: mine.iter().filter(|r| r.critic.is_some()).count(),
                passed_pct: 100.0 * mine.iter().filter(|r| r.passed).count() as f64 / n as f64,
                turns: mean(mine.iter().map(|r| f64::from(r.turns))).unwrap_or(0.0),
                tokens: mean(mine.iter().map(|r| r.tokens as f64)).unwrap_or(0.0),
                seconds: mean(mine.iter().map(|r| r.seconds)).unwrap_or(0.0),
                failed: mine.iter().filter(|r| r.error.is_some()).count(),
            })
        })
        .collect()
}

/// Generador pseudoaleatorio con semilla (SplitMix64): la misma semilla da
/// siempre las mismas claves y el mismo orden.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Un número de `0..n`.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n.max(1) as u64) as usize
    }

    /// Baraja `items` (Fisher–Yates).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }

    /// Una clave neutra de 6 caracteres (`k3f9a2`).
    pub fn key(&mut self) -> String {
        format!("k{:05x}", self.next_u64() & 0xF_FFFF)
    }
}

/// Un par de la comparación ciega: el mismo encargo hecho por dos
/// variantes, con sus imágenes a la izquierda (1) y a la derecha (2). Solo
/// lleva las claves, nunca las variantes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlindPair {
    /// Número del par (de 1 en adelante, en el orden en que se enseña).
    pub number: u32,
    pub task: String,
    pub left: String,
    pub right: String,
}

/// Los pares de la comparación: para cada encargo, cada pareja de variantes
/// que lo terminaron (sin error), en orden barajado y con los lados al azar.
pub fn plan_pairs(runs: &[RunMetrics], seed: u64) -> Vec<BlindPair> {
    let mut rng = Rng::new(seed);
    let mut tasks: Vec<&str> = Vec::new();
    for r in runs {
        if !tasks.contains(&r.task.as_str()) {
            tasks.push(&r.task);
        }
    }
    let mut pairs: Vec<(String, String, String)> = Vec::new();
    for task in tasks {
        let done: Vec<&RunMetrics> = runs
            .iter()
            .filter(|r| r.task == task && r.error.is_none())
            .collect();
        for i in 0..done.len() {
            for j in i + 1..done.len() {
                let (mut l, mut r) = (done[i].key.clone(), done[j].key.clone());
                if rng.below(2) == 1 {
                    std::mem::swap(&mut l, &mut r);
                }
                pairs.push((task.to_string(), l, r));
            }
        }
    }
    rng.shuffle(&mut pairs);
    pairs
        .into_iter()
        .enumerate()
        .map(|(i, (task, left, right))| BlindPair {
            number: i as u32 + 1,
            task,
            left,
            right,
        })
        .collect()
}

/// Qué imagen prefirió el usuario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Choice {
    /// La de la izquierda (1).
    Left,
    /// La de la derecha (2).
    Right,
    /// Igual de buenas (o igual de malas).
    Tie,
}

impl Choice {
    /// `1`, `2` o `=` (también «empate»).
    pub fn parse(raw: &str) -> Option<Choice> {
        match raw.trim().to_lowercase().as_str() {
            "1" | "i" | "izquierda" => Some(Choice::Left),
            "2" | "d" | "derecha" => Some(Choice::Right),
            "=" | "e" | "empate" | "igual" => Some(Choice::Tie),
            _ => None,
        }
    }
}

/// Una respuesta de la comparación.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    pub pair: u32,
    pub choice: Choice,
}

/// Preferencias de la comparación ciega, ya con las variantes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Preferences {
    /// Por variante: (ganados, perdidos, empates).
    pub by_variant: Vec<(Variant, [u32; 3])>,
    /// Por pareja de variantes (X, Y): veces que ganó X, que ganó Y y
    /// empates.
    pub head_to_head: Vec<((Variant, Variant), [u32; 3])>,
    /// Pares respondidos.
    pub answered: usize,
}

fn variant_of(runs: &[RunMetrics], key: &str) -> Option<Variant> {
    runs.iter().find(|r| r.key == key).map(|r| r.variant)
}

/// Cuenta las preferencias (las respuestas a pares que no existen se
/// ignoran; si un par tiene varias, vale la última).
pub fn tally(runs: &[RunMetrics], pairs: &[BlindPair], answers: &[Answer]) -> Preferences {
    let mut prefs = Preferences::default();
    let bump = |list: &mut Vec<(Variant, [u32; 3])>, v: Variant, slot: usize| match list
        .iter_mut()
        .find(|(x, _)| *x == v)
    {
        Some((_, c)) => c[slot] += 1,
        None => {
            let mut c = [0; 3];
            c[slot] = 1;
            list.push((v, c));
        }
    };
    for pair in pairs {
        let Some(answer) = answers.iter().rev().find(|a| a.pair == pair.number) else {
            continue;
        };
        let (Some(l), Some(r)) = (variant_of(runs, &pair.left), variant_of(runs, &pair.right))
        else {
            continue;
        };
        prefs.answered += 1;
        let (x, y) = if l <= r { (l, r) } else { (r, l) };
        let slot = match (answer.choice, l <= r) {
            (Choice::Tie, _) => 2,
            (Choice::Left, true) | (Choice::Right, false) => 0,
            _ => 1,
        };
        match prefs.head_to_head.iter_mut().find(|(k, _)| *k == (x, y)) {
            Some((_, c)) => c[slot] += 1,
            None => {
                let mut c = [0; 3];
                c[slot] = 1;
                prefs.head_to_head.push(((x, y), c));
            }
        }
        match answer.choice {
            Choice::Tie => {
                bump(&mut prefs.by_variant, l, 2);
                bump(&mut prefs.by_variant, r, 2);
            }
            Choice::Left => {
                bump(&mut prefs.by_variant, l, 0);
                bump(&mut prefs.by_variant, r, 1);
            }
            Choice::Right => {
                bump(&mut prefs.by_variant, r, 0);
                bump(&mut prefs.by_variant, l, 1);
            }
        }
    }
    prefs.by_variant.sort_by_key(|(v, _)| *v);
    prefs.head_to_head.sort_by_key(|(k, _)| *k);
    prefs
}

fn fmt_num(v: f64, decimals: usize) -> String {
    format!("{v:.decimals$}").replace('.', ",")
}

/// El informe (Markdown): métricas por variante, por encargo y, si ya se
/// hizo la comparación ciega, las preferencias y qué era cada par.
pub fn report(
    tasks: &[EvalTask],
    runs: &[RunMetrics],
    blind: Option<(&[BlindPair], &[Answer])>,
) -> String {
    let mut out = String::from("# Evaluación del bucle de LumeArt\n\n");
    out.push_str(&format!(
        "{} {} · variantes: {}\n\n",
        tasks.len(),
        if tasks.len() == 1 {
            "encargo"
        } else {
            "encargos"
        },
        summarize(runs)
            .iter()
            .map(|s| s.variant.label())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str("## Por variante\n\n");
    out.push_str("| Variante | Encargos | Nota del crítico | Pasa las comprobaciones | Vueltas | Tokens | Tiempo |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for s in summarize(runs) {
        let critic = match s.critic {
            Some(c) if s.critic_runs < s.runs => {
                format!("{} ({} de {})", fmt_num(c, 1), s.critic_runs, s.runs)
            }
            Some(c) => fmt_num(c, 1),
            None => "–".into(),
        };
        let runs = if s.failed > 0 {
            format!("{} ({} fallaron)", s.runs, s.failed)
        } else {
            s.runs.to_string()
        };
        out.push_str(&format!(
            "| {} | {runs} | {critic} | {} % | {} | {} | {} s |\n",
            s.variant.label(),
            fmt_num(s.passed_pct, 0),
            fmt_num(s.turns, 1),
            fmt_num(s.tokens, 0),
            fmt_num(s.seconds, 0),
        ));
    }
    out.push_str(
        "\nNota del crítico: 10 menos 3 por problema alto, 1,5 por medio y 0,5 por bajo. Tokens: entrada y salida, sin la caché. Las medias son por encargo.\n",
    );

    if let Some((pairs, answers)) = blind {
        let prefs = tally(runs, pairs, answers);
        out.push_str(&format!(
            "\n## Comparación ciega\n\n{} de {} pares respondidos.\n\n",
            prefs.answered,
            pairs.len()
        ));
        if prefs.answered > 0 {
            out.push_str(
                "| Variante | Preferida | No preferida | Empates |\n| --- | --- | --- | --- |\n",
            );
            for (v, [w, l, t]) in &prefs.by_variant {
                out.push_str(&format!("| {} | {w} | {l} | {t} |\n", v.label()));
            }
            out.push_str("\n| Pareja | Gana la primera | Gana la segunda | Empates |\n| --- | --- | --- | --- |\n");
            for ((x, y), [a, b, t]) in &prefs.head_to_head {
                out.push_str(&format!(
                    "| {} frente a {} | {a} | {b} | {t} |\n",
                    x.letter(),
                    y.letter()
                ));
            }
        }
    }

    out.push_str("\n## Por encargo\n\n| Encargo | Variante | Nota | Pasa | Vueltas | Tokens | Tiempo | Final |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for task in tasks {
        for r in runs.iter().filter(|r| r.task == task.id) {
            let end = match &r.error {
                Some(e) => format!("falló: {e}"),
                None => r.end.clone(),
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} s | {} |\n",
                task.id,
                r.variant.letter(),
                r.critic.map_or("–".into(), |c| fmt_num(c, 1)),
                if r.passed { "sí" } else { "no" },
                r.turns,
                r.tokens,
                fmt_num(r.seconds, 0),
                end.replace('|', "/").replace('\n', " "),
            ));
        }
    }
    out
}

/// Qué era cada par (solo cuando ya están todas las respuestas).
pub fn reveal(runs: &[RunMetrics], pairs: &[BlindPair], answers: &[Answer]) -> Vec<String> {
    pairs
        .iter()
        .map(|p| {
            let name = |key: &str| {
                variant_of(runs, key).map_or_else(|| "?".to_string(), |v| v.letter().to_string())
            };
            let choice = match answers.iter().rev().find(|a| a.pair == p.number) {
                Some(Answer {
                    choice: Choice::Left,
                    ..
                }) => format!("elegiste {}", name(&p.left)),
                Some(Answer {
                    choice: Choice::Right,
                    ..
                }) => format!("elegiste {}", name(&p.right)),
                Some(Answer {
                    choice: Choice::Tie,
                    ..
                }) => "empate".to_string(),
                None => "sin respuesta".to_string(),
            };
            format!(
                "Par {} ({}): 1 = {}, 2 = {} · {choice}",
                p.number,
                p.task,
                name(&p.left),
                name(&p.right)
            )
        })
        .collect()
}

/// Las dos imágenes de un par, una al lado de la otra con un hueco
/// transparente de un octavo del ancho entre ellas (las dos tienen el
/// tamaño del encargo).
pub fn side_by_side(left: &Canvas, right: &Canvas) -> Canvas {
    let (w, h) = (
        left.width().max(right.width()),
        left.height().max(right.height()),
    );
    let gap = (w / 8).max(2);
    let mut out =
        Canvas::new(i64::from(w * 2 + gap), i64::from(h)).unwrap_or_else(|_| left.clone());
    let mut copy = |src: &Canvas, dx: u32| {
        for y in 0..src.height() {
            for x in 0..src.width() {
                let c: Rgba = src.pixels()[(y * src.width() + x) as usize];
                let _ = out.set(i64::from(x + dx), i64::from(y), c);
            }
        }
    };
    copy(left, 0);
    copy(right, w + gap);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(task: &str, variant: Variant, key: &str) -> RunMetrics {
        RunMetrics {
            task: task.into(),
            variant,
            key: key.into(),
            critic: None,
            approved: None,
            issues: [0; 3],
            passed: false,
            turns: 1,
            tokens: 100,
            cached_tokens: 0,
            seconds: 10.0,
            end: "ok".into(),
            error: None,
        }
    }

    #[test]
    fn variants_parse_and_name_themselves() {
        assert_eq!(
            Variant::parse_list("c, a").unwrap(),
            [Variant::A, Variant::C]
        );
        assert_eq!(Variant::parse_list("ABCA").unwrap(), Variant::ALL);
        assert!(Variant::parse_list("D").is_err());
        assert!(Variant::parse_list("").is_err());
        assert_eq!(Variant::B.label(), "B (contexto vacío)");
    }

    #[test]
    fn the_critic_score_counts_by_severity() {
        assert_eq!(critic_score([0, 0, 0]), 10.0);
        assert_eq!(critic_score([1, 1, 2]), 4.5);
        assert_eq!(critic_score([4, 0, 0]), 0.0);
        assert_eq!(count_issues(["alta", "Media", "baja", "otra"]), [1, 1, 2]);
    }

    #[test]
    fn tasks_are_checked() {
        let t = |id: &str, w| EvalTask {
            id: id.into(),
            prompt: "una seta".into(),
            width: w,
            height: 32,
            origin: String::new(),
        };
        assert!(check_tasks(&[t("a", 32), t("b", 16)]).is_ok());
        assert!(check_tasks(&[]).is_err());
        assert!(check_tasks(&[t("a", 32), t("a", 16)]).is_err());
        assert!(check_tasks(&[t("a", 0)]).is_err());
    }

    #[test]
    fn summaries_average_per_variant() {
        let mut a1 = run("seta", Variant::A, "k1");
        a1.critic = Some(8.0);
        a1.passed = true;
        let mut a2 = run("cofre", Variant::A, "k2");
        a2.turns = 3;
        a2.error = Some("sin claude".into());
        let mut b1 = run("seta", Variant::B, "k3");
        b1.critic = Some(6.0);
        let s = summarize(&[a1, a2, b1]);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].variant, Variant::A);
        assert_eq!(s[0].critic, Some(8.0));
        assert_eq!(s[0].critic_runs, 1);
        assert_eq!(s[0].passed_pct, 50.0);
        assert_eq!(s[0].turns, 2.0);
        assert_eq!(s[0].failed, 1);
        assert_eq!(s[1].critic, Some(6.0));
    }

    fn three(task: &str, keys: [&str; 3]) -> Vec<RunMetrics> {
        Variant::ALL
            .iter()
            .zip(keys)
            .map(|(v, k)| run(task, *v, k))
            .collect()
    }

    /// Los pares no llevan las variantes, van barajados y con los lados al
    /// azar, y la misma semilla da el mismo plan.
    #[test]
    fn blind_pairs_hide_the_variants() {
        let mut runs = three("seta", ["k1", "k2", "k3"]);
        runs.extend(three("cofre", ["k4", "k5", "k6"]));
        runs.extend(three("lupa", ["k7", "k8", "k9"]));
        let pairs = plan_pairs(&runs, 7);
        assert_eq!(pairs.len(), 9);
        assert_eq!(pairs, plan_pairs(&runs, 7));
        assert_ne!(pairs, plan_pairs(&runs, 8));
        let numbers: Vec<u32> = pairs.iter().map(|p| p.number).collect();
        assert_eq!(numbers, (1..=9).collect::<Vec<_>>());
        // Nada en los pares dice la variante.
        let text = format!("{pairs:?}");
        for word in ["variant", "Variant", "plugin", "contexto", "conversación"] {
            assert!(!text.contains(word), "{text}");
        }
        // Barajados: no salen en el orden de los encargos…
        let tasks: Vec<&str> = pairs.iter().map(|p| p.task.as_str()).collect();
        assert_ne!(
            tasks,
            [
                "seta", "seta", "seta", "cofre", "cofre", "cofre", "lupa", "lupa", "lupa"
            ]
        );
        // …y A no está siempre a la izquierda.
        let a_left = pairs
            .iter()
            .filter(|p| variant_of(&runs, &p.left) == Some(Variant::A))
            .count();
        let a_right = pairs
            .iter()
            .filter(|p| variant_of(&runs, &p.right) == Some(Variant::A))
            .count();
        assert!(a_left > 0 && a_right > 0, "{pairs:?}");
        // Una ejecución que falló no entra.
        runs[0].error = Some("x".into());
        assert_eq!(plan_pairs(&runs, 7).len(), 7);
    }

    #[test]
    fn preferences_are_tallied_and_revealed() {
        let runs = three("seta", ["k1", "k2", "k3"]);
        let pairs = vec![
            BlindPair {
                number: 1,
                task: "seta".into(),
                left: "k2".into(),
                right: "k1".into(),
            },
            BlindPair {
                number: 2,
                task: "seta".into(),
                left: "k1".into(),
                right: "k3".into(),
            },
            BlindPair {
                number: 3,
                task: "seta".into(),
                left: "k3".into(),
                right: "k2".into(),
            },
        ];
        let answers = vec![
            Answer {
                pair: 1,
                choice: Choice::Left,
            },
            Answer {
                pair: 2,
                choice: Choice::Tie,
            },
            // Cambia de opinión: vale la última.
            Answer {
                pair: 3,
                choice: Choice::Left,
            },
            Answer {
                pair: 3,
                choice: Choice::Right,
            },
        ];
        let prefs = tally(&runs, &pairs, &answers);
        assert_eq!(prefs.answered, 3);
        assert_eq!(
            prefs.by_variant,
            [
                (Variant::A, [0, 1, 1]),
                (Variant::B, [2, 0, 0]),
                (Variant::C, [0, 1, 1])
            ]
        );
        assert_eq!(
            prefs.head_to_head,
            [
                ((Variant::A, Variant::B), [0, 1, 0]),
                ((Variant::A, Variant::C), [0, 0, 1]),
                ((Variant::B, Variant::C), [1, 0, 0])
            ]
        );
        let lines = reveal(&runs, &pairs, &answers);
        assert_eq!(lines[0], "Par 1 (seta): 1 = B, 2 = A · elegiste B");
        assert_eq!(lines[1], "Par 2 (seta): 1 = A, 2 = C · empate");
        assert_eq!(lines[2], "Par 3 (seta): 1 = C, 2 = B · elegiste B");
        assert_eq!(Choice::parse(" 2 "), Some(Choice::Right));
        assert_eq!(Choice::parse("="), Some(Choice::Tie));
        assert_eq!(Choice::parse("x"), None);
    }

    /// El informe trae nota del crítico, % que pasa, vueltas, tokens y
    /// tiempo por variante y por encargo.
    #[test]
    fn the_report_has_every_metric() {
        let task = EvalTask {
            id: "seta".into(),
            prompt: "una seta".into(),
            width: 32,
            height: 32,
            origin: String::new(),
        };
        let mut a = run("seta", Variant::A, "k1");
        a.critic = Some(7.5);
        a.passed = true;
        a.turns = 2;
        a.tokens = 1200;
        a.seconds = 61.0;
        let mut b = run("seta", Variant::B, "k2");
        b.critic = Some(8.5);
        let text = report(&[task], &[a, b], None);
        assert!(
            text.contains("| Variante | Encargos | Nota del crítico | Pasa las comprobaciones | Vueltas | Tokens | Tiempo |"),
            "{text}"
        );
        assert!(
            text.contains("| A (plugin oficial) | 1 | 7,5 | 100 % | 2,0 | 1200 | 61 s |"),
            "{text}"
        );
        assert!(
            text.contains("| B (contexto vacío) | 1 | 8,5 | 0 % |"),
            "{text}"
        );
        assert!(
            text.contains("| seta | A | 7,5 | sí | 2 | 1200 | 61 s | ok |"),
            "{text}"
        );
        assert!(!text.contains("Comparación ciega"), "{text}");
        assert!(
            text.contains("1 encargo · variantes: A (plugin oficial), B (contexto vacío)"),
            "{text}"
        );
    }

    #[test]
    fn pairs_are_drawn_side_by_side() {
        let mut a = Canvas::new(16, 8).unwrap();
        a.set(0, 0, Rgba::rgb(255, 0, 0)).unwrap();
        let mut b = Canvas::new(16, 8).unwrap();
        b.set(15, 7, Rgba::rgb(0, 0, 255)).unwrap();
        let both = side_by_side(&a, &b);
        assert_eq!((both.width(), both.height()), (34, 8));
        assert_eq!(both.get(0, 0).unwrap(), Rgba::rgb(255, 0, 0));
        assert_eq!(both.get(33, 7).unwrap(), Rgba::rgb(0, 0, 255));
        assert_eq!(both.get(17, 4).unwrap().a, 0);
    }
}
