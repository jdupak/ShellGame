Aby šel skript spustit (např. `./script.sh`), musí mít nastavené právo `x`.
Příkaz `chmod` (change mode) mění oprávnění.

### Proč je to důležité?
Když napíšete skript nebo stáhnete program, často není automaticky spustitelný.
Musíte mu explicitně dát právo ke spuštění - je to bezpečnostní opatření.

### Běžný pracovní postup
1. Napíšete skript: `nano muj_skript.sh`
2. Pokusíte se spustit: `./muj_skript.sh` → "Permission denied"
3. Přidáte právo: `chmod u+x muj_skript.sh`
4. Nyní funguje: `./muj_skript.sh` ✓

## Úkol:
Soubor `run_me.sh` nejde spustit. Přidejte mu právo pro spuštění pro vlastníka (`u`).

## Příkazy:
- `chmod u+x <soubor>`: Přidá právo execute pro usera

## Odevzdání:
Po přidání práva spuštění spusťte:
`shellgame submit`
(můžete také zadat: `shellgame submit run_me.sh`)