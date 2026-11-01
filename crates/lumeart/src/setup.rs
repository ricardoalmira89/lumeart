//! `lumeart setup`: registra el servidor MCP (`lumeart mcp`) en Claude Code
//! y en Codex y copia la skill `lumeart` donde la buscan.
//!
//! Antes de tocar nada mira qué hay: si el servidor ya está registrado con el
//! mismo comando o la skill ya está al día, no hace nada; así se puede repetir
//! sin duplicar. Para escribir usa las órdenes de cada agente (`claude mcp
//! add --scope user`, `codex mcp add`); para mirar, sus archivos
//! (`~/.claude.json`, `~/.codex/config.toml`). Con `--dry-run` dice
//! exactamente qué haría y no cambia nada.

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// La skill, dentro del ejecutable.
pub const SKILL: &str = include_str!("../../../skills/lumeart/SKILL.md");

/// Nombre del servidor MCP y de la carpeta de la skill.
pub const NAME: &str = "lumeart";

/// Dónde está cada cosa en esta máquina.
#[derive(Debug, Clone)]
pub struct SetupEnv {
    /// Comando para lanzar LumeArt (`lumeart` si está en el PATH; si no, la
    /// ruta de este ejecutable).
    pub lumeart: String,
    pub claude: Option<PathBuf>,
    pub codex: Option<PathBuf>,
    /// Configuración de usuario de Claude Code (`~/.claude.json`).
    pub claude_config: PathBuf,
    /// Skills de Claude Code (`~/.claude/skills`).
    pub claude_skills: PathBuf,
    /// Configuración de Codex (`~/.codex/config.toml`).
    pub codex_config: PathBuf,
    /// Skills de Codex y otros agentes (`~/.agents/skills`).
    pub agents_skills: PathBuf,
}

impl SetupEnv {
    /// Lo de esta máquina (respeta `CLAUDE_CONFIG_DIR` y `CODEX_HOME`).
    pub fn detect() -> Result<SetupEnv, String> {
        let home = directories::BaseDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .ok_or("no se encontró la carpeta personal")?;
        let env_dir = |name: &str| {
            std::env::var_os(name)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        };
        let (claude_config, claude_dir) = match env_dir("CLAUDE_CONFIG_DIR") {
            Some(dir) => (dir.join(".claude.json"), dir),
            None => (home.join(".claude.json"), home.join(".claude")),
        };
        let codex_home = env_dir("CODEX_HOME").unwrap_or_else(|| home.join(".codex"));
        Ok(SetupEnv {
            lumeart: lumeart_command(),
            claude: lume_agent::find_claude(),
            codex: lume_agent::find_codex(),
            claude_config,
            claude_skills: claude_dir.join("skills"),
            codex_config: codex_home.join("config.toml"),
            agents_skills: home.join(".agents").join("skills"),
        })
    }
}

/// `lumeart` si el del PATH es este mismo ejecutable; si no, su ruta.
fn lumeart_command() -> String {
    let exe = std::env::current_exe().ok();
    let same = |a: &Path, b: &Path| {
        std::fs::canonicalize(a)
            .ok()
            .is_some_and(|a| std::fs::canonicalize(b).ok() == Some(a))
    };
    let in_path = lume_agent::claude_bin::find_program_in(
        NAME,
        &std::env::var_os("PATH").unwrap_or_default(),
        &[],
    );
    match (exe, in_path) {
        (Some(exe), Some(found)) if same(&exe, &found) => NAME.to_string(),
        (Some(exe), _) => exe.to_string_lossy().into_owned(),
        (None, _) => NAME.to_string(),
    }
}

/// Cómo está registrado ahora el servidor en un agente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Registered {
    No,
    /// Con este mismo comando.
    Same,
    /// Con otro comando (el que tiene).
    Other(String),
}

fn compare(command: Option<&str>, args: Vec<String>, wanted: &str) -> Registered {
    let Some(command) = command else {
        return Registered::Other("(sin comando)".into());
    };
    if command == wanted && args == ["mcp"] {
        Registered::Same
    } else {
        let mut shown = vec![command.to_string()];
        shown.extend(args);
        Registered::Other(shown.join(" "))
    }
}

