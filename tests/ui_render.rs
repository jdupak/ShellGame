//! Rendering tests for the terminal UI primitives: markdown, panels, text helpers and paging.
//!
//! Everything renders with colour off (`Term::plain`) unless a test is about ANSI output.

use std::fs;
use std::path::{Path, PathBuf};

use shellgame::ui::text::{is_self_contained, strip_ansi, visible_width, wrap_plain};
use shellgame::ui::{Panel, Term, render_markdown, split_pages, style};

// ---------------------------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------------------------

/// Level 2.1 ("Navigace mezi sourozenci") as written in `section2.py`, dedented.
const LEVEL_2_1: &str = r"### Cíl
Přejděte z aktuálního podadresáře do sousedního (sourozeneckého) adresáře.

### Struktura adresářů
```
level-2/
├── start/       ← Zde se nacházíte (start)
└── finish/      ← Váš cíl
```

### Navigace mezi sourozenci
Adresáře `start` a `finish` leží vedle sebe ve stejném rodičovském adresáři (`level-2`).
Protože `finish` neleží uvnitř `start`, nelze použít přímý příkaz `cd finish`.
Musíte se nejprve vrátit k rodiči (`..`) a odtud vstoupit do cíle:
- Ve dvou krocích: nejprve o úroveň výše (`cd ..`) a potom do cíle.
- Nebo v jednom kroku spojenou relativní cestou: `cd ../<cílový_adresář>`.

### Úkol
1. Začínáte v adresáři `level-2/start`
2. Přejděte do sousedního adresáře `finish`
3. V cíli ověřte polohu příkazem `pwd` a odešlete řešení

### Odevzdání
V cílovém adresáři spusťte:
`shellgame submit`
";

/// The tree from the section 1 intro; one line is wider than 80 columns.
const TREE: &str = "\
/                          ← Kořen (root) - začátek všeho
├── home/                  ← Domovské adresáře uživatelů
│   └── student/           ← VÁŠ domovský adresář (~)
│       ├── dokumenty/
│       ├── stažené/
│       └── projekty/
├── tmp/                   ← Dočasné soubory
│   └── shellgame-<user>/  ← Váš pracovní prostor pro hru (např. shellgame-student/)
│       └── level-1/       ← ZDE ZAČÍNÁTE!
├── etc/                   ← Systémové konfigurace
└── usr/                   ← Programy a knihovny";

fn fenced(code: &str) -> String {
    format!("```\n{code}\n```")
}

fn plain(md: &str, width: usize) -> Vec<String> {
    render_markdown(md, &Term::plain(width))
}

fn ansi(md: &str, width: usize) -> Vec<String> {
    render_markdown(md, &Term::ansi(width))
}

/// Every markdown file of the game: the level texts in `src/levels/**/*.md`.
fn markdown_files() -> Vec<PathBuf> {
    fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                out.push(path);
            }
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect(&root.join("levels"), &mut files);
    files.sort();
    files
}

// ---------------------------------------------------------------------------------------------
// Markdown
// ---------------------------------------------------------------------------------------------

#[test]
fn a_real_instructions_block_reflows_into_clean_paragraphs() {
    assert_eq!(
        plain(LEVEL_2_1, 60),
        [
            "Cíl",
            "",
            "Přejděte z aktuálního podadresáře do sousedního",
            "(sourozeneckého) adresáře.",
            "",
            "Struktura adresářů",
            "",
            "  level-2/",
            "  ├── start/       ← Zde se nacházíte (start)",
            "  └── finish/      ← Váš cíl",
            "",
            "Navigace mezi sourozenci",
            "",
            "Adresáře start a finish leží vedle sebe ve stejném",
            "rodičovském adresáři (level-2). Protože finish neleží uvnitř",
            "start, nelze použít přímý příkaz cd finish. Musíte se",
            "nejprve vrátit k rodiči (..) a odtud vstoupit do cíle:",
            "",
            " • Ve dvou krocích: nejprve o úroveň výše (cd ..) a potom do",
            "   cíle.",
            " • Nebo v jednom kroku spojenou relativní cestou:",
            "   cd ../<cílový_adresář>.",
            "",
            "Úkol",
            "",
            " 1 Začínáte v adresáři level-2/start",
            " 2 Přejděte do sousedního adresáře finish",
            " 3 V cíli ověřte polohu příkazem pwd a odešlete řešení",
            "",
            "Odevzdání",
            "",
            "V cílovém adresáři spusťte: shellgame submit",
        ]
    );
}

