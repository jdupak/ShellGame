//! Strict `cd` levels never lock a player in, through the real game shell.
//!
//! Several navigation levels teach one exact movement command and refuse everything else (1.4,
//! 1.6, 1.7, 1.8, 1.9, 1.12, 2.2 and 2.3). Before the guarantees below existed, a player who took
//! one wrong step could end up in a directory where every permitted move led further away, with
//! no way back but deleting the workspace. For each of these levels, starting where `dev jump`
//! leaves the player:
//!
//! * **blocked**: a wrong move (`cd /etc`, a relative path, `cd -`) is refused with the recovery
//!   tip (`shellgame reset`), the `cd` fails and the player stays where they were;
//! * **back to the start**: moving to the level's start directory is always allowed, from the
//!   workspace root and from the section root;
//! * **free after the drill**: once the drill is done (the level's evidence exists), the move that
//!   was refused before is allowed;
//! * **reset recovers**: the tip's promise holds. `shellgame reset`, typed wherever the player
//!   wandered off to, brings the shell back to the start directory and makes the level strict
//!   again (the evidence of the drill is gone).
//!
//! Each of the four exists once per shell (bash, fish), as `strict_cd::<id>::<what>::<shell>`,
//! and every level also gets a `hook-contract` trial that talks to `cd-hook` without a shell:
//! the new hook form exits with status 3 for a refused move and writes the tip to stderr.
//!
//! Then the deliberate fixes of the rewrite get their own trials:
//!
//! * 1.8: `cd /` does not count as the absolute `cd`; the evidence comes only from arriving in
//!   `absolute-target`;
//! * 2.2: `cd -` back to the start directory records the evidence (the move is allowed because it
//!   returns to the start, and that must not cost the evidence);
//! * 1.7: a bare `cd` in the maze is refused with the maze message;
//! * 1.9: `cd ~`, a bare `cd` and `cd a/b` are refused, and a symlink that points home is no walk;
//! * 1.12: the exact `cd` does not count before `pwd` was used;
//! * 1.7: `shellgame reset` inside the maze, which every preparation deletes and builds again,
//!   leaves the player in a directory that exists.
//!
//! Run one level with `cargo test --test strict_cd -- strict_cd::1.8::`.

mod e2e_support;

use std::path::{Path, PathBuf};

use e2e_support::{
    Place, Problem, Sandbox, Script, ShellRun, StepRecord, announce, quote, shell_trial, squash,
    teleport,
};
use libtest_mimic::{Arguments, Failed, Trial};
use shellgame::messages;
use shellgame::shell::ShellKind;

const SHELLS: [ShellKind; 2] = [ShellKind::Bash, ShellKind::Fish];

/// What the drill of a level is.
enum Drill {
    /// These shell commands, run where the strict rule applies.
    Commands(&'static str),
    /// The level's reference solution without its final `shellgame submit`, run at the start.
    Solution,
}

/// One strict level.
struct Case {
    id: &'static str,
    /// Shell commands that take the player from the start directory to where the rule applies.
    setup: &'static str,
    /// Where the rule applies, relative to the section root; `None` is the start directory.
    strict: Option<&'static str>,
    /// An evidence file in the workspace that the rule needs before it applies (`pwd` leaves it).
    /// `hook-contract`, which has no shell, creates it directly.
    marker: Option<&'static str>,
    /// The argument of a `cd` that the level refuses there. It leads somewhere that exists, so a
    /// move that is allowed succeeds.
    wrong: &'static str,
    drill: Drill,
    /// Check the allowed move where the drill ended instead of going back to the strict place
    /// first: `cd -` goes to the previous directory, and going "back" would change that.
    stay_after_drill: bool,
}

const CASES: [Case; 8] = [
    Case {
        id: "1.4",
        setup: "",
        strict: None,
        marker: None,
        wrong: "/etc",
        drill: Drill::Commands("cd .."),
        stay_after_drill: false,
    },
    Case {
        id: "1.6",
        setup: "",
        strict: None,
        marker: None,
        wrong: "/etc",
        drill: Drill::Commands("cd ../../.."),
        stay_after_drill: false,
    },
    Case {
        id: "1.7",
        setup: "",
        strict: None,
        marker: None,
        wrong: "/etc",
        drill: Drill::Solution,
        stay_after_drill: false,
    },
    Case {
        id: "1.8",
        setup: "",
        strict: None,
        marker: None,
        wrong: "alpha",
        drill: Drill::Commands("cd \"$PWD/absolute-target\""),
        stay_after_drill: false,
    },
    Case {
        id: "1.9",
        setup: "",
        strict: None,
        marker: None,
        wrong: "/etc",
        drill: Drill::Solution,
        stay_after_drill: false,
    },
    Case {
        id: "1.12",
        setup: "pwd > /dev/null\ncd gamma/deep/a/b/c",
        strict: Some("gamma/deep/a/b/c"),
        marker: Some(".pwd_used"),
        wrong: "/etc",
        drill: Drill::Commands("cd ../../../.."),
        stay_after_drill: false,
    },
    Case {
        id: "2.2",
        setup: "",
        strict: None,
        marker: None,
        wrong: "-",
        drill: Drill::Commands("cd ../location-B\ncd -"),
        stay_after_drill: true,
    },
    Case {
        id: "2.3",
        setup: "",
        strict: None,
        marker: None,
        wrong: "/etc",
        drill: Drill::Commands("cd ../../other/target"),
        stay_after_drill: false,
    },
];

impl Case {
    /// Where the rule applies, in this sandbox.
    fn strict_dir(&self, place: &Place) -> PathBuf {
        match self.strict {
            Some(relative) => place.in_section(relative),
            None => start_of(place),
        }
    }

