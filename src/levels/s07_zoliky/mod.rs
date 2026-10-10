//! Section 7: wildcards.

use crate::engine::prelude::*;
use crate::levels::script;

/// Section 7 and its levels.
pub fn section() -> Section {
    Section::new(7, "level-7")
        .intro(intro(
            "Sekce 7: Žolíky (Wildcards)",
            include_str!("00-uvod.md"),
        ))
        .levels([l1(), l2(), l3(), l4(), l5()])
}

fn l1() -> Level {
    level(1, "Hvězdička *")
        .text(include_str!("01-hvezdicka.md"))
        .hints([
            "Hvězdička (*) nahrazuje libovolný počet znaků.",
            "Příkaz 'ls *.jpg' vypíše všechny soubory s příponou .jpg.",
            "Použijte 'cp *.jpg images/' pro zkopírování všech souborů s příponou .jpg do \
             adresáře images.",
        ])
        .start("star")
        .clean(["star/images"])
        .dir("star/images")
        .file("star/photo1.jpg", "")
        .file("star/photo2.jpg", "")
        .file("star/notes.txt", "")
        .require(file("star/images/photo1.jpg"))
        .require(file("star/images/photo2.jpg"))
        .require(no_file("star/images/notes.txt"))
        .solution(script(["cp *.jpg images/", "shellgame submit"]))
        .success(
            "Správně! Jediný vzor s hvězdičkou zastoupil libovolný zbytek názvu, takže jste \
             nemuseli soubory vypisovat po jednom.",
        )
}

fn l2() -> Level {
    level(2, "Otazník ?")
        .text(include_str!("02-otaznik.md"))
        .hints([
            "Otazník nahradí právě jeden znak. Kolik znaků je mezi 'data' a '.txt' v \
             data1.txt?",
            "data?.txt zachytí data1.txt a data2.txt, ale ne data10.txt (tam jsou dva \
             znaky).",
            "V Bashi použijte `cp data?.txt short_data/`. Z fish: `bash -c 'cp data?.txt \
             short_data/'`.",
        ])
        .start("question")
        .clean(["question/short_data"])
        .dir("question/short_data")
        .file("question/data1.txt", "")
        .file("question/data2.txt", "")
        .file("question/data10.txt", "")
        .require(file("question/short_data/data1.txt"))
        .require(file("question/short_data/data2.txt"))
        .require(no_file("question/short_data/data10.txt"))
        // The instructions teach `bash -c '...'` for fish; it works in bash too.
        .solution(script([
            "bash -c 'cp data?.txt short_data/'",
            "shellgame submit",
        ]))
        .success(
            "Správně! Otazník zastupuje právě jeden znak, proto se dvouciferný název do výběru \
             nedostal.",
        )
}

fn l3() -> Level {
    level(3, "Výběr znaků []")
        .text(include_str!("03-vyber-znaku.md"))
        .hints([
            "Hranaté závorky definují množinu povolených znaků na dané pozici.",
            "[ab] znamená 'a nebo b', takže file_[ab].txt zachytí file_a.txt a file_b.txt.",
            "V Bashi použijte `cp file_[ab].txt ab_files/`. Z fish: `bash -c 'cp \
             file_[ab].txt ab_files/'`.",
        ])
        .start("brackets")
        .clean(["brackets/ab_files"])
        .dir("brackets/ab_files")
        .file("brackets/file_a.txt", "")
        .file("brackets/file_b.txt", "")
        .file("brackets/file_c.txt", "")
        .require(file("brackets/ab_files/file_a.txt"))
        .require(file("brackets/ab_files/file_b.txt"))
        .require(no_file("brackets/ab_files/file_c.txt"))
        .solution(script([
            "bash -c 'cp file_[ab].txt ab_files/'",
            "shellgame submit",
        ]))
        .success(
            "Správně! Závorky povolují na jedné pozici jen vyjmenované znaky, takže vzor vybral \
             přesně požadovanou dvojici.",
        )
}

