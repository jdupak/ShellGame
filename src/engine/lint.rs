//! Catalog validation: the structural and content lints behind [`Catalog::validate`].
//!
//! These checks catch the mistakes an author can make that no type can: a path that restates
//! its section root, a level nobody can finish, an error message that gives the answer
//! away. They run at startup in debug builds and in the catalog test, so a malformed level
//! fails immediately instead of in front of a player.

use std::collections::HashSet;
use std::path::Path;

use crate::engine::answer::Answer;
use crate::engine::catalog::Catalog;
use crate::engine::level::{Level, Start};
use crate::engine::section::Section;
use crate::engine::{Check, Ctx};
use crate::markers::Marker;
use crate::paths::RelPath;

/// Answers shorter than this are too likely to occur in any sentence by chance, so the
/// leak lint ignores them (`c`, `3`, `log`).
const MIN_LEAK_LEN: usize = 4;

/// Run every lint over the catalog. Lines start with the level id or `section N`.
pub(crate) fn lint(catalog: &Catalog) -> Vec<String> {
    let mut errors = Vec::new();
    lint_structure(catalog, &mut errors);
    for section in catalog.sections() {
        lint_section(section, &mut errors);
    }
    for (section, level) in catalog.iter() {
        lint_level(section, level, &mut errors);
    }
    // The same fact can be reached through several paths of the same answer.
    let mut seen = HashSet::new();
    errors.retain(|error| seen.insert(error.clone()));
    errors
}

fn lint_structure(catalog: &Catalog, errors: &mut Vec<String>) {
    for pair in catalog.sections().windows(2) {
        if pair[0].number == pair[1].number {
            errors.push(format!(
                "section {}: declared more than once",
                pair[0].number
            ));
        }
    }
    let ids: Vec<_> = catalog.ids().collect();
    for pair in ids.windows(2) {
        if pair[0] == pair[1] {
            errors.push(format!("{}: duplicate level id", pair[0]));
        }
    }
    for section in catalog.sections() {
        for level in &section.levels {
            if level.id.section != section.number {
                errors.push(format!(
                    "{}: declared in section {}, whose number differs from the id",
                    level.id, section.number
                ));
            }
        }
    }
}

/// Check one declared path: it must be a plain relative path and must not restate the
/// section root (`level-2/x` inside section 2: paths are already relative to it).
fn check_path(section: &Section, owner: &str, what: &str, path: &str, errors: &mut Vec<String>) {
    let parsed = match RelPath::parse(path) {
        Ok(parsed) => parsed,
        Err(error) => {
            errors.push(format!("{owner}: {what} {path:?}: {error}"));
            return;
        }
    };
    if let Ok(root) = RelPath::parse(&section.root)
        && !root.is_root()
        && parsed.starts_with(&root)
    {
        errors.push(format!(
            "{owner}: {what} {path:?} restates the section root {:?}; paths are already relative to it",
            section.root
        ));
    }
}

fn lint_section(section: &Section, errors: &mut Vec<String>) {
    let owner = format!("section {}", section.number);
    if let Err(error) = RelPath::parse(&section.root) {
        errors.push(format!("{owner}: root {:?}: {error}", section.root));
    }
    for problem in section.fixture.problems() {
        errors.push(format!("{owner}: fixture: {problem}"));
    }
    for path in section.fixture.declared_paths() {
        if RelPath::parse(path).is_ok() {
            check_path(section, &owner, "fixture path", path, errors);
        }
    }
}

fn lint_level(section: &Section, level: &Level, errors: &mut Vec<String>) {
    let owner = level.id.to_string();

    lint_kind(level, &owner, errors);
    lint_paths(section, level, &owner, errors);
    lint_pairing(level, &owner, errors);
    lint_markers(level, &owner, errors);
    lint_hints(level, &owner, errors);
    if let Some(answer) = &level.answer {
        lint_answer(section, level, answer, &owner, errors);
    }
    lint_at_messages(section, level, &owner, errors);
}

