# Sekce 10: Žolíky (Wildcards)

V této sekci se naučíte pracovat s více soubory najednou pomocí žolíků (wildcards).
Žolíky vám umožní definovat vzory názvů souborů.

## Přehled žolíků
```
*       Jakýkoliv počet znaků (včetně nuly)
?       Právě jeden znak
[...]   Jeden ze znaků v závorkách
[a-z]   Rozsah znaků
```

## Příklady
```
*.txt         → všechny .txt soubory
data?.csv     → data1.csv, data2.csv, ale NE data10.csv
file_[ab].md  → file_a.md, file_b.md, ale NE file_c.md
[A-Z]*.py     → soubory začínající velkým písmenem
```

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
- Rozsahy znaků (`[a-z]`, `[0-9]`)
