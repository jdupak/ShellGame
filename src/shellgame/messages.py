"""Centralized message strings for ShellGame.

All user-facing Czech messages should be defined here for:
- Consistency across the application
- Easy localization if needed in the future
- Single source of truth for error messages
"""

from typing import Optional


class Messages:
    """Centralized Czech message strings."""

    # === Answer validation ===
    ANSWER_REQUIRED = "Musíte zadat odpověď: shellgame submit <odpověď>"
    ANSWER_REQUIRED_NAME = "Musíte zadat název."
    ANSWER_REQUIRED_NUMBER = "Musíte zadat číslo."
    ANSWER_REQUIRED_LIST = "Musíte zadat seznam."
    ANSWER_REQUIRED_FILE = "Musíte zadat název souboru."

    # === Directory validation ===
    WRONG_DIRECTORY = "Jste v '{actual}', ale měli byste být v '{expected}'."
    NOT_IN_DIRECTORY = (
        "Nejdřív musíte být v adresáři '{expected}'. Použijte: cd {expected}"
    )

    # === File/Directory existence ===
    FILE_EXISTS = "Soubor '{path}' existuje!"
    FILE_NOT_EXISTS = "Soubor '{path}' neexistuje"
    FILE_STILL_EXISTS = "Soubor '{path}' stále existuje"
    DIR_EXISTS = "Adresář '{path}' existuje!"
    DIR_NOT_EXISTS = "Adresář '{path}' neexistuje"
    DIR_STILL_EXISTS = "Adresář '{path}' stále existuje"

    # === File operations ===
    FILE_CONTENT_CORRECT = "Obsah souboru je správný!"
    FILE_CONTENT_MISMATCH = "Obsah souboru neodpovídá očekávání"
    FILE_READ_ERROR = "Nelze přečíst soubor: {error}"
    FILE_TYPE_ERROR = "Příkaz 'file' není dostupný."
    FILE_TYPE_MISMATCH = "Tento soubor není '{expected}', je to '{actual}'."

    # === Copy/Move validation ===
    COPY_SUCCESS = "Správně zkopírováno!"
    COPY_SOURCE_MISSING = (
        "Zdrojový soubor '{path}' chybí (možná jste ho přesunuli místo zkopírování?)."
    )
    COPY_DEST_MISSING = "Cílový soubor '{path}' neexistuje."
    COPY_CONTENT_MISMATCH = "Obsah zkopírovaného souboru neodpovídá originálu."
    MOVE_SUCCESS = "Správně přesunuto/přejmenováno!"
    MOVE_SOURCE_EXISTS = "Původní soubor '{path}' stále existuje (možná jste ho zkopírovali místo přesunutí?)."
    MOVE_DEST_MISSING = "Cílový soubor '{path}' neexistuje."

    # === Permissions ===
    PERMISSION_CORRECT = "Správně nastavená oprávnění!"
    PERMISSION_MISMATCH = "Očekáváno {expected}, nastaveno {actual}."
    EXECUTABLE_SUCCESS = "Soubor je spustitelný!"
    EXECUTABLE_FAIL = "Soubor není spustitelný."

    # === Marker validation ===
    MARKER_NOT_FOUND = "Zatím to nevypadá, že jste použili požadovaný příkaz."
    UNKNOWN_USER = "Chyba: Neznámý uživatel."

    # === Generic success/failure ===
    CORRECT = "Správně!"
    INCORRECT = "Nesprávně."
    ALL_CHECKS_PASSED = "Všechny kontroly proběhly úspěšně!"
    EXPECTED_GOT = "Očekáváno '{expected}', obdrženo '{actual}'"
    EXPECTED_GOT_INT = "Očekáváno {expected}, obdrženo {actual}"
    EXPECTED_INTEGER = "Očekáváno celé číslo"

    # === Level-specific messages ===
    LEVEL_COMPLETED = "Level dokončen."
    GAME_COMPLETED = "🎉 Gratulujeme! Dokončili jste všechny levely!"
    SECTION_COMPLETED = "🎉 Výborně! Dokončili jste sekci."

    # === Navigation messages ===
    TELEPORT = "Teleport"
    TELEPORT_NOTICE = "Pozor, byli jste teleportováni z {src} na {dest}"
    TELEPORT_OUTSIDE_WORKSPACE = "Teleport (mimo workspace)"

    # === Init/Status messages ===
    NOT_INITIALIZED = "ShellGame není inicializován. Spusťte 'shellgame init'."
    ALREADY_INITIALIZED = "ShellGame je již inicializován."
    WORKSPACE_RESTORED = "Pracovní prostor obnoven: {path}"
    WORKSPACE_DELETED_WARNING = (
        "⚠ Pracovní prostor byl smazán (např. restart systému). Obnovuji..."
    )
    LEVEL_NOT_FOUND = "Chyba: Level {level_id} nenalezen"

    # === Hint messages ===
    NO_MORE_HINTS = "Žádné další nápovědy nejsou k dispozici."

    # === Level 1 specific ===
    L1_1_USE_PWD_FIRST = "Nejdřív prosím použijte `pwd` (ShellGame sleduje použití příkazu) a pak odpověď odevzdejte znovu."
    L1_1_SUBMIT_WITH_NAME = (
        "V levelu 1.1 musíte zadat název aktuálního adresáře: shellgame submit level-1"
    )
    L1_2_NOT_A_DIRECTORY = "'{answer}' je soubor, ne adresář. Hledejte adresář začínající na 'd' a končící na 'a'."
    L1_3_INCLUDED_EXTENSION = "Zahrnuli jste příponu '.txt'. Úkolem je odevzdat název BEZ přípony. Zkuste jen 'inside'."
    L1_8_USE_ABSOLUTE_PATH = "Použijte absolutní cestu (začíná na /)."
    L1_8_RELATIVE_WOULD_WORK = "Tohle by fungovalo, ale je to relativní cesta. Zkuste to ještě jednou absolutně (začíná na /)."
    L1_8_NAME_CORRECT_NOT_THERE = "Jméno máte správně, ale nejste v cílovém adresáři. Nejdřív se tam přesuňte pomocí `cd /...` a pak použijte `shellgame submit`."
    L1_8_NOT_ABSOLUTE = "Jste ve správném adresáři, ale zatím to nevypadá, že jste použili absolutní `cd` (začíná na /). Zkuste to znovu: opusťte adresář a vraťte se absolutní cestou."
    ABSOLUTE_CD_NOT_USED = L1_8_NOT_ABSOLUTE
    L1_9_SEQUENCE_NOT_COMPLETE = "Vypadá to, že jste se domů dostali, ale ShellGame nezaznamenal povinnou sekvenci `cd` krok za krokem. Začněte znovu: `cd /` a projděte každý segment cesty do $HOME po jednom."
    CD_WALK_NOT_COMPLETED = L1_9_SEQUENCE_NOT_COMPLETE

    @classmethod
    def format(cls, message: str, **kwargs: object) -> str:
        """Format a message with the given keyword arguments.

        Args:
            message: Message string with {placeholders}
            **kwargs: Values to substitute

        Returns:
            Formatted message string
        """
        return message.format(**kwargs)

    @classmethod
    def wrong_directory(cls, actual: str, expected: str) -> str:
        """Format wrong directory message."""
        return cls.WRONG_DIRECTORY.format(actual=actual, expected=expected)

    @classmethod
    def expected_got(cls, expected: str, actual: str) -> str:
        """Format expected/got message."""
        return cls.EXPECTED_GOT.format(expected=expected, actual=actual)

    @classmethod
    def file_exists(cls, path: str, exists: bool = True) -> str:
        """Get appropriate file existence message."""
        if exists:
            return cls.FILE_EXISTS.format(path=path)
        return cls.FILE_NOT_EXISTS.format(path=path)

    @classmethod
    def dir_exists(cls, path: str, exists: bool = True) -> str:
        """Get appropriate directory existence message."""
        if exists:
            return cls.DIR_EXISTS.format(path=path)
        return cls.DIR_NOT_EXISTS.format(path=path)
