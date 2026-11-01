//! Claves de API en el llavero del sistema (`keyring`): Secret Service en
//! Linux (GNOME Keyring, KWallet), el Administrador de credenciales en
//! Windows y el Llavero en macOS.
//!
//! Los mensajes de error nunca incluyen el valor de una clave.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use keyring::Entry;
use lume_app::SecretStore;
use lume_app::ports::{PortError, Secret};

/// Servicio con el que se guardan las claves de LumeArt.
pub const SERVICE: &str = "lumeart";

/// `SecretStore` sobre el llavero del sistema.
pub struct KeyringSecrets {
    service: String,
    /// Una entrada por clave, creada al primer uso.
    entries: Mutex<HashMap<String, Arc<Entry>>>,
}

impl Default for KeyringSecrets {
    fn default() -> Self {
        Self::with_service(SERVICE)
    }
}

impl std::fmt::Debug for KeyringSecrets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyringSecrets")
            .field("service", &self.service)
            .finish_non_exhaustive()
    }
}

impl KeyringSecrets {
    /// Con otro nombre de servicio (para pruebas).
    pub fn with_service(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            entries: Mutex::new(HashMap::new()),
        }
    }

    fn entry(&self, key: &str) -> Result<Arc<Entry>, PortError> {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = entries.get(key) {
            return Ok(entry.clone());
        }
        let entry = Arc::new(Entry::new(&self.service, key).map_err(error)?);
        entries.insert(key.to_string(), entry.clone());
        Ok(entry)
    }
}

impl SecretStore for KeyringSecrets {
    fn get(&self, key: &str) -> Result<Option<Secret>, PortError> {
        match self.entry(key)?.get_password() {
            Ok(value) => Ok(Some(Secret::new(value))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(error(e)),
        }
    }

    fn set(&self, key: &str, value: &Secret) -> Result<(), PortError> {
        self.entry(key)?.set_password(value.expose()).map_err(error)
    }

    fn delete(&self, key: &str) -> Result<(), PortError> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(error(e)),
        }
    }
}

/// Error del llavero para el usuario, sin datos de la clave.
fn error(e: keyring::Error) -> PortError {
    let detail = match &e {
        // Lleva los bytes guardados: no se muestran.
        keyring::Error::BadEncoding(_) => "la clave guardada no es texto".to_string(),
        keyring::Error::NoStorageAccess(_) => "no se puede abrir el llavero".to_string(),
        keyring::Error::PlatformFailure(_) => "el llavero del sistema no responde".to_string(),
        keyring::Error::TooLong(..) => "la clave es demasiado larga".to_string(),
        _ => "error del llavero".to_string(),
    };
    PortError::Unavailable(format!("llavero del sistema: {detail}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Con el llavero de pruebas de `keyring` (en memoria), sin tocar el del
    /// sistema.
    #[test]
    fn keys_round_trip_through_the_keyring_api() {
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let secrets = KeyringSecrets::default();
        assert_eq!(secrets.get("anthropic-api-key").unwrap(), None);
        secrets
            .set("anthropic-api-key", &Secret::new("sk-ant-prueba"))
            .unwrap();
        assert_eq!(
            secrets.get("anthropic-api-key").unwrap().unwrap().expose(),
            "sk-ant-prueba"
        );
        assert_eq!(secrets.get("openai-api-key").unwrap(), None);
        secrets.delete("anthropic-api-key").unwrap();
        assert_eq!(secrets.get("anthropic-api-key").unwrap(), None);
        // Borrar lo que no existe no es un error.
        secrets.delete("anthropic-api-key").unwrap();
    }

    #[test]
    fn errors_never_carry_the_secret() {
        let e = error(keyring::Error::BadEncoding(b"sk-ant-SECRETO".to_vec()));
        assert!(!e.to_string().contains("SECRETO"), "{e}");
        assert!(!format!("{e:?}").contains("SECRETO"));
    }
}
