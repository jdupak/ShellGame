//! `shellgame update`: download the release archive for this platform (`curl`), unpack it (`tar`),
//! check that the new binary runs and replace the running one. The scratch directory is next to
//! the binary, so the final `rename` stays on one filesystem and is atomic.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::env::Env;

const RELEASES: &str = "https://github.com/jdupak/ShellGame/releases/latest/download";
pub const INSTALLER: &str =
    "curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | bash";

/// Why the update failed, in Czech; `installer` says that running the installer again would help.
pub struct Failed {
    pub why: String,
    pub installer: bool,
}

fn failed(why: String, installer: bool) -> Failed {
    Failed { why, installer }
}

/// `shellgame-<linux|macos>-<x86_64|aarch64>.tar.gz`
fn asset() -> String {
    let os = if cfg!(target_os = "macos") { "macos" } else { "linux" };
    let arch = if cfg!(target_arch = "aarch64") { "aarch64" } else { "x86_64" };
    format!("shellgame-{os}-{arch}.tar.gz")
}

/// Runs a tool and returns its stdout; a failure says `what` went wrong and carries the tool's own
/// words. A missing `curl` or `tar` is no case for the installer, which needs them too.
fn tool(program: &str, command: &mut Command, what: &str) -> Result<String, Failed> {
    let output = command.stdin(Stdio::null()).output().map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => failed(
            format!("Pro aktualizaci je potřeba příkaz '{program}', ale nebyl nalezen."),
            false,
        ),
        _ => failed(format!("{what}: {error}"), false),
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = Some(stderr.trim())
            .filter(|text| !text.is_empty())
            .map_or_else(|| output.status.to_string(), str::to_owned);
        return Err(failed(format!("{what}: {detail}"), false));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The new version line, or `None` when the downloaded binary is the one already running.
pub fn run(env: &Env, old: &str) -> Result<Option<String>, Failed> {
    let unknown = || failed("Nepodařilo se zjistit, kde je ShellGame nainstalován.".into(), false);
    let exe = env.exe.as_deref().ok_or_else(unknown)?;
    let dir = exe.parent().ok_or_else(unknown)?;
    let url = env.update_url.clone().unwrap_or_else(|| format!("{RELEASES}/{}", asset()));
    let scratch = tempfile::Builder::new().prefix(".shellgame-update-").tempdir_in(dir).map_err(|error| {
        failed(format!("Do adresáře {} nelze zapisovat, ShellGame se proto nemůže sám aktualizovat ({error}).", dir.display()), true)
    })?;
    let archive = scratch.path().join(asset());
    tool(
        "curl",
        // A dead connection must not hang the player's terminal.
        Command::new("curl")
            .args(["-fsSL", "--connect-timeout", "20", "--max-time", "300", "--retry", "2", "-o"])
            .arg(&archive)
            .arg(&url),
        &format!("Stažení {url} se nepodařilo"),
    )?;
    tool(
        "tar",
        Command::new("tar")
            .arg("-xzf")
            .arg(&archive)
            .arg("-C")
            .arg(scratch.path())
            .arg("shellgame"),
        "Rozbalení staženého archivu se nepodařilo",
    )?;
    let fresh = scratch.path().join("shellgame");
    fs::set_permissions(&fresh, fs::Permissions::from_mode(0o755))
        .map_err(|error| failed(error.to_string(), false))?;
    let new = version_of(&fresh)?;
    if new == old {
        return Ok(None);
    }
    fs::rename(&fresh, exe).map_err(|error| {
        failed(format!("Nahrazení programu {} se nepodařilo: {error}", exe.display()), true)
    })?;
    Ok(Some(new))
}

fn version_of(binary: &Path) -> Result<String, Failed> {
    let text =
        tool("shellgame", Command::new(binary).arg("--version"), "Stažený program nelze spustit")?;
    match text.lines().next().map(str::trim) {
        Some(line) if !line.is_empty() => Ok(line.to_owned()),
        _ => Err(failed("Stažený program nelze spustit: `--version` nic nevypsalo".into(), false)),
    }
}
