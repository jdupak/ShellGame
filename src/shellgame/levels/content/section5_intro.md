# Sekce 5: Zkoumání souborů

V této sekci se naučíte, jak zjistit více informací o souborech, než jen jejich název.

## 🎯 Proč je to důležité?

### Bezpečnost
Stáhli jste soubor `faktura.pdf` z e-mailu. Ale je to opravdu PDF?
Příkaz `file` vám prozradí pravdu - může to být škodlivý skript!

### Praktická práce
- **Vývojář**: "Proč mi nefunguje import? Ten soubor vypadá prázdný..." → `ls -l` ukáže, že má 0 bajtů
- **Admin**: "Který log zabírá místo na disku?" → `ls -lh` ukáže velikosti čitelně
- **Student**: "Je tohle textový soubor nebo binární?" → `file` řekne přesný typ

## Výstup `ls -l` vysvětlen
```
-rw-r--r-- 1 student users 12345 Dec  5 10:30 dokument.txt
│├──┼──┼──│   │       │     │     │            │
││  │  │  │   │       │     │     │            └─ název
││  │  │  │   │       │     │     └─ datum změny
││  │  │  │   │       │     └─ velikost v bajtech
││  │  │  │   │       └─ skupina
││  │  │  │   └─ vlastník
││  │  │  └─ oprávnění pro ostatní (r--)
││  │  └─ oprávnění pro skupinu (r--)
││  └─ oprávnění pro vlastníka (rw-)
│└─ typ souboru (- = soubor, d = adresář)
```

## Příkaz `file` - pravda o obsahu
```
$ file foto.jpg
foto.jpg: JPEG image data     ← Opravdu obrázek

$ file fake.jpg  
fake.jpg: ASCII text          ← Někdo lhal! Je to text.
```

## Co se naučíte:
- Zjišťovat velikost souborů (`ls -l`, `ls -lh`)
- Určovat typ souboru (`file`)
- Rozlišovat mezi textovými a binárními soubory

Názvy souborů mohou být klamavé - proto je důležité umět soubory správně prozkoumat.
