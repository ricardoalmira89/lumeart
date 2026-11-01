//! Lo común a los proveedores HTTP: el cliente, los reintentos con espera
//! creciente (respetando `retry-after`) y los fallos de conexión.

use std::time::Duration;

use lume_app::ports::PortError;

/// Cuántas veces y cuánto se espera antes de reintentar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_retries: u32,
    /// Espera antes del primer reintento; se dobla en cada uno.
    pub base_delay: Duration,
    /// Espera máxima (también para `retry-after`).
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
        }
    }
}

/// Cliente HTTP de los proveedores: 15 s para conectar y hasta 10 minutos
/// por respuesta (las largas, con razonamiento).
pub(crate) fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(600))
        .build()
        .expect("cliente HTTP")
}

/// Estados que merece la pena reintentar.
pub(crate) fn is_retryable_status(status: u16) -> bool {
    matches!(status, 408 | 409 | 429 | 500..=599)
}

/// Una respuesta HTTP fallida, ya traducida.
pub(crate) struct Failure {
    pub error: PortError,
    pub retryable: bool,
}

/// Envía la petición que construye `make` (una nueva en cada intento) y
/// devuelve el cuerpo de la primera respuesta 2xx. `fail` traduce una
/// respuesta fallida (estado y cuerpo) a error y dice si se reintenta.
pub(crate) async fn send_with_retries(
    retry: &RetryPolicy,
    provider: &str,
    make: impl Fn() -> reqwest::RequestBuilder,
    fail: impl Fn(u16, &str) -> Failure,
) -> Result<String, PortError> {
    let mut attempt = 0;
    loop {
        let (failure, retry_after) = match send_once(make()).await {
            Ok((status, _, text)) if (200..300).contains(&status) => return Ok(text),
            Ok((status, retry_after, text)) => (fail(status, &text), retry_after),
            Err(e) => (
                Failure {
                    error: connection_error(provider, &e),
                    retryable: true,
                },
                None,
            ),
        };
        if !failure.retryable || attempt >= retry.max_retries {
            return Err(failure.error);
        }
        let backoff = retry.base_delay * 2u32.pow(attempt);
        tokio::time::sleep(retry_after.unwrap_or(backoff).min(retry.max_delay)).await;
        attempt += 1;
    }
}

async fn send_once(
    request: reqwest::RequestBuilder,
) -> Result<(u16, Option<Duration>, String), reqwest::Error> {
    let response = request.send().await?;
    let status = response.status().as_u16();
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s >= 0.0)
        .map(Duration::from_secs_f64);
    let text = response.text().await?;
    Ok((status, retry_after, text))
}

/// Fallo de conexión sin la URL ni detalles de la petición.
fn connection_error(provider: &str, e: &reqwest::Error) -> PortError {
    if e.is_timeout() {
        PortError::Timeout
    } else if e.is_connect() {
        PortError::Unavailable(format!("{provider}: no se pudo conectar"))
    } else {
        PortError::Unavailable(format!("{provider}: la conexión falló"))
    }
}

/// Quita de un mensaje de error cualquier cosa con forma de clave
/// (`sk-…`), aunque venga enmascarada: algunos proveedores la repiten.
pub(crate) fn scrub_keys(message: &str) -> String {
    message
        .split(' ')
        .map(|word| {
            let bare = word.trim_start_matches(['"', '\'', '(', '[']);
            if bare.starts_with("sk-") || bare.starts_with("sk_") {
                "[clave]"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_scrubbed_from_messages() {
        assert_eq!(
            scrub_keys("Incorrect API key provided: sk-proj-****abcd. You can find…"),
            "Incorrect API key provided: [clave] You can find…"
        );
        assert_eq!(scrub_keys("sin claves aquí"), "sin claves aquí");
        assert_eq!(scrub_keys("(sk_live_123)"), "[clave]");
    }
}