#[test]
fn source_line_breaks_inside_a_paragraph_do_not_survive() {
    // The same paragraph written on one line or hard-wrapped renders identically.
    let wrapped = "Přejděte z aktuálního\npodadresáře do sousedního\n(sourozeneckého) adresáře.";
    let one_line = "Přejděte z aktuálního podadresáře do sousedního (sourozeneckého) adresáře.";
    for width in [20, 40, 80] {
        assert_eq!(
            plain(wrapped, width),
            plain(one_line, width),
            "width {width}"
        );
    }
}

#[test]
fn numbered_list_hangs_under_the_text_at_width_40() {
    let lines = plain(LEVEL_2_1, 40);
    let start = lines
        .iter()
        .position(|line| line.starts_with(" 1 "))
        .expect("the numbered list is rendered");
    assert_eq!(
        lines[start..start + 5],
        [
            " 1 Začínáte v adresáři level-2/start",
            " 2 Přejděte do sousedního adresáře",
            "   finish",
            " 3 V cíli ověřte polohu příkazem pwd a",
            "   odešlete řešení",
        ]
    );
}

#[test]
fn numbered_lists_of_ten_items_align_their_numbers() {
    let md: String = (1..=10).map(|n| format!("{n}. krok číslo {n}\n")).collect();
    let lines = plain(&md, 40);
    assert_eq!(lines.len(), 10);
    assert_eq!(lines[0], "  1 krok číslo 1");
    assert_eq!(lines[8], "  9 krok číslo 9");
    assert_eq!(lines[9], " 10 krok číslo 10");
}

#[test]
fn bullets_get_a_hanging_indent() {
    let md =
        "- Klávesa Tab (doplňování): Napište začátek jména a stiskněte Tab.\n- Šipky nahoru/dolů";
    assert_eq!(
        plain(md, 36),
        [
            " • Klávesa Tab (doplňování): Napište",
            "   začátek jména a stiskněte Tab.",
            " • Šipky nahoru/dolů",
        ]
    );
}

#[test]
fn nested_lists_are_indented_under_their_parent_item() {
    let md = "1. Příprava\n   - vytvořte adresář `projekt`\n   - přejděte do něj\n2. Práce\n   1. napište soubor\n   2. uložte ho";
    assert_eq!(
        plain(md, 40),
        [
            " 1 Příprava",
            "    • vytvořte adresář projekt",
            "    • přejděte do něj",
            " 2 Práce",
            "    1 napište soubor",
            "    2 uložte ho",
        ]
    );
}

#[test]
fn nested_list_text_wraps_under_its_own_marker() {
    let md = "- vnější\n  - vnitřní položka s dlouhým textem, který se zalomí";
    assert_eq!(
        plain(md, 24),
        [
            " • vnější",
            "    • vnitřní položka s",
            "      dlouhým textem,",
            "      který se zalomí",
        ]
    );
}

#[test]
fn a_fenced_tree_is_printed_verbatim_and_never_wrapped() {
    let md = fenced(TREE);
    let expected: Vec<String> = TREE.lines().map(|line| format!("  {line}")).collect();
    // 120 is wider than every line, 10 narrower than almost all of them: nothing changes.
    for width in [10, 40, 80, 120] {
        assert_eq!(plain(&md, width), expected, "width {width}");
    }
}

