//! Wildcards: `*`, `?` and `[...]` patterns that the shell expands before the command runs.
//!
//! **Purpose.** One pattern instead of a list of names: the basis of batch work. The shell, not the
//! command, expands a pattern, so `ls PATTERN` is a free preview of what a command would touch.
//!
//! **Learning objectives.** After this section a student can:
//! - select files with `*` (every `.jpg`), `?` (one character: `data?.txt` takes `data1.txt`, not
//!   `data10.txt`) and `[ab]` (one of the listed characters);
//! - use a POSIX class such as `[[:lower:]]` where a range like `[a-z]` may depend on the locale;
//! - preview a pattern with `ls` before `cp` acts on it, to catch one that selects too much;
//! - run a bash-only pattern from fish by wrapping the command, and its `ls` preview, in
//!   `bash -c '...'`;
//! - pick the right kind of pattern for each of three selections and check the resulting counts.
//!
//! **Introduces:** `*`, `?`, `[...]`, POSIX classes (`[[:lower:]]`, `[[:digit:]]`), `bash -c`.\
//! **Relies on:** [copy_move](super::s06_copy_move) (`cp`, and the first `*` met there),
//! [inspect](super::s05_inspect) (`file *`).\
//! **Leaves for later:** not in the course: deleting by pattern (only warned about in
//! [create](super::s04_create)), negated sets `[!...]`, `**`, brace expansion, patterns that match
//! nothing.

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Žolíky (Wildcards)",
    intro: r#"
        V této sekci se naučíte pracovat s více soubory najednou pomocí žolíků (wildcards).
        Žolíky vám umožní definovat vzory názvů souborů.

        ### Kdy potřebujete Bash?
        Hvězdička `*` funguje v Bashi i ve fish. Otazník `?`, množiny `[...]` a třídy znaků
        jako `[[:lower:]]` jsou ale syntaxe Bashe: fish hranaté závorky jako žolík nezná
        a otazník od verze 4 také ne. Levely s nimi i souhrn sekce proto vyžadují Bash.

        Pokud hrajete ve fish, každý z těchto levelů nabídne příkaz ve tvaru
        `bash -c 'příkaz'`. Ten spustí pouze daný příkaz v Bashi a vrátí vás do hry.
        Vnější jednoduché uvozovky zachovají žolíky pro Bash. Stejně si ověříte i výběr
        vzoru: `bash -c 'ls VZOR'`.
        `shellgame submit` pak zadejte jako obvykle ve svém herním shellu.

        ### Přehled žolíků v Bashi
        ```
        *           Jakýkoliv počet znaků (včetně nuly)
        ?           Právě jeden znak
        [...]       Jeden ze znaků v závorkách
        [a-z]       Rozsah znaků (může záviset na locale)
        [[:lower:]] Jeden znak klasifikovaný jako malé písmeno
        ```

        ### Příklady pro Bash
        ```
        *.txt           → všechny .txt soubory
        data?.csv       → data1.csv, data2.csv, ale NE data10.csv
        file_[ab].md    → file_a.md, file_b.md, ale NE file_c.md
        [[:lower:]]*.py → soubory začínající malým písmenem
        ```

        > ⚠️ **Pozor na rozsahy:** `[a-z]` a `[A-Z]` mohou vycházet z pořadí znaků
        > nastaveného locale, například ve starších verzích Bashe (3.2 na macOS).
        > Pro význam „malé písmeno“ proto v levelu o třídách znaků použijete
        > POSIX třídu `[[:lower:]]`; podobně existují `[[:upper:]]` a `[[:digit:]]`.

        ### Jak to funguje?
        ```
        Vy napíšete:     Shell expanduje na:
        cp *.jpg imgs/   cp foto1.jpg foto2.jpg foto3.jpg imgs/
               │                    │
               └── žolík ──────────┘ skutečné soubory
        ```

        ### Co se naučíte
        - Vybírat soubory hvězdičkou (`*`)
        - Přesně jeden znak otazníkem (`?`)
        - Množinu znaků hranatými závorkami (`[abc]`)
        - Rozsahy a locale-odolné třídy znaků (`[a-z]`, `[[:lower:]]`, `[[:digit:]]`)

        ### Pokračování
        Pro zahájení prvního levelu této sekce stiskněte Enter.
    "#,
    levels: &[
        Level {
            title: "Hvězdička *",
            text: r#"
                Hvězdička `*` nahradí JAKOUKOLIV sekvenci znaků (včetně prázdné).
                Je velmi užitečná pro výběr souborů se specifickou příponou.

                Potkali jste ji už dříve (`file *`, `mv *.log logs/`); tady si její pravidla poprvé pojmenujeme přesně.

                ### Proč je to užitečné
                Představte si, že máte 100 fotografií a chcete je všechny zkopírovat.
                Místo `cp foto1.jpg foto2.jpg foto3.jpg ...` stačí `cp *.jpg cíl/`.

                ### Úkol
                Zkopírujte všechny soubory s příponou `.jpg` do adresáře `images`
                (adresář `images` již existuje). Originály zůstanou na svém místě.

                ### Příkazy
                - `cp *.jpg adresář/` - zkopíruje všechny .jpg soubory

                ### Odevzdání
                Po splnění úkolu odevzdejte: `shellgame submit`
            "#,
            hints: &[
                "Hvězdička (*) nahrazuje libovolný počet znaků.",
                "Příkaz 'ls *.jpg' vypíše všechny soubory s příponou .jpg.",
                "Použijte 'cp *.jpg images/' pro zkopírování všech souborů s příponou .jpg do adresáře images.",
            ],
            start: "star",
            setup: &[
                clean("star/images"),
                dir("star/images"),
                file("star/photo1.jpg", ""),
                file("star/photo2.jpg", ""),
                file("star/notes.txt", ""),
            ],
            check: &[
                same("star/photo1.jpg", "star/images/photo1.jpg"),
                same("star/photo2.jpg", "star/images/photo2.jpg"),
                no_file("star/images/notes.txt").msg("V images/ je i notes.txt: vzor vybral víc, než měl. Soubor odstraňte, nebo začněte znovu po 'shellgame reset'."),
            ],
            solution: r#"
                cp *.jpg images/
                shellgame submit
            "#,
            success: "Správně! Jediný vzor s hvězdičkou zastoupil libovolný zbytek názvu, takže jste nemuseli soubory vypisovat po jednom.",
            ..LEVEL
        },
        Level {
            title: "Otazník ?",
            text: r#"
                Otazník `?` nahradí PRÁVĚ JEDEN znak.
                Je užitečný, když chcete být přesnější než s hvězdičkou.

                ### Požadavek: Bash
                Tento level vyžaduje **Bash**. Ve fish použijte variantu s `bash -c` níže.
                Ta spustí pouze daný příkaz v Bashi; odevzdávejte dál ve svém herním shellu.

                ### Rozdíl od hvězdičky
                - `*` = libovolný počet znaků (0 nebo více)
                - `?` = přesně jeden znak

                ### Úkol
                Zkopírujte `data1.txt` a `data2.txt` do adresáře `short_data/`.
                NEKOPÍRUJTE `data10.txt` (má dvouciferné číslo).

                ### Příkazy
                - V Bashi: `cp VZOR short_data/` - VZOR sestavte sami z `data`, žolíku a `.txt`
                - Z fish: `bash -c 'cp VZOR short_data/'` - uvozovky ponechte

                Vzor nejdřív ověřte pomocí `ls VZOR` (z fish `bash -c 'ls VZOR'`):
                vypsat se smějí jen soubory, které chcete zkopírovat.

                ### Odevzdání
                Po splnění úkolu odevzdejte: `shellgame submit`
            "#,
            hints: &[
                "Otazník nahradí právě jeden znak. Kolik znaků je mezi 'data' a '.txt' v data1.txt?",
                "data?.txt zachytí data1.txt a data2.txt, ale ne data10.txt (tam jsou dva znaky).",
                r#"V Bashi spusťte "cp data?.txt short_data/", z fish "bash -c 'cp data?.txt short_data/'"."#,
            ],
            start: "question",
            setup: &[
                clean("question/short_data"),
                dir("question/short_data"),
                file("question/data1.txt", ""),
                file("question/data2.txt", ""),
                file("question/data10.txt", ""),
            ],
            check: &[
                same("question/data1.txt", "question/short_data/data1.txt"),
                same("question/data2.txt", "question/short_data/data2.txt"),
                no_file("question/short_data/data10.txt").msg("V short_data/ je i data10.txt: '?' zastupuje právě jeden znak. Soubor odstraňte, nebo začněte znovu po 'shellgame reset'."),
            ],
            solution: r#"
                bash -c 'cp data?.txt short_data/'
                shellgame submit
            "#,
            success: "Správně! Otazník zastupuje právě jeden znak, proto se dvouciferný název do výběru nedostal.",
            ..LEVEL
        },
        Level {
            title: "Výběr znaků []",
            text: r#"
                Hranaté závorky `[...]` nahradí JEDEN ze znaků uvnitř.
                Například `[abc]` odpovídá znaku 'a', 'b' nebo 'c'.

                ### Požadavek: Bash
                Tento level vyžaduje **Bash**. Ve fish použijte variantu s `bash -c` níže.
                Ta spustí pouze daný příkaz v Bashi; odevzdávejte dál ve svém herním shellu.

                ### Příklady
                - `zprava_[xy].txt` → zprava_x.txt, zprava_y.txt
                - `log[123].txt` → log1.txt, log2.txt, log3.txt

                ### Úkol
                Zkopírujte `file_a.txt` a `file_b.txt` do adresáře `ab_files/`.
                NEKOPÍRUJTE `file_c.txt`.

                ### Příkazy
                - V Bashi: `cp VZOR ab_files/` - VZOR sestavte sami z `file_`, závorek s povolenými znaky a `.txt`
                - Z fish: `bash -c 'cp VZOR ab_files/'` - uvozovky ponechte

                Vzor nejdřív ověřte pomocí `ls VZOR` (z fish `bash -c 'ls VZOR'`):
                vypsat se smějí jen soubory, které chcete zkopírovat.

                ### Odevzdání
                Po splnění úkolu odevzdejte: `shellgame submit`
            "#,
            hints: &[
                "Hranaté závorky definují množinu povolených znaků na dané pozici.",
                "[ab] znamená 'a nebo b', takže file_[ab].txt zachytí file_a.txt a file_b.txt.",
                r#"V Bashi spusťte "cp file_[ab].txt ab_files/", z fish "bash -c 'cp file_[ab].txt ab_files/'"."#,
            ],
            start: "brackets",
            setup: &[
                clean("brackets/ab_files"),
                dir("brackets/ab_files"),
                file("brackets/file_a.txt", ""),
                file("brackets/file_b.txt", ""),
                file("brackets/file_c.txt", ""),
            ],
            check: &[
                same("brackets/file_a.txt", "brackets/ab_files/file_a.txt"),
                same("brackets/file_b.txt", "brackets/ab_files/file_b.txt"),
                no_file("brackets/ab_files/file_c.txt").msg("V ab_files/ je i file_c.txt: závorky mají povolit jen 'a' a 'b'. Soubor odstraňte, nebo začněte znovu po 'shellgame reset'."),
            ],
            solution: r#"
                bash -c 'cp file_[ab].txt ab_files/'
                shellgame submit
            "#,
            success: "Správně! Závorky povolují na jedné pozici jen vyjmenované znaky, takže vzor vybral přesně požadovanou dvojici.",
            ..LEVEL
        },
        Level {
            title: "Třídy znaků [[:lower:]]",
            text: r#"
                Rozsahy jako `[a-z]` mohou záviset na řazení nastaveného jazyka (locale),
                například ve starších verzích Bashe (3.2 na macOS).
                POSIX třída `[[:lower:]]` místo pořadí vybírá jeden znak klasifikovaný
                jako malé písmeno. Pro tento úkol je proto spolehlivější napříč locale.

                Podobně existují `[[:upper:]]` pro velká písmena a `[[:digit:]]` pro číslice.

                ### Požadavek: Bash
                Tento level vyžaduje **Bash**. Ve fish použijte variantu s `bash -c` níže.
                Ta spustí pouze daný příkaz v Bashi; odevzdávejte dál ve svém herním shellu.

                ### Úkol
                Zkopírujte všechny soubory `.txt` začínající malým písmenem do adresáře `lowercase/`.
                NEKOPÍRUJTE soubory začínající velkým písmenem.

                ### Příkazy
                - V Bashi: `cp VZOR lowercase/` - VZOR sestavte sami z třídy znaků, `*` a `.txt`
                - Z fish: `bash -c 'cp VZOR lowercase/'` - uvozovky ponechte

                Vzor nejdřív ověřte pomocí `ls VZOR` (z fish `bash -c 'ls VZOR'`):
                vypsat se smějí jen soubory, které chcete zkopírovat.

                Přípona `.txt` vyloučí cílový adresář `lowercase`, který také začíná malým písmenem.

                ### Odevzdání
                Po splnění úkolu odevzdejte: `shellgame submit`
            "#,
            hints: &[
                "Třída [[:lower:]] vybere na dané pozici právě jedno malé písmeno bez závislosti na pořadí znaků v locale.",
                "Vzor [[:lower:]]*.txt vybere názvy začínající malým písmenem a končící příponou .txt.",
                r#"V Bashi spusťte "cp [[:lower:]]*.txt lowercase/", z fish "bash -c 'cp [[:lower:]]*.txt lowercase/'"."#,
            ],
            start: "ranges",
            setup: &[
                clean("ranges/lowercase"),
                dir("ranges/lowercase"),
                file("ranges/apple.txt", ""),
                file("ranges/Banana.txt", ""),
                file("ranges/cherry.txt", ""),
                file("ranges/Date.txt", ""),
            ],
            check: &[
                same("ranges/apple.txt", "ranges/lowercase/apple.txt"),
                same("ranges/cherry.txt", "ranges/lowercase/cherry.txt"),
                no_file("ranges/lowercase/Banana.txt").msg("V lowercase/ je i soubor začínající velkým písmenem. Odstraňte ho, nebo začněte znovu po 'shellgame reset'."),
                no_file("ranges/lowercase/Date.txt").msg("V lowercase/ je i soubor začínající velkým písmenem. Odstraňte ho, nebo začněte znovu po 'shellgame reset'."),
            ],
            solution: r#"
                bash -c 'cp [[:lower:]]*.txt lowercase/'
                shellgame submit
            "#,
            success: "Správně! Třída znaků vybírá podle klasifikace písmene, takže výsledek nezávisí na řazení znaků v nastaveném locale.",
            ..LEVEL
        },
        Level {
            title: "Souhrn Sekce 7",
            text: r#"
                ### Výzva: Tři druhy žolíků

                ### Úkol
                V aktuálním adresáři použijte pro každý výběr jiný druh vzoru.
                Než začnete kopírovat, předpovězte si odpovídající názvy a vzor ověřte pomocí `ls VZOR`.

                1. Pomocí `*` zkopírujte všechny `.log` soubory do `logs/`.
                2. Pomocí `?` zkopírujte do `short_data/` jen soubory, jejichž název tvoří `data`, jeden znak a `.txt`.
                3. Pomocí množiny znaků `[...]` zkopírujte do `selected_reports/` reporty s písmenem `a` nebo `b`.

                Originály nechte na místě. Nakonec odevzdejte počty souborů v těchto třech cílových adresářích.

                `?` a `[...]` zde používají syntaxi Bashe. Ve fish spouštějte kopírování
                i náhled `ls` přes `bash -c '...'`.

                ### Odevzdání
                `shellgame submit <logy>,<data>,<reporty>`
            "#,
            hints: &[
                "Každý krok má procvičit jiný vzor. Nejdřív si pomocí 'ls' ověřte, že vzor nevybírá žádný z klamných souborů.",
                "Logy potřebují žolík pro libovolný zbytek názvu, krátká data žolík pro právě jeden znak a reporty množinu se dvěma povolenými písmeny; každý sestavený vzor ověřte pomocí 'ls'.",
                r#"Bash: "cp *.log logs/; cp data?.txt short_data/; cp report_[ab].csv selected_reports/". Fish: "bash -c 'cp *.log logs/; cp data?.txt short_data/; cp report_[ab].csv selected_reports/'"."#,
            ],
            start: "challenge",
            setup: &[
                clean("challenge"),
                dir("challenge/logs"),
                dir("challenge/short_data"),
                dir("challenge/selected_reports"),
                file("challenge/app.log", "log1"),
                file("challenge/error.log", "log2"),
                file("challenge/debug.log", "log3"),
                file("challenge/notes.txt", "txt"),
                file("challenge/data1.txt", "short1"),
                file("challenge/data2.txt", "short2"),
                file("challenge/data10.txt", "long"),
                file("challenge/report_a.csv", "a\n"),
                file("challenge/report_b.csv", "b\n"),
                file("challenge/report_c.csv", "c\n"),
                file("challenge/script.sh", "#!/bin/bash\n"),
            ],
            answer: parts(",", &[
                int(3).error("Počet souborů v logs/ není správný. Spočítejte je ve výpisu 'ls logs'."),
                int(2).error("Počet souborů v short_data/ není správný. Spočítejte je ve výpisu 'ls short_data'."),
                int(2).error("Počet souborů v selected_reports/ není správný. Spočítejte je ve výpisu 'ls selected_reports'."),
            ])
                .format("Formát: logy,data,reporty (tři čísla oddělená čárkou)"),
            check: &[
                same("challenge/app.log", "challenge/logs/app.log"),
                same("challenge/error.log", "challenge/logs/error.log"),
                same("challenge/debug.log", "challenge/logs/debug.log"),
                no_file("challenge/logs/notes.txt").msg("V logs/ je i soubor, který není log: vzor pro logy vybírá příliš mnoho. Ověřte ho pomocí 'ls' a začněte znovu po 'shellgame reset'."),
                same("challenge/data1.txt", "challenge/short_data/data1.txt"),
                same("challenge/data2.txt", "challenge/short_data/data2.txt"),
                no_file("challenge/short_data/data10.txt").msg("V short_data/ je i data10.txt: '?' zastupuje právě jeden znak. Soubor odstraňte, nebo začněte znovu po 'shellgame reset'."),
                same("challenge/report_a.csv", "challenge/selected_reports/report_a.csv"),
                same("challenge/report_b.csv", "challenge/selected_reports/report_b.csv"),
                no_file("challenge/selected_reports/report_c.csv").msg("V selected_reports/ je i report_c.csv: množina má povolit jen 'a' a 'b'. Soubor odstraňte, nebo začněte znovu po 'shellgame reset'."),
            ],
            solution: r#"
                bash -c 'cp *.log logs/'
                bash -c 'cp data?.txt short_data/'
                bash -c 'cp report_[ab].csv selected_reports/'
                shellgame submit "$(ls logs | wc -l),$(ls short_data | wc -l),$(ls selected_reports | wc -l)"
            "#,
            success: "Výborně! Dokončili jste Sekci 7: jeden vzor nahradí dlouhý seznam jmen a 'ls' vám předem ukáže, co vybere.",
            ..LEVEL
        },
    ],
};
