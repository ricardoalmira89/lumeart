//! El agente de la ventana: monta, para cada ejecución, el runner del
//! proveedor elegido en Ajustes (Claude Code, Anthropic, OpenAI) con las
//! herramientas sobre la sesión. Las claves salen del llavero en ese momento.

use std::path::PathBuf;
use std::sync::Arc;

use lume_agent::{
    AnthropicProvider, ApiRunner, ClaudeCodeRunner, OpenAiProvider, RalphPlugin, SessionTools,
    VecTools,
};
use lume_app::ports::{PortError, PreparedAgent, ToolBox};
use lume_app::{AgentBackend, DocumentStore, ProviderKind, SecretStore, SessionHandle, Settings};
use lume_domain::RunId;

/// `AgentBackend` de la ventana.
pub struct WindowAgent {
    pub session: SessionHandle,
    pub store: Arc<dyn DocumentStore>,
    pub secrets: Arc<dyn SecretStore>,
    /// Ruta de `claude`, si está instalado.
    pub claude: Option<PathBuf>,
    /// Este ejecutable (Claude Code lo usa como `lumeart`).
    pub lumeart: PathBuf,
    /// Archivo de estado de esta sesión (para `LUMEART_SESSION`).
    pub session_file: PathBuf,
    /// Carpeta de trabajo de las ejecuciones de Claude Code.
    pub work_dir: PathBuf,
    /// Motor de animación (herramientas `read_anim`, `write_anim`,
    /// `check_anim`).
    pub anim: Option<Arc<dyn lume_app::AnimEngine>>,
    /// Motor de `dibujo.js` y rasterizador (herramientas `read_vec`,
    /// `write_vec`, `check_vec`).
    pub vector: Option<VecTools>,
    /// El plugin oficial `ralph-loop` lleva el bucle (variante A de
    /// `lumeart eval`; solo con Claude Code).
    pub ralph: Option<RalphPlugin>,
}

impl WindowAgent {
    fn key(
        &self,
        provider: ProviderKind,
        name: &str,
    ) -> Result<lume_app::ports::Secret, PortError> {
        let key = provider.secret_key().unwrap_or_default();
        self.secrets.get(key)?.ok_or_else(|| {
            PortError::Unavailable(format!(
                "falta la clave de {name}; añádela en Ajustes › Proveedores de IA"
            ))
        })
    }
}

impl WindowAgent {
    /// El agente (o su crítico) del proveedor de los ajustes.
    fn build(
        &self,
        settings: &Settings,
        run: RunId,
        critic: bool,
    ) -> Result<PreparedAgent, PortError> {
        let mut tools = SessionTools::new(self.session.clone(), self.store.clone(), run);
        if let Some(engine) = &self.anim {
            tools = tools.with_anim(engine.clone());
        }
        if let Some(vector) = &self.vector {
            tools = tools.with_vec(vector.clone());
        }
        let tools: ToolBox = Arc::new(tools);
        if self.ralph.is_some() && !critic && settings.provider != ProviderKind::ClaudeCode {
            return Err(PortError::Unavailable(
                "el plugin oficial (variante A) solo funciona con Claude Code: elige Claude Code en Ajustes o deja fuera la A con --variants B,C".into(),
            ));
        }
        let (runner, model): (Arc<dyn lume_app::AgentRunner>, String) = match settings.provider {
            ProviderKind::ClaudeCode => {
                let runner = ClaudeCodeRunner::new(
                    self.claude.clone(),
                    self.lumeart.clone(),
                    self.session_file.clone(),
                    self.work_dir.clone(),
                );
                let runner = match (&self.ralph, critic) {
                    (_, true) => runner.as_critic(),
                    (Some(plugin), false) => runner.with_ralph(plugin.clone()),
                    (None, false) => runner,
                };
                (Arc::new(runner), settings.claude_code_model.clone())
            }
            ProviderKind::Anthropic => {
                let key = self.key(ProviderKind::Anthropic, "Anthropic")?;
                let runner = ApiRunner::new(Arc::new(AnthropicProvider::new(key)));
                let runner = if critic { runner.as_critic() } else { runner };
                (Arc::new(runner), settings.anthropic_model.clone())
            }
            ProviderKind::OpenAi => {
                let key = self.key(ProviderKind::OpenAi, "OpenAI")?;
                let runner = ApiRunner::new(Arc::new(OpenAiProvider::new(key)));
                let runner = if critic { runner.as_critic() } else { runner };
                (Arc::new(runner), settings.openai_model.clone())
            }
            ProviderKind::Local => {
                return Err(PortError::Unavailable(
                    "los modelos locales llegarán pronto; elige otro proveedor en Ajustes".into(),
                ));
            }
        };
        Ok(PreparedAgent {
            runner,
            tools,
            model,
        })
    }
}

