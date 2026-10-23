//! Searching: `grep` for text inside files and `find` for the files themselves.
//!
//! **Purpose.** The finale: search instead of browsing. Results are saved and counted with
//! redirection and pipes, patterns need quoting, and the last challenge reuses earlier tools.
//!
//! **Learning objectives.** After this section a student can:
//! - pull the matching lines out of a file with `grep PATTERN file` and save them with `>`;
//! - search a whole tree for a text with `grep -r` and report the path it prints;
//! - ignore case with `grep -i` and count the matching lines with `| wc -l`;
//! - find files by name and type below a directory with `find DIR -type f -name "NAME"`, quoting
//!   the pattern (`"*.py"`) so that `find`, not the shell, evaluates it, and sort the result;
//! - combine `find`, `grep`, `cp` and counting to track down a secret in a larger tree.
//!
//! **Introduces:** `grep`, `grep -r`, `grep -i`, `find` (`-type f`, `-name`, quoted patterns).\
//! **Relies on:** [redirection](super::s09_redirection) (`>`, `|`, `wc -l`, `sort`),
//! [wildcards](super::s07_wildcards) (patterns), [copy_move](super::s06_copy_move) (`cp`).\
//! **Leaves for later:** the last section; not in the course: `find -exec` and `-size`, `xargs`,
//! regular expressions.

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Vyhledávání",
    intro: r#"
        V této závěrečné sekci se naučíte, jak najít text uvnitř souborů a jak najít samotné soubory.

        ### Dva typy hledání
        ```
        grep = Hledání UVNITŘ souborů (obsah)
        find = Hledání SOUBORŮ samotných (podle názvu, typu, velikosti...)
        ```

        ### grep - hledání textu
        ```
        grep "error" log.txt           Najde řádky s "error"
        grep -r "TODO" projekt/        Rekurzivně v celém adresáři
        grep -i "warning" *.log        Ignoruje velikost písmen
        ```

        ### find - hledání souborů
        ```
        find . -type f -name "*.py"          Všechny běžné .py soubory
        find /home -type f -name "config*"   Soubory začínající na "config"
        find . -type d -name "test*"         Pouze adresáře
        ```

        Uvozovky kolem `"*.py"` jsou důležité: zabrání shellu, aby hvězdičku
        rozbalil předem. Doslovný vzor tak dostane příkaz `find`, který ho vyhodnotí
        v každém prohledávaném adresáři. Ve fish navíc vzor bez uvozovek, který
        v aktuálním adresáři nic nenajde, skončí chybou a příkaz se vůbec nespustí.

        ### Proč je to důležité?
        ```
        Ztratili jste soubor?          → find . -type f -name "soubor.txt"
        Hledáte, kde je chyba v kódu?  → grep -r "ERROR" src/
        Kolik poznámek TODO je v kódu? → grep -r "TODO" . | wc -l
        ```

        ### Co se naučíte
        - Hledat text v souborech (`grep`)
        - Rekurzivní hledání (`grep -r`)
        - Hledat soubory podle typu a názvu (`find -type f -name`)
        - Kombinovat nástroje
    "#,
    levels: &[
        Level {
            title: "Hledání v souboru (grep)",
            text: r#"
                ### Cíl
                Najděte řádek obsahující `ACTIVE_PROFILE` v souboru `config.txt` a uložte ho do `profile.txt`.

                ### Příkazy
                - `grep "vzor" soubor` - hledá vzor v souboru
                - `grep "vzor" soubor > výstup` - uloží nalezené řádky do souboru

                ### Úkol
                1. Najděte řádek s `ACTIVE_PROFILE`
                2. Přesměrujte výsledek do `profile.txt`

                ### Odevzdání
                Po vytvoření souboru spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Příkaz 'grep' hledá zadaný vzor v souborech.",
                "Vyhledejte klíč 'ACTIVE_PROFILE' v souboru 'config.txt' a výstup přesměrujte pomocí '>' do profile.txt.",
                "Spusťte 'grep ACTIVE_PROFILE config.txt > profile.txt', výsledek ověřte pomocí 'cat profile.txt' a potom spusťte 'shellgame submit'.",
            ],
            start: "grep",
            setup: &[
                clean("grep/profile.txt"),
                clean("grep/config.txt"),
                file("grep/config.txt", r#"
                    user=admin
                    host=localhost
                    port=8080
                    ACTIVE_PROFILE=production
                    debug=true
                "#),
            ],
            check: &[
                contains("grep/config.txt", &["ACTIVE_PROFILE=production"])
                    .msg("config.txt už neobsahuje hledaný řádek. Přesměrování '>' do souboru, ze kterého grep zároveň čte, ho nejdřív vyprázdní. Obnovte level příkazem 'shellgame reset' a výstup ukládejte do profile.txt.")
                    .missing("Zdrojový soubor config.txt chybí. Obnovte level příkazem 'shellgame reset'."),
                content_is("grep/profile.txt", "ACTIVE_PROFILE=production\n")
                    .msg("profile.txt má obsahovat přesně jeden řádek, ten s ACTIVE_PROFILE, tak jak ho grep vypíše (včetně konce řádku), a nic dalšího.")
                    .missing("Soubor profile.txt zatím neexistuje. Uložte do něj výstup grepu pomocí '>'."),
            ],
            solution: r#"
                grep ACTIVE_PROFILE config.txt > profile.txt
                shellgame submit
            "#,
            success: "Správně! grep vybere jen potřebné řádky a '>' je uloží pro další použití.",
            ..LEVEL
        },
        Level {
            title: "Rekurzivní hledání",
            text: r#"
                ### Cíl
                Pomocí `grep -r` zjistěte, který soubor v adresáři `project` obsahuje text `SECRET_KEY`.

                ### Příkazy
                - `grep -r "vzor" adresář/` - rekurzivní hledání

                ### Úkol
                Najděte soubor obsahující `SECRET_KEY` a odevzdejte jeho relativní cestu.

                ### Odevzdání
                `shellgame submit <cesta>`
            "#,
            hints: &[
                "Přepínač '-r' hledá rekurzivně ve všech podadresářích.",
                r#"Zkuste: grep -r "SECRET_KEY" project/"#,
                "grep -r vypíše cestu před dvojtečkou. Odevzdejte právě tu cestu.",
            ],
            start: "recursive",
            setup: &[
                file("recursive/project/README.md", "# Project\n"),
                file("recursive/project/src/main.py", "print('Hello')\n"),
                file("recursive/project/config/settings.py", r#"
                    SECRET_KEY = 'xyz'
                    DEBUG = True
                "#),
            ],
            answer: suffix("config/settings.py")
                .error("To není správná odpověď. 'grep -r' vypisuje před dvojtečkou celou relativní cestu — odevzdejte ji i s adresáři, ne jen samotný název souboru. Úvodní './' nevadí."),
            solution: r#"shellgame submit "$(grep -r SECRET_KEY project/ | cut -d: -f1)""#,
            success: "Správně! S přepínačem -r prochází grep celý strom adresářů a u každého nálezu uvede cestu.",
            ..LEVEL
        },
        Level {
            title: "Hledání bez ohledu na velikost písmen",
            text: r#"
                ### Cíl
                Najděte všechny řádky obsahující slovo `warning` bez ohledu na velikost písmen a spočítejte je.

                ### Příkazy
                - `grep -i "vzor" soubor` - ignoruje velikost písmen
                - `grep -i "vzor" soubor | wc -l` - spočítá nalezené řádky

                ### Úkol
                Spočítejte, kolik řádků v `messages.log` obsahuje `warning` psané malými, velkými či smíšenými písmeny.

                ### Odevzdání
                `shellgame submit <číslo>`
            "#,
            hints: &[
                "Bez '-i' najdete jen některé varianty. S '-i' najdete warning, WARNING, Warning…",
                r#"Zkuste: grep -i "warning" messages.log a spočítejte vypsané řádky."#,
                "Počítejte každý řádek, který grep -i vypíše — ne jen jednu velikost písmen.",
            ],
            start: "case",
            setup: &[
                file("case/messages.log", r#"
                    2024-01-01 10:00:00 INFO: Server started
                    2024-01-01 10:05:00 WARNING: Memory usage high
                    2024-01-01 10:10:00 ERROR: Connection lost
                    2024-01-01 10:15:00 warning: Disk space low
                    2024-01-01 10:20:00 INFO: User logged in
                    2024-01-01 10:25:00 Warning: CPU temperature elevated
                    2024-01-01 10:30:00 DEBUG: Cache cleared
                    2024-01-01 10:35:00 WARNING: Network latency detected
                    2024-01-01 10:40:00 INFO: Backup completed
                "#),
            ],
            answer: int(4)
                .mistakes(&[
                    ("2", "Našli jste jen 'WARNING'. Použijte -i pro nalezení všech variant (warning, Warning...)."),
                    ("1", "Našli jste jen jednu variantu. Přepínač -i ignoruje velikost písmen."),
                ])
                .error("Počet není správný. Zkontrolujte, že hledáte bez ohledu na velikost písmen."),
            solution: r#"shellgame submit "$(grep -i warning messages.log | wc -l)""#,
            success: "Správně! Přepínač -i je nepostradatelný pro robustní hledání.",
            ..LEVEL
        },
        Level {
            title: "Hledání souborů (find)",
            text: r#"
                ### Cíl
                Pomocí `find` najděte soubor `lost_file.txt` někde uvnitř `messy_dir`.

                ### Příkazy
                - `find adresář -type f -name "název"` - hledá pouze soubory podle názvu

                ### Úkol
                Najděte `lost_file.txt` a odevzdejte celou relativní cestu.

                ### Odevzdání
                `shellgame submit <cesta>`
            "#,
            hints: &[
                "Příkaz 'find' prohledá všechny podadresáře automaticky.",
                r#"Pro hledání souborů použijte: find kde_hledat -type f -name "co_hledat""#,
                r#"Použijte: find messy_dir -type f -name "lost_file.txt""#,
            ],
            start: "find",
            setup: &[
                file("find/messy_dir/a/b/c/d/lost_file.txt", ""),
                file("find/messy_dir/other.txt", ""),
                file("find/messy_dir/a/junk.txt", ""),
            ],
            answer: suffix("a/b/c/d/lost_file.txt")
                .mistakes(&[
                    ("messy_dir/a/b/c/d", "To je cesta k adresáři. Odevzdejte celou cestu i s názvem souboru na konci."),
                    ("./messy_dir/a/b/c/d", "To je cesta k adresáři. Odevzdejte celou cestu i s názvem souboru na konci."),
                ])
                .error("To není správná cesta. Odevzdejte ji tak, jak ji vypsal find — včetně všech adresářů, ne jen název souboru. Úvodní './' nevadí."),
            solution: r#"shellgame submit "$(find messy_dir -type f -name 'lost_file.txt')""#,
            success: "Správně! find hledá podle vlastností souboru, takže si poradí i s neznámým umístěním.",
            ..LEVEL
        },
        Level {
            title: "Hledání podle přípony",
            text: r#"
                ### Cíl
                Najděte všechny běžné `.py` soubory v adresáři `src_code` a uložte jejich cesty do `python_files.txt`.
                Výstup seřaďte, aby měl vždy stejné pořadí.

                **Důležité:** Vzor musí být v uvozovkách: `"*.py"`. V aktuálním adresáři leží `setup.py`,
                takže bez uvozovek by shell `*.py` ještě před spuštěním nahradil jménem `setup.py`
                a `find` by hledal jen soubory s tímto jménem. S uvozovkami dostane vzor doslova `find`.

                ### Stavební prvky
                - `find adresář -type f -name "*.py"` - hledá pouze soubory s příponou `.py`
                - `sort` - seřadí řádky
                - `>` - uloží výsledek

                ### Úkol
                Vytvořte `python_files.txt` se seřazenými cestami nalezených souborů, jednou na řádek,
                tak jak je vypíše `find` spuštěný na adresář `src_code` (cesty tedy začínají `src_code/`).

                ### Odevzdání
                Po vytvoření souboru spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Omezte find na běžné soubory pomocí '-type f'; adresář s příponou .py se počítat nemá.",
                "Vzor dejte do uvozovek a výstup pošlete přes 'sort' před uložením pomocí '>'.",
                r#"Použijte: find src_code -type f -name "*.py" | sort > python_files.txt"#,
            ],
            start: "extension",
            setup: &[
                clean("extension/python_files.txt"),
                file("extension/setup.py", ""),
                dir("extension/src_code/archive.py"),
                file("extension/src_code/main.py", ""),
                file("extension/src_code/utils.py", ""),
                file("extension/src_code/README.txt", ""),
                file("extension/src_code/data.csv", ""),
            ],
            check: &[
                content_is("extension/python_files.txt", r#"
                    src_code/main.py
                    src_code/utils.py
                "#)
                    .msg("Obsah python_files.txt nesedí. Má obsahovat jen cesty k běžným .py souborům v src_code, každou na vlastním řádku, začínající 'src_code/' a seřazené. Bez -type f se přidá adresář archive.py; bez uvozovek kolem vzoru shell vzor nahradí jménem setup.py; 'find .' místo 'find src_code' přidá './' a setup.py; a nezapomeňte na sort.")
                    .missing("Soubor python_files.txt zatím neexistuje. Výstup příkazu uložte pomocí '>'."),
            ],
            solution: r#"
                find src_code -type f -name "*.py" | sort > python_files.txt
                shellgame submit
            "#,
            success: "Správně! Spojení find, sort a '>' dává výsledek, který vyjde pokaždé stejně.",
            ..LEVEL
        },
        Level {
            title: "Finální výzva",
            text: r#"
                ### 🏆 Finální výzva: Terminálový ninja

                Gratulujeme! Dostali jste se na konec ShellGame.
                Tato výzva kombinuje vyhledávání, filtrování, kopírování a počítání z posledních sekcí.

                ### Úkol
                V `level-11/final`:

                1. Najděte **všechny soubory** s příponou `.sh` rekurzivně pomocí `find`.
                2. Jeden z nich obsahuje tajný kód — najděte ho pomocí `grep`.
                3. Skript s kódem zkopírujte do `found/secret.txt`.
                4. Spočítejte soubory `.sh`. Kopie má příponu `.txt`, takže se do počtu nezapočítá.

                ### Struktura odpovědi
                `<počet_skriptů>,<tajný_kód>`

                Kód je jen hodnota za rovnítkem, ne celý řádek.

                ### Odevzdání
                `shellgame submit <počet>,<kód>`
            "#,
            hints: &[
                "Pro hledání skriptů spojte 'find' s podmínkami '-type f' a '-name'; obsah pak prohledejte pomocí 'grep'.",
                r#"Skripty vypíše find . -type f -name "*.sh". V jejich obsahu hledejte řádek se slovem SECRET, například pomocí grep -r SECRET ."#,
                r#"Skript s kódem zkopírujte pomocí cp do found/secret.txt a počet zjistěte přes find . -type f -name "*.sh" | wc -l"#,
                "Odevzdejte počet a samotný kód, tedy hodnotu za '='.",
            ],
            start: "final",
            setup: &[
                clean("final"),
                dir("final/found"),
                file("final/run.sh", r#"
                    #!/bin/bash
                    echo 'Running...'
                "#),
                file("final/scripts/build.sh", r#"
                    #!/bin/bash
                    make all
                "#),
                file("final/scripts/test.sh", r#"
                    #!/bin/bash
                    pytest
                "#),
                file("final/hidden/secret.sh", r#"
                    #!/bin/bash
                    # SECRET_CODE=NINJA2024
                    echo 'You found me!'
                "#),
                file("final/readme.txt", "Look for shell scripts!\n"),
                file("final/data.csv", "col1,col2\n"),
            ],
            answer: parts(",", &[
                int(4).error(r#"Počet .sh souborů není správný. Použijte 'find . -type f -name "*.sh"'."#),
                exact("NINJA2024")
                    .nocase()
                    .mistakes(&[
                        ("SECRET_CODE=NINJA2024", "Odevzdejte jen hodnotu za '=', ne celé přiřazení."),
                        ("# SECRET_CODE=NINJA2024", "Odevzdejte jen hodnotu za '=', ne celý řádek."),
                    ])
                    .error("Tajný kód nesedí. Najděte ho pomocí 'grep' v obsahu skriptů a odevzdejte jen hodnotu za '=', ne celý řádek."),
            ])
                .format("Formát: počet,kód (např. 5,ABC123)"),
            check: &[
                same("final/hidden/secret.sh", "final/found/secret.txt")
                    .msg("found/secret.txt se neshoduje se skriptem, který obsahuje tajný kód. Zkopírujte ho pomocí 'cp'.")
                    .missing("V adresáři found/ chybí secret.txt: zkopírujte do něj skript s tajným kódem."),
            ],
            solution: r#"
                cp hidden/secret.sh found/secret.txt
                shellgame submit "$(find . -type f -name '*.sh' | wc -l),$(grep -rh SECRET_CODE --include='*.sh' . | cut -d= -f2)"
            "#,
            success: r#"
                Gratulujeme, dokončili jste ShellGame!

                Zvládáte navigaci v souborovém systému, práci se soubory a adresáři, oprávnění,
                přesměrování, žolíky i vyhledávání. Jste připraveni na další dobrodružství v Linuxu!
            "#,
            ..LEVEL
        },
    ],
};
