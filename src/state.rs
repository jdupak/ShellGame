//! Persistent game state: the `state.json` file shared with the Python implementation.
//!
//! The Rust binary is a drop-in replacement for the Python one. Students switch binaries in
//! the middle of the semester, so both must read and write the very same `state.json`
//! (schema `2.1`). This module therefore reproduces what pydantic's
//! `GameState.model_dump_json(indent=2)` writes, byte for byte, and reads every older layout
//! the Python version has written.
//!
//! * [`State`] and [`LevelCompletion`] are the model, together with the pure progress
//!   mutations of `core/progress.py`.
//! * [`StateStore`] loads, atomically saves and removes the file. A corrupt or unsupported
//!   file is an error, never "no state", and loading never overwrites it.
//! * [`state_dir`] and [`default_workspace`] say where things live. They take the relevant
//!   environment values as arguments: this module never reads the environment or the working
//!   directory itself.
//!
//! # File format
//!
//! Field order, 2-space indentation, `"key": value` spacing, the lack of a trailing newline and
//! non-ASCII text written as-is all match pydantic. Timestamps are naive local times written
//! as `YYYY-MM-DDTHH:MM:SS`, with `.ffffff` (always six digits) appended only when the
//! microseconds are not zero. The per-level maps keep their insertion order, like Python
//! dicts, and level IDs stay strings (`"1.12"`). Unknown fields are ignored. Only `username`,
//! `workspace`, `current_level` and `start_time` are required; a missing `version` means
//! `1.0`.
//!
//! Files of version `1.0` and `2.0` predate the section reordering of `2.1`; loading remaps
//! their level IDs (see [`State`]) and the next save writes version `2.1`.
//!
//! # Example
//!
//! ```
//! use chrono::NaiveDate;
//! use shellgame::state::{State, StateStore};
//!
//! let dir = tempfile::tempdir()?;
//! let store = StateStore::new(dir.path());
//! assert_eq!(store.load()?, None);
//!
//! let now = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap().and_hms_opt(10, 20, 30).unwrap();
//! let mut state = State::new("jana", "/tmp/shellgame-jana", now);
//! state.record_attempt("0.0");
//! store.save(&state)?;
//!
//! assert_eq!(store.load()?, Some(state));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::io::{self, Write as _};
use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
use std::path::{Path, PathBuf};

use chrono::{Local, NaiveDateTime, Timelike as _};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Schema version written by this binary, and by the Python implementation since the
/// section reordering.
pub const CURRENT_STATE_VERSION: &str = "2.1";

/// Name of the state file inside the state directory.
pub const STATE_FILE_NAME: &str = "state.json";

/// ID of the introduction that every new game starts on.
const FIRST_LEVEL_ID: &str = "0.0";

// ---------------------------------------------------------------------------------------------
// Locations
// ---------------------------------------------------------------------------------------------

/// Directory that holds the state file.
///
/// `$SHELLGAME_STATE_DIR` wins, then `$XDG_CONFIG_HOME/shellgame`, then
/// `~/.config/shellgame` (also on macOS). Empty values count as unset, as they do in Python,
/// where `""` is falsy. The caller reads the environment and passes the values in.
#[must_use]
pub fn state_dir(
    state_dir_env: Option<&OsStr>,
    xdg_config_home: Option<&OsStr>,
    home: &Path,
) -> PathBuf {
    if let Some(dir) = non_empty(state_dir_env) {
        normalize_path(Path::new(dir))
    } else if let Some(xdg) = non_empty(xdg_config_home) {
        normalize_path(&Path::new(xdg).join("shellgame"))
    } else {
        normalize_path(&home.join(".config").join("shellgame"))
    }
}

/// Workspace directory of a new game: `$SHELLGAME_WORKSPACE`, else `/tmp/shellgame-{user}`.
///
/// An empty `$SHELLGAME_WORKSPACE` counts as unset.
#[must_use]
pub fn default_workspace(workspace_env: Option<&OsStr>, user: &str) -> PathBuf {
    match non_empty(workspace_env) {
        Some(dir) => normalize_path(Path::new(dir)),
        None => normalize_path(Path::new(&format!("/tmp/shellgame-{user}"))),
    }
}

/// Treats an empty environment value like an unset one.
fn non_empty(value: Option<&OsStr>) -> Option<&OsStr> {
    value.filter(|value| !value.is_empty())
}

/// Normalizes a path the way Python's `pathlib.Path(str)` does, because `str(path)` is what
/// the Python version stored: repeated and trailing slashes and `.` components vanish, the
/// empty path becomes `.`, and exactly two leading slashes survive (POSIX gives them a
/// meaning). `..` is kept as it is, and the file system is never consulted.
fn normalize_path(path: &Path) -> PathBuf {
    let bytes = path.as_os_str().as_bytes();
    let leading_slashes = bytes.iter().take_while(|&&byte| byte == b'/').count();
    let mut normalized: Vec<u8> = match leading_slashes {
        0 => Vec::new(),
        2 => b"//".to_vec(),
        _ => b"/".to_vec(),
    };

    let components = bytes
        .split(|&byte| byte == b'/')
        .filter(|component| !component.is_empty() && *component != b".");
    for (index, component) in components.enumerate() {
        if index > 0 {
            normalized.push(b'/');
        }
        normalized.extend_from_slice(component);
    }

    if normalized.is_empty() {
        normalized.push(b'.');
    }
    PathBuf::from(OsString::from_vec(normalized))
}

// ---------------------------------------------------------------------------------------------
// Clock
// ---------------------------------------------------------------------------------------------

/// The current local time as a naive timestamp, truncated to microseconds like Python's
/// `datetime.now()`.
///
/// Every method of [`State`] that takes a timestamp truncates it the same way, so anything
/// stored in the state survives a save and a load unchanged.
#[must_use]
pub fn now() -> NaiveDateTime {
    truncate_micros(Local::now().naive_local())
}

/// Drops everything below a microsecond (the precision of Python's `datetime`).
fn truncate_micros(timestamp: NaiveDateTime) -> NaiveDateTime {
    // A leap second is represented as nanoseconds above 999_999_999. Python cannot store
    // one, so clamp it to the last microsecond of the second.
    let nanos = timestamp.nanosecond().min(999_999_999) / 1_000 * 1_000;
    timestamp.with_nanosecond(nanos).unwrap_or(timestamp)
}

// ---------------------------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------------------------

