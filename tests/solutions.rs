//! Solution replay: every regular level is solved by its reference solution, in a real game shell.
//!
//! One trial per level and variant, named like `solutions::1.4::bash-022`: bash under `umask 022`,
//! bash under `umask 077` and fish (ignored with a note when fish is not installed). Each trial
//! runs in its own sandbox (see `e2e_support/`: its own `HOME`, state directory and a workspace
//! whose path contains a space) with the real binary and the real integration script, in **one
//! shell process**, through these steps:
//!
//! 1. `shellgame` starts the game, then `shellgame dev jump <id>` prepares the level and moves
//!    the shell to its start directory;
//! 2. an empty `shellgame submit` must not complete the level (`levels_complete` has no `<id>`);
//! 3. the solution script, which a student would type at the start directory, must complete it
//!    (`levels_complete` has `<id>`);
//! 4. the same again, from a `dev jump` over the workspace the first run left behind: it clears
//!    the level's progress, prepares the level once more, and the empty submit and the solution
//!    must behave as before.
//!
//! `levels_complete` is what is checked, not `current_level`: finishing a level moves the game on
//! through the next intro, and the last level only sets `completed_at`.
//!
//! So this one harness proves that levels are winnable, that preparing a level over what an
//! earlier run left behind works, that `cd` hooks accept their own drill, that `pwd` and fd
//! evidence flow end to end, and that answers can be found with the taught tools, in both shells
//! and whatever the player's `umask` is. Around that, the shell must exit normally, no protocol
//! line may reach the player, the game must not panic, the steps that only run the game print
//! nothing on stderr and every step leaves the shell in a directory that exists (a `reset` that
//! deletes the directory the player stands in must move them). A trial that runs longer than a
//! minute is killed.
//!
//! Run one level with `cargo test --test solutions -- solutions::1.4::`, one variant with
//! `-- bash-077`, another locale with `SHELLGAME_TEST_LOCALE=cs_CZ.UTF-8`.
//!
//! Two more kinds of trials. `solutions::playthrough::<shell>` plays the whole course in order in
//! one game, without `dev jump`, and checks how the game moves on by itself (see
//! `playthrough`). `solutions::harness::*` check that the harness itself fails a wrong solution
//! and kills a shell that hangs.

mod e2e_support;

use std::path::Path;
use std::time::{Duration, Instant};

use e2e_support::{
    Problem, Sandbox, Script, ShellRun, announce, catalog, clip, plain, shell_trial, squash,
};
use libtest_mimic::{Arguments, Failed, Trial};
use shellgame::engine::prelude::{LevelId, Solution};
use shellgame::messages;
use shellgame::shell::ShellKind;

/// One way of running a solution.
#[derive(Clone, Copy)]
struct Variant {
    shell: ShellKind,
    umask: &'static str,
}

impl Variant {
    fn name(self) -> String {
        format!("{}-{}", self.shell, self.umask)
    }
}

const VARIANTS: [Variant; 3] = [
    Variant {
        shell: ShellKind::Bash,
        umask: "022",
    },
    Variant {
        shell: ShellKind::Bash,
        umask: "077",
    },
    Variant {
        shell: ShellKind::Fish,
        umask: "022",
    },
];

fn main() {
    let args = Arguments::from_args();
    announce("solutions");

    // The long trials go first: they start at once and run next to all the short ones, instead of
    // running alone after them.
    let mut trials = Vec::new();
    for shell in [ShellKind::Bash, ShellKind::Fish] {
        trials.push(shell_trial(
            format!("solutions::playthrough::{shell}"),
            shell,
            move || playthrough(shell),
        ));
    }
    trials.push(Trial::test(
        "solutions::harness::hung-shell-is-killed",
        hung_shell_is_killed,
    ));
    trials.push(Trial::test(
        "solutions::harness::wrong-solution-fails",
        wrong_solution_fails,
    ));

    for (_, level) in catalog().iter().filter(|(_, level)| !level.is_intro()) {
        let id = level.id;
        let bash_only = level.solution.as_ref().is_some_and(Solution::is_bash_only);
        for variant in VARIANTS {
            if bash_only && variant.shell == ShellKind::Fish {
                continue;
            }
            trials.push(shell_trial(
                format!("solutions::{id}::{}", variant.name()),
                variant.shell,
                move || replay(id, variant, &Tweak::default()).map_err(Failed::from),
            ));
        }
    }
    libtest_mimic::run(&args, trials).exit();
}

