Standardní chybový výstup (stderr) používá deskriptor souboru 2.
Pro přesměrování pouze chyb použijte `2>`.

### Proč je to důležité
Při běhu programů často chcete zachytit chybové hlášky do logu,
zatímco normální výstup zobrazíte uživateli. Oddělení stdout a stderr
je klíčové pro diagnostiku problémů.

### Úkol
V adresáři je skript `buggy.sh`, který vypisuje normální text i chybové zprávy.
Spusťte ho a přesměrujte POUZE chybové zprávy do souboru `errors.log`.

### Příkazy
- `./script 2> soubor` - přesměruje stderr do souboru

### Odevzdání
Odevzdejte název vytvořeného souboru.
`shellgame submit <název>`