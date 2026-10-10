### Cíl
Rozpoznejte obrázek mezi soubory bez přípony podle jejich obsahu.

V Linuxu přípona souboru (např. `.txt`, `.jpg`) neurčuje jeho typ. O tom rozhoduje obsah.
Příkaz `file` prozkoumá obsah souboru a řekne vám, o jaký typ se jedná.

### Příkazy
- `file <soubor>` (zjistí typ konkrétního souboru)
- `file *` (zjistí typ všech souborů v aktuálním adresáři)

### Úkol
V adresáři jsou tři soubory bez přípony: `file1`, `file2`, `file3`.
Jeden z nich je obrázek (`JPEG image data`). Zjistěte který a odevzdejte jeho název.

### Odevzdání
`shellgame submit <soubor>`