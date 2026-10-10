//! Section 5: examining files.

use crate::engine::prelude::*;

/// The first bytes of a JPEG file: enough for `file` to say `JPEG image data`.
const JPEG_HEADER: &[u8] = b"\xff\xd8\xff\xe0\x00\x10JFIF";

/// Section 5 and its levels.
pub fn section() -> Section {
    Section::new(5, "level-5")
        .intro(intro(
            "Sekce 5: Zkoumání souborů",
            include_str!("00-uvod.md"),
        ))
        .levels([l1(), l2(), l3(), l4(), l5(), l6(), l7()])
}

fn l1() -> Level {
    level(1, "Velikost souboru")
        .text(include_str!("01-velikost-souboru.md"))
        .hints([
            "Příkaz 'ls -l' zobrazí podrobnosti o souborech. Který sloupec obsahuje \
             velikost?",
            "Ve výstupu ls -l je velikost v bajtech - hledejte číslo před datem.",
            "Použijte 'ls -l database.db' a podívejte se na pátý sloupec.",
        ])
        .start("sizes")
        .file("sizes/database.db", vec![b'x'; 12345])
        .answer(
            int(12345)
                .secret()
                .accepts(["12345", " 12345 "])
                .rejects(["1234", "123456", "12345.0"]),
        )
        .solution(r#"shellgame submit "$(ls -l database.db | tr -s ' ' | cut -d' ' -f5)""#)
        .success(
            "Správně! Pátý sloupec `ls -l` udává velikost v bajtech; `-h` ji převede do \
             čitelnější podoby.",
        )
}

fn l2() -> Level {
    level(2, "Hledání podle velikosti")
        .text(include_str!("02-hledani-podle-velikosti.md"))
        .hints([
            "Příkaz 'ls -l' zobrazuje podrobný výpis souborů včetně velikosti v bajtech v \
             pátém sloupci.",
            "Spusťte 'ls -l' a hledejte řádek, kde je velikost přesně 1337.",
            "Název souboru je na konci příslušného řádku. Odevzdejte ho příkazem 'shellgame \
             submit <soubor>'.",
        ])
        .optional()
        .start("search")
        .file("search/file_a", vec![b'x'; 1000])
        .file("search/file_b", vec![b'x'; 2000])
        .file("search/file_c", vec![b'x'; 1338])
        .file("search/file_d", vec![b'x'; 1337])
        .answer(
            exact("file_d")
                .mistake(
                    "file_c",
                    "Soubor 'file_c' má 1338 bajtů (o 1 bajt více). Hledejte přesně 1337 bajtů.",
                )
                .mistake(
                    "file_a",
                    "Soubor 'file_a' má 1000 bajtů. Hledejte přesně 1337 bajtů.",
                )
                .mistake(
                    "file_b",
                    "Soubor 'file_b' má 2000 bajtů. Hledejte přesně 1337 bajtů.",
                )
                .accepts(["file_d"])
                .rejects(["file_c", "file_a", "file_b", "D"]),
        )
        .solution(r#"shellgame submit "$(ls -l | grep ' 1337 ' | grep -o '[^ ]*$')""#)
        .success(
            "Správně! Podrobný výpis se čte po sloupcích — stačí porovnat ten jeden, který vás \
             zajímá.",
        )
}

fn l3() -> Level {
    level(3, "Typ souboru")
        .text(include_str!("03-typ-souboru.md"))
        .hints([
            "Příkaz 'file' zkoumá obsah souboru, ne jeho název. Jak zjistíte typ všech \
             souborů najednou?",
            "Zkuste 'file *' nebo 'file file1 file2 file3'. Hledejte 'JPEG' ve výstupu.",
            "Použijte 'file *' a najděte soubor označený jako 'JPEG image data'.",
        ])
        .start("types")
        .file("types/file1", "This is a text file.")
        .file("types/file2", b"\x00\x01\x02\x03")
        .file("types/file3", JPEG_HEADER)
        .answer(
            exact("file3")
                .accepts(["file3"])
                .rejects(["file1", "file2"]),
        )
        .solution(r#"shellgame submit "$(file * | grep JPEG | cut -d: -f1)""#)
        .success("Správně! O typu souboru rozhoduje jeho obsah, ne název ani přípona.")
}

/// 5.4: the log is generated, see `server_log`.
fn l4() -> Level {
    level(4, "Hledání ve velkém souboru")
        .text(include_str!("04-velky-soubor.md"))
        .hints([
            "V less použijte / pro vyhledávání. Napište /CRITICAL a stiskněte Enter.",
            "Nalezený řádek obsahuje číslo na konci. Přečtěte ho.",
            "Pro návrat do shellu stiskněte klávesu 'q' (quit).",
            "Pozor: neodevzdáváte číslo řádku (v závorkách na začátku), ale kód na konci \
             věty.",
        ])
        .start("logs")
        .file("logs/server.log", server_log())
        .answer(
            int(42)
                .mistake(
                    137,
                    "137 je číslo řádku, ne kód na konci. Přečtěte celý CRITICAL řádek.",
                )
                .error(
                    "Tohle není správný kód. Najděte řádek s 'CRITICAL' a přečtěte číslo na \
                     konci.",
                )
                .invalid("Odpověď musí být číslo.")
                .accepts(["42"])
                .rejects(["137", "41", "code"]),
        )
        .solution(r#"shellgame submit "$(grep CRITICAL server.log | grep -o '[0-9]*$')""#)
        .success("Správně! Less usnadňuje hledání ve velkých souborech.")
}

fn l5() -> Level {
    level(5, "Zamaskovaný soubor")
        .text(include_str!("05-zamaskovany-soubor.md"))
        .hints([
            "Příkaz 'file' zkoumá skutečný obsah souboru bez ohledu na jeho příponu.",
            "Místo 'file *' použijte 'file *.jpg' — zkontrolujete jen soubory s příponou \
             .jpg.",
            "Hledejte mezi soubory .jpg ten, u kterého 'file' vypíše 'ASCII text'. \
             Odevzdejte jeho název.",
        ])
        .start("downloads")
        .file("downloads/photo1.jpg", JPEG_HEADER)
        .file("downloads/photo2.jpg", JPEG_HEADER)
        .file("downloads/photo3.jpg", JPEG_HEADER)
        .file("downloads/photo4.jpg", JPEG_HEADER)
        .file("downloads/photo5.jpg", JPEG_HEADER)
        .file("downloads/vacation.jpg", JPEG_HEADER)
        .file("downloads/sunset.jpg", JPEG_HEADER)
        .file("downloads/wallpaper.jpg", JPEG_HEADER)
        .file("downloads/avatar.jpg", JPEG_HEADER)
        .file("downloads/nature.jpg", JPEG_HEADER)
        .file(
            "downloads/secret.jpg",
            "This is actually a secret text file.\n",
        )
        .file("downloads/manual.pdf", b"%PDF-1.4\n%EOF\n")
        .file(
            "downloads/archive.zip",
            b"PK\x03\x04\x14\x00\x00\x00\x08\x00",
        )
        .file(
            "downloads/backup.tar.gz",
            b"\x1f\x8b\x08\x00\x00\x00\x00\x00",
        )
        .file("downloads/music.mp3", b"ID3\x03\x00\x00\x00")
        .file("downloads/notes.txt", "Random notes and meeting minutes.\n")
        .file("downloads/install.sh", "#!/bin/sh\necho 'installing...'\n")
        .file("downloads/setup.log", "[INFO] Setup initialized.\n")
        .file("downloads/table.csv", "id,name,value\n1,alpha,100\n")
        .file(
            "downloads/data.json",
            "{\"status\": \"ok\", \"count\": 42}\n",
        )
        .file("downloads/config.yaml", "env: production\ndebug: false\n")
        .file(
            "downloads/readme.md",
            "# Downloads Readme\nJust downloads.\n",
        )
        .file("downloads/styles.css", "body { margin: 0; padding: 0; }\n")
        .file(
            "downloads/index.html",
            "<!DOCTYPE html><html><body>Test</body></html>\n",
        )
        .file(
            "downloads/script.py",
            "#!/usr/bin/env python3\nprint('hello')\n",
        )
        .file(
            "downloads/checksums.sha256",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
        )
        .file("downloads/report.doc", b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1")
        .file(
            "downloads/app.bin",
            b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        )
        .answer(
            exact("secret.jpg")
                .accepts(["secret.jpg"])
                .rejects(["photo1.jpg", "secret"]),
        )
        .solution(r#"shellgame submit "$(file *.jpg | grep 'ASCII text' | cut -d: -f1)""#)
        .success(
            "Správně! Přípona je jen dohoda — `file` čte skutečný obsah, a tak odhalí i \
             zamaskovaný soubor.",
        )
}

fn l6() -> Level {
    level(6, "Rozpoznání Python skriptu")
        .text(include_str!("06-python-skript.md"))
        .hints([
            "Jste přímo v adresáři 'bin'. Spusťte 'file *' pro zjištění typů všech souborů.",
            "Spusťte 'file *' a hledejte soubor, u kterého výstup uvádí 'Python script'.",
            "Odevzdejte název souboru označeného jako 'Python script'; samotný popis \
             neověřuje právo x.",
        ])
        .start("bin")
        .file("bin/readme.txt", "Just text.")
        .file("bin/run.sh", "#!/bin/bash\necho hello")
        .file("bin/calc.py", "#!/usr/bin/env python3\nprint(1+1)\n")
        .file("bin/program", b"\x7fELF")
        .answer(
            exact("calc.py")
                .mistake(
                    "readme.txt",
                    "To je prostý text, ne skript. Řiďte se přesným popisem z `file *`, ne \
                     příponou názvu.",
                )
                .mistake(
                    "run.sh",
                    "To je shellový skript (`shell script`), ne Python. Přečtěte popis z `file \
                     *` celý.",
                )
                .mistake(
                    "program",
                    "To je zkompilovaný program (ELF), ne skript. Porovnejte popisy z `file *`.",
                )
                .error(
                    "To není Python skript. Rozhoduje přesný popis z `file *`; právo `x` z něj \
                     nevyčtete — to ukazuje až `ls -l`.",
                )
                .accepts(["calc.py"])
                .rejects(["run.sh", "readme.txt", "program", "calc"]),
        )
        .solution(r#"shellgame submit "$(file * | grep 'Python script' | cut -d: -f1)""#)
        .success(
            "Správně! `file` určí typ souboru podle obsahu, ale právo ke spuštění ukáže až `ls \
             -l`.",
        )
}

fn l7() -> Level {
    level(7, "Souhrn Sekce 5")
        .text(include_str!("07-souhrn-sekce-5.md"))
        .hints([
            "Použijte 'file *' k zobrazení typů všech souborů najednou.",
            "Pro prostý text hledejte popis začínající 'ASCII text'. Skripty do tohoto počtu \
             nepatří.",
            "Popis s 'image' znamená obrázek. Popis s 'Python script' znamená Python skript, \
             ne prostý text.",
        ])
        .start("mystery")
        .clean(["mystery"])
        .file("mystery/data.bin", "This is just plain text.\n")
        .file("mystery/notes.xyz", "More text content.\n")
        .file(
            "mystery/config.txt",
            b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\
              \x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc```\x00\x00\x00\x04\x00\x01\xf6\
              \x178U\x00\x00\x00\x00IEND\xaeB`\x82",
        )
        .file(
            "mystery/analyzer.dat",
            "#!/usr/bin/env python3\n\
             import sys\n\
             print('Hello')\n",
        )
        .file("mystery/readme.doc", b"\x7fELF\x02\x01\x01")
        .answer(
            parts(
                ",",
                [
                    int(2)
                        .error(
                            "Počet prostých textových souborů není správně. Počítejte popisy \
                             začínající 'ASCII text', nikoli skripty.",
                        )
                        .invalid("První dvě hodnoty musí být čísla."),
                    int(1)
                        .error("Počet obrázků není správně. Hledejte 'image' ve výstupu 'file *'.")
                        .invalid("První dvě hodnoty musí být čísla."),
                    exact("analyzer.dat")
                        .ignore_case()
                        .error(
                            "Název Python skriptu není správně. Hledejte 'Python' ve výstupu \
                             'file *'.",
                        )
                        .secret(),
                ],
            )
            .format_message("Formát: počet_textových,počet_obrázků,název_skriptu")
            .accepts(["2,1,analyzer.dat", "2, 1, ANALYZER.dat"])
            .rejects([
                "2,1",
                "3,1,analyzer.dat",
                "2,2,analyzer.dat",
                "2,1,notes.xyz",
            ]),
        )
        .solution(concat!(
            "shellgame submit \"$(file -b * | grep -c '^ASCII text'),",
            "$(file -b * | grep -c image),",
            "$(file * | grep 'Python script' | cut -d: -f1)\"",
        ))
        .success("Skvělá detektivní práce! Dokončili jste Sekci 5. Příponám se už nedáte zmást!")
}

/// The 200 lines of `logs/server.log`: routine `INFO` lines, a `WARNING` every tenth line, an
/// `ERROR` every seventh, and one `CRITICAL` line (137) that carries the code to find.
fn server_log() -> String {
    let lines: Vec<String> = (1..=200)
        .map(|line| {
            let entry = match line {
                137 => "CRITICAL: System failure detected - Code 42",
                n if n % 10 == 0 => "WARNING: High memory usage",
                n if n % 7 == 0 => "ERROR: Connection timeout",
                _ => "INFO: Normal operation",
            };
            format!("[{line:03}] {entry}")
        })
        .collect();
    lines.join("\n") + "\n"
}
