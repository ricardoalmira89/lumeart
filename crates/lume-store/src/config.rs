//! `config.toml`: los ajustes de LumeArt en la carpeta de configuración del
//! usuario. Nunca lleva claves de API (esas van al llavero).
//!
//! Al leer, cada campo se interpreta por separado: lo que falte o no se
//! entienda toma el valor de fábrica, sin perder el resto.

use std::path::{Path, PathBuf};

use lume_app::ports::PortError;
use lume_app::{ContextPolicy, DrawSpeed, ProviderKind, Settings, SettingsStore};
use toml::{Table, Value};

/// `SettingsStore` sobre un archivo TOML.
#[derive(Debug, Clone)]
pub struct FsSettings {
    path: PathBuf,
}

impl FsSettings {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl SettingsStore for FsSettings {
    fn load(&self) -> Result<Settings, PortError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Settings::default()),
            Err(e) => return Err(PortError::Io(format!("{}: {e}", self.path.display()))),
        };
        settings_from_toml(&text)
            .map_err(|e| PortError::Invalid(format!("{}: {e}", self.path.display())))
    }

    fn save(&self, settings: &Settings) -> Result<(), PortError> {
        let io = |e: std::io::Error| PortError::Io(format!("{}: {e}", self.path.display()));
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        // Se escribe aparte y se renombra: nunca queda un archivo a medias.
        let tmp = self.path.with_extension("toml.tmp");
        std::fs::write(&tmp, settings_to_toml(settings)).map_err(io)?;
        std::fs::rename(&tmp, &self.path).map_err(io)
    }
}