/// Everything the game remembers about one player: the contents of `state.json`.
///
/// The fields are written in declaration order, which is the order pydantic uses. The four
/// per-level maps are keyed by level ID and keep insertion order.
///
/// # Level ID migration
///
/// Versions `1.0` and `2.0` numbered the sections differently. When such a file is loaded,
/// the IDs in `current_level` and in the keys of the four per-level maps are remapped
/// (`2.7` and `2.8` swap, `6.7` and `6.8` swap, then section 7 becomes 8, 8 becomes 9, 9
/// becomes 10 and 10 becomes 7), so every level keeps its content and its credit.
///
/// The migration, the version check and the Czech error messages belong to
/// [`StateStore::load`]; deserializing a `State` directly bypasses them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Schema version. [`CURRENT_STATE_VERSION`] once loaded or created.
    #[serde(default = "current_version")]
    pub version: String,
    /// Name of the player; also names the default workspace.
    pub username: String,
    /// Directory in which the level fixtures are built.
    pub workspace: PathBuf,
    /// ID of the level the player is on, for example `"1.12"`.
    pub current_level: String,
    /// When the game was started.
    #[serde(with = "timestamp::scalar")]
    pub start_time: NaiveDateTime,
    /// Finished levels, in the order they were finished.
    #[serde(default)]
    pub levels_complete: IndexMap<String, LevelCompletion>,
    /// Failed submissions per level.
    #[serde(default)]
    pub level_attempts: IndexMap<String, u32>,
    /// Number of revealed hints per level.
    #[serde(default)]
    pub level_hints_used: IndexMap<String, u32>,
    /// When the player first saw each level.
    #[serde(default, with = "timestamp::map")]
    pub level_started_at: IndexMap<String, NaiveDateTime>,
    /// When the player finished the last level; `None` while the game is still running.
    #[serde(default, with = "timestamp::option")]
    pub completed_at: Option<NaiveDateTime>,
}

fn current_version() -> String {
    CURRENT_STATE_VERSION.to_owned()
}

/// The record of one finished level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelCompletion {
    /// Whole seconds between first seeing the level and finishing it.
    pub time_sec: u64,
    /// Hints revealed on the level.
    pub hints: u32,
    /// Failed submissions on the level.
    pub attempts: u32,
    /// When the level was finished.
    #[serde(with = "timestamp::scalar")]
    pub completed_at: NaiveDateTime,
}

impl State {
    /// A fresh game on the introduction `0.0` (Python's `StateManager.create`).
    ///
    /// `workspace` is normalized like a Python `Path`. The timer of the first level starts
    /// at `now`.
    #[must_use]
    pub fn new(
        username: impl Into<String>,
        workspace: impl AsRef<Path>,
        now: NaiveDateTime,
    ) -> Self {
        let now = truncate_micros(now);
        Self {
            version: current_version(),
            username: username.into(),
            workspace: normalize_path(workspace.as_ref()),
            current_level: FIRST_LEVEL_ID.to_owned(),
            start_time: now,
            levels_complete: IndexMap::new(),
            level_attempts: IndexMap::new(),
            level_hints_used: IndexMap::new(),
            level_started_at: IndexMap::from([(FIRST_LEVEL_ID.to_owned(), now)]),
            completed_at: None,
        }
    }

    /// Counts one failed submission on level `id`. Successful ones are never recorded here.
    pub fn record_attempt(&mut self, id: &str) {
        let attempts = self.level_attempts.entry(id.to_owned()).or_insert(0);
        *attempts = attempts.saturating_add(1);
    }

    /// Starts the timer of level `id` at `now`, unless it is already running.
    ///
    /// Returns whether the timer was started, that is, whether the state changed and needs
    /// saving.
    pub fn ensure_level_started(&mut self, id: &str, now: NaiveDateTime) -> bool {
        if self.level_started_at.contains_key(id) {
            return false;
        }
        self.level_started_at
            .insert(id.to_owned(), truncate_micros(now));
        true
    }

    /// Records level `id` as finished at `now` and returns the new record.
    ///
    /// The time is the number of whole seconds since the level was started, never negative
    /// and 0 when the start was never recorded. Hints and attempts are copied from the
    /// per-level counters.
    pub fn record_completion(&mut self, id: &str, now: NaiveDateTime) -> LevelCompletion {
        let now = truncate_micros(now);
        let time_sec = self.level_started_at.get(id).map_or(0, |&started| {
            u64::try_from(now.signed_duration_since(started).num_seconds()).unwrap_or(0)
        });
        let completion = LevelCompletion {
            time_sec,
            hints: self.level_hints_used.get(id).copied().unwrap_or(0),
            attempts: self.level_attempts.get(id).copied().unwrap_or(0),
            completed_at: now,
        };
        self.levels_complete
            .insert(id.to_owned(), completion.clone());
        completion
    }

    /// How many of the level's `total_hints` hints the player has seen already.
    #[must_use]
    pub fn hint_status(&self, id: &str, total_hints: usize) -> usize {
        let revealed = self.level_hints_used.get(id).copied().unwrap_or(0);
        usize::try_from(revealed)
            .unwrap_or(usize::MAX)
            .min(total_hints)
    }

    /// Reveals the next hint of level `id`.
    ///
    /// Returns the index of the newly revealed hint, or `None` when all `total_hints` hints
    /// have been revealed already (the state is then left unchanged).
    pub fn reveal_next_hint(&mut self, id: &str, total_hints: usize) -> Option<usize> {
        let revealed = self.hint_status(id, total_hints);
        if revealed >= total_hints {
            return None;
        }
        let count = u32::try_from(revealed + 1).unwrap_or(u32::MAX);
        self.level_hints_used.insert(id.to_owned(), count);
        Some(revealed)
    }

    /// Whether the player has finished the whole game.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Forgets everything recorded for level `id` and starts its timer anew at `now`.
    ///
    /// Also clears `completed_at`, so a finished game becomes playable again. Meant for
    /// `dev jump`, which must be able to replay any level from scratch. Other levels keep
    /// their entries and their order.
    pub fn reset_level_progress(&mut self, id: &str, now: NaiveDateTime) {
        self.levels_complete.shift_remove(id);
        self.level_attempts.shift_remove(id);
        self.level_hints_used.shift_remove(id);
        self.level_started_at
            .insert(id.to_owned(), truncate_micros(now));
        self.completed_at = None;
    }
}

// ---------------------------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------------------------

/// A failure to read, write or remove the state file.
///
/// The messages are the Czech ones the Python version prints, followed by a detail that says
/// what went wrong.
#[derive(Debug)]
pub enum StateError {
    /// An existing state file could not be read, parsed or migrated. It is left untouched.
    Load {
        /// The state file.
        file: PathBuf,
        /// What went wrong.
        detail: String,
    },
    /// The state could not be written. The previous file, if any, is intact.
    Save {
        /// The state file.
        file: PathBuf,
        /// What went wrong.
        detail: String,
    },
    /// The state file exists but could not be deleted.
    Remove {
        /// The state file.
        file: PathBuf,
        /// What went wrong.
        detail: String,
    },
}

impl StateError {
    /// The state file the error is about.
    #[must_use]
    pub fn file(&self) -> &Path {
        match self {
            Self::Load { file, .. } | Self::Save { file, .. } | Self::Remove { file, .. } => file,
        }
    }

    /// What went wrong, without the leading sentence.
    #[must_use]
    pub fn detail(&self) -> &str {
        match self {
            Self::Load { detail, .. } | Self::Save { detail, .. } | Self::Remove { detail, .. } => {
                detail
            }
        }
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load { file, detail } => {
                write!(
                    f,
                    "Stav ShellGame nelze načíst z {}: {detail}",
                    file.display()
                )
            }
            Self::Save { file, detail } => {
                write!(
                    f,
                    "Stav ShellGame nelze uložit do {}: {detail}",
                    file.display()
                )
            }
            Self::Remove { file, detail } => {
                write!(
                    f,
                    "Stav ShellGame nelze odstranit ({}): {detail}",
                    file.display()
                )
            }
        }
    }
}

