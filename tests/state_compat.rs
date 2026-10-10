//! Compatibility of the Rust state file with the Python implementation.
//!
//! Students switch between the two binaries in the middle of the semester, so both must read
//! and write the very same `state.json`. The golden files in `tests/state/` come in two kinds:
//!
//! * `python_*.json` and `migration_table.json` are written by the real Python code. Regenerate
//!   them with `tests/state/generate_golden.py`.
//! * `v1_0.json`, `v2_0.json` and the broken files (`corrupt.json`, `version_99.json`,
//!   `not_object.json`, `missing_workspace.json`) are written by hand.
//!
//! Every test works in its own temporary state directory; nothing touches the process
//! environment or the working directory.

use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{Local, NaiveDate, NaiveDateTime, TimeDelta, TimeZone as _};
use indexmap::IndexMap;
use serde_json::{Value, json};
use shellgame::state::{LevelCompletion, State, StateError, StateStore};
use tempfile::TempDir;

// ---------------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------------

fn golden_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/state")
        .join(name)
}

fn golden_bytes(name: &str) -> Vec<u8> {
    fs::read(golden_path(name)).unwrap_or_else(|error| panic!("cannot read golden {name}: {error}"))
}

fn golden_text(name: &str) -> String {
    String::from_utf8(golden_bytes(name)).unwrap_or_else(|_| panic!("{name} is not UTF-8"))
}

fn at_micros(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    micros: u32,
) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|date| date.and_hms_micro_opt(hour, minute, second, micros))
        .expect("valid test timestamp")
}

fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> NaiveDateTime {
    at_micros(year, month, day, hour, minute, second, 0)
}

/// Keys of a per-level map, in order.
fn keys<V>(map: &IndexMap<String, V>) -> Vec<&str> {
    map.keys().map(String::as_str).collect()
}

/// Entries of a per-level map, in order. (`IndexMap`'s own `==` ignores the order.)
fn entries<V: Copy>(map: &IndexMap<String, V>) -> Vec<(&str, V)> {
    map.iter()
        .map(|(key, &value)| (key.as_str(), value))
        .collect()
}

/// A temporary state directory with a store on it.
struct Sandbox {
    dir: TempDir,
    store: StateStore,
}

impl Sandbox {
    fn empty() -> Self {
        let dir = tempfile::tempdir().expect("create temporary directory");
        let store = StateStore::new(dir.path());
        Self { dir, store }
    }

    fn with_bytes(contents: impl AsRef<[u8]>) -> Self {
        let sandbox = Self::empty();
        sandbox.write(contents);
        sandbox
    }

    fn with_golden(name: &str) -> Self {
        Self::with_bytes(golden_bytes(name))
    }

    fn write(&self, contents: impl AsRef<[u8]>) {
        fs::write(self.store.path(), contents).expect("write the state file");
    }

    fn bytes(&self) -> Vec<u8> {
        fs::read(self.store.path()).expect("read the state file")
    }

    fn text(&self) -> String {
        String::from_utf8(self.bytes()).expect("the state file is UTF-8")
    }

