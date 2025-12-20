# Sekce 6: Kopírování a přesouvání

V této sekci se naučíte manipulovat se soubory a adresáři - vytvářet jejich kopie a přesouvat je na jiná místa.

## `cp` vs `mv` - jaký je rozdíl?
```
cp (copy)                      mv (move)
─────────────────────────────────────────────
Původní soubor ZŮSTANE         Původní soubor ZMIZÍ
Vytvoří duplikát               Přesune/přejmenuje

cp a.txt b.txt                 mv a.txt b.txt
├── a.txt  ← zůstává           └── b.txt  ← jediný soubor
└── b.txt  ← nový
```

## Přejmenování = přesun na stejném místě
```
mv stary_nazev.txt novy_nazev.txt
     │                  │
     └──────────────────┘
         "přesun" = přejmenování
```

## Co se naučíte:
- Kopírovat soubory (`cp`)
- Kopírovat adresáře (`cp -r`)
- Přejmenovávat soubory a adresáře (`mv`)
- Přesouvat soubory a adresáře (`mv`)

### Tip: `-r` znamená "rekurzivně"
Pro kopírování/mazání adresářů potřebujete `-r`, aby se zpracoval i jejich obsah.
