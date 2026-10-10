### Cíl
Naučte se prohlížet a prohledávat velké soubory pomocí interaktivního nástroje `less`.

Příkaz `cat` vypíše celý soubor najednou, což je u dlouhých souborů nepraktické.
Interaktivní stránkovač `less` umožní souborem pohodlně listovat a hledat v něm.

### 🚪 Jak z less odejít: klávesa `q` (Quit)
Nástroj `less` zabere celou obrazovku terminálu. Během jeho běhu nelze zadávat
běžné příkazy shellu.
**Pro ukončení prohlížeče a návrat do příkazové řádky stiskněte klávesu `q`.**

### Ovládání less
```
q                     → UKONČIT prohlížení (návrat do shellu)
/hledany_text         → Hledat text v souboru (potvrdit Enterem)
n                     → Další výskyt hledaného textu
Enter / šipka dolů    → Posun o jeden řádek dolů
Mezerník / Page Down  → Posun o celou stránku dolů
b / Page Up           → Posun o stránku nahoru
šipka nahoru          → Posun o řádek nahoru
g / G                 → Na začátek / na konec souboru
```

### Úkol
Soubor `server.log` má 200 řádků. Najděte v něm chybový kód na řádku s textem `CRITICAL`:

1. Otevřete soubor: `less server.log`
2. Hledejte: stiskněte `/`, napište `CRITICAL` a stiskněte **Enter**
3. Přečtěte chybový kód (číslo za slovem `Code`) na konci nalezeného řádku
4. **Stiskněte klávesu `q`** pro ukončení `less` a návrat do příkazové řádky
5. Odešlete nalezený kód pomocí `shellgame submit`

### Odevzdání
`shellgame submit <číslo>`