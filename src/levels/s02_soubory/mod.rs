//! Section 2: working with files.

use crate::engine::prelude::*;
use crate::levels::script;

/// Section 2 and its levels.
pub fn section() -> Section {
    Section::new(2, "level-2")
        .intro(intro("Práce se soubory", include_str!("00-uvod.md")))
        .levels([l1(), l2(), l3(), l4(), l5(), l6(), l7(), l8()])
}

fn l1() -> Level {
    level(1, "Navigace mezi sourozenci")
        .text(include_str!("01-sourozenci.md"))
        .hints([
            "Do sourozeneckého adresáře se dostanete přes společného rodiče ('..').",
            "Zkuste nejprve vystoupat o úroveň výše nebo použít spojenou relativní cestu \
             začínající '../'.",
            "Po přesunu ověřte polohu příkazem 'pwd' a spusťte 'shellgame submit'.",
        ])
        .start("start")
        .dirs(["start", "finish"])
        .answer(
            exact("finish")
                .secret()
                .accepts(["finish"])
                .rejects(["start", "Finish"]),
        )
        .require(at("finish"))
        .allow_empty()
        .solution(script(["cd ../finish", "shellgame submit"]))
        .success(
            "Správně! K sourozenci se chodí přes společného rodiče — nahoru a hned dolů jiným \
             směrem.",
        )
}

fn l2() -> Level {
    level(2, "Rychlý návrat")
        .text(include_str!("02-rychly-navrat.md"))
        .hints([
            "Příkaz 'cd -' vás vrátí do předchozího pracovního adresáře (jako tlačítko \
             Zpět).",
            "Nejprve přejděte do 'location-B' ('cd ../location-B') a odtud zadejte 'cd -'.",
            "Po návratu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
        ])
        .start("location-A")
        .dirs(["location-A", "location-B"])
        .require(at("location-A"))
        .require(cd_evidence(
            "Nejdřív přejděte do `location-B` a vraťte se příkazem `cd -`.",
        ))
        .cd(policy().target("-").source(
            "location-B",
            "Příkaz 'cd -' použijte až z adresáře location-B.",
        ))
        .solution(script(["cd ../location-B", "cd -", "shellgame submit"]))
        .success(
            "Správně! Shell si pamatuje předchozí adresář, takže `cd -` přepíná mezi dvěma místy \
             bez cesty.",
        )
}

fn l3() -> Level {
    level(3, "Hluboká navigace")
        .text(include_str!("03-hluboka-navigace.md"))
        .hints([
            "Pro přechod do jiné větve stromu musíte nejprve vystoupat nahoru přes '..' a \
             pak sestoupit dolů.",
            "Ze 'start' vystoupejte o dvě úrovně ('../..') a zadejte 'cd \
             ../../other/target'.",
            "Po přesunu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
        ])
        .start("deep/structure/start")
        .dirs(["deep/structure/start", "deep/other/target"])
        .require(at("deep/other/target"))
        .require(cd_evidence(
            "Použijte ze startu jeden relativní příkaz `cd ../../other/target`.",
        ))
        .cd(policy()
            .source(
                "deep/structure/start",
                "Použijte ze startu jeden příkaz 'cd ../../other/target'.",
            )
            .exact(
                "../../other/target",
                "Použijte ze startu jeden příkaz 'cd ../../other/target'.",
            ))
        .solution(script(["cd ../../other/target", "shellgame submit"]))
        .success(
            "Správně! Jedna relativní cesta umí obsahovat výstup i sestup — nejdřív `..`, potom \
             jména větve.",
        )
}

