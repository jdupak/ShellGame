//! The welcome: how the game works, and the first `shellgame submit`.
//!
//! **Purpose.** Teach the game's own loop before any shell skill is needed, so that no later
//! section has to explain mechanics. It works in `level-0` and asks for no shell command.
//!
//! **Learning objectives.** After this section a student can:
//! - read the intro pages (Enter continues) and say what an assignment may ask for: a typed answer,
//!   a location reached, or changed files;
//! - hand in an answer with `shellgame submit <answer>` (the warm-up wants the password it names);
//! - ask for a hint with `shellgame hint` and find the other game commands with `shellgame --help`.
//!
//! **Introduces:** `shellgame submit`, `shellgame hint`, `shellgame --help` (only pointed at).\
//! **Relies on:** nothing; it is the first section.\
//! **Leaves for later:** every shell command, beginning with [navigation](super::s01_navigation)
//! (where `shellgame reset` first appears); other game commands are only pointed at via `--help`.

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Vítejte v ShellGame",
    intro: r#"
        Vítejte v interaktivní výuce příkazové řádky!

        ### O hře

        ShellGame je série úkolů, které plníte přímo ve svém terminálu.
        Hra vám zadá úkol, vy ho splníte pomocí příkazů shellu a poté odevzdáte výsledek.
        Hra pak zkontroluje výsledek: vaši odpověď, adresář, ve kterém stojíte, a stav souborů.
        U několika úkolů si navíc všímá, zda jste použili příkaz `pwd` nebo předepsaný `cd`.
        Výsledkem může být textová odpověď, přesun do správného adresáře nebo úprava souborů a adresářů.

        ### Jak hrát
        1. Hra se skládá ze sekcí, které se věnují různým tématům.
        2. Každá sekce začíná úvodním textem. Úvod jen čtete — pokračujte stisknutím Enter.
        3. Každá sekce obsahuje několik levelů, ve kterých postupně plníte úkoly.
        4. Hotový úkol se odevzdává příkazem `shellgame submit`. Pokud úkol chce odpověď, píše se za něj
           (`shellgame submit <odpověď>`); jinak se příkaz spouští bez ní.
        5. Když si nevíte rady, nápovědu zobrazí příkaz `shellgame hint`.

        ### Další příkazy

        Další příkazy (např. obnovení levelu do výchozího stavu) najdete v nápovědě:
        `shellgame --help`
    "#,
    levels: &[
        Level {
            title: "Zahřívací kolo",
            text: r#"
                ### Zahřívací kolo

                Toto je první testovací úkol, abychom si ověřili, že vše funguje.

                ### Úkol
                Pro postup do první sekce stačí odevzdat heslo `start`.

                Odevzdejte pomocí: `shellgame submit start`
            "#,
            hints: &[
                "Heslo je uvedeno přímo v zadání úkolu výše.",
                "Zadejte příkaz 'shellgame submit' následovaný tímto heslem.",
                "Opravdu jen napište: shellgame submit start",
            ],
            answer: exact("start").nocase(),
            solution: "shellgame submit start",
            success: "Správně! Takhle se odevzdávají úkoly: 'shellgame submit', a když úkol chce odpověď, napíšete ji za něj.",
            ..LEVEL
        },
    ],
};
