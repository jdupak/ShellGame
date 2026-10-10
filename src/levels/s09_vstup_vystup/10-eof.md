# Interaktivní vstup a EOF

Když spustíte `cat > soubor`, příkaz čte řádky z klávesnice a zapisuje je do souboru.
Na prázdném řádku stiskněte **Ctrl+D**: terminál tím oznámí EOF (konec vstupu) a `cat`
řádně skončí. **Ctrl+C** místo toho běžící příkaz přeruší (interrupt).

## Úkol
Spusťte `cat > poznamka.txt` a zadejte přesně tyto dva řádky:

```text
První řádek
Druhý řádek
```

Po druhém řádku stiskněte Enter a potom na prázdném řádku Ctrl+D.

## Odevzdání
Po ukončení zápisu pomocí Ctrl+D spusťte:
`shellgame submit`
(můžete také zadat: `shellgame submit poznamka.txt`)