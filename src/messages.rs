//! Player-facing messages shared by the engine, the session and the UI.
//!
//! The texts are Czech and exact: levels, tests and the Python version all quote them.
//! A few carry a placeholder (`{path}`, `{actual}`, `{error}`); fill them with
//! [`with_path`], [`not_at_home`] and [`with_error`] instead of formatting by hand, so
//! the placeholder names stay in one place.

use std::fmt;

/// `shellgame submit` without an answer, for a level that asks for one.
pub const ANSWER_REQUIRED: &str = "Musíte zadat odpověď: shellgame submit <odpověď>";

/// Same, when the expected answer is a number.
pub const ANSWER_REQUIRED_NUMBER: &str = "Musíte zadat číslo.";

/// Same, when the expected answer is a list. No level uses it today; kept for parity.
pub const ANSWER_REQUIRED_LIST: &str = "Musíte zadat seznam.";

/// `at(..)` failed and the level supplied no message of its own.
pub const WRONG_DIRECTORY: &str =
    "Nejste v cílovém adresáři. Dokončete přesun podle zadání a ověřte svou polohu příkazem 'pwd'.";

/// `at_home()` failed. `{actual}` is the directory the player is in.
pub const NOT_AT_HOME: &str = "Nejste doma. Jste v: {actual}";

/// Success text for a file that exists. `{path}` is the declared path.
pub const FILE_EXISTS: &str = "Soubor '{path}' existuje!";

/// A required file is missing. `{path}` is the declared path.
pub const FILE_NOT_EXISTS: &str = "Soubor '{path}' neexistuje";

/// A file that should be gone is still there. `{path}` is the declared path.
pub const FILE_STILL_EXISTS: &str = "Soubor '{path}' stále existuje";

/// Success text for a directory that exists. `{path}` is the declared path.
pub const DIR_EXISTS: &str = "Adresář '{path}' existuje!";

/// A required directory is missing. `{path}` is the declared path.
pub const DIR_NOT_EXISTS: &str = "Adresář '{path}' neexistuje";

/// A directory that should be gone is still there. `{path}` is the declared path.
pub const DIR_STILL_EXISTS: &str = "Adresář '{path}' stále existuje";

/// Success text for a file whose content matches.
pub const FILE_CONTENT_CORRECT: &str = "Obsah souboru je správný!";

/// Default text when file content does not match the expectation.
pub const FILE_CONTENT_MISMATCH: &str = "Obsah souboru neodpovídá očekávání";

/// A file could not be read. `{error}` is the underlying reason.
pub const FILE_READ_ERROR: &str =
    "Nelze přečíst soubor: {error}. Obnovte level příkazem `shellgame reset`.";

/// Generic filesystem failure. `{error}` is the underlying reason.
pub const FILESYSTEM_ERROR: &str = "Chyba při práci se soubory: {error}";

/// Success text for a correct copy.
pub const COPY_SUCCESS: &str = "Správně zkopírováno!";

/// The source of a copy is gone, perhaps moved instead of copied. `{path}` is the declared path.
pub const COPY_SOURCE_MISSING: &str =
    "Zdrojový soubor '{path}' chybí (možná jste ho přesunuli místo zkopírování?).";

/// The destination of a copy does not exist. `{path}` is the declared path.
pub const COPY_DEST_MISSING: &str = "Cílový soubor '{path}' neexistuje.";

/// Default text when the copy differs from the original.
pub const COPY_CONTENT_MISMATCH: &str = "Obsah zkopírovaného souboru neodpovídá originálu.";

/// Success text for a correct move or rename.
pub const MOVE_SUCCESS: &str = "Správně přesunuto/přejmenováno!";

/// The source of a move still exists, perhaps copied instead of moved. `{path}` is the declared path.
pub const MOVE_SOURCE_EXISTS: &str =
    "Původní soubor '{path}' stále existuje (možná jste ho zkopírovali místo přesunutí?).";

/// The destination of a move does not exist. `{path}` is the declared path.
pub const MOVE_DEST_MISSING: &str = "Cílový soubor '{path}' neexistuje.";

/// Success text for correctly set permissions.
pub const PERMISSION_CORRECT: &str = "Správně nastavená oprávnění!";

/// Default text when permissions are not as required.
pub const PERMISSION_WRONG: &str = "Oprávnění nejsou nastavena správně.";

/// Default text when an evidence marker is missing.
pub const MARKER_NOT_FOUND: &str = "Zatím to nevypadá, že jste použili požadovaný příkaz.";

/// Printed under every blocked `cd`, so a player can always recover.
pub const CD_RECOVERY_TIP: &str =
    "Ztratili jste se? Příkaz `shellgame reset` vás vrátí na start levelu.";

/// The player's working directory no longer exists (usually deleted).
pub const CWD_MISSING: &str = "Aktuální adresář už neexistuje (nejspíš jste ho smazali). Příkaz `shellgame reset` vás vrátí na start levelu.";

