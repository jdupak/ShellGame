### Cíl
Najděte všechny řádky obsahující slovo `warning` bez ohledu na velikost písmen a spočítejte je.

### Příkazy
- `grep -i "vzor" soubor` - ignoruje velikost písmen
- `grep -i "vzor" soubor | wc -l` - spočítá nalezené řádky

### Úkol
Spočítejte, kolik řádků v `messages.log` odpovídá `warning` (case-insensitive).

### Odevzdání
`shellgame submit <číslo>`