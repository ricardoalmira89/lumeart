//! El crítico. Su núcleo (veredicto, herramientas de solo lectura y
//! `review`) está en `lume_app::critic`, porque lo usa el bucle de refinado;
//! aquí queda el papel de cada ejecución (dibujar o revisar).

pub use lume_app::critic::*;

/// Qué hace una ejecución: dibujar o revisar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentRole {
    #[default]
    Artist,
    Critic,
}
