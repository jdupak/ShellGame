//! A small, complete sample course for the engine tests.
//!
//! It exercises every kind of level the real course has (evidence, strict `cd`, computed
//! feedback, a dynamic answer, a challenge with several requirements) and passes every
//! catalog lint, so tests can mutate it to prove that a lint fires.

use crate::engine::prelude::*;

/// The sample catalog.
pub(crate) fn catalog() -> Catalog {
    Catalog::new(sections())
}

/// The sample sections, in declaration order.
pub(crate) fn sections() -> Vec<Section> {
    vec![section0(), section1(), section2()]
}

fn section0() -> Section {
    Section::new(0, "")
        .intro(
            intro("Vítejte v ShellGame", "Toto je úvod.\n---\nDruhá strana.")
                .start("")
                .success("Vítejte ve hře!"),
        )
        .levels([level(1, "Zahřívací kolo")
            .text("Odevzdejte heslo `start`.")
            .hints([
                "Heslo je v zadání.",
                "Opravdu jen napište: shellgame submit start",
            ])
            .start("")
            .answer(exact("start").ignore_case())
            .solution("shellgame submit start")])
}

fn section1() -> Section {
    Section::new(1, "level-1")
        .fixture(
            Fixture::new()
                .dirs(["alpha", "delta", "gamma", "patterns/omega"])
                .keep_file("alpha/inside.txt", "first step")
                .keep_file("delta/single.dat", "momentum gained"),
        )
        .intro(
            intro("Navigace", "Naučíte se `pwd`, `ls` a `cd`.")
                .start_at_workspace()
                .success("Jdeme na to!"),
        )
        .levels([
            pwd_level(),
            ls_level(),
            extension_level(),
            cd_up_level(),
            cd_three_up_level(),
            home_level(),
        ])
}

fn pwd_level() -> Level {
    level(1, "Aktuální umístění")
        .text("Spusťte `pwd` a odevzdejte název posledního adresáře.")
        .hints([
            "Příkaz 'pwd' ukáže, kde jste.",
            "Basename je poslední část cesty.",
        ])
        .start("")
        .answer(exact("level-1"))
        .require(evidence(Marker::Pwd, messages::L1_1_USE_PWD_FIRST))
        .solution("pwd\nshellgame submit level-1")
}

/// 1.2: literal mistakes first, then computed feedback about the shape `d...a`.
fn ls_level() -> Level {
    fn shape(answer: &str) -> Option<String> {
        if answer.starts_with('d') && !answer.ends_with('a') {
            Some(format!(
                "'{answer}' začíná na 'd', ale nekončí na 'a'. Hledáme vzor d...a."
            ))
        } else if answer.ends_with('a') && !answer.starts_with('d') {
            Some(format!(
                "'{answer}' končí na 'a', ale nezačíná na 'd'. Hledáme vzor d...a."
            ))
        } else {
            None
        }
    }
    level(2, "Soubory, nebo adresáře?")
        .text("Najděte **adresář** začínající na `d` a končící na `a`.")
        .hints(["Zkuste `ls -l`: adresáře začínají `d`."])
        .start("")
        .clean(["data", "dog", "dome"])
        .dirs(["delta", "dome"])
        .file("data", "")
        .file("dog", "")
        .answer(
            exact("delta")
                .trim_trailing_slash()
                .mistake("data", "'data' odpovídá názvem, ale není to adresář.")
                .mistake("dome", "'dome' je adresář, ale jeho název nekončí na 'a'.")
                .feedback(shape)
                .required("Musíte zadat název adresáře: shellgame submit <hodnota>")
                .accepts(["delta", "delta/", " delta "])
                .rejects(["Delta", "data", "dome", "dxa", "dxa/"]),
        )
        .solution("shellgame submit \"$(ls -F | grep '^d.*a/$' | tr -d /)\"")
}

fn extension_level() -> Level {
    fn dotted(answer: &str) -> Option<String> {
        answer
            .contains('.')
            .then(|| messages::L1_3_INCLUDED_EXTENSION.to_owned())
    }
    level(3, "Vstup a hlášení")
        .text("Jděte do `alpha` a odevzdejte název souboru bez přípony.")
        .hints(["Vypište obsah pomocí 'ls'."])
        .start("")
        .answer(
            exact("inside")
                .mistake("inside.txt", messages::L1_3_INCLUDED_EXTENSION)
                .mistake("alpha", "'alpha' je název adresáře, ne souboru uvnitř.")
                .feedback(dotted),
        )
        .require(at("alpha"))
        .solution("cd alpha\nshellgame submit \"$(ls | sed 's/\\.[^.]*$//')\"")
}