    /// The commands of the drill.
    fn drill_commands(&self, sandbox: &Sandbox) -> Result<String, String> {
        match &self.drill {
            Drill::Commands(commands) => Ok((*commands).to_owned()),
            Drill::Solution => Ok(sandbox
                .solution(self.id)?
                .lines()
                .filter(|line| !line.trim_start().starts_with("shellgame submit"))
                .collect::<Vec<_>>()
                .join("\n")),
        }
    }
}

fn start_of(place: &Place) -> PathBuf {
    place
        .start
        .clone()
        .expect("a strict level has a start directory")
}

fn main() {
    let args = Arguments::from_args();
    announce("strict_cd");

    let mut trials = Vec::new();
    for case in &CASES {
        for shell in SHELLS {
            let trial = |what: &str, run: fn(&'static Case, ShellKind) -> Result<(), Failed>| {
                shell_trial(trial_name(case.id, what, shell), shell, move || {
                    run(case, shell)
                })
            };
            trials.push(trial("blocked-move", blocked_move));
            trials.push(trial("back-to-start", back_to_start));
            trials.push(trial("free-after-drill", free_after_drill));
            trials.push(trial("reset-recovers", reset_recovers));
        }
        trials.push(Trial::test(
            format!("strict_cd::{}::hook-contract", case.id),
            move || hook_contract(case),
        ));
    }

    // The deliberate fixes, each in both shells.
    let fixes: [(&str, &str, Fix); 7] = [
        ("1.8", "cd-root-is-not-evidence", cd_root_is_not_evidence),
        (
            "2.2",
            "cd-dash-to-start-is-evidence",
            cd_dash_to_start_is_evidence,
        ),
        ("1.7", "bare-cd-is-blocked", bare_cd_in_the_maze),
        ("1.9", "shortcuts-are-blocked", home_shortcuts_are_blocked),
        ("1.9", "symlink-is-no-walk", symlink_is_no_walk),
        ("1.12", "pwd-comes-first", pwd_comes_first),
        ("1.7", "reset-inside-the-maze", reset_inside_the_maze),
    ];
    for (id, what, run) in fixes {
        for shell in SHELLS {
            trials.push(shell_trial(trial_name(id, what, shell), shell, move || {
                run(shell)
            }));
        }
    }
    libtest_mimic::run(&args, trials).exit();
}

/// A trial body about one of the deliberate fixes.
type Fix = fn(ShellKind) -> Result<(), Failed>;

fn trial_name(id: &str, what: &str, shell: ShellKind) -> String {
    format!("strict_cd::{id}::{what}::{shell}")
}

// ---------------------------------------------------------------------------------------------
// Running a scenario and judging it
// ---------------------------------------------------------------------------------------------

/// Plays one scenario in a fresh sandbox: `build` adds the steps, the shell runs them all in one
/// process, `judge` checks what each step did. A failure carries a full report.
fn scenario(
    title: &str,
    level: &str,
    shell: ShellKind,
    build: impl FnOnce(&mut Script, &Sandbox, &Place) -> Result<(), String>,
    judge: impl FnOnce(&ShellRun, &Place) -> Result<(), Problem>,
) -> Result<(), Failed> {
    let sandbox = Sandbox::new();
    let place = sandbox.place(level);
    let mut script = Script::new(shell, "022");
    build(&mut script, &sandbox, &place).map_err(Failed::from)?;
    let run = sandbox.run_shell(&script);
    run.sanity()
        .map_err(Problem::run)
        .and_then(|()| judge(&run, &place))
        .map_err(|problem| {
            Failed::from(format!(
                "{title} ({shell}): {}\nscript:\n{}{}",
                problem.headline(),
                script.describe(),
                run.report(None, problem.step.as_deref()),
            ))
        })
}

fn step<'a>(run: &'a ShellRun, name: &str) -> Result<&'a StepRecord, Problem> {
    run.step(name).map_err(|reason| Problem::at(name, reason))
}

