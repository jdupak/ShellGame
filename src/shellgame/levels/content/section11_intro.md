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
find . -name "*.py"            Všechny .py soubory
find /home -name "config*"     Soubory začínající na "config"
find . -type d -name "test*"   Pouze adresáře
```

## Proč je to důležité?
```
Ztratili jste soubor?          → find . -name "soubor.txt"
Hledáte kde je chyba v kódu?   → grep -r "ERROR" src/
Kolik TODOs máte v projektu?   → grep -r "TODO" . | wc -l
```

## Co se naučíte:
- Hledat text v souborech (`grep`)
- Rekurzivní hledání (`grep -r`)
- Hledat soubory podle názvu (`find -name`)
- Kombinovat nástroje
