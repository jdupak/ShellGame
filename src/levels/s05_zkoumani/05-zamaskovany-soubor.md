### Cíl
Najděte soubor s příponou `.jpg`, který ve skutečnosti není obrázkem.

V adresáři `downloads` je mnoho různých stažených souborů (dokumenty, archivy,
skripty i logy). Někdo se pokusil ukrýt tajnou textovou zprávu tím, že ji pojmenoval
s příponou `.jpg`.

### Příkazy
- `file *.jpg` (zkontroluje pouze soubory s příponou .jpg místo všech položek)

### Úkol
1. V adresáři `downloads` je mnoho různých souborů. Spuštění `file *` by vypsalo
   desítky položek bez užitku.
2. Pomocí `file *.jpg` zkontrolujte pouze soubory s příponou `.jpg`.
3. Najděte, který soubor je ve skutečnosti textový (`ASCII text`).
4. Odevzdejte název tohoto souboru.

### Odevzdání
`shellgame submit <soubor>`