//! Hidden files: names that start with a dot, `ls -a`, and working with them like any other file.
//!
//! **Purpose.** A plain `ls` leaves things out, the first real surprise of the shell. The section
//! shows that "hidden" is only a naming convention; `cat` and `cd` work as before.
//!
//! **Learning objectives.** After this section a student can:
//! - explain what makes an entry hidden (a leading dot) and reveal it with `ls -a`;
//! - tell hidden directories from hidden files in `ls -la` and count the directories, leaving out
//!   the entries `.` and `..`;
//! - read a hidden file with `cat` and enter a hidden directory with `cd`;
//! - pick a hidden file by its suffix and give its whole name, leading dot included.
//!
//! **Introduces:** `ls -a`, `ls -la` (type column only), `.` as the current directory.\
//! **Relies on:** [navigation](super::s01_navigation), [files](super::s02_files) (`cat`).\
//! **Leaves for later:** creating and deleting ([create](super::s04_create)), size and permission
//! columns of `ls -l` ([inspect](super::s05_inspect)), wildcards
//! ([wildcards](super::s07_wildcards)).

use crate::game::*;

#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Skryté soubory",
    intro: r#"
        V této sekci odhalíte tajemství skrytých souborů a adresářů.

        ### Proč je to důležité?
        „Chci upravit nastavení editoru, ale nemůžu ho nikde najít.“
        „Projekt má fungovat, ale chybí mu `.env`.“
        „Git se chová divně — kde vůbec ta konfigurace je?“

        Všechny tři odpovědi leží v souborech, které `ls` záměrně neukáže.
        Dokud o nich nevíte, vypadá adresář prázdně — i když prázdný není.

        ### Proč existují skryté soubory?
        V Linuxu soubory začínající tečkou (`.`) jsou "skryté" - nezobrazí se při běžném `ls`.
        Používají se pro konfigurační soubory, které nechcete mít na očích při běžné práci.

        ### Běžné skryté soubory
        ```
        ~/                          ← Váš domovský adresář
        ├── dokumenty/              ← Běžný adresář (viditelný)
        ├── .bashrc                 ← Konfigurace shellu (SKRYTÝ)
        ├── .gitconfig              ← Konfigurace Gitu (SKRYTÝ)
        ├── .config/                ← Konfigurační adresář (SKRYTÝ)
        │   └── nvim/               ← Nastavení editoru
        └── .ssh/                   ← SSH klíče (SKRYTÝ)

        Příkaz 'ls' zobrazí pouze: dokumenty/
        Příkaz 'ls -a' zobrazí VŠE včetně skrytých!
        Příkaz 'ls -la' přidá podrobnosti; adresář poznáte podle 'd' na začátku řádku.
        ```

        ### Co se naučíte
        - Co jsou skryté soubory (začínají tečkou)
        - Jak je zobrazit a rozlišit jejich typ (`ls -a`, `ls -la`)
        - Jak s nimi pracovat (čtení, navigace)

        ### Pokračování
        Pro zahájení prvního levelu této sekce stiskněte Enter.
    "#,
    levels: &[
        Level {
            title: "Počítání skrytých adresářů",
            text: r#"
                ### Cíl
                Najděte a spočítejte skryté adresáře.

                ### Příkazy k naučení
                - `ls -a` zobrazí všechny položky včetně skrytých
                - `ls -la` navíc zobrazí podrobnosti a typ položky

                ### Úkol
                Nacházíte se v adresáři `level-3/hub`.
                1. Použijte `ls -la` pro zobrazení všech položek i jejich typů.
                2. Spočítejte **skryté adresáře**: jejich název začíná tečkou a řádek znakem `d`.
                3. **Důležité:** Do počtu NEZAHRNUJTE speciální adresáře `.` (aktuální) a `..` (nadřazený).

                Odevzdejte počet nalezených skrytých adresářů (číslo).

                Odevzdejte pomocí: `shellgame submit <hodnota>`

                Potřebujete pomoc? Napište: `shellgame hint`
            "#,
            hints: &[
                "Použijte 'ls -la': -a zobrazí skryté položky a -l přidá podrobnosti.",
                "Adresář poznáte podle znaku 'd' na začátku řádku; běžný soubor začíná '-'.",
                "Počítejte skryté názvy v řádcích začínajících 'd', ale vynechte položky '.' a '..'.",
            ],
            start: "hub",
            setup: &[
                clean("hub"),
                dir("hub/.beta"),
                dir("hub/.gamma"),
                dir("hub/visible_dir"),
                file("hub/.config", ""),
                file("hub/visible_file.txt", ""),
            ],
            answer: int(2)
                .mistakes(&[
                    ("3", "Nejspíš počítáte i skrytý soubor '.config' (jeho řádek nezačíná 'd'), nebo viditelný adresář 'visible_dir' (jeho název nezačíná tečkou)."),
                    ("4", "Nejspíš počítáte i položky '.' a '..'. Jsou to speciální odkazy, ne hledané adresáře."),
                    ("5", "Nejspíš počítáte '.', '..' i skrytý soubor. Sledujte první znak řádku a přesný název."),
                ]),
            solution: r#"shellgame submit "$(ls -la | grep -c '^d.* \.[^.]')""#,
            success: "Správně! Skryté položky zobrazí 'ls -a' a typ položky poznáte podle prvního znaku řádku 'ls -l'.",
            ..LEVEL
        },
        Level {
            title: "Čtení skrytého souboru",
            text: r#"
                ### Cíl
                Přečtěte obsah skrytého souboru.

                ### Úkol
                V aktuálním adresáři je skrytý soubor `.secret_config`.
                1. Ověřte jeho existenci pomocí `ls -a`.
                2. Přečtěte jeho obsah pomocí `cat`.
                3. Odevzdejte obsah souboru.

                Odevzdejte pomocí: `shellgame submit <obsah>`

                Potřebujete pomoc? Napište: `shellgame hint`
            "#,
            hints: &[
                "Skryté soubory začínají tečkou. Jak je zobrazíte pomocí ls?",
                "Příkaz 'cat' funguje i na skryté soubory - stačí zadat správný název včetně tečky.",
                "Zkuste: cat .secret_config",
            ],
            setup: &[file(".secret_config", "mode=stealth\n")],
            answer: exact("mode=stealth"),
            solution: r#"shellgame submit "$(cat .secret_config)""#,
            success: "Správně! Tečka na začátku názvu soubor jen skryje ve výpisu, práci s ním nijak neomezuje.",
            ..LEVEL
        },
        Level {
            title: "Uvnitř skrytého adresáře",
            text: r#"
                ### Cíl
                Vstupte do skrytého adresáře.

                ### Úkol
                1. Najděte skrytý adresář `.vault`.
                2. Vstupte do něj (`cd .vault`).
                3. Uvnitř najděte soubor `key.txt` a přečtěte ho.
                4. Odevzdejte nalezený klíč.

                Odevzdejte pomocí: `shellgame submit <klíč>`

                Potřebujete pomoc? Napište: `shellgame hint`
            "#,
            hints: &[
                "Jak byste vstoupili do běžného adresáře? Stejně to funguje i se skrytými.",
                "Skrytý adresář .vault - jak se do něj dostanete pomocí cd?",
                "Jděte do .vault pomocí 'cd .vault', pak přečtěte key.txt.",
            ],
            start: "hub",
            setup: &[file("hub/.vault/key.txt", "platinum\n")],
            answer: exact("platinum").nocase(),
            check: &[at("hub/.vault")],
            solution: r#"
                cd .vault
                shellgame submit "$(cat key.txt)"
            "#,
            success: "Správně! Do skrytého adresáře se vstupuje příkazem 'cd' úplně stejně jako do viditelného.",
            ..LEVEL
        },
        Level {
            title: "Skrytá záloha",
            text: r#"
                ### Cíl
                Identifikujte skrytý soubor podle přípony.

                ### Úkol
                V adresáři `level-3/backup` je několik skrytých souborů.
                Najděte ten, který má příponu `.bak` (záloha).
                Odevzdejte jeho celý název.

                Odevzdejte pomocí: `shellgame submit <název-souboru>`

                Potřebujete pomoc? Napište: `shellgame hint`
            "#,
            hints: &[
                "Použijte 'ls -a' v adresáři backup.",
                "Hledejte soubor začínající tečkou a končící .bak.",
                "Odevzdejte celý název včetně tečky na začátku.",
            ],
            start: "backup",
            setup: &[file("backup/.config", ""), file("backup/.data.bak", ""), file("backup/normal.txt", "")],
            answer: suffix(".data.bak")
                .mistakes(&[
                    ("data.bak", "Skoro: úvodní tečka je součástí názvu, odevzdejte název celý i s ní."),
                    (".data", "Odevzdejte celý název souboru i s příponou."),
                ]),
            solution: r#"shellgame submit "$(ls -a | grep '\.bak$')""#,
            success: "Správně! Úvodní tečka i přípona jsou součástí názvu, takže se soubor odevzdává celým jménem.",
            ..LEVEL
        },
        Level {
            title: "Souhrn Sekce 3",
            text: r#"
                ### Výzva: Mistři skrytých souborů

                Ukažte, že ovládáte práci se skrytými soubory!

                ### Úkol
                V adresáři `level-3/final_test` jsou normální i skryté položky.

                1. Pomocí `ls -la` spočítejte **skryté adresáře** (název začíná tečkou, řádek znakem `d`; bez `.` a `..`)
                2. Najděte skrytý soubor `.secret_code`
                3. Přečtěte jeho obsah
                4. Odevzdejte dvojici ve schématu `POČET,KÓD`

                ### Shrnutí příkazů Sekce 3
                ```
                ls -a         → Zobrazí vše včetně skrytých
                ls -la        → Přidá podrobnosti; adresář má na začátku řádku d
                cat .soubor   → Přečíst skrytý soubor
                cd .adresar   → Vstoupit do skrytého adresáře
                ```

                ### Odevzdání
                `shellgame submit <hodnota>`
            "#,
            hints: &[
                "Skryté položky začínají tečkou. Použijte 'ls -la' pro zobrazení všech položek i jejich typů.",
                "Adresáře poznáte podle 'd' na začátku řádku; běžné soubory začínají '-'.",
                "Položky '.' a '..' nepočítejte. Obsah souboru zobrazíte pomocí 'cat .secret_code'.",
            ],
            start: "final_test",
            setup: &[
                clean("final_test"),
                dir("final_test/.hidden_dir1"),
                dir("final_test/.hidden_dir2"),
                dir("final_test/visible_dir"),
                file("final_test/.secret_code", "hidden_master\n"),
                file("final_test/.config", "not this one\n"),
                file("final_test/.notes", "Poznámky nejsou adresář.\n"),
                file("final_test/readme.txt", "Look for hidden items!\n"),
            ],
            answer: parts(",", &[
                int(2)
                    .mistakes(&[("4", "Možná počítáte i '.' a '..'. Ty vynechte; počítejte jen skryté adresáře.")])
                    .error("Počet skrytých adresářů není správný. V 'ls -la' sledujte první znak řádku a název."),
                exact("hidden_master")
                    .nocase()
                    .mistakes(&[("not this one", "To je obsah .config, ne .secret_code.")])
                    .error("Kód není správný. Přečtěte .secret_code."),
            ])
                .format("Formát odpovědi je: POČET,KÓD"),
            solution: r#"shellgame submit "$(ls -la | grep -c '^d.* \.[^.]'),$(cat .secret_code)""#,
            success: "Výborně! Dokončili jste Sekci 3. Skryté soubory před vámi nic neskryjí!",
            ..LEVEL
        },
    ],
};