/// A declared path failed containment (symlinked component, escape).
pub const PATH_ESCAPES_WORKSPACE: &str = "Cesta úkolu opouští pracovní prostor.";

/// Default success message of a level.
pub const CORRECT: &str = "Správně!";

/// Default wrong-answer text. Terse on purpose, and never contains the answer.
pub const INCORRECT: &str = "Nesprávně.";

/// The answer does not have the expected number of parts.
pub const ANSWER_FORMAT: &str = "Odpověď nemá očekávaný formát.";

/// Comparing two trees failed. `{error}` is the underlying reason.
pub const COMPARE_FAILED: &str = "Chyba při kontrole souborů: {error}";

/// The answer is not an integer.
pub const EXPECTED_INTEGER: &str = "Očekáváno celé číslo";

/// Level 1.1: the answer is right, but `pwd` was never used.
pub const L1_1_USE_PWD_FIRST: &str = "Nejdřív prosím použijte `pwd` (ShellGame sleduje použití příkazu) a pak odpověď odevzdejte znovu.";

/// Level 1.3: the answer still has the file extension.
pub const L1_3_INCLUDED_EXTENSION: &str =
    "Odevzdejte název souboru bez přípony: vynechte poslední tečku a část za ní.";

/// Level 1.8: the player is in the right place but never used an absolute `cd`.
pub const ABSOLUTE_CD_NOT_USED: &str = "Jste ve správném adresáři, ale zatím to nevypadá, že jste použili absolutní `cd` (začíná na /). Zkuste to znovu: opusťte adresář a vraťte se absolutní cestou.";

/// Level 1.9: the player is home but did not walk there segment by segment.
pub const CD_WALK_NOT_COMPLETED: &str = "Vypadá to, že jste se domů dostali, ale ShellGame nezaznamenal povinnou sekvenci `cd` krok za krokem. Začněte znovu: `cd /` a projděte každý segment cesty do $HOME po jednom.";

/// The single hint of every section intro.
pub const INTRO_HINT: &str = "Přečtěte si úvod a pokračujte stisknutím Enter.";

/// Fill the `{path}` placeholder of a message template.
pub fn with_path(template: &str, path: &str) -> String {
    template.replace("{path}", path)
}

/// Fill the `{error}` placeholder of a message template.
pub fn with_error(template: &str, error: impl fmt::Display) -> String {
    template.replace("{error}", &error.to_string())
}

/// [`NOT_AT_HOME`] with the directory the player is actually in.
pub fn not_at_home(actual: impl fmt::Display) -> String {
    NOT_AT_HOME.replace("{actual}", &actual.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_are_filled() {
        assert_eq!(
            with_path(FILE_NOT_EXISTS, "a/b.txt"),
            "Soubor 'a/b.txt' neexistuje"
        );
        assert_eq!(
            with_path(DIR_STILL_EXISTS, "tmp"),
            "Adresář 'tmp' stále existuje"
        );
        assert_eq!(not_at_home("/tmp/x"), "Nejste doma. Jste v: /tmp/x");
        assert_eq!(
            with_error(COMPARE_FAILED, "boom"),
            "Chyba při kontrole souborů: boom"
        );
        assert!(
            with_error(FILE_READ_ERROR, "denied").contains("Nelze přečíst soubor: denied."),
            "the error is substituted in place"
        );
    }

    #[test]
    fn templates_without_placeholders_are_left_alone() {
        assert_eq!(with_path(CORRECT, "x"), CORRECT);
        assert_eq!(with_error(INCORRECT, "x"), INCORRECT);
    }

    #[test]
    fn well_known_texts_are_exact() {
        assert_eq!(INCORRECT, "Nesprávně.");
        assert_eq!(CORRECT, "Správně!");
        assert_eq!(
            ANSWER_REQUIRED,
            "Musíte zadat odpověď: shellgame submit <odpověď>"
        );
        assert_eq!(ANSWER_REQUIRED_NUMBER, "Musíte zadat číslo.");
        assert_eq!(EXPECTED_INTEGER, "Očekáváno celé číslo");
        assert_eq!(ANSWER_FORMAT, "Odpověď nemá očekávaný formát.");
        assert_eq!(
            INTRO_HINT,
            "Přečtěte si úvod a pokračujte stisknutím Enter."
        );
        assert_eq!(
            CD_RECOVERY_TIP,
            "Ztratili jste se? Příkaz `shellgame reset` vás vrátí na start levelu."
        );
        assert_eq!(
            CWD_MISSING,
            "Aktuální adresář už neexistuje (nejspíš jste ho smazali). \
             Příkaz `shellgame reset` vás vrátí na start levelu."
        );
    }

    #[test]
    fn default_wrong_answer_text_is_terse() {
        // It must never leak anything about the expected answer.
        assert!(!INCORRECT.contains('{'));
        assert!(INCORRECT.chars().count() < 20);
    }
}