impl std::error::Error for StateError {}

// ---------------------------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------------------------

/// Reads and writes the state file in one state directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateStore {
    dir: PathBuf,
    path: PathBuf,
}

impl StateStore {
    /// A store for `dir/state.json`. Nothing is touched until a method is called.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        let path = dir.join(STATE_FILE_NAME);
        Self { dir, path }
    }

    /// The state directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The state file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether a state file exists.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Loads the state, migrating old versions.
    ///
    /// `Ok(None)` means there is no state file yet. A file that exists but cannot be read,
    /// parsed or migrated is a [`StateError::Load`], and it is left exactly as it was: the
    /// caller must not mistake it for a fresh start.
    ///
    /// # Errors
    ///
    /// [`StateError::Load`] for I/O errors, malformed JSON, a root that is not an object, an
    /// unsupported version, and missing or invalid fields.
    pub fn load(&self) -> Result<Option<State>, StateError> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if is_missing(&error) => return Ok(None),
            Err(error) => return Err(self.load_error(error)),
        };
        parse_state(&text)
            .map(Some)
            .map_err(|detail| self.load_error(detail))
    }

    /// Writes the state atomically: the JSON goes into a private temporary file in the same
    /// directory, is flushed to disk, and then replaces `state.json` in one step. The
    /// directory is created if needed. On failure no temporary file stays behind and the
    /// previous `state.json` is untouched.
    ///
    /// # Errors
    ///
    /// [`StateError::Save`] if the state cannot be serialized or the file cannot be written.
    pub fn save(&self, state: &State) -> Result<(), StateError> {
        let json = render_state(state).map_err(|detail| self.save_error(detail))?;
        self.write_atomically(json.as_bytes())
            .map_err(|detail| self.save_error(detail))
    }

    /// Deletes the state file. A missing file is fine.
    ///
    /// # Errors
    ///
    /// [`StateError::Remove`] if the file exists but cannot be deleted.
    pub fn remove(&self) -> Result<(), StateError> {
        match fs::remove_file(&self.path) {
            Err(error) if !is_missing(&error) => Err(StateError::Remove {
                file: self.path.clone(),
                detail: error.to_string(),
            }),
            _ => Ok(()),
        }
    }

    fn write_atomically(&self, bytes: &[u8]) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        // `NamedTempFile` is created with mode 0600 and deleted when dropped, which covers
        // every early return below, including a failing rename.
        let mut temporary = tempfile::Builder::new()
            .prefix(".state-")
            .suffix(".json.tmp")
            .tempfile_in(&self.dir)?;
        temporary.write_all(bytes)?;
        temporary.as_file().sync_all()?;
        temporary.persist(&self.path).map_err(|error| error.error)?;
        Ok(())
    }

    fn load_error(&self, detail: impl fmt::Display) -> StateError {
        StateError::Load {
            file: self.path.clone(),
            detail: detail.to_string(),
        }
    }

    fn save_error(&self, detail: impl fmt::Display) -> StateError {
        StateError::Save {
            file: self.path.clone(),
            detail: detail.to_string(),
        }
    }
}

/// Python's `Path.exists()` is false for these errors, so a state file that cannot exist
/// there (for example below a regular file) counts as absent too.
fn is_missing(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
    )
}

// ---------------------------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------------------------

/// Parses the text of a state file of any supported version.
///
/// The steps and their order follow Python's `StateManager.load`: the root must be an object,
/// then the version decides how the data is migrated, and only then are the fields checked.
/// The error is the detail that follows the Czech sentence in [`StateError::Load`].
fn parse_state(text: &str) -> Result<State, String> {
    let root: Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let migrated = migrate(root)?;
    serde_json::from_value(migrated).map_err(|error| error.to_string())
}

/// Renders the state exactly like pydantic's `model_dump_json(indent=2)`: two-space indent,
/// `"key": value`, empty containers as `{}`, no trailing newline and non-ASCII as-is.
fn render_state(state: &State) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(state)
}

/// Brings a decoded state file to [`CURRENT_STATE_VERSION`] (Python's `_migrate`).
fn migrate(mut root: Value) -> Result<Value, String> {
    let Value::Object(object) = &mut root else {
        return Err("Kořen souboru stavu musí být objekt.".to_owned());
    };

    let version = object
        .get("version")
        .map_or_else(|| "1.0".to_owned(), python_str);
    match version.as_str() {
        // Both predate the section reordering, so their level IDs point at whatever now
        // happens to carry that number. Remap them explicitly: otherwise the nearest-match
        // fallback for unknown levels would silently resume a player on unrelated content
        // and keep their finished levels credited to the wrong section.
        "1.0" | "2.0" => {
            remap_level_ids(object);
            object.insert(
                "version".to_owned(),
                Value::String(CURRENT_STATE_VERSION.to_owned()),
            );
        }
        CURRENT_STATE_VERSION => {}
        other => return Err(format!("Nepodporovaná verze stavu: {other}")),
    }
    Ok(root)
}

/// Python's `str()` of a decoded JSON value, which is what `_migrate` compares the version
/// with. Only scalars matter in practice.
fn python_str(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => "None".to_owned(),
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        other => other.to_string(),
    }
}

/// Section numbers that changed in 2.1: wildcards moved ahead of permissions and
/// redirection, so globbing is taught before the later sections rely on it.
const SECTION_ROTATION_2_1: [(u32, u32); 4] = [(7, 8), (8, 9), (9, 10), (10, 7)];

/// Levels swapped inside their section in 2.1 so that every section ends on its challenge.
/// Both directions are listed, which makes the mapping its own inverse.
const LEVEL_SWAPS_2_1: [(&str, &str); 4] = [
    ("2.7", "2.8"),
    ("2.8", "2.7"),
    ("6.7", "6.8"),
    ("6.8", "6.7"),
];

/// Keys of a saved state whose values are keyed by level ID.
const LEVEL_KEYED_FIELDS: [&str; 4] = [
    "levels_complete",
    "level_attempts",
    "level_hints_used",
    "level_started_at",
];

/// Maps a pre-reorder level ID onto the level that now holds the same content
/// (Python's `_migrate_level_id_to_2_1`).
///
/// IDs that are not `<digits>.<anything>` are returned unchanged. Only ASCII digits count as
/// digits; Python's `str.isdigit` also accepts other scripts, which no level ID uses.
fn migrate_level_id_to_2_1(level_id: &str) -> String {
    if let Some((_, swapped)) = LEVEL_SWAPS_2_1.iter().find(|(from, _)| *from == level_id) {
        return (*swapped).to_owned();
    }

    let Some((section, number)) = level_id.split_once('.') else {
        return level_id.to_owned();
    };
    if section.is_empty() || !section.bytes().all(|byte| byte.is_ascii_digit()) {
        return level_id.to_owned();
    }

    let rotated = section.parse::<u32>().ok().and_then(|section| {
        SECTION_ROTATION_2_1
            .iter()
            .find(|(from, _)| *from == section)
            .map(|&(_, to)| to)
    });
    match rotated {
        Some(rotated) => format!("{rotated}.{number}"),
        None => level_id.to_owned(),
    }
}

