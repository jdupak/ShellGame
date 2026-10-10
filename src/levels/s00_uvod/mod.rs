//! Section 0: the introduction and the game mechanics.

use crate::engine::prelude::*;

/// Section 0 and its levels.
pub fn section() -> Section {
    Section::new(0, "")
        .intro(intro("Vítejte v ShellGame", include_str!("00-uvod.md")).start("."))
        .levels([l1()])
}

fn l1() -> Level {
    level(1, "Zahřívací kolo")
        .text(include_str!("01-zahrivaci-kolo.md"))
        .hints([
            "Heslo je uvedeno přímo v zadání úkolu výše.",
            "Zadejte příkaz 'shellgame submit' následovaný tímto heslem.",
            "Opravdu jen napište: shellgame submit start",
        ])
        .start(".")
        .answer(
            exact("start")
                .ignore_case()
                .accepts(["start", "START", " Start "])
                .rejects(["stop", ""]),
        )
        .solution("shellgame submit start")
        .success("Správně! Takhle se odevzdává každý úkol: `shellgame submit` a vaše odpověď.")
}
