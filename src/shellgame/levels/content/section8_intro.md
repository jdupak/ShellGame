# Sekce 8: Vstup, výstup a stav příkazů

Příkaz obvykle posílá výstup na obrazovku. Tři operátory rozhodnou, kam poteče dál:

```text
příkaz > soubor       výstup přepíše soubor
příkaz >> soubor      výstup se přidá na konec
příkaz | další        výstup se stane vstupem dalšího příkazu
```

## Tři mentální modely

- `>` je **nový zápis**. Starý obsah cílového souboru zmizí.
- `>>` je **připojení**. Dosavadní obsah zůstane.
- `|` je **roura mezi příkazy**. Nevytváří soubor, jen předává data dál.

```bash
ls > seznam.txt
echo "další řádek" >> seznam.txt
grep ERROR access.log | wc -l
```

Má-li text nebo název souboru mezery, uzavřete každou takovou část zvlášť:

```bash
echo "Ahoj svete" > "muj pozdrav.txt"
```

---

# Klávesnicový vstup a stav příkazu

`cat > soubor` čte řádky z klávesnice. **Ctrl+D** oznámí EOF, tedy konec vstupu,
a `cat` řádně skončí. **Ctrl+C** je interrupt: běžící příkaz přeruší.

Každý příkaz také vrací stavový neboli návratový kód:

- `0` znamená úspěch,
- nenulový kód znamená neúspěch,
- `první && druhý` pokračuje jen po úspěchu,
- `první || druhý` pokračuje jen po neúspěchu.

`&&` a `||` můžete takto používat přímo v podporovaném Bash i Fish.

## Co se naučíte

- ukládat a přidávat výstup pomocí `>` a `>>`
- správně citovat víceslovný text i názvy souborů
- zapisovat interaktivní vstup přes `cat` a ukončit jej pomocí EOF
- propojovat příkazy pomocí `|`
- filtrovat a zpracovávat text přes `grep`, `head`, `tail`, `wc`, `sort` a `uniq`
- reagovat na úspěch či neúspěch příkazu pomocí `&&` a `||`
