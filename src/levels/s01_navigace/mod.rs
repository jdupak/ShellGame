//! Section 1: navigation with `pwd`, `ls` and `cd`.

mod maze;
mod walk;

use crate::engine::prelude::*;
use crate::levels::script;

/// Section 1 and its levels.
pub fn section() -> Section {
    Section::new(1, "level-1")
        .fixture(
            Fixture::new()
                .dirs(["alpha", "delta", "gamma", "patterns", "patterns/omega"])
                .keep_file("alpha/inside.txt", "first step")
                .keep_file("delta/single.dat", "momentum gained")
                .keep_file("patterns/data.txt", "")
                .keep_file("patterns/dog.md", "")
                .keep_file("patterns/drama.log", "")
                .keep_file("patterns/zebra.txt", ""),
        )
        .intro(intro("Navigace", include_str!("00-uvod.md")).start_at_workspace())
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
            l12(),
        ])
}

fn l1() -> Level {
    level(1, "Aktuální umístění")
        .text(include_str!("01-aktualni-umisteni.md"))
        .hints([
            "Příkaz 'pwd' (Print Working Directory) vám ukáže, kde jste. Zkuste ho!",
            "Podívejte se na výstup 'pwd'. Co je za posledním lomítkem?",
            "Basename je poslední část cesty. Z cesty '/home/student/dokumenty' byste \
             odevzdali 'dokumenty'.",
        ])
        .start(".")
        .answer(
            exact("level-1")
                .secret()
                .accepts(["level-1", " level-1 "])
                .rejects(["level-2", "Level-1"]),
        )
        .require(evidence(Marker::Pwd, messages::L1_1_USE_PWD_FIRST))
        .solution(script(["pwd", r#"shellgame submit "$(basename "$PWD")""#]))
        .success(
            "Správně! `pwd` vypisuje celou cestu; poslední část za lomítkem je adresář, ve \
             kterém stojíte.",
        )
}

/// 1.2: the answer may be typed with a trailing slash (tab completion adds one). Literal
/// mistakes are checked first, then the shape of the answer is diagnosed, quoting the
/// player's own text.
fn l2() -> Level {
    /// Answers of the right shape that are not the answer: the wrong first or last letter.
    fn shape(answer: &str) -> Option<String> {
        if answer.starts_with('d') && !answer.ends_with('a') {
            Some(format!(
                "'{answer}' začíná na 'd', ale nekončí na 'a'. Hledáme vzor d...a."
            ))
        } else if answer.ends_with('a') && !answer.starts_with('d') {
            Some(format!(
                "'{answer}' končí na 'a', ale nezačíná na 'd'. Hledáme vzor d...a."
            ))
        } else {
            None
        }
    }

    level(2, "Soubory, nebo adresáře?")
        .text(include_str!("02-soubory-nebo-adresare.md"))
        .hints([
            "Běžný výpis ukáže názvy, ale bez barev z něj typ položky spolehlivě nepoznáte.",
            "Zjistěte v nápovědě k příkazu `ls`, který přepínač zapíná dlouhý výpis.",
            "Otevřete nápovědu pomocí `ls --help` nebo `man ls` a hledejte výraz 'long \
             listing'.",
            "Spusťte `ls -l`: řádek adresáře začíná `d`, řádek běžného souboru `-`.",
        ])
        .start(".")
        .clean(["data", "dog", "dome"])
        .dirs(["delta", "dome"])
        .file("data", "")
        .file("dog", "")
        .answer(
            exact("delta")
                .trim_trailing_slash()
                .mistake(
                    "data",
                    "'data' odpovídá názvem, ale není to adresář: jeho řádek v 'ls -l' začíná \
                     '-'.",
                )
                .mistake("dome", "'dome' je adresář, ale jeho název nekončí na 'a'.")
                .feedback(shape)
                .required("Musíte zadat název adresáře: shellgame submit <hodnota>")
                .accepts(["delta", "delta/", " delta "])
                .rejects(["Delta", "data", "data/", "dome", "dog/", "dxa"]),
        )
        .solution(concat!(
            "shellgame submit \"$(ls -l | grep '^d' | grep -o '[^ ]*$' | ",
            "grep '^d.*a$')\"",
        ))
        .success("Správně! Podle prvního znaku výpisu `ls -l` jste rozlišili adresář od souboru.")
}

/// 1.3: any answer with a dot gets the "no extension" reminder.
fn l3() -> Level {
    /// A file name with its extension: the answer is the name without it.
    fn with_extension(answer: &str) -> Option<String> {
        answer
            .contains('.')
            .then(|| messages::L1_3_INCLUDED_EXTENSION.to_owned())
    }

    level(3, "Vstup a hlášení (Koncept přípony)")
        .text(include_str!("03-pripona.md"))
        .hints([
            "Použijte 'cd alpha' pro vstup do adresáře alpha.",
            "Vypište obsah pomocí 'ls'. Uvidíte soubor s příponou '.txt'.",
            "Odevzdejte název tohoto souboru, ale vynechejte část '.txt'.",
            "Příklad: Pokud je soubor 'data.csv', odevzdejte 'data'.",
        ])
        .start(".")
        .answer(
            exact("inside")
                .mistake("inside.txt", messages::L1_3_INCLUDED_EXTENSION)
                .mistake(
                    "alpha",
                    "'alpha' je název adresáře, ne souboru uvnitř. Nejdřív vstupte do alpha a \
                     podívejte se, co je uvnitř.",
                )
                .feedback(with_extension)
                .accepts(["inside", " inside "])
                .rejects(["inside.txt", "other.csv", "alpha"]),
        )
        .require(at("alpha"))
        .solution(script([
            "cd alpha",
            r#"shellgame submit "$(ls | sed 's/\.[^.]*$//')""#,
        ]))
        .success("Správně! Správně jste odstranili příponu.")
}

fn l4() -> Level {
    level(4, "Návrat na základnu")
        .text(include_str!("04-navrat-na-zakladnu.md"))
        .hints([
            "Dvě tečky '..' označují nadřazený (rodičovský) adresář.",
            "Použijte příkaz 'cd ..' pro přesun o jednu úroveň výše.",
            "Spusťte 'pwd' a odevzdejte jen název posledního adresáře (část za posledním \
             '/').",
        ])
        .start("alpha")
        .answer(exact("level-1"))
        .require(at(""))
        .require(cd_evidence("Použijte pro návrat přesně příkaz `cd ..`."))
        .cd(policy().exact("..", "Použijte přesně příkaz 'cd ..'."))
        .solution(script([
            "cd ..",
            r#"shellgame submit "$(basename "$PWD")""#,
        ]))
        .success(
            "Správně! `..` neodkazuje na konkrétní jméno, ale vždy na rodiče toho adresáře, kde \
             právě jste.",
        )
}

fn l5() -> Level {
    level(5, "Hluboký ponor")
        .text(include_str!("05-hluboky-ponor.md"))
        .hints([
            "Použijte 'cd' pro vstup do adresářů. Vyzkoušejte klávesu Tab pro automatické \
             doplňování!",
            "Můžete jít postupně: cd gamma, cd deep, cd a...",
            "Nebo najednou: cd gamma/deep/a/b/c (napište 'cd g' a stiskněte Tab)",
            "Po přesunu ověřte polohu příkazem 'pwd' a odevzdejte poslední část cesty.",
        ])
        .start(".")
        .dir("gamma/deep/a/b/c")
        .answer(exact("c"))
        .require(at("gamma/deep/a/b/c"))
        .solution(script([
            "cd gamma/deep/a/b/c",
            r#"shellgame submit "$(basename "$PWD")""#,
        ]))
        .success(
            "Správně! Lomítka spojují kroky do jedné relativní cesty — jeden `cd` zvládne celou \
             větev.",
        )
}

fn l6() -> Level {
    level(6, "Víceúrovňový výstup")
        .text(include_str!("06-viceurovnovy-vystup.md"))
        .hints([
            "Cesty lze řetězit: každé '..' představuje posun o jednu úroveň nahoru.",
            "Pro posun o tři úrovně najednou spojte tři segmenty: 'cd ../../..'.",
            "Polohu si ověřte příkazem 'pwd'. Pokud jste správně, spusťte jen 'shellgame \
             submit'.",
        ])
        .start("gamma/deep/a/b/c")
        .dir("gamma/deep/a/b/c")
        .require(at("gamma/deep"))
        .require(cd_evidence(
            "Vraťte se o tři úrovně jedním příkazem `cd ../../..`.",
        ))
        .cd(policy().exact("../../..", "Použijte jeden příkaz 'cd ../../..'."))
        .solution(script(["cd ../../..", "shellgame submit"]))
        .success("Správně! Úspěšně jste vystoupali o 3 úrovně.")
}

/// 1.7: the maze. The generator lives in `maze.rs`; the `cd` policy keeps the player inside
/// `maze/` until the sanctuary is reached, and arriving there lifts the cage.
fn l7() -> Level {
    level(7, "Navigace v bludišti")
        .text(include_str!("07-bludiste.md"))
        .hints([
            "Sledujte pouze názvy souborů začínající na 'GO_'. Vypište je pomocí 'ls'.",
            "U instrukce 'GO_UP_2_THEN_GO_TO_catacombs' se vraťte o dvě úrovně a vstupte do \
             catacombs.",
            "Spojený návrat lze zapsat jako 'cd ../..'; potom pokračujte příkazem 'cd \
             catacombs'.",
            "Ignorujte položky, které nezačínají na 'GO_'; jsou to odbočky nebo pasti.",
            "Pokud se ztratíte, příkaz 'shellgame reset' vás vrátí na start.",
        ])
        .start("maze/entry")
        .fixture(maze::fixture())
        .require(at(maze::SANCTUARY))
        .cd(policy()
            .within("maze", "Nemůžete opustit bludiště před jeho dokončením.")
            .bare_cd("Nemůžete opustit bludiště. Pokračujte v navigaci uvnitř maze/.")
            .arrive(maze::SANCTUARY))
        .solution(script([
            "cd hall",
            "cd nexus",
            "cd passages",
            "cd tunnel",
            "cd cavern",
            "cd depths",
            "cd ../../catacombs",
            "cd vault",
            "cd chamber",
            "cd ../../labyrinth",
            "cd corridor",
            "cd shrine",
            "cd final",
            "cd final_v2",
            "cd final_final",
            "cd opravdu_final_v2_FINAL",
            "shellgame submit",
        ]))
        .success("Správně! Prošli jste bludištěm až do skutečného cíle!")
}

/// 1.8: an absolute `cd`. Only arriving in `absolute-target` records the evidence, so a
/// bare `cd /` does not count.
fn l8() -> Level {
    level(8, "Skok absolutní cestou")
        .text(include_str!("08-absolutni-cesta.md"))
        .hints([
            "Absolutní cesty začínají na /. Použijte 'pwd' pro zobrazení vaší plné cesty.",
            "K celé cestě vypsané příkazem 'pwd' na startu připojte '/absolute-target'.",
            "Po přesunu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
            "Za 'cd' napište celou sestavenou cestu od /. Pokud obsahuje mezery, uzavřete ji \
             do uvozovek.",
        ])
        .start(".")
        .file("absolute-target/PLACEHOLDER.answer", "")
        .require(at("absolute-target"))
        .require(cd_evidence(messages::ABSOLUTE_CD_NOT_USED))
        .cd(policy()
            .absolute("Musíte použít absolutní cestu (začínající na /).")
            .bare_cd("Musíte zadat cestu.")
            .arrive("absolute-target"))
        .solution(script([r#"cd "$PWD/absolute-target""#, "shellgame submit"]))
        .success("Správně! Dostali jste se sem absolutní cestou.")
}

/// 1.9: the walk home, one directory at a time. The rules live in `walk.rs`.
fn l9() -> Level {
    level(9, "Cesta z kořene domů")
        .text(include_str!("09-cesta-domu.md"))
        .hints([
            "Zjistěte cestu k domovu pomocí 'echo $HOME'.",
            "Začněte v / a vstupujte do každého adresáře v cestě jeden po druhém (bez \
             přeskakování).",
            "Tip: pokud je HOME třeba /home/ada, udělejte: cd / ; cd home ; cd ada.",
            "V tomhle levelu neodevzdáváte textovou odpověď – důležitá je správná sekvence \
             `cd`.",
        ])
        .start_at_workspace()
        .require(at_home())
        .require(cd_evidence(messages::CD_WALK_NOT_COMPLETED))
        .cd(policy()
            .rule(walk::rule)
            .on_move(walk::progress)
            .manual_evidence())
        .resets([Marker::Own("walk")])
        .solution_with(walk::solution)
        .success("Správně! Došli jste domů krok za krokem.")
}

/// 1.10: the expected answer is the name of the player's own home directory.
fn l10() -> Level {
    level(10, "Zkratka pro domov (~)")
        .text(include_str!("10-domov.md"))
        .hints([
            "Znak '~' (vlnovka) je zkratka shellu pro domovský adresář ($HOME). Příkaz 'cd \
             ~' vás přenese domů odkudkoliv.",
            "Na české klávesnici napíšete '~' pomocí Pravého Alt (AltGr) + klávesy '+'.",
            "Po přesunu použijte 'pwd' pro zjištění plné cesty a odevzdejte pouze její \
             poslední část (basename).",
            "Pokud 'pwd' ukáže např. '/home/jan', odevzdáte 'jan' příkazem 'shellgame submit \
             jan'.",
        ])
        .start_at_workspace()
        .answer(exact_with(|ctx| ctx.home_name()))
        .require(at_home())
        .solution(script(["cd ~", r#"shellgame submit "$(basename "$PWD")""#]))
        .success(
            "Správně! Znak `~` (vlnovka) se v shellu vždy rozbalí na váš domovský adresář, ať \
             stojíte kdekoliv v systému.",
        )
}

fn l12() -> Level {
    level(12, "Souhrn")
        .text(include_str!("12-souhrn.md"))
        .hints([
            "Nejdřív si pomocí 'pwd' potvrďte výchozí polohu a cestu rozdělte na jednotlivé \
             adresáře.",
            "Při návratu spočítejte, kolik segmentů musíte z adresáře 'c' odstranit. Každý \
             segment '..' znamená jednu úroveň.",
            "Použijte `cd gamma/deep/a/b/c` a potom `cd ../../../..`. Výsledek ověřte pomocí \
             'pwd'.",
        ])
        .start(".")
        .dir("gamma/deep/a/b/c")
        .require(at("gamma"))
        .require(cd_evidence(
            "Nejdřív použijte `pwd`, přejděte do `c` a vraťte se jedním příkazem o čtyři \
             úrovně.",
        ))
        .cd(policy()
            .from_dir("gamma/deep/a/b/c")
            .after(Marker::Pwd, "Nejdřív použijte příkaz 'pwd'.")
            .exact(
                "../../../..",
                "Z adresáře 'c' použijte jeden příkaz 'cd ../../../..'.",
            ))
        .solution(script([
            "pwd",
            "cd gamma/deep/a/b/c",
            "cd ../../../..",
            "shellgame submit",
        ]))
        .success("Výborně! Ovládáte základy navigace!")
}
