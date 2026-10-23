//! Navigation: knowing where you are and moving around with `pwd`, `ls` and `cd`.
//!
//! **Purpose.** Everything else happens somewhere in the filesystem: the section builds the picture
//! (tree, working directory, paths) and the commands that read and change it.
//!
//! **Learning objectives.** After this section a student can:
//! - find the working directory with `pwd` and report its name (the last path component);
//! - tell directories from files in `ls -l` (`d` or `-`) and look options up in `--help` or `man`;
//! - move relatively (`cd dir`, `cd ..`, `cd ../../..`, long paths with Tab) and through a maze;
//! - use absolute paths: one `cd /...` jump, `/` to `$HOME` segment by segment, home with `cd ~`;
//! - tell a file's name from its extension (everything after the last dot).
//!
//! **Introduces:** `pwd`, `ls`, `ls -l`, `--help`, `man`, `cd` (`..`, `/`, `~`, bare),
//! `echo $HOME`, Tab completion, command history, `shellgame reset` (in the maze).\
//! **Relies on:** [welcome](super::s00_welcome) (`shellgame submit`, `shellgame hint`).\
//! **Leaves for later:** file contents (`cat`) and `cd -` ([files](super::s02_files)), hidden
//! entries ([hidden](super::s03_hidden)), creating and deleting ([create](super::s04_create)),
//! wildcards ([wildcards](super::s07_wildcards)).