#[test]
fn a_tree_inside_a_paragraph_flow_keeps_its_neighbours_apart() {
    let md = format!("Před stromem.\n\n{}\n\nZa stromem.", fenced(TREE));
    let lines = plain(&md, 80);
    assert_eq!(lines.first().map(String::as_str), Some("Před stromem."));
    assert_eq!(lines[1], "");
    assert_eq!(
        lines[2],
        "  /                          ← Kořen (root) - začátek všeho"
    );
    assert_eq!(lines[lines.len() - 2], "");
    assert_eq!(lines.last().map(String::as_str), Some("Za stromem."));
}

#[test]
fn h1_and_h2_are_rules_across_the_width() {
    for width in [30, 40, 80] {
        let lines = plain(
            "# Sekce 7: Žolíky (Wildcards)\n\n## Kdy potřebujete Bash?",
            width,
        );
        assert_eq!(lines.len(), 3);
        for line in [&lines[0], &lines[2]] {
            assert_eq!(visible_width(line), width, "{line:?}");
            assert!(line.ends_with('─'), "{line:?}");
        }
        assert!(lines[0].starts_with("Sekce 7: Žolíky (Wildcards) ─"));
        assert!(lines[2].starts_with("Kdy potřebujete Bash? ─"));
    }
}

#[test]
fn h3_and_deeper_are_plain_text_without_a_rule() {
    assert_eq!(plain("### Klíčové koncepty", 40), ["Klíčové koncepty"]);
    assert_eq!(plain("#### Bezpečnost", 40), ["Bezpečnost"]);
}

#[test]
fn h3_is_bold_when_colour_is_on() {
    let bold = style::BOLD.render().to_string();
    assert_eq!(
        ansi("### Klíčové koncepty", 40),
        [format!("{bold}Klíčové koncepty\x1b[0m")]
    );
    // Each word keeps the style when the heading wraps.
    let wrapped = ansi("### Klíčové koncepty sekce", 14);
    assert_eq!(
        wrapped,
        [
            format!("{bold}Klíčové\x1b[0m"),
            format!("{bold}koncepty sekce\x1b[0m"),
        ]
    );
}

#[test]
fn h1_and_h2_text_is_underlined_when_colour_is_on() {
    let underline = style::H1.render().to_string();
    for md in ["# Nadpis", "## Nadpis"] {
        let lines = ansi(md, 30);
        assert!(
            lines[0].starts_with(&format!("{underline}Nadpis\x1b[0m ")),
            "{lines:?}"
        );
    }
}

#[test]
fn inline_styles_follow_the_palette() {
    let lines = ansi("**tučně** *šikmo* `kód`", 40);
    let strong = style::STRONG.render().to_string();
    let emphasis = style::EMPHASIS.render().to_string();
    let code = style::INLINE_CODE.render().to_string();
    assert_eq!(
        lines,
        [format!(
            "{strong}tučně\x1b[0m {emphasis}šikmo\x1b[0m {code}kód\x1b[0m"
        )]
    );
}

#[test]
fn block_quotes_are_prefixed_on_every_line() {
    let md = "> ⚠️ **Pozor na rozsahy:** `[a-z]` a `[A-Z]` vycházejí z pořadí znaků\n> nastaveného locale.";
    let lines = plain(md, 36);
    assert_eq!(
        lines,
        [
            "▌ ⚠ Pozor na rozsahy: [a-z] a [A-Z]",
            "▌ vycházejí z pořadí znaků",
            "▌ nastaveného locale.",
        ]
    );
    assert!(lines.iter().all(|line| visible_width(line) <= 36));
}

#[test]
fn raw_text_that_looks_like_markup_stays_text_in_code_and_html() {
    let md = "Cesta `shellgame-<user>/` a <příkaz> zůstanou.\n\n```\n<user> **ne tučně**\n```";
    assert_eq!(
        plain(md, 60),
        [
            "Cesta shellgame-<user>/ a <příkaz> zůstanou.",
            "",
            "  <user> **ne tučně**"
        ]
    );
}

