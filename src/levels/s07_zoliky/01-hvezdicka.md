# Hvězdička *

Hvězdička `*` nahradí JAKOUKOLIV sekvenci znaků (včetně prázdné).
Je velmi užitečná pro výběr souborů se specifickou příponou.

Potkali jste ji už dříve (`file *`, `mv *.log logs/`); tady si její pravidla poprvé pojmenujeme přesně.

### Proč je to užitečné
Představte si, že máte 100 fotografií a chcete je všechny zkopírovat.
Místo `cp foto1.jpg foto2.jpg foto3.jpg ...` stačí `cp *.jpg cíl/`.

### Úkol
Zkopírujte všechny soubory s příponou `.jpg` do adresáře `images`.
(Adresář `images` již existuje).

### Příkazy
- `cp *.jpg adresář/` - zkopíruje všechny .jpg soubory

### Odevzdání
Po splnění úkolu odevzdejte: `shellgame submit`