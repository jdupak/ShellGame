# Třídy znaků [[:lower:]]

Rozsahy jako `[a-z]` závisejí na řazení nastaveného jazyka (locale).
POSIX třída `[[:lower:]]` místo pořadí vybírá jeden znak klasifikovaný
jako malé písmeno. Pro tento úkol je proto spolehlivější napříč locale.

Podobně existují `[[:upper:]]` pro velká písmena a `[[:digit:]]` pro číslice.

### Požadavek: Bash
Tento level vyžaduje **Bash**. Ve fish použijte variantu s `bash -c` níže.
Ta spustí pouze kopírování v Bashi; odevzdávejte dál ve svém herním shellu.

### Úkol
Zkopírujte všechny soubory `.txt` začínající malým písmenem do adresáře `lowercase/`.
NEKOPÍRUJTE soubory začínající velkým písmenem.

### Příkazy
- V Bashi: `cp VZOR lowercase/` - VZOR sestavte sami z třídy znaků, `*` a `.txt`
- Z fish: `bash -c 'cp VZOR lowercase/'` - uvozovky ponechte

Vzor nejdřív ověřte pomocí `ls VZOR`: vypsat se smějí jen soubory, které chcete zkopírovat.

Přípona `.txt` vyloučí cílový adresář `lowercase`, který také začíná malým písmenem.

### Odevzdání
Po splnění úkolu odevzdejte: `shellgame submit`