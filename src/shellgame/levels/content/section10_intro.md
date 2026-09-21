# Sekce 10: Žolíky (Wildcards)

V této sekci se naučíte pracovat s více soubory najednou pomocí žolíků (wildcards).
Žolíky vám umožní definovat vzory názvů souborů.

## Kdy potřebujete Bash?
Hvězdička `*` funguje v Bashi i ve fish. Levely **10.2–10.4 vyžadují Bash**:
otazník `?`, množiny `[...]` a znakové třídy nejsou přenositelné do fish.
Stejnou bashovou syntaxi si znovu procvičíte v souhrnu 10.5.

Pokud hrajete ve fish, každý z těchto levelů nabídne příkaz ve tvaru
`bash -c 'příkaz'`. Ten spustí pouze daný příkaz v Bashi a vrátí vás do hry.
Vnější jednoduché uvozovky zachovají žolíky pro Bash.
`shellgame submit` pak zadejte jako obvykle ve svém herním shellu.

## Přehled žolíků v Bashi
```
*       Jakýkoliv počet znaků (včetně nuly)
?       Právě jeden znak
[...]   Jeden ze znaků v závorkách
[a-z]   Rozsah znaků (závisí na locale)
[[:lower:]] Jeden znak klasifikovaný jako malé písmeno
```

## Příklady pro Bash
```
*.txt         → všechny .txt soubory
data?.csv     → data1.csv, data2.csv, ale NE data10.csv
file_[ab].md    → file_a.md, file_b.md, ale NE file_c.md
[[:lower:]]*.py → soubory začínající malým písmenem
```

> ⚠️ **Pozor na rozsahy:** `[a-z]` a `[A-Z]` vycházejí z pořadí znaků
> nastaveného locale. Pro význam „malé písmeno“ proto v levelu 10.4 použijete
> POSIX třídu `[[:lower:]]`; podobně existují `[[:upper:]]` a `[[:digit:]]`.

## Jak to funguje?
```
Vy napíšete:     Shell expanduje na:
cp *.jpg imgs/   cp foto1.jpg foto2.jpg foto3.jpg imgs/
       │                    │
       └── žolík ──────────┘ skutečné soubory
```

## Co se naučíte:
- Vybírat soubory hvězdičkou (`*`)
- Přesně jeden znak otazníkem (`?`)
- Množinu znaků hranatými závorkami (`[abc]`)
- Rozsahy a locale-odolné třídy znaků (`[a-z]`, `[[:lower:]]`, `[[:digit:]]`)

## Pokračování
Pro zahájení prvního levelu této sekce stiskněte Enter.