/// The recovery tip every refused `cd` shows, as words (panels fold lines and draw borders).
fn tip() -> String {
    assert!(
        messages::CD_RECOVERY_TIP.contains("shellgame reset"),
        "the recovery tip must name `shellgame reset`"
    );
    squash(messages::CD_RECOVERY_TIP)
}

fn pwd_text(step: &StepRecord) -> String {
    step.pwd
        .as_deref()
        .map_or_else(|| "nowhere".to_owned(), |pwd| pwd.display().to_string())
}

/// The `cd` of this step was refused: the recovery tip is on stderr, the `cd` failed, and the
/// player is still in `stayed`.
fn expect_blocked(run: &ShellRun, name: &str, stayed: &Path) -> Result<(), Problem> {
    let record = step(run, name)?;
    if !record.stderr_words().contains(&tip()) {
        return Err(Problem::at(
            name,
            format!(
                "the move was not refused: no recovery tip on stderr (status {:?}, now in {})",
                record.status,
                pwd_text(record)
            ),
        ));
    }
    if record.status != Some(1) {
        return Err(Problem::at(
            name,
            format!(
                "a refused `cd` must fail with status 1, got {:?}",
                record.status
            ),
        ));
    }
    if record.pwd.as_deref() != Some(stayed) {
        return Err(Problem::at(
            name,
            format!(
                "a refused `cd` must leave the player in {}, but they are in {}",
                stayed.display(),
                pwd_text(record)
            ),
        ));
    }
    Ok(())
}

/// The commands of this step were not refused: no recovery tip, and the last one succeeded.
fn expect_allowed(run: &ShellRun, name: &str) -> Result<(), Problem> {
    let record = step(run, name)?;
    if record.stderr_words().contains(&tip()) {
        return Err(Problem::at(
            name,
            "the move was refused, but it must be allowed",
        ));
    }
    if record.status != Some(0) {
        return Err(Problem::at(
            name,
            format!("the move failed with status {:?}", record.status),
        ));
    }
    Ok(())
}

/// This step ended with the player in `dir`.
fn expect_in(run: &ShellRun, name: &str, dir: &Path) -> Result<(), Problem> {
    let record = step(run, name)?;
    if record.pwd.as_deref() == Some(dir) {
        Ok(())
    } else {
        Err(Problem::at(
            name,
            format!(
                "the player should be in {}, but is in {}",
                dir.display(),
                pwd_text(record)
            ),
        ))
    }
}

/// The level is (not) complete after this step.
fn expect_completed(run: &ShellRun, name: &str, id: &str, expected: bool) -> Result<(), Problem> {
    match step(run, name)?.completed(id) {
        Some(done) if done == expected => Ok(()),
        Some(_) => Err(Problem::at(
            name,
            if expected {
                format!("level {id} should be complete now")
            } else {
                format!("level {id} must not be complete now")
            },
        )),
        None => Err(Problem::at(name, "no snapshot of the saved game")),
    }
}

/// The steps every scenario starts with: the game, the jump to the level and the setup.
fn begin(script: &mut Script, case: &Case) {
    script.step("init", "shellgame");
    script.step_with_state("jump", &format!("shellgame dev jump {}", case.id));
    if !case.setup.is_empty() {
        script.step("setup", case.setup);
    }
}

