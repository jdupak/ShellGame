Někdy chcete zachytit VŠECHNO - normální výstup i chyby do jednoho souboru.
K tomu slouží `&>`.

### Proč je to užitečné
Při ladění skriptů nebo automatizaci často potřebujete kompletní log
všeho, co program vypsal - ať už to byla informace nebo chyba.

### Úkol
Spusťte `buggy.sh` a přesměrujte OBOJÍ (stdout i stderr) do `all_output.log`.

### Příkazy
- `./script &> soubor` - přesměruje stdout i stderr

### Odevzdání
Odevzdejte název souboru.
`shellgame submit <název>`