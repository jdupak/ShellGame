### Výzva: Mistři skrytých souborů

Ukažte, že ovládáte práci se skrytými soubory!

### Úkol
V adresáři `level-3/final_test` jsou normální i skryté položky.

1. Pomocí `ls -la` spočítejte **skryté adresáře** (název začíná tečkou, řádek znakem `d`; bez `.` a `..`)
2. Najděte skrytý soubor `.secret_code`
3. Přečtěte jeho obsah
4. Odevzdejte dvojici ve schématu `POČET,KÓD`

### Shrnutí příkazů Sekce 3
```
ls -a         → Zobrazí vše včetně skrytých
ls -la        → Přidá podrobnosti; adresář má na začátku řádku d
cat .soubor   → Přečíst skrytý soubor
cd .adresar   → Vstoupit do skrytého adresáře
```

### Odevzdání
`shellgame submit <hodnota>`