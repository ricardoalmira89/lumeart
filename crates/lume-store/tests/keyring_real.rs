//! El llavero del sistema de verdad (GNOME Keyring/KWallet, Credenciales de
//! Windows, Llavero de macOS). Se salta por defecto: necesita un llavero
//! desbloqueado y escribe en él (con un servicio aparte, y lo borra al final).
//!
//! `cargo test -p lume-store --test keyring_real -- --ignored`

use lume_app::SecretStore;
use lume_app::ports::Secret;
use lume_store::KeyringSecrets;

#[test]
#[ignore = "usa el llavero del sistema"]
fn the_system_keyring_round_trips_a_key() {
    let secrets = KeyringSecrets::with_service("lumeart-prueba");
    let name = "prueba-api-key";
    secrets.delete(name).unwrap();
    assert_eq!(secrets.get(name).unwrap(), None);
    secrets
        .set(name, &Secret::new("sk-prueba-0123456789"))
        .unwrap();
    // Otra instancia (como otra ejecución) la encuentra.
    let again = KeyringSecrets::with_service("lumeart-prueba");
    assert_eq!(
        again.get(name).unwrap().unwrap().expose(),
        "sk-prueba-0123456789"
    );
    again.delete(name).unwrap();
    assert_eq!(
        KeyringSecrets::with_service("lumeart-prueba")
            .get(name)
            .unwrap(),
        None
    );
}
