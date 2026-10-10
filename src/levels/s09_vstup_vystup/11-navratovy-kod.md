# Návratový kód: `&&` a `||`

Každý příkaz skončí návratovým kódem (exit code): **0 znamená úspěch**, nenulová
hodnota neúspěch. V interaktivním Bash i Fish podle něj můžete spojovat příkazy:

- `první && druhý` spustí druhý jen po úspěchu prvního,
- `první || druhý` spustí druhý jen po neúspěchu prvního.

Příkazy `true` a `false` vracejí právě stav 0 a nenulový stav.

## Úkol
Spusťte postupně oba řetězce; zapíšou do souboru `status.txt` dva řádky:

```bash
true  && echo "stav: uspech"   ___ status.txt
false || echo "stav: neuspech" ___ status.txt
```

Na místo `___` doplňte přesměrování `>` nebo `>>`. Rozhodněte se podle toho,
že první řádek soubor zakládá a druhý se musí přidat za něj — výsledek má mít
právě dva řádky v tomto pořadí.

## Odevzdání
`shellgame submit status.txt`