use std::fs;
use std::io;
use std::path::Path;

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Navigace",
    intro: r#"
        Vítejte v první sekci ShellGame! Zde se naučíte základní dovednosti pro pohyb v příkazové řádce. Prvním krokem k ovládnutí příkazové řádky je pochopení, jak funguje souborový systém vašeho počítače.

        ### Stromová struktura

        Souborový systém vašeho počítače je organizován jako stromová struktura adresářů (složek) a souborů. Strom začíná v kořenovém adresáři (root, označeném `/`). Každý adresář může obsahovat další podadresáře, což vytváří hierarchii.

        ```
        /                          ← Kořen (root) - začátek všeho
        ├── home/                  ← Domovské adresáře uživatelů
        │   └── student/           ← VÁŠ domovský adresář (~)
        │       ├── dokumenty/
        │       ├── stažené/
        │       └── projekty/
        ├── tmp/                   ← Dočasné soubory
        │   └── shellgame-<user>/  ← Váš pracovní prostor pro hru
        │       └── level-1/       ← ZDE ZAČÍNÁTE!
        ├── etc/                   ← Systémové konfigurace
        └── usr/                   ← Programy a knihovny
        ```

        ### Pracovní adresář (working directory)

        V každé chvíli se v terminálu nacházíte v určitém adresáři, který se nazývá "pracovní adresář" (working directory). Je to stejné, jako když máte v grafickém rozhraní otevřenou složku a vidíte její obsah. V terminálu však musíte vědět, kde jste, abyste mohli správně zadávat příkazy.

        Představte si strom z obrázku výše a na něm šipku, která vždy ukazuje na váš aktuální pracovní adresář. Pokaždé, když se příkazem `cd` přesunete jinam, šipka se posune s vámi.

        ### Cesta (Path)

        Cesta (path) je textový řetězec popisující umístění souboru nebo adresáře v systému. Cesta se skládá z jednotlivých adresářů oddělených lomítky (např. `/home/student/dokumenty`).

        ---

        ### Klíčové koncepty

        V této sekci se zaměříme na tři klíčové dovednosti:
        1. **Orientace**: Jak zjistit, kde se právě nacházíte. V grafickém rozhraní to vidíte v záhlaví okna, v terminálu se musíte zeptat.
        2. **Průzkum**: Jak zjistit, co se nachází ve vašem okolí. Naučíte se vypisovat obsah adresářů a rozpoznávat různé typy položek.
        3. **Pohyb**: Jak se přesouvat mezi adresáři. Naučíte se chodit "dovnitř" do podadresářů, vracet se "zpět" a skákat na konkrétní místa.

        ### Příkazy, které si osvojíte

        - `pwd` (Print Working Directory) - „Kde jsem?“
        - `ls` (List) - „Co tu je?“
        - `<příkaz> --help` - stručná nápověda k příkazu
        - `man <příkaz>` - podrobný manuál
        - `cd` (Change Directory) - „Jdi tam!“

        ### Dva triky pro pohodlné psaní
        - **Klávesa Tab (doplňování)**: Napište začátek jména a stiskněte Tab. Shell zbytek doplní sám, což vás uchrání od překlepů.
        - **Šipky nahoru/dolů (historie)**: Stiskem šipky ↑ vyvoláte předchozí příkaz, aniž byste ho museli psát znovu.

        Jste připraveni začít svou cestu?
    "#,
    levels: &[
        Level {
            title: "Aktuální umístění",
            text: r#"
                ### Cíl
                Zjistěte název aktuálního adresáře.

                ### Příkazy
                - `pwd` - vypíše celou cestu k aktuálnímu adresáři

                ### Úkol
                1. Spusťte `pwd`
                2. Odevzdejte název posledního adresáře v cestě (basename)

                ### Příklad
                Cesta: `/home/student/dokumenty` -> Odpověď: `dokumenty`

                ### Odevzdání
                `shellgame submit <název>`
            "#,
            hints: &[
                "Příkaz 'pwd' (Print Working Directory) vám ukáže, kde jste. Zkuste ho!",
                "Podívejte se na výstup 'pwd'. Co je za posledním lomítkem?",
                "Basename je poslední část cesty. Z cesty '/home/student/dokumenty' byste odevzdali 'dokumenty'.",
            ],
            answer: exact("level-1"),
            check: &[
                pwd_used("Nejdřív prosím použijte 'pwd' (ShellGame sleduje použití příkazu) a pak odpověď odevzdejte znovu."),
            ],
            solution: r#"
                pwd
                shellgame submit "$(basename "$PWD")"
            "#,
            success: "Správně! 'pwd' vypisuje celou cestu; poslední část za lomítkem je adresář, ve kterém stojíte.",
            ..LEVEL
        },
        Level {
            title: "Soubory, nebo adresáře?",
            text: r#"
                ### Cíl
                Nejprve si prohlédněte běžný výpis. Potom v nápovědě zjistěte, jak zobrazit
                podrobnosti, a rozlište soubor od adresáře bez spoléhání na barvy terminálu.

                ### Příkazy
                - `ls` - vypíše názvy položek v aktuálním adresáři
                - `ls --help` - zobrazí stručnou nápovědu k příkazu `ls`
                - `man ls` - otevře podrobný manuál; ukončíte ho klávesou `q`

                ### Úkol
                1. Spusťte nejprve obyčejné `ls`
                2. V nápovědě najděte přepínač pro dlouhý (podrobný) výpis a použijte ho
                3. V podrobném výpisu značí první znak `d` adresář a `-` běžný soubor
                4. Najděte **adresář**, jehož název začíná na `d` a končí na `a`
                5. Odevzdejte jeho název

                ### Odevzdání
                `shellgame submit <hodnota>`
            "#,
            hints: &[
                "Běžný výpis ukazuje hlavně názvy. Typ položky na něm poznáte jen někdy (podle barvy nebo lomítka za názvem, to záleží na shellu); spolehlivě ho ukáže podrobný výpis.",
                "Zjistěte v nápovědě k příkazu 'ls', který přepínač zapíná dlouhý (podrobný) výpis.",
                "Otevřete nápovědu pomocí 'ls --help' nebo 'man ls' a hledejte popis 'use a long listing format' (v české verzi 'vypíše výstup ve dlouhém formátu').",
                "Spusťte 'ls -l': řádek adresáře začíná 'd', řádek běžného souboru '-'.",
            ],
            setup: &[
                clean("data"), clean("dog"), clean("dome"),
                dir("alpha"), dir("delta"), dir("dome"), dir("gamma"),
                file("data", ""), file("dog", ""),
            ],
            answer: exact("delta")
                .mistakes(&[
                    ("data", "'data' odpovídá názvem, ale není to adresář: jeho řádek v 'ls -l' začíná '-'."),
                    ("dome", "'dome' je adresář, ale jeho název nekončí na 'a'."),
                    ("dog", "'dog' začíná na 'd', ale nekončí na 'a'. Hledáme vzor d...a."),
                    ("alpha", "'alpha' končí na 'a', ale nezačíná na 'd'. Hledáme vzor d...a."),
                    ("gamma", "'gamma' končí na 'a', ale nezačíná na 'd'. Hledáme vzor d...a."),
                    ("Delta", "Shell rozlišuje velká a malá písmena. Odevzdejte název přesně tak, jak ho vypisuje 'ls'."),
                ]),
            solution: r#"shellgame submit "$(ls -l | grep '^d' | grep -o '[^ ]*$' | grep '^d.*a$')""#,
            success: "Správně! Podle prvního znaku výpisu 'ls -l' jste rozlišili adresář od souboru.",
            ..LEVEL
        },
        Level {
            title: "Název bez přípony",
            text: r#"
                ### Cíl
                Zjistěte název souboru bez přípony.

                ### Příkazy
                - `cd <název>` - změnit adresář

                ### Úkol
                1. Jděte do `alpha` (`cd alpha`)
                2. Najděte soubor uvnitř (`ls`)
                3. Odevzdejte název souboru **BEZ** přípony (vynechte poslední tečku a část za ní)
            "#,
            hints: &[
                "Použijte 'cd alpha' pro vstup do adresáře alpha.",
                "Vypište obsah pomocí 'ls'. Uvidíte soubor s příponou '.txt'.",
                "Odevzdejte název tohoto souboru, ale vynechejte část '.txt'.",
                "Příklad: Pokud je soubor 'data.csv', odevzdejte 'data'.",
            ],
            setup: &[file("alpha/inside.txt", "first step")],
            answer: exact("inside")
                .mistakes(&[
                    ("alpha", "'alpha' je adresář, ve kterém stojíte. Odevzdejte název souboru, který v něm vypíše 'ls', bez přípony."),
                    ("inside.txt", "Odevzdejte název souboru bez přípony: vynechte poslední tečku a část za ní."),
                    ("txt", "To je právě přípona. Odevzdejte tu část názvu, která je před poslední tečkou."),
                    (".txt", "To je právě přípona. Odevzdejte tu část názvu, která je před poslední tečkou."),
                    ("data", "'data' byl jen příklad z nápovědy. Odevzdejte název souboru, který leží v adresáři alpha."),
                ]),
            check: &[at("alpha")],
            solution: r#"
                cd alpha
                shellgame submit "$(ls | sed 's/\.[^.]*$//')"
            "#,
            success: "Správně! Přípona je jen část názvu za poslední tečkou; to, co zbude, je název souboru bez přípony.",
            ..LEVEL
        },
        Level {
            title: "Návrat na základnu",
            text: r#"
                ### Cíl
                Vraťte se o úroveň výše.

                ### Příkazy
                - `cd ..` - jít o úroveň výše

                ### Úkol
                1. Jděte do nadřazeného adresáře (`cd ..`)
                2. Odevzdejte název tohoto adresáře
            "#,
            hints: &[
                "Dvě tečky '..' označují nadřazený (rodičovský) adresář.",
                "Použijte příkaz 'cd ..' pro přesun o jednu úroveň výše.",
                "Spusťte 'pwd' a odevzdejte jen název posledního adresáře (část za posledním '/').",
            ],
            start: "alpha",
            setup: &[file("alpha/inside.txt", "first step")],
            answer: exact("level-1"),
            check: &[at(""), cd_done("Použijte pro návrat přesně příkaz 'cd ..'.")],
            cd: &[cd::exact("..", "Použijte přesně příkaz 'cd ..'.")],
            solution: r#"
                cd ..
                shellgame submit "$(basename "$PWD")"
            "#,
            success: "Správně! '..' neodkazuje na konkrétní jméno, ale vždy na rodiče toho adresáře, kde právě jste.",
        },
        Level {
            title: "Hluboký ponor",
            text: r#"
                ### Cíl
                Sestupte hluboko do adresářové struktury.

                ### 💡 Tip: Klávesa Tab (doplňování názvů)
                Dlouhé cesty nemusíte vypisovat celé písmeno po písmenu!
                Napište `cd g` a stiskněte **Tab** — shell název `gamma/` doplní za vás.
                Pak napište `d` a znovu stiskněte **Tab**. Klávesa Tab je v terminálu váš největší pomocník proti překlepům.

                ### Úkol
                1. Začínáte v adresáři `level-1`
                2. Jděte do `gamma/deep/a/b/c/` (v adresáři `level-1`) — vyzkoušejte klávesu Tab!
                3. V cíli spusťte pouze `shellgame submit`
            "#,
            hints: &[
                "Použijte 'cd' pro vstup do adresářů. Vyzkoušejte klávesu Tab pro automatické doplňování!",
                "Můžete jít postupně: cd gamma, cd deep, cd a...",
                "Nebo najednou: cd gamma/deep/a/b/c (napište 'cd g' a stiskněte Tab)",
                "Po přesunu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
            ],
            setup: &[dir("gamma/deep/a/b/c")],
            check: &[at("gamma/deep/a/b/c")],
            solution: r#"
                cd gamma/deep/a/b/c
                shellgame submit
            "#,
            success: "Správně! Lomítka spojují kroky do jedné relativní cesty — jeden 'cd' zvládne celou větev.",
            ..LEVEL
        },
        Level {
            title: "Víceúrovňový výstup",
            text: r#"
                ### Cíl
                Vystoupejte o více úrovní najednou.

                ### Příkazy
                - `cd ../../..` - jít o 3 úrovně výše

                ### Úkol
                1. ShellGame vás na začátku umístí do správného adresáře (nemusíte řešit, kde jste skončili minule).
                2. Vraťte se o 3 úrovně výše **JEDNÍM** příkazem
                3. V cíli spusťte pouze `shellgame submit`
            "#,
            hints: &[
                "Cesty lze řetězit: každé '..' představuje posun o jednu úroveň nahoru.",
                "Pro posun o tři úrovně najednou spojte tři segmenty: 'cd ../../..'.",
                "Polohu si ověřte příkazem 'pwd'. Pokud jste správně, spusťte jen 'shellgame submit'.",
            ],
            start: "gamma/deep/a/b/c",
            setup: &[dir("gamma/deep/a/b/c")],
            check: &[at("gamma/deep"), cd_done("Vraťte se o tři úrovně jedním příkazem 'cd ../../..'.")],
            cd: &[cd::exact("../../..", "Použijte jeden příkaz 'cd ../../..'.")],
            solution: r#"
                cd ../../..
                shellgame submit
            "#,
            success: "Správně! Úspěšně jste vystoupali o 3 úrovně.",
            ..LEVEL
        },
        Level {
            title: "Navigace v bludišti",
            text: r#"
                ### Cíl
                Projděte bludištěm podle instrukcí.

                ### Pravidla
                - Start: `level-1/maze/entry/`
                - Instrukce je v **názvu** souboru `GO_…` (příkaz `ls`)
                - `GO_TO_DIR_x` → `cd x`
                - `GO_UP_N` → jděte pomocí `cd ..` o N úrovní výše
                - `GO_UP_N_THEN_GO_TO_x` → jděte o N úrovní výše a potom do `x`
                - Během bludiště nemůžete odejít mimo `maze/`

                ### Úkol
                1. Ze startu sledujte názvy souborů `GO_…`
                2. Bludiště je záměrně delší: procvičíte opakované `ls`, sestup, návraty i přechod k sourozencům
                3. Jakmile vstoupíte do skutečného cíle, dokončete level příkazem `shellgame submit`

                Pokud se v bludišti ztratíte, příkaz `shellgame reset` vás vrátí zpět na začátek (`maze/entry`).
            "#,
            hints: &[
                "Sledujte pouze názvy souborů začínající na 'GO_'. Vypište je pomocí 'ls'.",
                "U instrukce 'GO_UP_2_THEN_GO_TO_catacombs' se vraťte o dvě úrovně a vstupte do catacombs.",
                "Spojený návrat lze zapsat jako 'cd ../..'; potom pokračujte příkazem 'cd catacombs'.",
                "Vstupujte jen do adresářů, kam vás pošle soubor 'GO_…'; ostatní adresáře jsou odbočky nebo pasti.",
                "Pokud se ztratíte, příkaz 'shellgame reset' vás vrátí na start.",
            ],
            start: "maze/entry",
            setup: &[
                clean("maze"),
                build(|root| build_maze(root, r#"
                    entry GO_TO_DIR_hall | courtyard dungeon
                      hall GO_TO_DIR_nexus | alcove
                        side_door GO_UP_1
                        nexus GO_TO_DIR_passages | sanctum archives
                          rotunda GO_UP_1
                          passages GO_TO_DIR_tunnel | gallery
                            shaft GO_UP_1
                            tunnel GO_TO_DIR_cavern | crevice
                              cavern GO_TO_DIR_depths | grotto abyss
                                depths GO_UP_2_THEN_GO_TO_catacombs echoes.txt
                              catacombs GO_TO_DIR_vault | ossuary
                                crypts GO_UP_1
                                vault GO_TO_DIR_chamber | iron_cell sepulcher
                                  chamber GO_UP_2_THEN_GO_TO_labyrinth relic.txt
                                labyrinth GO_TO_DIR_corridor | dead_end ruins
                                  corridor GO_TO_DIR_shrine | false_exit
                                    mist GO_UP_1
                                    shrine GO_TO_DIR_final | mirage shadow
                                      final GO_TO_DIR_final_v2 draft.txt | decoy_exit
                                        final_v2 GO_TO_DIR_final_final old.bak | abandoned_branch
                                          final_final GO_TO_DIR_opravdu_final_v2_FINAL | fake_end
                                            opravdu_final_v2_FINAL VICTORY.marker
                "#)),
            ],
            check: &[at("maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2/final_final/opravdu_final_v2_FINAL")],
            cd: &[
                cd::within("maze", "Nemůžete opustit bludiště před jeho dokončením."),
                cd::arrive("maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2/final_final/opravdu_final_v2_FINAL"),
            ],
            solution: r#"
                cd hall
                cd nexus
                cd passages
                cd tunnel
                cd cavern
                cd depths
                cd ../../catacombs
                cd vault
                cd chamber
                cd ../../labyrinth
                cd corridor
                cd shrine
                cd final
                cd final_v2
                cd final_final
                cd opravdu_final_v2_FINAL
                shellgame submit
            "#,
            success: "Správně! Prošli jste bludištěm až do skutečného cíle!",
            ..LEVEL
        },
        Level {
            title: "Skok absolutní cestou",
            text: r#"
                ### Cíl
                Použijte absolutní cestu.

                ### Příkazy
                - `cd /cesta` - absolutní cesta (od kořene)

                ### Úkol
                1. Začínáte v `level-1`. Zjistěte celou cestu příkazem `pwd`
                2. Použijte **JEDEN** příkaz `cd` s absolutní cestou do podadresáře `absolute-target`
                3. V cíli spusťte pouze `shellgame submit`
            "#,
            hints: &[
                "Absolutní cesty začínají na /. Použijte 'pwd' pro zobrazení vaší plné cesty.",
                "K celé cestě vypsané příkazem 'pwd' na startu připojte '/absolute-target'.",
                "Po přesunu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
                "Za 'cd' napište celou sestavenou cestu od /. Pokud obsahuje mezery, uzavřete ji do uvozovek.",
            ],
            setup: &[file("absolute-target/PLACEHOLDER.answer", "")],
            check: &[
                at("absolute-target"),
                cd_done("Jste ve správném adresáři, ale zatím to nevypadá, že jste použili absolutní 'cd' (začíná na /). Zkuste to znovu: opusťte adresář a vraťte se absolutní cestou."),
            ],
            cd: &[
                cd::absolute("Musíte použít absolutní cestu (začínající na /)."),
                cd::arrive("absolute-target"),
            ],
            solution: r#"
                cd "$PWD/absolute-target"
                shellgame submit
            "#,
            success: "Správně! Dostali jste se sem absolutní cestou.",
            ..LEVEL
        },
        Level {
            title: "Cesta z kořene domů",
            text: r#"
                ### Cíl
                Zrekonstruujte cestu domů.

                ### Příkazy
                - `cd /` - jít do kořene
                - `echo $HOME` - zobrazit cestu domů

                ### Úkol
                1. Začínáte v `level-1`. Jděte do kořene (`cd /`)
                2. Zjistěte cestu domů (`echo $HOME`)
                3. Jděte domů krok za krokem (každý segment zvlášť)
                4. Nakonec použijte jen `shellgame submit`

                Poznámka: Váš domovský adresář (`$HOME`) leží mimo herní pracovní prostor.
            "#,
            hints: &[
                "Zjistěte cestu k domovu pomocí 'echo $HOME'.",
                "Začněte v / a vstupujte do každého adresáře v cestě jeden po druhém (bez přeskakování).",
                "Tip: pokud je HOME třeba /home/ada, udělejte: cd / ; cd home ; cd ada.",
                "V tomhle levelu neodevzdáváte textovou odpověď – důležitá je správná sekvence 'cd'.",
            ],
            check: &[
                at_home(),
                cd_done("Vypadá to, že jste se domů dostali, ale ShellGame nezaznamenal povinnou sekvenci 'cd' krok za krokem. Začněte znovu: 'cd /' a projděte každý segment cesty do $HOME po jednom."),
            ],
            cd: &[cd::custom(walk_rule, walk_track)],
            solution: r#"
                cd /
                eval "$(echo "$HOME" | tr / '\n' | grep . | sed "s/.*/cd -- '&'/")"
                shellgame submit
            "#,
            success: "Správně! Došli jste domů krok za krokem.",
            ..LEVEL
        },
        Level {
            title: "Zkratka pro domov (~)",
            text: r#"
                ### Cíl
                Naučte se používat zkratku `~` (vlnovku) pro rychlý návrat do domovského adresáře.

                ### Zkratka pro domov: `~` (vlnovka / tilda)
                V předchozím levelu jste šli do domovského adresáře krok za krokem. V běžné práci
                však celou cestu psát nemusíte:
                - Znak `~` (vlnovka) je v shellu zkratka pro váš domovský adresář (`$HOME`).
                - Příkaz `cd ~` vás okamžitě přenese domů, ať se nacházíte kdekoliv v systému.
                - Zkratku lze použít i v cestách, např. `cd ~/dokumenty`.
                - *(Tip: do domovského adresáře vás přenese i samotný příkaz `cd` bez parametrů.)*

                ### 💡 Jak napsat znak `~` na klávesnici
                - **Česká klávesnice v Linuxu**: `Pravý Alt` (AltGr) + `A`.
                - **Česká klávesnice ve Windows** (třeba v terminálu WSL nebo při připojení přes SSH):
                  `Pravý Alt` (AltGr) + klávesa `+` (vlevo nahoře, hned vedle `;`), potom stiskněte mezerník.
                - **Anglická klávesnice**: `Shift` + klávesa pod `Esc` (vlevo nahoře).

                ### Úkol
                1. Začínáte v herním prostoru (`level-1`)
                2. Přejděte do domovského adresáře pomocí zkratky: `cd ~`
                3. Příkazem `pwd` ověřte, kde se nacházíte
                4. V cíli spusťte pouze `shellgame submit`
            "#,
            hints: &[
                "Znak '~' (vlnovka) je zkratka shellu pro domovský adresář ($HOME). Příkaz 'cd ~' vás přenese domů odkudkoliv.",
                "Na české klávesnici napíšete '~' v Linuxu pomocí AltGr + A, ve Windows pomocí AltGr + '+' a potom mezerníku.",
                "Po přesunu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
            ],
            check: &[
                at_home(),
                pwd_used("Jste doma, ale ještě ověřte polohu příkazem 'pwd' a potom odevzdejte znovu."),
            ],
            solution: r#"
                cd ~
                pwd
                shellgame submit
            "#,
            success: "Správně! Shell nahradí '~' na začátku cesty vaším domovským adresářem, ať stojíte kdekoliv. Pozor: uvnitř uvozovek se '~' nerozbalí.",
            ..LEVEL
        },
        Level {
            title: "Souhrn",
            text: r#"
                ### Výzva: Otestujte své dovednosti!

                Ukažte, co jste se naučili v této sekci. Proveďte následující kroky:

                ### Úkol
                1. Začínáte v `level-1`. Zjistěte svou aktuální polohu (`pwd`)
                2. Přejděte do adresáře `gamma/deep/a/b/c`
                3. Vraťte se o 4 úrovně výše jedním příkazem
                4. V cíli spusťte pouze `shellgame submit`

                ### Shrnutí příkazů
                ```
                pwd           → Kde jsem?
                ls            → Co tu je?
                cd adresář    → Vstup do adresáře
                cd ..         → O úroveň výše
                cd ../..      → O více úrovní výše
                cd /cesta     → Absolutní cesta
                cd ~          → Domů
                ```

                ### Odevzdání
                `shellgame submit`
            "#,
            hints: &[
                "Nejdřív si pomocí 'pwd' potvrďte výchozí polohu a cestu rozdělte na jednotlivé adresáře.",
                "Při návratu spočítejte, kolik segmentů musíte z adresáře 'c' odstranit. Každý segment '..' znamená jednu úroveň.",
                "Použijte 'cd gamma/deep/a/b/c' a potom 'cd ../../../..'. Výsledek ověřte pomocí 'pwd'.",
            ],
            setup: &[dir("gamma/deep/a/b/c")],
            check: &[
                at("gamma"),
                pwd_used("Nejdřív použijte 'pwd'."),
                cd_done("Přejděte do 'c' a vraťte se jedním příkazem o čtyři úrovně."),
            ],
            cd: &[
                cd::when_in("gamma/deep/a/b/c"),
                cd::exact("../../../..", "Z adresáře 'c' se vraťte o čtyři úrovně jediným příkazem 'cd': každá úroveň je jeden segment '..' a segmenty se spojují lomítkem."),
            ],
            solution: r#"
                pwd
                cd gamma/deep/a/b/c
                cd ../../../..
                shellgame submit
            "#,
            success: "Výborně! Ovládáte základy navigace!",
            ..LEVEL
        },
    ],
};

const TRAP: &str = "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE";
const VICTORY: &str = "VICTORY.marker";

/// A short note so `cat` is not a blank page, as text for `file_text` (the trap note is a block); an
/// instruction file keeps its name as the instruction.
fn note(name: &str) -> &'static str {
    if name.starts_with("GO_") {
        "Instrukce je v názvu tohoto souboru. Řiďte se jménem, ne obsahem.\n"
    } else if name == TRAP {
        r#"
            Tady nemáte být. Vraťte se a sledujte soubory začínající na GO_.
            Ztratili jste se? Příkaz 'shellgame reset' vás vrátí na začátek bludiště (maze/entry).
        "#
    } else if name == VICTORY {
        "Konečně skutečný cíl (opravdu_final_v2_FINAL)! Level dokončíte příkazem: shellgame submit\n"
    } else {
        ""
    }
}

