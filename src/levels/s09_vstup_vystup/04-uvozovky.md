# Uvozovky v textu i názvu souboru

Shell dělí příkaz podle mezer. Uvozovky proto chrání víceslovný text i název
souboru s mezerami — každou část uzavřete zvlášť.

## Úkol
Pomocí `echo` vytvořte soubor `muj pozdrav.txt` s jediným řádkem `Ahoj svete`.

## Příkaz
- `echo "Ahoj svete" > "muj pozdrav.txt"`

První dvojice uvozovek chrání text, druhá název výstupního souboru.

## Odevzdání
Uvozovky potřebuje i název předaný příkazu `shellgame`:
`shellgame submit "muj pozdrav.txt"`