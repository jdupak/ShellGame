# Sort a Uniq - řazení a deduplikace

Příkazy `sort` a `uniq` jsou mocné nástroje pro zpracování textových dat.

## Stavební kameny
- `sort soubor` vypíše řádky seřazeně, takže stejné hodnoty skončí vedle sebe.
- `uniq` zahodí opakující se řádky, ale pozná jen ty **sousední**.
- `wc -l` spočítá řádky, které dostane na vstupu.

Pořadí proto není libovolné: rozmyslete si, co musí `uniq` dostat na vstup, aby fungoval.

## Úkol
V souboru `visitors.txt` jsou jména návštěvníků (někteří přišli vícekrát).
Zjistěte, kolik je UNIKÁTNÍCH návštěvníků. Příkazy si pospojujte rourami sami.

## Odevzdání
Odevzdejte počet unikátních návštěvníků.
`shellgame submit <číslo>`