/// What the tests of the harness change about a replay.
#[derive(Default)]
struct Tweak {
    /// Run this instead of the level's solution.
    solution: Option<&'static str>,
    /// Kill the shell after this long instead of after a minute.
    timeout: Option<Duration>,
}

/// The replay of one level in one variant. An `Err` carries the full report.
fn replay(id: LevelId, variant: Variant, tweak: &Tweak) -> Result<(), String> {
    let id_text = id.to_string();
    let mut sandbox = Sandbox::new();
    if let Some(timeout) = tweak.timeout {
        sandbox = sandbox.with_timeout(timeout);
    }
    let solution = match tweak.solution {
        Some(solution) => solution.to_owned(),
        None => sandbox.solution(&id_text)?,
    };
    let start = sandbox.place(&id_text).start;

    let jump = format!("shellgame dev jump {id_text}");
    let mut script = Script::new(variant.shell, variant.umask);
    script
        .step_with_state("init", "shellgame")
        .step_with_state("jump-1", &jump)
        .step_with_state("empty-submit-1", "shellgame submit")
        .step_with_state("solution-1", &solution)
        .step_with_state("jump-2", &jump)
        .step_with_state("empty-submit-2", "shellgame submit")
        .step_with_state("solution-2", &solution);
    let run = sandbox.run_shell(&script);

    judge(&id_text, start.as_deref(), &run).map_err(|problem| {
        format!(
            "level {id_text}, {}: {}\n{}",
            variant.name(),
            problem.headline(),
            run.report(Some(&solution), problem.step.as_deref()),
        )
    })
}

