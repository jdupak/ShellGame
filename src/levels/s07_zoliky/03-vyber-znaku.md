# Výběr znaků []

Hranaté závorky `[...]` nahradí JEDEN ze znaků uvnitř.
Například `[abc]` odpovídá znaku 'a', 'b' nebo 'c'.

### Požadavek: Bash
Tento level vyžaduje **Bash**. Ve fish použijte variantu s `bash -c` níže.
Ta spustí pouze kopírování v Bashi; odevzdávejte dál ve svém herním shellu.

### Příklady
- `zprava_[xy].txt` → zprava_x.txt, zprava_y.txt
- `log[123].txt` → log1.txt, log2.txt, log3.txt

### Úkol
Zkopírujte `file_a.txt` a `file_b.txt` do adresáře `ab_files/`.
NEKOPÍRUJTE `file_c.txt`.

### Příkazy
- V Bashi: `cp VZOR ab_files/` - VZOR sestavte sami z `file_`, závorek s povolenými znaky a `.txt`
- Z fish: `bash -c 'cp VZOR ab_files/'` - uvozovky ponechte

Vzor nejdřív ověřte pomocí `ls VZOR`: vypsat se smějí jen soubory, které chcete zkopírovat.

### Odevzdání
Po splnění úkolu odevzdejte: `shellgame submit`