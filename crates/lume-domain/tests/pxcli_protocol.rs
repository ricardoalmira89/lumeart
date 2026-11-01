//! Reproduce una sesión real del daemon de pxcli y compara cada respuesta.
//!
//! `fixtures/pxcli-protocol.txt` se generó enviando cada línea al daemon de
//! pxcli (`pxcli daemon --headless --size 32x32`) por su socket.

use lume_domain::protocol::{Executed, handle_line};
use lume_domain::{Canvas, History, Origin};

#[test]
fn same_responses_as_pxcli_daemon() {
    let fixture = include_str!("fixtures/pxcli-protocol.txt");
    let mut canvas = Canvas::new(32, 32).unwrap();
    let mut history = History::new();
    let mut checked = 0;
    for entry in fixture
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let (request, expected) = entry
            .split_once(" => ")
            .expect("formato <petición> => <respuesta>");
        let got = match handle_line(request, &mut canvas, &mut history, Origin::Cli) {
            Executed::Reply(response) => response.to_string(),
            Executed::Stop | Executed::Export { .. } => "ok".to_string(),
        };
        assert_eq!(got, expected, "petición {request:?}");
        checked += 1;
    }
    assert!(
        checked > 60,
        "el fixture debería tener más de 60 peticiones"
    );
}