/// Applies [`migrate_level_id_to_2_1`] to `current_level` and to the keys of the per-level
/// maps (Python's `_remap_level_ids`). Entries keep their order; should two keys collide,
/// the later value wins, as in a Python dict comprehension.
fn remap_level_ids(root: &mut serde_json::Map<String, Value>) {
    if let Some(Value::String(current)) = root.get_mut("current_level") {
        *current = migrate_level_id_to_2_1(current);
    }

    for field in LEVEL_KEYED_FIELDS {
        if let Some(Value::Object(entries)) = root.get_mut(field) {
            *entries = std::mem::take(entries)
                .into_iter()
                .map(|(key, value)| (migrate_level_id_to_2_1(&key), value))
                .collect();
        }
    }
}

/// The text format of timestamps and the serde glue for it.
///
/// Writing matches pydantic 2.12 for a naive `datetime`. Reading accepts what pydantic
/// accepts for ISO 8601 text and therefore also what Python's `datetime.isoformat()` and
/// `str()` produce: a `T`, `t`, `_` or space between date and time, optional seconds,
/// any number of fraction digits (beyond microseconds they are truncated), and an optional
/// `Z`/`±HH:MM`/`±HHMM` offset, which is converted to local time. A bare date is read as
/// midnight. Two exotic inputs that pydantic takes are not: a Unicode minus sign in an offset,
/// and a unix timestamp written as text. Nothing that writes state files produces either.
mod timestamp {
    use chrono::{
        Datelike as _, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone as _,
        Timelike as _,
    };
    use indexmap::IndexMap;
    use serde::de::Error as _;
    use serde::ser::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// The years Python's `datetime` can represent.
    const YEARS: std::ops::RangeInclusive<i32> = 1..=9999;

    const SHAPE: &str = "expected YYYY-MM-DDTHH:MM:SS[.ffffff][Z|+HH:MM]";
    const OUT_OF_RANGE: &str = "the year is outside 1..=9999";