impl AgentBackend for WindowAgent {
    fn prepare(&self, settings: &Settings, run: RunId) -> Result<PreparedAgent, PortError> {
        self.build(settings, run, false)
    }

    fn prepare_critic(&self, settings: &Settings, run: RunId) -> Result<PreparedAgent, PortError> {
        self.build(settings, run, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_app::ports::Secret;
    use lume_app::testing::{MemorySecretStore, MemoryStore};
    use lume_domain::Document;

    fn agent(rt: &tokio::runtime::Runtime, secrets: Arc<MemorySecretStore>) -> WindowAgent {
        let session = {
            let _guard = rt.enter();
            lume_app::Session::spawn(
                Document::new("a", 8, 8).unwrap(),
                Arc::new(MemoryStore::new()),
            )
        };
        WindowAgent {
            session: session.handle.clone(),
            store: Arc::new(MemoryStore::new()),
            secrets,
            claude: Some("/usr/bin/claude".into()),
            lumeart: "/usr/bin/lumeart".into(),
            session_file: "/tmp/s.json".into(),
            work_dir: "/tmp/agente".into(),
            anim: None,
            vector: None,
            ralph: None,
        }
    }

    /// Cada proveedor de los ajustes da su agente, con su modelo; sin clave,
    /// un error que dice dónde ponerla.
    #[test]
    fn each_provider_gets_its_agent() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let secrets = Arc::new(MemorySecretStore::default());
        let a = agent(&rt, secrets.clone());
        let mut settings = Settings {
            claude_code_model: "haiku".into(),
            ..Settings::default()
        };
        assert_eq!(a.prepare(&settings, 1).unwrap().model, "haiku");

        settings.provider = ProviderKind::Anthropic;
        let err = a.prepare(&settings, 2).unwrap_err();
        assert_eq!(
            err.to_string(),
            "no disponible: falta la clave de Anthropic; añádela en Ajustes › Proveedores de IA"
        );
        secrets
            .set("anthropic-api-key", &Secret::new("sk-ant-prueba"))
            .unwrap();
        assert_eq!(a.prepare(&settings, 3).unwrap().model, "claude-opus-5-5");

        settings.provider = ProviderKind::OpenAi;
        assert!(matches!(
            a.prepare(&settings, 4),
            Err(PortError::Unavailable(_))
        ));
        secrets
            .set("openai-api-key", &Secret::new("sk-prueba"))
            .unwrap();
        assert_eq!(a.prepare(&settings, 5).unwrap().model, "gpt-5");

        assert_eq!(a.prepare_critic(&settings, 5).unwrap().model, "gpt-5");

        settings.provider = ProviderKind::Local;
        assert!(matches!(
            a.prepare(&settings, 6),
            Err(PortError::Unavailable(_))
        ));
        assert!(matches!(
            a.prepare_critic(&settings, 6),
            Err(PortError::Unavailable(_))
        ));
        settings.provider = ProviderKind::ClaudeCode;
        assert_eq!(a.prepare_critic(&settings, 7).unwrap().model, "haiku");
    }
}
