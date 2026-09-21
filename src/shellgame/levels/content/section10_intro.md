# Sekce 10: Chybové výstupy

Každý příkaz v Linuxu má dva výstupní kanály:
1. **stdout** (fd 1) - standardní výstup pro normální výsledky
2. **stderr** (fd 2) - chybový výstup pro chybové zprávy

## Dva proudy výstupu
```
┌─────────┐
│ příkaz  │──── stdout (1) ───▶ Normální výstup
│         │──── stderr (2) ───▶ Chybové zprávy
└─────────┘
```

## Operátory přesměrování
```
>     Přesměruje stdout (normální výstup)
2>    Přesměruje stderr (chyby)
&>    Přesměruje OBOJÍ (stdout + stderr)
>>    Přidá stdout na konec souboru
2>>   Přidá stderr na konec souboru
```

## Příklad
```bash
./skript.sh           # Obojí na obrazovku
./skript.sh > out.log        # stdout do souboru, stderr na obrazovku
./skript.sh 2> err.log       # stderr do souboru, stdout na obrazovku  
./skript.sh &> all.log       # Všechno do souboru
./skript.sh &> /dev/null     # Zahodí všechno (ticho)
```

## Co se naučíte:
- Standardní chybový výstup (stderr)
- Přesměrování chybových zpráv (`2>`)
- Přesměrování všeho (`&>`)
- Zahazování výstupu (`/dev/null`)
