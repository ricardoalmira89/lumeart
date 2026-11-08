//! Ajustes de LumeArt: proveedor del agente, modelos, bucle de refinado y
//! vista. Se guardan en `config.toml` (lo hace `lume-store`); las claves de
//! API no están aquí: van por [`SecretStore`](crate::SecretStore).

use std::ops::RangeInclusive;

use crate::ports::PortError;

/// Con qué dibuja el agente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProviderKind {
    /// `claude -p` con la sesión de Claude Code del usuario (sin clave).
    #[default]
    ClaudeCode,
    /// Messages API de Anthropic con la clave del usuario.
    Anthropic,
    /// API de OpenAI con la clave del usuario.
    OpenAi,
    /// Servidor local compatible con la API de OpenAI (Ollama, LM Studio…).
    Local,
}

impl ProviderKind {
    pub const ALL: [ProviderKind; 4] = [
        ProviderKind::ClaudeCode,
        ProviderKind::Anthropic,
        ProviderKind::OpenAi,
        ProviderKind::Local,
    ];

    /// Nombre en `config.toml`.
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderKind::ClaudeCode => "claude-code",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::OpenAi => "openai",
            ProviderKind::Local => "local",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == s)
    }

    /// Nombre de su clave en el [`SecretStore`](crate::SecretStore), si usa una.
    pub fn secret_key(self) -> Option<&'static str> {
        match self {
            ProviderKind::Anthropic => Some("anthropic-api-key"),
            ProviderKind::OpenAi => Some("openai-api-key"),
            ProviderKind::ClaudeCode | ProviderKind::Local => None,
        }
    }
}

/// Qué recuerda cada vuelta del bucle de refinado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContextPolicy {
    /// Cada vuelta empieza con el contexto vacío (recomendado).
    #[default]
    Fresh,
    /// Cada vuelta sigue la conversación anterior.
    Resume,
}

impl ContextPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            ContextPolicy::Fresh => "fresh",
            ContextPolicy::Resume => "resume",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "fresh" => Some(ContextPolicy::Fresh),
            "resume" => Some(ContextPolicy::Resume),
            _ => None,
        }
    }
}

/// A qué velocidad se ve dibujar al agente en el lienzo (como el studio web
/// de pxcli): cada comando se pinta píxel a píxel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DrawSpeed {
    /// Píxel a píxel: 40 píxeles por segundo.
    Slow,
    /// 160 píxeles por segundo.
    #[default]
    Normal,
    /// 900 píxeles por segundo.
    Fast,
    /// Sin animación: el cambio aparece entero.
    Instant,
}

impl DrawSpeed {
    pub const ALL: [DrawSpeed; 4] = [
        DrawSpeed::Slow,
        DrawSpeed::Normal,
        DrawSpeed::Fast,
        DrawSpeed::Instant,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            DrawSpeed::Slow => "lento",
            DrawSpeed::Normal => "normal",
            DrawSpeed::Fast => "rapido",
            DrawSpeed::Instant => "instante",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    /// Nombre en la interfaz.
    pub fn label(self) -> &'static str {
        match self {
            DrawSpeed::Slow => "Píxel a píxel",
            DrawSpeed::Normal => "Normal",
            DrawSpeed::Fast => "Rápido",
            DrawSpeed::Instant => "Al instante",
        }
    }

    /// Píxeles por segundo y en cuántos segundos como mucho se pone al día
    /// si va por detrás (`None`: al instante).
    pub fn rate(self) -> Option<(f32, f32)> {
        match self {
            DrawSpeed::Slow => Some((40.0, 20.0)),
            DrawSpeed::Normal => Some((160.0, 6.0)),
            DrawSpeed::Fast => Some((900.0, 2.0)),
            DrawSpeed::Instant => None,
        }
    }
}

/// Vueltas máximas permitidas en el bucle de refinado.
pub const MAX_TURNS: RangeInclusive<u32> = 1..=50;

/// Modelo de la Messages API por defecto.
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-opus-5-5";
/// Modelo de OpenAI por defecto.
pub const DEFAULT_OPENAI_MODEL: &str = "gpt-5";
/// Dirección por defecto del servidor local (Ollama).
pub const DEFAULT_LOCAL_URL: &str = "http://localhost:11434/v1";

/// Todos los ajustes. `Default` da los valores de fábrica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Proveedor por defecto del agente.
    pub provider: ProviderKind,
    /// Modelo para `claude -p --model` (vacío: el que use Claude Code).
    pub claude_code_model: String,
    /// Claude Code sin restricciones: cualquier herramienta, comando o
    /// archivo, sin pedir permiso (`--permission-mode bypassPermissions`).
    /// Apagado, solo puede usar `lumeart` y sus archivos.
    pub claude_code_full_access: bool,
    pub anthropic_model: String,
    pub openai_model: String,
    pub local_url: String,
    /// Vueltas máximas del bucle de refinado.
    pub max_turns: u32,
    pub context: ContextPolicy,
    /// Parar si dos vueltas seguidas no avanzan.
    pub stop_without_progress: bool,
    /// Otro modelo revisa cada vuelta.
    pub critic: bool,
    /// Resaltar en el lienzo el último comando del agente o de la CLI.
    pub show_commands: bool,
    /// Velocidad a la que se ve dibujar al agente.
    pub draw_speed: DrawSpeed,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: ProviderKind::ClaudeCode,
            claude_code_model: String::new(),
            claude_code_full_access: false,
            anthropic_model: DEFAULT_ANTHROPIC_MODEL.into(),
            openai_model: DEFAULT_OPENAI_MODEL.into(),
            local_url: DEFAULT_LOCAL_URL.into(),
            max_turns: 8,
            context: ContextPolicy::Fresh,
            stop_without_progress: true,
            critic: false,
            // Apagado: el resalte verde de cada comando tapaba el dibujo
            // mientras se hacía (ahora se ve dibujar con el pincel).
            show_commands: false,
            draw_speed: DrawSpeed::Normal,
        }
    }
}

