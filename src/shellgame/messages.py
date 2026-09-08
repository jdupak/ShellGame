"""Shared validation messages."""


class Messages:
    ANSWER_REQUIRED = "Musíte zadat odpověď: shellgame submit <odpověď>"
    ANSWER_REQUIRED_NUMBER = "Musíte zadat číslo."
    ANSWER_REQUIRED_LIST = "Musíte zadat seznam."

    WRONG_DIRECTORY = "Nejste v cílovém adresáři. Dokončete přesun podle zadání a ověřte svou polohu příkazem 'pwd'."
    NOT_AT_HOME = "Nejste doma. Jste v: {actual}"

    FILE_EXISTS = "Soubor '{path}' existuje!"
    FILE_NOT_EXISTS = "Soubor '{path}' neexistuje"
    FILE_STILL_EXISTS = "Soubor '{path}' stále existuje"
    DIR_EXISTS = "Adresář '{path}' existuje!"
    DIR_NOT_EXISTS = "Adresář '{path}' neexistuje"
    DIR_STILL_EXISTS = "Adresář '{path}' stále existuje"

    FILE_CONTENT_CORRECT = "Obsah souboru je správný!"
    FILE_CONTENT_MISMATCH = "Obsah souboru neodpovídá očekávání"
    FILE_READ_ERROR = "Nelze přečíst soubor: {error}. Obnovte level příkazem `shellgame reset`."
    FILESYSTEM_ERROR = "Chyba při práci se soubory: {error}"

    COPY_SUCCESS = "Správně zkopírováno!"
    COPY_SOURCE_MISSING = "Zdrojový soubor '{path}' chybí (možná jste ho přesunuli místo zkopírování?)."
    COPY_DEST_MISSING = "Cílový soubor '{path}' neexistuje."
    COPY_CONTENT_MISMATCH = "Obsah zkopírovaného souboru neodpovídá originálu."
    MOVE_SUCCESS = "Správně přesunuto/přejmenováno!"
    MOVE_SOURCE_EXISTS = "Původní soubor '{path}' stále existuje (možná jste ho zkopírovali místo přesunutí?)."
    MOVE_DEST_MISSING = "Cílový soubor '{path}' neexistuje."

    PERMISSION_CORRECT = "Správně nastavená oprávnění!"
    PERMISSION_WRONG = "Oprávnění nejsou nastavena správně."

    MARKER_NOT_FOUND = "Zatím to nevypadá, že jste použili požadovaný příkaz."

    CD_RECOVERY_TIP = "Ztratili jste se? Příkaz `shellgame reset` vás vrátí na start levelu."
    CWD_MISSING = (
        "Aktuální adresář už neexistuje (nejspíš jste ho smazali). Příkaz `shellgame reset` vás vrátí na start levelu."
    )
    PATH_ESCAPES_WORKSPACE = "Cesta úkolu opouští pracovní prostor."

    CORRECT = "Správně!"
    INCORRECT = "Nesprávně."
    ANSWER_FORMAT = "Odpověď nemá očekávaný formát."
    COMPARE_FAILED = "Chyba při kontrole souborů: {error}"
    EXPECTED_INTEGER = "Očekáváno celé číslo"

    L1_1_USE_PWD_FIRST = (
        "Nejdřív prosím použijte `pwd` (ShellGame sleduje použití příkazu) a pak odpověď odevzdejte znovu."
    )
    L1_3_INCLUDED_EXTENSION = "Odevzdejte název souboru bez přípony: vynechte poslední tečku a část za ní."
    ABSOLUTE_CD_NOT_USED = (
        "Jste ve správném adresáři, ale zatím to nevypadá, že jste použili absolutní `cd` (začíná na /). Zkuste to "
        "znovu: opusťte adresář a vraťte se absolutní cestou."
    )
    CD_WALK_NOT_COMPLETED = (
        "Vypadá to, že jste se domů dostali, ale ShellGame nezaznamenal povinnou sekvenci `cd` krok za krokem. "
        "Začněte znovu: `cd /` a projděte každý segment cesty do $HOME po jednom."
    )