// ---------------------------------------------------------------------------------------------
// Panels
// ---------------------------------------------------------------------------------------------

#[test]
fn panel_with_markdown_matches_the_documented_sample() {
    let md = "### Cíl\nPřejděte do adresáře `finish` pomocí **jednoho** příkazu `cd`.\n\n```\nlevel-2/\n├── start/    ← Jste tady\n└── finish/   ← Cíl\n```\n\n1. Ověřte polohu příkazem `pwd`\n2. Odešlete řešení\n";
    let panel = Panel::new(style::CYAN).title("2.1: Sourozenci");
    let lines = panel.render_markdown(md, &Term::plain(60));
    assert_eq!(
        lines,
        [
            "╭──────────────────── 2.1: Sourozenci ─────────────────────╮",
            "│                                                          │",
            "│  Cíl                                                     │",
            "│                                                          │",
            "│  Přejděte do adresáře finish pomocí jednoho příkazu cd.  │",
            "│                                                          │",
            "│    level-2/                                              │",
            "│    ├── start/    ← Jste tady                             │",
            "│    └── finish/   ← Cíl                                   │",
            "│                                                          │",
            "│   1 Ověřte polohu příkazem pwd                           │",
            "│   2 Odešlete řešení                                      │",
            "│                                                          │",
            "╰──────────────────────────────────────────────────────────╯",
        ]
    );
}

#[test]
fn panel_lines_have_the_same_width_with_diacritics_and_emoji() {
    let md = "### 🏆 Finální výzva: Terminálový ninja\nPříliš žluťoučký kůň úpěl ďábelské ódy. 💡 ✅ ❌ 🚪 ⚠️ ✓\n\n- **Důležité:** `ls` a `cd`\n- „české uvozovky“\n\n```\n├── žluťoučký/   ← kůň\n```";
    for width in [40, 61, 80, 120] {
        for term in [Term::plain(width), Term::ansi(width)] {
            let panel = Panel::new(style::CYAN)
                .title("💡 Nápověda č. 1 ze 3")
                .title_style(style::BOLD_CYAN);
            for line in panel.render_markdown(md, &term) {
                assert_eq!(visible_width(&line), width, "{line:?} at {width}");
            }
        }
    }
}

#[test]
fn ansi_coloured_body_lines_are_padded_by_their_visible_width() {
    let red = style::RED;
    let body = vec![
        format!("{}Chyba:{} soubor chybí", red.render(), red.render_reset()),
        format!(
            "{}žluťoučký{}",
            style::BOLD_YELLOW.render(),
            style::BOLD_YELLOW.render_reset()
        ),
        "bez barvy".to_owned(),
    ];
    let lines = Panel::new(style::RED).render(&body, &Term::ansi(50));
    for line in &lines {
        assert_eq!(visible_width(line), 50, "{line:?}");
        assert!(is_self_contained(line), "{line:?}");
    }
    // The colour codes of the body survive; the text next to them is padded, not cut.
    assert!(lines[2].contains(&format!("{}Chyba:", red.render())));
    assert!(strip_ansi(&lines[2]).starts_with("│  Chyba: soubor chybí"));
}

#[test]
fn emoji_with_a_variation_selector_are_normalised_so_boxes_stay_aligned() {
    // Terminals disagree whether `⚠️` takes one or two columns; the bare symbol is always one.
    assert_eq!(
        plain("### ⚠\u{fe0f} Důležité varování", 40),
        ["⚠ Důležité varování"]
    );
    // Emoji that are wide anyway keep their look.
    assert_eq!(
        plain("### 💡 Tip: klávesa Tab", 40),
        ["💡 Tip: klávesa Tab"]
    );
    // Raw lines handed to a panel are normalised too.
    let lines =
        Panel::new(style::CYAN).render(&["⚠\u{fe0f} ruční řádek".to_owned()], &Term::plain(30));
    assert!(
        lines.iter().all(|line| visible_width(line) == 30),
        "{lines:?}"
    );
    assert!(
        lines.iter().all(|line| !line.contains('\u{fe0f}')),
        "{lines:?}"
    );
}

