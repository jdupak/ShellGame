//! Permissions: reading modes in `ls -l` and changing them with `chmod`, for files and directories.
//!
//! **Purpose.** Permissions explain most "Permission denied" messages. The section ties the
//! notation (rwx, symbolic and octal) to effects the student predicts and then observes, for
//! directories too.
//!
//! **Learning objectives.** After this section a student can:
//! - read the mode string of `ls -l` and find the file that its owner may run;
//! - add or remove permissions symbolically (`chmod u+x`, `a-w`, `g+r`) and set an exact mode in
//!   octal (`chmod 755`, `644`);
//! - predict and verify that creating a file needs `w` on the directory, not on the file;
//! - predict and verify that `x` on a directory allows entering it (`chmod a-x ..` breaks `cd ..`).
//!
//! **Introduces:** `chmod` (symbolic and octal), `r`/`w`/`x` on files and on directories.\
//! **Relies on:** [inspect](super::s05_inspect) (`ls -l`), [copy_move](super::s06_copy_move)
//! (`cp`).\
//! **Leaves for later:** running scripts (shown in examples only; [streams](super::s10_streams)
//! runs prepared ones); not in the course: ownership, special bits, `umask` (named once).

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Oprávnění",
    intro: r#"
        V Linuxu má každý soubor a adresář nastavená oprávnění, která určují, kdo s ním může co dělat.

        ### Proč je to důležité?

        ### Bezpečnost systému
        - **Hesla**: Soubor `/etc/shadow` obsahuje hesla - smí ho číst jen root!
        - **Konfigurace**: Webový server nesmí měnit vlastní konfiguraci (jen číst)
        - **Sdílení**: Spolužáci nevidí vaše soukromé soubory v domovském adresáři

        ### Běžné situace
        ```bash
        # "Permission denied" při spuštění skriptu?
        $ ./muj_skript.sh
        bash: ./muj_skript.sh: Permission denied
        $ chmod u+x muj_skript.sh   # Přidá spuštění pro vlastníka
        $ ./muj_skript.sh
        Hello World!

        # Webový server nevidí soubory?
        $ chmod 644 index.html     # Ostatní mohou číst
        ```

        ### Tři typy oprávnění
        ```
        r (read)     → Číst obsah souboru / vypsat obsah adresáře
        w (write)    → Měnit obsah souboru / vytvářet a mazat položky v adresáři
        x (execute)  → Spustit jako program / vstoupit do adresáře
        ```

        Smazání souboru závisí na právech `w` a `x` nadřazeného adresáře,
        ne na právu `w` samotného souboru.

        ---

        ### Tři skupiny uživatelů
        ```
        u (user)   → Vlastník souboru (vy)
        g (group)  → Členové skupiny vlastníka
        o (other)  → Všichni ostatní
        ```

        ### Jak číst `ls -l`
        ```
        -rwxr-xr--  =  vlastník: rwx, skupina: r-x, ostatní: r--
        │└┬┘└┬┘└┬┘
        │ u  g  o
        │
        └─ typ (- soubor, d adresář)
        ```

        ### Dva způsoby zápisu chmod
        ```
        Symbolický:              Číselný (oktalový):
        chmod u+x soubor         chmod 755 soubor
        chmod g-w soubor
        chmod o=r soubor         r=4, w=2, x=1
                                 755 = rwx|r-x|r-x
        ```

        ### Co se naučíte
        - Číst oprávnění (`ls -l`)
        - Měnit oprávnění (`chmod`)
        - Používat symbolický zápis (`u+x`) i číselný zápis (`755`)
        - Předvídat a prakticky ověřit, jak `w` a `x` ovlivňují práci s adresáři
    "#,
    levels: &[
        Level {
            title: "Hledání spustitelného souboru",
            text: r#"
                Příkaz `ls -l` zobrazuje oprávnění v prvním sloupci (např. `-rw-r--r--`).
                První trojice práv po znaku typu patří vlastníkovi souboru.

                ### Úkol
                V adresáři je několik skriptů. Najděte jediný, který má právo spuštění (`x`)
                nastavené pro vlastníka. `x` u skupiny nebo ostatních nestačí.

                ### Příkazy
                - `ls -l`: Zobrazí oprávnění

                ### Odevzdání
                Odevzdejte název spustitelného souboru.
                `shellgame submit <soubor>`
            "#,
            hints: &[
                "Podrobný výpis souborů včetně sloupců s oprávněními získáte přepínačem '-l'.",
                "Spusťte 'ls -l' a rozdělte práva po znaku typu na trojice pro vlastníka, skupinu a ostatní.",
                "Hledejte 'x' v první trojici (pozice vlastníka), ne v trojici skupiny nebo ostatních.",
            ],
            start: "executables",
            setup: &[
                file("executables/test.sh", "#!/bin/bash").mode(0o654),
                file("executables/data.sh", "#!/bin/bash").mode(0o645),
                file("executables/script.sh", r#"
                    #!/bin/bash
                    echo Hi
                "#).mode(0o744),
            ],
            answer: suffix("script.sh")
                .error("To není ten soubor. Ve výpisu 'ls -l' čtěte práva po znaku typu po trojicích: vlastník, skupina, ostatní. 'x' u skupiny nebo ostatních se nepočítá."),
            solution: r#"shellgame submit "$(ls -l | grep '^-..x' | grep -o '[^ ]*$')""#,
            success: "Správně! Práva se čtou po trojicích a rozhoduje ta, která odpovídá vaší roli.",
            ..LEVEL
        },
        Level {
            title: "Nastavení spustitelnosti",
            text: r#"
                Aby šel skript spustit (např. `./script.sh`), musí mít nastavené právo `x`.
                Příkaz `chmod` (change mode) mění oprávnění.

                ### Proč je to důležité?
                Když napíšete skript nebo stáhnete program, často není automaticky spustitelný.
                Musíte mu explicitně dát právo ke spuštění - je to bezpečnostní opatření.

                ### Běžný pracovní postup
                1. Napíšete skript: `nano muj_skript.sh`
                2. Pokusíte se spustit: `./muj_skript.sh` → "Permission denied"
                3. Přidáte právo: `chmod u+x muj_skript.sh`
                4. Nyní funguje: `./muj_skript.sh` ✓

                ### Úkol
                Soubor `run_me.sh` nejde spustit. Přidejte mu právo pro spuštění pro vlastníka (`u`).

                ### Příkazy
                - `chmod u+x <soubor>`: Přidá právo spuštění (execute) vlastníkovi (u = user)

                ### Odevzdání
                Po přidání práva spuštění spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Právo 'x' (execute) je potřeba pro spuštění. Jak ho přidáte pro vlastníka (user)?",
                "Syntaxe chmod: chmod kdo+co soubor. 'u' = user, 'x' = execute.",
                "Použijte 'chmod u+x run_me.sh'.",
            ],
            start: "permissions",
            setup: &[
                file("permissions/run_me.sh", r#"
                    #!/bin/bash
                    echo Run me
                "#).mode(0o644),
            ],
            check: &[
                mode_has("permissions/run_me.sh", 0o100)
                    .msg("Vlastník stále nemá právo 'x'. Zkontrolujte 'ls -l run_me.sh': rozhoduje první trojice po znaku typu, ne trojice skupiny ani ostatních."),
            ],
            solution: r#"
                chmod u+x run_me.sh
                shellgame submit
            "#,
            success: "Správně! Symbolický chmod se skládá ze tří částí: kdo, operace a které právo.",
            ..LEVEL
        },
        Level {
            title: "Odebrání práva zápisu",
            text: r#"
                Někdy chcete zabránit nechtěnému přepsání souboru. Můžete mu odebrat právo pro zápis (`w`).

                ### Úkol
                Odeberte souboru `config.readonly` právo zápisu pro všechny: vlastníka, skupinu i ostatní.

                ### Příkazy
                - `chmod a-w <soubor>`: Odebere právo zápisu (write) všem (a = all)

                Písmeno `a` nevynechávejte: bez něj výsledek ovlivňuje výchozí maska práv (`umask`).

                ### Odevzdání
                Po odebrání všech práv zápisu spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "'a' znamená all (všechny: vlastníka, skupinu i ostatní), '-w' odebírá právo zápisu.",
                "Tvar je 'chmod <kdo><operace><právo> <soubor>': tady je kdo = všichni, operace = odebrat a právo = zápis.",
                "Spusťte 'chmod a-w config.readonly'.",
            ],
            start: "permissions",
            setup: &[file("permissions/config.readonly", "Do not touch").mode(0o644)],
            check: &[
                mode_lacks("permissions/config.readonly", 0o222)
                    .msg("Někde právo zápisu zůstalo. V 'ls -l config.readonly' projděte všechny tři trojice po řadě: 'w' nesmí být ani u vlastníka, ani u skupiny, ani u ostatních."),
            ],
            solution: r#"
                chmod a-w config.readonly
                shellgame submit
            "#,
            success: "Správně! Symbolický zápis mění právo jen vybraným trojicím a zbytek nechává být.",
            ..LEVEL
        },
        Level {
            title: "Číselný zápis",
            text: r#"
                Oprávnění lze nastavit i číselně (oktalově).
                - 4 = read (r)
                - 2 = write (w)
                - 1 = execute (x)

                Součet dává kombinaci (např. 7 = 4+2+1 = rwx, 5 = 4+1 = r-x).
                Zadávají se tři čísla: pro vlastníka, skupinu a ostatní.
                Např. `755` znamená `rwx` pro vlastníka, `r-x` pro skupinu, `r-x` pro ostatní.

                ### Úkol
                Nastavte spouštěcímu skriptu `deploy.sh` oprávnění `755` (`rwxr-xr-x`).

                ### Příkazy
                - `chmod 755 <soubor>`

                ### Odevzdání
                Po nastavení přesného režimu spusťte:
                `shellgame submit`
            "#,
            hints: &[
                "Oktalový zápis: 7 = 4+2+1 (rwx) pro vlastníka, 5 = 4+1 (r-x) pro skupinu a ostatní.",
                "Tvar je 'chmod <tři číslice> <soubor>': každá číslice patří jedné trojici v pořadí vlastník, skupina, ostatní a je součtem 4 (r), 2 (w) a 1 (x).",
                "Spusťte 'chmod 755 deploy.sh' a výsledek zkontrolujte pomocí 'ls -l deploy.sh'.",
            ],
            start: "permissions",
            setup: &[
                file("permissions/deploy.sh", r#"
                    #!/bin/sh
                    echo 'Nasazuji aplikaci'
                "#).mode(0o600),
            ],
            check: &[
                mode_is("permissions/deploy.sh", 0o755)
                    .msg("Režim zatím nesedí. Přečtěte si aktuální stav příkazem 'ls -l deploy.sh' a přeložte každou trojici zpět na číslici (r=4, w=2, x=1, '-'=0). Nejčastější chybou je právo zápisu navíc u skupiny nebo ostatních."),
            ],
            solution: r#"
                chmod 755 deploy.sh
                shellgame submit
            "#,
            success: "Správně! Číselný režim nastavuje všechny tři trojice najednou na přesnou hodnotu.",
            ..LEVEL
        },
        Level {
            title: "Kontrolní úkol: Oprávnění souborů",
            text: r#"
                ### Kontrolní úkol: Oprávnění souborů

                Ukažte, že rozumíte oprávněním běžných souborů předtím, než přejdeme k adresářům!

                ### Úkol
                V `level-8/challenge` jsou 4 soubory:

                1. `script.sh` - potřebuje být **spustitelný** vlastníkem
                2. `secret.txt` - odeberte **právo zápisu všem**
                3. `shared.txt` - nastavte oprávnění **644** (rw-r--r--)
                4. `team.txt` - přidejte skupině právo čtení, ostatní práva neměňte

                ### Shrnutí příkazů Sekce 8
                ```
                ls -l            → Zobrazí oprávnění
                chmod u+x soubor → Přidá vlastníkovi právo spuštění
                chmod a-w soubor → Odebere všem právo zápisu
                chmod g+r soubor → Přidá skupině právo čtení
                chmod 755 soubor → Nastaví rwxr-xr-x
                chmod 644 soubor → Nastaví rw-r--r--
                ```

                ### Oktalové oprávnění
                ```
                4 = read    2 = write    1 = execute
                7 = rwx     6 = rw-      5 = r-x     4 = r--
                ```

                ### Odevzdání
                Po splnění všech bodů spusťte `shellgame submit`.
            "#,
            hints: &[
                "U každého souboru určete skupinu uživatelů (u, g, o nebo a), operaci (+, - nebo =) a požadované právo.",
                "První, druhý a čtvrtý úkol lze vyřešit symbolicky; u shared.txt nastavte přesný číselný režim.",
                "Použijte 'chmod u+x script.sh', 'chmod a-w secret.txt', 'chmod 644 shared.txt' a 'chmod g+r team.txt'.",
            ],
            start: "challenge",
            setup: &[
                clean("challenge"),
                file("challenge/script.sh", r#"
                    #!/bin/bash
                    echo 'Hello'
                "#).mode(0o600),
                file("challenge/secret.txt", "Top secret!\n").mode(0o666),
                file("challenge/shared.txt", "Shared content\n").mode(0o777),
                file("challenge/team.txt", "Team notes\n").mode(0o600),
            ],
            check: &[
                mode_has("challenge/script.sh", 0o100).msg("Vlastník stále nemůže spustit script.sh."),
                mode_lacks("challenge/secret.txt", 0o222).msg("secret.txt má stále alespoň jedno právo zápisu."),
                mode_is("challenge/shared.txt", 0o644).msg("shared.txt nemá přesně oprávnění rw-r--r--."),
                mode_is("challenge/team.txt", 0o640).msg("team.txt nemá právo čtení pro skupinu při zachování ostatních práv."),
            ],
            solution: r#"
                chmod u+x script.sh
                chmod a-w secret.txt
                chmod 644 shared.txt
                chmod g+r team.txt
                shellgame submit
            "#,
            success: "Správně! Oprávnění běžných souborů máte pod kontrolou. Nyní se podíváme na adresáře.",
            ..LEVEL
        },
        Level {
            title: "Kdo dovoluje zápis do adresáře?",
            text: r#"
                ### Cíl
                Prakticky ověřte, že vytvoření nové položky řídí oprávnění **adresáře**,
                ne oprávnění kopírovaného souboru.

                Adresář `locked` má režim `550`: lze ho číst a procházet, ale nelze do něj zapisovat.
                Soubor `source.txt` má režim `444`: je jen pro čtení, nikdo do něj nesmí zapisovat.

                ### Předpověď a pozorování
                **Nejdřív předpověď:** než cokoli spustíte, řekněte nahlas nebo si zapište, zda `cp source.txt locked/`
                projde, nebo selže - a proč. Chybová hláška není překážka, ale hlavní výsledek pokusu; přečtěte si ji celou.

                1. Spusťte `cp source.txt locked/` a porovnejte hlášku se svou předpovědí.
                2. Ověřte, že z adresáře `locked` lze kopírovat ven: `cp locked/report.txt recovered.txt`.
                3. Přidejte vlastníkovi adresáře právo zápisu: `chmod u+w locked`.
                4. Zopakujte `cp source.txt locked/`; tentokrát musí uspět, přestože `source.txt` je pořád jen pro čtení.

                ### Vysvětlení
                Co rozhodovalo o možnosti vytvořit `locked/source.txt` — oprávnění
                zdrojového **souboru**, nebo cílového **adresáře**? Odpověď si řekněte sami;
                hra ověří provedené kroky.

                ### Odevzdání
                Až bude kopie v `locked/`, spusťte `shellgame submit`.
            "#,
            hints: &[
                "Právo 'w' na soubor řídí změnu jeho obsahu. Vytvoření nového jména je operace nad adresářem.",
                "Režim 550 nedává vlastníkovi adresáře právo 'w'; příkaz 'chmod u+w locked' ho přidá.",
                "Spusťte 'cp locked/report.txt recovered.txt', 'chmod u+w locked', 'cp source.txt locked/' a potom 'shellgame submit'.",
            ],
            start: "directory-write",
            setup: &[
                clean("directory-write"),
                dir("directory-write/locked").mode(0o550),
                file("directory-write/source.txt", "nová data\n").mode(0o444),
                file("directory-write/locked/report.txt", "existující zpráva\n"),
            ],
            check: &[
                same("directory-write/locked/report.txt", "directory-write/recovered.txt")
                    .missing("Chybí recovered.txt: zkopírujte locked/report.txt ven pod jménem recovered.txt (krok 2)."),
                same("directory-write/source.txt", "directory-write/locked/source.txt")
                    .missing("Po přidání práva zápisu zkopírujte source.txt do locked/."),
                mode_has("directory-write/locked", 0o200).msg("Vlastník adresáře locked nemá právo zápisu. Přidejte ho příkazem 'chmod u+w locked'."),
            ],
            solution: r#"
                cp locked/report.txt recovered.txt
                chmod u+w locked
                cp source.txt locked/
                shellgame submit
            "#,
            success: "Správně! Vytváření a mazání jmen řídí právo zápisu k nadřazenému adresáři.",
            ..LEVEL
        },
        Level {
            title: "Právo x u adresáře",
            text: r#"
                ### Cíl
                Zažijte rozdíl mezi čtením názvů a **průchodem** adresářem.

                Začínáte v `parent/child`. Nadřazený adresář má běžná práva `755`.

                ### Předpověď a pozorování
                **Nejdřív předpověď:** po `chmod a-x ..` si zapište nebo řekněte nahlas, co udělá `cd ..` - a proč.
                Hlášku, kterou shell vypíše, čtěte pozorně: právě ona je odpovědí, ne překážkou.

                1. Příkazem `chmod a-x ..` odeberte všem právo průchodu nadřazeným adresářem.
                2. Zkuste `cd ..` a porovnejte výsledek se svou předpovědí.
                3. Příkazem `chmod u+x ..` vraťte průchod jen vlastníkovi.
                4. Zopakujte `cd ..`, nyní musí uspět.

                ### Vysvětlení
                Které písmeno oprávnění umožňuje vstoupit do adresáře a procházet přes něj?
                Odpověď si řekněte sami; hra ověří provedené kroky.

                ### Odevzdání
                Až budete v adresáři `parent`, spusťte `shellgame submit`.
            "#,
            hints: &[
                "U adresáře neznamená 'x' spuštění programu, ale možnost průchodu cestou.",
                "Po 'chmod a-x ..' nadřazený adresář stále může mít 'r', ale cesta přes něj nefunguje.",
                "Obnovte vlastníkovi průchod příkazem 'chmod u+x ..', přejděte o úroveň výš pomocí 'cd ..' a spusťte 'shellgame submit'.",
            ],
            start: "directory-traverse/parent/child",
            setup: &[
                clean("directory-traverse"),
                dir("directory-traverse/parent").mode(0o755),
                dir("directory-traverse/parent/child").mode(0o755),
            ],
            check: &[
                at("directory-traverse/parent"),
                mode_has("directory-traverse/parent", 0o100).msg("Adresář parent, ve kterém stojíte, nemá právo 'x' pro vlastníka. Přidejte mu ho příkazem 'chmod u+x .'."),
                mode_lacks("directory-traverse/parent", 0o011).msg("Právo 'x' k adresáři parent, ve kterém stojíte, má mít jen vlastník. Skupině a ostatním ho odeberte příkazem 'chmod go-x .', nebo začněte znovu po 'shellgame reset'."),
            ],
            solution: r#"
                chmod a-x ..
                chmod u+x ..
                cd ..
                shellgame submit
            "#,
            success: "Výborně! Dokončili jste Sekci 8 a rozumíte právům souborů i adresářů.",
            ..LEVEL
        },
    ],
};
