//! Raíz de composición de LumeArt.
//!
//! Lee la configuración, crea los adaptadores, los inyecta en `lume-app` y
//! arranca la forma pedida: studio (ventana), sin ventana (`start --headless`),
//! servidor MCP (`mcp`) o un comando de la CLI.

mod agent;
mod anim_cmd;
mod daemon;
mod eval_cmd;
mod mcp;
mod setup;
mod vec_cmd;
mod vectorcraft;

use clap::Parser;
use lume_cli::{Cli, Command, Context};

fn main() {
    let cli = Cli::parse();
    let state = state_path(&cli);
    let Some(command) = cli.command else {
        // Sin comando: el studio.
        std::process::exit(daemon::studio(state.as_deref()));
    };
    if let Command::Mcp(_) = command {
        // El transporte MCP lee y escribe la entrada y la salida estándar por
        // su cuenta: no se pueden bloquear aquí.
        let cwd = std::env::current_dir().unwrap_or_default();
        std::process::exit(mcp::run(state.as_deref(), cwd));
    }
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    let mut ctx = Context {
        cwd: std::env::current_dir().unwrap_or_default(),
        session_env: std::env::var(lume_domain::discovery::SESSION_ENV).ok(),
        agent_run: std::env::var(lume_domain::discovery::RUN_ENV)
            .ok()
            .and_then(|v| v.trim().parse().ok()),
        default_session: lume_api::default_state_path(),
        stdin: &mut stdin,
        stdout: &mut stdout,
        stderr: &mut stderr,
    };
    let code = match &command {
        Command::Start(_) | Command::Stop(_) | Command::Serve(_) => {
            match lume_cli::session_path(cli.session.as_deref(), &ctx) {
                Err(e) => {
                    let _ = std::io::Write::write_fmt(ctx.stderr, format_args!("Error: {}\n", e.0));
                    1
                }
                Ok(state) => match &command {
                    Command::Start(args) => daemon::start(args, &state, ctx.stdout, ctx.stderr),
                    Command::Stop(_) => daemon::stop(&state, ctx.stdout, ctx.stderr),
                    Command::Serve(args) => daemon::serve(args, &state),
                    _ => unreachable!(),
                },
            }
        }
        Command::Init(args) => init(args, &ctx.cwd, ctx.stdout, ctx.stderr),
        Command::Anim(args) => anim_cmd::run(args, ctx.stdout, ctx.stderr),
        Command::Vec(args) => vec_cmd::run(args, ctx.stdout, ctx.stderr),
        Command::Eval(args) => {
            let mut stdin = std::io::BufReader::new(&mut *ctx.stdin);
            eval_cmd::run(args, &mut stdin, ctx.stdout, ctx.stderr)
        }
        Command::Setup(args) => match setup::SetupEnv::detect() {
            Ok(env) => match &args.print {
                Some(client) => setup::print(&env, client, ctx.stdout),
                None => setup::setup(&env, args.dry_run, ctx.stdout),
            },
            Err(e) => {
                let _ = writeln!(ctx.stderr, "Error: {e}");
                1
            }
        },
        other => lume_cli::execute(other, cli.session.as_deref(), &mut ctx),
    };
    std::process::exit(code);
}

/// Archivo de estado: `--session`, `LUMEART_SESSION` o el de por defecto.
fn state_path(cli: &Cli) -> Option<std::path::PathBuf> {
    cli.session
        .clone()
        .or_else(|| {
            std::env::var(lume_domain::discovery::SESSION_ENV)
                .ok()
                .filter(|s| !s.trim().is_empty())
                .map(Into::into)
        })
        .or_else(lume_api::default_state_path)
}

/// `lumeart init`: crea la guía de arte en la carpeta actual.
fn init(
    args: &lume_cli::InitArgs,
    cwd: &std::path::Path,
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> i32 {
    let created = lume_cli::guide_from_init(args)
        .map_err(|e| e.0)
        .and_then(|guide| {
            lume_store::guide::init_guide(cwd, &guide, args.force)
                .map(|path| (guide, path))
                .map_err(|e| e.to_string())
        });
    match created {
        Ok((guide, path)) => {
            let _ = writeln!(stdout, "Guía de arte creada en {}", path.display());
            let _ = writeln!(
                stdout,
                "Los recursos irán a {}/. Edita el archivo para cambiar el estilo, el tamaño o la paleta.",
                guide.output
            );
            0
        }
        Err(message) => {
            let _ = writeln!(stderr, "Error: {message}");
            1
        }
    }
}
