//! Examining files: what `ls -l`, `file` and `less` say about a file beyond its name.
//!
//! **Purpose.** A name or an extension is only a claim. The section teaches the student to ask the
//! system instead: how big a file is, what it really is, and what is in a file too long for `cat`.
//!
//! **Learning objectives.** After this section a student can:
//! - read a file's size in bytes from `ls -l`; find a file by its exact size;
//! - identify a file's real type with `file`, whatever its extension (an image, a text file
//!   disguised as `.jpg`, a Python script), and count files by type;
//! - page through and search a long file with `less` (`/text`, `n`, `q`) instead of printing it;
//! - know that `file` does not tell whether a file may be run: that is a permission (`ls -l`).
//!
//! **Introduces:** the size column of `ls -l`, `file`, `less`.\
//! **Relies on:** [navigation](super::s01_navigation) (`ls -l`), [files](super::s02_files) (`cat`)
//! and, ahead of [wildcards](super::s07_wildcards), `file *` and `file *.jpg` (given ready-made).\
//! **Leaves for later:** reading and changing permissions ([permissions](super::s08_permissions)),
//! copying and moving ([copy_move](super::s06_copy_move)).

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Zkoumání souborů",
    intro: r#"
        V této sekci se naučíte, jak zjistit více informací o souborech, než jen jejich název.

        ### Proč je to důležité?

        #### Bezpečnost
        Stáhli jste soubor `faktura.pdf` z e-mailu. Ale je to opravdu PDF?
        Příkaz `file` pomůže odhalit skutečný formát a podezřelý nesoulad s názvem.
        Nejde však o antivirovou ani úplnou bezpečnostní kontrolu.

        #### Praktická práce
        - **Vývojář**: "Proč mi nefunguje import? Ten soubor vypadá prázdný..." → `ls -l` ukáže, že má 0 bajtů
        - **Admin**: "Který log zabírá místo na disku?" → `ls -lh` ukáže velikosti čitelně
        - **Admin**: "Kde je chyba v dlouhém logu?" → `less` umožní soubor procházet a prohledávat
        - **Student**: "Je tohle textový soubor nebo binární?" → `file` řekne přesný typ

        ### Výstup `ls -l` vysvětlen
        ```
        -rw-r--r-- 1 student users 20480 Dec  5 10:30 dokument.txt
        │└┬┘└┬┘└┬┘    │       │     │     │            │
        │ │  │  │     │       │     │     │            └─ název
        │ │  │  │     │       │     │     └─ datum změny
        │ │  │  │     │       │     └─ velikost v bajtech
        │ │  │  │     │       └─ skupina
        │ │  │  │     └─ vlastník
        │ │  │  └─ oprávnění pro ostatní (r--)
        │ │  └─ oprávnění pro skupinu (r--)
        │ └─ oprávnění pro vlastníka (rw-)
        └─ typ souboru (- = soubor, d = adresář)
        ```

        ---

        ### Příkaz `file` — pravda o obsahu
        Názvy souborů mohou být klamavé; příkaz `file` zkoumá skutečný obsah:

        ```
        $ file foto.jpg
        foto.jpg: JPEG image data     ← Opravdu obrázek

        $ file fake.jpg  
        fake.jpg: ASCII text          ← Někdo lhal! Je to text.
        ```

        ### Co se naučíte
        - Zjišťovat velikost souborů (`ls -l`, `ls -lh`)
        - Procházet a prohledávat velké soubory (`less`, `/`, `q`)
        - Určovat typ souboru (`file`)
        - Rozlišovat mezi textovými a binárními soubory

        ### Pokračování
        Pro zahájení prvního levelu této sekce stiskněte Enter.
    "#,
    levels: &[
        Level {
            title: "Velikost souboru",
            text: r#"
                ### Cíl
                Zjistěte přesnou velikost souboru v bajtech.

                Příkaz `ls -l` (long listing) zobrazí podrobné informace o souborech včetně jejich
                velikosti v bajtech (5. sloupec). Přepínač `-h` ji zaokrouhlí a zapíše s příponou
                `K`, `M` nebo `G` (kilobajty, megabajty, gigabajty), takže přesný počet bajtů s ním neuvidíte.

                ### Příkazy
                - `ls -l` (zobrazí detaily; velikost je v pátém sloupci)

                ### Úkol
                1. V aktuálním adresáři je soubor `database.db`.
                2. Pomocí `ls -l` zjistěte jeho přesnou velikost v bajtech.
                3. Odevzdejte tuto velikost jako číslo.

                ### Odevzdání
                `shellgame submit <bajty>`
            "#,
            hints: &[
                "Příkaz 'ls -l' zobrazí podrobnosti o souborech. Který sloupec obsahuje velikost?",
                "Ve výstupu ls -l je velikost v bajtech - hledejte číslo před datem.",
                "Použijte 'ls -l database.db' a podívejte se na pátý sloupec.",
            ],
            start: "sizes",
            setup: &[binary("sizes/database.db", &[b'x'; 12345])],
            answer: int(12345)
                .mistakes(&[
                    ("13", "To je zaokrouhlená velikost v kilobajtech z 'ls -lh'. Odevzdejte přesný počet bajtů z 'ls -l' bez '-h'."),
                    ("12", "To je velikost v kilobajtech. Odevzdejte přesný počet bajtů z 'ls -l' bez '-h'."),
                ])
                .error("Odevzdejte přesný počet bajtů jako samotné číslo z pátého sloupce 'ls -l' (bez '-h' a bez jednotek)."),
            solution: r#"shellgame submit "$(ls -l database.db | tr -s ' ' | cut -d' ' -f5)""#,
            success: "Správně! Pátý sloupec 'ls -l' udává velikost v bajtech; '-h' ji zaokrouhlí do čitelnější podoby.",
            ..LEVEL
        },
        Level {
            title: "Hledání podle velikosti",
            text: r#"
                ### Cíl
                Najděte v podrobném výpisu soubor s konkrétní velikostí.

                ### Příkazy
                - `ls -l` (zobrazí podrobnosti včetně velikosti v 5. sloupci)

                ### Úkol
                V adresáři jsou čtyři soubory.
                Pouze jeden z nich má velikost přesně **1337 bajtů**.

                1. Spusťte `ls -l` a zkontrolujte sloupec s velikostí.
                2. Najděte soubor s velikostí přesně 1337 bajtů.
                3. Odevzdejte název tohoto souboru.

                ### Odevzdání
                `shellgame submit <název_souboru>`
            "#,
            hints: &[
                "Příkaz 'ls -l' zobrazuje podrobný výpis souborů včetně velikosti v bajtech v pátém sloupci.",
                "Spusťte 'ls -l' a hledejte řádek, kde je velikost přesně 1337.",
                "Název souboru je na konci příslušného řádku. Odevzdejte ho příkazem 'shellgame submit <soubor>'.",
            ],
            start: "search",
            setup: &[
                binary("search/file_a", &[b'x'; 1000]),
                binary("search/file_b", &[b'x'; 2000]),
                binary("search/file_c", &[b'x'; 1338]),
                binary("search/file_d", &[b'x'; 1337]),
            ],
            answer: exact("file_d")
                .mistakes(&[
                    ("file_c", "Soubor 'file_c' má 1338 bajtů (o 1 bajt více). Hledejte přesně 1337 bajtů."),
                    ("file_a", "Soubor 'file_a' má 1000 bajtů. Hledejte přesně 1337 bajtů."),
                    ("file_b", "Soubor 'file_b' má 2000 bajtů. Hledejte přesně 1337 bajtů."),
                ]),
            solution: r#"shellgame submit "$(ls -l | grep ' 1337 ' | grep -o '[^ ]*$')""#,
            success: "Správně! Podrobný výpis se čte po sloupcích — stačí porovnat ten jeden, který vás zajímá.",
            ..LEVEL
        },
        Level {
            title: "Typ souboru",
            text: r#"
                ### Cíl
                Rozpoznejte obrázek mezi soubory bez přípony podle jejich obsahu.

                V Linuxu přípona souboru (např. `.txt`, `.jpg`) neurčuje jeho typ. O tom rozhoduje obsah.
                Příkaz `file` prozkoumá obsah souboru a řekne vám, o jaký typ se jedná.

                ### Příkazy
                - `file <soubor>` (zjistí typ konkrétního souboru)
                - `file *` (zjistí typ všech souborů v aktuálním adresáři)

                ### Úkol
                V adresáři jsou tři soubory bez přípony.
                Jeden z nich je obrázek (`JPEG image data`). Zjistěte který a odevzdejte jeho název.

                ### Odevzdání
                `shellgame submit <soubor>`
            "#,
            hints: &[
                "Příkaz 'file' zkoumá obsah souboru, ne jeho název. Jak zjistíte typ všech souborů najednou?",
                "Zkuste 'file *': vypíše typ každého souboru v adresáři na samostatný řádek. Hledejte 'JPEG' ve výstupu.",
                "Použijte 'file *' a najděte soubor označený jako 'JPEG image data'.",
            ],
            start: "types",
            setup: &[
                file("types/file1", "This is a text file."),
                binary("types/file2", b"\x00\x01\x02\x03"),
                binary("types/file3", JPEG),
            ],
            answer: exact("file3"),
            solution: r#"shellgame submit "$(file * | grep JPEG | cut -d: -f1)""#,
            success: "Správně! O typu souboru rozhoduje jeho obsah, ne název ani přípona.",
            ..LEVEL
        },
        Level {
            title: "Hledání ve velkém souboru",
            text: r#"
                ### Cíl
                Naučte se prohlížet a prohledávat velké soubory pomocí interaktivního nástroje `less`.

                Příkaz `cat` vypíše celý soubor najednou, což je u dlouhých souborů nepraktické.
                Interaktivní stránkovač `less` umožní souborem pohodlně listovat a hledat v něm.

                ### 🚪 Jak z less odejít: klávesa `q` (Quit)
                Nástroj `less` zabere celou obrazovku terminálu. Během jeho běhu nelze zadávat
                běžné příkazy shellu.
                **Pro ukončení prohlížeče a návrat do příkazové řádky stiskněte klávesu `q`.**

                ### Ovládání less
                ```
                q                     → UKONČIT prohlížení (návrat do shellu)
                /hledany_text         → Hledat text v souboru (potvrdit Enterem)
                n                     → Další výskyt hledaného textu
                Enter / šipka dolů    → Posun o jeden řádek dolů
                Mezerník / Page Down  → Posun o celou stránku dolů
                b / Page Up           → Posun o stránku nahoru
                šipka nahoru          → Posun o řádek nahoru
                g / G                 → Na začátek / na konec souboru
                ```

                ### Úkol
                Soubor `server.log` má 200 řádků. Najděte v něm chybový kód na řádku s textem `CRITICAL`:

                1. Otevřete soubor: `less server.log`
                2. Hledejte: stiskněte `/`, napište `CRITICAL` a stiskněte **Enter**
                3. Přečtěte chybový kód (číslo za slovem `Code`) na konci nalezeného řádku
                4. **Stiskněte klávesu `q`** pro ukončení `less` a návrat do příkazové řádky
                5. Odešlete nalezený kód pomocí `shellgame submit`

                ### Odevzdání
                `shellgame submit <číslo>`
            "#,
            hints: &[
                "V less použijte / pro vyhledávání. Napište /CRITICAL a stiskněte Enter.",
                "Nalezený řádek obsahuje číslo na konci. Přečtěte ho.",
                "Pro návrat do shellu stiskněte klávesu 'q' (quit).",
                "Pozor: neodevzdáváte číslo řádku (v hranatých závorkách na začátku), ale kód na konci věty.",
            ],
            start: "logs",
            setup: &[build(|root| write(root, "logs/server.log", server_log()))],
            answer: int(42)
                .mistakes(&[("137", "137 je číslo řádku, ne kód na konci. Přečtěte celý CRITICAL řádek.")])
                .error("Tohle není správný kód. Najděte řádek s 'CRITICAL' a přečtěte číslo na konci."),
            solution: r#"shellgame submit "$(grep CRITICAL server.log | grep -o '[0-9]*$')""#,
            success: "Správně! 'less' usnadňuje hledání ve velkých souborech: '/' hledá, 'n' skočí na další výskyt a 'q' ho ukončí.",
            ..LEVEL
        },
        Level {
            title: "Zamaskovaný soubor",
            text: r#"
                ### Cíl
                Najděte soubor s příponou `.jpg`, který ve skutečnosti není obrázkem.

                V adresáři `downloads` je mnoho různých stažených souborů (dokumenty, archivy,
                skripty i logy). Někdo se pokusil ukrýt tajnou textovou zprávu tím, že ji pojmenoval
                s příponou `.jpg`.

                ### Příkazy
                - `file *.jpg` (zkontroluje pouze soubory s příponou .jpg místo všech položek)

                ### Úkol
                1. V adresáři `downloads` je mnoho různých souborů. Spuštění `file *` by vypsalo
                   desítky položek bez užitku.
                2. Pomocí `file *.jpg` zkontrolujte pouze soubory s příponou `.jpg`.
                3. Najděte, který soubor je ve skutečnosti textový (`ASCII text`).
                4. Odevzdejte název tohoto souboru.

                ### Odevzdání
                `shellgame submit <soubor>`
            "#,
            hints: &[
                "Příkaz 'file' zkoumá skutečný obsah souboru bez ohledu na jeho příponu.",
                "Místo 'file *' použijte 'file *.jpg' — zkontrolujete jen soubory s příponou .jpg.",
                "Hledejte mezi soubory .jpg ten, u kterého 'file' vypíše 'ASCII text'. Odevzdejte jeho název.",
            ],
            start: "downloads",
            setup: &[
                binary("downloads/photo1.jpg", JPEG),
                binary("downloads/photo2.jpg", JPEG),
                binary("downloads/photo3.jpg", JPEG),
                binary("downloads/photo4.jpg", JPEG),
                binary("downloads/photo5.jpg", JPEG),
                binary("downloads/vacation.jpg", JPEG),
                binary("downloads/sunset.jpg", JPEG),
                binary("downloads/wallpaper.jpg", JPEG),
                binary("downloads/avatar.jpg", JPEG),
                binary("downloads/nature.jpg", JPEG),
                file("downloads/secret.jpg", "This is actually a secret text file.\n"),
                binary("downloads/manual.pdf", b"%PDF-1.4\n%EOF\n"),
                binary("downloads/archive.zip", b"PK\x03\x04\x14\x00\x00\x00\x08\x00"),
                binary("downloads/backup.tar.gz", b"\x1f\x8b\x08\x00\x00\x00\x00\x00"),
                binary("downloads/music.mp3", b"ID3\x03\x00\x00\x00"),
                file("downloads/notes.txt", "Random notes and meeting minutes.\n"),
                file("downloads/install.sh", r#"
                    #!/bin/sh
                    echo 'installing...'
                "#),
                file("downloads/setup.log", "[INFO] Setup initialized.\n"),
                file("downloads/table.csv", r#"
                    id,name,value
                    1,alpha,100
                "#),
                file("downloads/data.json", r#"
                    {"status": "ok", "count": 42}
                "#),
                file("downloads/config.yaml", r#"
                    env: production
                    debug: false
                "#),
                file("downloads/readme.md", r#"
                    # Downloads Readme
                    Just downloads.
                "#),
                file("downloads/styles.css", "body { margin: 0; padding: 0; }\n"),
                file("downloads/index.html", "<!DOCTYPE html><html><body>Test</body></html>\n"),
                file("downloads/script.py", r#"
                    #!/usr/bin/env python3
                    print('hello')
                "#),
                file("downloads/checksums.sha256", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n"),
                binary("downloads/report.doc", b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"),
                binary("downloads/app.bin", b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00"),
            ],
            answer: exact("secret.jpg"),
            solution: r#"shellgame submit "$(file *.jpg | grep 'ASCII text' | cut -d: -f1)""#,
            success: "Správně! Přípona je jen dohoda — 'file' čte skutečný obsah, a tak odhalí i zamaskovaný soubor.",
            ..LEVEL
        },
        Level {
            title: "Rozpoznání skriptu v Pythonu",
            text: r#"
                ### Cíl
                Rozpoznejte skript v Pythonu mezi různými typy souborů.

                Příkaz `file` dokáže podle obsahu spolehlivě rozpoznat skript v Pythonu bez ohledu na jeho název.
                Popis `Python script` ale neříká, zda má soubor právo ke spuštění (`x`) — typ zjišťuje `file`,
                kdežto oprávnění zobrazuje `ls -l` a mění `chmod` (tomu se věnuje sekce „Oprávnění“).

                ### 💡 Pozor na slovo `executable`
                U skriptu vypíše `file` například `Python script, ASCII text executable`. Slovo `executable`
                tu znamená jen to, že soubor začíná řádkem `#!` (shebang), který říká, jakým programem se má
                skript spustit. O oprávněních nic neříká: když spustíte `ls -l`, uvidíte, že žádný soubor
                v tomto adresáři právo `x` nemá.

                ### Příkazy
                - `file *` (vypíše typy všech souborů v aktuálním adresáři)

                ### Úkol
                Nacházíte se přímo v adresáři `bin` (nikam nemusíte přecházet):
                1. Prozkoumejte soubory v aktuálním adresáři pomocí `file *`.
                2. Najděte soubor, který příkaz `file` označí jako `Python script`.
                3. Odevzdejte název tohoto souboru.

                ### Odevzdání
                `shellgame submit <soubor>`
            "#,
            hints: &[
                "Jste přímo v adresáři 'bin'. Spusťte 'file *' pro zjištění typů všech souborů.",
                "Spusťte 'file *' a hledejte soubor, u kterého výstup uvádí 'Python script'.",
                "Odevzdejte název souboru označeného jako 'Python script'; slovo 'executable' v popisu neznamená právo x.",
            ],
            start: "bin",
            setup: &[
                file("bin/readme.txt", "Just text."),
                file("bin/run.sh", r#"
                    #!/bin/bash
                    echo hello
                "#),
                file("bin/calc.py", r#"
                    #!/usr/bin/env python3
                    print(1+1)
                "#),
                binary("bin/program", b"\x7fELF"),
            ],
            answer: suffix("calc.py")
                .mistakes(&[
                    ("calc", "Odevzdejte celý název souboru i s příponou."),
                    ("readme.txt", "To je prostý text, ne skript. Řiďte se přesným popisem z 'file *', ne příponou názvu."),
                    ("run.sh", "To je shellový skript ('shell script'), ne Python. Přečtěte popis z 'file *' celý."),
                    ("program", "To je zkompilovaný program (ELF), ne skript. Porovnejte popisy z 'file *'."),
                ])
                .error("To není skript v Pythonu. Rozhoduje přesný popis z 'file *', ne přípona názvu."),
            solution: r#"shellgame submit "$(file * | grep 'Python script' | cut -d: -f1)""#,
            success: "Správně! 'file' určí typ souboru podle obsahu; slovo 'executable' znamená jen řádek '#!', právo ke spuštění ukáže až 'ls -l'.",
            ..LEVEL
        },
        Level {
            title: "Souhrn Sekce 5",
            text: r#"
                ### Výzva: Detektiv souborů

                Ukažte, že umíte identifikovat typy souborů!

                ### Úkol
                V `level-5/mystery` je 5 souborů s podivnými názvy.
                Zjistěte typ každého a odpovězte na otázky:

                1. Kolik je tam **prostých textových** souborů (popis začíná `ASCII text`, bez skriptů)?
                2. Kolik je tam **obrázků** (image)?
                3. Jaký je název jediného skriptu v **Pythonu** (bez cesty)?

                ### Formát odpovědi
                `<počet_textových>,<počet_obrázků>,<název_skriptu>`

                Skript počítejte samostatně, i když jeho popis také obsahuje `ASCII text`.

                ### Shrnutí příkazů Sekce 5
                ```
                file soubor     → Zjistí typ souboru
                file *          → Typy všech souborů
                file -b soubor  → Jen typ bez názvu
                ```

                ### Odevzdání
                `shellgame submit <počet_textových>,<počet_obrázků>,<název_skriptu>`
            "#,
            hints: &[
                "Použijte 'file *' k zobrazení typů všech souborů najednou.",
                "Pro prostý text hledejte popis začínající 'ASCII text'. Skripty do tohoto počtu nepatří.",
                "Popis s 'image' znamená obrázek. Popis s 'Python script' znamená skript v Pythonu, ne prostý text.",
            ],
            start: "mystery",
            setup: &[
                clean("mystery"),
                file("mystery/data.bin", "This is just plain text.\n"),
                file("mystery/notes.xyz", "More text content.\n"),
                binary("mystery/config.txt", b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc```\x00\x00\x00\x04\x00\x01\xf6\x178U\x00\x00\x00\x00IEND\xaeB`\x82"),
                file("mystery/analyzer.dat", r#"
                    #!/usr/bin/env python3
                    import sys
                    print('Hello')
                "#),
                binary("mystery/readme.doc", b"\x7fELF\x02\x01\x01"),
            ],
            answer: parts(",", &[
                int(2)
                    .error("Počet prostých textových souborů není správný. Počítejte popisy začínající 'ASCII text', nikoli skripty."),
                int(1)
                    .error("Počet obrázků není správný. Hledejte 'image' ve výstupu 'file *'."),
                exact("analyzer.dat").nocase().error("Název skriptu v Pythonu není správný. Hledejte 'Python' ve výstupu 'file *'."),
            ])
                .format("Formát: počet_textových,počet_obrázků,název_skriptu"),
            solution: r#"shellgame submit "$(file -b * | grep -c '^ASCII text'),$(file -b * | grep -c image),$(file * | grep 'Python script' | cut -d: -f1)""#,
            success: "Skvělá detektivní práce! Dokončili jste Sekci 5. Příponám se už nedáte zmást!",
            ..LEVEL
        },
    ],
};

const JPEG: &[u8] = b"\xff\xd8\xff\xe0\x00\x10JFIF";

/// The 200 lines of `logs/server.log`: routine `INFO` lines, a `WARNING` every tenth line, an `ERROR`
/// every seventh, and one `CRITICAL` line (137) that carries the code to find.
fn server_log() -> String {
    let lines: Vec<String> = (1..=200)
        .map(|line| {
            let entry = match line {
                137 => "CRITICAL: System failure detected - Code 42",
                n if n % 10 == 0 => "WARNING: High memory usage",
                n if n % 7 == 0 => "ERROR: Connection timeout",
                _ => "INFO: Normal operation",
            };
            format!("[{line:03}] {entry}")
        })
        .collect();
    lines.join("\n") + "\n"
}
