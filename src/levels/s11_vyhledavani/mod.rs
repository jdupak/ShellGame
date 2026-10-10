//! Section 11: searching.

use crate::engine::prelude::*;
use crate::levels::script;

/// Section 11 and its levels.
pub fn section() -> Section {
    Section::new(11, "level-11")
        .intro(intro("Sekce 11: Vyhledávání", include_str!("00-uvod.md")))
        .levels([l1(), l2(), l3(), l4(), l5(), l6()])
}

fn l1() -> Level {
    level(1, "Hledání v souboru (grep)")
        .text(include_str!("01-grep.md"))
        .hints([
            "Příkaz `grep` hledá zadaný vzor v souborech.",
            "Vyhledejte klíč 'ACTIVE_PROFILE' v souboru 'config.txt' a výstup přesměrujte \
             pomocí `>`.",
            "Spusťte `grep ACTIVE_PROFILE config.txt > profile.txt` a výsledek ověřte pomocí \
             `cat profile.txt`.",
        ])
        .start("grep")
        .clean(["grep/profile.txt", "grep/config.txt"])
        .file(
            "grep/config.txt",
            "user=admin\n\
             host=localhost\n\
             port=8080\n\
             ACTIVE_PROFILE=production\n\
             debug=true\n",
        )
        .answer(exact("profile.txt"))
        .require(
            content("grep/config.txt")
                .contains(["ACTIVE_PROFILE=production"])
                .message(
                    "Zdrojový soubor config.txt chybí. Obnovte level příkazem `shellgame reset`.",
                )
                .missing(
                    "Zdrojový soubor config.txt chybí. Obnovte level příkazem `shellgame reset`.",
                )
                .unreadable(
                    "Zdrojový soubor config.txt nelze přečíst. Obnovte level příkazem `shellgame \
                     reset`.",
                ),
        )
        .require(
            content("grep/profile.txt")
                .exact("ACTIVE_PROFILE=production")
                .strip()
                .message("Soubor neobsahuje přesně hledaný řádek.")
                .missing(
                    "Výstup profile.txt musí být soubor. Obnovte level příkazem `shellgame \
                     reset`.",
                )
                .unreadable(
                    "Soubor profile.txt nelze přečíst. Obnovte level příkazem `shellgame reset`.",
                ),
        )
        .solution(script([
            "grep ACTIVE_PROFILE config.txt > profile.txt",
            "shellgame submit 'profile.txt'",
        ]))
        .success("Správně! `grep` vybere jen potřebné řádky a `>` je uloží pro další použití.")
}

