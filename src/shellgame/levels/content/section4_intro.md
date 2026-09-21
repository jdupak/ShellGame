### Sekce 4: Vytváření a mazání

V této sekci se naučíte, jak vytvářet nové soubory a adresáře a jak po sobě uklidit.

### Proč je to důležité?
Zatím jste se pohybovali po struktuře, kterou někdo připravil za vás.
Každý nový projekt ale začíná prázdným adresářem, který si musíte založit sami —
a na serveru, kde není žádné grafické rozhraní, je příkazová řádka jediná možnost.
S úklidem přichází i odpovědnost: mazání v terminálu je okamžité a konečné.

### Životní cyklus položek
Každá položka v systému prochází třemi fázemi:
- **Vytvoření**: `touch soubor.txt` / `mkdir projekt/`
- **Práce**: úprava obsahu, čtení, navigace
- **Úklid**: `rm soubor.txt` / `rm -r projekt/`

### Vytváření zanořených struktur (`mkdir -p`)
Příkaz `mkdir` běžně vyžaduje, aby rodičovský adresář už existoval:

```
$ mkdir a/b/c
mkdir: cannot create directory 'a/b/c': No such file or directory
```
❌ **Chyba**: nadřazené složky `a` ani `b` ještě neexistují.

S přepínačem **`-p`** (parents) shell vytvoří celou cestu najednou:

```
$ mkdir -p a/b/c
```
✓ **Výsledek**: automaticky se vytvoří celá větev `a/` → `a/b/` → `a/b/c/`.

### Co se naučíte
- Vytvářet prázdné soubory (`touch`)
- Vytvářet adresáře (`mkdir`)
- Vytvářet zanořené struktury (`mkdir -p`)
- Mazat soubory (`rm`)
- Mazat adresáře (`rmdir`, `rm -r`)

### ⚠️ Pozor na `rm`!
Příkaz `rm` nepřesouvá soubory do koše — maže je rovnou a nevratně.

### Pokračování
Pro zahájení prvního levelu této sekce stiskněte Enter.
