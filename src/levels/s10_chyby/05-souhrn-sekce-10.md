### Výzva: Oddělení streamů

Ukažte, že umíte uložit stdout a stderr odděleně.

### Úkol
V `level-10/challenge` je skript `mixed.sh` který vypisuje:
- normální výstup na stdout
- chyby na stderr

Spusťte skript **jednou** a současně uložte:
1. pouze normální výstup do `output.log`
2. pouze chyby do `errors.log`

Odpovězte: kolik řádků má errors.log a kolik output.log?
Formát: `chyby,výstup` (např. `3,5`)

### Připomenutí
- `>` přesměruje standardní výstup.
- `2>` přesměruje chybový výstup.

V této výzvě potřebujete oba proudy zachytit zvlášť.

### Odevzdání
`shellgame submit <chyby>,<výstup>`