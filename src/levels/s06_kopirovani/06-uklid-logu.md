V adresáři je nepořádek. Všechny soubory s příponou `.log` by měly být v adresáři `logs`.

## Úkol:
Přesuňte všechny `.log` soubory (`app.log`, `error.log`) do adresáře `logs`.
Zvládnete to jedním příkazem, když místo výčtu jmen zadáte vzor se žolíkem `*`.

## Příkazy:
- `mv VZOR logs/`: za `VZOR` dosaďte vzor, který vybere právě `.log` soubory

Vzor si nejdřív ověřte pomocí `ls VZOR` - uvidíte přesně ty soubory, které by `mv` přesunul.
Znak `*` je žolík (zástupný znak); celou rodinu žolíků probereme hned v následující Sekci 7.

## Odevzdání:
Odevzdejte název adresáře, kam jste soubory přesunuli.
`shellgame submit logs`