fn lint_kind(level: &Level, owner: &str, errors: &mut Vec<String>) {
    if level.is_intro() != level.id.is_intro() {
        errors.push(format!(
            "{owner}: only an intro may be numbered N.0, and every N.0 must be an intro"
        ));
    }
    if level.is_intro() {
        if level.answer.is_some() || !level.requirements.is_empty() || level.cd.is_some() {
            errors.push(format!(
                "{owner}: an intro cannot have an answer, requirements or a cd policy"
            ));
        }
        return;
    }
    if level.answer.is_none() && level.requirements.is_empty() {
        errors.push(format!(
            "{owner}: a regular level needs an answer or at least one requirement"
        ));
    }
    if level.solution.is_none() {
        errors.push(format!(
            "{owner}: a regular level needs a solution (.solution(..), .solution_bash(..) or .solution_with(..))"
        ));
    }
}

fn lint_paths(section: &Section, level: &Level, owner: &str, errors: &mut Vec<String>) {
    for problem in level.fixture.problems() {
        errors.push(format!("{owner}: fixture: {problem}"));
    }
    for path in level.fixture.declared_paths() {
        if RelPath::parse(path).is_ok() {
            check_path(section, owner, "fixture path", path, errors);
        }
    }
    if let Start::Section(path) = &level.start {
        check_path(section, owner, "start", path, errors);
    }
    for requirement in &level.requirements {
        for path in requirement.paths() {
            check_path(section, owner, "requirement path", path, errors);
        }
        for problem in requirement.lint() {
            errors.push(format!("{owner}: {problem}"));
        }
    }
    if let Some(policy) = &level.cd {
        for path in policy.paths() {
            check_path(section, owner, "cd path", path, errors);
        }
        for problem in policy.lint() {
            errors.push(format!("{owner}: cd policy: {problem}"));
        }
    }
}

/// `cd_evidence` is only ever written by a `cd` policy, `fd_evidence` only by `on_fd`.
fn lint_pairing(level: &Level, owner: &str, errors: &mut Vec<String>) {
    for requirement in &level.requirements {
        match requirement.marker() {
            Some(Marker::Cd) if level.cd.is_none() => errors.push(format!(
                "{owner}: cd_evidence needs a cd policy (.cd(policy())): nothing else records it"
            )),
            Some(Marker::Fd) if level.on_fd.is_none() => errors.push(format!(
                "{owner}: fd_evidence needs an .on_fd(..) rule: nothing else records it"
            )),
            _ => {}
        }
    }
}

/// Private marker names become part of a file name, so they must be plain.
fn lint_markers(level: &Level, owner: &str, errors: &mut Vec<String>) {
    for marker in level.referenced_markers() {
        if let Marker::Own(name) = marker
            && (name.is_empty() || name.contains(['/', '\0']))
        {
            errors.push(format!(
                "{owner}: marker name {name:?} must be non-empty and contain no '/'"
            ));
        }
    }
}

fn lint_hints(level: &Level, owner: &str, errors: &mut Vec<String>) {
    for (index, hint) in level.hints.iter().enumerate() {
        if hint.trim().is_empty() {
            errors.push(format!("{owner}: hint {} is empty", index + 1));
        }
    }
}

fn lint_answer(
    section: &Section,
    level: &Level,
    answer: &Answer,
    owner: &str,
    errors: &mut Vec<String>,
) {
    for problem in answer.lint() {
        errors.push(format!("{owner}: answer: {problem}"));
    }
    lint_leaks(answer, owner, errors);
    lint_secrets(level, answer, owner, errors);
    lint_examples(section, level, answer, owner, errors);
}

/// The answer must never appear in a message shown for a *wrong* answer: it would hand the
/// player the solution after one failed guess.
fn lint_leaks(answer: &Answer, owner: &str, errors: &mut Vec<String>) {
    let messages = answer.message_texts();
    for expected in answer.expected_texts() {
        if expected.chars().count() < MIN_LEAK_LEN {
            continue;
        }
        let needle = expected.to_lowercase();
        for message in &messages {
            if message.to_lowercase().contains(&needle) {
                errors.push(format!(
                    "{owner}: the answer {expected:?} appears in the message {message:?}"
                ));
            }
        }
    }
}

