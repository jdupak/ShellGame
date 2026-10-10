//! Section 4: creating and deleting.

use crate::engine::prelude::*;
use crate::levels::script;

/// Section 4 and its levels.
pub fn section() -> Section {
    Section::new(4, "level-4")
        .intro(intro(
            "Sekce 4: Vytváření a mazání",
            include_str!("00-uvod.md"),
        ))
        .levels([l1(), l2(), l3(), l4(), l5(), l6(), l7(), l8()])
}

fn l1() -> Level {
    level(1, "Vytvoření souboru")
        .text(include_str!("01-vytvoreni-souboru.md"))
        .hints([
            "Příkaz 'touch' vytvoří prázdný soubor. Jaký název má mít?",
            "Syntaxe je jednoduchá: touch název_souboru",
            "Použijte 'touch novy_soubor.txt', ověřte výsledek pomocí 'ls -l' a spusťte \
             'shellgame submit'.",
        ])
        .start("creation")
        .clean(["creation/novy_soubor.txt"])
        .dir("creation")
        .require(file("creation/novy_soubor.txt"))
        .require(
            content("creation/novy_soubor.txt")
                .exact("")
                .message("Soubor novy_soubor.txt existuje, ale není prázdný."),
        )
        .solution(script(["touch novy_soubor.txt", "shellgame submit"]))
        .success("Správně! Soubor může vzniknout úplně prázdný a obsah do něj doplníte až později.")
}

fn l2() -> Level {
    level(2, "Vytvoření adresáře")
        .text(include_str!("02-vytvoreni-adresare.md"))
        .hints([
            "Příkaz 'mkdir' slouží k vytvoření nového adresáře.",
            "Spusťte 'mkdir data' pro vytvoření adresáře data.",
            "Ověřte pomocí 'ls -l' (řádek adresáře začíná 'd') a odevzdejte 'data'.",
        ])
        .start("creation")
        .clean(["creation/data"])
        .dir("creation")
        .answer(exact("data").accepts(["data"]).rejects(["data/", "Data"]))
        .require(dir("creation/data").message(
            "Požadovaný adresář zatím neexistuje. Ověřte pomocí 'ls -l', že jste ho \
             založili v aktuálním adresáři a bez překlepu v názvu.",
        ))
        .solution(script(["mkdir data", "shellgame submit 'data'"]))
        .success("Správně! Nový adresář vznikne jediným příkazem a hned do něj můžete vstoupit.")
}

fn l3() -> Level {
    level(3, "Vytváření zanořených adresářů")
        .text(include_str!("03-vytvareni-zanorenych-adresaru.md"))
        .hints([
            "Co se stane, když zkusíte 'mkdir projekt/src/tests' bez přepínače -p?",
            "Přepínač -p (parents) vytvoří i všechny nadřazené adresáře, které chybí.",
            "Použijte 'mkdir -p projekt/src/tests'.",
        ])
        .start("nested")
        .clean(["nested/projekt"])
        .dir("nested")
        .require(dir("nested/projekt/src/tests").message(
            "Struktura projekt/src/tests zatím není celá. Zkontrolujte ji pomocí 'ls -R \
             projekt': bez přepínače -p vytvoří mkdir jen poslední článek cesty, a to \
             jen když jeho rodič už existuje. S -p vzniknou chybějící rodičovské \
             adresáře zároveň s ním.",
        ))
        .solution(script(["mkdir -p projekt/src/tests", "shellgame submit"]))
        .success(
            "Správně! Přepínač -p vytvoří celou cestu naráz, takže rodiče nemusíte zakládat po \
             jednom.",
        )
}

fn l4() -> Level {
    level(4, "Mazání souborů")
        .text(include_str!("04-mazani-souboru.md"))
        .hints([
            "Příkaz rm permanentně maže soubory. Jaký soubor máte smazat?",
            "Syntaxe je jednoduchá: rm název_souboru",
            "Použijte 'rm stary_log.txt'.",
        ])
        .start("cleanup")
        .file("cleanup/stary_log.txt", "old data")
        .answer(exact("stary_log.txt"))
        .require(no_file("cleanup/stary_log.txt").message(
            "Soubor stary_log.txt tu pořád je. Zkontrolujte pomocí 'ls', že mažete ve \
             správném adresáři a že jste název napsali bez překlepu.",
        ))
        .solution(script([
            "rm stary_log.txt",
            "shellgame submit 'stary_log.txt'",
        ]))
        .success(
            "Správně! Mazání v shellu je okamžité a nevratné, proto se vyplatí nejdřív ověřit, \
             co mažete.",
        )
}

fn l5() -> Level {
    level(5, "Mazání adresářů")
        .text(include_str!("05-mazani-adresaru.md"))
        .hints([
            "Zkuste nejdřív 'rmdir temp'. Co se stane?",
            "Pokud adresář není prázdný, rmdir selže. Jaký přepínač potřebujete pro \
             rekurzivní mazání?",
            "Použijte 'rm -r temp' pro smazání adresáře včetně obsahu.",
        ])
        .start("cleanup")
        .file("cleanup/temp/junk.txt", "junk")
        .answer(exact("temp"))
        .require(no_dir("cleanup/temp").message(
            "Zadaný adresář tu pořád je. Pokud rmdir skončil hláškou 'Directory not \
             empty', pracoval přesně podle očekávání: umí odstranit jen prázdný adresář \
             a nesmí se dotknout obsahu. Adresář s obsahem proto smažte až rekurzivní \
             variantou příkazu rm.",
        ))
        .solution(script(["rm -r temp", "shellgame submit 'temp'"]))
        .success(
            "Správně! Prázdný adresář zvládne rmdir, na adresář s obsahem je potřeba rekurzivní \
             mazání.",
        )
}

