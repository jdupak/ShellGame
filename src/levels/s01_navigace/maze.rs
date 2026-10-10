//! The maze of level 1.7: a tree of directories whose `GO_*` files say, by their *names*, where
//! to go next.
//!
//! Every room holds exactly one kind of exit: an instruction file (`GO_TO_DIR_x`, `GO_UP_N`,
//! `GO_UP_N_THEN_GO_TO_x`), the marker of a dead end, or the victory marker of the real goal.
//! The main route is long on purpose: it practises `ls`, descending, backtracking and moving to
//! siblings, and every branch off it is a decoy that ends in a dead end or leads back.

use crate::engine::prelude::*;

/// The real goal, relative to the section root.
pub(super) const SANCTUARY: &str = "maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth/\
     corridor/shrine/final/final_v2/final_final/opravdu_final_v2_FINAL";

/// The marker of a dead end.
const TRAP: &str = "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE";

/// The marker of the real goal.
const VICTORY: &str = "VICTORY.marker";

/// Every room of the maze (relative to `maze/`) and what it contains: instruction files, files
/// with no meaning, and the subdirectories that are not files.
const STRUCTURE: &[(&str, &[&str])] = &[
    // Entrance area
    ("entry", &["GO_TO_DIR_hall", "dungeon", "courtyard"]),
    ("entry/courtyard", &[TRAP]),
    ("entry/dungeon", &[TRAP]),
    ("entry/hall", &["GO_TO_DIR_nexus", "alcove", "side_door"]),
    ("entry/hall/alcove", &[TRAP]),
    ("entry/hall/side_door", &["GO_UP_1"]),
    // Nexus hub
    (
        "entry/hall/nexus",
        &["GO_TO_DIR_passages", "sanctum", "rotunda", "archives"],
    ),
    ("entry/hall/nexus/sanctum", &[TRAP]),
    ("entry/hall/nexus/archives", &[TRAP]),
    ("entry/hall/nexus/rotunda", &["GO_UP_1"]),
    // Passages branch
    (
        "entry/hall/nexus/passages",
        &["GO_TO_DIR_tunnel", "gallery", "shaft"],
    ),
    ("entry/hall/nexus/passages/gallery", &[TRAP]),
    ("entry/hall/nexus/passages/shaft", &["GO_UP_1"]),
    (
        "entry/hall/nexus/passages/tunnel",
        &["GO_TO_DIR_cavern", "crevice"],
    ),
    ("entry/hall/nexus/passages/tunnel/crevice", &[TRAP]),
    (
        "entry/hall/nexus/passages/tunnel/cavern",
        &["GO_TO_DIR_depths", "grotto", "abyss"],
    ),
    ("entry/hall/nexus/passages/tunnel/cavern/grotto", &[TRAP]),
    ("entry/hall/nexus/passages/tunnel/cavern/abyss", &[TRAP]),
    (
        "entry/hall/nexus/passages/tunnel/cavern/depths",
        &["GO_UP_2_THEN_GO_TO_catacombs", "echoes.txt"],
    ),
    // Catacombs branch
    (
        "entry/hall/nexus/passages/tunnel/catacombs",
        &["GO_TO_DIR_vault", "ossuary", "crypts"],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/ossuary",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/crypts",
        &["GO_UP_1"],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/vault",
        &["GO_TO_DIR_chamber", "iron_cell", "sepulcher"],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/vault/iron_cell",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/vault/sepulcher",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/vault/chamber",
        &["GO_UP_2_THEN_GO_TO_labyrinth", "relic.txt"],
    ),
    // Labyrinth and final sequence
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth",
        &["GO_TO_DIR_corridor", "dead_end", "ruins"],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/dead_end",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/ruins",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor",
        &["GO_TO_DIR_shrine", "false_exit", "mist"],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/false_exit",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/mist",
        &["GO_UP_1"],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine",
        &["GO_TO_DIR_final", "mirage", "shadow"],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/mirage",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/shadow",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final",
        &["GO_TO_DIR_final_v2", "decoy_exit", "draft.txt"],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/decoy_exit",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2",
        &["GO_TO_DIR_final_final", "abandoned_branch", "old.bak"],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2/\
         abandoned_branch",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2/\
         final_final",
        &["GO_TO_DIR_opravdu_final_v2_FINAL", "fake_end"],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2/\
         final_final/fake_end",
        &[TRAP],
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2/\
         final_final/opravdu_final_v2_FINAL",
        &[VICTORY],
    ),
];

/// A name in `STRUCTURE` that is a file; anything else is a subdirectory.
fn is_file(item: &str) -> bool {
    item.starts_with("GO_")
        || item == TRAP
        || [".txt", ".md", ".marker", ".bak"]
            .iter()
            .any(|extension| item.ends_with(extension))
}

