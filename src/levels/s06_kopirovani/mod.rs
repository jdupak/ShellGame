//! Section 6: copying and moving.

use crate::engine::prelude::*;
use crate::levels::script;

/// Section 6 and its levels.
pub fn section() -> Section {
    Section::new(6, "level-6")
        .intro(intro(
            "Sekce 6: Kopírování a přesouvání",
            include_str!("00-uvod.md"),
        ))
        .levels([l1(), l2(), l3(), l4(), l5(), l6(), l7(), l8()])
}

fn l1() -> Level {
    level(1, "Kopírování souboru")
        .text(include_str!("01-kopirovani-souboru.md"))
        .hints([
            "Příkaz cp má dva argumenty: odkud a kam kopírujete.",
            "Syntaxe je: cp zdrojový_soubor cílový_soubor",
            "Použijte 'cp dulezite.txt dulezite.bak'.",
        ])
        .start("copying")
        .clean(["copying/dulezite.bak"])
        .file("copying/dulezite.txt", "Very important data.")
        .answer(
            exact("dulezite.bak")
                .mistake(
                    "dulezite.txt",
                    "Odevzdali jste název původního souboru, ne jeho kopie. Příkaz cp bere \
                     nejdřív zdroj a potom cíl: 'cp zdroj cíl'.",
                )
                .accepts(["dulezite.bak"])
                .rejects(["dulezite.txt"]),
        )
        .require(same("copying/dulezite.txt", "copying/dulezite.bak"))
        .allow_empty()
        .solution(script([
            "cp dulezite.txt dulezite.bak",
            "shellgame submit 'dulezite.bak'",
        ]))
        .success(
            "Správně! Záloha je samostatná kopie pod novým jménem, originál zůstává nedotčený.",
        )
}

fn l2() -> Level {
    level(2, "Kopírování adresáře")
        .text(include_str!("02-kopirovani-adresare.md"))
        .hints([
            "Pro kopírování celého adresáře včetně obsahu je nutné použít rekurzivní \
             přepínač '-r'.",
            "Tvar je 'cp -r <zdroj> <cíl>': přepínač stojí před oběma argumenty, zdrojový \
             adresář je první a název nové kopie druhý.",
            "Spusťte 'cp -r projekt projekt_zaloha'.",
        ])
        .start("copying")
        .clean(["copying/projekt_zaloha"])
        .file("copying/projekt/main.py", "print('hello')")
        .answer(exact("projekt_zaloha"))
        .require(same("copying/projekt", "copying/projekt_zaloha"))
        .allow_empty()
        .solution(script([
            "cp -r projekt projekt_zaloha",
            "shellgame submit 'projekt_zaloha'",
        ]))
        .success("Správně! Přepínač -r říká příkazu cp, že má sestoupit do celého stromu adresáře.")
}

fn l3() -> Level {
    level(3, "Přejmenování souboru")
        .text(include_str!("03-prejmenovani-souboru.md"))
        .hints([
            "Příkaz 'mv' slouží nejen k přesunu, ale i k přejmenování souboru: 'mv staré \
             nové'.",
            "Oba argumenty jsou jména souborů v aktuálním adresáři: první je to současné, \
             druhé to požadované.",
            "Spusťte 'mv spatne_jmeno.txt spravne_jmeno.txt'.",
        ])
        .start("moving")
        .clean(["moving/spravne_jmeno.txt"])
        .file("moving/spatne_jmeno.txt", "content")
        .answer(exact("spravne_jmeno.txt"))
        .require(moved("moving/spatne_jmeno.txt", "moving/spravne_jmeno.txt"))
        .allow_empty()
        .solution(script([
            "mv spatne_jmeno.txt spravne_jmeno.txt",
            "shellgame submit 'spravne_jmeno.txt'",
        ]))
        .success("Správně! Přejmenování je jen přesun na novou cestu ve stejném adresáři.")
}

fn l4() -> Level {
    level(4, "Přesun souboru")
        .text(include_str!("04-presun-souboru.md"))
        .hints([
            "O tom, co 'mv' udělá, rozhoduje podoba cíle: soubor znamená přejmenování, \
             existující adresář přesun.",
            "Tvar je 'mv <soubor> <cílový_adresář>/': první argument je přesouvaný soubor, \
             druhý cílový adresář zakončený lomítkem.",
            "Spusťte 'mv report.pdf dokumenty/'. Lomítko na konci značí adresář.",
        ])
        .start("moving")
        .clean(["moving/dokumenty/report.pdf"])
        .dir("moving/dokumenty")
        .file("moving/report.pdf", "report data")
        .answer(exact("dokumenty"))
        .require(
            moved("moving/report.pdf", "moving/dokumenty/report.pdf")
                .source_error(
                    "report.pdf je stále na původním místě. Kopie originál ponechává, přesun ho \
                     odtud odebere.",
                )
                .destination_error(
                    "report.pdf z původního místa zmizel, ale v cílovém adresáři není. Když cíl \
                     neexistuje, 'mv' soubor jen přejmenuje - zkontrolujte 'ls'.",
                ),
        )
        .allow_empty()
        .solution(script([
            "mv report.pdf dokumenty/",
            "shellgame submit 'dokumenty'",
        ]))
        .success("Správně! Podoba cíle rozhoduje, jestli mv přejmenovává, nebo přesouvá.")
}

