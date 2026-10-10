Příkaz `cp` (copy) vytvoří kopii souboru.

### Proč zálohovat?
Před úpravou důležitého souboru je dobré si udělat zálohu.
Když něco pokazíte, máte se kam vrátit!

V praxi uvidíte:
- `cp config.yaml config.yaml.bak` (před úpravou konfigurace)
- `cp report.docx report_v1.docx` (verzování dokumentů)

### Úkol
1. Začínáte v `level-6/copying`. Vytvořte zálohu souboru `dulezite.txt`. Kopii pojmenujte `dulezite.bak`.
2. Ověřte výsledek pomocí `ls`.

## Příkazy:
- `cp <zdroj> <cíl>`: Zkopíruje zdroj do cíle

## Odevzdání:
Po vytvoření zálohy spusťte:
`shellgame submit`
(můžete také zadat: `shellgame submit dulezite.bak`)