fn cd_up_level() -> Level {
    level(4, "Návrat na základnu")
        .text("Vraťte se o úroveň výše příkazem `cd ..`.")
        .hints(["Dvě tečky označují rodiče."])
        .start("alpha")
        .answer(exact("level-1"))
        .require(at(""))
        .require(cd_evidence("Použijte pro návrat přesně příkaz `cd ..`."))
        .cd(policy().exact("..", "Použijte přesně příkaz 'cd ..'."))
        .success("Správně! `..` vždy odkazuje na rodiče.")
        .solution("cd ..\nshellgame submit level-1")
}

fn cd_three_up_level() -> Level {
    level(6, "Víceúrovňový výstup")
        .text("Vraťte se o 3 úrovně výše **jedním** příkazem.")
        .hints(["Cesty lze řetězit."])
        .start("gamma/deep/a/b/c")
        .dir("gamma/deep/a/b/c")
        .require(at("gamma/deep"))
        .require(cd_evidence("Vraťte se jedním příkazem `cd ../../..`."))
        .cd(policy().exact("../../..", "Použijte jeden příkaz 'cd ../../..'."))
        .solution("cd ../../..\nshellgame submit")
}

/// 1.10: the answer depends on the machine, and so does the solution.
fn home_level() -> Level {
    level(10, "Zkratka pro domov (~)")
        .text("Jděte domů zkratkou `~` a odevzdejte název domovského adresáře.")
        .hints(["Zkratka `~` je váš domov."])
        .start("")
        .answer(exact_with(|ctx| ctx.home_name()))
        .require(at_home())
        .solution_with(|ctx| format!("cd ~\nshellgame submit {}", ctx.home_name()))
}

fn section2() -> Section {
    Section::new(2, "level-2")
        .intro(intro("Práce se soubory", "Úvod do sekce 2."))
        .levels([toggle_level(), challenge_level()])
}

/// 2.2: `cd -`, scoped to its target, with a lifted move that must still count.
fn toggle_level() -> Level {
    level(2, "Rychlý návrat")
        .text("Přejděte do `location-B` a vraťte se příkazem `cd -`.")
        .hints(["Příkaz 'cd -' vás vrátí do předchozího adresáře."])
        .start("location-A")
        .dirs(["location-A", "location-B"])
        .require(at("location-A"))
        .require(cd_evidence(
            "Nejdřív přejděte do `location-B` a vraťte se příkazem `cd -`.",
        ))
        .cd(policy().target("-").source(
            "location-B",
            "Příkaz 'cd -' použijte až z adresáře location-B.",
        ))
        .solution("cd ../location-B\ncd -\nshellgame submit")
        .allow_empty()
}

