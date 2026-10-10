Oprávnění lze nastavit i číselně (oktalově).
- 4 = read (r)
- 2 = write (w)
- 1 = execute (x)

Součet dává kombinaci (např. 7 = 4+2+1 = rwx, 5 = 4+1 = r-x).
Zadávají se tři čísla: pro vlastníka, skupinu a ostatní.
Např. `755` znamená `rwx` pro vlastníka, `r-x` pro skupinu, `r-x` pro ostatní.

## Úkol:
Nastavte spouštěcímu skriptu `deploy.sh` oprávnění `755` (`rwxr-xr-x`).

## Příkazy:
- `chmod 755 <soubor>`

## Odevzdání:
Po nastavení přesného režimu spusťte:
`shellgame submit`