/// Lee los ajustes de un texto TOML. Solo falla si el texto no es TOML; los
/// campos que falten o no valgan toman el valor de fábrica.
pub fn settings_from_toml(text: &str) -> Result<Settings, String> {
    let root: Table = text
        .parse()
        .map_err(|e: toml::de::Error| e.message().to_string())?;
    let d = Settings::default();
    let get = |section: &str, key: &str| -> Option<&Value> { root.get(section)?.get(key) };
    let string = |section, key| {
        get(section, key)
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let boolean = |section, key, default| {
        get(section, key)
            .and_then(Value::as_bool)
            .unwrap_or(default)
    };
    Ok(Settings {
        provider: string("agent", "provider")
            .and_then(|s| ProviderKind::parse(&s))
            .unwrap_or(d.provider),
        claude_code_model: string("models", "claude_code").unwrap_or(d.claude_code_model),
        anthropic_model: string("models", "anthropic").unwrap_or(d.anthropic_model),
        openai_model: string("models", "openai").unwrap_or(d.openai_model),
        local_url: string("models", "local_url").unwrap_or(d.local_url),
        max_turns: get("loop", "max_turns")
            .and_then(Value::as_integer)
            .map(|n| n.clamp(0, i64::from(u32::MAX)) as u32)
            .unwrap_or(d.max_turns),
        context: string("loop", "context")
            .and_then(|s| ContextPolicy::parse(&s))
            .unwrap_or(d.context),
        stop_without_progress: boolean("loop", "stop_without_progress", d.stop_without_progress),
        critic: boolean("review", "critic", d.critic),
        show_commands: boolean("review", "show_commands", d.show_commands),
        draw_speed: string("canvas", "draw_speed")
            .and_then(|s| DrawSpeed::parse(&s))
            .unwrap_or(d.draw_speed),
    }
    .validated())
}

/// Escribe los ajustes como TOML comentado, en el orden de la pantalla.
pub fn settings_to_toml(s: &Settings) -> String {
    let q = |text: &str| Value::String(text.to_string()).to_string();
    format!(
        "# Ajustes de LumeArt. Las claves de API no se guardan aquí: van al
# llavero del sistema.

[agent]
# claude-code | anthropic | openai | local
provider = {provider}

[models]
# Vacío: el modelo que tenga configurado Claude Code.
claude_code = {claude_code}
anthropic = {anthropic}
openai = {openai}
local_url = {local_url}

[loop]
max_turns = {max_turns}
# fresh: cada vuelta empieza con el contexto vacío · resume: misma conversación
context = {context}
stop_without_progress = {stop}

[review]
critic = {critic}
show_commands = {show_commands}

[canvas]
# Velocidad a la que se ve dibujar al agente: lento | normal | rapido | instante
draw_speed = {draw_speed}
",
        provider = q(s.provider.as_str()),
        claude_code = q(&s.claude_code_model),
        anthropic = q(&s.anthropic_model),
        openai = q(&s.openai_model),
        local_url = q(&s.local_url),
        max_turns = s.max_turns,
        context = q(s.context.as_str()),
        stop = s.stop_without_progress,
        critic = s.critic,
        show_commands = s.show_commands,
        draw_speed = q(s.draw_speed.as_str()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom() -> Settings {
        Settings {
            provider: ProviderKind::Anthropic,
            claude_code_model: "haiku".into(),
            anthropic_model: "claude-sonnet-5-5".into(),
            openai_model: "gpt-5-mini".into(),
            local_url: "http://127.0.0.1:1234/v1".into(),
            max_turns: 12,
            context: ContextPolicy::Resume,
            stop_without_progress: false,
            critic: true,
            show_commands: true,
            draw_speed: DrawSpeed::Fast,
        }
    }

    #[test]
    fn toml_round_trip() {
        for s in [Settings::default(), custom()] {
            let text = settings_to_toml(&s);
            assert_eq!(settings_from_toml(&text).unwrap(), s, "{text}");
        }
    }

    #[test]
    fn missing_or_bad_fields_take_the_factory_value() {
        let text = r#"
[agent]
provider = "gemini"
[loop]
max_turns = "muchas"
context = "resume"
[review]
critic = true
[otra]
cosa = 1
"#;
        let s = settings_from_toml(text).unwrap();
        let d = Settings::default();
        assert_eq!(s.provider, d.provider);
        assert_eq!(s.max_turns, d.max_turns);
        assert_eq!(s.context, ContextPolicy::Resume);
        assert!(s.critic);
        assert_eq!(s.show_commands, d.show_commands);
        assert_eq!(s.draw_speed, d.draw_speed);
        let s = settings_from_toml("[canvas]\ndraw_speed = \"lento\"").unwrap();
        assert_eq!(s.draw_speed, DrawSpeed::Slow);
        let s = settings_from_toml("[canvas]\ndraw_speed = \"volando\"").unwrap();
        assert_eq!(s.draw_speed, d.draw_speed);
        assert_eq!(settings_from_toml("").unwrap(), d);
        // Fuera de rango: se corrige.
        let s = settings_from_toml("[loop]\nmax_turns = -3").unwrap();
        assert_eq!(s.max_turns, 1);
        assert!(settings_from_toml("[loop\nmax_turns = 3").is_err());
    }

    #[test]
    fn strings_are_escaped() {
        let s = Settings {
            local_url: "http://localhost:1/\"v1\"".into(),
            ..Settings::default()
        };
        let back = settings_from_toml(&settings_to_toml(&s)).unwrap();
        assert_eq!(back.local_url, s.local_url);
    }

    #[test]
    fn the_file_survives_between_runs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lumeart").join("config.toml");
        let store = FsSettings::new(&path);
        assert_eq!(store.load().unwrap(), Settings::default(), "sin archivo");
        store.save(&custom()).unwrap();
        // Otra «ejecución»: otro FsSettings sobre el mismo archivo.
        assert_eq!(FsSettings::new(&path).load().unwrap(), custom());
        assert!(!path.with_extension("toml.tmp").exists());
        std::fs::write(&path, "esto no es toml [").unwrap();
        assert!(matches!(store.load(), Err(PortError::Invalid(_))));
    }
}
