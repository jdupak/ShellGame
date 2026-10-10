### Cíl
Najděte všechny běžné `.py` soubory v `src_code` a uložte jejich cesty do `python_files.txt`.
Výstup seřaďte, aby měl vždy stejné pořadí.

**Důležité:** Vzor musí být v uvozovkách: `"*.py"`.
Bez uvozovek by se ho nejprve pokusil rozbalit shell. S uvozovkami dostane vzor doslova `find`.

### Stavební prvky
- `find adresář -type f -name "*.py"` - hledá pouze soubory s příponou `.py`
- `sort` - seřadí řádky
- `>` - uloží výsledek

### Úkol
Vytvořte `python_files.txt` obsahující přesně seřazené cesty nalezených souborů.

### Odevzdání
`shellgame submit <název-souboru>`