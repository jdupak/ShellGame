//! Section 10: error output.

use crate::engine::prelude::*;
use crate::levels::script;

/// A script that writes to both standard output and standard error.
const BUGGY_SCRIPT: &str = "\
    #!/bin/bash\n\
    echo 'This is normal output'\n\
    echo 'This is an error message' >&2\n";

/// A script that interleaves three lines of output with two errors.
const MIXED_SCRIPT: &str = "\
    #!/bin/bash\n\
    echo \"Line 1 - normal output\"\n\
    echo \"ERROR: Something went wrong\" >&2\n\
    echo \"Line 2 - more output\"\n\
    echo \"ERROR: Another problem\" >&2\n\
    echo \"Line 3 - final output\"\n";

/// Section 10 and its levels.
pub fn section() -> Section {
    Section::new(10, "level-10")
        .intro(intro(
            "Sekce 10: Chybové výstupy",
            include_str!("00-uvod.md"),
        ))
        .levels([l1(), l2(), l3(), l4(), l5()])
}

fn l1() -> Level {
    level(1, "Přesměrování chyb")
        .text(include_str!("01-presmerovani-chyb.md"))
        .hints([
            "Běžný výstup jde na stdout (1), chyby na stderr (2). Jak přesměrujete jen \
             dvojku?",
            "Syntaxe je: příkaz 2> soubor. Zkuste to se skriptem buggy.sh.",
            "Použijte './buggy.sh 2> errors.log'.",
        ])
        .start(".")
        .clean(["errors.log"])
        .file_mode("buggy.sh", BUGGY_SCRIPT, 0o755)
        .answer(exact("errors.log"))
        .require(
            content("errors.log")
                .excludes(["This is normal output"])
                .message("Soubor obsahuje i normální výstup (použili jste &> nebo chybí 2?).")
                .missing("Soubor neexistuje."),
        )
        .require(
            content("errors.log")
                .contains(["This is an error message"])
                .message("Soubor neobsahuje očekávanou chybu."),
        )
        .solution(script([
            "./buggy.sh 2> errors.log",
            "shellgame submit 'errors.log'",
        ]))
        .success("Správně! Soubor obsahuje pouze chyby.")
}

fn l2() -> Level {
    level(2, "Přidávání chyb")
        .text(include_str!("02-pridavani-chyb.md"))
        .hints([
            "Jaký je rozdíl mezi > a >>? Jeden přepisuje, druhý přidává.",
            "Pro přidání chyb na konec použijte dvě šipky: 2>>",
            "Použijte './buggy.sh 2>> errors.log'.",
        ])
        .start(".")
        .file_mode("buggy.sh", BUGGY_SCRIPT, 0o755)
        .file("errors.log", "Old error 1\n")
        .answer(exact("errors.log"))
        .require(
            content("errors.log")
                .contains(["Old error 1"])
                .message("Původní obsah zmizel (použili jste 2> místo 2>>?).")
                .missing("Soubor neexistuje."),
        )
        .require(
            content("errors.log")
                .contains(["This is an error message"])
                .message("Soubor neobsahuje novou chybu."),
        )
        .solution(script([
            "./buggy.sh 2>> errors.log",
            "shellgame submit 'errors.log'",
        ]))
        .success(
            "Správně! Dvojitá šipka zapisuje na konec, takže předchozí záznamy v logu zůstaly \
             zachovány.",
        )
}

fn l3() -> Level {
    level(3, "Všechny výstupy")
        .text(include_str!("03-vsechny-vystupy.md"))
        .hints([
            "Ampersand (&) v tomto kontextu znamená 'obojí' - stdout i stderr.",
            "Kombinace &> je zkratka pro přesměrování obou výstupů.",
            "Použijte './buggy.sh &> all_output.log'.",
        ])
        .start(".")
        .clean(["all_output.log"])
        .file_mode("buggy.sh", BUGGY_SCRIPT, 0o755)
        .answer(exact("all_output.log"))
        .require(
            content("all_output.log")
                .contains(["This is normal output", "This is an error message"])
                .message(
                    "Soubor neobsahuje oba typy výstupů. Chybí-li normální výstup, přesměrovali \
                     jste jen chyby (2>). Chybí-li chyby, zůstaly na obrazovce, protože samotné \
                     > bere jen stdout.",
                )
                .missing("Soubor neexistuje."),
        )
        .solution(script([
            "./buggy.sh &> all_output.log",
            "shellgame submit 'all_output.log'",
        ]))
        .success("Správně! Máme všechno.")
}

