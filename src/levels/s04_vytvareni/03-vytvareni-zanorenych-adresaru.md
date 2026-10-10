### Cíl
Vytvořte zanořenou strukturu adresářů jedním příkazem.

### Vytváření cest s přepínačem `-p`
Při vytváření celé cesty (např. `projekt/src/tests`) příkaz `mkdir` bez přepínače
selže, pokud nadřazené adresáře neexistují.
Přepínač `-p` (parents) vytvoří celou cestu včetně chybějících rodičovských adresářů.

### Příkazy
- `mkdir -p <cesta>` (vytvoří adresář včetně všech chybějících nadřazených složek)

### Úkol
Vytvořte strukturu adresářů `projekt/src/tests`.

### Odevzdání
Až bude celá struktura vytvořená, spusťte:
`shellgame submit`