fn l4() -> Level {
    level(4, "Třídy znaků [[:lower:]]")
        .text(include_str!("04-tridy-znaku.md"))
        .hints([
            "Třída [[:lower:]] vybere na dané pozici právě jedno malé písmeno bez závislosti \
             na pořadí znaků v locale.",
            "Vzor [[:lower:]]*.txt vybere názvy začínající malým písmenem a končící příponou \
             .txt.",
            "V Bashi použijte `cp [[:lower:]]*.txt lowercase/`. Z fish: `bash -c 'cp \
             [[:lower:]]*.txt lowercase/'`.",
        ])
        .start("ranges")
        .clean(["ranges/lowercase"])
        .dir("ranges/lowercase")
        .file("ranges/apple.txt", "")
        .file("ranges/Banana.txt", "")
        .file("ranges/cherry.txt", "")
        .file("ranges/Date.txt", "")
        .require(file("ranges/lowercase/apple.txt"))
        .require(file("ranges/lowercase/cherry.txt"))
        .require(no_file("ranges/lowercase/Banana.txt"))
        .require(no_file("ranges/lowercase/Date.txt"))
        .solution(script([
            "bash -c 'cp [[:lower:]]*.txt lowercase/'",
            "shellgame submit",
        ]))
        .success(
            "Správně! Třída znaků vybírá podle klasifikace písmene, takže výsledek nezávisí na \
             řazení znaků v nastaveném locale.",
        )
}

fn l5() -> Level {
    level(5, "Souhrn Sekce 7")
        .text(include_str!("05-souhrn-sekce-7.md"))
        .hints([
            "Každý krok má procvičit jiný vzor. Nejdřív si pomocí 'ls' ověřte, zda vzor \
             nevybírá některý z decoy souborů.",
            "Logy potřebují žolík pro libovolný zbytek názvu, krátká data žolík pro právě \
             jeden znak a reporty znakovou třídu se dvěma povolenými písmeny; každý \
             sestavený vzor ověřte pomocí `ls`.",
            "Bash: `cp *.log logs/; cp data?.txt short_data/; cp report_[ab].csv \
             selected_reports/`. Fish: `bash -c 'cp *.log logs/; cp data?.txt short_data/; \
             cp report_[ab].csv selected_reports/'`.",
        ])
        .start("challenge")
        .clean(["challenge"])
        .dirs([
            "challenge/logs",
            "challenge/short_data",
            "challenge/selected_reports",
        ])
        .file("challenge/app.log", "log1")
        .file("challenge/error.log", "log2")
        .file("challenge/debug.log", "log3")
        .file("challenge/notes.txt", "txt")
        .file("challenge/data1.txt", "short1")
        .file("challenge/data2.txt", "short2")
        .file("challenge/data10.txt", "long")
        .file("challenge/report_a.csv", "a\n")
        .file("challenge/report_b.csv", "b\n")
        .file("challenge/report_c.csv", "c\n")
        .file("challenge/script.sh", "#!/bin/bash\n")
        .answer(
            parts(
                ",",
                [
                    int(3)
                        .error("Počet zkopírovaných logů není správně. Zkontrolujte vzor s `*`.")
                        .invalid("Všechny tři hodnoty musí být čísla."),
                    int(2)
                        .error(
                            "Počet krátkých datových názvů není správně. Zkontrolujte vzor s \
                             `?`.",
                        )
                        .invalid("Všechny tři hodnoty musí být čísla."),
                    int(2)
                        .error("Počet vybraných reportů není správně. Zkontrolujte znakovou třídu.")
                        .invalid("Všechny tři hodnoty musí být čísla."),
                ],
            )
            .format_message("Formát: logy,data,reporty (tři čísla oddělená čárkou)")
            .secret()
            .accepts(["3,2,2", "3, 2, 2"])
            .rejects(["3,2", "2,2,3", "3,2,3", "a,b,c"]),
        )
        .require(file("challenge/logs/app.log"))
        .require(file("challenge/logs/error.log"))
        .require(file("challenge/logs/debug.log"))
        .require(file("challenge/short_data/data1.txt"))
        .require(file("challenge/short_data/data2.txt"))
        .require(no_file("challenge/short_data/data10.txt"))
        .require(file("challenge/selected_reports/report_a.csv"))
        .require(file("challenge/selected_reports/report_b.csv"))
        .require(no_file("challenge/selected_reports/report_c.csv"))
        // Bash patterns through `bash -c`, as the final hint teaches for fish.
        .solution(script([
            "bash -c 'cp *.log logs/'",
            "bash -c 'cp data?.txt short_data/'",
            "bash -c 'cp report_[ab].csv selected_reports/'",
            concat!(
                "shellgame submit \"$(ls logs | wc -l),$(ls short_data | wc -l),",
                "$(ls selected_reports | wc -l)\"",
            ),
        ]))
        .success("Výborně! Dokončili jste Sekci 7. Wildcards jsou váš nejlepší přítel!")
}
