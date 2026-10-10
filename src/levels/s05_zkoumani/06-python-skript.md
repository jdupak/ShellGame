### Cíl
Rozpoznejte Python skript mezi různými typy souborů.

Příkaz `file` dokáže podle obsahu spolehlivě rozpoznat Python skript i bez ohledu na jeho název.
Popis `Python script` ale neříká, zda má soubor právo ke spuštění (`x`) — typ zjišťuje `file`,
kdežto oprávnění zobrazuje `ls -l` a mění `chmod` (tomu se věnuje Sekce 8).

### Příkazy
- `file *` (vypíše typy všech souborů v aktuálním adresáři)

### Úkol
Nacházíte se přímo v adresáři `bin` (nikam nemusíte přecházet):
1. Prozkoumejte soubory v aktuálním adresáři pomocí `file *`.
2. Najděte soubor, který příkaz `file` označí jako `Python script`.
3. Odevzdejte název tohoto souboru.

### Odevzdání
`shellgame submit <soubor>`