/// A short note so `cat` is not a blank page. For an instruction file, its name stays the
/// instruction.
fn note(name: &str) -> &'static str {
    if name.starts_with("GO_") {
        "Instrukce je v názvu tohoto souboru. Řiďte se jménem, ne obsahem.\n"
    } else if name == TRAP {
        "Tady nemáte být. Vraťte se a sledujte soubory začínající na GO_.\n\
         Ztratili jste se? Příkaz 'shellgame reset' vás vrátí na začátek bludiště (maze/entry).\n"
    } else if name == VICTORY {
        "Konečně skutečný cíl (opravdu_final_v2_FINAL)! \
         Level dokončíte příkazem: shellgame submit\n"
    } else {
        ""
    }
}

/// The directories and files of the maze, to be applied below the section root. Applying it
/// first removes the old `maze/`, so `shellgame reset` always hands back the untouched maze.
pub(super) fn fixture() -> Fixture {
    let mut dirs: Vec<String> = Vec::new();
    let mut files: Vec<(String, &'static str)> = Vec::new();
    let mut add_dir = |path: String| {
        if !dirs.contains(&path) {
            dirs.push(path);
        }
    };
    let mut add_file = |path: String, name: &str| {
        if !files.iter().any(|(existing, _)| *existing == path) {
            files.push((path, note(name)));
        }
    };

    for (room, contents) in STRUCTURE {
        add_dir(format!("maze/{room}"));
        for item in *contents {
            let path = format!("maze/{room}/{item}");
            if is_file(item) {
                add_file(path, item);
            } else {
                add_dir(path);
            }
        }
        let has_instruction = contents.iter().any(|item| item.starts_with("GO_"));
        if !has_instruction && !contents.contains(&VICTORY) {
            add_file(format!("maze/{room}/{TRAP}"), TRAP);
        }
    }

    let fixture = Fixture::new().clean(["maze"]).dirs(dirs);
    files
        .into_iter()
        .fold(fixture, |fixture, (path, text)| fixture.file(path, text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// What an instruction file's name asks the player to do.
    #[derive(Debug, PartialEq, Eq)]
    enum Instruction<'a> {
        /// `GO_TO_DIR_x`: `cd x`.
        Enter(&'a str),
        /// `GO_UP_N`: `cd` up `N` levels.
        Up(usize),
        /// `GO_UP_N_THEN_GO_TO_x`: up `N` levels, then into `x`.
        UpThenEnter(usize, &'a str),
    }

    impl<'a> Instruction<'a> {
        fn parse(name: &'a str) -> Option<Self> {
            if let Some(target) = name.strip_prefix("GO_TO_DIR_") {
                return (!target.is_empty()).then_some(Self::Enter(target));
            }
            let rest = name.strip_prefix("GO_UP_")?;
            let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
            let levels = rest[..digits].parse().ok()?;
            match &rest[digits..] {
                "" => Some(Self::Up(levels)),
                tail => tail
                    .strip_prefix("_THEN_GO_TO_")
                    .filter(|target| !target.is_empty())
                    .map(|target| Self::UpThenEnter(levels, target)),
            }
        }

        /// Where the instruction leads from `from`.
        fn destination(&self, from: &Path) -> PathBuf {
            let up = |levels: usize| {
                from.ancestors()
                    .nth(levels)
                    .map_or_else(|| PathBuf::from("/"), Path::to_path_buf)
            };
            match self {
                Self::Enter(target) => from.join(target),
                Self::Up(levels) => up(*levels),
                Self::UpThenEnter(levels, target) => up(*levels).join(target),
            }
        }
    }

    /// The maze, built in a temporary directory.
    fn built() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        fixture().apply(dir.path()).unwrap();
        let maze = dir.path().join("maze");
        (dir, maze)
    }

    fn rooms(maze: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut pending = vec![maze.to_path_buf()];
        while let Some(dir) = pending.pop() {
            for entry in fs::read_dir(&dir).unwrap().flatten() {
                if entry.file_type().unwrap().is_dir() {
                    pending.push(entry.path());
                }
            }
            found.push(dir);
        }
        found
    }

    fn names(room: &Path) -> Vec<String> {
        fs::read_dir(room)
            .unwrap()
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn instruction_names_are_parsed_like_the_instructions_describe() {
        assert_eq!(
            Instruction::parse("GO_TO_DIR_hall"),
            Some(Instruction::Enter("hall"))
        );
        assert_eq!(Instruction::parse("GO_UP_1"), Some(Instruction::Up(1)));
        assert_eq!(Instruction::parse("GO_UP_12"), Some(Instruction::Up(12)));
        assert_eq!(
            Instruction::parse("GO_UP_2_THEN_GO_TO_catacombs"),
            Some(Instruction::UpThenEnter(2, "catacombs"))
        );
        for not_an_instruction in [
            "GO_",
            "GO_UP_",
            "GO_UP_x",
            "GO_UP_2_THEN_GO_TO_",
            "GO_UP_2_THEN",
            "GO_TO_DIR_",
            TRAP,
            VICTORY,
            "echoes.txt",
        ] {
            assert_eq!(Instruction::parse(not_an_instruction), None);
        }
    }

    #[test]
    fn every_instruction_leads_to_an_existing_directory() {
        let (_dir, maze) = built();
        let mut checked = 0;
        for room in rooms(&maze) {
            for name in names(&room) {
                if !name.starts_with("GO_") {
                    continue;
                }
                let instruction = Instruction::parse(&name)
                    .unwrap_or_else(|| panic!("{name} in {room:?} is not a valid instruction"));
                let destination = instruction.destination(&room);
                assert!(
                    destination.is_dir(),
                    "{name} in {room:?} leads to {destination:?}, which does not exist"
                );
                assert!(
                    destination.starts_with(&maze),
                    "{name} in {room:?} leads out of the maze"
                );
                checked += 1;
            }
        }
        assert!(checked > 10, "the maze has instructions to follow");
    }

    #[test]
    fn every_room_has_an_instruction_or_is_a_marked_dead_end_or_the_goal() {
        let (_dir, maze) = built();
        for room in rooms(&maze) {
            let names = names(&room);
            let go = names.iter().any(|name| name.starts_with("GO_"));
            let trap = names.iter().any(|name| name == TRAP);
            let victory = names.iter().any(|name| name == VICTORY);
            if victory {
                assert!(
                    !go && !trap,
                    "the goal {room:?} must not look like anything else"
                );
            } else if room == maze {
                assert!(!go && !trap, "the maze directory itself holds only `entry`");
            } else {
                assert!(
                    go != trap,
                    "{room:?} needs exactly one of an instruction and a dead-end marker: {names:?}"
                );
            }
        }
    }

    #[test]
    fn the_main_route_leads_from_the_entry_to_the_goal_without_a_loop() {
        let (_dir, maze) = built();
        let goal = maze.parent().unwrap().join(SANCTUARY);
        let mut room = maze.join("entry");
        let mut visited = vec![room.clone()];
        while room != goal {
            let instructions: Vec<String> = names(&room)
                .into_iter()
                .filter(|name| name.starts_with("GO_"))
                .collect();
            assert_eq!(instructions.len(), 1, "one instruction in {room:?}");
            let next = Instruction::parse(&instructions[0])
                .unwrap()
                .destination(&room);
            assert!(!visited.contains(&next), "the route loops at {next:?}");
            visited.push(next.clone());
            room = next;
        }
        assert!(room.join(VICTORY).is_file());
    }

    #[test]
    fn the_notes_are_never_blank_where_a_player_would_cat_them() {
        let (_dir, maze) = built();
        for room in rooms(&maze) {
            for name in names(&room) {
                if name.starts_with("GO_") || name == TRAP || name == VICTORY {
                    let text = fs::read_to_string(room.join(&name)).unwrap();
                    assert!(!text.trim().is_empty(), "{name} in {room:?} is blank");
                }
            }
        }
    }

    #[test]
    fn the_dead_end_note_tells_how_to_recover() {
        let (_dir, maze) = built();
        let text = fs::read_to_string(maze.join("entry/dungeon").join(TRAP)).unwrap();
        assert!(text.contains("shellgame reset") && text.contains("maze/entry"));
    }

    #[test]
    fn the_goal_note_tells_the_player_to_submit() {
        let (_dir, maze) = built();
        let goal = maze.parent().unwrap().join(SANCTUARY);
        let text = fs::read_to_string(goal.join(VICTORY)).unwrap();
        assert!(text.contains("shellgame submit"));
        assert!(
            !text.contains("automaticky"),
            "nothing completes the level by itself"
        );
    }

    #[test]
    fn applying_the_fixture_again_restores_a_damaged_maze() {
        let (dir, maze) = built();
        fs::remove_dir_all(maze.join("entry/hall/nexus")).unwrap();
        fs::write(maze.join("entry/stray.txt"), "left behind").unwrap();
        fixture().apply(dir.path()).unwrap();
        assert!(maze.join("entry/hall/nexus/passages").is_dir());
        assert!(!maze.join("entry/stray.txt").exists());
    }
}