/// 2.8: several files, a decoy, a literal mistake and a requirement, as in the plan.
fn challenge_level() -> Level {
    level(8, "Integrační výzva")
        .text("Najděte heslo v sousedním adresáři.")
        .start(".")
        .hints([
            "Ze startu přejděte do 'challenge/room1' a přečtěte hint.txt.",
            "Heslo je v souboru, který se jmenuje jinak než decoy.",
            "Zadejte: shellgame submit <heslo>",
        ])
        .file("challenge/room1/hint.txt", "Heslo je v sousedním adresáři room2.\n")
        .file("challenge/room2/password.txt", "navigator\n")
        .file("challenge/room2/decoy.txt", "Toto není heslo.\n")
        .answer(
            exact("navigator")
                .ignore_case()
                .mistake(
                    "toto není heslo",
                    "To je obsah decoy.txt, ne password.txt. Hledejte jiný soubor.",
                )
                .error("Heslo není správné. Hledejte password.txt v room2.")
                .required("Musíte zadat heslo: shellgame submit <hodnota>")
                .accepts(["Navigator", " navigator "])
                .rejects(["navigator2", "toto není heslo"]),
        )
        .require(at("challenge/room2"))
        .solution("cd challenge/room1\ncat hint.txt\ncd ../room2\nshellgame submit \"$(cat password.txt)\"")
        .success("Výborně! Dokončili jste sekci.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// The sample course in a temporary workspace, with a home directory.
    struct Game {
        _tmp: tempfile::TempDir,
        workspace: PathBuf,
        home: PathBuf,
        catalog: Catalog,
    }

    impl Game {
        fn new() -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let base = fs::canonicalize(tmp.path()).unwrap();
            let workspace = base.join("shellgame workspace");
            let home = base.join("home/ada");
            fs::create_dir_all(&home).unwrap();
            Self {
                _tmp: tmp,
                workspace,
                home,
                catalog: catalog(),
            }
        }

        fn id(section: u16, number: u16) -> LevelId {
            LevelId::new(section, number)
        }

        /// Prepare a level and return its context with the player at `cwd`.
        fn prepare(&self, id: LevelId, cwd: Option<PathBuf>) -> Ctx<'_> {
            let (section, level) = self.catalog.get(id).unwrap();
            level.prepare(section, &self.workspace).unwrap();
            section
                .ctx(level, &self.workspace, cwd, &self.home)
                .unwrap()
        }

        fn level(&self, id: LevelId) -> &Level {
            self.catalog.get(id).unwrap().1
        }

        fn start(&self, id: LevelId) -> PathBuf {
            let (section, level) = self.catalog.get(id).unwrap();
            let root = section.section_root(&self.workspace).unwrap();
            level.start_dir(&self.workspace, &root).unwrap()
        }
    }

    fn pass(check: &Check) {
        assert_eq!(*check, Check::Pass);
    }

    #[test]
    fn the_sample_has_the_expected_levels() {
        let catalog = catalog();
        let ids: Vec<String> = catalog.ids().map(|id| id.to_string()).collect();
        assert_eq!(
            ids,
            [
                "0.0", "0.1", "1.0", "1.1", "1.2", "1.3", "1.4", "1.6", "1.10", "2.0", "2.2", "2.8"
            ]
        );
        assert_eq!(
            catalog.get(Game::id(1, 4)).unwrap().1.display_title(),
            "1.4: Návrat na základnu"
        );
        assert_eq!(
            catalog.get(Game::id(1, 0)).unwrap().1.display_title(),
            "Navigace"
        );
    }

    #[test]
    fn every_regular_level_is_prepared_and_starts_in_an_existing_directory() {
        let game = Game::new();
        for (section, level) in game.catalog.iter() {
            level.prepare(section, &game.workspace).unwrap();
            let root = section.section_root(&game.workspace).unwrap();
            if let Some(start) = level.start_dir(&game.workspace, &root) {
                assert!(start.is_dir(), "{}: start {start:?} should exist", level.id);
            }
        }
        assert!(game.workspace.join("level-1/alpha/inside.txt").is_file());
        assert!(
            game.workspace
                .join("level-2/challenge/room2/password.txt")
                .is_file()
        );
    }

    #[test]
    fn the_challenge_is_won_by_its_documented_route() {
        let game = Game::new();
        let id = Game::id(2, 8);
        let ctx = game.prepare(id, Some(game.start(id)));
        let level = game.level(id);

        // At the start: nothing done yet.
        assert_eq!(
            level.validate(&ctx, Some("navigator")),
            Check::fail(messages::WRONG_DIRECTORY),
            "requirements are unconditional: the right answer is not enough"
        );
        // The decoy and the empty submission.
        let room2 = Ctx {
            cwd: Some(ctx.section_root.join("challenge/room2")),
            ..ctx.clone()
        };
        assert_eq!(
            level.validate(&room2, Some("Toto není heslo")),
            Check::fail("To je obsah decoy.txt, ne password.txt. Hledejte jiný soubor.")
        );
        assert_eq!(
            level.validate(&room2, Some("wrong")),
            Check::fail("Heslo není správné. Hledejte password.txt v room2.")
        );
        assert_eq!(
            level.validate(&room2, None),
            Check::fail("Musíte zadat heslo: shellgame submit <hodnota>")
        );
        pass(&level.validate(&room2, Some(" NAVIGATOR ")));
    }

    #[test]
    fn evidence_levels_need_the_evidence_and_the_answer() {
        let game = Game::new();
        let id = Game::id(1, 1);
        let ctx = game.prepare(id, Some(game.start(id)));
        let level = game.level(id);
        assert_eq!(
            level.validate(&ctx, Some("level-1")),
            Check::fail(messages::L1_1_USE_PWD_FIRST)
        );
        ctx.markers().create(&Marker::Pwd).unwrap();
        pass(&level.validate(&ctx, Some("level-1")));
        // Preparing the level again forgets the evidence.
        let ctx = game.prepare(id, Some(game.start(id)));
        assert!(level.validate(&ctx, Some("level-1")).is_fail());
    }

    #[test]
    fn the_strict_cd_level_is_won_with_the_exact_command() {
        let game = Game::new();
        let id = Game::id(1, 4);
        let alpha = game.start(id);
        let ctx = game.prepare(id, Some(alpha.clone()));
        let level = game.level(id);
        let root = ctx.section_root.clone();

        // Standing at the start, no evidence: at("") fails first.
        assert_eq!(
            level.validate(&ctx, Some("level-1")),
            Check::fail(messages::WRONG_DIRECTORY)
        );

        // `cd ../..` is blocked; `cd ..` is allowed, and recorded after it succeeds.
        let args = vec!["../..".to_owned()];
        let mv = CdMove::new(&ctx, "../..", &args, &alpha);
        assert_eq!(
            level.cd_pre_move(&mv),
            CdDecision::Block("Použijte přesně příkaz 'cd ..'.".to_owned())
        );
        let args = vec!["..".to_owned()];
        let mv = CdMove::new(&ctx, "..", &args, &alpha);
        assert_eq!(level.cd_pre_move(&mv), CdDecision::Allow);
        level.cd_post_move(&mv.with_dest(&root));

        let ctx = Ctx {
            cwd: Some(root),
            ..ctx
        };
        pass(&level.validate(&ctx, Some("level-1")));
    }

    #[test]
    fn the_toggle_level_counts_cd_dash_even_though_it_returns_to_the_start() {
        let game = Game::new();
        let id = Game::id(2, 2);
        let a = game.start(id);
        let b = a.parent().unwrap().join("location-B");
        let ctx = game.prepare(id, Some(a.clone()));
        let level = game.level(id);
        assert!(level.validate(&ctx, None).is_fail());

        let args = vec!["-".to_owned()];
        let mv = CdMove::new(&ctx, "-", &args, &b).with_prev(Some(&a));
        assert_eq!(
            level.cd_pre_move(&mv),
            CdDecision::Allow,
            "lifted: it is the start directory"
        );
        level.cd_post_move(&mv.with_dest(&a));
        pass(&level.validate(&ctx, None));
    }

    #[test]
    fn the_dynamic_answer_follows_the_home_directory() {
        let game = Game::new();
        let id = Game::id(1, 10);
        let ctx = game.prepare(id, Some(game.home.clone()));
        let level = game.level(id);
        pass(&level.validate(&ctx, Some("ada")));
        assert!(level.validate(&ctx, Some("bob")).is_fail());
        let elsewhere = Ctx {
            cwd: Some(game.workspace.clone()),
            ..ctx
        };
        assert!(
            level.validate(&elsewhere, Some("ada")).is_fail(),
            "at_home is a requirement"
        );
        let script = level.solution.as_ref().unwrap().script(&elsewhere);
        assert_eq!(script, "cd ~\nshellgame submit ada");
        assert!(level.solution.as_ref().unwrap().is_dynamic());
    }

    #[test]
    fn computed_feedback_guides_the_ls_level() {
        let game = Game::new();
        let id = Game::id(1, 2);
        let ctx = game.prepare(id, Some(game.start(id)));
        let level = game.level(id);
        pass(&level.validate(&ctx, Some("delta/")));
        assert_eq!(
            level.validate(&ctx, Some("dome")),
            Check::fail("'dome' je adresář, ale jeho název nekončí na 'a'."),
            "the literal mistake wins over the computed shape feedback"
        );
        assert_eq!(
            level.validate(&ctx, Some("dxx")),
            Check::fail("'dxx' začíná na 'd', ale nekončí na 'a'. Hledáme vzor d...a.")
        );
        assert_eq!(
            level.validate(&ctx, Some("xxa/")),
            Check::fail("'xxa' končí na 'a', ale nezačíná na 'd'. Hledáme vzor d...a.")
        );
        assert_eq!(
            level.validate(&ctx, Some("zzz")),
            Check::fail(messages::INCORRECT)
        );
    }

    #[test]
    fn no_static_answer_passes_from_the_start_directory_right_after_prepare() {
        // The shortcut probe: requirements are unconditional, so submitting the right
        // answer without doing the work must fail.
        let game = Game::new();
        for (section, level) in game.catalog.iter() {
            let Some(answer) = level.answer.as_ref().and_then(Answer::static_answer) else {
                continue;
            };
            if level.requirements.is_empty() {
                continue;
            }
            level.prepare(section, &game.workspace).unwrap();
            let root = section.section_root(&game.workspace).unwrap();
            let cwd = level.start_dir(&game.workspace, &root);
            let ctx = section
                .ctx(level, &game.workspace, cwd, &game.home)
                .unwrap();
            assert!(
                level.validate(&ctx, Some(&answer)).is_fail(),
                "{}: the answer {answer:?} must not pass without the work",
                level.id
            );
        }
    }

    #[test]
    fn solutions_exist_for_every_regular_level() {
        let game = Game::new();
        let ctx_home = game.home.clone();
        for (section, level) in game.catalog.iter().filter(|(_, level)| !level.is_intro()) {
            let ctx = Ctx::new(
                &game.workspace,
                game.workspace.join(&section.root),
                None,
                &ctx_home,
                level.id,
            );
            let script = level.solution.as_ref().unwrap().script(&ctx);
            assert!(!script.trim().is_empty(), "{}", level.id);
        }
    }

    #[test]
    fn resolving_the_stale_ids_of_the_sample() {
        let catalog = catalog();
        assert_eq!(catalog.resolve_str("1.5"), Some(Game::id(1, 6)));
        assert_eq!(catalog.resolve_str("1.11"), Some(Game::id(2, 0)));
        assert_eq!(catalog.resolve_str("9.9"), Some(Game::id(2, 8)));
    }
}
