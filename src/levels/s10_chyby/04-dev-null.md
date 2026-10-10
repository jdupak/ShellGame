`/dev/null` je speciální soubor, který zahodí všechno, co do něj pošlete.
Je užitečný pro umlčení hlučných příkazů.

### Proč je to užitečné
Některé příkazy vypisují spoustu informací, které nepotřebujete.
Místo zahlcení obrazovky je můžete "poslat do černé díry".

### Úkol
Spusťte `buggy.sh` a umlčte VŠECHNY výstupy (stdout i stderr) přesměrováním do `/dev/null`.

### Příkazy
- `./script &> /dev/null` - zahodí veškerý výstup

### Odevzdání
Odevzdejte název speciálního souboru, který jste použili.
`shellgame submit <název>`