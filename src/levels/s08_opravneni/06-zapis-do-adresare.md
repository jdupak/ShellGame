### Cíl
Prakticky ověřte, že vytvoření nové položky řídí oprávnění **adresáře**,
ne oprávnění kopírovaného souboru.

Adresář `locked` má režim `550`: lze ho číst a procházet, ale nelze do něj zapisovat.

### Předpověď a pozorování
**Nejdřív předpověď:** než cokoli spustíte, řekněte nahlas nebo si zapište, zda `cp source.txt locked/`
projde, nebo selže - a proč. Chybová hláška není překážka, ale hlavní výsledek pokusu; přečtěte si ji celou.

1. Spusťte `cp source.txt locked/` a porovnejte hlášku se svou předpovědí.
2. Ověřte, že čtení ven funguje: `cp locked/report.txt recovered.txt`.
3. Přidejte vlastníkovi adresáře právo zápisu: `chmod u+w locked`.
4. Zopakujte `cp source.txt locked/`; tentokrát musí uspět.

### Vysvětlení
Co rozhodovalo o možnosti vytvořit `locked/source.txt` — oprávnění
zdrojového **souboru**, nebo cílového **adresáře**?

### Odevzdání
`shellgame submit adresář`