#[test]
fn panel_titles_are_centered_like_rich() {
    let lines = Panel::new(style::CYAN)
        .title("Nápověda")
        .render(&["x".to_owned()], &Term::plain(30));
    assert_eq!(lines[0], "╭───────── Nápověda ─────────╮");
}

#[test]
fn a_success_panel_centers_its_lines() {
    let body = vec!["Výborně!".to_owned(), String::new(), "Čas: 3s".to_owned()];
    let lines = Panel::new(style::GREEN)
        .padding(1, 4)
        .centered()
        .render(&body, &Term::plain(30));
    assert_eq!(
        lines,
        [
            "╭────────────────────────────╮",
            "│                            │",
            "│          Výborně!          │",
            "│                            │",
            "│          Čas: 3s           │",
            "│                            │",
            "╰────────────────────────────╯",
        ]
    );
}

#[test]
fn panels_fold_code_lines_that_do_not_fit_without_losing_characters() {
    let md = fenced(TREE);
    let tree_chars: String = TREE.chars().filter(|c| !c.is_whitespace()).collect();
    for width in [40, 60, 80] {
        let lines = Panel::new(style::CYAN).render_markdown(&md, &Term::plain(width));
        assert!(
            lines.iter().all(|line| visible_width(line) == width),
            "width {width}"
        );
        let kept: String = lines
            .iter()
            .filter(|line| line.starts_with('│'))
            .flat_map(|line| line.chars().skip(1).take(visible_width(line) - 2))
            .filter(|c| !c.is_whitespace())
            .collect();
        assert_eq!(kept, tree_chars, "width {width}");
    }
}

// ---------------------------------------------------------------------------------------------
// Plain text
// ---------------------------------------------------------------------------------------------

#[test]
fn wrap_plain_never_interprets_markup() {
    let hint = "Použijte vzor `ls [ab]*.txt` nebo **ls** _data_?.csv [a-z] a \\* beze změny";
    let lines = wrap_plain(hint, 1000);
    assert_eq!(lines, [hint]);

    let narrow = wrap_plain(hint, 24);
    assert!(narrow.len() > 1);
    assert_eq!(narrow.join(" "), hint);
    for line in &narrow {
        assert!(visible_width(line) <= 24, "{line:?}");
    }
}

#[test]
fn wrap_plain_keeps_glob_patterns_and_backticks_intact_on_one_line() {
    let lines = wrap_plain("Spusťte `ls [ab]*.txt` a porovnejte *.log", 20);
    assert!(lines.iter().any(|line| line.contains("[ab]*.txt`")));
    assert!(lines.iter().any(|line| line.contains("*.log")));
    assert!(lines.iter().all(|line| !line.contains('\x1b')));
}

#[test]
fn wrap_plain_keeps_explicit_line_breaks() {
    assert_eq!(
        wrap_plain("1. cd start\n2. ls\n\nHotovo.", 40),
        ["1. cd start", "2. ls", "", "Hotovo."]
    );
}

#[test]
fn a_hint_panel_prints_the_hint_verbatim() {
    let hint = "Zkuste `ls [ab]*.txt` — hvězdička * a [závorky] zůstanou.";
    let panel = Panel::new(style::YELLOW).title("💡 Nápověda 1 z 3");
    let lines = panel.render_text(hint, style::PLAIN, &Term::plain(80));
    assert!(lines.iter().all(|line| visible_width(line) == 80));
    assert!(
        lines
            .iter()
            .any(|line| line.contains("`ls [ab]*.txt` — hvězdička * a [závorky] zůstanou."))
    );
}

// ---------------------------------------------------------------------------------------------
// Paging
// ---------------------------------------------------------------------------------------------

