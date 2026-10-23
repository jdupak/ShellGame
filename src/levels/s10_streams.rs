//! Error output: stdout and stderr as separate streams, steered with `2>`, `&>` and `/dev/null`.
//!
//! **Purpose.** A command's complaints do not travel with its results. The section separates the
//! two streams: log errors, keep both, silence a noisy command, or split them in one run.
//!
//! **Learning objectives.** After this section a student can:
//! - name the two output streams (stdout, fd 1; stderr, fd 2) and say which operator touches which
//!   (`>` stdout only, `2>` stderr only);
//! - save only the errors with `2>` and append them with `2>>` without losing earlier lines;
//! - send both streams to one file with `&>`, or discard everything with `&> /dev/null`;
//! - split the streams of one run into two files with one command (`> out 2> err`) and count lines.
//!
//! **Introduces:** stdout and stderr (fd 1 and 2), `2>`, `2>>`, `&>`, `/dev/null`, running a script
//! as `./name` (or `bash name` where /tmp does not allow running programs).\
//! **Relies on:** [redirection](super::s09_redirection) (`>`, `>>`, `wc -l`); the scripts come
//! ready to run, as shown in [permissions](super::s08_permissions).\
//! **Leaves for later:** not in the course: `2>&1`, streams inside pipelines, `tee`.

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Chybové výstupy",
    intro: r#"
        Každý příkaz v Linuxu má dva výstupní kanály:
        1. **stdout** (fd 1) - standardní výstup pro normální výsledky
        2. **stderr** (fd 2) - chybový výstup pro chybové zprávy

        ### Dva proudy výstupu
        ```
        ┌─────────┐
        │ příkaz  │──── stdout (1) ───▶ Normální výstup
        │         │──── stderr (2) ───▶ Chybové zprávy
        └─────────┘
        ```

        ### Operátory přesměrování
        ```
        >     Přesměruje stdout (normální výstup)
        2>    Přesměruje stderr (chyby)
        &>    Přesměruje OBOJÍ (stdout + stderr)
        >>    Přidá stdout na konec souboru
        2>>   Přidá stderr na konec souboru
        ```

        ### Příklad
        ```bash
        ./skript.sh                  # Obojí na obrazovku
        ./skript.sh > out.log        # stdout do souboru, stderr na obrazovku
        ./skript.sh 2> err.log       # stderr do souboru, stdout na obrazovku
        ./skript.sh &> all.log       # Všechno do souboru
        ./skript.sh &> /dev/null     # Zahodí všechno (ticho)
        ```

        ### Co se naučíte
        - Standardní chybový výstup (stderr)
        - Přesměrování chybových zpráv (`2>`)
        - Přesměrování všeho (`&>`)
        - Zahazování výstupu (`/dev/null`)
    "#,
    levels: &[
        Level {
            title: "Přesměrování chyb",
            text: r#"
                Standardní chybový výstup (stderr) používá deskriptor souboru 2.
                Pro přesměrování pouze chyb použijte `2>`.

                ### Proč je to důležité
                Při běhu programů často chcete zachytit chybové hlášky do logu,
                zatímco normální výstup zobrazíte uživateli. Oddělení stdout a stderr
                je klíčové pro diagnostiku problémů.

                ### Jak spustit skript
                `./buggy.sh` spustí soubor `buggy.sh` z aktuálního adresáře. Předpona `./` říká shellu,
                kde soubor leží; bez ní by ho hledal jen v adresářích s programy (proměnná `PATH`).
                Skript potřebuje právo `x`, které už má. Kdyby přesto hlásil „Permission denied“
                (některé systémy nedovolují spouštět programy z /tmp), spusťte ho jako `bash buggy.sh`.

                ### Úkol
                V adresáři je skript `buggy.sh`, který vypisuje normální text i chybové zprávy.
                Spusťte ho a přesměrujte POUZE chybové zprávy do souboru `errors.log`.

                ### Příkazy
                - `./script 2> soubor` - přesměruje stderr do souboru

                ### Odevzdání
                Po vytvoření souboru spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Běžný výstup jde na stdout (1), chyby na stderr (2). Jak přesměrujete jen dvojku?",
                "Syntaxe je: příkaz 2> soubor. Zkuste to se skriptem buggy.sh.",
                "Použijte './buggy.sh 2> errors.log' a potom 'shellgame submit'.",
            ],
            setup: &[
                clean("errors.log"),
                file("buggy.sh", r#"
                    #!/bin/bash
                    echo 'This is normal output'
                    echo 'This is an error message' >&2
                "#).mode(0o755),
            ],
            check: &[
                content_is("errors.log", "This is an error message\n")
                    .msg("Soubor má obsahovat jen chybovou hlášku skriptu, jednou a bez normálního výstupu (použili jste &> nebo chybí 2?).")
                    .missing("Soubor errors.log zatím neexistuje."),
            ],
            solution: r#"
                ./buggy.sh 2> errors.log
                shellgame submit
            "#,
            success: "Správně! Soubor obsahuje pouze chyby.",
            ..LEVEL
        },
        Level {
            title: "Přidávání chyb",
            text: r#"
                Stejně jako u normálního výstupu se i u chyb rozhodujete mezi dvěma operátory:
                jeden soubor před zápisem vyprázdní, druhý zapisuje na jeho konec.

                ### Úkol
                V souboru `errors.log` už je z minula záznam `Old error 1`.
                Spusťte `buggy.sh` znovu tak, aby v souboru zůstal starý záznam i nová chyba.

                Než příkaz spustíte, rozmyslete si: který z obou operátorů soubor nejdřív
                vyprázdní a co by se v takovém případě stalo s řádkem `Old error 1`?

                ### Příkazy
                - `./script 2> soubor` - chybový výstup přepíše obsah souboru
                - `./script 2>> soubor` - chybový výstup přidá na konec souboru

                ### Odevzdání
                Po spuštění skriptu spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Jaký je rozdíl mezi > a >>? Jeden přepisuje, druhý přidává.",
                "Pro přidání chyb na konec použijte dvě šipky: 2>>",
                "Použijte './buggy.sh 2>> errors.log' a potom 'shellgame submit'.",
            ],
            setup: &[
                file("buggy.sh", r#"
                    #!/bin/bash
                    echo 'This is normal output'
                    echo 'This is an error message' >&2
                "#).mode(0o755),
                file("errors.log", "Old error 1\n"),
            ],
            check: &[
                contains("errors.log", &["Old error 1"]).msg("Původní obsah zmizel (použili jste 2> místo 2>>?). Původní stav vrátí 'shellgame reset'.").missing("Soubor errors.log chybí. Původní stav vrátí 'shellgame reset'."),
                contains("errors.log", &["This is an error message"]).msg("Soubor neobsahuje novou chybu."),
            ],
            solution: r#"
                ./buggy.sh 2>> errors.log
                shellgame submit
            "#,
            success: "Správně! Dvojitá šipka zapisuje na konec, takže předchozí záznamy v logu zůstaly zachovány.",
            ..LEVEL
        },
        Level {
            title: "Všechny výstupy",
            text: r#"
                Někdy chcete zachytit VŠECHNO - normální výstup i chyby do jednoho souboru.
                K tomu slouží `&>`.

                ### Proč je to užitečné
                Při ladění skriptů nebo automatizaci často potřebujete kompletní log
                všeho, co program vypsal - ať už to byla informace nebo chyba.

                ### Úkol
                Spusťte `buggy.sh` a přesměrujte OBOJÍ (stdout i stderr) do `all_output.log`.

                ### Příkazy
                - `./script &> soubor` - přesměruje stdout i stderr

                ### Odevzdání
                Po vytvoření souboru spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Ampersand (&) v tomto kontextu znamená 'obojí' - stdout i stderr.",
                "Kombinace &> je zkratka pro přesměrování obou výstupů.",
                "Použijte './buggy.sh &> all_output.log' a potom 'shellgame submit'.",
            ],
            setup: &[
                clean("all_output.log"),
                file("buggy.sh", r#"
                    #!/bin/bash
                    echo 'This is normal output'
                    echo 'This is an error message' >&2
                "#).mode(0o755),
            ],
            check: &[
                contains("all_output.log", &["This is normal output", "This is an error message"])
                    .msg("Soubor neobsahuje oba typy výstupů. Chybí-li normální výstup, přesměrovali jste jen chyby (2>). Chybí-li chyby, zůstaly na obrazovce, protože samotné > bere jen stdout.")
                    .missing("Soubor all_output.log zatím neexistuje."),
            ],
            solution: r#"
                ./buggy.sh &> all_output.log
                shellgame submit
            "#,
            success: "Správně! Operátor &> poslal do souboru oba proudy najednou.",
            ..LEVEL
        },
        Level {
            title: "Černá díra",
            text: r#"
                `/dev/null` je speciální soubor, který zahodí všechno, co do něj pošlete.
                Je užitečný pro umlčení hlučných příkazů.

                ### Proč je to užitečné
                Některé příkazy vypisují spoustu informací, které nepotřebujete.
                Místo zahlcení obrazovky je můžete "poslat do černé díry".

                ### Úkol
                Spusťte `buggy.sh` a umlčte VŠECHNY výstupy (stdout i stderr) přesměrováním do `/dev/null`.
                Na obrazovce se nesmí objevit nic.

                ### Příkazy
                - `./script &> /dev/null` - zahodí veškerý výstup

                ### Odevzdání
                Po spuštění umlčeného skriptu spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Kam v Linuxu 'vyhodíte' data, která nechcete? Existuje speciální soubor...",
                "Soubor /dev/null je jako černá díra - vše pohltí a nic nevrátí. Přesměrujte do něj oba proudy.",
                "Použijte './buggy.sh &> /dev/null' a potom 'shellgame submit'.",
            ],
            setup: &[
                file("buggy.sh", r#"
                    #!/bin/bash
                    echo 'This is normal output'
                    echo 'This is an error message' >&2
                    # Zjistí, zda jste výstup i chybový výstup poslali do /dev/null.
                    if [ /dev/stdout -ef /dev/null ] && [ /dev/stderr -ef /dev/null ]; then touch "$(dirname "$0")/.silenced"; fi
                "#).mode(0o755),
                clean(".silenced"),
            ],
            check: &[is_file(".silenced").msg("Spusťte './buggy.sh' a přesměrujte stdout i stderr do /dev/null, aby se na obrazovce neobjevilo nic.")],
            solution: r#"
                ./buggy.sh &> /dev/null
                shellgame submit
            "#,
            success: "Správně! Zápis do /dev/null se rovnou zahodí, takže příkaz umlčíte bez zakládání logu.",
            ..LEVEL
        },
        Level {
            title: "Souhrn Sekce 10",
            text: r#"
                ### Výzva: Oddělení streamů

                Ukažte, že umíte uložit stdout a stderr odděleně.

                ### Úkol
                V `level-10/challenge` je skript `mixed.sh`, který vypisuje:
                - normální výstup na stdout
                - chyby na stderr

                Spusťte skript **jednou** a současně uložte:
                1. pouze normální výstup do `output.log`
                2. pouze chyby do `errors.log`

                Odpovězte: kolik řádků má errors.log a kolik output.log?
                Formát: `chyby,výstup` (např. `3,5`)

                ### Připomenutí
                - `>` přesměruje standardní výstup.
                - `2>` přesměruje chybový výstup.

                V této výzvě potřebujete oba proudy zachytit zvlášť.

                ### Odevzdání
                `shellgame submit <chyby>,<výstup>`
            "#,
            hints: &[
                "Chyby se zapisují na stderr (deskriptor 2), standardní výstup na stdout (deskriptor 1). Každý proud může mít vlastní cíl.",
                "Za jeden příkaz lze zapsat přesměrování '>' i '2>'; pořadí zde nevadí, protože oba proudy míří do různých souborů.",
                "Spusťte './mixed.sh > output.log 2> errors.log'. Počty zjistěte pomocí 'wc -l errors.log output.log'.",
            ],
            start: "challenge",
            setup: &[
                clean("challenge"),
                file("challenge/mixed.sh", r#"
                    #!/bin/bash
                    echo "Line 1 - normal output"
                    echo "ERROR: Something went wrong" >&2
                    echo "Line 2 - more output"
                    echo "ERROR: Another problem" >&2
                    echo "Line 3 - final output"
                "#).mode(0o755),
            ],
            answer: parts(",", &[
                int(2).error("Počet chyb není správný. Zkontrolujte, že do errors.log směřuje pouze deskriptor 2."),
                int(3).error("Počet normálních řádků není správný. Zkontrolujte, že do output.log směřuje pouze standardní výstup."),
            ])
                .format("Formát odpovědi: chyby,výstup (např. 3,5)"),
            check: &[
                content_is("challenge/errors.log", r#"
                    ERROR: Something went wrong
                    ERROR: Another problem
                "#)
                    .msg("errors.log neobsahuje přesně pouze chybový výstup skriptu. Buď do něj spadl i standardní výstup (oba proudy míří do jednoho souboru), nebo jste skript spustili víckrát a záznamy se nasčítaly.")
                    .missing("Soubor errors.log zatím neexistuje."),
                content_is("challenge/output.log", r#"
                    Line 1 - normal output
                    Line 2 - more output
                    Line 3 - final output
                "#)
                    .msg("output.log neobsahuje přesně pouze standardní výstup skriptu. Buď v něm skončily i chyby (chybí samostatné přesměrování druhého proudu), nebo jste skript spustili víckrát a záznamy se nasčítaly.")
                    .missing("Soubor output.log zatím neexistuje."),
            ],
            solution: r#"
                ./mixed.sh > output.log 2> errors.log
                shellgame submit "$(wc -l < errors.log),$(wc -l < output.log)"
            "#,
            success: "Perfektní! Dokončili jste Sekci 10. Stdout a stderr jsou pro vás jako otevřená kniha!",
            ..LEVEL
        },
    ],
};
