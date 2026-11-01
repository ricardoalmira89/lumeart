//! Cómo encuentra un cliente (la CLI, `lumeart mcp`) la sesión abierta.
//!
//! Sustituye a `/tmp/pxcli.sock` y `/tmp/pxcli.pid`: la API local escribe un
//! archivo de estado con el puerto, el PID y el token, y el cliente lo lee. El
//! archivo es JSON; leerlo y escribirlo es cosa de los adaptadores.

use serde::{Deserialize, Serialize};

/// Variable de entorno con la ruta del archivo de estado (como `PXCLI_SOCKET`).
pub const SESSION_ENV: &str = "LUMEART_SESSION";

/// Variable de entorno con el número de ejecución del agente: la CLI la pasa
/// a la API (`run=N`) y sus cambios quedan como del agente, no de la CLI.
pub const RUN_ENV: &str = "LUMEART_RUN";

/// Cabecera HTTP con el token: `Authorization: Bearer <token>`.
pub const AUTH_SCHEME: &str = "Bearer";

/// Contenido del archivo de estado de una sesión abierta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Puerto de la API en `127.0.0.1`.
    pub port: u16,
    /// PID del proceso que sirve la sesión.
    pub pid: u32,
    /// Token que hay que enviar en cada petición.
    pub token: String,
    /// Versión de LumeArt que la sirve.
    pub version: String,
}

impl SessionInfo {
    /// URL base de la API (`http://127.0.0.1:<puerto>`).
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}
