//! Input, output and exit status: `>`, `>>`, pipes, quoting, keyboard input, `&&` and `||`.
//!
//! **Purpose.** Commands become building blocks: output is saved, appended or passed on, input can
//! come from the keyboard, and success or failure decides what runs next. It is also where the
//! small filters that pipelines are made of first appear.
//!
//! **Learning objectives.** After this section a student can:
//! - save output with `>` (overwrites) or `>>` (keeps content), and join files with `cat a b > c`;
//! - quote text and file names that contain spaces;
//! - use `head -n`, `tail -n` and `wc` on files, and build pipelines (`sort | uniq | wc -l`);
//! - type a file in with `cat > file`, end it with Ctrl+D (EOF), and know that Ctrl+C interrupts
//!   instead (lines already sent with Enter stay; the game cannot tell which one ended the input);
//! - use the exit status: `&&` continues only after success, `||` only after failure.
//!
//! **Introduces:** `>`, `>>`, `|`, `echo` for text, `head`, `tail`, `wc`, `sort`, `uniq`, `true`,
//! `false`, `&&`, `||`, quoting, EOF.\
//! **Relies on:** [navigation](super::s01_navigation) (`ls`), [files](super::s02_files) (`cat`)
//! and, ahead of [search](super::s11_search), `grep "pattern" file` as a ready-made filter.\
//! **Leaves for later:** error output (`2>`, `&>`, `/dev/null`: [streams](super::s10_streams)),
//! searching itself (`grep` options, `find`: [search](super::s11_search)).

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Vstup, výstup a stav příkazů",
    intro: r#"
        Příkaz obvykle posílá výstup na obrazovku. Tři operátory rozhodnou, kam poteče dál:

        ```text
        příkaz > soubor       výstup přepíše soubor
        příkaz >> soubor      výstup se přidá na konec
        příkaz | další        výstup se stane vstupem dalšího příkazu
        ```

        ### Tři mentální modely

        - `>` je **nový zápis**. Starý obsah cílového souboru zmizí.
        - `>>` je **připojení**. Dosavadní obsah zůstane.
        - `|` je **roura mezi příkazy**. Nevytváří soubor, jen předává data dál.

        ```bash
        ls > seznam.txt
        echo "další řádek" >> seznam.txt
        grep ERROR access.log | wc -l
        ```

        Má-li text nebo název souboru mezery, uzavřete každou takovou část zvlášť:

        ```bash
        echo "Ahoj svete" > "muj pozdrav.txt"
        ```

        ---

        # Klávesnicový vstup a stav příkazu

        `cat > soubor` čte řádky z klávesnice. **Ctrl+D** oznámí EOF, tedy konec vstupu,
        a `cat` řádně skončí. **Ctrl+C** je interrupt: běžící příkaz přeruší.

        Každý příkaz také vrací stavový neboli návratový kód:

        - `0` znamená úspěch,
        - nenulový kód znamená neúspěch,
        - `první && druhý` pokračuje jen po úspěchu,
        - `první || druhý` pokračuje jen po neúspěchu.

        `&&` a `||` fungují stejně v Bashi i ve fish.

        ### Co se naučíte

        - ukládat a přidávat výstup pomocí `>` a `>>`
        - správně citovat víceslovný text i názvy souborů
        - zapisovat interaktivní vstup přes `cat` a ukončit jej pomocí EOF
        - propojovat příkazy pomocí `|`
        - filtrovat a zpracovávat text přes `grep`, `head`, `tail`, `wc`, `sort` a `uniq`
        - reagovat na úspěch či neúspěch příkazu pomocí `&&` a `||`
    "#,
    levels: &[
        Level {
            title: "Uložení výstupu",
            text: r#"
                Operátor `>` přesměruje výstup příkazu do souboru. Pokud soubor neexistuje, vytvoří se.
                Pokud existuje, **přepíše se**.

                ### Úkol
                Uložte seznam souborů v aktuálním adresáři (výstup `ls`) do souboru `seznam.txt`.

                ### Příkazy
                - `ls > seznam.txt`

                ### Odevzdání
                Po vytvoření souboru spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Použijte operátor '>' pro přesměrování výstupu.",
                "Příkaz 'ls' vypíše obsah adresáře.",
                "Zkuste: 'ls > seznam.txt'.",
            ],
            start: "redirection",
            setup: &[clean("redirection/seznam.txt"), file("redirection/file1", ""), file("redirection/file2", "")],
            check: &[
                contains("redirection/seznam.txt", &["file1", "file2"])
                    .msg("Soubor neobsahuje očekávaný výstup příkazu ls.")
                    .missing("Soubor seznam.txt zatím neexistuje."),
            ],
            solution: r#"
                ls > seznam.txt
                shellgame submit
            "#,
            success: "Správně! Výstup příkazu nemusí skončit na obrazovce — dá se uložit a dál s ním pracovat.",
            ..LEVEL
        },
        Level {
            title: "Přidání na konec",
            text: r#"
                Operátor `>>` (append) přidá výstup na konec souboru, aniž by smazal původní obsah.

                ### Úkol
                Máte soubor `log.txt` s nějakým obsahem. Přidejte na jeho konec text "Konec logu"
                pomocí příkazu `echo`.

                ### Příkazy
                - `echo "Text" >> soubor`

                ### Odevzdání
                Po přidání textu spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Dvě šipky '>>' znamenají append (připojení na konec souboru bez přepsání obsahu).",
                r#"Spusťte 'echo "Konec logu" >> log.txt'."#,
            ],
            start: "redirection",
            setup: &[
                file("redirection/log.txt", r#"
                    Start logu
                    Zaznam 1
                "#),
            ],
            check: &[
                content_is("redirection/log.txt", r#"
                    Start logu
                    Zaznam 1
                    Konec logu
                "#)
                    .msg("Soubor musí zachovat původní obsah a přidat nový řádek přesně na konec. Zmizely-li původní řádky, použili jste jednoduchou šipku '>', která soubor přepíše; připojení zajistí až zdvojená šipka. Původní stav vrátí 'shellgame reset'.")
                    .missing("Soubor log.txt chybí. Původní stav vrátí 'shellgame reset'."),
            ],
            solution: r#"
                echo 'Konec logu' >> log.txt
                shellgame submit
            "#,
            success: "Správně! Zdvojená šipka připojuje, takže předchozí obsah souboru zůstane zachovaný.",
            ..LEVEL
        },
        Level {
            title: "Spojování souborů",
            text: r#"
                Příkaz `cat` (concatenate) umí vypsat obsah více souborů za sebou.
                Když to zkombinujete s přesměrováním, můžete spojit více souborů do jednoho.

                ### Úkol
                Spojte obsah souborů `part1.txt` a `part2.txt` do nového souboru `full.txt`.

                ### Příkazy
                - `cat soubor1 soubor2 > novy_soubor`

                ### Odevzdání
                Po vytvoření spojeného souboru spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Příkaz 'cat' umí přijmout více souborů najednou a vypsat jejich obsahy za sebou.",
                "Výstup více souborů z 'cat' můžete přesměrovat pomocí '>' do cílového souboru.",
                "Spusťte 'cat part1.txt part2.txt > full.txt'. Pořadí argumentů určuje pořadí v souboru.",
            ],
            start: "concat",
            setup: &[
                clean("concat/full.txt"),
                file("concat/part1.txt", "First part.\n"),
                file("concat/part2.txt", "Second part.\n"),
            ],
            check: &[
                content_is("concat/full.txt", r#"
                    First part.
                    Second part.
                "#)
                    .msg("Soubor musí obsahovat obě části přesně v zadaném pořadí. Jsou-li obsahy na místě, ale prohozené, prohodili jste argumenty příkazu cat: jejich pořadí určuje pořadí řádků ve výstupu.")
                    .missing("Soubor full.txt zatím neexistuje."),
            ],
            solution: r#"
                cat part1.txt part2.txt > full.txt
                shellgame submit
            "#,
            success: "Správně! Příkaz cat čte soubory v pořadí argumentů a stejné pořadí má i výsledek.",
            ..LEVEL
        },
        Level {
            title: "Uvozovky v textu i názvu",
            text: r#"
                Shell dělí příkaz podle mezer. Uvozovky proto chrání víceslovný text i název
                souboru s mezerami — každou část uzavřete zvlášť.

                ### Úkol
                Pomocí `echo` vytvořte soubor `muj pozdrav.txt` s jediným řádkem `Ahoj svete`.

                ### Příkaz
                - `echo "Ahoj svete" > "muj pozdrav.txt"`

                První dvojice uvozovek chrání text, druhá název výstupního souboru.
                Bez nich by shell vytvořil soubor `muj` a slovo `pozdrav.txt` by poslal do `echo`.

                ### Odevzdání
                Po vytvoření souboru spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Mezery oddělují argumenty. Uvozovky udrží více slov pohromadě jako jeden text nebo jednu cestu.",
                "Uzavřete do uvozovek text za 'echo' a zvlášť také název za '>'.",
                r#"Spusťte echo "Ahoj svete" > "muj pozdrav.txt" a potom 'shellgame submit'."#,
            ],
            start: "echo",
            setup: &[clean("echo/pozdrav.txt"), clean("echo/muj"), clean("echo/muj pozdrav.txt"), dir("echo")],
            check: &[
                content_is("echo/muj pozdrav.txt", "Ahoj svete\n")
                    .msg("Soubor 'muj pozdrav.txt' má obsahovat jediný řádek 'Ahoj svete' zakončený novým řádkem, přesně tak, jak ho zapíše echo.")
                    .missing("Soubor 'muj pozdrav.txt' zatím neexistuje. Bez uvozovek kolem názvu vznikne jen soubor 'muj'."),
            ],
            solution: r#"
                echo "Ahoj svete" > "muj pozdrav.txt"
                shellgame submit
            "#,
            success: "Správně! Uvozovky drží pohromadě to, co by shell jinak rozdělil podle mezer.",
            ..LEVEL
        },
        Level {
            title: "Propojení příkazů (Pipes)",
            text: r#"
                Znak `|` (pipe/roura) pošle výstup jednoho příkazu jako vstup druhému.

                ### Stavební kameny
                - `grep "vzor" soubor` vypíše jen ty řádky souboru, které vzor obsahují.
                  Podrobně se `grep` probírá v sekci „Vyhledávání“; tady ho použijeme jako hotový filtr.
                - `wc -l` spočítá řádky, které dostane na vstupu.
                - `|` pošle výstup levého příkazu na vstup pravého.

                ### Úkol
                V aktuálním adresáři je soubor `access.log` s mnoha řádky.
                Spočítejte, kolik řádků obsahuje slovo "ERROR". Příkaz si složte sami ze tří dílů výše.

                ### Odevzdání
                Odevzdejte nalezený počet (číslo).
                `shellgame submit <číslo>`
            "#,
            hints: &[
                "Pipe (|) propojuje výstup prvního příkazu se vstupem druhého.",
                "grep najde řádky s 'ERROR', wc -l je spočítá. Spojte je pomocí |.",
                r#"Použijte: grep "ERROR" access.log | wc -l"#,
            ],
            start: "pipes",
            setup: &[
                file("pipes/access.log", r#"
                    2024-01-01 10:00:00 INFO Server started
                    2024-01-01 10:05:23 ERROR Connection refused
                    2024-01-01 10:10:45 INFO User logged in
                    2024-01-01 10:15:00 WARNING Low memory
                    2024-01-01 10:20:12 ERROR Database timeout
                    2024-01-01 10:25:00 INFO Request processed
                    2024-01-01 10:30:33 ERROR File not found
                    2024-01-01 10:35:00 INFO Cache cleared
                    2024-01-01 10:40:55 ERROR Permission denied
                    2024-01-01 10:45:00 DEBUG Verbose output
                    2024-01-01 10:50:18 ERROR Network unreachable
                    2024-01-01 10:55:00 INFO Backup completed
                    2024-01-01 11:00:00 ERROR Disk full
                    2024-01-01 11:05:00 INFO Server shutdown
                    2024-01-01 11:10:42 ERROR Service unavailable
                "#),
            ],
            answer: int(7).mistakes(&[("15", "Spočítali jste všechny řádky. Potřebujete jen ty s 'ERROR'. Použijte grep před wc.")]),
            solution: r#"shellgame submit "$(grep ERROR access.log | wc -l)""#,
            success: "Správně! Roura spojí jednoduché příkazy v nástroj, jakým žádný z nich sám o sobě není.",
            ..LEVEL
        },
        Level {
            title: "Začátek a konec souboru",
            text: r#"
                Příkazy `head` a `tail` ukážou jen začátek nebo konec souboru.

                ### Úkol
                V souboru `long_file.txt` je 50 řádků.
                1. Zjistěte první slovo na 1. řádku (pomocí `head -n 1`)
                2. Zjistěte první slovo na posledním řádku (pomocí `tail -n 1`)

                ### Odevzdání
                Odevzdejte obě slova oddělená čárkou, nejdřív slovo z prvního řádku:
                `shellgame submit <slovo1>,<slovo2>`
            "#,
            hints: &[
                "head -n 1 zobrazí první řádek, tail -n 1 zobrazí poslední.",
                "Odevzdejte první slovo z každého z těchto dvou řádků.",
                "Formát odpovědi je 'slovo1,slovo2' - dvě slova oddělená čárkou, bez mezery.",
            ],
            start: "headtail",
            setup: &[
                build(|root| {
                    let text = long_file("Pelican flew over the dam - this is line 1", "Walrus slept on the pier - this is line 50");
                    write(root, "headtail/long_file.txt", text)
                }),
            ],
            answer: parts(",", &[
                exact("Pelican").nocase().error("Slovo z prvního řádku není správné. Použijte 'head -n 1 long_file.txt'."),
                exact("Walrus").nocase().error("Slovo z posledního řádku není správné. Použijte 'tail -n 1 long_file.txt'."),
            ])
                .format("Formát: slovo_z_prvního_řádku,slovo_z_posledního_řádku (např. AHOJ,SVET)"),
            solution: r#"shellgame submit "$(head -n 1 long_file.txt | cut -d' ' -f1),$(tail -n 1 long_file.txt | cut -d' ' -f1)""#,
            success: "Správně! Head a tail jsou skvělé pro rychlý náhled do souborů.",
            ..LEVEL
        },
        Level {
            title: "Počítání (wc)",
            text: r#"
                Příkaz `wc` (word count) počítá řádky, slova a znaky.

                ### Úkol
                Zjistěte o souboru `article.txt`:
                1. Kolik má řádků? (`wc -l`)
                2. Kolik má slov? (`wc -w`)

                ### Odevzdání
                Odevzdejte: `řádky,slova` (např. `10,50`)
                `shellgame submit <řádky>,<slova>`
            "#,
            hints: &[
                "wc -l počítá řádky, wc -w počítá slova.",
                "Spusťte oba příkazy na 'article.txt' a zapište si obě čísla.",
                "Odevzdejte je v pořadí řádky,slova — bez mezery za čárkou.",
            ],
            start: "wc",
            setup: &[
                file("wc/article.txt", r#"
                    Linux je svobodný operační systém.
                    Byl vytvořen Linusem Torvaldsem v roce 1991.
                    Dnes pohání většinu serverů na internetu.
                    Je základem systému Android a mnoha dalších.
                    Open source komunita ho neustále vylepšuje.
                "#),
            ],
            answer: parts(",", &[
                int(5)
                    .mistakes(&[
                        ("31", "První číslo má být počet řádků, ne slov — vypadá to, že máte hodnoty prohozené. Pořadí je řádky,slova."),
                    ])
                    .error("Počet řádků není správný. Použijte 'wc -l article.txt'."),
                int(31)
                    .mistakes(&[
                        ("5", "Druhé číslo má být počet slov, ne řádků. Ten už jste zapsali jako první hodnotu."),
                    ])
                    .error("Počet slov není správný. Použijte 'wc -w article.txt'."),
            ])
                .format("Formát: řádky,slova (např. 10,50) — na pořadí obou čísel záleží."),
            solution: r#"shellgame submit "$(wc -l < article.txt),$(wc -w < article.txt)""#,
            success: "Správně! Příkaz wc je nepostradatelný pro rychlou analýzu souborů.",
            ..LEVEL
        },
        Level {
            title: "Řazení a odstranění duplicit",
            text: r#"
                Příkazy `sort` a `uniq` jsou mocné nástroje pro zpracování textových dat.

                ### Stavební kameny
                - `sort soubor` vypíše řádky seřazeně, takže stejné hodnoty skončí vedle sebe.
                - `uniq` zahodí opakující se řádky, ale pozná jen ty **sousední**.
                - `wc -l` spočítá řádky, které dostane na vstupu.

                Pořadí proto není libovolné: rozmyslete si, co musí `uniq` dostat na vstup, aby fungoval.

                ### Úkol
                V souboru `visitors.txt` jsou jména návštěvníků (někteří přišli vícekrát).
                Zjistěte, kolik je UNIKÁTNÍCH návštěvníků. Příkazy si pospojujte rourami sami.

                ### Odevzdání
                Odevzdejte počet unikátních návštěvníků.
                `shellgame submit <číslo>`
            "#,
            hints: &[
                "Příkaz uniq odstraní duplikáty, ale jen sousedící! Proto nejdřív sort.",
                "Řetězec: sort → uniq → wc -l spočítá unikátní řádky.",
                "Spusťte 'sort visitors.txt | uniq | wc -l' a odevzdejte číslo z výstupu.",
            ],
            start: "sort",
            setup: &[
                file("sort/visitors.txt", r#"
                    Alice
                    Bob
                    Charlie
                    Alice
                    David
                    Bob
                    Eve
                    Alice
                "#),
            ],
            answer: int(5)
                .mistakes(&[
                    ("8", "Spočítali jste všechny řádky. Bez předchozího sort nenajde uniq žádné duplicity, protože stejná jména nejsou vedle sebe."),
                    ("2", "Spočítali jste jen jména, která se opakují (to dělá uniq -d). Hledáme počet všech různých jmen."),
                    ("3", "Spočítali jste jen jména, která přišla jedinkrát (to dělá uniq -u). Hledáme počet všech různých jmen."),
                ]),
            solution: r#"shellgame submit "$(sort visitors.txt | uniq | wc -l)""#,
            success: "Správně! Sort | uniq je klasická kombinace pro práci s daty.",
            ..LEVEL
        },
        Level {
            title: "Výzva: Přesměrování a roury",
            text: r#"
                ### Úkol
                V aktuálním adresáři:

                1. Vytvořte `message.txt` s prvním řádkem `Hello World`.
                2. Přidejte na konec druhý řádek `Goodbye`, aniž by první zmizel.
                3. Propojte výpis obsahu adresáře s počítáním řádků a zjistěte počet položek.

                Vystačíte si s `echo`, `>`, `>>`, `ls`, `|` a `wc -l`.
                Odevzdejte zjištěný počet položek.

                ### Odevzdání
                `shellgame submit <počet>`
            "#,
            hints: &[
                "První přesměrování má soubor vytvořit, druhé musí zachovat jeho obsah. Výpis pak pošlete rourou do počítadla řádků.",
                r#"Soubor vytvoříte pomocí echo "Hello World" > message.txt a druhý řádek přidáte přes '>>'."#,
                "Počet položek zjistíte příkazem 'ls | wc -l'.",
            ],
            start: "challenge",
            setup: &[clean("challenge"), file("challenge/sample1.txt", "sample"), file("challenge/sample2.txt", "sample")],
            answer: int(3).mistakes(&[("2", "Spočítali jste jen sample1.txt a sample2.txt. Vytvořili jste message.txt?")]),
            check: &[
                content_is("challenge/message.txt", r#"
                    Hello World
                    Goodbye
                "#)
                    .msg("message.txt má mít přesně dva řádky, 'Hello World' a pod ním 'Goodbye', každý zakončený novým řádkem (tak je zapíše echo) a nic dalšího. Chybí-li první řádek, přepsalo ho druhé přesměrování '>'.")
                    .missing(r#"Chybí message.txt. Vytvořte ho pomocí echo "Hello World" > message.txt."#),
            ],
            solution: r#"
                echo "Hello World" > message.txt
                echo "Goodbye" >> message.txt
                shellgame submit "$(ls | wc -l)"
            "#,
            success: "Skvělé! Přesměrování i roury máte v malíku!",
            ..LEVEL
        },
        Level {
            title: "Interaktivní vstup a EOF",
            text: r#"
                Když spustíte `cat > soubor`, příkaz čte řádky z klávesnice a zapisuje je do souboru.
                Na prázdném řádku stiskněte **Ctrl+D**: terminál tím oznámí EOF (konec vstupu) a `cat`
                řádně skončí.

                **Ctrl+C** místo toho běžící příkaz přeruší (interrupt). Řádky, které už jste odeslali
                Enterem, `cat` zapsal hned, takže v souboru zůstanou; ztratí se jen rozepsaný řádek.
                Výsledný soubor tak může vypadat stejně – hra proto nepozná, čím jste vstup ukončili.
                Rozdíl je v tom, že Ctrl+D je řádný konec vstupu, kdežto Ctrl+C příkaz násilně ukončí;
                program, který zapisuje až na konci (třeba `sort > soubor`), by po Ctrl+C nezapsal nic.

                ### Úkol
                Spusťte `cat > poznamka.txt` a zadejte přesně tyto dva řádky:

                ```text
                Koupit chleba
                Zavolat mamince
                ```

                Po druhém řádku stiskněte Enter a potom na prázdném řádku Ctrl+D.

                ### Odevzdání
                Po ukončení zápisu pomocí Ctrl+D spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "'cat' bez názvu vstupního souboru čte standardní vstup; EOF mu oznámí, že už žádná data nepřijdou.",
                "Po 'cat > poznamka.txt' napište oba řádky, každý ukončete Enterem. Ctrl+D použijte až na novém prázdném řádku.",
                "Při překlepu spusťte znovu 'cat > poznamka.txt' ('>' soubor přepíše) a zápis zopakujte.",
            ],
            start: "stdin",
            setup: &[clean("stdin/poznamka.txt"), dir("stdin")],
            check: &[
                content_is("stdin/poznamka.txt", r#"
                    Koupit chleba
                    Zavolat mamince
                "#)
                    .msg("Obsah musí přesně odpovídat zadání: řádek 'Koupit chleba', pod ním 'Zavolat mamince', každý ukončený Enterem, a nic dalšího. Zkontrolujte překlepy a mezery; znovu začnete příkazem 'cat > poznamka.txt', který soubor přepíše.")
                    .missing("Soubor poznamka.txt chybí. Začněte příkazem 'cat > poznamka.txt'."),
            ],
            solution: r#"
                printf '%s\n' 'Koupit chleba' 'Zavolat mamince' | cat > poznamka.txt
                shellgame submit
            "#,
            success: "Správně! Soubor obsahuje oba řádky. Ctrl+D vstup řádně ukončí, Ctrl+C příkaz přeruší.",
            ..LEVEL
        },
        Level {
            title: "Návratový kód: && a ||",
            text: r#"
                Každý příkaz skončí návratovým kódem (exit code): **0 znamená úspěch**, nenulová
                hodnota neúspěch. V Bashi i ve fish podle něj můžete spojovat příkazy:

                - `první && druhý` spustí druhý jen po úspěchu prvního,
                - `první || druhý` spustí druhý jen po neúspěchu prvního.

                Příkaz `true` vrací stav 0 (úspěch), `false` nenulový stav (neúspěch).

                ### Úkol
                Spusťte postupně oba řetězce; zapíšou do souboru `status.txt` dva řádky:

                ```bash
                true  && echo "stav: uspech"   ___ status.txt
                false || echo "stav: neuspech" ___ status.txt
                ```

                Na místo `___` doplňte přesměrování `>` nebo `>>`. Rozhodněte se podle toho,
                že první řádek soubor zakládá a druhý se musí přidat za něj — výsledek má mít
                právě dva řádky v tomto pořadí.

                ### Odevzdání
                Až bude soubor hotový, spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Návratový kód 0 značí úspěch, nenulový kód neúspěch. Operátory sledují právě tento stav.",
                "Za '&&' pokračuje úspěšný příkaz; za '||' pokračuje neúspěšný. První řetězec soubor zakládá (stačí přepsání), druhý k němu jen přidává.",
                r#"Spusťte true && echo "stav: uspech" > status.txt a potom false || echo "stav: neuspech" >> status.txt."#,
            ],
            start: "status",
            setup: &[clean("status/status.txt"), dir("status")],
            check: &[
                content_is("status/status.txt", r#"
                    stav: uspech
                    stav: neuspech
                "#)
                    .msg("Soubor musí obsahovat přesně oba řádky ve správném pořadí. Zbyl-li jen jeden řádek, přepsalo druhé přesměrování to první; přebývá-li řádek, první příkaz soubor nepřepsal: použijte u něj '>'.")
                    .missing("Soubor ještě neexistuje. Spusťte oba zadané řetězce příkazů."),
            ],
            solution: r#"
                true && echo "stav: uspech" > status.txt
                false || echo "stav: neuspech" >> status.txt
                shellgame submit
            "#,
            success: "Výborně! Dokončili jste Sekci 9 a umíte reagovat na stav příkazu.",
            ..LEVEL
        },
    ],
};

/// 50 lines: `first`, 48 lines of filler, `last`.
fn long_file(first: &str, last: &str) -> String {
    let middle: Vec<String> =
        (2..50).map(|n| format!("Line number {n} with some content")).collect();
    format!("{first}\n{}\n{last}\n", middle.join("\n"))
}