    /// `YYYY-MM-DDTHH:MM:SS`, plus `.ffffff` only when the microseconds are not zero.
    pub(super) fn format(timestamp: NaiveDateTime) -> String {
        let micros = timestamp.nanosecond().min(999_999_999) / 1_000;
        let text = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            timestamp.year(),
            timestamp.month(),
            timestamp.day(),
            timestamp.hour(),
            timestamp.minute(),
            timestamp.second(),
        );
        if micros == 0 {
            text
        } else {
            format!("{text}.{micros:06}")
        }
    }

    /// Parses a timestamp; see the module documentation for what is accepted.
    pub(super) fn parse(text: &str) -> Result<NaiveDateTime, &'static str> {
        let bytes = text.as_bytes();
        let (date_part, rest) = bytes.split_first_chunk::<10>().ok_or(SHAPE)?;
        let date = parse_date(date_part).ok_or(SHAPE)?;

        // A bare date is midnight, as in pydantic.
        let Some((&separator, time_part)) = rest.split_first() else {
            return Ok(date.and_time(NaiveTime::MIN));
        };
        if !matches!(separator, b'T' | b't' | b'_' | b' ') {
            return Err(SHAPE);
        }

        let (time, tail) = parse_time(time_part).ok_or(SHAPE)?;
        let wall_clock = date.and_time(time);
        match parse_offset(tail).ok_or(SHAPE)? {
            None => Ok(wall_clock),
            Some(offset) => to_local(wall_clock, offset),
        }
    }

    /// The value of a non-empty run of ASCII digits (and of nothing else).
    fn number(digits: &[u8]) -> Option<u32> {
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
            return None;
        }
        digits.iter().try_fold(0_u32, |acc, &digit| {
            acc.checked_mul(10)?.checked_add(u32::from(digit - b'0'))
        })
    }

    /// `YYYY-MM-DD`. Year 0 does not exist in Python.
    fn parse_date(date: &[u8; 10]) -> Option<NaiveDate> {
        if date[4] != b'-' || date[7] != b'-' {
            return None;
        }
        let year = i32::try_from(number(&date[0..4])?).ok()?;
        if !YEARS.contains(&year) {
            return None;
        }
        NaiveDate::from_ymd_opt(year, number(&date[5..7])?, number(&date[8..10])?)
    }

    /// `HH:MM[:SS[.f+]]`; returns the time and the unparsed rest.
    fn parse_time(bytes: &[u8]) -> Option<(NaiveTime, &[u8])> {
        let hour = number(bytes.get(0..2)?)?;
        if *bytes.get(2)? != b':' {
            return None;
        }
        let minute = number(bytes.get(3..5)?)?;
        let mut rest = &bytes[5..];

        let (mut second, mut micros) = (0, 0);
        if let Some(after_colon) = rest.strip_prefix(b":") {
            second = number(after_colon.get(0..2)?)?;
            rest = &after_colon[2..];

            if let Some(fraction) = rest.strip_prefix(b".").or_else(|| rest.strip_prefix(b",")) {
                let digits = fraction
                    .iter()
                    .take_while(|byte| byte.is_ascii_digit())
                    .count();
                if digits == 0 {
                    return None;
                }
                let (fraction, tail) = fraction.split_at(digits);
                // The first six digits are the microseconds; missing ones count as zero and
                // further ones are truncated, which is what pydantic does.
                micros = (0..6).fold(0, |micros, index| {
                    micros * 10
                        + fraction
                            .get(index)
                            .map_or(0, |&digit| u32::from(digit - b'0'))
                });
                rest = tail;
            }
        }

        Some((
            NaiveTime::from_hms_micro_opt(hour, minute, second, micros)?,
            rest,
        ))
    }

    /// `Z`, `z`, `±HH:MM` or `±HHMM` as seconds east of UTC; `Some(None)` when there is no
    /// offset at all and `None` when what follows is not an offset.
    fn parse_offset(bytes: &[u8]) -> Option<Option<i32>> {
        let (sign, digits) = match bytes {
            [] => return Some(None),
            [b'Z' | b'z'] => return Some(Some(0)),
            [b'+', digits @ ..] => (1, digits),
            [b'-', digits @ ..] => (-1, digits),
            _ => return None,
        };
        let (hours, minutes) = match digits {
            [h1, h2, b':', m1, m2] | [h1, h2, m1, m2] => {
                (number(&[*h1, *h2])?, number(&[*m1, *m2])?)
            }
            _ => return None,
        };
        if hours > 23 || minutes > 59 {
            return None;
        }
        Some(Some(
            sign * i32::try_from(hours * 3600 + minutes * 60).ok()?,
        ))
    }

    /// Converts a wall-clock time at the given offset to local naive time. Python would keep
    /// such a value timezone-aware, which the game could not subtract from its naive
    /// timestamps, so converting is the useful reading.
    fn to_local(
        wall_clock: NaiveDateTime,
        offset_seconds: i32,
    ) -> Result<NaiveDateTime, &'static str> {
        let offset = FixedOffset::east_opt(offset_seconds).ok_or(SHAPE)?;
        let utc = wall_clock.checked_sub_offset(offset).ok_or(OUT_OF_RANGE)?;
        let local = Local.from_utc_datetime(&utc).naive_local();
        if YEARS.contains(&local.year()) {
            Ok(local)
        } else {
            Err(OUT_OF_RANGE)
        }
    }

    fn parse_text<E: serde::de::Error>(text: &str) -> Result<NaiveDateTime, E> {
        parse(text).map_err(|why| E::custom(format_args!("invalid timestamp {text:?}: {why}")))
    }

    /// A timestamp as it is written inside containers.
    struct Stamp(NaiveDateTime);

    impl Serialize for Stamp {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            scalar::serialize(&self.0, serializer)
        }
    }

    /// For `#[serde(with = "timestamp::scalar")]` on a [`NaiveDateTime`].
    pub(super) mod scalar {
        use super::*;

        pub fn serialize<S: Serializer>(
            timestamp: &NaiveDateTime,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            if !YEARS.contains(&timestamp.year()) {
                return Err(S::Error::custom(OUT_OF_RANGE));
            }
            serializer.serialize_str(&format(*timestamp))
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<NaiveDateTime, D::Error> {
            parse_text(&String::deserialize(deserializer)?)
        }
    }

    /// For `#[serde(with = "timestamp::option")]` on an `Option<NaiveDateTime>`.
    pub(super) mod option {
        use super::*;

        pub fn serialize<S: Serializer>(
            timestamp: &Option<NaiveDateTime>,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            match timestamp {
                Some(timestamp) => serializer.serialize_some(&Stamp(*timestamp)),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Option<NaiveDateTime>, D::Error> {
            Option::<String>::deserialize(deserializer)?
                .map(|text| parse_text(&text))
                .transpose()
        }
    }

    /// For `#[serde(with = "timestamp::map")]` on an `IndexMap<String, NaiveDateTime>`.
    pub(super) mod map {
        use super::*;
        use serde::ser::SerializeMap as _;

        pub fn serialize<S: Serializer>(
            map: &IndexMap<String, NaiveDateTime>,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut out = serializer.serialize_map(Some(map.len()))?;
            for (key, timestamp) in map {
                out.serialize_entry(key, &Stamp(*timestamp))?;
            }
            out.end()
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<IndexMap<String, NaiveDateTime>, D::Error> {
            IndexMap::<String, String>::deserialize(deserializer)?
                .into_iter()
                .map(|(key, text)| {
                    let timestamp = parse(&text).map_err(|why| {
                        D::Error::custom(format_args!(
                            "invalid timestamp {text:?} for level {key:?}: {why}"
                        ))
                    })?;
                    Ok((key, timestamp))
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::{NaiveDate, TimeZone as _};
    use serde_json::json;

    use super::*;

    fn at_micros(hour: u32, minute: u32, second: u32, micros: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 3, 1)
            .and_then(|date| date.and_hms_micro_opt(hour, minute, second, micros))
            .expect("valid test timestamp")
    }

    fn at(hour: u32, minute: u32, second: u32) -> NaiveDateTime {
        at_micros(hour, minute, second, 0)
    }

    fn fresh() -> State {
        State::new("tester", "/tmp/shellgame-tester", at(10, 0, 0))
    }

    fn keys<V>(map: &IndexMap<String, V>) -> Vec<&str> {
        map.keys().map(String::as_str).collect()
    }

    // ----- State::new -----

    #[test]
    fn a_new_game_starts_on_the_introduction() {
        let state = fresh();

        assert_eq!(state.version, CURRENT_STATE_VERSION);
        assert_eq!(state.username, "tester");
        assert_eq!(state.workspace, Path::new("/tmp/shellgame-tester"));
        assert_eq!(state.current_level, "0.0");
        assert_eq!(state.start_time, at(10, 0, 0));
        assert!(state.levels_complete.is_empty());
        assert!(state.level_attempts.is_empty());
        assert!(state.level_hints_used.is_empty());
        assert_eq!(
            state.level_started_at,
            IndexMap::from([("0.0".to_owned(), at(10, 0, 0))])
        );
        assert!(!state.is_finished());
    }

    #[test]
    fn a_new_game_normalizes_the_workspace_like_a_python_path() {
        let state = State::new("tester", "/tmp//shellgame test/./ws/", at(10, 0, 0));

        assert_eq!(state.workspace, Path::new("/tmp/shellgame test/ws"));
    }

    #[test]
    fn timestamps_are_truncated_to_microseconds() {
        let nanos = at(10, 0, 0)
            .with_nanosecond(123_456_789)
            .expect("valid nanoseconds");
        let mut state = State::new("tester", "/tmp/ws", nanos);
        state.ensure_level_started("1.1", nanos);
        let completion = state.record_completion("1.1", nanos);

        let expected = at_micros(10, 0, 0, 123_456);
        assert_eq!(state.start_time, expected);
        assert_eq!(state.level_started_at["0.0"], expected);
        assert_eq!(state.level_started_at["1.1"], expected);
        assert_eq!(completion.completed_at, expected);
    }

    #[test]
    fn a_leap_second_is_clamped_to_the_last_microsecond() {
        let leap = NaiveDate::from_ymd_opt(2016, 12, 31)
            .and_then(|date| date.and_hms_nano_opt(23, 59, 59, 1_500_000_000))
            .expect("chrono represents leap seconds");

        let truncated = truncate_micros(leap);

        assert_eq!(truncated.second(), 59);
        assert_eq!(truncated.nanosecond(), 999_999_000);
        assert_eq!(timestamp::format(leap), "2016-12-31T23:59:59.999999");
    }

    #[test]
    fn now_is_current_and_has_microsecond_precision() {
        let before = Local::now().naive_local();
        let stamp = now();
        let after = Local::now().naive_local();

        assert_eq!(stamp.nanosecond() % 1_000, 0);
        assert!(stamp >= truncate_micros(before) && stamp <= after);
    }

    // ----- progress -----

    #[test]
    fn record_attempt_counts_per_level_in_first_seen_order() {
        let mut state = fresh();

        state.record_attempt("1.1");
        state.record_attempt("2.0");
        state.record_attempt("1.1");

        assert_eq!(state.level_attempts["1.1"], 2);
        assert_eq!(state.level_attempts["2.0"], 1);
        assert_eq!(keys(&state.level_attempts), ["1.1", "2.0"]);
    }

    #[test]
    fn ensure_level_started_sets_the_start_only_once() {
        let mut state = fresh();

        assert!(state.ensure_level_started("1.1", at(10, 0, 5)));
        assert!(!state.ensure_level_started("1.1", at(10, 0, 15)));
        assert!(
            !state.ensure_level_started("0.0", at(10, 0, 15)),
            "the first level is running"
        );

        assert_eq!(state.level_started_at["1.1"], at(10, 0, 5));
        assert_eq!(state.level_started_at["0.0"], at(10, 0, 0));
    }

    #[test]
    fn record_completion_uses_the_tracking_data() {
        let mut state = fresh();
        state
            .level_started_at
            .insert("1.1".to_owned(), at(10, 0, 0));
        state.level_attempts.insert("1.1".to_owned(), 2);
        state.level_hints_used.insert("1.1".to_owned(), 1);

        let completion = state.record_completion("1.1", at(10, 0, 12));

        assert_eq!(
            completion,
            LevelCompletion {
                time_sec: 12,
                hints: 1,
                attempts: 2,
                completed_at: at(10, 0, 12),
            }
        );
        assert_eq!(state.levels_complete["1.1"], completion);
    }

    #[test]
    fn record_completion_counts_whole_seconds_only() {
        let mut state = fresh();
        state
            .level_started_at
            .insert("1.1".to_owned(), at_micros(10, 0, 0, 750_000));

        // 12.9 s elapsed: Python's `int(total_seconds())` truncates.
        let completion = state.record_completion("1.1", at_micros(10, 0, 13, 650_000));

        assert_eq!(completion.time_sec, 12);
    }

    #[test]
    fn record_completion_without_a_recorded_start_takes_no_time() {
        let mut state = fresh();

        let completion = state.record_completion("5.2", at(10, 0, 30));

        assert_eq!(completion.time_sec, 0);
        assert_eq!((completion.hints, completion.attempts), (0, 0));
    }

    #[test]
    fn record_completion_never_reports_negative_time() {
        let mut state = fresh();
        state
            .level_started_at
            .insert("1.1".to_owned(), at(10, 0, 30));

        // The clock went backwards.
        let completion = state.record_completion("1.1", at(10, 0, 0));

        assert_eq!(completion.time_sec, 0);
    }

    #[test]
    fn record_completion_replaces_an_earlier_record_in_place() {
        let mut state = fresh();
        state.record_completion("1.1", at(10, 0, 5));
        state.record_completion("1.2", at(10, 0, 6));

        state.record_completion("1.1", at(10, 0, 9));

        assert_eq!(keys(&state.levels_complete), ["1.1", "1.2"]);
        assert_eq!(state.levels_complete["1.1"].completed_at, at(10, 0, 9));
    }

    #[test]
    fn hint_status_is_clamped_to_the_available_hints() {
        let mut state = fresh();
        assert_eq!(state.hint_status("1.1", 3), 0);

        state.level_hints_used.insert("1.1".to_owned(), 2);
        assert_eq!(state.hint_status("1.1", 3), 2);
        assert_eq!(
            state.hint_status("1.1", 1),
            1,
            "fewer hints than were once revealed"
        );
        assert_eq!(state.hint_status("1.1", 0), 0);

        state.level_hints_used.insert("1.1".to_owned(), u32::MAX);
        assert_eq!(state.hint_status("1.1", 3), 3);
    }

    #[test]
    fn reveal_next_hint_walks_through_all_hints_and_then_stops() {
        let mut state = fresh();

        assert_eq!(state.reveal_next_hint("1.1", 3), Some(0));
        assert_eq!(state.reveal_next_hint("1.1", 3), Some(1));
        assert_eq!(state.hint_status("1.1", 3), 2);
        assert_eq!(state.reveal_next_hint("1.1", 3), Some(2));
        assert_eq!(state.reveal_next_hint("1.1", 3), None);
        assert_eq!(state.reveal_next_hint("1.1", 3), None);

        assert_eq!(state.level_hints_used["1.1"], 3);
        assert_eq!(state.hint_status("1.1", 3), 3);
        assert_eq!(
            state.hint_status("1.2", 3),
            0,
            "other levels are unaffected"
        );
    }

    #[test]
    fn reveal_next_hint_changes_nothing_when_there_are_no_hints() {
        let mut state = fresh();
        let before = state.clone();

        assert_eq!(state.reveal_next_hint("1.1", 0), None);

        assert_eq!(state, before);
    }

    #[test]
    fn reveal_next_hint_leaves_an_oversized_counter_alone() {
        let mut state = fresh();
        state.level_hints_used.insert("1.1".to_owned(), 5);

        assert_eq!(state.reveal_next_hint("1.1", 3), None);

        assert_eq!(state.level_hints_used["1.1"], 5);
    }

    #[test]
    fn a_game_is_finished_once_completed_at_is_set() {
        let mut state = fresh();
        assert!(!state.is_finished());

        state.completed_at = Some(at(11, 0, 0));

        assert!(state.is_finished());
    }

    #[test]
    fn reset_level_progress_replays_a_level_from_scratch() {
        let mut state = fresh();
        for id in ["1.1", "1.2", "1.3", "1.4"] {
            state.ensure_level_started(id, at(10, 0, 1));
            state.record_attempt(id);
            state.reveal_next_hint(id, 3);
            state.record_completion(id, at(10, 0, 20));
        }
        state.completed_at = Some(at(11, 0, 0));

        state.reset_level_progress("1.2", at(12, 0, 0));

        assert!(!state.levels_complete.contains_key("1.2"));
        assert!(!state.level_attempts.contains_key("1.2"));
        assert!(!state.level_hints_used.contains_key("1.2"));
        assert_eq!(
            state.level_started_at["1.2"],
            at(12, 0, 0),
            "the timer starts anew"
        );
        assert!(
            !state.is_finished(),
            "a finished game becomes playable again"
        );
        // Everything else keeps its entries and its order.
        assert_eq!(keys(&state.levels_complete), ["1.1", "1.3", "1.4"]);
        assert_eq!(keys(&state.level_attempts), ["1.1", "1.3", "1.4"]);
        assert_eq!(keys(&state.level_hints_used), ["1.1", "1.3", "1.4"]);
        assert_eq!(
            keys(&state.level_started_at),
            ["0.0", "1.1", "1.2", "1.3", "1.4"]
        );
        assert_eq!(state.level_started_at["1.1"], at(10, 0, 1));
        // A replay is timed from the reset, not from the first visit.
        assert_eq!(state.record_completion("1.2", at(12, 0, 7)).time_sec, 7);
    }

    #[test]
    fn reset_level_progress_on_an_untouched_level_just_starts_it() {
        let mut state = fresh();

        state.reset_level_progress("4.1", at(10, 5, 0));

        assert_eq!(state.level_started_at["4.1"], at(10, 5, 0));
        assert!(state.levels_complete.is_empty());
        assert!(state.level_attempts.is_empty());
    }

    // ----- level ID migration -----

    #[test]
    fn level_ids_follow_their_content() {
        for (old, new) in [
            // Untouched sections keep their IDs.
            ("0.0", "0.0"),
            ("1.1", "1.1"),
            ("5.4", "5.4"),
            ("11.6", "11.6"),
            // Permissions, redirection and error streams moved back one slot ...
            ("7.2", "8.2"),
            ("8.5", "9.5"),
            ("9.4", "10.4"),
            // ... and wildcards moved forward.
            ("10.3", "7.3"),
            // The help lesson and the challenge of section 2, the alias aside and the
            // challenge of section 6 swapped.
            ("2.7", "2.8"),
            ("2.8", "2.7"),
            ("6.7", "6.8"),
            ("6.8", "6.7"),
            // Only the section part rotates; the rest is kept as it is.
            ("7.", "8."),
            ("7.1.2", "8.1.2"),
            ("07.1", "8.1"),
            ("10.x", "7.x"),
            // Anything that is not `<digits>.<anything>` stays.
            ("", ""),
            (".", "."),
            ("7", "7"),
            (".7", ".7"),
            ("x.1", "x.1"),
            ("+7.1", "+7.1"),
            (" 7.1", " 7.1"),
            ("100.1", "100.1"),
            ("99999999999999999999.1", "99999999999999999999.1"),
        ] {
            assert_eq!(migrate_level_id_to_2_1(old), new, "migrating {old:?}");
        }
    }

    #[test]
    fn level_id_migration_never_merges_two_levels() {
        let ids: Vec<String> = (0..=12)
            .flat_map(|section| (0..=30).map(move |number| format!("{section}.{number}")))
            .collect();

        let migrated: HashSet<String> = ids.iter().map(|id| migrate_level_id_to_2_1(id)).collect();

        assert_eq!(migrated.len(), ids.len());
        // It only permutes: every ID that exists afterwards existed before.
        assert_eq!(migrated, ids.into_iter().collect::<HashSet<_>>());
    }

    #[test]
    fn remapping_keeps_the_order_and_lets_the_later_duplicate_win() {
        let mut root = json!({
            "current_level": "10.1",
            "level_attempts": {"7.1": 1, "9.9": 2, "07.1": 3, "1.1": 4},
            "level_hints_used": {"6.7": 1},
            "levels_complete": "not a map",
            "unrelated": {"7.1": true},
        });
        let Value::Object(object) = &mut root else {
            unreachable!()
        };

        remap_level_ids(object);

        assert_eq!(root["current_level"], "7.1");
        // `7.1` and `07.1` both become `8.1`: the first position, the last value.
        let attempts: Vec<(&String, &Value)> =
            root["level_attempts"].as_object().unwrap().iter().collect();
        assert_eq!(
            attempts,
            [
                (&"8.1".to_owned(), &json!(3)),
                (&"10.9".to_owned(), &json!(2)),
                (&"1.1".to_owned(), &json!(4))
            ]
        );
        assert_eq!(root["level_hints_used"], json!({"6.8": 1}));
        // What is not a per-level map is left alone, and so is everything else.
        assert_eq!(root["levels_complete"], "not a map");
        assert_eq!(root["unrelated"], json!({"7.1": true}));
    }

    // ----- parsing and versions -----

    fn minimal_file() -> Value {
        json!({
            "version": "2.1",
            "username": "tester",
            "workspace": "/tmp/shellgame-tester",
            "current_level": "7.1",
            "start_time": "2026-03-01T10:00:00",
        })
    }

    fn parse(file: &Value) -> Result<State, String> {
        parse_state(&file.to_string())
    }

    #[test]
    fn version_2_1_is_taken_as_it_is() {
        let state = parse(&minimal_file()).expect("parses");

        assert_eq!(state.version, "2.1");
        assert_eq!(state.current_level, "7.1");
    }

    #[test]
    fn old_and_missing_versions_are_migrated() {
        for version in [Some("1.0"), Some("2.0"), None] {
            let mut file = minimal_file();
            match version {
                Some(version) => file["version"] = json!(version),
                None => drop(file.as_object_mut().expect("object").remove("version")),
            }

            let state = parse(&file).expect("parses");

            assert_eq!(state.version, "2.1", "version {version:?}");
            assert_eq!(state.current_level, "8.1", "version {version:?}");
        }
    }

    #[test]
    fn the_version_is_compared_as_python_would_print_it() {
        for (version, expected) in [
            (json!("99.0"), "Nepodporovaná verze stavu: 99.0"),
            (json!("2.10"), "Nepodporovaná verze stavu: 2.10"),
            (json!(""), "Nepodporovaná verze stavu: "),
            (json!(2), "Nepodporovaná verze stavu: 2"),
            (json!(null), "Nepodporovaná verze stavu: None"),
            (json!(true), "Nepodporovaná verze stavu: True"),
        ] {
            let mut file = minimal_file();
            file["version"] = version.clone();

            assert_eq!(
                parse(&file).expect_err("unsupported"),
                expected,
                "version {version}"
            );
        }

        // Python's `str(1.0)` is "1.0", so that old file is migrated ...
        let mut file = minimal_file();
        file["version"] = json!(1.0);
        assert_eq!(parse(&file).expect("parses").current_level, "8.1");
        // ... while `str(2.1)` passes the version check and then fails validation, because
        // the field is a string.
        file["version"] = json!(2.1);
        assert!(
            parse(&file)
                .expect_err("not a string")
                .contains("expected a string")
        );
    }

    #[test]
    fn the_root_must_be_an_object() {
        for text in ["[]", "[1, 2]", "42", "\"2.1\"", "null", "true"] {
            assert_eq!(
                parse_state(text).expect_err("not an object"),
                "Kořen souboru stavu musí být objekt.",
                "{text}"
            );
        }
        assert!(parse_state("").is_err());
        assert!(parse_state("{not json").is_err());
    }

    #[test]
    fn the_version_is_checked_before_the_fields() {
        // Python validates fields only after migrating, so an unsupported version wins.
        let state = parse_state(r#"{"version": "99.0"}"#);

        assert_eq!(
            state.expect_err("unsupported"),
            "Nepodporovaná verze stavu: 99.0"
        );
    }

    #[test]
    fn missing_required_fields_are_named() {
        for field in ["username", "workspace", "current_level", "start_time"] {
            let mut file = minimal_file();
            file.as_object_mut().expect("object").remove(field);

            let error = parse(&file).expect_err("incomplete");

            assert!(
                error.contains(&format!("missing field `{field}`")),
                "{error}"
            );
        }
    }

    #[test]
    fn optional_fields_default_and_unknown_fields_are_ignored() {
        let mut file = minimal_file();
        file["from_the_future"] = json!([1, 2, 3]);

        let state = parse(&file).expect("parses");

        assert!(state.levels_complete.is_empty());
        assert!(state.level_started_at.is_empty());
        assert_eq!(state.completed_at, None);
        let rendered = render_state(&state).expect("renders");
        assert!(!rendered.contains("from_the_future"));
    }

    #[test]
    fn a_bad_timestamp_names_the_text_and_the_level() {
        let mut file = minimal_file();
        file["level_started_at"] = json!({"1.1": "soon"});

        let error = parse(&file).expect_err("bad timestamp");

        assert!(
            error.contains(r#"invalid timestamp "soon" for level "1.1""#),
            "{error}"
        );
    }

    // ----- timestamps -----

    #[test]
    fn timestamps_are_formatted_like_pydantic() {
        for (timestamp, expected) in [
            (at_micros(10, 20, 30, 0), "2026-03-01T10:20:30"),
            (at_micros(10, 20, 30, 1), "2026-03-01T10:20:30.000001"),
            (at_micros(10, 20, 30, 500_000), "2026-03-01T10:20:30.500000"),
            (at_micros(0, 0, 0, 999_999), "2026-03-01T00:00:00.999999"),
        ] {
            assert_eq!(timestamp::format(timestamp), expected);
        }

        let first = NaiveDate::from_ymd_opt(1, 1, 1).and_then(|date| date.and_hms_opt(0, 0, 0));
        assert_eq!(
            timestamp::format(first.expect("valid")),
            "0001-01-01T00:00:00"
        );
    }

    #[test]
    fn formatted_timestamps_parse_back() {
        for micros in [0, 1, 10, 100_000, 123_456, 999_999] {
            let original = at_micros(23, 59, 59, micros);

            assert_eq!(timestamp::parse(&timestamp::format(original)), Ok(original));
        }
    }

    #[test]
    fn fractions_are_padded_or_truncated_to_microseconds() {
        for (text, micros) in [
            ("2026-03-01T10:20:30.5", 500_000),
            ("2026-03-01T10:20:30.05", 50_000),
            ("2026-03-01T10:20:30.000001", 1),
            ("2026-03-01T10:20:30.0000019", 1),
            ("2026-03-01T10:20:30.1234567890123", 123_456),
        ] {
            assert_eq!(
                timestamp::parse(text),
                Ok(at_micros(10, 20, 30, micros)),
                "{text}"
            );
        }
    }

    #[test]
    fn malformed_timestamps_are_rejected() {
        for text in [
            "",
            "2026",
            "2026-03-01T",
            "2026-03-01T10",
            "2026-03-01T10:20:3",
            "2026-03-01T10:20:30.",
            "2026-03-01T10:20:30Z ",
            "2026-03-01T10:20:30+1",
            "2026-02-29T10:20:30",
            "2026-03-01T10:20:30\u{b2}",
            "２０２６-03-01T10:20:30",
            "2026-03-01T10:20:30+٠٢:٠٠",
        ] {
            assert!(timestamp::parse(text).is_err(), "{text:?} must not parse");
        }
    }

    #[test]
    fn an_offset_is_converted_to_local_time() {
        let utc = at(10, 20, 30);
        let local = |utc: NaiveDateTime| Local.from_utc_datetime(&utc).naive_local();

        assert_eq!(timestamp::parse("2026-03-01T10:20:30Z"), Ok(local(utc)));
        assert_eq!(
            timestamp::parse("2026-03-01T12:20:30+02:00"),
            Ok(local(utc))
        );
        assert_eq!(
            timestamp::parse("2026-03-01T04:50:30-05:30"),
            Ok(local(utc))
        );
    }

    #[test]
    fn years_python_cannot_hold_are_not_written() {
        let mut state = fresh();
        state.start_time = NaiveDate::from_ymd_opt(0, 12, 31)
            .and_then(|date| date.and_hms_opt(0, 0, 0))
            .expect("chrono has a year 0");

        let error = render_state(&state).expect_err("year 0 is out of range");

        assert!(error.to_string().contains("1..=9999"), "{error}");
    }

    // ----- locations -----

    fn os(text: &str) -> Option<&OsStr> {
        Some(OsStr::new(text))
    }

    #[test]
    fn state_dir_prefers_the_explicit_directory_then_xdg_then_home() {
        let home = Path::new("/home/jana");

        assert_eq!(
            state_dir(os("/data/sg"), os("/xdg"), home),
            Path::new("/data/sg")
        );
        assert_eq!(
            state_dir(None, os("/xdg"), home),
            Path::new("/xdg/shellgame")
        );
        assert_eq!(
            state_dir(None, None, home),
            Path::new("/home/jana/.config/shellgame")
        );
    }

    #[test]
    fn state_dir_treats_empty_values_as_unset() {
        let home = Path::new("/home/jana");

        assert_eq!(
            state_dir(os(""), os("/xdg"), home),
            Path::new("/xdg/shellgame")
        );
        assert_eq!(
            state_dir(os(""), os(""), home),
            Path::new("/home/jana/.config/shellgame")
        );
        assert_eq!(
            state_dir(None, os(""), home),
            Path::new("/home/jana/.config/shellgame")
        );
    }

    #[test]
    fn state_dir_is_normalized_like_a_python_path() {
        let home = Path::new("/home/jana/");

        assert_eq!(
            state_dir(os("/data//sg/"), None, home),
            Path::new("/data/sg")
        );
        assert_eq!(
            state_dir(None, os("/xdg/./"), home),
            Path::new("/xdg/shellgame")
        );
        assert_eq!(
            state_dir(None, None, home),
            Path::new("/home/jana/.config/shellgame")
        );
        // A relative directory stays relative to wherever the game runs.
        assert_eq!(state_dir(os("./state"), None, home), Path::new("state"));
    }

    #[test]
    fn the_default_workspace_is_in_tmp_unless_the_environment_says_otherwise() {
        assert_eq!(
            default_workspace(None, "jana"),
            Path::new("/tmp/shellgame-jana")
        );
        assert_eq!(
            default_workspace(os(""), "jana"),
            Path::new("/tmp/shellgame-jana")
        );
        assert_eq!(
            default_workspace(os("/work/sg"), "jana"),
            Path::new("/work/sg")
        );
        assert_eq!(
            default_workspace(os("/work//sg/"), "jana"),
            Path::new("/work/sg")
        );
        assert_eq!(
            default_workspace(None, "Jůlie Novák"),
            Path::new("/tmp/shellgame-Jůlie Novák")
        );
    }

    #[test]
    fn paths_are_normalized_like_pathlib() {
        // The expected values are what `str(pathlib.Path(text))` returns.
        for (text, expected) in [
            ("/tmp/ws", "/tmp/ws"),
            ("/tmp/ws/", "/tmp/ws"),
            ("/tmp//ws", "/tmp/ws"),
            ("/tmp/./ws", "/tmp/ws"),
            ("/tmp/shell game/", "/tmp/shell game"),
            ("./ws", "ws"),
            ("././a", "a"),
            ("ws/", "ws"),
            ("a//b/./c/", "a/b/c"),
            ("a/../b", "a/../b"),
            ("/..", "/.."),
            (".", "."),
            ("./.", "."),
            ("", "."),
            ("/", "/"),
            ("/./", "/"),
            ("/a/.", "/a"),
            ("//", "//"),
            ("///", "/"),
            ("////a", "/a"),
            ("//a//b/", "//a/b"),
        ] {
            assert_eq!(
                normalize_path(Path::new(text)),
                Path::new(expected),
                "{text:?}"
            );
        }
    }

    #[test]
    fn normalizing_keeps_bytes_that_are_not_utf8() {
        let path = Path::new(OsStr::from_bytes(b"/tmp//ws\xff/"));

        assert_eq!(normalize_path(path).as_os_str().as_bytes(), b"/tmp/ws\xff");
    }

    // ----- store and errors -----

    #[test]
    fn a_store_keeps_its_state_file_in_its_directory() {
        let store = StateStore::new("/data/shellgame");

        assert_eq!(store.dir(), Path::new("/data/shellgame"));
        assert_eq!(store.path(), Path::new("/data/shellgame/state.json"));
        assert!(!StateStore::new("/definitely/not/a/directory").exists());
    }

    #[test]
    fn errors_are_reported_in_czech_with_the_file_and_the_detail() {
        let file = PathBuf::from("/data/state.json");
        let load = StateError::Load {
            file: file.clone(),
            detail: "boom".to_owned(),
        };
        let save = StateError::Save {
            file: file.clone(),
            detail: "boom".to_owned(),
        };
        let remove = StateError::Remove {
            file: file.clone(),
            detail: "boom".to_owned(),
        };

        assert_eq!(
            load.to_string(),
            "Stav ShellGame nelze načíst z /data/state.json: boom"
        );
        assert_eq!(
            save.to_string(),
            "Stav ShellGame nelze uložit do /data/state.json: boom"
        );
        assert_eq!(
            remove.to_string(),
            "Stav ShellGame nelze odstranit (/data/state.json): boom"
        );
        for error in [&load, &save, &remove] {
            assert_eq!(error.file(), file);
            assert_eq!(error.detail(), "boom");
        }
    }
}
