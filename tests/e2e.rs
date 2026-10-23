//! The course against the real game shell: the real binary, integration script and hooks in ONE bash
//! or fish process per trial, confined by Landlock where the kernel has it (so no level may need to
//! write outside /tmp). One trial per level and shell, named like `solutions::3.2::bash`; run one
//! with `cargo test --test e2e -- solutions::3.2::bash`.
//!
//! * `solutions`: `shellgame resume`, an empty submit must not complete the level, its solution must,
//!   then the same again over the dirty workspace (what `reset` and `resume` meet). Each line of the
//!   solution is a step of its own: it may fail, but never with 126 or 127 (a command that is not
//!   there or cannot run). bash runs under `umask 077`.
//! * `playthrough`: a fresh game, every solution in course order, no resumes.
//! * `strict_cd`: levels with a `cd` policy never lock the player in: a wrong move is refused, the
//!   way back to the start is always open, the drill frees every move and `reset` restores the rule.
//! * `regression`: the traps found in single levels, and a player who deletes the directory they
//!   stand in or takes the `x` away from the one above it. Each finds its level in the course; a
//!   level it cannot find fails that trial only.
//! * `harness`: the replay itself fails a wrong solution.
//!
//! fish is optional; below fish 3.4 a trial whose solution uses `$(...)` is skipped with a note.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use libtest_mimic::{Arguments, Completion, Failed, Trial};
use shellgame::game::messages::{CD_RECOVERY_TIP, CWD_MISSING};
use shellgame::game::{Ctx, Pos, dedent, prepare, start_dir};
use shellgame::shell::ShellKind::{self, Bash, Fish};
use support::{ShellRun, Step, World, landlock, squash};

