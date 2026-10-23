//! Creating and deleting: `mkdir`, `touch`, `rm`, `rmdir` and `rm -r`, and why deleting is final.
//!
//! **Purpose.** Until now the student only walked through a prepared world. Here they shape it, and
//! as the shell has no trash can, care is part of the lesson: levels judge the files left behind.
//!
//! **Learning objectives.** After this section a student can:
//! - create a directory with `mkdir` and a nested path with `mkdir -p`, and say why plain `mkdir`
//!   fails when a parent is missing;
//! - create empty files with `touch`, several paths at once with one `mkdir -p` or `touch`;
//! - delete a file with `rm`, an empty directory with `rmdir`, a directory with content with
//!   `rm -r`, knowing there is no undo;
//! - delete several named files with one `rm` while the file that must stay stays.
//!
//! **Introduces:** `mkdir`, `mkdir -p`, `rm`, `rmdir`, `rm -r`.\
//! **Relies on:** [navigation](super::s01_navigation), [files](super::s02_files) (`touch`).\
//! **Leaves for later:** copying and moving ([copy_move](super::s06_copy_move)), wildcards (only
//! warned about, as in `rm *.log`: [wildcards](super::s07_wildcards)), writing file content
//! ([redirection](super::s09_redirection)).

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Vytváření a mazání",
    intro: r#"
        V této sekci se naučíte, jak vytvářet nové soubory a adresáře a jak po sobě uklidit.

        ### Proč je to důležité?
        Zatím jste se pohybovali po struktuře, kterou někdo připravil za vás.
        Každý nový projekt ale začíná prázdným adresářem, který si musíte založit sami —
        a na serveru, kde není žádné grafické rozhraní, je příkazová řádka jediná možnost.
        S úklidem přichází i odpovědnost: mazání v terminálu je okamžité a konečné.

        ### Životní cyklus položek
        Každá položka v systému prochází třemi fázemi:
        - **Vytvoření**: `touch soubor.txt` / `mkdir projekt/`
        - **Práce**: úprava obsahu, čtení, navigace
        - **Úklid**: `rm soubor.txt` / `rm -r projekt/`

        ### Vytváření zanořených struktur (`mkdir -p`)
        Příkaz `mkdir` běžně vyžaduje, aby rodičovský adresář už existoval:

        ```
        $ mkdir a/b/c
        mkdir: cannot create directory 'a/b/c': No such file or directory
        ```
        ❌ **Chyba**: nadřazené složky `a` ani `b` ještě neexistují.

        S přepínačem **`-p`** (parents) vytvoří `mkdir` celou cestu najednou:

        ```
        $ mkdir -p a/b/c
        ```
        ✓ **Výsledek**: automaticky se vytvoří celá větev `a/` → `a/b/` → `a/b/c/`.

        ### Co se naučíte
        - Vytvářet prázdné soubory (`touch`)
        - Vytvářet adresáře (`mkdir`)
        - Vytvářet zanořené struktury (`mkdir -p`)
        - Mazat soubory (`rm`)
        - Mazat adresáře (`rmdir`, `rm -r`)

        ### ⚠️ Pozor na `rm`!
        Příkaz `rm` nepřesouvá soubory do koše — maže je rovnou a nevratně.

        ### Pokračování
        Pro zahájení prvního levelu této sekce stiskněte Enter.
    "#,
    levels: &[
        Level {
            title: "Vytvoření souboru",
            text: r#"
                ### Cíl
                Vytvořte nový prázdný soubor.

                ### Příkazy
                - `touch <název_souboru>` (vytvoří prázdný soubor nebo aktualizuje časové značky)

                ### Úkol
                Vytvořte prázdný soubor s názvem `novy_soubor.txt` v aktuálním adresáři.

                ### Odevzdání
                Až bude soubor existovat a bude zcela prázdný, spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Příkaz 'touch' vytvoří prázdný soubor. Jaký název má mít?",
                "Syntaxe je jednoduchá: touch název_souboru",
                "Použijte 'touch novy_soubor.txt', ověřte výsledek pomocí 'ls -l' a spusťte 'shellgame submit'.",
            ],
            start: "creation",
            setup: &[clean("creation/novy_soubor.txt"), dir("creation")],
            check: &[
                is_file("creation/novy_soubor.txt")
                    .msg("Soubor novy_soubor.txt zatím neexistuje. Vytvořte ho v adresáři, kde level začíná, a ověřte pomocí 'ls'."),
                content_is("creation/novy_soubor.txt", "").msg("Soubor novy_soubor.txt existuje, ale není prázdný."),
            ],
            solution: r#"
                touch novy_soubor.txt
                shellgame submit
            "#,
            success: "Správně! Soubor může vzniknout úplně prázdný a obsah do něj doplníte až později.",
            ..LEVEL
        },
        Level {
            title: "Vytvoření adresáře",
            text: r#"
                ### Cíl
                Vytvořte nový adresář.

                ### Příkazy
                - `mkdir <název_adresáře>` (make directory)

                ### Úkol
                Vytvořte adresář s názvem `data` v aktuálním adresáři.

                ### Odevzdání
                Až bude adresář existovat, spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Příkaz 'mkdir' slouží k vytvoření nového adresáře.",
                "Spusťte 'mkdir data' pro vytvoření adresáře data.",
                "Ověřte pomocí 'ls -l' (řádek adresáře začíná 'd') a spusťte 'shellgame submit'.",
            ],
            start: "creation",
            setup: &[clean("creation/data"), dir("creation")],
            check: &[
                is_dir("creation/data")
                    .msg("Požadovaný adresář zatím neexistuje. Ověřte pomocí 'ls -l', že jste ho založili v aktuálním adresáři a bez překlepu v názvu."),
            ],
            solution: r#"
                mkdir data
                shellgame submit
            "#,
            success: "Správně! Nový adresář vznikne jediným příkazem a hned do něj můžete vstoupit.",
            ..LEVEL
        },
        Level {
            title: "Vytváření zanořených adresářů",
            text: r#"
                ### Cíl
                Vytvořte zanořenou strukturu adresářů jedním příkazem.

                ### Vytváření cest s přepínačem `-p`
                Při vytváření celé cesty (např. `projekt/src/tests`) příkaz `mkdir` bez přepínače
                selže, pokud nadřazené adresáře neexistují.
                Přepínač `-p` (parents) vytvoří celou cestu včetně chybějících rodičovských adresářů.

                ### Příkazy
                - `mkdir -p <cesta>` (vytvoří adresář včetně všech chybějících nadřazených složek)

                ### Úkol
                Vytvořte strukturu adresářů `projekt/src/tests`.

                ### Odevzdání
                Až bude celá struktura vytvořená, spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Co se stane, když zkusíte 'mkdir projekt/src/tests' bez přepínače -p?",
                "Přepínač -p (parents) vytvoří i všechny nadřazené adresáře, které chybí.",
                "Použijte 'mkdir -p projekt/src/tests'.",
            ],
            start: "nested",
            setup: &[clean("nested/projekt"), dir("nested")],
            check: &[
                is_dir("nested/projekt/src/tests")
                    .msg("Struktura projekt/src/tests zatím není celá. Zkontrolujte ji pomocí 'ls projekt' a 'ls projekt/src': bez přepínače -p vytvoří mkdir jen poslední článek cesty, a to jen když jeho rodič už existuje. S -p vzniknou chybějící rodičovské adresáře zároveň s ním."),
            ],
            solution: r#"
                mkdir -p projekt/src/tests
                shellgame submit
            "#,
            success: "Správně! Přepínač -p vytvoří celou cestu naráz, takže rodiče nemusíte zakládat po jednom.",
            ..LEVEL
        },
        Level {
            title: "Mazání souborů",
            text: r#"
                ### Cíl
                Smažte nepotřebný soubor.

                ### Příkazy
                - `rm <soubor>` (remove — smaže zadaný soubor)

                ### ⚠️ Důležité varování
                Příkaz `rm` nepřesouvá soubory do koše jako grafické prostředí — maže je okamžitě a nevratně.
                - `rm -i <soubor>` se před smazáním zeptá na potvrzení.
                - Před hromadným mazáním (např. `rm *.log`) si obsah zkontrolujte pomocí `ls *.log`.

                ### Úkol
                Smažte soubor `stary_log.txt`, který se nachází v aktuálním adresáři.

                ### Odevzdání
                Až bude soubor smazaný, spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Příkaz rm permanentně maže soubory. Jaký soubor máte smazat?",
                "Syntaxe je jednoduchá: rm název_souboru",
                "Použijte 'rm stary_log.txt' a spusťte 'shellgame submit'.",
            ],
            start: "cleanup",
            setup: &[file("cleanup/stary_log.txt", "old data")],
            check: &[
                no_file("cleanup/stary_log.txt")
                    .msg("Soubor stary_log.txt tu pořád je. Zkontrolujte pomocí 'ls', že mažete ve správném adresáři a že jste název napsali bez překlepu."),
            ],
            solution: r#"
                rm stary_log.txt
                shellgame submit
            "#,
            success: "Správně! Mazání v shellu je okamžité a nevratné, proto se vyplatí nejdřív ověřit, co mažete.",
            ..LEVEL
        },
        Level {
            title: "Mazání adresářů",
            text: r#"
                ### Cíl
                Smažte adresář i s jeho obsahem.

                ### Příkazy
                - `rmdir <adresář>` (odstraní prázdný adresář; s obsahem selže)
                - `rm -r <adresář>` (recursive — rekurzivně smaže adresář i s obsahem)

                ### Úkol
                Smažte adresář `temp`, který obsahuje nějaké dočasné soubory.

                ### Odevzdání
                Až bude adresář smazaný, spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Zkuste nejdřív 'rmdir temp'. Co se stane?",
                "Pokud adresář není prázdný, rmdir selže. Jaký přepínač potřebujete pro rekurzivní mazání?",
                "Použijte 'rm -r temp' pro smazání adresáře včetně obsahu a spusťte 'shellgame submit'.",
            ],
            start: "cleanup",
            setup: &[file("cleanup/temp/junk.txt", "junk")],
            check: &[
                no_file("cleanup/temp")
                    .msg("Zadaný adresář tu pořád je. Pokud rmdir skončil hláškou 'Directory not empty', pracoval přesně podle očekávání: umí odstranit jen prázdný adresář a nesmí se dotknout obsahu. Adresář s obsahem proto smažte až rekurzivní variantou příkazu rm."),
            ],
            solution: r#"
                rm -r temp
                shellgame submit
            "#,
            success: "Správně! Prázdný adresář zvládne 'rmdir', na adresář s obsahem je potřeba rekurzivní mazání ('rm -r').",
            ..LEVEL
        },
        Level {
            title: "Příprava projektu",
            text: r#"
                ### Cíl
                Připravte kompletní kostru nového projektu.

                ### Příkazy k použití
                - `mkdir -p <cesta>` pro adresáře
                - `touch <cesta>` pro soubory

                ### Úkol
                Vytvořte následující strukturu v adresáři `web`:
                - `web/index.html` (soubor)
                - `web/css/style.css` (soubor v podadresáři)
                - `web/js` (prázdný adresář)

                ### Odevzdání
                Až bude celá struktura hotová, spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Nejdřív si rozdělte úkol na adresáře a soubory. Všechny potřebné adresáře lze vytvořit jedním příkazem.",
                "Pro adresáře použijte 'mkdir -p' a pro prázdné soubory 'touch'; oba příkazy přijímají více cest.",
                "Spusťte 'mkdir -p web/css web/js', potom 'touch web/index.html web/css/style.css' a nakonec 'shellgame submit'.",
            ],
            start: "project",
            setup: &[clean("project/web"), dir("project")],
            check: &[
                is_file("project/web/index.html").msg("Chybí soubor web/index.html. Prázdný soubor založí 'touch'."),
                is_file("project/web/css/style.css")
                    .msg("Chybí soubor web/css/style.css. Nejdřív musí existovat podadresář css, teprve potom do něj lze soubor založit."),
                is_dir("project/web/js").msg("Chybí adresář web/js. Má zůstat prázdný, ale existovat musí."),
            ],
            solution: r#"
                mkdir -p web/css web/js
                touch web/index.html web/css/style.css
                shellgame submit
            "#,
            success: "Správně! Kostru projektu připravíte předem: adresáře jedním příkazem, prázdné soubory druhým.",
            ..LEVEL
        },
        Level {
            title: "Úklid nepořádku",
            text: r#"
                ### Cíl
                Smažte více souborů najednou a zachovejte důležitá data.

                ### Příkazy
                - `rm soubor1 soubor2 ...` (smaže více souborů najednou)

                ### Úkol
                Level začíná přímo v adresáři `mess`. Smažte v něm soubory `error.log`, `temp.dat` a `junk.tmp`.
                Soubor `keep_me.txt` musí zůstat zachovaný. Hodnotí se výsledný stav, ne počet příkazů.

                ### Odevzdání
                Až budou nepotřebné soubory pryč, spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Příkaz 'rm' dokáže smazat více souborů najednou, stačí je uvést oddělené mezerami.",
                "Spusťte 'rm error.log temp.dat junk.tmp' (nebo je smažte postupně po jednom).",
                "Ověřte pomocí 'ls', že zůstal jen soubor keep_me.txt, a spusťte 'shellgame submit'.",
            ],
            start: "mess",
            setup: &[
                file("mess/error.log", ""),
                file("mess/temp.dat", ""),
                file("mess/junk.tmp", ""),
                file("mess/keep_me.txt", "important"),
            ],
            check: &[
                no_file("mess/error.log").msg("Soubor error.log stále existuje. Smažte ho příkazem 'rm'."),
                no_file("mess/temp.dat").msg("Soubor temp.dat stále existuje. Smažte ho příkazem 'rm'."),
                no_file("mess/junk.tmp").msg("Soubor junk.tmp stále existuje. Smažte ho příkazem 'rm'."),
                content_is("mess/keep_me.txt", "important")
                    .msg("Soubor keep_me.txt se změnil, a ten měl zůstat netknutý. Level obnovíte příkazem 'shellgame reset'.")
                    .missing("Soubor keep_me.txt zmizel, a ten měl zůstat. Typická příčina je příliš široký argument jako 'rm *', který smaže všechno v adresáři. Příště si nejdřív vypište, co by takový příkaz zasáhl. Level obnovíte příkazem 'shellgame reset'."),
            ],
            solution: r#"
                rm error.log temp.dat junk.tmp
                shellgame submit
            "#,
            success: "Správně! Jeden příkaz zvládne víc cest, ale odpovídáte za to, které soubory zasáhne.",
            ..LEVEL
        },
        Level {
            title: "Souhrn Sekce 4",
            text: r#"
                ### Výzva: Stavitel souborového systému

                Ukažte, že umíte vytvářet i mazat!

                ### Úkol
                V `level-4/challenge`:

                1. Vytvořte adresářovou strukturu: `myproject/src` a `myproject/docs`
                2. Vytvořte soubor `myproject/README.md`
                3. Smažte existující soubor `delete_me.txt`
                4. Smažte existující prázdný adresář `empty_dir`

                ### Shrnutí příkazů Sekce 4
                ```
                mkdir adresar       → Nový adresář
                mkdir -p a/b/c      → Celá cesta najednou
                touch soubor        → Nový prázdný soubor
                rm soubor           → Smazat soubor
                rmdir adresar       → Smazat prázdný adresář
                rm -r adresar       → Smazat adresář s obsahem
                ```

                ### Odevzdání
                Po splnění všech bodů spusťte `shellgame submit`.
            "#,
            hints: &[
                "Rozdělte úkol na výsledky: dva adresáře a jeden soubor vytvořit, dvě existující položky odstranit.",
                "Vnořené adresáře vytvoří 'mkdir -p'; prázdný soubor 'touch'. Soubor a prázdný adresář se mažou různými příkazy.",
                "Použijte 'mkdir -p myproject/src myproject/docs', 'touch myproject/README.md', 'rm delete_me.txt' a 'rmdir empty_dir'.",
            ],
            start: "challenge",
            setup: &[clean("challenge"), dir("challenge/empty_dir"), file("challenge/delete_me.txt", "Delete this!\n")],
            check: &[
                is_dir("challenge/myproject/src").msg("Chybí adresář myproject/src. Zkontrolujte vytvořenou strukturu."),
                is_dir("challenge/myproject/docs").msg("Chybí adresář myproject/docs."),
                is_file("challenge/myproject/README.md").msg("Chybí soubor myproject/README.md. Zkontrolujte jeho název a umístění."),
                no_file("challenge/delete_me.txt").msg("Soubor delete_me.txt stále existuje."),
                no_file("challenge/empty_dir").msg("Prázdný adresář empty_dir stále existuje."),
            ],
            solution: r#"
                mkdir -p myproject/src myproject/docs
                touch myproject/README.md
                rm delete_me.txt
                rmdir empty_dir
                shellgame submit
            "#,
            success: "Brilantní! Dokončili jste Sekci 4. Umíte vytvářet i bourat!",
            ..LEVEL
        },
    ],
};
