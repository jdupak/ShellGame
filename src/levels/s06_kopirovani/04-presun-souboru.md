Pokud jako cíl příkazu `mv` uvedete existující adresář,
soubor se do něj přesune (a zachová si své jméno, pokud neuvedete jiné).

## Úkol:
Přesuňte soubor `report.pdf` do adresáře `dokumenty`.

## Příkazy:
- `mv <soubor> <adresář>/`: Přesune soubor do adresáře

## Proč to lomítko na konci?
Pokud cílový adresář **neexistuje**, `mv` soubor tiše přejmenuje na zadané jméno.
S lomítkem `mv` místo toho ohlásí chybu, takže se o překlepu v názvu adresáře dozvíte hned.

## Odevzdání:
Po přesunu souboru spusťte:
`shellgame submit`
(můžete také zadat: `shellgame submit dokumenty`)