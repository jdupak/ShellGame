# Propojení příkazů pomocí rour (pipes)

Znak `|` (pipe/roura) pošle výstup jednoho příkazu jako vstup druhému.

## Stavební kameny
- `grep "vzor" soubor` vypíše jen ty řádky souboru, které vzor obsahují.
  Vyhledávání se naplno věnuje Sekce 11; tady `grep` použijeme jako hotový filtr.
- `wc -l` spočítá řádky, které dostane na vstupu.
- `|` pošle výstup levého příkazu na vstup pravého.

## Úkol
V aktuálním adresáři je soubor `access.log` s mnoha řádky.
Spočítejte, kolik řádků obsahuje slovo "ERROR". Příkaz si složte sami ze tří dílů výše.

## Odevzdání
Odevzdejte nalezený počet (číslo).
`shellgame submit <číslo>`