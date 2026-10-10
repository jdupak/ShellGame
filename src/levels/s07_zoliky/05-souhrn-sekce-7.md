### Výzva: Tři druhy žolíků

### Úkol
V aktuálním adresáři použijte pro každý výběr jiný druh vzoru.
Než začnete kopírovat, nejprve si předpovězte odpovídající názvy a vzor ověřte pomocí `ls VZOR`.

1. Pomocí `*` zkopírujte všechny `.log` soubory do `logs/`.
2. Pomocí `?` zkopírujte do `short_data/` jen názvy `data`, jeden znak a `.txt`.
3. Pomocí znakové třídy zkopírujte do `selected_reports/` reporty s písmenem `a` nebo `b`.

Nakonec odevzdejte počty souborů v těchto třech cílových adresářích.
Formát: `<logy>,<kratka_data>,<vybrane_reporty>`

`?` a `[...]` zde používají syntaxi Bashe. Ve fish spusťte kopírování přes `bash -c`.

### Odevzdání
`shellgame submit <logy>,<data>,<reporty>`