//! `lumeart start` y `stop` con el binario real, sin ventana. Cada test usa
//! su propio archivo de estado en una carpeta temporal (`--session`), así que
//! no toca la sesión del usuario. La API escucha solo en 127.0.0.1.

mod common;

use common::{Lumeart, text};

#[test]
fn start_draw_export_stop() {
    let dir = tempfile::tempdir().unwrap();
    let lumeart = Lumeart::new(dir.path());

    let out = lumeart.run(&["start", "--headless", "--size", "8x6"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let pid: u32 = text(&out.stdout)
        .trim()
        .parse()
        .expect("start imprime el PID");
    let info: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&lumeart.state).unwrap()).unwrap();
    assert_eq!(info["pid"], pid);

    for args in [
        &["fill_rect", "0", "0", "8", "6", "#102030"][..],
        &["line", "0", "0", "7", "5", "red"],
        &["set_pixel", "7", "0", "#00ff0080"],
    ] {
        let out = lumeart.run(args);
        assert_eq!(text(&out.stdout), "ok\n", "{args:?}: {}", text(&out.stderr));
    }
    let out = lumeart.run(&["get_pixel", "7", "0"]);
    assert_eq!(text(&out.stdout), "ok #00ff0080\n");
    let out = lumeart.run(&["export", "--scale", "2", "sprite.png"]);
    assert_eq!(text(&out.stdout), "ok\n", "{}", text(&out.stderr));

    let out = lumeart.run(&["stop"]);
    assert_eq!(
        (out.status.code(), text(&out.stdout)),
        (Some(0), "ok\n".to_string()),
        "{}",
        text(&out.stderr)
    );
    assert!(
        !lumeart.state.exists(),
        "stop deja borrado el archivo de estado"
    );

    // El PNG es correcto: 16×12, con los colores en su sitio (y alfa intacto).
    let file = std::fs::File::open(dir.path().join("sprite.png")).unwrap();
    let mut reader = png::Decoder::new(std::io::BufReader::new(file))
        .read_info()
        .unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut buf).unwrap();
    assert_eq!((frame.width, frame.height), (16, 12));
    let px = |x: usize, y: usize| buf[(y * 16 + x) * 4..(y * 16 + x) * 4 + 4].to_vec();
    assert_eq!(px(0, 0), vec![255, 0, 0, 255]);
    assert_eq!(px(15, 11), vec![255, 0, 0, 255]);
    assert_eq!(px(2, 10), vec![0x10, 0x20, 0x30, 255]);
    assert_eq!(px(15, 0), vec![0, 255, 0, 0x80]);

    // Después de stop, los comandos dicen que no hay sesión.
    let out = lumeart.run(&["get_pixel", "0", "0"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stderr).contains("daemon_not_running"));
}

#[test]
fn second_start_reports_an_open_session() {
    let dir = tempfile::tempdir().unwrap();
    let lumeart = Lumeart::new(dir.path());
    let out = lumeart.run(&["start", "--headless"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let first_pid = text(&out.stdout).trim().to_string();

    let out = lumeart.run(&["start", "--headless"]);
    assert_eq!(out.status.code(), Some(1));
    let stderr = text(&out.stderr);
    assert!(
        stderr.starts_with("Error: err daemon_already_running ya hay una sesión abierta"),
        "{stderr}"
    );
    assert!(stderr.contains(&format!("pid {first_pid}")), "{stderr}");
    // La primera sigue viva.
    assert_eq!(
        text(&lumeart.run(&["get_pixel", "0", "0"]).stdout),
        "ok #00000000\n"
    );
}

#[test]
fn orphan_state_is_cleaned_on_start() {
    let dir = tempfile::tempdir().unwrap();
    let lumeart = Lumeart::new(dir.path());
    // Un puerto en el que ya no escucha nadie.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    std::fs::create_dir_all(lumeart.state.parent().unwrap()).unwrap();
    std::fs::write(
        &lumeart.state,
        format!(r#"{{"port": {port}, "pid": 4000000, "token": "viejo", "version": "0.0.0"}}"#),
    )
    .unwrap();

    let out = lumeart.run(&["start", "--headless"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let pid = text(&out.stdout).trim().to_string();
    let info: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&lumeart.state).unwrap()).unwrap();
    assert_eq!(info["pid"].to_string(), pid);
    assert_ne!(info["token"], "viejo");

    // Un archivo de estado roto también se limpia.
    let out = lumeart.run(&["stop"]);
    assert_eq!(out.status.code(), Some(0));
    std::fs::write(&lumeart.state, "esto no es json").unwrap();
    let out = lumeart.run(&["start", "--headless"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
}

#[test]
fn stop_without_a_session() {
    let dir = tempfile::tempdir().unwrap();
    let lumeart = Lumeart::new(dir.path());
    let out = lumeart.run(&["stop"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).starts_with("Error: err daemon_not_running"),
        "{}",
        text(&out.stderr)
    );

    // Con un estado huérfano, stop lo borra y lo dice.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    std::fs::create_dir_all(lumeart.state.parent().unwrap()).unwrap();
    std::fs::write(
        &lumeart.state,
        format!(r#"{{"port": {port}, "pid": 1, "token": "x", "version": "0"}}"#),
    )
    .unwrap();
    let out = lumeart.run(&["stop"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("stale state removed"),
        "{}",
        text(&out.stderr)
    );
    assert!(!lumeart.state.exists());
}

#[test]
fn start_validates_its_options() {
    let dir = tempfile::tempdir().unwrap();
    let lumeart = Lumeart::new(dir.path());
    // Sin --headless y sin pantalla (solo se puede simular en Linux).
    #[cfg(target_os = "linux")]
    {
        let out = lumeart.run_without_display(&["start"]);
        assert_eq!(out.status.code(), Some(1));
        assert!(
            text(&out.stderr).contains("err renderer_unavailable"),
            "{}",
            text(&out.stderr)
        );
    }
    let out = lumeart.run(&["start", "--headless", "--size", "8"]);
    assert_eq!(
        text(&out.stderr),
        "Error: invalid size \"8\": expected WxH\n"
    );
    let out = lumeart.run(&["start", "--headless", "--size", "600x8"]);
    assert_eq!(
        text(&out.stderr),
        "Error: err invalid_args width and height must be between 1 and 512\n"
    );
    assert!(!lumeart.state.exists());
}
