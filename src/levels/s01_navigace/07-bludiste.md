### Cíl
Projděte bludištěm podle instrukcí.

### Pravidla
- Start: `level-1/maze/entry/`
- Instrukce je v **názvu** souboru `GO_…` (příkaz `ls`)
- `GO_TO_DIR_x` → `cd x`
- `GO_UP_N` → jděte pomocí `cd ..` o N úrovní výše
- `GO_UP_N_THEN_GO_TO_x` → jděte o N úrovní výše a potom do `x`
- Během bludiště nemůžete odejít mimo `maze/`

### Úkol
1. Ze startu sledujte názvy souborů `GO_…`
2. Bludiště je záměrně delší: procvičíte opakované `ls`, sestup, návraty i přechod k sourozencům
3. Jakmile vstoupíte do skutečného cíle, dokončete level příkazem `shellgame submit`

Pokud se v bludišti ztratíte, příkaz `shellgame reset` vás vrátí zpět na začátek (`maze/entry`).