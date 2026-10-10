//! Section 8: permissions.

use crate::engine::prelude::*;
use crate::levels::script;

/// Section 8 and its levels.
pub fn section() -> Section {
    Section::new(8, "level-8")
        .intro(intro("Sekce 8: Oprávnění", include_str!("00-uvod.md")))
        .levels([l1(), l2(), l3(), l4(), l5(), l6(), l7()])
}

fn l1() -> Level {
    level(1, "Hledání spustitelného souboru")
        .text(include_str!("01-spustitelny-soubor.md"))
        .hints([
            "Podrobný výpis souborů včetně sloupců s oprávněními získáte přepínačem '-l'.",
            "Spusťte 'ls -l' a rozdělte práva po znaku typu na trojice pro vlastníka, \
             skupinu a ostatní.",
            "Hledejte 'x' v první trojici (pozice vlastníka), ne v trojici skupiny nebo \
             ostatních.",
        ])
        .start("executables")
        .file_mode("executables/test.sh", "#!/bin/bash", 0o654)
        .file_mode("executables/data.sh", "#!/bin/bash", 0o645)
        .file_mode("executables/script.sh", "#!/bin/bash\necho Hi", 0o744)
        .answer(
            exact("script.sh")
                .error(
                    "Tenhle soubor nemá 'x' v první trojici. Ve výpisu 'ls -l' čtěte práva po \
                     znaku typu po trojicích: vlastník, skupina, ostatní. 'x' u skupiny nebo \
                     ostatních se nepočítá.",
                )
                .secret()
                .accepts(["script.sh"])
                .rejects(["test.sh", "data.sh", "script"]),
        )
        .solution(r#"shellgame submit "$(ls -l | grep '^-..x' | grep -o '[^ ]*$')""#)
        .success("Správně! Práva se čtou po trojicích a rozhoduje ta, která odpovídá vaší roli.")
}

fn l2() -> Level {
    level(2, "Nastavení spustitelnosti")
        .text(include_str!("02-nastaveni-spustitelnosti.md"))
        .hints([
            "Právo 'x' (execute) je potřeba pro spuštění. Jak ho přidáte pro vlastníka \
             (user)?",
            "Syntaxe chmod: chmod kdo+co soubor. 'u' = user, 'x' = execute.",
            "Použijte 'chmod u+x run_me.sh'.",
        ])
        .start("permissions")
        .file_mode("permissions/run_me.sh", "#!/bin/bash\necho Run me", 0o644)
        .answer(exact("run_me.sh"))
        .require(perm_bits("permissions/run_me.sh").has(0o100).message(
            "Vlastník stále nemá právo 'x'. Zkontrolujte 'ls -l run_me.sh': rozhoduje \
             první trojice po znaku typu, ne trojice skupiny ani ostatních.",
        ))
        .allow_empty()
        .solution(script([
            "chmod u+x run_me.sh",
            "shellgame submit 'run_me.sh'",
        ]))
        .success("Správně! Symbolický chmod se skládá ze tří částí: kdo, operace a které právo.")
}

fn l3() -> Level {
    level(3, "Odebrání práva zápisu")
        .text(include_str!("03-odebrani-prava-zapisu.md"))
        .hints([
            "'a' znamená all (všechny: vlastníka, skupinu i ostatní), '-w' odebírá právo \
             zápisu.",
            "Tvar je 'chmod <kdo><operace><právo> <soubor>': tady je kdo = všichni, operace \
             = odebrat a právo = zápis.",
            "Spusťte 'chmod a-w config.readonly'.",
        ])
        .start("permissions")
        .file_mode("permissions/config.readonly", "Do not touch", 0o644)
        .require(
            perm_bits("permissions/config.readonly")
                .lacks(0o222)
                .message(
                    "Někde právo zápisu zůstalo. V 'ls -l config.readonly' projděte všechny tři \
                     trojice po řadě: 'w' nesmí být ani u vlastníka, ani u skupiny, ani u \
                     ostatních.",
                ),
        )
        .solution(script(["chmod a-w config.readonly", "shellgame submit"]))
        .success("Správně! Symbolický zápis mění právo jen vybraným trojicím a zbytek nechává být.")
}

fn l4() -> Level {
    level(4, "Číselný zápis")
        .text(include_str!("04-ciselny-zapis.md"))
        .hints([
            "Oktalový zápis: 7 = 4+2+1 (rwx) pro vlastníka, 5 = 4+1 (r-x) pro skupinu a \
             ostatní.",
            "Tvar je 'chmod <tři číslice> <soubor>': každá číslice patří jedné trojici v \
             pořadí vlastník, skupina, ostatní a je součtem 4 (r), 2 (w) a 1 (x).",
            "Spusťte 'chmod 755 deploy.sh' a výsledek zkontrolujte pomocí 'ls -l deploy.sh'.",
        ])
        .start("permissions")
        .file_mode(
            "permissions/deploy.sh",
            "#!/bin/sh\necho 'Nasazuji aplikaci'\n",
            0o600,
        )
        .require(mode("permissions/deploy.sh", 0o755).message(
            "Režim zatím nesedí. Přečtěte si aktuální stav příkazem 'ls -l deploy.sh' a \
             přeložte každou trojici zpět na číslici (r=4, w=2, x=1, '-'=0). Nejčastější \
             chybou je právo zápisu navíc u skupiny nebo ostatních.",
        ))
        .solution(script(["chmod 755 deploy.sh", "shellgame submit"]))
        .success(
            "Správně! Číselný režim nastavuje všechny tři trojice najednou na přesnou hodnotu.",
        )
}

fn l5() -> Level {
    level(5, "Kontrolní úkol: Oprávnění souborů")
        .text(include_str!("05-kontrolni-ukol-opravneni.md"))
        .hints([
            "U každého souboru určete skupinu uživatelů (u, g, o nebo a), operaci (+, - nebo \
             =) a požadované právo.",
            "První, druhý a čtvrtý úkol lze vyřešit symbolicky; u shared.txt nastavte přesný \
             číselný režim.",
            "Použijte 'chmod u+x script.sh', 'chmod a-w secret.txt', 'chmod 644 shared.txt' \
             a 'chmod g+r team.txt'.",
        ])
        .start("challenge")
        .clean(["challenge"])
        .file_mode("challenge/script.sh", "#!/bin/bash\necho 'Hello'\n", 0o600)
        .file_mode("challenge/secret.txt", "Top secret!\n", 0o666)
        .file_mode("challenge/shared.txt", "Shared content\n", 0o777)
        .file_mode("challenge/team.txt", "Team notes\n", 0o600)
        .require(
            perm_bits("challenge/script.sh")
                .has(0o100)
                .message("Vlastník stále nemůže spustit script.sh."),
        )
        .require(
            perm_bits("challenge/secret.txt")
                .lacks(0o222)
                .message("secret.txt má stále alespoň jedno právo zápisu."),
        )
        .require(
            mode("challenge/shared.txt", 0o644)
                .message("shared.txt nemá přesně oprávnění rw-r--r--."),
        )
        .require(
            mode("challenge/team.txt", 0o640)
                .message("team.txt nemá právo čtení pro skupinu při zachování ostatních práv."),
        )
        .solution(script([
            "chmod u+x script.sh",
            "chmod a-w secret.txt",
            "chmod 644 shared.txt",
            "chmod g+r team.txt",
            "shellgame submit",
        ]))
        .success(
            "Správně! Oprávnění běžných souborů máte pod kontrolou. Nyní se podíváme na \
             adresáře.",
        )
}

fn l6() -> Level {
    level(6, "Kdo dovoluje zápis do adresáře?")
        .text(include_str!("06-zapis-do-adresare.md"))
        .hints([
            "Právo 'w' na soubor řídí změnu jeho obsahu. Vytvoření nového jména je operace \
             nad adresářem.",
            "Režim 550 nedává vlastníkovi adresáře právo 'w'; příkaz 'chmod u+w locked' ho \
             přidá.",
            "Zkopírujte report ven, přidejte 'w' adresáři locked, zopakujte kopii dovnitř a \
             odevzdejte 'adresář'.",
        ])
        .start("directory-write")
        .clean(["directory-write"])
        .dir_mode("directory-write/locked", 0o550)
        .file("directory-write/source.txt", "nová data\n")
        .file("directory-write/locked/report.txt", "existující zpráva\n")
        .answer(
            one_of(["adresář", "adresar", "directory"])
                .ignore_case()
                .error("Zaměřte se na objekt, ve kterém vzniká nové jméno souboru.")
                .required("Odevzdejte odpověď: shellgame submit <odpověď>")
                .accepts(["adresář", "Adresar", "DIRECTORY"])
                .rejects(["soubor", "file"]),
        )
        .require(
            same(
                "directory-write/locked/report.txt",
                "directory-write/recovered.txt",
            )
            .destination_error("Nejdřív zkopírujte existující report z locked ven."),
        )
        .require(
            same(
                "directory-write/source.txt",
                "directory-write/locked/source.txt",
            )
            .destination_error("Po přidání práva zápisu zkopírujte source.txt do locked/."),
        )
        .require(
            dir_mode("directory-write/locked", 0o750)
                .message("Adresář locked nemá očekávaná práva po příkazu 'chmod u+w locked'."),
        )
        .solution(script([
            "cp locked/report.txt recovered.txt",
            "chmod u+w locked",
            "cp source.txt locked/",
            "shellgame submit 'adresář'",
        ]))
        .success("Správně! Vytváření a mazání jmen řídí zapisovatelnost nadřazeného adresáře.")
}

fn l7() -> Level {
    level(7, "Právo x u adresáře")
        .text(include_str!("07-pravo-x.md"))
        .hints([
            "U adresáře neznamená 'x' spuštění programu, ale možnost průchodu cestou.",
            "Po 'chmod a-x ..' nadřazený adresář stále může mít 'r', ale cesta přes něj \
             nefunguje.",
            "Obnovte vlastníkovi průchod příkazem 'chmod u+x ..', přejděte do parent a \
             odevzdejte 'x'.",
        ])
        .start("directory-traverse/parent/child")
        .clean(["directory-traverse"])
        .dir_mode("directory-traverse/parent", 0o755)
        .dir_mode("directory-traverse/parent/child", 0o755)
        .answer(
            one_of(["x", "execute", "průchod", "pruchod"])
                .ignore_case()
                .error("Hledejte právo, které u adresáře umožňuje procházet přes něj cestou.")
                .required("Odevzdejte označení práva: shellgame submit <písmeno>")
                .accepts(["x", "X", "execute", "průchod"])
                .rejects(["w", "r", "write"]),
        )
        .require(at("directory-traverse/parent"))
        .require(
            dir_mode("directory-traverse/parent", 0o744)
                .message("Obnovte právo průchodu pouze vlastníkovi nadřazeného adresáře."),
        )
        .solution(script([
            "chmod a-x ..",
            "chmod u+x ..",
            "cd ..",
            "shellgame submit 'x'",
        ]))
        .success("Výborně! Dokončili jste Sekci 8 a rozumíte právům souborů i adresářů.")
}
