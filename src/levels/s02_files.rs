//! Working with files: moving between directories with purpose, and reading and creating files.
//!
//! **Purpose.** Real work means jumping between places and reading files: shortcuts for moving, a
//! first look inside files (`cat`), a first created file (`touch`), and looking commands up.
//!
//! **Learning objectives.** After this section a student can:
//! - reach a sibling or another branch with one relative path (`cd ../finish`), return with `cd -`;
//! - read a file with `cat`, report a word from it, follow a trail of clues to a password, and stop
//!   a command that waits for input with Ctrl+C;
//! - create an empty file with `touch` and check it with `ls`;
//! - search `--help` or `man` (`/`, `n`, `q`) and match a short option to its long name (`-h`).
//!
//! **Introduces:** `cd -`, `cat`, `touch`, Ctrl+C, searching in `--help` and `man`.\
//! **Relies on:** [navigation](super::s01_navigation) (`pwd`, `ls`, `cd`, `..`, relative paths).\
//! **Leaves for later:** hidden files ([hidden](super::s03_hidden)), `mkdir` and deleting
//! ([create](super::s04_create)), copying and moving ([copy_move](super::s06_copy_move)), wildcards
//! ([wildcards](super::s07_wildcards)), writing into files ([redirection](super::s09_redirection)).

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Práce se soubory",
    intro: r#"
        V této sekci se naučíte pracovat se soubory a pokročilejší navigaci.

        ### Proč je to důležité?
        Skutečná práce v terminálu je stálé přebíhání mezi několika místy:
        zdrojový kód a testy, konfigurace a log. Kdo umí jen `cd jmeno`,
        vypisuje pokaždé celou cestu znovu. Po této sekci se mezi dvěma místy
        přepnete jedním krátkým příkazem — a obsah souboru si zobrazíte,
        aniž byste otevírali editor.

        ### Navigace mezi sourozeneckými adresáři
        ```
        level-2/
        ├── start/      ← Jste tady
        ├── finish/     ← Chcete se sem dostat
        └── other/

        Jak se dostat ze 'start' do 'finish'?
        1. Krok zpět: cd ..        (jste v level-2)
        2. Krok dovnitř: cd finish (jste ve finish)

        Nebo jedním příkazem: cd ../finish
        ```

        ### Co se naučíte
        - Navigace mezi sousedními adresáři (`cd ../jiný`)
        - Rychlý návrat do předchozího adresáře (`cd -`)
        - Čtení obsahu souborů (`cat`)
        - Přerušení běžícího příkazu klávesovou zkratkou (**Ctrl+C**)
        - Vytváření souborů (`touch`)

        ### Proč je `cd -` užitečný?
        Když často přepínáte mezi dvěma místy (např. zdrojový kód a testy),
        `cd -` vás vrátí tam, kde jste byli naposledy - bez psaní celé cesty.

        ### Pokračování
        Pro zahájení prvního levelu této sekce stiskněte Enter.
    "#,
    levels: &[
        Level {
            title: "Navigace mezi sourozenci",
            text: r#"
                ### Cíl
                Přejděte z aktuálního podadresáře do sousedního (sourozeneckého) adresáře.

                ### Struktura adresářů
                ```
                level-2/
                ├── start/       ← Zde se nacházíte (start)
                └── finish/      ← Váš cíl
                ```

                ### Navigace mezi sourozenci
                Adresáře `start` a `finish` leží vedle sebe ve stejném rodičovském adresáři (`level-2`).
                Protože `finish` neleží uvnitř `start`, nelze použít přímý příkaz `cd finish`.
                Musíte se nejprve vrátit k rodiči (`..`) a odtud vstoupit do cíle:
                - Ve dvou krocích: nejprve o úroveň výše (`cd ..`) a potom do cíle.
                - Nebo v jednom kroku spojenou relativní cestou: `cd ../<cílový_adresář>`.

                ### Úkol
                1. Začínáte v adresáři `level-2/start`
                2. Přejděte do sousedního adresáře `finish`
                3. V cíli ověřte polohu příkazem `pwd` a odešlete řešení

                ### Odevzdání
                V cílovém adresáři spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Do sourozeneckého adresáře se dostanete přes společného rodiče ('..').",
                "Zkuste nejprve vystoupat o úroveň výše nebo použít spojenou relativní cestu začínající '../'.",
                "Po přesunu ověřte polohu příkazem 'pwd' a spusťte 'shellgame submit'.",
            ],
            start: "start",
            setup: &[dir("start"), dir("finish")],
            check: &[at("finish")],
            solution: r#"
                cd ../finish
                shellgame submit
            "#,
            success: "Správně! K sourozenci se chodí přes společného rodiče — nahoru a hned dolů jiným směrem.",
            ..LEVEL
        },
        Level {
            title: "Rychlý návrat",
            text: r#"
                ### Cíl
                Naučte se rychle přepínat mezi dvěma adresáři.

                ### Příkazy k naučení
                - `cd -` (návrat do předchozího adresáře)

                ### Úkol
                1. Začínáte v `level-2/location-A`.
                2. Přejděte do `level-2/location-B` (použijte `cd ../location-B`).
                3. Použijte příkaz `cd -` pro okamžitý návrat zpět do `location-A`.
                4. Po návratu spusťte pouze `shellgame submit`.
            "#,
            hints: &[
                "Příkaz 'cd -' vás vrátí do předchozího pracovního adresáře (jako tlačítko Zpět).",
                "Nejprve přejděte do 'location-B' ('cd ../location-B') a odtud zadejte 'cd -'.",
                "Po návratu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
            ],
            start: "location-A",
            setup: &[dir("location-A"), dir("location-B")],
            check: &[at("location-A"), cd_done("Nejdřív přejděte do 'location-B' a vraťte se příkazem 'cd -'.")],
            cd: &[
                cd::when_in("location-B"),
                cd::exact("-", "Z adresáře location-B se vraťte příkazem 'cd -'."),
                cd::arrive("location-A"),
            ],
            solution: r#"
                cd ../location-B
                cd -
                shellgame submit
            "#,
            success: "Správně! Shell si pamatuje předchozí adresář, takže 'cd -' přepíná mezi dvěma místy bez cesty.",
            ..LEVEL
        },
        Level {
            title: "Hluboká navigace",
            text: r#"
                ### Cíl
                Navigace složitější strukturou pomocí relativních cest.

                ### Úkol
                Nacházíte se hluboko ve struktuře adresářů.
                Vaším cílem je přejít do jiné větve stromu pomocí jediného příkazu `cd`.

                - Start: `.../level-2/deep/structure/start`
                - Cíl: `.../level-2/deep/other/target`

                Musíte jít o dvě úrovně výše a pak dolů do `other/target`.

                V cíli spusťte pouze `shellgame submit`.

                Potřebujete pomoc? Napište: `shellgame hint`
            "#,
            hints: &[
                "Pro přechod do jiné větve stromu musíte nejprve vystoupat nahoru přes '..' a pak sestoupit dolů.",
                "Ze 'start' vystoupejte o dvě úrovně ('../..') a zadejte 'cd ../../other/target'.",
                "Po přesunu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
            ],
            start: "deep/structure/start",
            setup: &[dir("deep/structure/start"), dir("deep/other/target")],
            check: &[
                at("deep/other/target"),
                cd_done("Použijte ze startu jeden relativní příkaz 'cd ../../other/target'."),
            ],
            cd: &[cd::exact("../../other/target", "Použijte ze startu jeden příkaz 'cd ../../other/target'.")],
            solution: r#"
                cd ../../other/target
                shellgame submit
            "#,
            success: "Správně! Jedna relativní cesta umí obsahovat výstup i sestup — nejdřív '..', potom jména větve.",
            ..LEVEL
        },
        Level {
            title: "Čtení souboru",
            text: r#"
                ### Cíl
                Přečtěte si obsah souboru.

                ### Příkazy k naučení
                - `cat <soubor>` (vypíše obsah souboru do terminálu)

                ### ⚠️ Bezpečnostní pojistka: Ctrl+C
                Kdybyste omylem spustili `cat` bez názvu souboru, příkaz neví, co číst,
                a začne čekat na vstup z klávesnice (terminál se zdánlivě „zasekne“).
                Kdykoliv se vám to stane, stiskněte **Ctrl+C** — to běžící příkaz okamžitě přeruší.

                ### Úkol
                1. V aktuálním adresáři je soubor `message.txt`.
                2. Přečtěte si jeho obsah pomocí `cat message.txt`.
                3. Odevzdejte **PRVNÍ SLOVO**, které v souboru najdete.

                Odevzdejte pomocí: `shellgame submit <slovo>`

                Potřebujete pomoc? Napište: `shellgame hint`
            "#,
            hints: &[
                "Příkaz 'cat' vypíše obsah textového souboru na obrazovku.",
                "Spusťte 'cat message.txt' pro zobrazení obsahu souboru.",
                "Z výstupu vezměte jen první slovo (před první mezerou) a zadejte: 'shellgame submit <slovo>'.",
            ],
            setup: &[file("message.txt", "Secret is the key.\n")],
            answer: exact("secret").nocase(),
            solution: r#"shellgame submit "$(cut -d' ' -f1 message.txt)""#,
            success: "Správně! 'cat' vysype celý obsah souboru do terminálu — ideální na krátké textové soubory.",
            ..LEVEL
        },
        Level {
            title: "Sledování stop",
            text: r#"
                ### Cíl
                Sledujte stopy v souborech k nalezení hesla.

                ### Úkol
                1. Přečtěte si soubor `start.txt` v aktuálním adresáři.
                2. Postupujte podle instrukcí v souboru.
                3. Najděte finální heslo a odevzdejte ho.

                Odevzdejte pomocí: `shellgame submit <heslo>`

                Potřebujete pomoc? Napište: `shellgame hint`
            "#,
            hints: &[
                "Začněte čtením start.txt - co vám soubor říká?",
                "Instrukce vás posílají někam dál. Jaký příkaz použijete pro přesun do adresáře?",
                "Přečtěte start.txt, přejděte do adresáře 'next', přečtěte clue.txt.",
            ],
            setup: &[
                file("start.txt", "Jděte do adresáře 'next' a přečtěte si clue.txt\n"),
                file("next/clue.txt", "Heslo je 'sunshine'\n"),
            ],
            answer: exact("sunshine").nocase(),
            solution: r#"
                cat start.txt
                cd next
                shellgame submit "$(cut -d"'" -f2 clue.txt)"
            "#,
            success: "Správně! Střídat 'ls', 'cd' a 'cat' stačí k prozkoumání libovolné neznámé struktury.",
            ..LEVEL
        },
        Level {
            title: "Vytvoření souboru",
            text: r#"
                ### Cíl
                Vytvořte nový prázdný soubor.

                ### Příkazy k naučení
                - `touch <název>` (vytvoří prázdný soubor nebo aktualizuje časové značky existujícího souboru)

                ### Úkol
                1. Vytvořte soubor s názvem `my_file.txt` v aktuálním adresáři.
                2. Ověřte jeho existenci pomocí `ls`.

                Odevzdejte pomocí: `shellgame submit`

                Potřebujete pomoc? Napište: `shellgame hint`
            "#,
            hints: &[
                "Příkaz 'touch' vytvoří prázdný soubor se zadaným názvem.",
                "Spusťte 'touch my_file.txt' v aktuálním adresáři.",
                "Ověřte vytvoření souboru příkazem 'ls' a odešlete: 'shellgame submit'.",
            ],
            setup: &[clean("my_file.txt")],
            check: &[is_file("my_file.txt")],
            solution: r#"
                touch my_file.txt
                shellgame submit
            "#,
            success: "Správně! 'touch' založí prázdný soubor; u existujícího jen posune časové značky.",
            ..LEVEL
        },
        Level {
            title: "Jak najít pomoc",
            text: r#"
                ### Cíl
                Naučte se vyhledávat v nápovědě k příkazům.

                ### Dva způsoby, jak získat pomoc
                Nikdo si nepamatuje všechny přepínače všech příkazů:
                1. `<příkaz> --help` → stručný přehled přepínačů přímo v terminálu
                2. `man <příkaz>` → podrobný manuál (stránkovaný; ukončíte ho klávesou `q`)

                ### 💡 Hledání v manuálu (`man`)
                V manuálu můžete vyhledávat:
                - Stiskněte klávesu `/`, napište hledaný text (např. `-h`) a stiskněte **Enter**.
                - Klávesou `n` přejdete na další výskyt.
                - Klávesou `q` manuál ukončíte.

                ### Úkol
                1. Spusťte `ls --help` nebo otevřete manuál `man ls`.
                2. Vyhledejte přepínač `-h`.
                3. Krátké přepínače mívají svůj dlouhý ekvivalent začínající na `--` (např. `-a` má `--all`).
                   Jaký dlouhý název má přepínač `-h`?
                4. Odevzdejte tento dlouhý název, s úvodními pomlčkami nebo bez nich
                   (pro přepínač `-a` byste odevzdali `all` nebo `--all`).

                ### Odevzdání
                `shellgame submit <název>`
            "#,
            hints: &[
                "Spusťte 'ls --help' nebo 'man ls' a vyhledejte řádek s přepínačem '-h'.",
                "V nápovědě uvidíte zápis ve tvaru '-h, --název'. Hledejte slovo za dvěma pomlčkami.",
                "V 'man ls' napište '/-h,' a stiskněte Enter: manuál skočí na řádek, který začíná '-h,'. Za čárkou na něm stojí dlouhý název začínající '--'; odevzdejte celé slovo za pomlčkami, i se spojovníkem uprostřed.",
            ],
            answer: one_of(&[
                "human-readable",
                "--human-readable",
                "human readable",
                "human",
                "čitelné",
                "citelne",
                "čitelné formátování",
            ])
                .nocase()
                .error("Odpověď není správná. V nápovědě vyhledejte řádek s přepínačem '-h' a najděte jeho dlouhý název (--...)."),
            solution: r#"shellgame submit "$(ls --help | grep -o -e '-h, --[a-z-]*' | cut -d' ' -f2)""#,
            success: "Správně! Teď víte, jak najít pomoc: přepínač '--help' a příkaz 'man' jsou vaši nejlepší přátelé.",
            ..LEVEL
        },
        Level {
            title: "Integrační výzva",
            text: r#"
                ### Výzva: Propojte navigaci a čtení souborů

                Závěrečný úkol sekce: použijete v něm několik dovedností najednou.

                ### Úkol
                1. Začínáte v `level-2`. Přejděte do `challenge/room1`.
                2. Přečtěte soubor `hint.txt`.
                3. Podle stopy přejděte do `room2`.
                4. Přečtěte `password.txt` a odevzdejte nalezené heslo.

                ### Odevzdání
                `shellgame submit <hodnota>`
            "#,
            hints: &[
                "Ze startu přejděte do 'challenge/room1' a přečtěte 'hint.txt'.",
                "Do sousedního 'room2' se dostanete například příkazem 'cd ../room2'.",
                "V 'room2' přečtěte soubor 'password.txt' pomocí 'cat'.",
            ],
            setup: &[
                file("challenge/room1/hint.txt", "Heslo je v sousedním adresáři room2.\n"),
                file("challenge/room2/password.txt", "navigator\n"),
                file("challenge/room2/decoy.txt", "Toto není heslo.\n"),
            ],
            answer: exact("navigator")
                .nocase()
                .mistakes(&[
                    ("toto není heslo.", "To je obsah decoy.txt, ne password.txt. Přečtěte správný soubor."),
                    ("toto není heslo", "To je obsah decoy.txt, ne password.txt. Přečtěte správný soubor."),
                    ("toto neni heslo.", "To je obsah decoy.txt, ne password.txt. Přečtěte správný soubor."),
                    ("toto neni heslo", "To je obsah decoy.txt, ne password.txt. Přečtěte správný soubor."),
                ])
                .error("Heslo není správné. Hledejte password.txt v room2."),
            check: &[at("challenge/room2")],
            solution: r#"
                cd challenge/room1
                cat hint.txt
                cd ../room2
                shellgame submit "$(cat password.txt)"
            "#,
            success: "Výborně! Dokončili jste Sekci 2 a umíte propojit navigaci se čtením souborů.",
            ..LEVEL
        },
    ],
};
