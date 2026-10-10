### Výzva: Detektiv souborů

Ukažte, že umíte identifikovat typy souborů!

### Úkol
V `level-5/mystery` je 5 souborů s podivnými názvy.
Zjistěte typ každého a odpovězte na otázky:

1. Kolik je tam **prostých textových** souborů (popis začíná `ASCII text`, bez skriptů)?
2. Kolik je tam **obrázků** (image)?
3. Jaký je název jediného **Python** skriptu (bez cesty)?

### Formát odpovědi
`<text_count>,<image_count>,<script_name>`

Skript počítejte samostatně, i když jeho popis také obsahuje `ASCII text`.

### Shrnutí příkazů Sekce 5
```
file soubor     → Zjistí typ souboru
file *          → Typy všech souborů
file -b soubor  → Jen typ bez názvu
```

### Odevzdání
`shellgame submit <text>,<img>,<script>`