/// `mcpServers.lumeart` de `~/.claude.json` (ámbito de usuario).
pub fn claude_registration(config: &Path, wanted: &str) -> Registered {
    let Some(server) = std::fs::read(config)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|json| json.get("mcpServers")?.get(NAME).cloned())
    else {
        return Registered::No;
    };
    let args = server["args"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    compare(server["command"].as_str(), args, wanted)
}

/// `[mcp_servers.lumeart]` de `~/.codex/config.toml`.
pub fn codex_registration(config: &Path, wanted: &str) -> Registered {
    let Some(server) = std::fs::read_to_string(config)
        .ok()
        .and_then(|text| text.parse::<toml::Table>().ok())
        .and_then(|table| table.get("mcp_servers")?.get(NAME).cloned())
    else {
        return Registered::No;
    };
    let args = server
        .get("args")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    compare(server.get("command").and_then(|c| c.as_str()), args, wanted)
}

/// Un paso que cambia algo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Ejecutar un programa con sus argumentos.
    Run(Vec<OsString>),
    /// Escribir la skill (crear o actualizar).
    Skill { path: PathBuf, update: bool },
}

/// Una orden como se escribiría en una terminal.
pub fn shell_line(args: &[OsString]) -> String {
    args.iter()
        .map(|a| {
            let a = a.to_string_lossy();
            if !a.is_empty()
                && a.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_./:=@+".contains(c))
            {
                a.into_owned()
            } else {
                format!("'{}'", a.replace('\'', r"'\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Un agente: su nombre, su programa y sus pasos (o por qué no hay).
struct Plan {
    title: &'static str,
    program: Option<PathBuf>,
    /// Líneas que solo informan («ya está…»).
    notes: Vec<String>,
    steps: Vec<Step>,
}

fn skill_step(dir: &Path, notes: &mut Vec<String>, steps: &mut Vec<Step>) {
    let path = dir.join(NAME).join("SKILL.md");
    match std::fs::read_to_string(&path) {
        Ok(text) if text == SKILL => {
            notes.push(format!("Skill: ya está al día en {}", path.display()))
        }
        Ok(_) => steps.push(Step::Skill { path, update: true }),
        Err(_) => steps.push(Step::Skill {
            path,
            update: false,
        }),
    }
}

fn os(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

fn plans(env: &SetupEnv) -> Vec<Plan> {
    let mut out = Vec::new();

    let mut claude = Plan {
        title: "Claude Code",
        program: env.claude.clone(),
        notes: Vec::new(),
        steps: Vec::new(),
    };
    if let Some(program) = &env.claude {
        let mut add = vec![program.clone().into_os_string()];
        add.extend(os(&[
            "mcp",
            "add",
            "--scope",
            "user",
            NAME,
            "--",
            &env.lumeart,
            "mcp",
        ]));
        match claude_registration(&env.claude_config, &env.lumeart) {
            Registered::Same => claude
                .notes
                .push(format!("MCP: ya está registrado ({} mcp)", env.lumeart)),
            Registered::No => claude.steps.push(Step::Run(add)),
            Registered::Other(old) => {
                claude
                    .notes
                    .push(format!("MCP: estaba registrado con otro comando ({old})"));
                let mut remove = vec![program.clone().into_os_string()];
                remove.extend(os(&["mcp", "remove", "--scope", "user", NAME]));
                claude.steps.push(Step::Run(remove));
                claude.steps.push(Step::Run(add));
            }
        }
        skill_step(&env.claude_skills, &mut claude.notes, &mut claude.steps);
    }
    out.push(claude);

    let mut codex = Plan {
        title: "Codex",
        program: env.codex.clone(),
        notes: Vec::new(),
        steps: Vec::new(),
    };
    if let Some(program) = &env.codex {
        let mut add = vec![program.clone().into_os_string()];
        add.extend(os(&["mcp", "add", NAME, "--", &env.lumeart, "mcp"]));
        match codex_registration(&env.codex_config, &env.lumeart) {
            Registered::Same => codex
                .notes
                .push(format!("MCP: ya está registrado ({} mcp)", env.lumeart)),
            Registered::No => codex.steps.push(Step::Run(add)),
            Registered::Other(old) => {
                codex
                    .notes
                    .push(format!("MCP: estaba registrado con otro comando ({old})"));
                let mut remove = vec![program.clone().into_os_string()];
                remove.extend(os(&["mcp", "remove", NAME]));
                codex.steps.push(Step::Run(remove));
                codex.steps.push(Step::Run(add));
            }
        }
        skill_step(&env.agents_skills, &mut codex.notes, &mut codex.steps);
    }
    out.push(codex);
    out
}

fn write_skill(path: &Path) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, SKILL)
}

fn run(args: &[OsString]) -> Result<(), String> {
    let output = std::process::Command::new(&args[0])
        .args(&args[1..])
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = stderr
        .lines()
        .chain(stdout.lines())
        .find(|l| !l.trim().is_empty())
        .unwrap_or("sin mensaje")
        .trim()
        .to_string();
    Err(format!("{} ({detail})", output.status))
}

/// `lumeart setup`. Devuelve el código de salida.
pub fn setup(env: &SetupEnv, dry_run: bool, out: &mut dyn Write) -> i32 {
    let plans = plans(env);
    let mut failed = false;
    let mut found = 0;
    let mut changes = 0;
    for plan in &plans {
        let Some(program) = &plan.program else {
            let name = if plan.title == "Codex" {
                "codex"
            } else {
                "claude"
            };
            let _ = writeln!(
                out,
                "{}: no está instalado (no se encontró `{name}`); me lo salto.",
                plan.title
            );
            continue;
        };
        found += 1;
        let _ = writeln!(out, "{} ({}):", plan.title, program.display());
        for note in &plan.notes {
            let _ = writeln!(out, "  {note}");
        }
        for step in &plan.steps {
            changes += 1;
            match step {
                Step::Run(args) => {
                    let line = shell_line(args);
                    if dry_run {
                        let _ = writeln!(out, "  Ejecutaría: {line}");
                        continue;
                    }
                    match run(args) {
                        Ok(()) => {
                            let _ = writeln!(out, "  Hecho: {line}");
                        }
                        Err(e) => {
                            failed = true;
                            let _ = writeln!(out, "  Falló: {line}: {e}");
                        }
                    }
                }
                Step::Skill { path, update } => {
                    let (would, done) = if *update {
                        ("Actualizaría", "Skill actualizada")
                    } else {
                        ("Crearía", "Skill creada")
                    };
                    if dry_run {
                        let _ = writeln!(out, "  {would} {}", path.display());
                        continue;
                    }
                    match write_skill(path) {
                        Ok(()) => {
                            let _ = writeln!(out, "  {done}: {}", path.display());
                        }
                        Err(e) => {
                            failed = true;
                            let _ = writeln!(out, "  Falló al escribir {}: {e}", path.display());
                        }
                    }
                }
            }
        }
    }
    if found == 0 {
        let _ = writeln!(
            out,
            "No se encontró ni Claude Code ni Codex: instala uno y vuelve a ejecutar `lumeart setup`."
        );
        return 1;
    }
    if dry_run {
        let _ = writeln!(out, "--dry-run: no se ha cambiado nada.");
    } else if changes == 0 {
        let _ = writeln!(out, "Todo estaba ya listo.");
    }
    i32::from(failed)
}

/// `lumeart setup --print [CLIENTE]`: la configuración para registrarlo a
/// mano en Claude Code o en Codex (o las dos), con la ruta de `lumeart` de
/// este equipo, o la skill entera con `skill`. No cambia nada.
pub fn print(env: &SetupEnv, client: &str, out: &mut dyn Write) -> i32 {
    let command = &env.lumeart;
    let json_command = serde_json::to_string(command).unwrap_or_default();
    let claude = || {
        format!(
            "## Claude Code

Regístralo (para todos tus proyectos):

    claude mcp add --scope user {name} -- {line} mcp

o añade esto a `mcpServers` en {config}:

    \"{name}\": {{ \"type\": \"stdio\", \"command\": {json_command}, \"args\": [\"mcp\"] }}

Instrucciones (la skill): `lumeart setup` la copia; a mano:

    mkdir -p {dir} && {line} setup --print skill > {skill}

Después, en Claude Code: /mcp para conectarlo.
",
            name = NAME,
            line = shell_line(&[OsString::from(command)]),
            config = env.claude_config.display(),
            skill = env.claude_skills.join(NAME).join("SKILL.md").display(),
            dir = env.claude_skills.join(NAME).display(),
        )
    };
    let codex = || {
        format!(
            "## Codex

Regístralo:

    codex mcp add {name} -- {line} mcp

o añade esto a {config}:

    [mcp_servers.{name}]
    command = {json_command}
    args = [\"mcp\"]

Instrucciones (la skill): `lumeart setup` la copia; a mano:

    mkdir -p {dir} && {line} setup --print skill > {skill}

Después, abre una sesión nueva de Codex.
",
            name = NAME,
            line = shell_line(&[OsString::from(command)]),
            config = env.codex_config.display(),
            skill = env.agents_skills.join(NAME).join("SKILL.md").display(),
            dir = env.agents_skills.join(NAME).display(),
        )
    };
    let text = match client.trim().to_lowercase().as_str() {
        "todos" | "all" | "" => format!("{}\n{}", claude(), codex()),
        "claude" | "claude-code" | "claude_code" => claude(),
        "codex" => codex(),
        "skill" => SKILL.to_string(),
        other => {
            let _ = writeln!(
                out,
                "No conozco «{other}»: de momento, claude-code, codex o skill."
            );
            return 1;
        }
    };
    let _ = write!(out, "{text}");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registrations_are_read_from_each_config() {
        let dir = tempfile::tempdir().unwrap();
        let claude = dir.path().join(".claude.json");
        assert_eq!(claude_registration(&claude, "lumeart"), Registered::No);
        std::fs::write(
            &claude,
            r#"{"mcpServers": {"otro": {"command": "x"}, "lumeart": {"type": "stdio", "command": "lumeart", "args": ["mcp"]}}}"#,
        )
        .unwrap();
        assert_eq!(claude_registration(&claude, "lumeart"), Registered::Same);
        assert_eq!(
            claude_registration(&claude, "/opt/lumeart"),
            Registered::Other("lumeart mcp".into())
        );

        let codex = dir.path().join("config.toml");
        assert_eq!(codex_registration(&codex, "lumeart"), Registered::No);
        std::fs::write(
            &codex,
            "model = \"o3\"\n[mcp_servers.otro]\ncommand = \"y\"\n[mcp_servers.lumeart]\ncommand = \"/viejo/lumeart\"\nargs = [\"mcp\"]\n",
        )
        .unwrap();
        assert_eq!(
            codex_registration(&codex, "lumeart"),
            Registered::Other("/viejo/lumeart mcp".into())
        );
        assert_eq!(
            codex_registration(&codex, "/viejo/lumeart"),
            Registered::Same
        );
        // Un archivo roto cuenta como «no registrado» (y la orden lo dirá).
        std::fs::write(&codex, "esto no es toml [").unwrap();
        assert_eq!(codex_registration(&codex, "lumeart"), Registered::No);
    }

    #[test]
    fn commands_are_shown_as_in_a_terminal() {
        assert_eq!(
            shell_line(&os(&[
                "/usr/bin/claude",
                "mcp",
                "add",
                "--",
                "/mi juego/lumeart",
                "mcp"
            ])),
            "/usr/bin/claude mcp add -- '/mi juego/lumeart' mcp"
        );
        assert_eq!(shell_line(&os(&["a'b"])), r"'a'\''b'");
    }

    #[test]
    fn the_skill_names_every_mcp_tool() {
        for tool in lume_mcp::TOOLS {
            assert!(
                SKILL.contains(&format!("`{tool}`")),
                "la skill no nombra `{tool}`"
            );
        }
        assert!(SKILL.starts_with("---\nname: lumeart\ndescription: "));
        assert!(SKILL.contains("lumeart init"));
    }
}