/// The jump brought the player to the start directory, and the setup to where the rule applies.
fn expect_begun(run: &ShellRun, case: &Case, place: &Place) -> Result<(), Problem> {
    expect_in(run, "jump", &start_of(place))?;
    if !case.setup.is_empty() {
        expect_in(run, "setup", &case.strict_dir(place))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// The table: blocked, back to the start, free after the drill
// ---------------------------------------------------------------------------------------------

/// A wrong move is refused, with the recovery tip, and changes nothing.
fn blocked_move(case: &'static Case, shell: ShellKind) -> Result<(), Failed> {
    scenario(
        &format!("level {}: a wrong move", case.id),
        case.id,
        shell,
        |script, _, _| {
            begin(script, case);
            script.step("wrong", &format!("cd {}", case.wrong));
            Ok(())
        },
        |run, place| {
            expect_begun(run, case, place)?;
            expect_blocked(run, "wrong", &case.strict_dir(place))
        },
    )
}

/// The places the recovery move starts from: the workspace root, the section root, the player's
/// home (outside the workspace altogether) and, for a rule that applies deeper, there too.
fn places(case: &Case, place: &Place) -> Vec<(&'static str, PathBuf)> {
    let mut places = vec![
        ("workspace", place.workspace.clone()),
        ("section", place.section_root.clone()),
        ("home", place.home.clone()),
    ];
    if case.strict.is_some() {
        places.push(("strict", case.strict_dir(place)));
    }
    places
}

/// A place other than the start directory to go to and come back from with `cd -`. (Moving to
/// the directory one is in changes nothing in fish, so `cd -` would go back further than that.)
fn away_from_start(place: &Place) -> PathBuf {
    if place.start.as_deref() == Some(place.workspace.as_path()) {
        place.section_root.clone()
    } else {
        place.workspace.clone()
    }
}

/// Moving to the start directory is always allowed, by its path and with `cd -`.
fn back_to_start(case: &'static Case, shell: ShellKind) -> Result<(), Failed> {
    scenario(
        &format!("level {}: back to the start", case.id),
        case.id,
        shell,
        |script, _, place| {
            begin(script, case);
            for (name, dir) in places(case, place) {
                script.step(&format!("to-{name}"), &teleport(shell, &dir));
                script.step(
                    &format!("back-from-{name}"),
                    &format!("cd {}", quote(&start_of(place))),
                );
            }
            // `cd -` goes back to the start as well, when that is where the player came from.
            script.step("away", &teleport(shell, &away_from_start(place)));
            script.step("back-with-dash", "cd -");
            Ok(())
        },
        |run, place| {
            expect_begun(run, case, place)?;
            for (name, dir) in places(case, place) {
                expect_in(run, &format!("to-{name}"), &dir)?;
                let back = format!("back-from-{name}");
                expect_allowed(run, &back)?;
                expect_in(run, &back, &start_of(place))?;
            }
            expect_in(run, "away", &away_from_start(place))?;
            expect_allowed(run, "back-with-dash")?;
            expect_in(run, "back-with-dash", &start_of(place))
        },
    )
}

/// After the drill, the move that was refused before is allowed.
fn free_after_drill(case: &'static Case, shell: ShellKind) -> Result<(), Failed> {
    scenario(
        &format!("level {}: free after the drill", case.id),
        case.id,
        shell,
        |script, sandbox, place| {
            begin(script, case);
            script.step("wrong-before", &format!("cd {}", case.wrong));
            script.step("drill", &case.drill_commands(sandbox)?);
            if !case.stay_after_drill {
                script.step("back-to-strict", &teleport(shell, &case.strict_dir(place)));
            }
            script.step("wrong-after", &format!("cd {}", case.wrong));
            Ok(())
        },
        |run, place| {
            expect_begun(run, case, place)?;
            // Without this the case could be stale: the move must be refused before the drill.
            expect_blocked(run, "wrong-before", &case.strict_dir(place))?;
            expect_allowed(run, "drill")?;
            if !case.stay_after_drill {
                expect_in(run, "back-to-strict", &case.strict_dir(place))?;
            }
            expect_allowed(run, "wrong-after")
        },
    )
}

/// `shellgame reset` brings the player back to the start of the level from wherever they are, in
/// a directory that exists, and the level is strict again: the evidence of the drill is gone.
/// It is tried where the drill ended and after the player wandered off to their home.
fn reset_recovers(case: &'static Case, shell: ShellKind) -> Result<(), Failed> {
    scenario(
        &format!("level {}: shellgame reset", case.id),
        case.id,
        shell,
        |script, sandbox, place| {
            begin(script, case);
            script.step("drill", &case.drill_commands(sandbox)?);
            script.step("reset-here", "shellgame reset");
            script.step("lost", &teleport(shell, &place.home));
            script.step("reset-from-home", "shellgame reset");
            if !case.setup.is_empty() {
                script.step("setup-again", case.setup);
            }
            script.step("wrong-again", &format!("cd {}", case.wrong));
            Ok(())
        },
        |run, place| {
            expect_begun(run, case, place)?;
            expect_allowed(run, "drill")?;
            expect_in(run, "reset-here", &start_of(place))?;
            expect_alive(run, "reset-here")?;
            expect_in(run, "lost", &place.home)?;
            expect_in(run, "reset-from-home", &start_of(place))?;
            expect_alive(run, "reset-from-home")?;
            if !case.setup.is_empty() {
                expect_in(run, "setup-again", &case.strict_dir(place))?;
            }
            expect_blocked(run, "wrong-again", &case.strict_dir(place))
        },
    )
}

/// 1.7: the maze is deleted and built again by every preparation, so a player who stands inside
/// it when they reset is standing in a directory that is gone. They must end up in the new maze's
/// entrance, in a directory that exists. The second reset is the hard one: the player already
/// stands at the entrance, whose path does not change although the directory is replaced.
fn reset_inside_the_maze(shell: ShellKind) -> Result<(), Failed> {
    let case = strict_case("1.7");
    scenario(
        "level 1.7: reset inside the maze",
        case.id,
        shell,
        |script, _, _| {
            begin(script, case);
            script.step("into-the-maze", "cd hall");
            script.step("reset-inside", "shellgame reset");
            script.step("reset-at-entrance", "shellgame reset");
            Ok(())
        },
        |run, place| {
            let entrance = start_of(place);
            expect_begun(run, case, place)?;
            expect_allowed(run, "into-the-maze")?;
            expect_in(run, "reset-inside", &entrance)?;
            expect_alive(run, "reset-inside")?;
            expect_in(run, "reset-at-entrance", &entrance)?;
            expect_alive(run, "reset-at-entrance")
        },
    )
}

/// The shell is in a directory that exists and is the one its path names: nothing deleted it
/// underneath the shell (and built it again at the same path) without moving the shell.
fn expect_alive(run: &ShellRun, name: &str) -> Result<(), Problem> {
    let record = step(run, name)?;
    if record.alive == Some(true) {
        Ok(())
    } else {
        Err(Problem::at(
            name,
            format!(
                "the shell is in a directory that was deleted and built again; its path is {}",
                pwd_text(record)
            ),
        ))
    }
}

/// The new hook form, without a shell: `cd-hook pre <pwd> <prev> -- <cd args>` exits with status 3
/// for a refused move, with the tip on stderr, and with 0 for the way back to the start.
fn hook_contract(case: &'static Case) -> Result<(), Failed> {
    let sandbox = Sandbox::new();
    let place = sandbox.place(case.id);
    let strict = case.strict_dir(&place);
    let start = start_of(&place);

    let started = sandbox.game(&[]);
    let jumped = sandbox.game(&["dev", "jump", case.id]);
    for (what, run) in [("starting the game", &started), ("dev jump", &jumped)] {
        if run.status != Some(0) {
            return Err(Failed::from(format!(
                "level {}: {what} exited with {:?}\n{}",
                case.id, run.status, run.stderr
            )));
        }
    }
    if let Some(marker) = case.marker {
        std::fs::write(place.workspace.join(marker), "").expect("create the evidence file");
    }

    let hook = |pwd: &Path, target: &str| {
        let pwd = pwd.to_str().expect("sandbox paths are UTF-8");
        sandbox.game(&["cd-hook", "pre", pwd, "", "--", target])
    };

    let refused = hook(&strict, case.wrong);
    let explanation = squash(&refused.stderr);
    if refused.status != Some(3) {
        return Err(Failed::from(format!(
            "level {}: `cd {}` from {} exited with {:?}, a refused move exits with 3\nstderr: {}",
            case.id,
            case.wrong,
            strict.display(),
            refused.status,
            refused.stderr
        )));
    }
    if !explanation.contains(&tip()) {
        return Err(Failed::from(format!(
            "level {}: the refused move did not print the recovery tip\nstderr: {}",
            case.id, refused.stderr
        )));
    }

    let start_text = start.to_str().expect("sandbox paths are UTF-8");
    for (name, from) in [
        ("workspace", &place.workspace),
        ("section", &place.section_root),
    ] {
        let back = hook(from, start_text);
        if back.status != Some(0) {
            return Err(Failed::from(format!(
                "level {}: `cd {}` from the {name} root exited with {:?}, the way back to the start is \
                 always allowed\nstderr: {}",
                case.id,
                start.display(),
                back.status,
                back.stderr
            )));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// The deliberate fixes
// ---------------------------------------------------------------------------------------------

/// The case of a level.
fn strict_case(id: &str) -> &'static Case {
    CASES
        .iter()
        .find(|case| case.id == id)
        .unwrap_or_else(|| panic!("no case for level {id}"))
}

/// 1.8: `cd /` is absolute but not the lesson. It earns no evidence, and relative moves from `/`
/// stay refused; only arriving in `absolute-target` by an absolute path does.
fn cd_root_is_not_evidence(shell: ShellKind) -> Result<(), Failed> {
    let case = strict_case("1.8");
    scenario(
        "level 1.8: `cd /` is not the absolute cd",
        case.id,
        shell,
        |script, _, place| {
            begin(script, case);
            script.step("cd-root", "cd /");
            script.step("relative-from-root", "cd etc");
            script.step(
                "to-target",
                &teleport(shell, &place.in_section("absolute-target")),
            );
            script.step_with_state("submit-after-root", "shellgame submit");
            script.step("to-start", &teleport(shell, &start_of(place)));
            script.step("absolute-cd", "cd \"$PWD/absolute-target\"");
            script.step_with_state("submit", "shellgame submit");
            Ok(())
        },
        |run, place| {
            let target = place.in_section("absolute-target");
            expect_begun(run, case, place)?;
            expect_allowed(run, "cd-root")?;
            expect_in(run, "cd-root", Path::new("/"))?;
            expect_blocked(run, "relative-from-root", Path::new("/"))?;
            expect_in(run, "to-target", &target)?;
            // In the right directory, but only `cd /` was used: not enough.
            expect_completed(run, "submit-after-root", case.id, false)?;
            expect_allowed(run, "absolute-cd")?;
            expect_in(run, "absolute-cd", &target)?;
            expect_completed(run, "submit", case.id, true)
        },
    )
}

/// 2.2: `cd -` back into the start directory is allowed because it returns to the start, and the
/// evidence is recorded all the same. Going back any other way is no evidence.
fn cd_dash_to_start_is_evidence(shell: ShellKind) -> Result<(), Failed> {
    let case = strict_case("2.2");
    scenario(
        "level 2.2: `cd -` back to the start",
        case.id,
        shell,
        |script, _, place| {
            begin(script, case);
            script.step("to-b", "cd ../location-B");
            script.step("teleport-back", &teleport(shell, &start_of(place)));
            script.step_with_state("submit-without-dash", "shellgame submit");
            script.step("to-b-again", "cd ../location-B");
            script.step("dash", "cd -");
            script.step_with_state("submit", "shellgame submit");
            Ok(())
        },
        |run, place| {
            let start = start_of(place);
            expect_begun(run, case, place)?;
            expect_allowed(run, "to-b")?;
            expect_in(run, "to-b", &place.in_section("location-B"))?;
            expect_in(run, "teleport-back", &start)?;
            // Standing in the right place is not enough: the drill is `cd -` from location-B.
            expect_completed(run, "submit-without-dash", case.id, false)?;
            expect_allowed(run, "to-b-again")?;
            expect_allowed(run, "dash")?;
            expect_in(run, "dash", &start)?;
            expect_completed(run, "submit", case.id, true)
        },
    )
}

/// 1.7: a bare `cd` would leave the maze, and says so in the maze's own words.
fn bare_cd_in_the_maze(shell: ShellKind) -> Result<(), Failed> {
    let case = strict_case("1.7");
    scenario(
        "level 1.7: a bare `cd` in the maze",
        case.id,
        shell,
        |script, _, _| {
            begin(script, case);
            script.step("bare-cd", "cd");
            Ok(())
        },
        |run, place| {
            expect_begun(run, case, place)?;
            expect_blocked(run, "bare-cd", &start_of(place))?;
            // The message of a bare `cd` points back into `maze/`.
            if !step(run, "bare-cd")?.stderr_words().contains("maze/") {
                return Err(Problem::at(
                    "bare-cd",
                    "the bare `cd` did not get the maze message (it names `maze/`)",
                ));
            }
            Ok(())
        },
    )
}

/// 1.9: every way of jumping home is refused: `cd ~`, a bare `cd`, a path of several segments.
fn home_shortcuts_are_blocked(shell: ShellKind) -> Result<(), Failed> {
    let case = strict_case("1.9");
    scenario(
        "level 1.9: shortcuts home",
        case.id,
        shell,
        |script, _, _| {
            begin(script, case);
            script.step("make-dirs", "mkdir -p a/b");
            script.step("tilde", "cd ~");
            script.step("bare", "cd");
            script.step("two-segments", "cd a/b");
            script.step_with_state("submit", "shellgame submit");
            Ok(())
        },
        |run, place| {
            expect_begun(run, case, place)?;
            for name in ["tilde", "bare", "two-segments"] {
                expect_blocked(run, name, &start_of(place))?;
            }
            expect_completed(run, "submit", case.id, false)
        },
    )
}

/// 1.9: a symlink that points home is a single step, but the shell's logical directory is not
/// `$HOME`, so it is no walk. A real walk afterwards still completes the level, which shows
/// that it was the symlink and nothing else that failed.
fn symlink_is_no_walk(shell: ShellKind) -> Result<(), Failed> {
    let case = strict_case("1.9");
    scenario(
        "level 1.9: a symlink to home",
        case.id,
        shell,
        |script, sandbox, place| {
            begin(script, case);
            script.step("link", "ln -s \"$HOME\" \"$SHELLGAME_WORKSPACE/shortcut\"");
            // Walk down to the workspace one directory at a time, then take the shortcut.
            let mut walk = String::from("cd /\n");
            for part in place.workspace.components().skip(1) {
                walk.push_str(&format!("cd {}\n", quote(Path::new(part.as_os_str()))));
            }
            script.step("walk-to-workspace", &walk);
            script.step("shortcut", "cd shortcut");
            script.step_with_state("submit-shortcut", "shellgame submit");
            script.step_with_state("real-walk", &sandbox.solution(case.id)?);
            Ok(())
        },
        |run, place| {
            expect_begun(run, case, place)?;
            expect_allowed(run, "link")?;
            expect_allowed(run, "walk-to-workspace")?;
            expect_in(run, "walk-to-workspace", &place.workspace)?;
            expect_allowed(run, "shortcut")?;
            expect_in(run, "shortcut", &place.workspace.join("shortcut"))?;
            expect_completed(run, "submit-shortcut", case.id, false)?;
            expect_completed(run, "real-walk", case.id, true)
        },
    )
}

/// 1.12: from the deepest directory the exact `cd` counts only after `pwd` was used.
fn pwd_comes_first(shell: ShellKind) -> Result<(), Failed> {
    let case = strict_case("1.12");
    scenario(
        "level 1.12: pwd first",
        case.id,
        shell,
        |script, _, _| {
            script.step("init", "shellgame");
            script.step_with_state("jump", &format!("shellgame dev jump {}", case.id));
            script.step("descend", "cd gamma/deep/a/b/c");
            script.step("before-pwd", "cd ../../../..");
            script.step("use-pwd", "pwd > /dev/null");
            script.step("after-pwd", "cd ../../../..");
            Ok(())
        },
        |run, place| {
            expect_in(run, "jump", &start_of(place))?;
            let deep = case.strict_dir(place);
            expect_allowed(run, "descend")?;
            expect_in(run, "descend", &deep)?;
            expect_blocked(run, "before-pwd", &deep)?;
            expect_allowed(run, "use-pwd")?;
            expect_allowed(run, "after-pwd")?;
            expect_in(run, "after-pwd", &place.in_section("gamma"))
        },
    )
}