/// What a step of a scene must do. Everything but `Blocked` and `Runs` ends with status 0; a game
/// command (`shellgame ...`) must not write to stderr, where the player would see it.
enum Want {
    Ok,
    /// A line of a solution: it may fail (some do on purpose), but its command must exist and run.
    Runs,
    /// And the player stands in this directory afterwards.
    At(PathBuf),
    /// The `cd` is refused: status 1, the recovery tip and this text on stderr, the player stays.
    Blocked(&'static str),
    /// The level is (not) done in the saved game afterwards.
    Done(bool),
    /// This text is in what the step printed.
    Says(&'static str),
    /// This text is not in what the step printed.
    Lacks(&'static str),
}

type Scene = Vec<(String, Want)>;
/// A regression scene for the level at `pos`, built in a world.
type Build = fn(&World, ShellKind, Pos) -> Scene;
/// Picks the level of a regression scene from the course.
type Find = fn() -> Result<Pos, String>;

impl Step {
    fn done(&self, pos: Pos) -> bool {
        !self.state["levels"][pos.to_string()]["done"].is_null()
    }
}

impl ShellRun {
    /// The run itself was sound: the shell exited normally after the last step and printed nothing of
    /// its own, no protocol line or panic reached the player, every step left the shell in a
    /// directory that exists (a `reset` that rebuilds the player's directory must move them) and
    /// there was a saved game throughout.
    fn check(&self) -> Result<(), String> {
        if !self.finished || self.code != 0 || !self.noise.is_empty() {
            let at =
                self.steps.iter().position(|step| step.status < 0).unwrap_or(self.steps.len() - 1);
            return Err(self.report(
                at,
                &format!(
                    "the shell stopped early or printed itself: {} {:?}",
                    self.code, self.noise
                ),
            ));
        }
        for (n, step) in self.steps.iter().enumerate() {
            let why = if (step.out.clone() + &step.err).contains("__SHELLGAME_EXEC__") {
                "a protocol line leaked to the player"
            } else if step.err.contains("panicked at") {
                "the game panicked"
            } else if !step.alive {
                "the shell was left in a deleted directory"
            } else if step.state.is_null() {
                "there is no saved game"
            } else {
                continue;
            };
            return Err(self.report(n, why));
        }
        Ok(())
    }

    /// `why`, and what step `n` did.
    fn report(&self, n: usize, why: &str) -> String {
        let Step { cmd, status, pwd, out, err, .. } = &self.steps[n];
        format!(
            "{why}\n[{n}] {cmd}\n  status {status}, in {}\n  stdout: {}\n  stderr: {}",
            pwd.display(),
            squash(out),
            squash(err)
        )
    }
}

fn ok(cmd: impl Into<String>) -> (String, Want) {
    (cmd.into(), Want::Ok)
}

fn runs(cmd: impl Into<String>) -> (String, Want) {
    (cmd.into(), Want::Runs)
}

fn at(cmd: impl Into<String>, dir: &Path) -> (String, Want) {
    (cmd.into(), Want::At(dir.to_owned()))
}

fn blocked(cmd: impl Into<String>, text: &'static str) -> (String, Want) {
    (cmd.into(), Want::Blocked(text))
}

fn done(cmd: impl Into<String>, done: bool) -> (String, Want) {
    (cmd.into(), Want::Done(done))
}

fn says(cmd: impl Into<String>, text: &'static str) -> (String, Want) {
    (cmd.into(), Want::Says(text))
}

fn lacks(cmd: impl Into<String>, text: &'static str) -> (String, Want) {
    (cmd.into(), Want::Lacks(text))
}

/// Plays the scene in one shell of a new world (bash under `umask 077`) and judges every step.
fn play(kind: ShellKind, pos: Pos, build: impl FnOnce(&World) -> Scene) -> Result<(), String> {
    let world = World::new();
    let scene = build(&world);
    let cmds: Vec<&str> = scene.iter().map(|(cmd, _)| cmd.as_str()).collect();
    let run = world.shell(kind, if kind == Bash { "umask 077" } else { "" }, &cmds);
    run.check()?;
    let tip = squash(CD_RECOVERY_TIP);
    for (n, ((cmd, want), step)) in scene.iter().zip(&run.steps).enumerate() {
        let refused = squash(&step.err);
        let moved = n > 0 && step.pwd != run.steps[n - 1].pwd;
        let why = match want {
            Want::Blocked(text) if !refused.contains(&tip) || !refused.contains(text) => {
                "not refused with the tip"
            }
            Want::Blocked(_) if step.status != 1 => "a refused `cd` must fail with status 1",
            Want::Blocked(_) if moved => "a refused `cd` must not move the player",
            Want::Blocked(_) => continue,
            Want::Runs if matches!(step.status, 126 | 127) => {
                "the command was not found or could not run (status 126 or 127)"
            }
            Want::Ok | Want::At(_) | Want::Done(_) | Want::Says(_) | Want::Lacks(_)
                if step.status != 0 =>
            {
                "the step failed"
            }
            Want::At(dir) if step.pwd != *dir => "the player is in the wrong directory",
            Want::Done(done) if step.done(pos) != *done => {
                ["done too early", "not done"][*done as usize]
            }
            Want::Says(text) if !squash(&step.out).contains(&squash(text)) => {
                "the output lacks the text"
            }
            Want::Lacks(text) if squash(&step.out).contains(&squash(text)) => {
                "the output has the text"
            }
            _ if cmd.starts_with("shellgame ") && !step.err.is_empty() => {
                "the game wrote to stderr"
            }
            _ => continue,
        };
        return Err(run.report(n, why));
    }
    Ok(())
}

/// One shell word that bash and fish read alike (the world's paths have spaces, never quotes).
fn quote(path: &Path) -> String {
    assert!(!path.to_str().unwrap().contains(['\'', '\\']));
    format!("'{}'", path.display())
}

/// Moves the shell without asking the game's `cd` hook, as the `shellgame` function teleports.
fn teleport(kind: ShellKind, dir: &Path) -> String {
    format!("{} -- {}", if kind == Bash { "builtin cd" } else { "__shellgame_cd" }, quote(dir))
}

fn resume(pos: Pos) -> String {
    format!("shellgame resume {pos}")
}

fn solution(pos: Pos) -> String {
    dedent(pos.level().solution)
}

/// The solution without its final `shellgame submit`: what a player does before submitting.
fn drill(pos: Pos) -> Vec<String> {
    let lines = solution(pos).lines().map(str::to_owned).collect::<Vec<_>>();
    lines.into_iter().filter(|line| !line.starts_with("shellgame submit")).collect()
}

/// `cd /` and one `cd` per directory of `dir`: walking there a step at a time.
fn walk(dir: &Path) -> String {
    let steps =
        dir.components().skip(1).map(|part| format!("cd {}", quote(Path::new(part.as_os_str()))));
    ["cd /".to_owned()].into_iter().chain(steps).collect::<Vec<_>>().join("\n")
}

fn replay(kind: ShellKind, pos: Pos, solution: &str) -> Result<(), String> {
    // `resume` keeps the completion record, so the second round forgets it first: an empty submit must
    // not complete the re-prepared level either.
    let forget = r#"sed 's/"done": [0-9][0-9]*/"done": null/' "$SHELLGAME_STATE_DIR/state.json" > "$SHELLGAME_STATE_DIR/forget" && mv "$SHELLGAME_STATE_DIR/forget" "$SHELLGAME_STATE_DIR/state.json""#;
    // One step per line: the last one submits and completes the level, the others only have to run.
    let lines: Vec<&str> = solution.lines().filter(|line| !line.trim().is_empty()).collect();
    let Some((submit, work)) = lines.split_last() else {
        return Err("the solution is empty".into());
    };
    let round = || {
        let mut scene = vec![done(resume(pos), false), done("shellgame submit", false)];
        scene.extend(work.iter().map(|line| runs(*line)));
        scene.push(done(*submit, true));
        scene
    };
    play(kind, pos, |_| {
        let mut scene = round();
        scene.push(done(forget, false));
        scene.extend(round());
        scene
    })
}

/// A fresh game, every solution in course order with no resumes: the game advances itself, and
/// intros pass because stdin is not a terminal. In the end the game is finished.
fn playthrough(kind: ShellKind) -> Result<(), String> {
    let world = World::new();
    let all: Vec<Pos> = Pos::all().collect();
    let cmds: Vec<String> =
        ["shellgame".to_owned()].into_iter().chain(all.iter().map(|&pos| solution(pos))).collect();
    let run = world.shell(kind, "", &cmds);
    run.check()?;
    for (n, &pos) in all.iter().enumerate() {
        let (step, next) = (&run.steps[n + 1], all.get(n + 1));
        let problem = match next {
            _ if !step.done(pos) => "the solution did not complete the level",
            Some(next) if step.state["level"] != next.to_string() => {
                "the game did not move on to the next level"
            }
            Some(next) if step.pwd != start_dir(*next, &world.ws) => {
                "the shell was not moved to the next start"
            }
            None if step.state["finished"] != true => {
                "the last level is done, but the game is not finished"
            }
            _ => continue,
        };
        return Err(run.report(n + 1, &format!("level {pos}: {problem}")));
    }
    Ok(())
}

/// Levels with a `cd` policy never lock the player in. After `resume`: a move the policy refuses is
/// refused, with the recovery tip; moving to the start directory is always allowed, from the
/// workspace and the section root; once the drill is done the same move is allowed; `reset` makes
/// the level strict again.
fn strict_cd(kind: ShellKind, pos: Pos) -> Result<(), String> {
    play(kind, pos, |world| {
        let (start, section) = (start_dir(pos, &world.ws), pos.section_dir(&world.ws));
        // The binary's `cd-hook` tells which of these moves the level refuses at its start.
        assert_eq!(world.run(&["resume", &pos.to_string()]).code, 0);
        let refuses = |target: &str| {
            world.run(&["cd-hook", "pre", start.to_str().unwrap(), "", "--", target]).code == 3
        };
        let lines = drill(pos);
        let (wrong, lead) = match ["/etc", "..", "-"].into_iter().find(|target| refuses(target)) {
            Some(target) => (target, 0),
            // A rule that applies only deeper (`when_in`): the drill's first `cd` leads there.
            None => (
                "/etc",
                lines
                    .iter()
                    .position(|line| line.starts_with("cd "))
                    .expect("a strict level's drill moves")
                    + 1,
            ),
        };
        let (lead, rest) = lines.split_at(lead);
        let lead_in =
            |scene: &mut Scene| scene.extend((!lead.is_empty()).then(|| ok(lead.join("\n"))));
        let (go_wrong, to_start) = (format!("cd {wrong}"), format!("cd {}", quote(&start)));

        let mut scene = vec![at(resume(pos), &start)];
        lead_in(&mut scene);
        scene.push(blocked(&go_wrong, ""));
        for from in [&world.ws, &section] {
            scene.extend([ok(teleport(kind, from)), at(&to_start, &start)]);
        }
        lead_in(&mut scene);
        scene.push(ok(rest.join("\n")));
        if !lead.is_empty() {
            scene.push(ok(teleport(kind, &start)));
            lead_in(&mut scene);
        }
        scene.extend([ok(&go_wrong), at("shellgame reset", &start)]);
        lead_in(&mut scene);
        scene.push(blocked(&go_wrong, ""));
        scene
    })
}

/// The one level with a `cd` policy whose solution fits `is`.
fn strict_level(what: &str, is: impl Fn(&str) -> bool) -> Result<Pos, String> {
    let found: Vec<Pos> =
        Pos::all().filter(|&pos| !pos.level().cd.is_empty() && is(&solution(pos))).collect();
    match found[..] {
        [pos] => Ok(pos),
        _ => Err(format!("expected one strict level for {what}, found {found:?}")),
    }
}

/// What the solution of the absolute-path level starts with: `cd` to the level's own directory.
const ABSOLUTE_CD: &str = r#"cd "$PWD/"#;

/// `cd /` is absolute but not the lesson, and relative moves from `/` stay refused: only arriving
/// by an absolute `cd` from the start completes the level.
fn absolute_path(world: &World, kind: ShellKind, pos: Pos) -> Scene {
    let start = start_dir(pos, &world.ws);
    let solution = solution(pos);
    let target =
        solution.lines().next().unwrap().strip_prefix(ABSOLUTE_CD).unwrap().trim_end_matches('"');
    vec![
        at(resume(pos), &start),
        at("cd /", Path::new("/")),
        blocked("cd etc", ""),
        ok(teleport(kind, &start.join(target))),
        done("shellgame submit", false),
        ok(teleport(kind, &start)),
        done(solution, true),
    ]
}

/// `cd -` back to the start is allowed because it returns to the start, and it still counts; being
/// back by any other way does not.
fn cd_dash(world: &World, kind: ShellKind, pos: Pos) -> Scene {
    let start = start_dir(pos, &world.ws);
    let away = solution(pos).lines().next().unwrap().to_owned();
    vec![
        at(resume(pos), &start),
        ok(&away),
        ok(teleport(kind, &start)),
        done("shellgame submit", false),
        ok(&away),
        at("cd -", &start),
        done("shellgame submit", true),
    ]
}

/// A bare `cd` would leave the maze and is refused; `reset` inside the maze, which is rebuilt
/// under the player, leaves them in the new entrance, twice.
fn maze(world: &World, _: ShellKind, pos: Pos) -> Scene {
    let entrance = start_dir(pos, &world.ws);
    let inside = solution(pos).lines().next().unwrap().to_owned();
    vec![
        at(resume(pos), &entrance),
        blocked("cd", ""),
        ok(inside),
        at("shellgame reset", &entrance),
        at("shellgame reset", &entrance),
    ]
}

/// Every shortcut home is refused, and a symlink to `$HOME` is a single step but no walk, since the
/// shell's logical directory is not `$HOME`; a real walk completes the level.
fn home_walk(world: &World, _: ShellKind, pos: Pos) -> Scene {
    let (start, shortcut) = (start_dir(pos, &world.ws), world.ws.join("shortcut"));
    vec![
        at(resume(pos), &start),
        blocked("cd ~", ""),
        blocked("cd", ""),
        blocked("cd a/b", ""),
        done("shellgame submit", false),
        ok(r#"ln -s "$HOME" "$SHELLGAME_WORKSPACE/shortcut""#),
        at(walk(&world.ws), &world.ws),
        at("cd shortcut", &shortcut),
        done("shellgame submit", false),
        done(format!("{}\nshellgame submit", walk(&world.home)), true),
    ]
}

/// The first level that is judged on where the player stands: with no directory, it says so.
fn locating_level() -> Result<Pos, String> {
    let world = World::new();
    let lost = |pos: &Pos| {
        prepare(*pos, &world.ws).is_ok_and(|()| {
            let ctx = Ctx::new(&world.ws, *pos, None, &world.home);
            pos.level().judge(&ctx, None) == Err(CWD_MISSING.to_owned())
        })
    };
    Pos::all().find(lost).ok_or_else(|| "no level is judged on the player's directory".into())
}

/// The player deletes the directory they stand in (one outside the level, which a strict `cd` rule
/// would not let them enter): the game still runs there and tells them, `reset` brings them back to
/// the start, and the level can be played.
fn deleted_cwd(world: &World, kind: ShellKind, pos: Pos) -> Scene {
    let lost = world.tmp.join("lost");
    let delete = format!("mkdir {0}\n{1}\nrmdir {0}", quote(&lost), teleport(kind, &lost));
    vec![
        at(resume(pos), &start_dir(pos, &world.ws)),
        says(format!("{delete}\nshellgame submit\nshellgame reset"), CWD_MISSING),
        done(solution(pos), true),
    ]
}

/// The player takes the `x` away from the directory above them (`chmod a-x ..`): `$PWD` cannot be
/// resolved any more, but the directory is still there, and the game judges them where they are.
fn locked_parent(world: &World, kind: ShellKind, pos: Pos) -> Scene {
    let start = start_dir(pos, &world.ws);
    let child = start.join("child");
    vec![
        at(resume(pos), &start),
        ok(format!("mkdir {}", quote(&child))),
        at(teleport(kind, &child), &child),
        ok("chmod a-x .."),
        lacks("shellgame submit", CWD_MISSING),
        ok("chmod u+x .."),
        at("shellgame reset", &start),
    ]
}

/// A trial in `kind` of the level `find` picks; a level it cannot pick fails this trial only.
fn trial(
    name: String,
    kind: ShellKind,
    find: impl FnOnce() -> Result<Pos, String> + Send + 'static,
    run: impl FnOnce(Pos) -> Result<(), String> + Send + 'static,
) -> Trial {
    Trial::ignorable_test(name, move || {
        let pos = find().map_err(Failed::from)?;
        judged(kind, &solution(pos), || run(pos))
    })
}

/// What `run` says, or ignored with a note when `kind` cannot run `script` here.
fn judged(
    kind: ShellKind,
    script: &str,
    run: impl FnOnce() -> Result<(), String>,
) -> Result<Completion, Failed> {
    match skip(kind, script) {
        Some(why) => Ok(Completion::ignored_with(why)),
        None => run().map(|()| Completion::Completed).map_err(Failed::from),
    }
}

/// The first line of `tool --version`.
fn version(tool: &str) -> String {
    let output = Command::new(tool).arg("--version").output();
    let text =
        output.map(|o| String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").to_owned());
    text.unwrap_or_else(|_| format!("no {tool}"))
}

/// Why `kind` cannot run `script` here: fish is optional, and only fish 3.4 has `$(...)`.
fn skip(kind: ShellKind, script: &str) -> Option<&'static str> {
    static FISH: OnceLock<[bool; 2]> = OnceLock::new();
    let runs = |script| {
        let probe = Command::new("fish").args(["--no-config", "-c", script]).output();
        probe.is_ok_and(|output| output.status.success())
    };
    let [installed, substitutes] = *FISH.get_or_init(|| [runs("true"), runs("echo $(echo)")]);
    match kind {
        Fish if !installed => Some("fish is missing"),
        Fish if !substitutes && script.contains("$(") => {
            Some("the solution uses `$(...)`, which fish has only from 3.4")
        }
        _ => None,
    }
}

fn main() {
    let args = Arguments::from_args();
    let (bash, fish) = (version("bash"), version("fish"));
    eprintln!("e2e: {bash}; {fish}; Landlock {}", ["off", "on"][landlock() as usize]);

    // The long trials first: they run next to all the short ones instead of alone after them.
    let mut trials = vec![];
    // Each finds its level when it runs, so a level that changed fails its own trial only.
    let regressions: [(&str, Find, Build); 6] = [
        ("absolute-path", || strict_level("absolute", |s| s.contains(ABSOLUTE_CD)), absolute_path),
        ("cd-dash", || strict_level("cd -", |s| s.lines().any(|line| line == "cd -")), cd_dash),
        (
            "maze",
            || strict_level("maze", |s| s.lines().filter(|l| l.starts_with("cd ")).count() > 10),
            maze,
        ),
        // The one that starts its walk at the root.
        ("home-walk", || strict_level("home walk", |s| s.starts_with("cd /\n")), home_walk),
        ("deleted-cwd", locating_level, deleted_cwd),
        ("locked-parent", locating_level, locked_parent),
    ];
    let course: String = Pos::all().map(solution).collect();
    for kind in [Bash, Fish] {
        let course = course.clone();
        trials.push(Trial::ignorable_test(format!("playthrough::{kind}"), move || {
            judged(kind, &course, || playthrough(kind))
        }));
        for (name, find, scene) in regressions {
            trials.push(trial(format!("regression::{name}::{kind}"), kind, find, move |pos| {
                play(kind, pos, |world| scene(world, kind, pos))
            }));
        }
    }
    for pos in Pos::all() {
        for kind in [Bash, Fish] {
            let name = format!("solutions::{pos}::{kind}");
            trials.push(trial(
                name,
                kind,
                move || Ok(pos),
                move |pos| replay(kind, pos, &solution(pos)),
            ));
            if !pos.level().cd.is_empty() {
                let name = format!("strict_cd::{pos}::{kind}");
                trials.push(trial(name, kind, move || Ok(pos), move |pos| strict_cd(kind, pos)));
            }
        }
    }
    trials.push(Trial::test("harness::a-wrong-solution-fails", || {
        let pos = Pos::all().next().unwrap();
        let unknown = format!("no-such-command-in-the-replay\n{}", solution(pos));
        let wrong = [("does nothing", "true".to_owned()), ("runs an unknown command", unknown)];
        for (what, wrong) in wrong {
            if replay(Bash, pos, &wrong).is_ok() {
                return Err(format!("a solution that {what} passed").into());
            }
        }
        Ok(())
    }));
    libtest_mimic::run(&args, trials).exit();
}
