# Sekce 4: Vytváření a mazání

V této sekci se naučíte, jak vytvářet nové soubory a adresáře a jak po sobě uklízet.

## Životní cyklus souborů
```
Vytvoření          Práce           Úklid
─────────────────────────────────────────────
touch soubor.txt   (editace...)   rm soubor.txt
mkdir projekt/     (práce...)     rm -r projekt/
```

## Vytváření struktur jedním příkazem
```
BEZ -p (selže):              S -p (funguje):
mkdir a/b/c                  mkdir -p a/b/c
❌ "No such file or         ✓ Vytvoří a/, a/b/, a/b/c/
   directory"
```

## Co se naučíte:
- Vytvářet prázdné soubory (`touch`)
- Vytvářet adresáře (`mkdir`)
- Vytvářet zanořené struktury (`mkdir -p`)
- Mazat soubory (`rm`)
- Mazat adresáře (`rmdir`, `rm -r`)

### ⚠️ Pozor na `rm`!
V příkazové řádce neexistuje koš. `rm` = pryč navždy.