    /// Names of everything in the state directory, sorted.
    fn file_names(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.dir.path())
            .expect("list the state directory")
            .map(|entry| {
                entry
                    .expect("directory entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }

    fn load(&self) -> State {
        self.store
            .load()
            .expect("the state loads")
            .expect("the state file exists")
    }

    fn load_error(&self) -> StateError {
        self.store.load().expect_err("the state must not load")
    }
}

/// A minimal valid v2.1 file whose `start_time` is the given JSON value.
fn state_file_with_start_time(start_time: &Value) -> String {
    json!({
        "version": "2.1",
        "username": "tester",
        "workspace": "/tmp/shellgame-tester",
        "current_level": "0.0",
        "start_time": start_time,
    })
    .to_string()
}

// ---------------------------------------------------------------------------------------------
// Files written by Python
// ---------------------------------------------------------------------------------------------

/// Generates one test per golden file: load it, save it again, and demand the same bytes.
macro_rules! round_trip_tests {
    ($($test:ident => $file:literal),* $(,)?) => {$(
        #[test]
        fn $test() {
            let sandbox = Sandbox::with_golden($file);
            let state = sandbox.load();

            sandbox.store.save(&state).expect("the state saves");

            assert_eq!(sandbox.text(), golden_text($file), "re-saving {} changed it", $file);
            assert_eq!(sandbox.file_names(), ["state.json"], "no temporary file may remain");
        }
    )*};
}

round_trip_tests! {
    mid_game_file_round_trips_byte_for_byte => "python_v2_1.json",
    finished_game_file_round_trips_byte_for_byte => "python_v2_1_finished.json",
    fresh_game_file_round_trips_byte_for_byte => "python_v2_1_fresh.json",
    file_with_escapes_round_trips_byte_for_byte => "python_v2_1_escapes.json",
    migrated_v1_0_file_round_trips_byte_for_byte => "python_v1_0_migrated.json",
    migrated_v2_0_file_round_trips_byte_for_byte => "python_v2_0_migrated.json",
}

#[test]
fn mid_game_file_loads_with_every_field() {
    let state = Sandbox::with_golden("python_v2_1.json").load();

    assert_eq!(state.version, "2.1");
    assert_eq!(state.username, "Jůlie Novák");
    assert_eq!(state.workspace, Path::new("/tmp/shellgame-Jůlie Novák"));
    assert_eq!(state.current_level, "10.5");
    assert_eq!(state.start_time, at(2026, 2, 16, 9, 15, 42));
    assert_eq!(state.completed_at, None);
    assert!(!state.is_finished());

    // Insertion order is play order, not numeric order: section 10 was reached by `dev jump`.
    assert_eq!(
        keys(&state.levels_complete),
        [
            "0.0", "1.0", "1.1", "1.2", "1.3", "1.4", "1.10", "1.12", "2.0", "2.1", "2.2", "10.3",
            "10.4"
        ]
    );
    assert_eq!(
        state.levels_complete["1.12"],
        LevelCompletion {
            time_sec: 95,
            hints: 3,
            attempts: 0,
            completed_at: at_micros(2026, 2, 16, 9, 23, 6, 123_456),
        }
    );
    assert_eq!(
        state.levels_complete["10.4"],
        LevelCompletion {
            time_sec: 213,
            hints: 3,
            attempts: 0,
            completed_at: at_micros(2026, 2, 16, 9, 40, 17, 500_000),
        }
    );
    assert_eq!(
        state.levels_complete["1.1"].completed_at,
        at(2026, 2, 16, 9, 16, 32)
    );

    assert_eq!(
        entries(&state.level_attempts),
        [
            ("1.2", 2),
            ("1.4", 3),
            ("1.10", 1),
            ("2.2", 1),
            ("10.3", 3),
            ("10.5", 1)
        ]
    );
    assert_eq!(
        entries(&state.level_hints_used),
        [("1.2", 1), ("1.4", 2), ("1.12", 3), ("2.2", 1), ("10.4", 3)]
    );
    assert_eq!(
        keys(&state.level_started_at),
        [
            "0.0", "1.0", "1.1", "1.2", "1.3", "1.4", "1.10", "1.12", "2.0", "2.1", "2.2", "10.3",
            "10.4", "10.5"
        ]
    );
    assert_eq!(
        state.level_started_at["0.0"],
        at_micros(2026, 2, 16, 9, 15, 42, 417)
    );
    assert_eq!(state.level_started_at["10.3"], at(2026, 2, 16, 9, 35, 53));
}

#[test]
fn finished_game_file_is_finished() {
    let state = Sandbox::with_golden("python_v2_1_finished.json").load();

    assert!(state.is_finished());
    assert_eq!(
        state.completed_at,
        Some(at_micros(2026, 3, 2, 8, 5, 22, 654_321))
    );
    assert_eq!(state.username, "Eliška");
    assert_eq!(state.current_level, "11.6");
    assert_eq!(keys(&state.levels_complete), ["0.0", "1.0", "11.6"]);
}

#[test]
fn fresh_game_file_has_only_the_first_level_started() {
    let state = Sandbox::with_golden("python_v2_1_fresh.json").load();

    assert_eq!(state.current_level, "0.0");
    assert_eq!(state.start_time, at_micros(2026, 9, 1, 12, 0, 0, 250_000));
    assert!(state.levels_complete.is_empty());
    assert!(state.level_attempts.is_empty());
    assert!(state.level_hints_used.is_empty());
    assert_eq!(
        entries(&state.level_started_at),
        [("0.0", at_micros(2026, 9, 1, 12, 0, 0, 250_009))]
    );
    assert_eq!(state.completed_at, None);
}

#[test]
fn escapes_and_non_ascii_text_survive() {
    let sandbox = Sandbox::with_golden("python_v2_1_escapes.json");
    let state = sandbox.load();

    assert_eq!(
        state.username,
        "Quote \" back\\slash tab\t newline\n bell\u{7} unit-sep\u{1f} del\u{7f} \
         line-sep\u{2028} ünï čödé 😀 日本"
    );
    assert_eq!(state.workspace, Path::new("/tmp/shell game/sp ace ✓"));

    // Pydantic escapes control characters but writes everything else as it is.
    let text = golden_text("python_v2_1_escapes.json");
    assert!(text.contains(r"bell\u0007 unit-sep\u001f"));
    assert!(text.contains("ünï čödé 😀 日本"));
    assert!(text.contains('\u{7f}') && text.contains('\u{2028}'));
}

#[test]
fn json_layout_matches_pydantic() {
    let mut state = State::new(
        "jana",
        "/tmp//shellgame-jana/",
        at_micros(2026, 3, 1, 10, 20, 30, 123_456),
    );
    state.record_attempt("0.0");
    state.record_completion("0.0", at(2026, 3, 1, 10, 20, 35));
    state.current_level = "1.0".to_owned();
    state.ensure_level_started("1.0", at(2026, 3, 1, 10, 20, 35));

    let sandbox = Sandbox::empty();
    sandbox.store.save(&state).expect("the state saves");

    // What `GameState.model_dump_json(indent=2)` prints for this state: two-space indent,
    // `"key": value`, the workspace normalized like a Python `Path`, no trailing newline.
    let expected = r#"{
  "version": "2.1",
  "username": "jana",
  "workspace": "/tmp/shellgame-jana",
  "current_level": "1.0",
  "start_time": "2026-03-01T10:20:30.123456",
  "levels_complete": {
    "0.0": {
      "time_sec": 4,
      "hints": 0,
      "attempts": 1,
      "completed_at": "2026-03-01T10:20:35"
    }
  },
  "level_attempts": {
    "0.0": 1
  },
  "level_hints_used": {},
  "level_started_at": {
    "0.0": "2026-03-01T10:20:30.123456",
    "1.0": "2026-03-01T10:20:35"
  },
  "completed_at": null
}"#;
    assert_eq!(sandbox.text(), expected);
}

// ---------------------------------------------------------------------------------------------
// Migrations
// ---------------------------------------------------------------------------------------------

#[test]
fn version_1_0_file_is_migrated_like_python() {
    let sandbox = Sandbox::with_golden("v1_0.json");
    let state = sandbox.load();

    // The file has no `version` key at all.
    assert_eq!(state.version, "2.1");
    // 7.3 was a permissions level; permissions moved from section 7 to section 8.
    assert_eq!(state.current_level, "8.3");
    // 2.7 and 2.8 swap, so do 6.7 and 6.8; section 7 becomes 8. Order is untouched.
    assert_eq!(
        keys(&state.levels_complete),
        ["1.1", "2.8", "2.7", "6.8", "6.7", "8.0", "8.1", "8.2"]
    );
    assert_eq!(
        entries(&state.level_attempts),
        [("2.8", 1), ("6.8", 4), ("8.2", 3)]
    );
    assert_eq!(
        entries(&state.level_hints_used),
        [("2.8", 1), ("6.8", 2), ("8.1", 2)]
    );
    assert_eq!(
        keys(&state.level_started_at),
        ["1.1", "2.8", "6.7", "8.2", "8.3"]
    );
    // Values travel with their keys: 2.7 took 190 s, 7.1 took 77 s.
    assert_eq!(state.levels_complete["2.8"].time_sec, 190);
    assert_eq!(state.levels_complete["8.1"].time_sec, 77);
    // Fields the old file lacks get their defaults.
    assert_eq!(state.completed_at, None);

    // The migrated version is written on the next save, byte for byte like Python writes it.
    sandbox.store.save(&state).expect("the state saves");
    assert_eq!(sandbox.text(), golden_text("python_v1_0_migrated.json"));
}

#[test]
fn version_2_0_file_is_migrated_like_python() {
    let sandbox = Sandbox::with_golden("v2_0.json");
    let state = sandbox.load();

    assert_eq!(state.version, "2.1");
    // 10.3 was a wildcards level; wildcards moved from section 10 to section 7.
    assert_eq!(state.current_level, "7.3");
    assert_eq!(
        keys(&state.levels_complete),
        [
            "8.0", "8.1", "9.0", "9.5", "10.0", "10.4", "7.0", "7.1", "7.2"
        ]
    );
    assert_eq!(
        entries(&state.level_attempts),
        [("9.5", 3), ("10.4", 1), ("7.2", 2)]
    );
    assert_eq!(
        entries(&state.level_hints_used),
        [("9.5", 3), ("10.4", 2), ("7.2", 1)]
    );
    assert_eq!(
        keys(&state.level_started_at),
        ["8.0", "9.5", "10.4", "7.2", "7.3"]
    );
    assert_eq!(state.levels_complete["10.4"].time_sec, 175);
    assert_eq!(state.levels_complete["7.2"].time_sec, 98);
    assert_eq!(state.username, "Eliška Dvořáková");

    sandbox.store.save(&state).expect("the state saves");
    assert_eq!(sandbox.text(), golden_text("python_v2_0_migrated.json"));
}

#[test]
fn a_migrated_file_is_not_migrated_again() {
    let sandbox = Sandbox::with_golden("v1_0.json");
    let migrated = sandbox.load();
    sandbox.store.save(&migrated).expect("the state saves");

    // Version 2.1 on disk now: loading it must not rotate the sections a second time.
    assert_eq!(sandbox.load(), migrated);
    assert!(sandbox.text().contains(r#""version": "2.1""#));
}

fn migration_table() -> Value {
    serde_json::from_str(&golden_text("migration_table.json")).expect("migration table is JSON")
}

#[test]
fn level_id_migration_matches_python_for_every_id() {
    let table = migration_table();
    let pairs = table["pairs"].as_array().expect("pairs");
    assert!(
        pairs.len() > 250,
        "the table should cover every section and many odd IDs"
    );

    let sandbox = Sandbox::empty();
    for (index, pair) in pairs.iter().enumerate() {
        let (old, expected) = (
            pair[0].as_str().expect("old"),
            pair[1].as_str().expect("new"),
        );

        // Both old versions migrate, and so does a file without any version.
        let mut file = json!({
            "username": "tester",
            "workspace": "/tmp/shellgame-tester",
            "current_level": old,
            "start_time": "2026-03-01T10:20:30",
        });
        match index % 3 {
            0 => file["version"] = json!("1.0"),
            1 => file["version"] = json!("2.0"),
            _ => {}
        }
        sandbox.write(file.to_string());

        let state = sandbox.load();
        assert_eq!(state.current_level, expected, "migrating level ID {old:?}");
        assert_eq!(state.version, "2.1");
    }
}

#[test]
fn migrating_a_whole_map_matches_python_even_when_ids_collide() {
    let table = migration_table();
    let pairs = table["pairs"].as_array().expect("pairs");
    let expected: Vec<(String, u32)> = table["remapped_map"]
        .as_array()
        .expect("remapped_map")
        .iter()
        .map(|entry| {
            let value = u32::try_from(entry[1].as_u64().expect("position")).expect("small");
            (entry[0].as_str().expect("key").to_owned(), value)
        })
        .collect();
    assert!(
        expected.len() < pairs.len(),
        "the table must contain colliding IDs"
    );

    // Every old ID as a key of one map, with its position as the value.
    let attempts: serde_json::Map<String, Value> = pairs
        .iter()
        .enumerate()
        .map(|(position, pair)| (pair[0].as_str().expect("old").to_owned(), json!(position)))
        .collect();
    let file = json!({
        "version": "2.0",
        "username": "tester",
        "workspace": "/tmp/shellgame-tester",
        "current_level": "1.1",
        "start_time": "2026-03-01T10:20:30",
        "level_attempts": attempts,
    });

    let state = Sandbox::with_bytes(file.to_string()).load();

    let actual: Vec<(String, u32)> = state
        .level_attempts
        .iter()
        .map(|(key, &value)| (key.clone(), value))
        .collect();
    assert_eq!(actual, expected);
}

// ---------------------------------------------------------------------------------------------
// Files that must not load
// ---------------------------------------------------------------------------------------------

#[test]
fn unusable_files_are_load_errors_and_are_never_touched() {
    let cases = [
        ("corrupt.json", "line 6"),
        ("version_99.json", "Nepodporovaná verze stavu: 99.0"),
        ("not_object.json", "Kořen souboru stavu musí být objekt."),
        ("missing_workspace.json", "missing field `workspace`"),
    ];

    for (file, expected_detail) in cases {
        let sandbox = Sandbox::with_golden(file);
        let before = sandbox.bytes();

        let error = sandbox.load_error();

        let prefix = format!(
            "Stav ShellGame nelze načíst z {}: ",
            sandbox.store.path().display()
        );
        let message = error.to_string();
        assert!(
            matches!(error, StateError::Load { .. }),
            "{file}: {error:?}"
        );
        assert!(message.starts_with(&prefix), "{file}: {message}");
        assert!(message.contains(expected_detail), "{file}: {message}");
        assert_eq!(error.file(), sandbox.store.path());
        assert!(sandbox.store.exists(), "{file}: a broken file still exists");
        assert_eq!(
            sandbox.bytes(),
            before,
            "{file}: loading must not modify the file"
        );
        assert_eq!(
            sandbox.file_names(),
            ["state.json"],
            "{file}: loading must not add files"
        );
    }
}

#[test]
fn every_required_field_is_required() {
    let complete: Value =
        serde_json::from_str(&golden_text("python_v2_1_fresh.json")).expect("golden file is JSON");

    for field in ["username", "workspace", "current_level", "start_time"] {
        let mut broken = complete.clone();
        broken.as_object_mut().expect("object").remove(field);
        let sandbox = Sandbox::with_bytes(broken.to_string());

        let error = sandbox.load_error();

        assert!(
            matches!(error, StateError::Load { .. }),
            "{field}: {error:?}"
        );
        assert!(
            error.detail().contains(&format!("missing field `{field}`")),
            "{field}: {error}"
        );
    }
}

#[test]
fn only_the_per_level_fields_and_completed_at_may_be_missing() {
    let sandbox = Sandbox::with_bytes(
        json!({
            "version": "2.1",
            "username": "tester",
            "workspace": "/tmp/shellgame-tester",
            "current_level": "3.1",
            "start_time": "2026-03-01T10:20:30",
        })
        .to_string(),
    );

    let state = sandbox.load();

    assert_eq!(state.current_level, "3.1");
    assert!(state.levels_complete.is_empty());
    assert!(state.level_attempts.is_empty());
    assert!(state.level_hints_used.is_empty());
    assert!(state.level_started_at.is_empty());
    assert_eq!(state.completed_at, None);
}

#[test]
fn unknown_fields_are_ignored_and_dropped_on_save() {
    let mut file: Value = serde_json::from_str(&golden_text("python_v2_1.json")).expect("JSON");
    file["theme"] = json!({"dark": true});
    file["levels_complete"]["1.1"]["bonus"] = json!("ignored");
    let sandbox = Sandbox::with_bytes(file.to_string());

    let state = sandbox.load();
    sandbox.store.save(&state).expect("the state saves");

    // Pydantic ignores extra fields too, so Python would write the same bytes.
    assert_eq!(sandbox.text(), golden_text("python_v2_1.json"));
}

#[test]
fn duplicate_keys_keep_the_first_position_and_the_last_value() {
    // A Python dict does exactly this when `json.load` meets a repeated key.
    let sandbox = Sandbox::with_bytes(
        r#"{"username": "tester", "workspace": "/tmp/shellgame-tester", "current_level": "1.1",
            "start_time": "2026-03-01T10:20:30",
            "level_attempts": {"1.1": 1, "1.2": 2, "1.1": 5}}"#,
    );

    assert_eq!(
        entries(&sandbox.load().level_attempts),
        [("1.1", 5), ("1.2", 2)]
    );
}

#[test]
fn wrongly_typed_fields_are_load_errors() {
    let broken = [
        json!({"levels_complete": []}),
        json!({"levels_complete": {"1.1": {"time_sec": -1, "hints": 0, "attempts": 0,
                                           "completed_at": "2026-03-01T10:20:30"}}}),
        json!({"levels_complete": {"1.1": {"time_sec": 1, "hints": 0, "attempts": 0}}}),
        json!({"level_attempts": {"1.1": "three"}}),
        json!({"level_hints_used": {"1.1": 1.5}}),
        json!({"level_started_at": {"1.1": 12}}),
        json!({"completed_at": 5}),
        json!({"username": null}),
        json!({"workspace": 7}),
        json!({"current_level": 1.1}),
    ];

    for overrides in broken {
        let mut file: Value = serde_json::from_str(&golden_text("python_v2_1_fresh.json")).unwrap();
        for (key, value) in overrides.as_object().expect("object") {
            file[key] = value.clone();
        }
        let sandbox = Sandbox::with_bytes(file.to_string());

        let error = sandbox.load_error();

        assert!(
            matches!(error, StateError::Load { .. }),
            "{overrides}: {error:?}"
        );
        assert_eq!(
            sandbox.text(),
            file.to_string(),
            "{overrides}: file was modified"
        );
    }
}

#[test]
fn other_unreadable_states_are_load_errors_too() {
    // Empty file, a JSON scalar, a BOM (Python's `json.load` refuses it too) and invalid UTF-8.
    let contents: [&[u8]; 4] = [
        b"",
        b"42",
        b"\xef\xbb\xbf{}",
        b"{\"username\": \"\xff\xfe\"}",
    ];
    for bytes in contents {
        let sandbox = Sandbox::with_bytes(bytes);

        let error = sandbox.load_error();

        assert!(
            matches!(error, StateError::Load { .. }),
            "{bytes:?}: {error:?}"
        );
        assert_eq!(
            sandbox.bytes(),
            bytes,
            "{bytes:?}: loading must not modify the file"
        );
    }
}

#[test]
fn a_directory_in_place_of_the_file_is_a_load_error() {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.store.path()).expect("create directory");

    assert!(matches!(sandbox.load_error(), StateError::Load { .. }));
    assert!(sandbox.store.path().is_dir());
}

#[test]
fn a_trailing_newline_is_accepted_and_not_written_back() {
    let sandbox = Sandbox::with_bytes(format!("{}\n", golden_text("python_v2_1.json")));

    let state = sandbox.load();
    sandbox.store.save(&state).expect("the state saves");

    assert_eq!(sandbox.text(), golden_text("python_v2_1.json"));
}

#[test]
fn no_file_means_no_state() {
    let sandbox = Sandbox::empty();
    assert_eq!(sandbox.store.load().expect("no error"), None);
    assert!(!sandbox.store.exists());

    // A missing directory is no error either ...
    let missing = StateStore::new(sandbox.dir.path().join("not/created/yet"));
    assert_eq!(missing.load().expect("no error"), None);

    // ... and neither is a regular file where the directory should be: no state can exist there.
    let blocker = sandbox.dir.path().join("blocker");
    fs::write(&blocker, "x").expect("write file");
    assert_eq!(
        StateStore::new(blocker.join("shellgame"))
            .load()
            .expect("no error"),
        None
    );
    assert!(!sandbox.store.exists());
}

// ---------------------------------------------------------------------------------------------
// Timestamps
// ---------------------------------------------------------------------------------------------

#[test]
fn timestamps_are_written_like_pydantic() {
    let mut state = State::new("jana", "/tmp/shellgame-jana", at(2026, 3, 1, 10, 20, 30));
    for (id, micros) in [
        ("1.1", 5),
        ("1.2", 100_000),
        ("1.3", 999_999),
        ("1.4", 0),
        ("1.5", 120),
    ] {
        state
            .level_started_at
            .insert(id.to_owned(), at_micros(2026, 3, 1, 10, 20, 30, micros));
    }
    state.completed_at = Some(at_micros(2026, 12, 31, 23, 59, 59, 123_456));
    let sandbox = Sandbox::empty();
    sandbox.store.save(&state).expect("the state saves");
    let text = sandbox.text();

    for expected in [
        // No fraction at all when the microseconds are zero ...
        r#""start_time": "2026-03-01T10:20:30","#,
        r#""0.0": "2026-03-01T10:20:30","#,
        r#""1.4": "2026-03-01T10:20:30","#,
        // ... otherwise always six digits, trailing zeros included.
        r#""1.1": "2026-03-01T10:20:30.000005","#,
        r#""1.2": "2026-03-01T10:20:30.100000","#,
        r#""1.3": "2026-03-01T10:20:30.999999","#,
        r#""1.5": "2026-03-01T10:20:30.000120""#,
        r#""completed_at": "2026-12-31T23:59:59.123456""#,
    ] {
        assert!(text.contains(expected), "missing {expected} in\n{text}");
    }
}

#[test]
fn timestamps_are_truncated_to_microseconds_like_python() {
    let nanos = NaiveDate::from_ymd_opt(2026, 3, 1)
        .and_then(|date| date.and_hms_nano_opt(10, 20, 30, 123_456_789))
        .expect("valid timestamp");

    let mut state = State::new("jana", "/tmp/shellgame-jana", nanos);
    state.record_completion("0.0", nanos);
    let sandbox = Sandbox::empty();
    sandbox.store.save(&state).expect("the state saves");

    assert!(
        sandbox
            .text()
            .contains(r#""start_time": "2026-03-01T10:20:30.123456""#)
    );
    // What is saved is what is held: nothing changes on a round trip.
    assert_eq!(sandbox.load(), state);
}

fn local_from_utc(utc: NaiveDateTime) -> NaiveDateTime {
    Local.from_utc_datetime(&utc).naive_local()
}

#[test]
fn timestamps_are_read_leniently() {
    let micros = |value| at_micros(2026, 3, 1, 10, 20, 30, value);
    let instant = at(2026, 3, 1, 10, 20, 30);

    let cases: Vec<(&str, NaiveDateTime)> = vec![
        // What pydantic writes, and what `datetime.isoformat()` / `str(datetime)` produce.
        ("2026-03-01T10:20:30", at(2026, 3, 1, 10, 20, 30)),
        ("2026-03-01T10:20:30.123456", micros(123_456)),
        ("2026-03-01 10:20:30.123456", micros(123_456)),
        // Any number of fraction digits; more than six are truncated, not rounded.
        ("2026-03-01T10:20:30.1", micros(100_000)),
        ("2026-03-01T10:20:30.123", micros(123_000)),
        ("2026-03-01T10:20:30.123456789", micros(123_456)),
        ("2026-03-01T10:20:30.9999999999", micros(999_999)),
        ("2026-03-01T10:20:30.0000009", micros(0)),
        ("2026-03-01T10:20:30,5", micros(500_000)),
        // Other separators, a missing seconds part, a bare date.
        ("2026-03-01t10:20:30", at(2026, 3, 1, 10, 20, 30)),
        ("2026-03-01_10:20:30", at(2026, 3, 1, 10, 20, 30)),
        ("2026-03-01T10:20", at(2026, 3, 1, 10, 20, 0)),
        ("2026-03-01", at(2026, 3, 1, 0, 0, 0)),
        // Offsets are converted to local time; all of these are the same instant.
        ("2026-03-01T10:20:30Z", local_from_utc(instant)),
        ("2026-03-01T10:20:30z", local_from_utc(instant)),
        ("2026-03-01T12:20:30+02:00", local_from_utc(instant)),
        ("2026-03-01T12:20:30+0200", local_from_utc(instant)),
        ("2026-03-01T05:20:30-05:00", local_from_utc(instant)),
        ("2026-03-01T15:50:30+05:30", local_from_utc(instant)),
        (
            "2026-03-01T10:20:30.5Z",
            local_from_utc(at_micros(2026, 3, 1, 10, 20, 30, 500_000)),
        ),
        (
            "2026-03-01T10:20:30.123456789Z",
            local_from_utc(micros(123_456)),
        ),
        // The edges of Python's `datetime`.
        ("0001-01-01T00:00:00", at(1, 1, 1, 0, 0, 0)),
        (
            "9999-12-31T23:59:59.999999",
            at_micros(9999, 12, 31, 23, 59, 59, 999_999),
        ),
    ];

    let sandbox = Sandbox::empty();
    for (text, expected) in cases {
        sandbox.write(state_file_with_start_time(&json!(text)));
        let state = sandbox
            .store
            .load()
            .unwrap_or_else(|error| panic!("{text}: {error}"));
        assert_eq!(
            state.expect("state").start_time,
            expected,
            "reading {text:?}"
        );
    }
}

#[test]
fn malformed_timestamps_are_load_errors() {
    let broken = [
        json!(""),
        json!("yesterday"),
        json!("2026-13-01T00:00:00"),
        json!("2026-02-30T00:00:00"),
        json!("2025-02-29T00:00:00"),
        json!("0000-01-01T00:00:00"),
        json!("2026-3-1T10:20:30"),
        json!("20260301T102030"),
        json!("2026-03-01T24:00:00"),
        json!("2026-03-01T10:60:00"),
        json!("2026-03-01T10:20:61"),
        json!("2026-03-01T10"),
        json!("2026-03-01T10:20:30."),
        json!("2026-03-01T10:20:30.Z"),
        json!("2026-03-01T10:20:30 "),
        json!(" 2026-03-01T10:20:30"),
        json!("2026-03-01T10:20:30Zjunk"),
        json!("2026-03-01T10:20:30+02"),
        json!("2026-03-01T10:20:30+24:00"),
        json!("2026-03-01T10:20:30+02:60"),
        json!("2026-03-01T10:20:30 +02:00"),
        json!("2026-03-01X10:20:30"),
        json!(1_772_360_430),
        json!(null),
    ];

    let sandbox = Sandbox::empty();
    for start_time in broken {
        sandbox.write(state_file_with_start_time(&start_time));
        let error = sandbox.load_error();
        assert!(
            matches!(error, StateError::Load { .. }),
            "{start_time}: {error:?}"
        );
    }
}

#[test]
fn timestamps_are_read_exactly_like_pydantic_reads_them() {
    // `timestamp_table.json` is what the real pydantic made of each string.
    let rows: Vec<Value> = serde_json::from_str(&golden_text("timestamp_table.json"))
        .expect("timestamp table is JSON");
    let (mut accepted, mut refused) = (0, 0);

    let sandbox = Sandbox::empty();
    for row in &rows {
        let text = row["text"].as_str().expect("text");
        sandbox.write(state_file_with_start_time(&json!(text)));
        let loaded = sandbox.store.load();

        if row["wall"].is_null() {
            assert!(
                matches!(loaded, Err(StateError::Load { .. })),
                "pydantic refuses {text:?}, but Rust loaded {loaded:?}"
            );
            refused += 1;
            continue;
        }

        let wall: Vec<u32> = row["wall"]
            .as_array()
            .expect("wall")
            .iter()
            .map(|field| u32::try_from(field.as_u64().expect("number")).expect("small"))
            .collect();
        let wall_clock = at_micros(
            i32::try_from(wall[0]).expect("year"),
            wall[1],
            wall[2],
            wall[3],
            wall[4],
            wall[5],
            wall[6],
        );
        let expected = match row["offset"].as_i64() {
            None => wall_clock,
            Some(offset) => local_from_utc(wall_clock - TimeDelta::seconds(offset)),
        };

        let state = loaded
            .unwrap_or_else(|error| panic!("pydantic accepts {text:?}, but Rust says: {error}"))
            .expect("state exists");
        assert_eq!(state.start_time, expected, "reading {text:?}");
        accepted += 1;
    }

    // The table must keep covering both sides.
    assert!(
        accepted > 50 && refused > 50,
        "{accepted} accepted, {refused} refused"
    );
}

// ---------------------------------------------------------------------------------------------
// Saving
// ---------------------------------------------------------------------------------------------

/// A state with every kind of content, built through the public API.
fn rust_built_state() -> State {
    let mut state = State::new(
        "Žofie \"Zoe\" Černá\t😀",
        "/tmp//shellgame test/./ws/",
        at_micros(2026, 9, 1, 12, 0, 0, 250_000),
    );
    state.record_completion("0.0", at_micros(2026, 9, 1, 12, 0, 5, 1));

    state.current_level = "1.12".to_owned();
    state.ensure_level_started("1.12", at_micros(2026, 9, 1, 12, 0, 5, 1));
    state.record_attempt("1.12");
    state.record_attempt("1.12");
    state.reveal_next_hint("1.12", 3);
    state.reveal_next_hint("1.12", 3);
    state.record_completion("1.12", at(2026, 9, 1, 12, 5, 0));

    state.current_level = "10.4".to_owned();
    state.ensure_level_started("10.4", at(2026, 9, 1, 12, 5, 0));
    state.record_completion("10.4", at_micros(2026, 9, 1, 12, 7, 30, 999_999));
    state.completed_at = Some(at_micros(2026, 9, 1, 12, 7, 30, 999_999));
    state
}

#[test]
fn saved_state_loads_back_unchanged() {
    let sandbox = Sandbox::empty();
    let state = rust_built_state();

    sandbox.store.save(&state).expect("the state saves");

    assert_eq!(sandbox.load(), state);
    // And saving what was loaded is a fixed point.
    let first = sandbox.bytes();
    sandbox
        .store
        .save(&sandbox.load())
        .expect("the state saves");
    assert_eq!(sandbox.bytes(), first);
}

#[test]
fn save_replaces_an_existing_state_and_leaves_only_the_state_file() {
    let sandbox = Sandbox::with_golden("python_v2_1.json");
    let mut state = sandbox.load();
    state.current_level = "10.6".to_owned();
    state.record_attempt("10.5");

    sandbox.store.save(&state).expect("the state saves");

    assert_eq!(sandbox.load(), state);
    assert_eq!(sandbox.file_names(), ["state.json"]);
}

#[test]
fn saved_file_is_private() {
    let sandbox = Sandbox::empty();
    sandbox
        .store
        .save(&rust_built_state())
        .expect("the state saves");

    // Python's `NamedTemporaryFile` also leaves the state file at 0600.
    let mode = fs::metadata(sandbox.store.path())
        .expect("metadata")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn save_creates_the_state_directory() {
    let root = tempfile::tempdir().expect("temporary directory");
    let store = StateStore::new(root.path().join("config/shellgame"));
    assert!(!store.exists());

    store.save(&rust_built_state()).expect("the state saves");

    assert!(store.exists());
    assert_eq!(store.load().expect("loads"), Some(rust_built_state()));
}

/// Makes a directory read-only and restores it on drop, so that the temporary directory can be
/// cleaned up even when an assertion fails.
struct ReadOnly<'a>(&'a Path);

impl<'a> ReadOnly<'a> {
    fn new(dir: &'a Path) -> Self {
        fs::set_permissions(dir, fs::Permissions::from_mode(0o555)).expect("make read-only");
        Self(dir)
    }

    /// Root ignores permission bits; then these tests prove nothing and must be skipped.
    fn is_enforced(&self) -> bool {
        let probe = self.0.join("probe");
        let writable = fs::File::create(&probe).is_ok();
        let _ = fs::remove_file(probe);
        !writable
    }
}

impl Drop for ReadOnly<'_> {
    fn drop(&mut self) {
        let _ = fs::set_permissions(self.0, fs::Permissions::from_mode(0o755));
    }
}

#[test]
fn saving_into_a_read_only_directory_fails_and_leaves_everything_as_it_was() {
    let sandbox = Sandbox::with_golden("python_v2_1_fresh.json");
    let before = sandbox.bytes();
    let state = sandbox.load();

    let read_only = ReadOnly::new(sandbox.dir.path());
    if !read_only.is_enforced() {
        eprintln!("skipping: file permissions are not enforced here (running as root?)");
        return;
    }
    let result = sandbox.store.save(&state);
    drop(read_only);

    let error = result.expect_err("the directory is read-only");
    let prefix = format!(
        "Stav ShellGame nelze uložit do {}: ",
        sandbox.store.path().display()
    );
    assert!(matches!(error, StateError::Save { .. }), "{error:?}");
    assert!(error.to_string().starts_with(&prefix), "{error}");
    assert_eq!(
        sandbox.bytes(),
        before,
        "the previous state must survive a failed save"
    );
    assert_eq!(
        sandbox.file_names(),
        ["state.json"],
        "no temporary file may remain"
    );
}

#[test]
fn saving_into_a_new_directory_below_a_read_only_one_fails() {
    let sandbox = Sandbox::empty();
    let read_only = ReadOnly::new(sandbox.dir.path());
    if !read_only.is_enforced() {
        eprintln!("skipping: file permissions are not enforced here (running as root?)");
        return;
    }
    let store = StateStore::new(sandbox.dir.path().join("shellgame"));

    let result = store.save(&rust_built_state());
    drop(read_only);

    assert!(matches!(result, Err(StateError::Save { .. })), "{result:?}");
    assert!(sandbox.file_names().is_empty());
}

#[test]
fn a_failing_replace_leaves_no_temporary_file() {
    // A non-empty directory cannot be replaced by a file, not even by root, so the write
    // succeeds and only the final rename fails.
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.store.path()).expect("create directory");
    fs::write(sandbox.store.path().join("keep"), "x").expect("write file");

    let error = sandbox
        .store
        .save(&rust_built_state())
        .expect_err("rename must fail");

    assert!(matches!(error, StateError::Save { .. }), "{error:?}");
    assert_eq!(
        sandbox.file_names(),
        ["state.json"],
        "only the directory may remain"
    );
    assert!(sandbox.store.path().join("keep").is_file());
}

#[test]
fn saving_below_a_regular_file_fails() {
    let root = tempfile::tempdir().expect("temporary directory");
    let blocker = root.path().join("blocker");
    fs::write(&blocker, "x").expect("write file");
    let store = StateStore::new(blocker.join("shellgame"));

    let error = store
        .save(&rust_built_state())
        .expect_err("no directory can be created");

    assert!(matches!(error, StateError::Save { .. }), "{error:?}");
    assert_eq!(fs::read(&blocker).expect("read"), b"x");
}

#[test]
fn a_state_that_python_could_not_store_is_not_written() {
    let sandbox = Sandbox::with_golden("python_v2_1_fresh.json");
    let before = sandbox.bytes();
    let mut state = sandbox.load();
    // Python's datetime stops at year 9999.
    state.completed_at = NaiveDate::from_ymd_opt(10_000, 1, 1).and_then(|d| d.and_hms_opt(0, 0, 0));

    let error = sandbox
        .store
        .save(&state)
        .expect_err("year 10000 cannot be written");

    assert!(matches!(error, StateError::Save { .. }), "{error:?}");
    assert_eq!(sandbox.bytes(), before);
    assert_eq!(sandbox.file_names(), ["state.json"]);
}

#[test]
fn a_workspace_that_is_not_utf8_cannot_be_stored() {
    // JSON text is UTF-8; Python could not store such a path either.
    let sandbox = Sandbox::with_golden("python_v2_1_fresh.json");
    let before = sandbox.bytes();
    let mut state = sandbox.load();
    state.workspace = PathBuf::from(OsStr::from_bytes(b"/tmp/shellgame-\xff"));

    let error = sandbox.store.save(&state).expect_err("not representable");

    assert!(matches!(error, StateError::Save { .. }), "{error:?}");
    assert_eq!(sandbox.bytes(), before);
    assert_eq!(sandbox.file_names(), ["state.json"]);
}

#[test]
fn remove_deletes_the_state_and_tolerates_a_missing_file() {
    let sandbox = Sandbox::with_golden("python_v2_1.json");
    assert!(sandbox.store.exists());

    sandbox.store.remove().expect("the state is removed");
    assert!(!sandbox.store.exists());
    assert_eq!(sandbox.store.load().expect("no error"), None);

    sandbox.store.remove().expect("removing nothing is fine");
    StateStore::new(sandbox.dir.path().join("no/such/dir"))
        .remove()
        .expect("removing from a missing directory is fine");
}

#[test]
fn remove_works_on_a_corrupt_state() {
    // `shellgame remove` is the way out of a corrupt state.
    let sandbox = Sandbox::with_golden("corrupt.json");

    sandbox.store.remove().expect("the state is removed");

    assert!(!sandbox.store.exists());
    assert_eq!(sandbox.store.load().expect("no error"), None);
}

#[test]
fn remove_reports_a_file_it_cannot_delete() {
    let sandbox = Sandbox::with_golden("python_v2_1_fresh.json");
    let read_only = ReadOnly::new(sandbox.dir.path());
    if !read_only.is_enforced() {
        eprintln!("skipping: file permissions are not enforced here (running as root?)");
        return;
    }

    let result = sandbox.store.remove();
    drop(read_only);

    let error = result.expect_err("the directory is read-only");
    assert!(matches!(error, StateError::Remove { .. }), "{error:?}");
    assert!(sandbox.store.exists());
}

// ---------------------------------------------------------------------------------------------
// The other direction: Python reads what Rust wrote
// ---------------------------------------------------------------------------------------------

/// Exit status of `tests/state/verify_python_load.py` when Python rejects the file.
const PYTHON_REJECTED: i32 = 42;

enum Verdict {
    Accepted,
    Rejected(String),
    /// `uv`, pydantic or the Python sources are not available here.
    Unavailable(String),
}

/// Loads `state_dir/state.json` with the real Python implementation, through `uv` with a pinned
/// pydantic, and checks that Python would write the same bytes back.
fn ask_python(state_dir: &Path) -> Verdict {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let python_sources = manifest.join("src");
    if !python_sources.join("shellgame/state/manager.py").is_file() {
        return Verdict::Unavailable("the Python implementation is no longer in this tree".into());
    }

    let output = Command::new("uv")
        .args([
            "run",
            "--no-project",
            "--with",
            "pydantic==2.12.5",
            "python",
        ])
        .arg(manifest.join("tests/state/verify_python_load.py"))
        .arg(state_dir)
        .env("PYTHONPATH", &python_sources)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .current_dir(state_dir)
        .output();

    match output {
        Err(error) => Verdict::Unavailable(format!("cannot run uv: {error}")),
        Ok(output) if output.status.success() => Verdict::Accepted,
        Ok(output) if output.status.code() == Some(PYTHON_REJECTED) => {
            Verdict::Rejected(String::from_utf8_lossy(&output.stdout).into_owned())
        }
        Ok(output) => Verdict::Unavailable(format!(
            "uv or Python failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )),
    }
}

#[test]
fn python_accepts_files_written_by_rust() {
    // One file Rust built from scratch, one that Rust migrated from the old v1.0 layout.
    let built = Sandbox::empty();
    built
        .store
        .save(&rust_built_state())
        .expect("the state saves");

    let migrated = Sandbox::with_golden("v1_0.json");
    migrated
        .store
        .save(&migrated.load())
        .expect("the state saves");

    for (what, sandbox) in [
        ("built by the API", &built),
        ("migrated from v1.0", &migrated),
    ] {
        match ask_python(sandbox.dir.path()) {
            Verdict::Accepted => {}
            Verdict::Rejected(why) => {
                panic!("Python does not accept the Rust-written state ({what}):\n{why}")
            }
            Verdict::Unavailable(why) => {
                eprintln!("skipping the Python check: {why}");
                return;
            }
        }
    }
}
