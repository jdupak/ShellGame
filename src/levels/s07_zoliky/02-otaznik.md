# Otazník ?

Otazník `?` nahradí PRÁVĚ JEDEN znak.
Je užitečný, když chcete být přesnější než s hvězdičkou.

### Požadavek: Bash
Tento level vyžaduje **Bash**. Ve fish použijte variantu s `bash -c` níže.
Ta spustí pouze kopírování v Bashi; odevzdávejte dál ve svém herním shellu.

### Rozdíl od hvězdičky
- `*` = libovolný počet znaků (0 nebo více)
- `?` = přesně jeden znak

### Úkol
Zkopírujte `data1.txt` a `data2.txt` do adresáře `short_data/`.
NEKOPÍRUJTE `data10.txt` (má dvouciferné číslo).

### Příkazy
- V Bashi: `cp VZOR short_data/` - VZOR sestavte sami z `data`, žolíku a `.txt`
- Z fish: `bash -c 'cp VZOR short_data/'` - uvozovky ponechte

Vzor nejdřív ověřte pomocí `ls VZOR`: vypsat se smějí jen soubory, které chcete zkopírovat.

### Odevzdání
Po splnění úkolu odevzdejte: `shellgame submit`