/// Judges the run: the shell was sound, and every step did what the replay expects.
fn judge(id: &str, start: Option<&Path>, run: &ShellRun) -> Result<(), Problem> {
    run.sanity().map_err(Problem::run)?;
    let step = |name: &str| run.step(name).map_err(|reason| Problem::at(name, reason));

    // Every step ends with a `shellgame` call, which leaves the shell in a directory that exists.
    // The steps that only run the game say nothing on stderr: the protocol lines are the
    // function's to consume, and nothing else is the game's to print there (a refused `cd` is the
    // only other thing, and a solution never does one).
    for record in &run.steps {
        if !record.name.starts_with("solution") && !record.stderr.is_empty() {
            return Err(Problem::at(
                &record.name,
                format!(
                    "the game wrote to stderr, which the player sees: {:?}",
                    clip(&plain(&record.stderr), 300)
                ),
            ));
        }
        if record.alive == Some(false) {
            return Err(Problem::at(&record.name, STALE_DIRECTORY));
        }
    }

    if step("init")?.state.is_none() {
        return Err(Problem::at(
            "init",
            "the first `shellgame` did not leave a readable state.json",
        ));
    }

    for (jump, empty, solution) in [
        ("jump-1", "empty-submit-1", "solution-1"),
        ("jump-2", "empty-submit-2", "solution-2"),
    ] {
        let record = step(jump)?;
        if record.current_level() != Some(id) {
            return Err(Problem::at(
                jump,
                format!(
                    "`dev jump {id}` left the game on {:?}",
                    record.current_level().unwrap_or("nothing")
                ),
            ));
        }
        if record.completed(id) != Some(false) {
            return Err(Problem::at(
                jump,
                "`dev jump` must clear the progress of the level it jumps to",
            ));
        }
        if let Some(start) = start
            && record.pwd.as_deref() != Some(start)
        {
            return Err(Problem::at(
                jump,
                format!(
                    "the shell was not moved to the start directory {}",
                    start.display()
                ),
            ));
        }

        if step(empty)?.completed(id) != Some(false) {
            return Err(Problem::at(
                empty,
                "an empty `shellgame submit` completed the level right after it started",
            ));
        }

        let record = step(solution)?;
        if record.completed(id) != Some(true) {
            let refused = record
                .stderr_words()
                .contains(&squash(messages::CD_RECOVERY_TIP));
            return Err(Problem::at(
                solution,
                if refused {
                    "the level's own `cd` hook refused a move of its solution (the recovery tip \
                     is on stderr), so the level was not completed"
                } else {
                    "the solution did not complete the level (`levels_complete` has no entry for it)"
                },
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// The whole course, in order
// ---------------------------------------------------------------------------------------------

/// A student plays the course from the first level to the last in one game shell and one game,
/// with no `dev jump`: each solution is typed wherever the game left the player after the level
/// before. The replay above reaches every level by jumping; this is how the game moves on by
/// itself. After each level the saved game must say that it is done and has moved on to the next
/// level (intros pass by themselves), the shell must have been moved to that level's start
/// directory, and after the last level the game is finished.
fn playthrough(shell: ShellKind) -> Result<(), Failed> {
    let sandbox = Sandbox::new();
    let ids: Vec<String> = catalog()
        .iter()
        .filter(|(_, level)| !level.is_intro())
        .map(|(_, level)| level.id.to_string())
        .collect();

    let mut script = Script::new(shell, "022");
    script.step_with_state("init", "shellgame");
    for id in &ids {
        script.step_with_state(&format!("solve-{id}"), &sandbox.solution(id)?);
    }
    let run = sandbox.run_shell(&script);

    run.sanity()
        .map_err(Problem::run)
        .and_then(|()| judge_playthrough(&sandbox, &ids, &run))
        .map_err(|problem| {
            Failed::from(format!(
                "the whole course ({shell}): {}\n{}",
                problem.headline(),
                run.report(None, problem.step.as_deref()),
            ))
        })
}

/// Judges the playthrough of the levels `ids`, in course order.
fn judge_playthrough(sandbox: &Sandbox, ids: &[String], run: &ShellRun) -> Result<(), Problem> {
    let init = run
        .step("init")
        .map_err(|reason| Problem::at("init", reason))?;
    if !init.stderr.is_empty() {
        return Err(Problem::at("init", "the game wrote to stderr"));
    }
    for (index, id) in ids.iter().enumerate() {
        let name = format!("solve-{id}");
        let record = run
            .step(&name)
            .map_err(|reason| Problem::at(&name, reason))?;
        if record.alive == Some(false) {
            return Err(Problem::at(&name, STALE_DIRECTORY));
        }
        if record.completed(id) != Some(true) {
            return Err(Problem::at(
                &name,
                format!(
                    "the solution did not complete level {id} (it was typed where the game left \
                     the player)"
                ),
            ));
        }
        let Some(next) = ids.get(index + 1) else {
            if record.game_finished() != Some(true) {
                return Err(Problem::at(
                    &name,
                    "the last level is done, but the game is not finished",
                ));
            }
            continue;
        };
        if record.current_level() != Some(next.as_str()) {
            return Err(Problem::at(
                &name,
                format!(
                    "the game should have moved on to level {next}, it is on {:?}",
                    record.current_level().unwrap_or("nothing")
                ),
            ));
        }
        if let Some(start) = sandbox.place(next).start
            && record.pwd.as_deref() != Some(start.as_path())
        {
            return Err(Problem::at(
                &name,
                format!(
                    "the shell should have been moved to the start of level {next}, {}",
                    start.display()
                ),
            ));
        }
    }
    Ok(())
}

/// What it means when a step's `alive` check fails.
const STALE_DIRECTORY: &str =
    "the shell was left in a directory that was deleted (and perhaps built again at the same path)";

// ---------------------------------------------------------------------------------------------
// The harness checks itself
// ---------------------------------------------------------------------------------------------

/// A solution that does not solve the level makes the trial fail, at the step of the solution.
fn wrong_solution_fails() -> Result<(), Failed> {
    let report = match replay(
        LevelId::new(1, 4),
        VARIANTS[0],
        &Tweak {
            solution: Some("true"),
            ..Tweak::default()
        },
    ) {
        Err(report) => report,
        Ok(()) => return Err("a solution that does nothing completed level 1.4".into()),
    };
    if report.contains("step `solution-1`") && report.contains("did not complete the level") {
        Ok(())
    } else {
        Err(Failed::from(format!(
            "the failure does not say what went wrong:\n{report}"
        )))
    }
}

/// A shell that does not finish is killed, and the trial says so.
fn hung_shell_is_killed() -> Result<(), Failed> {
    let started = Instant::now();
    let report = match replay(
        LevelId::new(1, 4),
        VARIANTS[0],
        &Tweak {
            solution: Some("sleep 4"),
            timeout: Some(Duration::from_secs(1)),
        },
    ) {
        Err(report) => report,
        Ok(()) => return Err("a shell that sleeps past the time limit was not stopped".into()),
    };
    if started.elapsed() >= Duration::from_secs(4) {
        return Err("the shell was not killed when the time limit passed".into());
    }
    if report.contains("did not finish within 1 s") {
        Ok(())
    } else {
        Err(Failed::from(format!(
            "the failure does not mention the time limit:\n{report}"
        )))
    }
}
