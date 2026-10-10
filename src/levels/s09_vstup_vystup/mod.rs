//! Section 9: input, output and the status of commands.

use crate::engine::prelude::*;
use crate::levels::script;

/// Section 9 and its levels.
pub fn section() -> Section {
    Section::new(9, "level-9")
        .intro(intro(
            "Sekce 9: Vstup, výstup a stav příkazů",
            include_str!("00-uvod.md"),
        ))
        .levels([
            l1(),
            l2(),
            l3(),
            l4(),
            l5(),
            l6(),
            l7(),
            l8(),
            l9(),
            l10(),
            l11(),
        ])
}

fn l1() -> Level {
    level(1, "Uložení výstupu")
        .text(include_str!("01-ulozeni-vystupu.md"))
        .hints([
            "Použijte operátor '>' pro přesměrování výstupu.",
            "Příkaz 'ls' vypíše obsah adresáře.",
            "Zkuste: 'ls > seznam.txt'.",
        ])
        .start("redirection")
        .clean(["redirection/seznam.txt"])
        .file("redirection/file1", "")
        .file("redirection/file2", "")
        .answer(exact("seznam.txt"))
        .require(
            content("redirection/seznam.txt")
                .contains(["file1", "file2"])
                .message("Soubor neobsahuje očekávaný výstup příkazu ls.")
                .missing("Soubor neexistuje."),
        )
        .allow_empty()
        .solution(script(["ls > seznam.txt", "shellgame submit 'seznam.txt'"]))
        .success(
            "Správně! Výstup příkazu nemusí skončit na obrazovce — dá se uložit a dál s ním \
             pracovat.",
        )
}

fn l2() -> Level {
    level(2, "Přidání na konec")
        .text(include_str!("02-pridani-konec.md"))
        .hints([
            "Dvě šipky '>>' znamenají append (připojení na konec souboru bez přepsání \
             obsahu).",
            r#"Spusťte 'echo "Konec logu" >> log.txt'."#,
        ])
        .start("redirection")
        .file("redirection/log.txt", "Start logu\nZaznam 1\n")
        .answer(exact("log.txt"))
        .require(
            content("redirection/log.txt")
                .exact(
                    "Start logu\n\
                     Zaznam 1\n\
                     Konec logu\n",
                )
                .message(
                    "Soubor musí zachovat původní obsah a přidat nový řádek přesně na konec. \
                     Zmizely-li původní řádky, použili jste jednoduchou šipku `>`, která soubor \
                     přepíše; připojení zajistí až zdvojená šipka. Původní stav vrátí `shellgame \
                     reset`.",
                )
                .missing("Soubor neexistuje."),
        )
        .allow_empty()
        .solution(script([
            "echo 'Konec logu' >> log.txt",
            "shellgame submit 'log.txt'",
        ]))
        .success(
            "Správně! Zdvojená šipka připojuje, takže předchozí obsah souboru zůstane zachovaný.",
        )
}

fn l3() -> Level {
    level(3, "Spojování souborů")
        .text(include_str!("03-spojovani-souboru.md"))
        .hints([
            "Příkaz 'cat' umí přijmout více souborů najednou a vypsat jejich obsahy za \
             sebou.",
            "Výstup více souborů z 'cat' můžete přesměrovat pomocí '>' do cílového souboru.",
            "Spusťte 'cat part1.txt part2.txt > full.txt'. Pořadí argumentů určuje pořadí v \
             souboru.",
        ])
        .start("concat")
        .clean(["concat/full.txt"])
        .file("concat/part1.txt", "First part.\n")
        .file("concat/part2.txt", "Second part.\n")
        .answer(exact("full.txt"))
        .require(
            content("concat/full.txt")
                .exact("First part.\nSecond part.\n")
                .message(
                    "Soubor musí obsahovat obě části přesně v zadaném pořadí. Jsou-li obsahy na \
                     místě, ale prohozené, prohodili jste argumenty příkazu cat: jejich pořadí \
                     určuje pořadí řádků ve výstupu.",
                )
                .missing("Soubor neexistuje."),
        )
        .allow_empty()
        .solution(script([
            "cat part1.txt part2.txt > full.txt",
            "shellgame submit 'full.txt'",
        ]))
        .success(
            "Správně! Příkaz cat čte soubory v pořadí argumentů a stejné pořadí má i výsledek.",
        )
}

