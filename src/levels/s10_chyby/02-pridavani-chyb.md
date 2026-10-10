Stejně jako u normálního výstupu se i u chyb rozhodujete mezi dvěma operátory:
jeden soubor před zápisem vyprázdní, druhý zapisuje na jeho konec.

### Úkol
V souboru `errors.log` už je z minula záznam `Old error 1`.
Spusťte `buggy.sh` znovu tak, aby v souboru zůstal starý záznam i nová chyba.

Než příkaz spustíte, rozmyslete si: který z obou operátorů soubor nejdřív
vyprázdní a co by se v takovém případě stalo s řádkem `Old error 1`?

### Příkazy
- `./script 2> soubor` - chybový výstup přepíše obsah souboru
- `./script 2>> soubor` - chybový výstup přidá na konec souboru

### Odevzdání
Odevzdejte název souboru.
`shellgame submit <název>`