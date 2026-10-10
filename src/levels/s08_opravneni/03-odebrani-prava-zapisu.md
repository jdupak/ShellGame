Někdy chcete zabránit nechtěnému přepsání souboru. Můžete mu odebrat právo pro zápis (`w`).

## Úkol:
Odeberte souboru `config.readonly` právo zápisu pro všechny: vlastníka, skupinu i ostatní.

## Příkazy:
- `chmod a-w <soubor>`: Odeberte write pro all (všechny)

Písmeno `a` nevynechávejte: bez něj výsledek ovlivňuje výchozí maska práv (`umask`).

## Odevzdání:
Po odebrání všech práv zápisu spusťte:
`shellgame submit`