/// A `secret` answer must not be given away by a hint or by a ready-made
/// `shellgame submit <answer>` line in the text.
fn lint_secrets(level: &Level, answer: &Answer, owner: &str, errors: &mut Vec<String>) {
    let text: String = level
        .text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    for node in answer.nodes().into_iter().filter(|node| node.is_secret()) {
        for expected in node.expected_texts() {
            let needle = expected.to_lowercase();
            if needle.is_empty() {
                continue;
            }
            for (index, hint) in level.hints.iter().enumerate() {
                if hint.to_lowercase().contains(&needle) {
                    errors.push(format!(
                        "{owner}: the secret answer {expected:?} appears in hint {}",
                        index + 1
                    ));
                }
            }
            let submitted = [
                format!("shellgame submit {needle}"),
                format!("shellgame submit \"{needle}\""),
                format!("shellgame submit '{needle}'"),
            ];
            if submitted.iter().any(|example| text.contains(example)) {
                errors.push(format!(
                    "{owner}: the text shows `shellgame submit {expected}` for a secret answer"
                ));
            }
        }
    }
}

/// `.accepts` and `.rejects` examples document what a rule means; they must hold. Rules
/// that depend on the machine (`exact_with`) cannot be evaluated here and are skipped.
fn lint_examples(
    section: &Section,
    level: &Level,
    answer: &Answer,
    owner: &str,
    errors: &mut Vec<String>,
) {
    let workspace = Path::new("/nonexistent/shellgame-lint");
    let home = Path::new("/home/student");
    let ctx = Ctx::new(
        workspace,
        workspace.join(&section.root),
        None,
        home,
        level.id,
    );
    for node in answer.nodes() {
        if node.is_dynamic() {
            continue;
        }
        for example in node.accepted_examples() {
            if let Check::Fail(message) = node.check(&ctx, example) {
                errors.push(format!(
                    "{owner}: accepts({example:?}) is rejected with {message:?}"
                ));
            }
        }
        for example in node.rejected_examples() {
            if node.check(&ctx, example).is_pass() {
                errors.push(format!("{owner}: rejects({example:?}) is accepted"));
            }
        }
    }
}

/// A custom `at(..).message(..)` must not name the directory the player has to find.
fn lint_at_messages(section: &Section, level: &Level, owner: &str, errors: &mut Vec<String>) {
    for requirement in &level.requirements {
        let Some((path, message)) = requirement.custom_at_message() else {
            continue;
        };
        let Ok(rel) = RelPath::parse(path) else {
            continue;
        };
        // `at("")` is the section root: its name is the root directory's.
        let name = rel.file_name().map(str::to_owned).or_else(|| {
            RelPath::parse(&section.root)
                .ok()?
                .file_name()
                .map(str::to_owned)
        });
        let Some(name) = name else { continue };
        // A single letter is also a Czech word ("a", "k", "s", "v", "z"): not a leak signal.
        if name.chars().count() >= 2 && contains_word(message, &name) {
            errors.push(format!(
                "{owner}: the at({path:?}) message names its target directory {name:?}: {message:?}"
            ));
        }
    }
}