#[test]
fn split_pages_cuts_the_section_1_intro_into_two_pages() {
    let intro = include_str!("../src/levels/s01_navigace/00-uvod.md");
    let pages = split_pages(intro);
    assert_eq!(pages.len(), 2, "{pages:#?}");
    assert!(pages[0].starts_with("Vítejte v první sekci ShellGame!"));
    assert!(pages[1].starts_with("### Klíčové koncepty"));
    assert!(pages.iter().all(|page| !page.contains("\n---\n")));
    assert!(pages.iter().all(|page| page == page.trim()));
}

#[test]
fn split_pages_matches_the_python_split() {
    // Python: [p.strip() for p in raw.split("\n---\n") if p.strip()]
    assert_eq!(split_pages("a\n---\nb\n---\nc"), ["a", "b", "c"]);
    assert_eq!(split_pages("a\n\n---\n\nb\n"), ["a", "b"]);
    assert_eq!(split_pages("bez oddělovače"), ["bez oddělovače"]);
}

// ---------------------------------------------------------------------------------------------
// Invariants over all level texts
// ---------------------------------------------------------------------------------------------

#[test]
fn coloured_lines_are_self_contained() {
    let mut samples = vec![LEVEL_2_1.to_owned(), fenced(TREE)];
    samples.push(
        "**tučný text, který se zalomí na několik řádků a nesmí nechat styl otevřený**".to_owned(),
    );
    samples.push(
        "> citace s `kódem` a **tučným** textem, která se musí zalomit na víc řádků".to_owned(),
    );
    samples.push("- položka s `inline kódem` a *šikmým* textem, který se zalomí\n  - vnořená s **tučným** textem".to_owned());
    for file in markdown_files() {
        samples.push(fs::read_to_string(&file).expect("readable markdown"));
    }
    for sample in &samples {
        for width in [12, 40, 80] {
            for line in ansi(sample, width) {
                assert!(is_self_contained(&line), "{line:?} at {width}");
            }
            let panel = Panel::new(style::CYAN)
                .title("Titulek")
                .title_style(style::BOLD_CYAN);
            for line in panel.render_markdown(sample, &Term::ansi(width.max(20))) {
                assert!(is_self_contained(&line), "{line:?} at {width}");
            }
        }
    }
}

#[test]
fn every_markdown_file_renders_at_common_widths_without_overflow() {
    let files = markdown_files();
    assert!(!files.is_empty(), "no markdown content found");
    let code_open = style::CODE_BLOCK.render().to_string();

    for file in &files {
        let text = fs::read_to_string(file).expect("readable markdown");
        for page in split_pages(&text) {
            for width in [40, 80, 120] {
                let plain_lines = plain(&page, width);
                let ansi_lines = ansi(&page, width);
                assert_eq!(plain_lines.len(), ansi_lines.len(), "{}", file.display());
                for (plain_line, ansi_line) in plain_lines.iter().zip(&ansi_lines) {
                    assert!(
                        !plain_line.contains('\x1b'),
                        "{}: {plain_line:?}",
                        file.display()
                    );
                    assert_eq!(
                        strip_ansi(ansi_line),
                        plain_line.as_str(),
                        "{}",
                        file.display()
                    );
                    // Only code blocks may be wider than the terminal.
                    if visible_width(plain_line) > width {
                        assert!(
                            ansi_line.contains(&code_open),
                            "{}: overflow at {width}: {plain_line:?}",
                            file.display()
                        );
                    }
                }

                let panel = Panel::new(style::CYAN).title("1.1: Titulek 💡");
                let boxed = panel.render_markdown(&page, &Term::plain(width));
                assert!(
                    boxed.iter().all(|line| visible_width(line) == width),
                    "{}: panel width at {width}",
                    file.display()
                );
                // Colour changes nothing but the escape sequences.
                let coloured = panel.render_markdown(&page, &Term::ansi(width));
                let stripped: Vec<String> = coloured
                    .iter()
                    .map(|line| strip_ansi(line).into_owned())
                    .collect();
                assert_eq!(stripped, boxed, "{}: panel at {width}", file.display());
            }
        }
    }
}
