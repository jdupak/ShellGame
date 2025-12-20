# Sekce 7: Oprávnění

V Linuxu má každý soubor a adresář nastavená oprávnění, která určují, kdo s ním může co dělat.

## 🎯 Proč je to důležité?

### Bezpečnost systému
- **Hesla**: Soubor `/etc/shadow` obsahuje hesla - smí ho číst jen root!
- **Konfigurace**: Webový server nesmí měnit vlastní konfiguraci (jen číst)
- **Sdílení**: Spolužáci nevidí vaše soukromé soubory v domovském adresáři

### Běžné situace
```bash
# "Permission denied" při spuštění skriptu?
$ ./muj_skript.sh
bash: ./muj_skript.sh: Permission denied
$ chmod +x muj_skript.sh   # Řešení!
$ ./muj_skript.sh
Hello World!

# Webový server nevidí soubory?
$ chmod 644 index.html     # Ostatní mohou číst
```

## Tři typy oprávnění
```
r (read)     → Číst obsah souboru / vypsat obsah adresáře
w (write)    → Změnit/smazat soubor / vytvořit soubory v adresáři  
x (execute)  → Spustit jako program / vstoupit do adresáře
```

## Tři skupiny uživatelů
```
u (user)   → Vlastník souboru (vy)
g (group)  → Členové skupiny vlastníka
o (other)  → Všichni ostatní
```

## Jak číst `ls -l`
```
-rwxr-xr--  =  vlastník: rwx, skupina: r-x, ostatní: r--
 ││││││││
 │├┴┤├┴┤├┴┤
 │ u  g  o
 │
 └─ typ (- soubor, d adresář)
```

## Dva způsoby zápisu chmod
```
Symbolický:              Číselný (oktalový):
chmod u+x soubor         chmod 755 soubor
chmod g-w soubor         
chmod o=r soubor         r=4, w=2, x=1
                         755 = rwx|r-x|r-x
```

## Co se naučíte:
- Číst oprávnění (`ls -l`)
- Měnit oprávnění (`chmod`)
- Používat symbolický zápis (`u+x`) i číselný zápis (`755`)