fn l6() -> Level {
    level(6, "Příprava projektu")
        .text(include_str!("06-priprava-projektu.md"))
        .hints([
            "Nejdřív si rozdělte úkol na adresáře a soubory. Všechny potřebné adresáře lze \
             vytvořit jedním příkazem.",
            "Pro adresáře použijte 'mkdir -p' a pro prázdné soubory 'touch'; oba příkazy \
             přijímají více cest.",
            "Spusťte 'mkdir -p web/css web/js' a potom 'touch web/index.html \
             web/css/style.css'.",
        ])
        .start("project")
        .clean(["project/web"])
        .dir("project")
        .answer(exact("web"))
        .require(
            file("project/web/index.html")
                .message("Chybí soubor web/index.html. Prázdný soubor založí 'touch'."),
        )
        .require(file("project/web/css/style.css").message(
            "Chybí soubor web/css/style.css. Nejdřív musí existovat podadresář css, \
             teprve potom do něj lze soubor založit.",
        ))
        .require(
            dir("project/web/js")
                .message("Chybí adresář web/js. Má zůstat prázdný, ale existovat musí."),
        )
        .solution(script([
            "mkdir -p web/css web/js",
            "touch web/index.html web/css/style.css",
            "shellgame submit 'web'",
        ]))
        .success(
            "Správně! Kostru projektu připravíte předem: adresáře jedním příkazem, prázdné \
             soubory druhým.",
        )
}

fn l7() -> Level {
    level(7, "Úklid nepořádku")
        .text(include_str!("07-uklid-neporadku.md"))
        .hints([
            "Příkaz 'rm' dokáže smazat více souborů najednou, stačí je uvést oddělené \
             mezerami.",
            "Spusťte 'rm error.log temp.dat junk.tmp' (nebo je smažte postupně po jednom).",
            "Ověřte pomocí 'ls', že zůstal jen soubor keep_me.txt, a spusťte 'shellgame \
             submit'.",
        ])
        .start("mess")
        .file("mess/error.log", "")
        .file("mess/temp.dat", "")
        .file("mess/junk.tmp", "")
        .file("mess/keep_me.txt", "important")
        .require(no_file("mess/error.log"))
        .require(no_file("mess/temp.dat"))
        .require(no_file("mess/junk.tmp"))
        .require(file("mess/keep_me.txt").message(
            "Soubor keep_me.txt zmizel, a ten měl zůstat. Typická příčina je příliš \
             široký argument jako 'rm *', který smaže všechno v adresáři. Příště si \
             nejdřív vypište, co by takový příkaz zasáhl. Level obnovíte příkazem \
             'shellgame reset'.",
        ))
        .solution(script([
            "rm error.log temp.dat junk.tmp",
            "shellgame submit",
        ]))
        .success(
            "Správně! Jeden příkaz zvládne víc cest, ale odpovídáte za to, které soubory \
             zasáhne.",
        )
}

fn l8() -> Level {
    level(8, "Souhrn Sekce 4")
        .text(include_str!("08-souhrn-sekce-4.md"))
        .hints([
            "Rozdělte úkol na výsledky: dva adresáře a jeden soubor vytvořit, dvě existující \
             položky odstranit.",
            "Vnořené adresáře vytvoří 'mkdir -p'; prázdný soubor 'touch'. Soubor a prázdný \
             adresář se mažou různými příkazy.",
            "Použijte 'mkdir -p myproject/src myproject/docs', 'touch myproject/README.md', \
             'rm delete_me.txt' a 'rmdir empty_dir'.",
        ])
        .start("challenge")
        .clean(["challenge"])
        .dir("challenge/empty_dir")
        .file("challenge/delete_me.txt", "Delete this!\n")
        .require(
            dir("challenge/myproject/src")
                .message("Chybí adresář myproject/src. Zkontrolujte vytvořenou strukturu."),
        )
        .require(dir("challenge/myproject/docs").message("Chybí adresář myproject/docs."))
        .require(
            file("challenge/myproject/README.md")
                .message("Chybí soubor myproject/README.md. Zkontrolujte jeho název a umístění."),
        )
        .require(no_file("challenge/delete_me.txt").message("Soubor delete_me.txt stále existuje."))
        .require(no_dir("challenge/empty_dir").message("Prázdný adresář empty_dir stále existuje."))
        .solution(script([
            "mkdir -p myproject/src myproject/docs",
            "touch myproject/README.md",
            "rm delete_me.txt && rmdir empty_dir",
            "shellgame submit",
        ]))
        .success("Brilantní! Dokončili jste Sekci 4. Umíte vytvářet i bourat!")
}