fn l2() -> Level {
    level(2, "Rekurzivní hledání")
        .text(include_str!("02-grep-r.md"))
        .hints([
            "Přepínač `-r` hledá rekurzivně ve všech podadresářích.",
            r#"Zkuste `grep -r "SECRET_KEY" project/`."#,
            "grep -r vypíše cestu před dvojtečkou. Odevzdejte právě tu cestu.",
        ])
        .start("recursive")
        .file("recursive/project/README.md", "# Project\n")
        .file("recursive/project/src/main.py", "print('Hello')\n")
        .file(
            "recursive/project/config/settings.py",
            "SECRET_KEY = 'xyz'\nDEBUG = True\n",
        )
        .answer(
            suffix("config/settings.py")
                .error(
                    "To není správná odpověď. `grep -r` vypisuje před dvojtečkou celou relativní \
                     cestu — odevzdejte ji i s adresáři, ne jen samotný název souboru. Úvodní \
                     `./` nevadí.",
                )
                .secret()
                .accepts([
                    "config/settings.py",
                    "./config/settings.py",
                    "project/config/settings.py",
                    "/tmp/shellgame-student/level-11/recursive/project/config/settings.py",
                ])
                .rejects(["settings.py", "project/src/main.py", "config/settings.pyc"]),
        )
        .solution(r#"shellgame submit "$(grep -r SECRET_KEY project/ | cut -d: -f1)""#)
        .success(
            "Správně! S přepínačem -r prochází grep celý strom adresářů a u každého nálezu uvede \
             cestu.",
        )
}

fn l3() -> Level {
    level(3, "Hledání bez ohledu na velikost písmen")
        .text(include_str!("03-grep-i.md"))
        .hints([
            "Bez `-i` najdete jen některé varianty. S `-i` najdete `warning`, `WARNING`, \
             `Warning`…",
            r#"Zkuste `grep -i "warning" messages.log` a spočítejte vypsané řádky."#,
            "Počítejte každý řádek, který grep -i vypíše — ne jen jednu velikost písmen.",
        ])
        .start("case")
        .file(
            "case/messages.log",
            "2024-01-01 10:00:00 INFO: Server started\n\
             2024-01-01 10:05:00 WARNING: Memory usage high\n\
             2024-01-01 10:10:00 ERROR: Connection lost\n\
             2024-01-01 10:15:00 warning: Disk space low\n\
             2024-01-01 10:20:00 INFO: User logged in\n\
             2024-01-01 10:25:00 Warning: CPU temperature elevated\n\
             2024-01-01 10:30:00 DEBUG: Cache cleared\n\
             2024-01-01 10:35:00 WARNING: Network latency detected\n\
             2024-01-01 10:40:00 INFO: Backup completed\n",
        )
        .answer(
            int(4)
                .mistake(
                    2,
                    "Našli jste jen 'WARNING'. Použijte -i pro nalezení všech variant (warning, \
                     Warning...).",
                )
                .mistake(
                    1,
                    "Našli jste jen jednu variantu. Přepínač -i ignoruje velikost písmen.",
                )
                .error(
                    "Počet není správně. Zkontrolujte, že hledáte bez ohledu na velikost písmen.",
                )
                .invalid("Odpověď musí být číslo.")
                .accepts(["4"])
                .rejects(["2", "1", "5"]),
        )
        .solution(r#"shellgame submit "$(grep -i warning messages.log | wc -l)""#)
        .success("Správně! Přepínač -i je nepostradatelný pro robustní hledání.")
}

fn l4() -> Level {
    level(4, "Hledání souborů (find)")
        .text(include_str!("04-find.md"))
        .hints([
            "Příkaz `find` prohledá všechny podadresáře automaticky.",
            r#"Pro hledání souborů použijte: `find kde_hledat -type f -name "co_hledat"`."#,
            r#"Použijte: `find messy_dir -type f -name "lost_file.txt"`."#,
        ])
        .start("find")
        .file("find/messy_dir/a/b/c/d/lost_file.txt", "")
        .file("find/messy_dir/other.txt", "")
        .file("find/messy_dir/a/junk.txt", "")
        .answer(
            suffix("messy_dir/a/b/c/d/lost_file.txt")
                .error(
                    "To není správná cesta. Odevzdejte ji přesně tak, jak ji vypsal `find` — \
                     včetně všech adresářů, ne jen název souboru. Úvodní `./` nevadí.",
                )
                .secret()
                .accepts([
                    "messy_dir/a/b/c/d/lost_file.txt",
                    "./messy_dir/a/b/c/d/lost_file.txt",
                ])
                .rejects(["lost_file.txt", "messy_dir/other.txt", "d/lost_file.txt/"]),
        )
        .solution(r#"shellgame submit "$(find messy_dir -type f -name 'lost_file.txt')""#)
        .success(
            "Správně! `find` hledá podle vlastností souboru, takže si poradí i s neznámým \
             umístěním.",
        )
}

fn l5() -> Level {
    level(5, "Hledání podle přípony")
        .text(include_str!("05-hledani-podle-pripony.md"))
        .hints([
            "Omezte `find` na běžné soubory pomocí `-type f`; adresář s příponou `.py` se \
             počítat nemá.",
            "Vzor dejte do uvozovek a výstup pošlete přes `sort` před uložením pomocí `>`.",
            r#"Použijte: `find src_code -type f -name "*.py" | sort > python_files.txt`."#,
        ])
        .start("extension")
        .clean(["extension/python_files.txt"])
        .dir("extension/src_code/archive.py")
        .file("extension/src_code/main.py", "")
        .file("extension/src_code/utils.py", "")
        .file("extension/src_code/README.txt", "")
        .file("extension/src_code/data.csv", "")
        .answer(exact("python_files.txt"))
        .require(
            content("extension/python_files.txt")
                .exact("src_code/main.py\nsrc_code/utils.py")
                .strip()
                .message(
                    "Obsah souboru nesedí. Bez `-type f` se do výpisu dostane i adresář s \
                     příponou `.py`; bez uvozovek kolem vzoru ho rozbalí shell ještě před \
                     spuštěním `find`. Nezapomeňte také na `sort`.",
                )
                .missing(
                    "Soubor zatím neexistuje. Výstup příkazu uložte pomocí `>` do souboru \
                     zadaného v úkolu.",
                ),
        )
        .solution(script([
            r#"find src_code -type f -name "*.py" | sort > python_files.txt"#,
            "shellgame submit 'python_files.txt'",
        ]))
        .success("Správně! Spojení `find`, `sort` a `>` dává výsledek, který vyjde pokaždé stejně.")
}

fn l6() -> Level {
    level(6, "Finální výzva")
        .text(include_str!("06-finalni-vyzva.md"))
        .hints([
            "Pro hledání skriptů spojte `find` s podmínkami `-type f` a `-name`; obsah pak \
             prohledejte pomocí `grep`.",
            "Skripty vypíše `find . -type f -name \"*.sh\"`. V nalezených souborech hledejte \
             řádek se slovem `SECRET`.",
            "Kopii vytvořte jako `found/secret.txt` a počet zjistěte přes `find . -type f \
             -name \"*.sh\" | wc -l`. ",
            "Odevzdejte počet a samotný kód.",
        ])
        .start("final")
        .clean(["final"])
        .dir("final/found")
        .file("final/run.sh", "#!/bin/bash\necho 'Running...'\n")
        .file("final/scripts/build.sh", "#!/bin/bash\nmake all\n")
        .file("final/scripts/test.sh", "#!/bin/bash\npytest\n")
        .file(
            "final/hidden/secret.sh",
            "#!/bin/bash\n\
             # SECRET_CODE=NINJA2024\n\
             echo 'You found me!'\n",
        )
        .file("final/readme.txt", "Look for shell scripts!\n")
        .file("final/data.csv", "col1,col2\n")
        .answer(
            parts(
                ",",
                [
                    int(4)
                        .error(
                            "Počet .sh souborů není správně. Použijte 'find . -type f -name \
                             \"*.sh\"'.",
                        )
                        .invalid("První hodnota musí být číslo."),
                    exact("NINJA2024")
                        .ignore_case()
                        .error(
                            "Tajný kód není správně. Hledejte 'SECRET' v obsahu skriptů pomocí \
                             'grep'.",
                        )
                        .secret(),
                ],
            )
            .format_message("Formát: počet,kód (např. 5,SECRET123)")
            .accepts(["4,NINJA2024", "4, ninja2024"])
            .rejects(["3,NINJA2024", "4", "NINJA2024,4", "4,NINJA"]),
        )
        .require(
            same("final/hidden/secret.sh", "final/found/secret.txt")
                .message("Do složky found/ zkopírujte skript s tajným kódem.")
                .destination_error("Do složky found/ zkopírujte skript s tajným kódem."),
        )
        .solution(script([
            "cp hidden/secret.sh found/secret.txt",
            concat!(
                "shellgame submit \"$(find . -type f -name '*.sh' | wc -l),",
                "$(grep -rh SECRET_CODE --include='*.sh' . | cut -d= -f2)\"",
            ),
        ]))
        .success(
            "GRATULUJEME!\n\
             \n\
             Úspěšně jste dokončili ShellGame!\n\
             \n\
             Nyní ovládáte základy práce s terminálem:\n\
             ✓ Navigace v souborovém systému\n\
             ✓ Práce se soubory a adresáři\n\
             ✓ Oprávnění a typy souborů\n\
             ✓ Přesměrování a streamy\n\
             ✓ Wildcards a vyhledávání\n\
             \n\
             Jste připraveni na další dobrodružství v Linuxu!",
        )
}