impl Settings {
    /// Corrige lo que no sea válido con el valor de fábrica (vueltas fuera de
    /// rango, modelos con caracteres raros, dirección vacía).
    pub fn validated(mut self) -> Self {
        let defaults = Settings::default();
        self.max_turns = self.max_turns.clamp(*MAX_TURNS.start(), *MAX_TURNS.end());
        if !self.claude_code_model.is_empty() && !valid_model(&self.claude_code_model) {
            self.claude_code_model = defaults.claude_code_model;
        }
        if !valid_model(&self.anthropic_model) {
            self.anthropic_model = defaults.anthropic_model;
        }
        if !valid_model(&self.openai_model) {
            self.openai_model = defaults.openai_model;
        }
        let url = self.local_url.trim();
        self.local_url = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            defaults.local_url
        };
        self
    }
}

/// ¿Es un nombre de modelo aceptable? Como `validModel` de pxcli: hasta 64
/// caracteres `a-z`, `0-9` y `-._[]` (así nunca se cuela nada en `claude
/// --model`).
pub fn valid_model(model: &str) -> bool {
    !model.is_empty()
        && model.len() <= 64
        && model
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "-._[]".contains(c))
}

/// Dónde se guardan los ajustes. Lo implementa `lume-store` (`config.toml`).
pub trait SettingsStore: Send + Sync {
    /// Los ajustes guardados, o los de fábrica si aún no hay ninguno. Lo que
    /// no se entienda se sustituye por el valor de fábrica.
    fn load(&self) -> Result<Settings, PortError>;
    fn save(&self, settings: &Settings) -> Result<(), PortError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_design() {
        let s = Settings::default();
        assert_eq!(s.provider, ProviderKind::ClaudeCode);
        assert_eq!(s.max_turns, 8);
        assert_eq!(s.context, ContextPolicy::Fresh);
        assert!(s.stop_without_progress);
        assert!(!s.critic);
        // Decisión del 31 oct 2026: el resalte de cada comando, apagado; se
        // ve dibujar a velocidad normal.
        assert!(!s.show_commands);
        assert_eq!(s.draw_speed, DrawSpeed::Normal);
        assert_eq!(s.anthropic_model, "claude-opus-5-5");
        assert_eq!(s.local_url, "http://localhost:11434/v1");
    }

    #[test]
    fn names_round_trip() {
        for p in ProviderKind::ALL {
            assert_eq!(ProviderKind::parse(p.as_str()), Some(p));
        }
        for v in DrawSpeed::ALL {
            assert_eq!(DrawSpeed::parse(v.as_str()), Some(v));
        }
        for c in [ContextPolicy::Fresh, ContextPolicy::Resume] {
            assert_eq!(ContextPolicy::parse(c.as_str()), Some(c));
        }
        assert_eq!(ProviderKind::parse("gemini"), None);
        assert_eq!(
            ProviderKind::Anthropic.secret_key(),
            Some("anthropic-api-key")
        );
        assert_eq!(ProviderKind::ClaudeCode.secret_key(), None);
    }

    #[test]
    fn models_are_checked_like_pxcli() {
        assert!(valid_model("sonnet"));
        assert!(valid_model("claude-opus-5-5[1m]"));
        assert!(valid_model("gpt-5.1"));
        assert!(!valid_model("opus; rm -rf /"));
        assert!(!valid_model("Sonnet"));
        assert!(!valid_model(""));
        assert!(!valid_model(&"a".repeat(65)));
    }

    #[test]
    fn validation_fixes_bad_values() {
        let s = Settings {
            max_turns: 0,
            claude_code_model: "opus --dangerously".into(),
            anthropic_model: String::new(),
            openai_model: "gpt 5".into(),
            local_url: "  ftp://x ".into(),
            ..Settings::default()
        }
        .validated();
        let d = Settings::default();
        assert_eq!(s.max_turns, 1);
        assert_eq!(s.claude_code_model, "");
        assert_eq!(s.anthropic_model, d.anthropic_model);
        assert_eq!(s.openai_model, d.openai_model);
        assert_eq!(s.local_url, d.local_url);

        let s = Settings {
            max_turns: 500,
            claude_code_model: "haiku".into(),
            local_url: " http://127.0.0.1:1234/v1 ".into(),
            ..Settings::default()
        }
        .validated();
        assert_eq!(s.max_turns, 50);
        assert_eq!(s.claude_code_model, "haiku");
        assert_eq!(s.local_url, "http://127.0.0.1:1234/v1");
    }
}