/// True when `word` occurs in `text` delimited by non-word characters on both sides.
fn contains_word(text: &str, word: &str) -> bool {
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_' || c == '-';
    text.match_indices(word).any(|(start, found)| {
        let before = text[..start].chars().next_back();
        let after = text[start + found.len()..].chars().next();
        !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::answer::{exact, exact_with, int, one_of, parts, suffix};
    use crate::engine::cd::policy;
    use crate::engine::fd::Fds;
    use crate::engine::fixture::Fixture;
    use crate::engine::level::{intro, level};
    use crate::engine::require::{
        at, at_home, cd_evidence, check, evidence, fd_evidence, file, perm_bits,
    };
    use crate::engine::sample;

    fn errors_of(sections: Vec<Section>) -> Vec<String> {
        Catalog::new(sections).validate()
    }

    /// One section with the given regular level (already carrying everything it needs).
    fn errors_for(level: Level) -> Vec<String> {
        errors_of(vec![Section::new(1, "level-1").levels([level])])
    }

    /// A regular level that passes every lint.
    fn valid(number: u16) -> Level {
        level(number, "ok")
            .hints(["Nápověda."])
            .answer(exact("abc"))
            .solution("shellgame submit abc")
    }

    fn has(errors: &[String], needles: &[&str]) -> bool {
        errors
            .iter()
            .any(|error| needles.iter().all(|needle| error.contains(needle)))
    }

    #[test]
    fn the_sample_catalog_is_sound() {
        let errors = sample::catalog().validate();
        assert!(errors.is_empty(), "{errors:#?}");
    }

    #[test]
    fn an_empty_catalog_is_sound() {
        assert!(errors_of(vec![]).is_empty());
    }

    #[test]
    fn a_minimal_valid_level_has_no_errors() {
        assert!(errors_for(valid(1)).is_empty());
    }

    // ------------------------------------------------------------- structure -------

    #[test]
    fn duplicate_level_ids_are_reported() {
        let errors = errors_of(vec![Section::new(1, "level-1").levels([
            valid(3),
            valid(3),
            valid(4),
        ])]);
        assert!(has(&errors, &["1.3", "duplicate"]), "{errors:?}");
        assert_eq!(errors.len(), 1, "{errors:?}");
    }

    #[test]
    fn duplicate_sections_are_reported() {
        let errors = errors_of(vec![
            Section::new(2, "level-2").levels([valid(1)]),
            Section::new(2, "level-2").levels([valid(2)]),
        ]);
        assert!(has(&errors, &["section 2", "more than once"]), "{errors:?}");
    }

    #[test]
    fn a_level_must_carry_its_sections_number() {
        let mut stray = valid(1);
        stray.id.section = 7;
        let mut section = Section::new(1, "level-1");
        section.levels.push(stray);
        let errors = errors_of(vec![section]);
        assert!(has(&errors, &["7.1", "section 1"]), "{errors:?}");
    }

    #[test]
    fn intro_if_and_only_if_number_zero() {
        // An intro placed at a non-zero number.
        let mut wrong_number = intro("Úvod", "text");
        wrong_number.id = crate::engine::LevelId::new(1, 3);
        let mut section = Section::new(1, "level-1");
        section.levels.push(wrong_number);
        assert!(has(&errors_of(vec![section]), &["1.3", "N.0"]));

        // A regular level numbered 0.
        let errors = errors_for(valid(0));
        assert!(has(&errors, &["1.0", "N.0"]), "{errors:?}");

        // The proper intro is fine and needs no solution.
        let errors = errors_of(vec![
            Section::new(1, "level-1").intro(intro("Úvod", "text")),
        ]);
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn an_intro_cannot_be_graded() {
        let errors = errors_of(vec![
            Section::new(1, "level-1").intro(intro("Úvod", "text").answer(exact("x"))),
        ]);
        assert!(has(&errors, &["1.0", "intro cannot"]), "{errors:?}");
    }

    #[test]
    fn a_regular_level_needs_an_answer_or_a_requirement() {
        let errors = errors_for(level(1, "x").solution("true"));
        assert!(
            has(&errors, &["1.1", "answer or at least one requirement"]),
            "{errors:?}"
        );
        // A requirement alone is enough.
        assert!(errors_for(level(1, "x").require(file("a")).solution("true")).is_empty());
        // So is an answer alone.
        assert!(errors_for(level(1, "x").answer(exact("a")).solution("true")).is_empty());
    }

    #[test]
    fn a_regular_level_needs_a_solution() {
        let errors = errors_for(level(1, "x").answer(exact("a")));
        assert!(has(&errors, &["1.1", "solution"]), "{errors:?}");
        for with_solution in [
            level(1, "x").answer(exact("a")).solution("true"),
            level(1, "x").answer(exact("a")).solution_bash("true"),
            level(1, "x")
                .answer(exact("a"))
                .solution_with(|_| "true".to_owned()),
        ] {
            assert!(errors_for(with_solution).is_empty());
        }
    }

    // ----------------------------------------------------------------- paths -------

    #[test]
    fn paths_must_not_restate_the_section_root() {
        let cases = [
            valid(1).file("level-1/x.txt", "x"),
            valid(1).dir("level-1"),
            valid(1).clean(["level-1/old"]),
            valid(1).dir_mode("level-1/d", 0o700),
            valid(1).start("level-1/alpha"),
            valid(1).start("level-1"),
            valid(1).require(file("level-1/x")),
            valid(1).require(at("level-1/x")),
            valid(1).cd(policy().from_dir("level-1/a")),
            valid(1).cd(policy().source("level-1/a", "m")),
            valid(1).cd(policy().within("level-1/a", "m")),
            valid(1).cd(policy().arrive("level-1/a")),
        ];
        for level in cases {
            let errors = errors_for(level.clone());
            assert!(
                has(&errors, &["1.1", "restates the section root"]),
                "{level:?}: {errors:?}"
            );
        }
    }

    #[test]
    fn a_sibling_that_merely_starts_like_the_root_is_fine() {
        let errors = errors_for(
            valid(1)
                .file("level-10/x", "x")
                .file("level-1x/y", "y")
                .dir("level"),
        );
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn section_zero_has_no_root_to_restate() {
        let errors = errors_of(vec![
            Section::new(0, "").levels([valid(1).file("level-1/x", "x")]),
        ]);
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn paths_must_be_plain_relative_paths() {
        let cases = [
            valid(1).file("../x", "x"),
            valid(1).file("/abs", "x"),
            valid(1).dir("a/../b"),
            valid(1).start("../elsewhere"),
            valid(1).require(file("/etc/passwd")),
            valid(1).cd(policy().within("..", "m")),
        ];
        for level in cases {
            let errors = errors_for(level.clone());
            assert!(!errors.is_empty(), "{level:?} should be rejected");
            assert!(has(&errors, &["1.1"]), "{errors:?}");
        }
    }

    #[test]
    fn fixture_declarations_that_target_the_root_or_use_bad_modes_are_reported() {
        let errors = errors_for(valid(1).file(".", "x"));
        assert!(has(&errors, &["1.1", "root"]), "{errors:?}");
        let errors = errors_for(valid(1).clean([""]));
        assert!(has(&errors, &["1.1", "root"]), "{errors:?}");
        let errors = errors_for(valid(1).dir_mode("d", 0o10000));
        assert!(has(&errors, &["1.1", "0o10000"]), "{errors:?}");
    }

    #[test]
    fn the_section_fixture_is_linted_too() {
        let errors = errors_of(vec![
            Section::new(1, "level-1")
                .fixture(Fixture::new().dir("level-1/alpha").file("../x", "x"))
                .levels([valid(1)]),
        ]);
        assert!(has(&errors, &["section 1", "restates"]), "{errors:?}");
        assert!(has(&errors, &["section 1", "../x"]), "{errors:?}");
    }

    #[test]
    fn an_invalid_section_root_is_reported() {
        let errors = errors_of(vec![Section::new(1, "../level-1")]);
        assert!(has(&errors, &["section 1", "root"]), "{errors:?}");
    }

    #[test]
    fn contradictory_requirement_bits_are_reported() {
        let errors = errors_for(valid(1).require(perm_bits("f").has(0o100).lacks(0o100)));
        assert!(has(&errors, &["1.1", "requires and forbids"]), "{errors:?}");
    }

    // --------------------------------------------------------------- pairing -------

    #[test]
    fn cd_evidence_needs_a_cd_policy() {
        let errors = errors_for(valid(1).require(cd_evidence("m")));
        assert!(has(&errors, &["1.1", "cd_evidence"]), "{errors:?}");
        assert!(
            errors_for(
                valid(1)
                    .require(cd_evidence("m"))
                    .cd(policy().exact("..", "m"))
            )
            .is_empty()
        );
    }

    #[test]
    fn fd_evidence_needs_an_on_fd_rule() {
        let errors = errors_for(valid(1).require(fd_evidence("m")));
        assert!(has(&errors, &["1.1", "fd_evidence"]), "{errors:?}");
        fn both(fds: &Fds) -> bool {
            fds.both_null()
        }
        assert!(errors_for(valid(1).require(fd_evidence("m")).on_fd(both)).is_empty());
    }

    #[test]
    fn other_evidence_needs_no_pairing() {
        assert!(errors_for(valid(1).require(evidence(Marker::Pwd, "m"))).is_empty());
        assert!(
            errors_for(
                valid(1)
                    .require(evidence(Marker::Own("walk"), "m"))
                    .resets([Marker::Own("walk")])
            )
            .is_empty()
        );
    }

    #[test]
    fn private_marker_names_must_be_plain() {
        for bad in ["", "a/b"] {
            let errors = errors_for(valid(1).resets([Marker::Own(bad)]));
            assert!(has(&errors, &["1.1", "marker name"]), "{bad:?}: {errors:?}");
        }
    }

    #[test]
    fn cd_policies_that_cannot_work_are_reported() {
        let errors = errors_for(valid(1).cd(policy().bare_cd("b")));
        assert!(has(&errors, &["1.1", "cd policy", "bare_cd"]), "{errors:?}");
    }

    // --------------------------------------------------------------- content -------

    #[test]
    fn error_and_mistake_messages_must_not_contain_the_answer() {
        let leaking = [
            exact("navigator").error("Heslo není navigator."),
            exact("navigator").mistake("x", "Skoro: je to NAVIGATOR."),
            exact("level-1").error("Zkuste level-1."),
            one_of(["seznam.txt", "seznam"]).error("Je to seznam.txt"),
            suffix("password.txt").error("Hledejte password.txt"),
            int(12345).error("Skoro 12345"),
            parts(",", [int(3), exact("SECRET123")]).format_message("Formát: 3,SECRET123"),
            parts(",", [int(3), exact("SECRET123").error("tajné SECRET123")]),
            parts(",", [int(1), exact("ab")])
                .error("1,ab")
                .format_message("x"),
            exact("report.pdf").invalid("report.pdf"),
            exact("report.pdf").required("Odevzdejte report.pdf"),
        ];
        for answer in leaking {
            let errors = errors_for(level(1, "x").answer(answer.clone()).solution("true"));
            assert!(
                has(&errors, &["1.1", "appears in the message"]),
                "{answer:?} should be flagged: {errors:?}"
            );
        }
    }

    #[test]
    fn short_answers_are_not_leak_checked() {
        let errors = errors_for(
            level(1, "x")
                .answer(exact("c").error("Adresář c nebo a?"))
                .solution("true"),
        );
        assert!(errors.is_empty(), "{errors:?}");
        let errors = errors_for(
            level(1, "x")
                .answer(int(123).error("Není to 123 ani 124."))
                .solution("true"),
        );
        assert!(
            errors.is_empty(),
            "three characters are below the threshold"
        );
    }

    #[test]
    fn a_leak_is_matched_case_insensitively_and_a_clean_message_passes() {
        let clean = exact("navigator")
            .ignore_case()
            .error("Heslo není správné.")
            .mistake("toto není heslo", "To je obsah decoy.txt.");
        assert!(errors_for(level(1, "x").answer(clean).solution("true")).is_empty());
        let leaking = exact("Navigator").error("zkuste NAVIGATOR");
        assert!(!errors_for(level(1, "x").answer(leaking).solution("true")).is_empty());
    }

    #[test]
    fn secret_answers_stay_out_of_hints_and_submit_examples() {
        let secret = exact("sunshine").secret();
        let in_hint = level(1, "x")
            .hints(["Je to něco s Sunshine."])
            .answer(secret.clone())
            .solution("true");
        let errors = errors_for(in_hint);
        assert!(has(&errors, &["1.1", "secret", "hint 1"]), "{errors:?}");

        for text in [
            "Odevzdejte: `shellgame submit sunshine`",
            "Odevzdejte: `shellgame  submit \n  SUNSHINE`",
            "shellgame submit \"sunshine\"",
            "shellgame submit 'sunshine'",
        ] {
            let errors = errors_for(
                level(1, "x")
                    .text(text)
                    .answer(secret.clone())
                    .solution("true"),
            );
            assert!(
                has(&errors, &["1.1", "secret", "shellgame submit"]),
                "{text:?}: {errors:?}"
            );
        }

        // A placeholder is fine, and so is the same text for a non-secret answer.
        let ok = level(1, "x")
            .text("Odevzdejte: `shellgame submit <heslo>`")
            .hints(["Hledejte v souboru."])
            .answer(secret)
            .solution("true");
        assert!(errors_for(ok).is_empty());
        let not_secret = level(1, "x")
            .text("`shellgame submit sunshine`")
            .hints(["sunshine"])
            .answer(exact("sunshine"))
            .solution("true");
        assert!(errors_for(not_secret).is_empty());
    }

    #[test]
    fn secret_parts_are_checked_individually_and_joined() {
        let secret = parts(",", [int(3), exact("SECRET123").secret()]);
        let errors = errors_for(
            level(1, "x")
                .hints(["Heslo je SECRET123."])
                .answer(secret)
                .solution("true"),
        );
        assert!(has(&errors, &["secret", "hint 1"]), "{errors:?}");
    }

    #[test]
    fn custom_at_messages_must_not_name_the_target() {
        for (path, message) in [
            ("challenge/room2", "Jděte do room2."),
            ("challenge/room2", "room2: tam to je"),
            ("location-A", "Vraťte se do location-A."),
            ("deep/start", "Zpět na 'start'."),
        ] {
            let errors = errors_for(valid(1).require(at(path).message(message)));
            assert!(
                has(&errors, &["1.1", "names its target"]),
                "{path}: {errors:?}"
            );
        }
        // The section root's own name counts for at("").
        let errors = errors_for(valid(1).require(at("").message("Vraťte se do level-1.")));
        assert!(has(&errors, &["names its target", "level-1"]), "{errors:?}");
    }

    #[test]
    fn custom_at_messages_that_do_not_name_the_target_are_fine() {
        let cases = [
            ("challenge/room2", "Ještě tam nejste."),
            // A longer word that merely contains the name.
            ("deep/start", "Restart a začněte znovu."),
            ("room", "Hledejte room2."),
            // Compound names are other words.
            ("start", "start-up je něco jiného"),
            // One-letter directory names are also Czech words and are not checked.
            ("gamma/deep/a/b/c", "Vraťte se k c a pokračujte."),
        ];
        for (path, message) in cases {
            let errors = errors_for(valid(1).require(at(path).message(message)));
            assert!(errors.is_empty(), "{path} / {message}: {errors:?}");
        }
        // The default message names nothing, and so cannot be flagged.
        assert!(errors_for(valid(1).require(at("deep/start"))).is_empty());
        assert!(errors_for(valid(1).require(at_home())).is_empty());
    }

    #[test]
    fn whole_word_matching_respects_unicode_and_boundaries() {
        assert!(contains_word("jděte do room2.", "room2"));
        assert!(contains_word("room2", "room2"));
        assert!(contains_word("(room2)", "room2"));
        assert!(!contains_word("room22", "room2"));
        assert!(!contains_word("myroom2", "room2"));
        assert!(!contains_word("room2-b", "room2"));
        assert!(!contains_word("_room2", "room2"));
        assert!(contains_word("do složky Dokumenty.", "Dokumenty"));
        assert!(
            !contains_word("Dokumentyč", "Dokumenty"),
            "a Czech letter continues the word"
        );
        assert!(!contains_word("dokumenty", "Dokumenty"), "case-sensitive");
    }

    #[test]
    fn empty_hints_are_reported() {
        let errors = errors_for(valid(1).hints(["Dobrá.", "", "  \n"]));
        assert!(has(&errors, &["1.1", "hint 2"]), "{errors:?}");
        assert!(has(&errors, &["1.1", "hint 3"]), "{errors:?}");
        assert!(errors_for(valid(1).hints(["a", "b"])).is_empty());
        assert!(errors_for(valid(1).hints(Vec::<String>::new())).is_empty());
    }

    #[test]
    fn accepts_and_rejects_examples_are_checked() {
        let wrong_accept = valid(1).answer(exact("abc").ignore_case().accepts(["ABC", "abd"]));
        let errors = errors_for(wrong_accept);
        assert!(has(&errors, &["1.1", "accepts(\"abd\")"]), "{errors:?}");

        let wrong_reject = valid(1).answer(exact("abc").ignore_case().rejects(["xyz", "ABC"]));
        let errors = errors_for(wrong_reject);
        assert!(has(&errors, &["1.1", "rejects(\"ABC\")"]), "{errors:?}");

        let right = valid(1).answer(
            exact("abc")
                .ignore_case()
                .trim_trailing_slash()
                .accepts(["ABC", " abc/ "])
                .rejects(["abcd", ""]),
        );
        assert!(errors_for(right).is_empty());
    }

    #[test]
    fn examples_on_parts_and_their_members_are_checked() {
        let answer = parts(
            ",",
            [
                int(3).accepts(["+3"]).rejects(["x"]),
                exact("log").accepts(["log"]).rejects(["LOG"]),
            ],
        )
        .accepts(["3,log", " 3 , log "])
        .rejects(["3", "3,LOG", "4,log"]);
        assert!(errors_for(valid(1).answer(answer)).is_empty());

        let wrong = parts(",", [int(3), exact("log")]).accepts(["3;log"]);
        assert!(has(
            &errors_for(valid(1).answer(wrong)),
            &["accepts(\"3;log\")"]
        ));
    }

    #[test]
    fn dynamic_answers_are_skipped_not_misjudged() {
        let dynamic = exact_with(|ctx| ctx.home_name())
            .accepts(["anything"])
            .rejects(["ada"]);
        assert!(errors_for(valid(1).answer(dynamic)).is_empty());
        let in_parts = parts(",", [int(1), exact_with(|ctx| ctx.home_name())]).accepts(["nope"]);
        assert!(errors_for(valid(1).answer(in_parts)).is_empty());
        // ...but static members of the same tree are still checked.
        let static_member = parts(
            ",",
            [int(1).accepts(["2"]), exact_with(|ctx| ctx.home_name())],
        );
        assert!(has(
            &errors_for(valid(1).answer(static_member)),
            &["accepts(\"2\")"]
        ));
    }

    #[test]
    fn answer_construction_mistakes_are_reported() {
        let errors = errors_for(valid(1).answer(parts(",", [int(1)]).error("x")));
        assert!(has(&errors, &["1.1", "answer", "error"]), "{errors:?}");
        let errors = errors_for(valid(1).answer(one_of(Vec::<String>::new())));
        assert!(has(&errors, &["1.1", "one_of"]), "{errors:?}");
    }

    #[test]
    fn custom_requirements_are_not_judged_but_still_count_as_requirements() {
        let level = level(1, "x")
            .require(check(|_| Check::Pass))
            .solution("true");
        assert!(errors_for(level).is_empty());
    }

    #[test]
    fn errors_are_deduplicated_and_prefixed_with_the_level_id() {
        // The same leak reachable twice (joined text and part) is reported once.
        let answer = parts(",", [exact("alphabet")]).error("alphabet");
        let errors = errors_for(level(1, "x").answer(answer).solution("true"));
        let unique: HashSet<&String> = errors.iter().collect();
        assert_eq!(unique.len(), errors.len(), "{errors:?}");
        assert!(
            errors.iter().all(|error| error.starts_with("1.1:")),
            "{errors:?}"
        );
    }
}
