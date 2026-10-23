//! Czech texts shared by checks, answers and the runtime. A `{}` is filled with [`fill`].

pub const ANSWER_REQUIRED: &str = "Musíte zadat odpověď: shellgame submit <odpověď>";
pub const ANSWER_FORMAT: &str = "Odpověď nemá očekávaný formát.";
pub const CORRECT: &str = "Správně!";
pub const INCORRECT: &str = "Nesprávně.";

pub const WRONG_DIRECTORY: &str =
    "Nejste v cílovém adresáři. Dokončete přesun podle zadání a ověřte svou polohu příkazem 'pwd'.";
pub const NOT_AT_HOME: &str = "Nejste doma. Jste v: {}";
pub const FILE_NOT_EXISTS: &str = "Soubor '{}' neexistuje";
pub const FILE_STILL_EXISTS: &str = "Soubor '{}' stále existuje";
pub const DIR_NOT_EXISTS: &str = "Adresář '{}' neexistuje";
pub const FILE_CONTENT_MISMATCH: &str = "Obsah souboru neodpovídá očekávání";
pub const FILE_READ_ERROR: &str =
    "Nelze přečíst soubor: {}. Obnovte level příkazem 'shellgame reset'.";
pub const FILESYSTEM_ERROR: &str = "Chyba při práci se soubory: {}";
pub const COPY_SOURCE_MISSING: &str =
    "Zdrojový soubor '{}' chybí (možná jste ho přesunuli místo zkopírování?).";
pub const DEST_MISSING: &str = "Cílový soubor '{}' neexistuje.";
pub const COPY_CONTENT_MISMATCH: &str = "Obsah zkopírovaného souboru neodpovídá originálu.";
pub const MOVE_SOURCE_EXISTS: &str =
    "Původní soubor '{}' stále existuje (možná jste ho zkopírovali místo přesunutí?).";
pub const COMPARE_FAILED: &str = "Chyba při kontrole souborů: {}";
pub const PERMISSION_WRONG: &str = "Oprávnění nejsou nastavena správně.";
pub const CWD_MISSING: &str = "Aktuální adresář už neexistuje (nejspíš jste ho smazali). Příkaz 'shellgame reset' vás vrátí na start levelu.";
pub const PATH_ESCAPES_WORKSPACE: &str = "Cesta úkolu vede přes symbolický odkaz mimo pracovní prostor. Level obnovíte příkazem 'shellgame reset'.";
/// Printed under every blocked `cd`, so a player can always recover.
pub const CD_RECOVERY_TIP: &str =
    "Ztratili jste se? Příkaz 'shellgame reset' vás vrátí na start levelu.";

/// Put `value` where the template has `{}`.
pub fn fill(template: &str, value: impl std::fmt::Display) -> String {
    template.replacen("{}", &value.to_string(), 1)
}

/// The text a level wrote, or the default when it wrote none (an empty text).
pub fn or(custom: &str, default: impl Into<String>) -> String {
    if custom.is_empty() { default.into() } else { custom.to_owned() }
}
