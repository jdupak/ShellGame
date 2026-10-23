//! Copying and moving: `cp`, `cp -r` and `mv`, a copy versus a move, and an alias for `shellgame`.
//!
//! **Purpose.** `cp` and `mv` can overwrite data without a word, so they come with the habit of
//! checking the target. The section also hands over a first convenience: an alias for `shellgame`.
//!
//! **Learning objectives.** After this section a student can:
//! - copy a file (`cp source target`, the original stays) and a directory (`cp -r`);
//! - rename (`mv old new`) and move into a directory (`mv file dir/`), knowing the form of the
//!   target decides which happens; move a directory with `mv`, which needs no `-r`;
//! - move a set of files with one pattern (`mv *.log logs/`), previewed with `ls`;
//! - choose copy or move per file in a small tidy-up task;
//! - define an alias (`alias hra='shellgame'`; fish: `alias hra shellgame`), check it with
//!   `type hra`, and ask `type` what a name is to the shell (`shellgame` itself is a function).
//!
//! **Introduces:** `cp`, `cp -r`, `mv`, `alias`, `type`, and a first `*`.\
//! **Relies on:** [navigation](super::s01_navigation) (`ls`, relative paths); `-r` was met in
//! `rm -r` ([create](super::s04_create)).\
//! **Leaves for later:** the rest of the wildcard family ([wildcards](super::s07_wildcards),
//! announced in the level text); `-i`, which the intro only names.

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Kopírování a přesouvání",
    intro: r#"
        V této sekci se naučíte manipulovat se soubory a adresáři - vytvářet jejich kopie a přesouvat je na jiná místa.

        ## Proč je to důležité?
        „Před úpravou konfigurace si udělej zálohu.“ — rada, kterou uslyšíte všude,
        a která se v terminálu vejde do jednoho příkazu.

        `cp` a `mv` jsou zároveň první příkazy, které doopravdy přepisují vaše data.
        Oba umí bez varování přepsat existující cíl — proto se už teď vyplatí
        zvyknout si nejdřív zkontrolovat, co na cílovém místě leží.

        ## `cp` vs `mv` - jaký je rozdíl?
        ```
        cp (copy)                      mv (move)
        ─────────────────────────────────────────────
        Původní soubor ZŮSTANE         Původní soubor ZMIZÍ
        Vytvoří duplikát               Přesune/přejmenuje

        cp a.txt b.txt                 mv a.txt b.txt
        ├── a.txt  ← zůstává           └── b.txt  ← jediný soubor
        └── b.txt  ← nový
        ```

        ## Pozor na existující cíl
        `cp` i `mv` mohou bez dalšího dotazu přepsat existující cílový soubor.
        Předem zkontrolujte cíl pomocí `ls`, případně použijte `cp -i` nebo `mv -i`.

        ## Přejmenování = přesun na stejném místě
        ```
        mv stary_nazev.txt novy_nazev.txt
             │                  │
             └──────────────────┘
                 "přesun" = přejmenování
        ```

        ## Co se naučíte:
        - Kopírovat soubory (`cp`)
        - Kopírovat adresáře (`cp -r`)
        - Přejmenovávat soubory a adresáře (`mv`)
        - Přesouvat soubory a adresáře (`mv`)
        - Zkrátit si dlouhý příkaz aliasem (`alias`) a ověřit ho (`type`)

        ### Tip: `-r` znamená "rekurzivně"
        `cp` a `rm` potřebují u adresářů `-r`, aby prošly celý strom včetně obsahu. `mv` ho nepotřebuje.
    "#,
    levels: &[
        Level {
            title: "Kopírování souboru",
            text: r#"
                Příkaz `cp` (copy) vytvoří kopii souboru.

                ### Proč zálohovat?
                Před úpravou důležitého souboru je dobré si udělat zálohu.
                Když něco pokazíte, máte se kam vrátit!

                V praxi uvidíte:
                - `cp config.yaml config.yaml.bak` (před úpravou konfigurace)
                - `cp report.docx report_v1.docx` (verzování dokumentů)

                ### Úkol
                1. Začínáte v `level-6/copying`. Vytvořte zálohu souboru `dulezite.txt`. Kopii pojmenujte `dulezite.bak`.
                2. Ověřte výsledek pomocí `ls`.

                ## Příkazy:
                - `cp <zdroj> <cíl>`: Zkopíruje zdroj do cíle

                ## Odevzdání:
                Po vytvoření zálohy spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Příkaz cp má dva argumenty: odkud a kam kopírujete.",
                "Syntaxe je: cp zdrojový_soubor cílový_soubor",
                "Použijte 'cp dulezite.txt dulezite.bak'.",
            ],
            start: "copying",
            setup: &[clean("copying/dulezite.bak"), file("copying/dulezite.txt", "Very important data.")],
            check: &[same("copying/dulezite.txt", "copying/dulezite.bak")],
            solution: r#"
                cp dulezite.txt dulezite.bak
                shellgame submit
            "#,
            success: "Správně! Záloha je samostatná kopie pod novým jménem, originál zůstává nedotčený.",
            ..LEVEL
        },
        Level {
            title: "Kopírování adresáře",
            text: r#"
                Bez přepínače `-r` (recursive) příkaz `cp` adresář vůbec nezkopíruje, jen ohlásí, že ho vynechal.
                S přepínačem zkopíruje adresář i s celým obsahem.

                ## Úkol:
                Zkopírujte celý adresář `projekt` do nového adresáře `projekt_zaloha`.

                ## Příkazy:
                - `cp -r <zdroj> <cíl>`: Zkopíruje adresář

                ## Odevzdání:
                Po zkopírování adresáře spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Pro kopírování celého adresáře včetně obsahu je nutné použít rekurzivní přepínač '-r'.",
                "Tvar je 'cp -r <zdroj> <cíl>': přepínač stojí před oběma argumenty, zdrojový adresář je první a název nové kopie druhý.",
                "Spusťte 'cp -r projekt projekt_zaloha'.",
            ],
            start: "copying",
            setup: &[clean("copying/projekt_zaloha"), file("copying/projekt/main.py", "print('hello')")],
            check: &[same("copying/projekt", "copying/projekt_zaloha")],
            solution: r#"
                cp -r projekt projekt_zaloha
                shellgame submit
            "#,
            success: "Správně! Přepínač -r říká příkazu cp, že má sestoupit do celého stromu adresáře.",
            ..LEVEL
        },
        Level {
            title: "Přejmenování souboru",
            text: r#"
                Příkaz `mv` (move) se používá k přesouvání,
                ale pokud přesouváte soubor ve stejném adresáři na nové jméno, jde o přejmenování.

                ## Úkol:
                Soubor `spatne_jmeno.txt` má překlep. Přejmenujte ho na `spravne_jmeno.txt`.

                ## Příkazy:
                - `mv <staré_jméno> <nové_jméno>`: Přejmenuje soubor

                ## Odevzdání:
                Po přejmenování spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Příkaz 'mv' slouží nejen k přesunu, ale i k přejmenování souboru: 'mv staré nové'.",
                "Oba argumenty jsou jména souborů v aktuálním adresáři: první je to současné, druhé to požadované.",
                "Spusťte 'mv spatne_jmeno.txt spravne_jmeno.txt'.",
            ],
            start: "moving",
            setup: &[clean("moving/spravne_jmeno.txt"), file("moving/spatne_jmeno.txt", "content")],
            check: &[moved("moving/spatne_jmeno.txt", "moving/spravne_jmeno.txt")],
            solution: r#"
                mv spatne_jmeno.txt spravne_jmeno.txt
                shellgame submit
            "#,
            success: "Správně! Přejmenování je jen přesun na novou cestu ve stejném adresáři.",
            ..LEVEL
        },
        Level {
            title: "Přesun souboru",
            text: r#"
                Pokud jako cíl příkazu `mv` uvedete existující adresář,
                soubor se do něj přesune (a zachová si své jméno, pokud neuvedete jiné).

                ## Úkol:
                Přesuňte soubor `report.pdf` do adresáře `dokumenty`.

                ## Příkazy:
                - `mv <soubor> <adresář>/`: Přesune soubor do adresáře

                ## Proč to lomítko na konci?
                Pokud cílový adresář **neexistuje**, `mv` soubor tiše přejmenuje na zadané jméno.
                S lomítkem `mv` místo toho ohlásí chybu, takže se o překlepu v názvu adresáře dozvíte hned.

                ## Odevzdání:
                Po přesunu souboru spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "O tom, co 'mv' udělá, rozhoduje podoba cíle: soubor znamená přejmenování, existující adresář přesun.",
                "Tvar je 'mv <soubor> <cílový_adresář>/': první argument je přesouvaný soubor, druhý cílový adresář zakončený lomítkem.",
                "Spusťte 'mv report.pdf dokumenty/'. Lomítko na konci značí adresář.",
            ],
            start: "moving",
            setup: &[clean("moving/dokumenty/report.pdf"), dir("moving/dokumenty"), file("moving/report.pdf", "report data")],
            check: &[
                moved("moving/report.pdf", "moving/dokumenty/report.pdf")
                    .msg("report.pdf je stále na původním místě. Kopie originál ponechává, přesun ho odtud odebere.")
                    .missing("report.pdf z původního místa zmizel, ale v cílovém adresáři není. Když cíl neexistuje, 'mv' soubor jen přejmenuje - zkontrolujte 'ls'."),
            ],
            solution: r#"
                mv report.pdf dokumenty/
                shellgame submit
            "#,
            success: "Správně! Podoba cíle rozhoduje, jestli mv přejmenovává, nebo přesouvá.",
            ..LEVEL
        },
        Level {
            title: "Přesun adresáře",
            text: r#"
                Příkaz `mv` přesouvá celé adresáře včetně jejich obsahu; přepínač `-r` nepotřebuje.

                ## Úkol:
                Přesuňte celý adresář `projekt` do existujícího adresáře `archiv`.

                ## Příkazy:
                - `mv <adresář> <cílový_adresář>/`

                ## Odevzdání:
                Až bude `projekt` uvnitř `archiv`, spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Adresář se přesouvá stejným příkazem jako soubor: 'mv' vezme celý podstrom a přepínač '-r' nepotřebuje.",
                "Tvar je 'mv <adresář> <cílový_adresář>/': první argument je přesouvaný adresář, druhý už existující rodič, do kterého má vklouznout.",
                "Spusťte 'mv projekt archiv/' a ověřte výsledek pomocí 'ls archiv/'.",
            ],
            start: "moving",
            setup: &[
                clean("moving/projekt"),
                clean("moving/archiv/projekt"),
                dir("moving/archiv"),
                file("moving/projekt/data/file.txt", "content"),
            ],
            check: &[
                moved("moving/projekt", "moving/archiv/projekt")
                    .msg("Adresář projekt je stále na původním místě - zatím se nic nepřesunulo.")
                    .missing("Projekt z původního místa zmizel, ale v archivu není. Když cílový adresář neexistuje, 'mv' zdroj jen přejmenuje - porovnejte 'ls' a 'ls archiv/'."),
                content_is("moving/archiv/projekt/data/file.txt", "content").msg("Přesunutý adresář nemá původní obsah."),
            ],
            solution: r#"
                mv projekt archiv/
                shellgame submit
            "#,
            success: "Správně! Přesun adresáře vezme celý jeho obsah s sebou jedním krokem.",
            ..LEVEL
        },
        Level {
            title: "Úklid logů",
            text: r#"
                V adresáři je nepořádek. Všechny soubory s příponou `.log` by měly být v adresáři `logs`.

                ## Úkol:
                Přesuňte všechny `.log` soubory (`app.log`, `error.log`) do adresáře `logs`.
                Zvládnete to jedním příkazem, když místo výčtu jmen zadáte vzor se žolíkem `*`.

                ## Příkazy:
                - `mv VZOR logs/`: za `VZOR` dosaďte vzor, který vybere právě `.log` soubory

                Vzor si nejdřív ověřte pomocí `ls VZOR` - uvidíte přesně ty soubory, které by `mv` přesunul.
                Znak `*` je žolík (zástupný znak); celou rodinu žolíků probereme v sekci „Žolíky (Wildcards)“.

                ## Odevzdání:
                Po přesunu zkontrolujte výsledek pomocí `ls logs` a spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Žolík '*' nahradí libovolný počet znaků, takže vzor '*.log' vybere všechny soubory s touto příponou.",
                "Příkaz 'mv' dokáže přesunout více souborů najednou do cílového adresáře: 'mv <soubory> <cíl>/'.",
                "Spusťte 'mv *.log logs/' a potom 'shellgame submit'.",
            ],
            start: "organize",
            setup: &[
                clean("organize/logs"),
                dir("organize/logs"),
                file("organize/app.log", "log1"),
                file("organize/error.log", "log2"),
                file("organize/other.txt", "keep me"),
            ],
            check: &[
                moved("organize/app.log", "organize/logs/app.log"),
                moved("organize/error.log", "organize/logs/error.log"),
                is_file("organize/other.txt").msg("other.txt není log, a přesto zmizel. Vzor vybral víc souborů, než měl; ověřte ho pomocí 'ls' a začněte znovu po 'shellgame reset'."),
            ],
            solution: r#"
                mv *.log logs/
                shellgame submit
            "#,
            success: "Správně! Vzor se žolíkem rozbalí shell na seznam jmen ještě před spuštěním příkazu.",
            ..LEVEL
        },
        Level {
            title: "Rychlejší práce: alias",
            text: r#"
                ### Cíl
                Zkraťte si časté psaní příkazu `shellgame` pomocí aliasu `hra`.

                ### Nastavení
                Použijte příkaz pro svůj shell:
                - **Bash:** `alias hra='shellgame'`
                - **Fish:** `alias hra shellgame`

                Alias ověřte příkazem `type hra`. Příkaz `type` prozradí, čím je dané jméno pro shell:
                například alias, vestavěný příkaz nebo program uložený na disku.

                Alias platí jen v aktuálním herním shellu. Po jeho ukončení zmizí,
                a když hru spustíte znovu, musíte ho zadat znovu.

                ### Úkol
                Zjistěte příkazem `type shellgame`, čím je pro váš herní shell samotné `shellgame`:
                stačí slovo, které na prvním řádku výpisu následuje po „is a“ (v češtině po „je“).

                ### Odevzdání
                Odpověď odevzdejte přes nový alias:
                `hra submit <slovo>`

                V dalších levelech můžete místo dlouhého `shellgame` používat kratší `hra`.
            "#,
            hints: &[
                "Alias dává dlouhému příkazu kratší jméno. Vyberte syntaxi pro svůj shell a ověřte ji pomocí 'type hra'.",
                "Spusťte 'type shellgame'. První řádek výpisu má tvar 'shellgame is a <slovo>' (česky 'shellgame je <slovo>').",
                "Odevzdejte to slovo přes alias: 'hra submit <slovo>'.",
            ],
            answer: one_of(&["function", "funkce", "funkci", "a function"])
                .nocase()
                .error("Nesprávně. Odevzdejte slovo, které 'type shellgame' vypíše po 'is a' (česky po 'je').")
                .format("Odevzdejte slovo z výpisu 'type shellgame' přes alias: hra submit <slovo>"),
            solution: r#"
                test -n "$BASH_VERSION" && shopt -s expand_aliases
                alias hra='shellgame'
                type hra
                hra submit "$(type shellgame | head -n 1 | cut -d' ' -f4)"
            "#,
            success: "Výborně! Příkaz type ukáže, co se skrývá za jménem příkazu. Alias hra vám teď ušetří psaní.",
            ..LEVEL
        },
        Level {
            title: "Výzva: Organizátor souborů",
            text: r#"
                Ukažte, že ovládáte kopírování a přesouvání!

                ### Úkol
                V aktuálním adresáři:

                1. **Zkopírujte** `original.txt` do adresáře `backup/` (zachovejte originál)
                2. **Přejmenujte** `temp_data.csv` na `data.csv`
                3. **Přesuňte** `misplaced.log` do adresáře `logs/`

                ### Přehled příkazů
                ```
                cp zdroj cíl       → Kopíruje soubor
                cp zdroj dir/      → Kopíruje do adresáře
                cp -r dir1 dir2    → Kopíruje celý adresář
                mv zdroj cíl       → Přesune/přejmenuje
                mv soubor dir/     → Přesune do adresáře
                ```

                ### Odevzdání
                Po splnění všech bodů spusťte `shellgame submit`.
            "#,
            hints: &[
                "Rozhodněte u každého souboru, zda musí původní cesta zůstat zachovaná. Podle toho zvolte 'cp' nebo 'mv'.",
                "Přejmenování i přesun používají 'mv'; rozdíl určuje podoba cíle. Kopie do adresáře zachová původní soubor.",
                "Použijte 'cp original.txt backup/', 'mv temp_data.csv data.csv' a 'mv misplaced.log logs/'.",
            ],
            start: "final_test",
            setup: &[
                clean("final_test"),
                dir("final_test/backup"),
                dir("final_test/logs"),
                file("final_test/original.txt", "Important data - do not delete!\n"),
                file("final_test/temp_data.csv", r#"
                    col1,col2
                    1,2
                "#),
                file("final_test/misplaced.log", "Log entry\n"),
            ],
            check: &[
                is_file("final_test/original.txt").msg("Smazali jste original.txt! Měli jste ho zkopírovat, ne přesunout."),
                same("final_test/original.txt", "final_test/backup/original.txt")
                    .msg("Kopie v backup/ neodpovídá originálu.")
                    .missing("Chybí kopie original.txt v adresáři backup/."),
                moved("final_test/temp_data.csv", "final_test/data.csv")
                    .msg("temp_data.csv stále existuje. Přejmenujte ho na data.csv pomocí 'mv'.")
                    .missing("temp_data.csv zmizel, ale data.csv neexistuje. Pomocí 'ls' zjistěte, pod jakým jménem soubor skončil."),
                moved("final_test/misplaced.log", "final_test/logs/misplaced.log")
                    .msg("misplaced.log je stále v aktuálním adresáři. Přesuňte ho do logs/ pomocí 'mv'.")
                    .missing("Chybí misplaced.log v adresáři logs/."),
            ],
            solution: r#"
                cp original.txt backup/
                mv temp_data.csv data.csv
                mv misplaced.log logs/
                shellgame submit
            "#,
            success: "Výborně! Dokončili jste Sekci 6 — kopírování, přesun i přejmenování máte pod kontrolou.",
            ..LEVEL
        },
    ],
};