fn l4() -> Level {
    level(4, "Uvozovky v textu i názvu")
        .text(include_str!("04-uvozovky.md"))
        .hints([
            "Mezery oddělují argumenty. Uvozovky udrží více slov pohromadě jako jeden text \
             nebo jednu cestu.",
            "Uzavřete do uvozovek text za `echo` a zvlášť také název za `>`.",
            "Spusťte `echo \"Ahoj svete\" > \"muj pozdrav.txt\"` a název v uvozovkách také \
             odevzdejte.",
        ])
        .start("echo")
        .clean(["echo/pozdrav.txt", "echo/muj pozdrav.txt"])
        .dir("echo")
        .answer(
            exact("muj pozdrav.txt")
                .required(r#"Odevzdejte název v uvozovkách: shellgame submit "<název>""#)
                .accepts(["muj pozdrav.txt"])
                .rejects(["muj", "pozdrav.txt", "muj_pozdrav.txt"]),
        )
        .require(
            content("echo/muj pozdrav.txt")
                .exact("Ahoj svete\n")
                .message("Soubor nemá přesně požadovaný jeden řádek.")
                .missing("Chybí soubor s požadovaným názvem obsahujícím mezeru."),
        )
        .solution(script([
            r#"echo "Ahoj svete" > "muj pozdrav.txt""#,
            "shellgame submit 'muj pozdrav.txt'",
        ]))
        .success("Správně! Uvozovky drží pohromadě to, co by shell jinak rozdělil podle mezer.")
}

fn l5() -> Level {
    level(5, "Propojení příkazů (Pipes)")
        .text(include_str!("05-pipes.md"))
        .hints([
            "Pipe (|) propojuje výstup prvního příkazu se vstupem druhého.",
            "grep najde řádky s 'ERROR', wc -l je spočítá. Spojte je pomocí |.",
            r#"Použijte: grep "ERROR" access.log | wc -l"#,
        ])
        .start("pipes")
        .file(
            "pipes/access.log",
            "2024-01-01 10:00:00 INFO Server started\n\
             2024-01-01 10:05:23 ERROR Connection refused\n\
             2024-01-01 10:10:45 INFO User logged in\n\
             2024-01-01 10:15:00 WARNING Low memory\n\
             2024-01-01 10:20:12 ERROR Database timeout\n\
             2024-01-01 10:25:00 INFO Request processed\n\
             2024-01-01 10:30:33 ERROR File not found\n\
             2024-01-01 10:35:00 INFO Cache cleared\n\
             2024-01-01 10:40:55 ERROR Permission denied\n\
             2024-01-01 10:45:00 DEBUG Verbose output\n\
             2024-01-01 10:50:18 ERROR Network unreachable\n\
             2024-01-01 10:55:00 INFO Backup completed\n\
             2024-01-01 11:00:00 ERROR Disk full\n\
             2024-01-01 11:05:00 INFO Server shutdown\n\
             2024-01-01 11:10:42 ERROR Service unavailable\n",
        )
        .answer(
            int(7)
                .mistake(
                    15,
                    "Spočítali jste všechny řádky. Potřebujete jen ty s 'ERROR'. Použijte grep \
                     před wc.",
                )
                .accepts(["7", " 7 "])
                .rejects(["15", "6", "seven"]),
        )
        .solution(r#"shellgame submit "$(grep ERROR access.log | wc -l)""#)
        .success(
            "Správně! Roura spojí jednoduché příkazy v nástroj, jakým žádný z nich sám o sobě \
             není.",
        )
}

fn l6() -> Level {
    level(6, "Začátek a konec souboru")
        .text(include_str!("06-head-tail.md"))
        .hints([
            "head -n 1 zobrazí první řádek, tail -n 1 zobrazí poslední.",
            "Odevzdejte první slovo z každého z těchto dvou řádků.",
            "Formát odpovědi je `prvni,posledni` - dvě slova oddělená čárkou, bez mezery.",
        ])
        .start("headtail")
        .file(
            "headtail/long_file.txt",
            "START of the file - this is line 1\n\
             Line number 2 with some content\n\
             Line number 3 with some content\n\
             Line number 4 with some content\n\
             Line number 5 with some content\n\
             Line number 6 with some content\n\
             Line number 7 with some content\n\
             Line number 8 with some content\n\
             Line number 9 with some content\n\
             Line number 10 with some content\n\
             Line number 11 with some content\n\
             Line number 12 with some content\n\
             Line number 13 with some content\n\
             Line number 14 with some content\n\
             Line number 15 with some content\n\
             Line number 16 with some content\n\
             Line number 17 with some content\n\
             Line number 18 with some content\n\
             Line number 19 with some content\n\
             Line number 20 with some content\n\
             Line number 21 with some content\n\
             Line number 22 with some content\n\
             Line number 23 with some content\n\
             Line number 24 with some content\n\
             Line number 25 with some content\n\
             Line number 26 with some content\n\
             Line number 27 with some content\n\
             Line number 28 with some content\n\
             Line number 29 with some content\n\
             Line number 30 with some content\n\
             Line number 31 with some content\n\
             Line number 32 with some content\n\
             Line number 33 with some content\n\
             Line number 34 with some content\n\
             Line number 35 with some content\n\
             Line number 36 with some content\n\
             Line number 37 with some content\n\
             Line number 38 with some content\n\
             Line number 39 with some content\n\
             Line number 40 with some content\n\
             Line number 41 with some content\n\
             Line number 42 with some content\n\
             Line number 43 with some content\n\
             Line number 44 with some content\n\
             Line number 45 with some content\n\
             Line number 46 with some content\n\
             Line number 47 with some content\n\
             Line number 48 with some content\n\
             Line number 49 with some content\n\
             END of the file - this is line 50\n",
        )
        .answer(
            parts(
                ",",
                [
                    exact("START")
                        .error("První slovo není správně. Použijte 'head -n 1 long_file.txt'."),
                    exact("END")
                        .error("Poslední slovo není správně. Použijte 'tail -n 1 long_file.txt'."),
                ],
            )
            .format_message("Formát: první_slovo,poslední_slovo (např. AHOJ,SVET)")
            .required("Zadejte odpověď ve formátu: první_slovo,poslední_slovo")
            .secret()
            .accepts(["START,END", "START, END", " START , END "])
            .rejects(["start,end", "END,START", "START", "START,END,"]),
        )
        .solution(concat!(
            "shellgame submit \"$(head -n 1 long_file.txt | cut -d' ' -f1),",
            "$(tail -n 1 long_file.txt | cut -d' ' -f1)\"",
        ))
        .success("Správně! Head a tail jsou skvělé pro rychlý náhled do souborů.")
}

fn l7() -> Level {
    level(7, "Počítání (wc)")
        .text(include_str!("07-wc.md"))
        .hints([
            "wc -l počítá řádky, wc -w počítá slova.",
            "Spusťte oba příkazy na `article.txt` a zapište si obě čísla.",
            "Odevzdejte je v pořadí řádky,slova — bez mezery za čárkou.",
        ])
        .start("wc")
        .file(
            "wc/article.txt",
            "Linux je svobodný operační systém.\n\
             Byl vytvořen Linusem Torvaldsem v roce 1991.\n\
             Dnes pohání většinu serverů na internetu.\n\
             Je základem systému Android a mnoha dalších.\n\
             Open source komunita ho neustále vylepšuje.\n",
        )
        .answer(
            parts(
                ",",
                [
                    int(5)
                        .mistake(
                            31,
                            "První číslo má být počet řádků, ne slov — vypadá to, že máte \
                             hodnoty prohozené. Pořadí je řádky,slova.",
                        )
                        .error("Počet řádků není správně. Použijte 'wc -l article.txt'.")
                        .invalid("Obě hodnoty musí být čísla."),
                    int(31)
                        .mistake(
                            5,
                            "Druhé číslo má být počet slov, ne řádků. Ten už jste zapsali jako \
                             první hodnotu.",
                        )
                        .error("Počet slov není správně. Použijte 'wc -w article.txt'.")
                        .invalid("Obě hodnoty musí být čísla."),
                ],
            )
            .format_message("Formát: řádky,slova (např. 10,50) — na pořadí obou čísel záleží.")
            .required("Zadejte odpověď ve formátu: řádky,slova")
            .accepts(["5,31", "5, 31"])
            .rejects(["31,5", "5", "5,31,0"]),
        )
        .solution(r#"shellgame submit "$(wc -l < article.txt),$(wc -w < article.txt)""#)
        .success("Správně! Příkaz wc je nepostradatelný pro rychlou analýzu souborů.")
}

fn l8() -> Level {
    level(8, "Řazení a odstranění duplicit")
        .text(include_str!("08-sort-uniq.md"))
        .hints([
            "Příkaz uniq odstraní duplikáty, ale jen sousedící! Proto nejdřív sort.",
            "Řetězec: sort → uniq → wc -l spočítá unikátní řádky.",
            "Spusťte 'sort visitors.txt | uniq | wc -l' a odevzdejte číslo z výstupu.",
        ])
        .start("sort")
        .file(
            "sort/visitors.txt",
            "Alice\n\
             Bob\n\
             Charlie\n\
             Alice\n\
             David\n\
             Bob\n\
             Eve\n\
             Alice\n",
        )
        .answer(
            int(5)
                .mistake(
                    8,
                    "Spočítali jste všechny řádky, ne unikátní. Zkuste: sort visitors.txt | uniq \
                     | wc -l",
                )
                .mistake(
                    3,
                    "Možná jste spočítali jen duplikáty. Hledáme počet unikátních jmen.",
                )
                .secret()
                .accepts(["5", " 5 "])
                .rejects(["8", "3", "five"]),
        )
        .solution(r#"shellgame submit "$(sort visitors.txt | uniq | wc -l)""#)
        .success("Správně! Sort | uniq je klasická kombinace pro práci s daty.")
}

fn l9() -> Level {
    level(9, "Výzva: Přesměrování a roury")
        .text(include_str!("09-vyzva.md"))
        .hints([
            "První přesměrování má soubor vytvořit, druhé musí zachovat jeho obsah. Výpis \
             pak pošlete rourou do počítadla řádků.",
            "Soubor vytvoříte pomocí `echo \"Hello World\" > message.txt` a druhý řádek \
             přidáte přes `>>`.",
            "Počet položek zjistíte příkazem `ls | wc -l`.",
        ])
        .start("challenge")
        .clean(["challenge"])
        .file("challenge/sample1.txt", "sample")
        .file("challenge/sample2.txt", "sample")
        .answer(
            int(3)
                .mistake(
                    2,
                    "Spočítali jste jen sample1.txt a sample2.txt. Vytvořili jste message.txt?",
                )
                .secret()
                .accepts(["3"])
                .rejects(["2", "4"]),
        )
        .require(
            content("challenge/message.txt")
                .exact("Hello World\nGoodbye")
                .strip()
                .message(
                    "message.txt musí obsahovat řádky 'Hello World' a 'Goodbye' v tomto pořadí.",
                )
                .missing(
                    r#"Chybí message.txt. Vytvořte pomocí 'echo "Hello World" > message.txt'."#,
                ),
        )
        .solution(script([
            r#"echo "Hello World" > message.txt"#,
            r#"echo "Goodbye" >> message.txt"#,
            r#"shellgame submit "$(ls | wc -l)""#,
        ]))
        .success("Skvělé! Přesměrování i roury máte v malíku!")
}

fn l10() -> Level {
    level(10, "Interaktivní vstup a EOF")
        .text(include_str!("10-eof.md"))
        .hints([
            "`cat` bez názvu vstupního souboru čte standardní vstup; EOF mu oznámí, že už \
             žádná data nepřijdou.",
            "Po `cat > poznamka.txt` napište oba řádky. Ctrl+D použijte až na novém prázdném \
             řádku.",
            "Jestli jste použili Ctrl+C nebo udělali překlep, spusťte `shellgame reset` a \
             zopakujte zápis s Ctrl+D.",
        ])
        .start("stdin")
        .clean(["stdin/poznamka.txt"])
        .dir("stdin")
        .answer(
            exact("poznamka.txt").required("Odevzdejte název souboru: shellgame submit <název>"),
        )
        .require(
            content("stdin/poznamka.txt")
                .exact("První řádek\nDruhý řádek\n")
                .message(
                    "Soubor neobsahuje přesně oba zadané řádky. Častá příčina je Ctrl+C: to \
                     příkaz přeruší uprostřed práce, takže text zůstane useknutý nebo vůbec \
                     nedojde na disk. Vstup řádně ukončí až Ctrl+D na prázdném řádku. Začněte \
                     znovu po `shellgame reset`.",
                )
                .missing(
                    "Soubor chybí. Začněte příkazem `cat > poznamka.txt`; pokud jste ho \
                     přerušili pomocí Ctrl+C, nemusel vzniknout vůbec.",
                ),
        )
        .allow_empty()
        .solution(script([
            "printf '%s\\n' 'První řádek' 'Druhý řádek' | cat > poznamka.txt",
            "shellgame submit 'poznamka.txt'",
        ]))
        .success("Správně! EOF ukončilo vstup a `cat` soubor uzavřel.")
}

fn l11() -> Level {
    level(11, "Návratový kód: && a ||")
        .text(include_str!("11-navratovy-kod.md"))
        .hints([
            "Návratový kód 0 značí úspěch, nenulový kód neúspěch. Operátory sledují právě \
             tento stav.",
            "Za `&&` pokračuje úspěšný příkaz; za `||` pokračuje neúspěšný. První řetězec \
             soubor zakládá (stačí přepsání), druhý k němu jen přidává.",
            "Spusťte `true && echo \"stav: uspech\" > status.txt` a potom `false || echo \
             \"stav: neuspech\" >> status.txt`.",
        ])
        .start("status")
        .clean(["status/status.txt"])
        .dir("status")
        .answer(exact("status.txt").required("Odevzdejte název souboru: shellgame submit <název>"))
        .require(
            content("status/status.txt")
                .exact("stav: uspech\nstav: neuspech\n")
                .message(
                    "status.txt musí obsahovat přesně oba řádky ve správném pořadí. Zbyl-li jen \
                     jeden řádek, přepsalo druhé přesměrování to první.",
                )
                .missing("Chybí status.txt. Spusťte oba zadané řetězce příkazů."),
        )
        .solution(script([
            r#"true && echo "stav: uspech" > status.txt"#,
            r#"false || echo "stav: neuspech" >> status.txt"#,
            "shellgame submit 'status.txt'",
        ]))
        .success("Výborně! Dokončili jste Sekci 9 a umíte reagovat na stav příkazu.")
}
