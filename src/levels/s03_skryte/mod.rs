//! Section 3: hidden files.

use crate::engine::prelude::*;
use crate::levels::script;

/// Section 3 and its levels.
pub fn section() -> Section {
    Section::new(3, "level-3")
        .intro(intro("Sekce 3: Skryté soubory", include_str!("00-uvod.md")))
        .levels([l1(), l2(), l3(), l4(), l5()])
}

fn l1() -> Level {
    level(1, "Počítání skrytých adresářů")
        .text(include_str!("01-skryte-adresare.md"))
        .hints([
            "Použijte 'ls -la': -a zobrazí skryté položky a -l přidá podrobnosti.",
            "Adresář poznáte podle znaku 'd' na začátku řádku; běžný soubor začíná '-'.",
            "Počítejte skryté názvy v řádcích začínajících 'd', ale vynechte položky '.' a \
             '..'.",
        ])
        .start("hub")
        .clean(["hub"])
        .dirs(["hub/.beta", "hub/.gamma", "hub/visible_dir"])
        .file("hub/.config", "")
        .file("hub/visible_file.txt", "")
        .answer(
            int(2)
                .mistake(
                    3,
                    "Nejspíš počítáte i skrytý soubor '.config'. Jeho řádek v 'ls -la' nezačíná \
                     'd'.",
                )
                .mistake(
                    4,
                    "Nejspíš počítáte i položky '.' a '..'. Jsou to speciální odkazy, ne hledané \
                     adresáře.",
                )
                .mistake(
                    5,
                    "Nejspíš počítáte '.', '..' i skrytý soubor. Sledujte první znak řádku a \
                     přesný název.",
                )
                .accepts(["2", " 2 ", "+2"])
                .rejects(["3", "4", "5", "dva"]),
        )
        .solution(r#"shellgame submit "$(ls -la | grep -c '^d.* \.[^.]')""#)
        .success(
            "Správně! Skryté položky zobrazí `ls -a` a typ položky poznáte podle prvního znaku \
             řádku `ls -l`.",
        )
}

fn l2() -> Level {
    level(2, "Čtení skrytého souboru")
        .text(include_str!("02-cteni-skryteho-souboru.md"))
        .hints([
            "Skryté soubory začínají tečkou. Jak je zobrazíte pomocí ls?",
            "Příkaz 'cat' funguje i na skryté soubory - stačí zadat správný název včetně \
             tečky.",
            "Zkuste: cat .secret_config",
        ])
        .start(".")
        .file(".secret_config", "mode=stealth\n")
        .answer(
            exact("mode=stealth")
                .accepts(["mode=stealth"])
                .rejects(["mode=", "stealth"]),
        )
        .solution(r#"shellgame submit "$(cat .secret_config)""#)
        .success(
            "Správně! Tečka na začátku názvu soubor jen skryje ve výpisu, práci s ním nijak \
             neomezuje.",
        )
}

fn l3() -> Level {
    level(3, "Uvnitř skrytého adresáře")
        .text(include_str!("03-skryty-adresar.md"))
        .hints([
            "Jak byste vstoupili do běžného adresáře? Stejně to funguje i se skrytými.",
            "Skrytý adresář .vault - jak se do něj dostanete pomocí cd?",
            "Jděte do .vault pomocí 'cd .vault', pak přečtěte key.txt.",
        ])
        .start("hub")
        .file("hub/.vault/key.txt", "platinum\n")
        .answer(
            exact("platinum")
                .ignore_case()
                .accepts(["platinum", "Platinum"])
                .rejects(["gold"]),
        )
        .require(at("hub/.vault"))
        .solution(script([
            "cd .vault",
            r#"shellgame submit "$(cat key.txt)""#,
        ]))
        .success(
            "Správně! Do skrytého adresáře se vstupuje příkazem `cd` úplně stejně jako do \
             viditelného.",
        )
}

fn l4() -> Level {
    level(4, "Skrytá záloha")
        .text(include_str!("04-skryta-zaloha.md"))
        .hints([
            "Použijte 'ls -a' v adresáři backup.",
            "Hledejte soubor začínající tečkou a končící .bak.",
            "Odevzdejte celý název včetně tečky na začátku.",
        ])
        .start("backup")
        .file("backup/.config", "")
        .file("backup/.data.bak", "")
        .file("backup/normal.txt", "")
        .answer(exact(".data.bak").accepts([".data.bak"]).rejects([
            "data.bak",
            ".config",
            "normal.txt",
        ]))
        .solution(r#"shellgame submit "$(ls -a | grep '\.bak$')""#)
        .success(
            "Správně! Úvodní tečka i přípona jsou součástí názvu, takže se soubor odevzdává \
             celým jménem.",
        )
}

fn l5() -> Level {
    level(5, "Souhrn Sekce 3")
        .text(include_str!("05-souhrn-sekce-3.md"))
        .hints([
            "Skryté položky začínají tečkou. Použijte 'ls -la' pro zobrazení všech položek i \
             jejich typů.",
            "Adresáře poznáte podle 'd' na začátku řádku; běžné soubory začínají '-'.",
            "Položky '.' a '..' nepočítejte. Obsah souboru zobrazíte pomocí 'cat \
             .secret_code'.",
        ])
        .start("final_test")
        .clean(["final_test"])
        .dirs([
            "final_test/.hidden_dir1",
            "final_test/.hidden_dir2",
            "final_test/visible_dir",
        ])
        .file("final_test/.secret_code", "hidden_master\n")
        .file("final_test/.config", "not this one\n")
        .file("final_test/.notes", "Poznámky nejsou adresář.\n")
        .file("final_test/readme.txt", "Look for hidden items!\n")
        .answer(
            parts(
                ",",
                [
                    int(2)
                        .mistake(
                            4,
                            "Možná počítáte i ./ a ../. Ty vynechte; počítejte jen skryté \
                             adresáře.",
                        )
                        .error(
                            "Počet skrytých adresářů není správně. V 'ls -la' sledujte první \
                             znak řádku a název.",
                        )
                        .invalid("První část musí být číslo (počet skrytých adresářů).")
                        .accepts(["2"])
                        .rejects(["4"]),
                    exact("hidden_master")
                        .ignore_case()
                        .mistake("not this one", "To je obsah .config, ne .secret_code.")
                        .error("Kód není správný. Přečtěte .secret_code.")
                        .secret(),
                ],
            )
            .format_message("Formát odpovědi je: POČET,KÓD")
            .accepts(["2,hidden_master", " 2 , Hidden_Master "])
            .rejects([
                "2",
                "4,hidden_master",
                "2,not this one",
                "hidden_master,2",
                "2;hidden_master",
            ]),
        )
        .solution(concat!(
            r#"shellgame submit "$(ls -la | grep -c '^d.* \.[^.]'),"#,
            "$(cat .secret_code)\"",
        ))
        .success("Výborně! Dokončili jste Sekci 3. Skryté soubory před vámi nic neskryjí!")
}
