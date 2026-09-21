# Sekce 11: Vyhledávání

V této závěrečné sekci se naučíte, jak najít text uvnitř souborů a jak najít samotné soubory.

## Dva typy hledání
```
grep = Hledání UVNITŘ souborů (obsah)
find = Hledání SOUBORŮ samotných (podle názvu, typu, velikosti...)
```

## grep - hledání textu
```
grep "error" log.txt           Najde řádky s "error"
grep -r "TODO" projekt/        Rekurzivně v celém adresáři
grep -i "warning" *.log        Ignoruje velikost písmen
```

## find - hledání souborů
```
find . -type f -name "*.py"          Všechny běžné .py soubory
find /home -type f -name "config*"   Soubory začínající na "config"
find . -type d -name "test*"         Pouze adresáře
```

Uvozovky kolem `"*.py"` jsou důležité: zabrání shellu, aby hvězdičku
rozbalil předem. Doslovný vzor tak dostane příkaz `find`, který ho vyhodnotí
v každém prohledávaném adresáři.

## Proč je to důležité?
```
Ztratili jste soubor?          → find . -type f -name "soubor.txt"
Hledáte kde je chyba v kódu?   → grep -r "ERROR" src/
Kolik TODOs máte v projektu?   → grep -r "TODO" . | wc -l
```

## Co se naučíte:
- Hledat text v souborech (`grep`)
- Rekurzivní hledání (`grep -r`)
- Hledat soubory podle typu a názvu (`find -type f -name`)
- Kombinovat nástroje