/// Builds the maze of "Navigace v bludišti" below `maze/` from its `plan`, an indented block with
/// one line per room nested by indentation (two spaces per level): `room file... | dead end...`. A
/// dead end is a room holding only the trap note.
fn build_maze(root: &Path, plan: &str) -> io::Result<()> {
    let mut rooms = vec!["maze".to_owned()];
    for line in dedent(plan).lines() {
        rooms.truncate((line.len() - line.trim_start().len()) / 2 + 1);
        let (room, dead_ends) = line.trim().split_once(" | ").unwrap_or((line.trim(), ""));
        let mut words = room.split(' ');
        rooms.push(words.next().unwrap_or_default().to_owned());
        let dir = rooms.join("/");
        mkdir(root, &dir)?;
        for file in words {
            write(root, &format!("{dir}/{file}"), file_text(note(file)))?;
        }
        for dead_end in dead_ends.split_whitespace() {
            mkdir(root, &format!("{dir}/{dead_end}"))?;
            write(root, &format!("{dir}/{dead_end}/{TRAP}"), file_text(note(TRAP)))?;
        }
    }
    Ok(())
}

/// "Cesta z kořene domů" walks from `/` to `$HOME` one directory at a time. A `cd` that would skip
/// steps is blocked: a bare `cd`, any absolute path but `/` (`cd ~` and `cd $HOME` arrive expanded)
/// and any path with more than one segment.
fn walk_rule(mv: &CdMove) -> Result<(), String> {
    if mv.is_bare() {
        Err("Skoky nejsou povoleny. Jděte krok za krokem.".to_owned())
    } else if mv.target == "/" {
        Ok(())
    } else if mv.target.starts_with('/') {
        Err("Absolutní skoky nejsou povoleny (kromě cd /).".to_owned())
    } else if mv.target.trim_end_matches('/').contains('/') {
        Err("Cestujte po jednom segmentu (adresáři).".to_owned())
    } else {
        Ok(())
    }
}

