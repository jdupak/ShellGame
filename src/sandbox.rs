//! Isolation of the game shell: outside `/tmp`, the state directory and `/dev` nothing can be changed.
//!
//! [`confine`] applies a Landlock ruleset to a command between fork and exec, so the shell, every
//! program it starts and the game binary all inherit it. It handles the write rights of ABI 5
//! (create, delete, rename, write, truncate, device ioctl), best effort on older kernels; reading
//! and executing stay free. Newer ABIs are not requested on purpose: ABI 9 would forbid connecting to
//! unix sockets (X11, Wayland, D-Bus, ssh-agent), and signal scoping is misreported by RHEL 9.
//! Landlock cannot restrict `chmod`, so the shell scripts send a typed `chmod` through [`guard`].
//! Only Linux can confine; elsewhere [`confine`] fails and [`probe`] says no.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::game::paths::inside_tmp;

/// Makes `cmd` start confined to writing in `/tmp`, `state_dir` and `/dev`. The ruleset is built
/// here, in the calling process, which is never confined itself; the child applies it just before
/// exec. When the kernel cannot enforce it, the exec fails (with `ENOSYS`), so a command is never
/// started unconfined by accident. `restrict_self` also sets no_new_privs: `sudo` and `su` stop
/// working inside.
#[cfg(target_os = "linux")]
pub fn confine(cmd: &mut Command, state_dir: &Path) -> io::Result<()> {
    use std::os::unix::process::CommandExt;

    use landlock::{
        ABI, AccessFs, Ruleset, RulesetAttr, RulesetCreatedAttr, RulesetStatus, path_beneath_rules,
    };

    let write = AccessFs::from_write(ABI::V5);
    let writable = [Path::new("/tmp"), state_dir, Path::new("/dev")];
    let ruleset = Ruleset::default()
        .handle_access(write)
        .and_then(|ruleset| ruleset.create())
        .and_then(|ruleset| ruleset.add_rules(path_beneath_rules(writable, write)))
        .map_err(io::Error::other)?;
    let mut ruleset = Some(ruleset);
    // SAFETY: the closure runs in the forked child and only makes two system calls (no_new_privs
    // and landlock_restrict_self; no allocation unless it fails). The launcher is single-threaded
    // when it spawns, so nothing else can hold a lock in the child.
    unsafe {
        cmd.pre_exec(move || {
            match ruleset.take().and_then(|ruleset| ruleset.restrict_self().ok()) {
                Some(status) if status.ruleset != RulesetStatus::NotEnforced => Ok(()),
                _ => Err(rustix::io::Errno::NOSYS.into()),
            }
        });
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn confine(_cmd: &mut Command, _state_dir: &Path) -> io::Result<()> {
    Err(io::ErrorKind::Unsupported.into())
}

/// Whether confinement works here: `<binary> --version` runs successfully under [`confine`].
pub fn probe(binary: &Path, state_dir: &Path) -> bool {
    let mut cmd = Command::new(binary);
    cmd.arg("--version").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    confine(&mut cmd, state_dir).is_ok() && cmd.status().is_ok_and(|status| status.success())
}

/// The `chmod` guard: refuses (with the text for the player) when an argument names an existing path
/// that does not resolve to somewhere strictly inside `/tmp`. Arguments are taken relative to `cwd`,
/// symlinks are resolved; modes, options and paths that do not exist pass, and `chmod` reports them.
pub fn guard<S: AsRef<OsStr>>(cwd: &Path, args: &[S]) -> Result<(), String> {
    for arg in args.iter().map(AsRef::as_ref).filter(|arg| !arg.is_empty()) {
        let path = cwd.join(arg);
        if fs::symlink_metadata(&path).is_ok() && !inside_tmp(&path) {
            return Err(format!(
                "Hra chrání soubory mimo /tmp: chmod pro „{}“ byl zablokován.",
                Path::new(arg).display()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;

    #[test]
    fn chmod_is_allowed_only_strictly_inside_tmp() {
        let root = tempfile::Builder::new().tempdir_in("/tmp").unwrap();
        let (a, b) = (root.path().join("a"), root.path().join("a/b"));
        fs::create_dir_all(&b).unwrap();
        fs::write(b.join("file"), "").unwrap();
        symlink("/etc/hosts", a.join("out")).unwrap();
        symlink("b/file", a.join("in")).unwrap();
        let ok = |cwd: &Path, args: &[&str]| guard(cwd, args).is_ok();

        assert!(ok(&a, &["u+x", "755", "b/file", "in", "-R", "--", "missing", ""]));
        assert!(ok(&b, &["..", "../b", "file"]), "`..` stays inside /tmp");
        assert!(!ok(root.path(), &[".."]), "`..` is /tmp itself");
        assert!(!ok(&a, &["../.."]) && !ok(&b, &["../../.."]), "`..` is /tmp itself");
        assert!(!ok(root.path(), &["../.."]), "`..` leaves /tmp");
        assert!(!ok(&a, &["755", "out"]), "a link to a file in /etc");
        assert!(!ok(&a, &["/etc/hosts"]) && !ok(Path::new("/"), &["etc"]));
        assert!(guard(&a, &[OsStr::new("/etc")]).unwrap_err().contains("„/etc“"));
    }
}