/// 10.4: `buggy.sh` ends by running `$SHELLGAME_FD_HOOK`, which tells the game where its
/// output was sent.
fn l4() -> Level {
    level(4, "Černá díra")
        .text(include_str!("04-dev-null.md"))
        .hints([
            "Kam v Linuxu 'vyhodíte' data, která nechcete? Existuje speciální soubor...",
            "Soubor /dev/null je jako černá díra - vše pohltí a nic nevrátí.",
            "Použijte './buggy.sh &> /dev/null'.",
        ])
        .start(".")
        .file_mode(
            "buggy.sh",
            format!("{BUGGY_SCRIPT}\"$SHELLGAME_FD_HOOK\"\n"),
            0o755,
        )
        .answer(
            exact("/dev/null")
                .mistake(
                    "null",
                    "Skoro. Odevzdejte celou cestu k tomu speciálnímu souboru, ne jen jeho \
                     název.",
                )
                .mistake(
                    "dev/null",
                    "Téměř. Jde o absolutní cestu, začíná tedy lomítkem od kořenového adresáře.",
                )
                .accepts(["/dev/null"])
                .rejects(["null", "dev/null", "/dev/zero"]),
        )
        .require(fd_evidence(
            "Spusťte `./buggy.sh` a přesměrujte stdout i stderr do `/dev/null`.",
        ))
        .on_fd(|fds| fds.stdout_is_null() && fds.stderr_is_null())
        .solution(script([
            "./buggy.sh &> /dev/null",
            "shellgame submit '/dev/null'",
        ]))
        .success(
            "Správně! Zápis do /dev/null se rovnou zahodí, takže příkaz umlčíte bez zakládání \
             logu.",
        )
}

fn l5() -> Level {
    level(5, "Souhrn Sekce 10")
        .text(include_str!("05-souhrn-sekce-10.md"))
        .hints([
            "Chyby se zapisují na stderr (descriptor 2), standardní výstup na stdout \
             (descriptor 1). Každý proud může mít vlastní cíl.",
            "Za jeden příkaz lze zapsat přesměrování '>' i '2>'; pořadí zde nevadí, protože \
             oba proudy míří do různých souborů.",
            "Spusťte './mixed.sh > output.log 2> errors.log'. Počty zjistěte pomocí 'wc -l \
             errors.log output.log'.",
        ])
        .start("challenge")
        .clean(["challenge"])
        .file_mode("challenge/mixed.sh", MIXED_SCRIPT, 0o755)
        .answer(
            parts(
                ",",
                [
                    int(2)
                        .error(
                            "Počet chyb není správně. Zkontrolujte, že do errors.log směřuje \
                             pouze descriptor 2.",
                        )
                        .invalid("Obě hodnoty musí být čísla."),
                    int(3)
                        .error(
                            "Počet normálních řádků není správně. Zkontrolujte, že do output.log \
                             směřuje pouze standardní výstup.",
                        )
                        .invalid("Obě hodnoty musí být čísla."),
                ],
            )
            .format_message("Formát odpovědi: chyby,výstup (např. 3,5)")
            .accepts(["2,3", "2, 3"])
            .rejects(["3,2", "2", "2,3,0"]),
        )
        .require(line_count(
            "challenge/errors.log",
            2,
            "errors.log neobsahuje přesně pouze chybový výstup skriptu. Buď do něj spadl i \
             standardní výstup (oba proudy míří do jednoho souboru), nebo jste skript \
             spustili víckrát a záznamy se nasčítaly.",
        ))
        .require(line_count(
            "challenge/output.log",
            3,
            "output.log neobsahuje přesně pouze standardní výstup skriptu. Buď v něm \
             skončily i chyby (chybí samostatné přesměrování druhého proudu), nebo jste \
             skript spustili víckrát a záznamy se nasčítaly.",
        ))
        .solution(script([
            "./mixed.sh > output.log 2> errors.log",
            r#"shellgame submit "$(wc -l < errors.log),$(wc -l < output.log)""#,
        ]))
        .success(
            "Perfektní! Dokončili jste Sekci 10. Stdout a stderr jsou pro vás jako otevřená \
             kniha!",
        )
}