/// Tracks the walk after each successful move in `.cd_walk`, the directories visited since `/`, one
/// per line; reaching `$HOME` at the end of an unbroken walk records the evidence and stepping aside
/// or up restarts it. Everything compares the logical directories the shell reports, never
/// canonical paths: a symlink to `$HOME` is no shortcut.
fn walk_track(mv: &CdMove) {
    let (ctx, Some(dest)) = (mv.ctx, mv.dest) else { return };
    if dest == Path::new("/") {
        let _ = ctx.mark(".cd_walk", "/");
    } else if let Ok(walked) = fs::read_to_string(ctx.marker(".cd_walk")) {
        let last = Path::new(walked.lines().last().unwrap_or_default());
        if dest.parent() == Some(last) {
            let _ = ctx.mark(".cd_walk", &format!("{walked}\n{}", dest.display()));
            if dest == ctx.home {
                let _ = ctx.mark(CD_DONE, "");
            }
        } else if dest != last {
            let _ = fs::remove_file(ctx.marker(".cd_walk"));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::game::testing::World;

    /// The instruction files of a room, and where each sends the player.
    fn instructions(room: &Path) -> Vec<(String, PathBuf)> {
        let names = fs::read_dir(room)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned());
        let go = |name: &String| {
            if let Some(dir) = name.strip_prefix("GO_TO_DIR_") {
                return room.join(dir);
            }
            let rest = name.strip_prefix("GO_UP_").unwrap();
            let (up, then) = rest.split_once("_THEN_GO_TO_").unwrap_or((rest, ""));
            room.ancestors().nth(up.parse().unwrap()).unwrap().join(then)
        };
        names
            .filter(|name| name.starts_with("GO_"))
            .map(|name| (go(&name), name))
            .map(|(to, name)| (name, to))
            .collect()
    }

    #[test]
    fn every_instruction_leads_to_a_room_and_the_main_route_reaches_the_goal() {
        let pos = Pos::all().find(|pos| pos.level().start == "maze/entry").unwrap();
        let world = World::new();
        let maze = world.start(pos).section_root.join("maze");
        let (mut rooms, mut followed, mut goal) = (vec![maze.clone()], 0, None);
        while let Some(room) = rooms.pop() {
            rooms.extend(
                fs::read_dir(&room)
                    .unwrap()
                    .flatten()
                    .map(|e| e.path())
                    .filter(|path| path.is_dir()),
            );
            for (name, to) in instructions(&room) {
                assert!(to.is_dir() && to.starts_with(&maze), "{name} in {room:?}");
                followed += 1;
            }
            if room.join(VICTORY).is_file() {
                goal = Some(room);
            }
        }
        assert!(followed > 10);
        let (goal, mut room) = (goal.expect("a room holds the victory marker"), maze.join("entry"));
        for _ in 0..40 {
            if room == goal {
                return;
            }
            let go = instructions(&room);
            assert_eq!(go.len(), 1, "one instruction in {room:?}");
            room = go[0].1.clone();
        }
        panic!("the main route does not reach the goal");
    }
}
