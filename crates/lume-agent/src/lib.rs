//! Adaptador del agente de LumeArt.
//!
//! - `AgentRunner`: `ClaudeCodeRunner` (lanza `claude -p`) y `ApiRunner`
//!   (bucle propio contra un `ModelProvider`).
//! - `ModelProvider`: `AnthropicProvider`, `OpenAiProvider` y
//!   `OpenAiCompatibleProvider` (Ollama, LM Studio, llama.cpp).
//! - `ToolBox`: `draw`, `view_canvas`, `get_pixel`, `undo`, `palette` y, en
//!   animación, `anim_check`, `anim_sheet`, `write_anim_js`.
//! - `RefineLoop`: el ralph loop del producto. Cada vuelta empieza con el
//!   contexto vacío y solo hereda lo guardado en el documento.
//!
//! Regla: depende de `lume-app` y `lume-domain`, nunca de otro adaptador.

pub mod anthropic;
pub mod api_runner;
pub mod claude_bin;
pub mod claude_code;
pub mod critic;
pub mod http;
pub mod openai;
pub mod prompts;
pub mod tools;

pub use anthropic::AnthropicProvider;
pub use api_runner::ApiRunner;
pub use claude_bin::{find_claude, find_codex};
pub use claude_code::{ClaudeCodeRunner, RalphPlugin};
pub use critic::{AgentRole, Critique, Issue, ReadOnlyTools, Verdict, parse_critique, review};
pub use openai::{OpenAiCompatibleProvider, OpenAiProvider};
pub use tools::{SessionTools, TOOL_NAMES, VEC_TOOL_NAMES, VecTools, vec_specs};