fn l4() -> Level {
    level(4, "Čtení souboru")
        .text(include_str!("04-cteni-souboru.md"))
        .hints([
            "Příkaz 'cat' vypíše obsah textového souboru na obrazovku.",
            "Spusťte 'cat message.txt' pro zobrazení obsahu souboru.",
            "Z výstupu vezměte jen první slovo (před první mezerou) a zadejte: 'shellgame \
             submit <slovo>'.",
        ])
        .start(".")
        .file("message.txt", "Secret is the key.\n")
        .answer(
            exact("secret")
                .ignore_case()
                .required("Musíte zadat první slovo ze zprávy: shellgame submit <slovo>")
                .secret()
                .accepts(["secret", "Secret", " SECRET "])
                .rejects(["is", "Secret is"]),
        )
        .solution(r#"shellgame submit "$(cut -d' ' -f1 message.txt)""#)
        .success(
            "Správně! `cat` vysype celý obsah souboru do terminálu — ideální na krátké textové \
             soubory.",
        )
}

fn l5() -> Level {
    level(5, "Sledování stop")
        .text(include_str!("05-stopy.md"))
        .hints([
            "Začněte čtením start.txt - co vám soubor říká?",
            "Instrukce vás posílají někam dál. Jaký příkaz použijete pro přesun do adresáře?",
            "Přečtěte start.txt, přejděte do adresáře 'next', přečtěte clue.txt.",
        ])
        .start(".")
        .file(
            "start.txt",
            "Jděte do adresáře 'next' a přečtěte si clue.txt\n",
        )
        .file("next/clue.txt", "Heslo je 'sunshine'\n")
        .answer(
            exact("sunshine")
                .ignore_case()
                .required("Musíte zadat heslo: shellgame submit <heslo>")
                .accepts(["sunshine", "Sunshine"])
                .rejects(["'sunshine'", "next"]),
        )
        .solution(script([
            "cat start.txt",
            "cd next",
            r#"shellgame submit "$(cut -d"'" -f2 clue.txt)""#,
        ]))
        .success(
            "Správně! Střídat `ls`, `cd` a `cat` stačí k prozkoumání libovolné neznámé \
             struktury.",
        )
}

fn l6() -> Level {
    level(6, "Vytvoření souboru")
        .text(include_str!("06-vytvoreni-souboru.md"))
        .hints([
            "Příkaz 'touch' vytvoří prázdný soubor se zadaným názvem.",
            "Spusťte 'touch my_file.txt' v aktuálním adresáři.",
            "Ověřte vytvoření souboru příkazem 'ls' a odešlete: 'shellgame submit'.",
        ])
        .start(".")
        .clean(["my_file.txt"])
        .require(file("my_file.txt"))
        .solution(script(["touch my_file.txt", "shellgame submit"]))
        .success("Správně! `touch` založí prázdný soubor; u existujícího jen posune časové značky.")
}

fn l7() -> Level {
    level(7, "Jak najít pomoc")
        .text(include_str!("07-napoveda.md"))
        .hints([
            "Spusťte 'ls --help' nebo 'man ls' a vyhledejte řádek s přepínačem '-h'.",
            "V nápovědě uvidíte zápis ve tvaru '-h, --název'. Hledejte slovo za dvěma \
             pomlčkami.",
            "Přepínač '-h' je zkratka pro 'human-readable'. Odevzdejte: shellgame submit \
             human-readable",
        ])
        .start(".")
        .answer(
            one_of([
                "human-readable",
                "--human-readable",
                "human readable",
                "human",
                "čitelné",
                "citelne",
                "čitelné formátování",
            ])
            .ignore_case()
            .error(
                "Odpověď není správně. V nápovědě vyhledejte řádek s přepínačem '-h' a \
                 najděte jeho dlouhý název (--...).",
            )
            .required("Musíte zadat odpověď: shellgame submit <název>")
            .accepts([
                "human-readable",
                "--human-readable",
                "Human readable",
                "human",
                "ČITELNÉ",
                "citelne",
            ])
            .rejects(["all", "--all", "-h", "readable"]),
        )
        .solution(concat!(
            "shellgame submit \"$(ls --help | grep -o -e '-h, --[a-z-]*' | ",
            "cut -d' ' -f2)\"",
        ))
        .success(
            "Správně! Teď víte, jak najít pomoc. Příkaz --help a man jsou vaši nejlepší přátelé!",
        )
}

fn l8() -> Level {
    level(8, "Integrační výzva")
        .text(include_str!("08-integracni-vyzva.md"))
        .hints([
            "Ze startu přejděte do 'challenge/room1' a přečtěte 'hint.txt'.",
            "Do sousedního 'room2' se dostanete například příkazem 'cd ../room2'.",
            "V 'room2' přečtěte soubor 'password.txt' pomocí 'cat'.",
        ])
        .start(".")
        .file(
            "challenge/room1/hint.txt",
            "Heslo je v sousedním adresáři room2.\n",
        )
        .file("challenge/room2/password.txt", "navigator\n")
        .file("challenge/room2/decoy.txt", "Toto není heslo.\n")
        .answer(
            exact("navigator")
                .ignore_case()
                .mistake(
                    "toto není heslo",
                    "To je obsah decoy.txt, ne password.txt. Přečtěte správný soubor.",
                )
                .mistake(
                    "toto neni heslo",
                    "To je obsah decoy.txt, ne password.txt. Přečtěte správný soubor.",
                )
                .error("Heslo není správné. Hledejte password.txt v room2.")
                .required("Musíte zadat heslo: shellgame submit <hodnota>")
                .secret()
                .accepts(["navigator", " Navigator "])
                .rejects(["toto není heslo", "Toto neni heslo", "decoy"]),
        )
        .require(at("challenge/room2"))
        .solution(script([
            "cd challenge/room1",
            "cat hint.txt",
            "cd ../room2",
            r#"shellgame submit "$(cat password.txt)""#,
        ]))
        .success("Výborně! Dokončili jste Sekci 2 a umíte propojit navigaci se čtením souborů.")
}
