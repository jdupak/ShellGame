### Cíl
Smažte nepotřebný soubor.

### Příkazy
- `rm <soubor>` (remove — smaže zadaný soubor)

### ⚠️ Důležité varování
Příkaz `rm` nepřesouvá soubory do koše jako grafické prostředí — maže je okamžitě a nevratně.
- `rm -i <soubor>` se před smazáním zeptá na potvrzení.
- Před hromadným mazáním (např. `rm *.log`) si obsah zkontrolujte pomocí `ls *.log`.

### Úkol
Smažte soubor `stary_log.txt`, který se nachází v aktuálním adresáři.

### Odevzdání
Odevzdejte název smazaného souboru:
`shellgame submit <název>`