fn l5() -> Level {
    level(5, "Přesun adresáře")
        .text(include_str!("05-presun-adresare.md"))
        .hints([
            "Adresář se přesouvá stejným příkazem jako soubor: 'mv' vezme celý podstrom a \
             přepínač '-r' nepotřebuje.",
            "Tvar je 'mv <adresář> <cílový_adresář>/': první argument je přesouvaný adresář, \
             druhý už existující rodič, do kterého má vklouznout.",
            "Spusťte 'mv projekt archiv/' a ověřte výsledek pomocí 'ls archiv/'.",
        ])
        .start("moving")
        .clean(["moving/projekt", "moving/archiv/projekt"])
        .dir("moving/archiv")
        .file("moving/projekt/data/file.txt", "content")
        .require(
            moved("moving/projekt", "moving/archiv/projekt")
                .source_error(
                    "Adresář projekt je stále na původním místě - zatím se nic nepřesunulo.",
                )
                .destination_error(
                    "Projekt z původního místa zmizel, ale v archivu není. Když cílový adresář \
                     neexistuje, 'mv' zdroj jen přejmenuje - porovnejte 'ls' a 'ls archiv/'.",
                ),
        )
        .require(
            content("moving/archiv/projekt/data/file.txt")
                .exact("content")
                .message("Přesunutý adresář nemá původní obsah."),
        )
        .solution(script(["mv projekt archiv/", "shellgame submit"]))
        .success("Správně! Přesun adresáře vezme celý jeho obsah s sebou jedním krokem.")
}

fn l6() -> Level {
    level(6, "Úklid logů")
        .text(include_str!("06-uklid-logu.md"))
        .hints([
            "Žolík '*' nahradí libovolný počet znaků, takže vzor '*.log' vybere všechny \
             soubory s touto příponou.",
            "Příkaz 'mv' dokáže přesunout více souborů najednou do cílového adresáře: 'mv \
             <soubory> <cíl>/'.",
            "Spusťte 'mv *.log logs/' a odevzdejte 'logs'.",
        ])
        .start("organize")
        .clean(["organize/logs"])
        .dir("organize/logs")
        .file("organize/app.log", "log1")
        .file("organize/error.log", "log2")
        .file("organize/other.txt", "keep me")
        .answer(exact("logs"))
        .require(moved("organize/app.log", "organize/logs/app.log"))
        .require(moved("organize/error.log", "organize/logs/error.log"))
        .require(file("organize/other.txt"))
        .solution(script(["mv *.log logs/", "shellgame submit 'logs'"]))
        .success(
            "Správně! Vzor se žolíkem rozbalí shell na seznam jmen ještě před spuštěním příkazu.",
        )
}

fn l7() -> Level {
    level(7, "Rychlejší práce: alias")
        .text(include_str!("07-alias.md"))
        .hints([
            "Alias dává dlouhému příkazu kratší jméno. Vyberte syntaxi pro svůj shell.",
            "Po nastavení spusťte 'type sg' a ověřte, že sg odkazuje na shellgame.",
            "Level dokončete přes alias příkazem 'sg submit alias-ready'.",
        ])
        .answer(
            exact("alias-ready")
                .required("Nejprve nastavte a ověřte alias, potom spusťte: sg submit <odpověď>")
                .accepts(["alias-ready"])
                .rejects(["ready", "alias"]),
        )
        // Bash expands aliases only in interactive shells unless asked to; fish has no `shopt`,
        // so the line is guarded.
        .solution(script([
            r#"test -n "$BASH_VERSION" && shopt -s expand_aliases"#,
            "alias sg='shellgame'",
            "type sg",
            "sg submit alias-ready",
        ]))
        .success(
            "Výborně! Alias je jen krátké jméno pro delší příkaz — v závěrečné výzvě se vám bude \
             hodit.",
        )
}

fn l8() -> Level {
    level(8, "Výzva: Organizátor souborů")
        .text(include_str!("08-organizator.md"))
        .hints([
            "Rozhodněte u každého souboru, zda musí původní cesta zůstat zachovaná. Podle \
             toho zvolte 'cp' nebo 'mv'.",
            "Přejmenování i přesun používají 'mv'; rozdíl určuje podoba cíle. Kopie do \
             adresáře zachová původní soubor.",
            "Použijte 'cp original.txt backup/', 'mv temp_data.csv data.csv' a 'mv \
             misplaced.log logs/'.",
        ])
        .start("final_test")
        .clean(["final_test"])
        .dirs(["final_test/backup", "final_test/logs"])
        .file(
            "final_test/original.txt",
            "Important data - do not delete!\n",
        )
        .file("final_test/temp_data.csv", "col1,col2\n1,2\n")
        .file("final_test/misplaced.log", "Log entry\n")
        .require(
            file("final_test/original.txt")
                .message("Smazali jste original.txt! Měli jste ho zkopírovat, ne přesunout."),
        )
        .require(
            same("final_test/original.txt", "final_test/backup/original.txt")
                .message("Kopie v backup/ neodpovídá originálu.")
                .destination_error("Chybí kopie original.txt v adresáři backup/."),
        )
        .require(
            moved("final_test/temp_data.csv", "final_test/data.csv")
                .source_error(
                    "temp_data.csv stále existuje. Přejmenujte ho na data.csv pomocí 'mv'.",
                )
                .destination_error(
                    "Chybí data.csv. Zkontrolujte, zda jste soubor přejmenovali místo \
                     kopírování.",
                ),
        )
        .require(
            moved("final_test/misplaced.log", "final_test/logs/misplaced.log")
                .source_error("misplaced.log stále v hlavní složce. Přesuňte do logs/ pomocí 'mv'.")
                .destination_error("Chybí misplaced.log v adresáři logs/."),
        )
        .solution(script([
            "cp original.txt backup/",
            "mv temp_data.csv data.csv",
            "mv misplaced.log logs/",
            "shellgame submit",
        ]))
        .success(
            "Výborně! Dokončili jste Sekci 6 — kopírování, přesun i přejmenování máte pod \
             kontrolou.",
        )
}
