# Sekce 8: Přesměrování a roury (pipes)

Většina příkazů vypisuje svůj výstup na obrazovku (standardní výstup - stdout).
Tento výstup můžete přesměrovat do souboru nebo poslat jinému příkazu.

## 🎯 Proč je to důležité?

### Automatizace a skripty
```bash
# Denní záloha - výstup do logu
./backup.sh > /var/log/backup_$(date +%F).log

# Monitorování serveru
uptime >> server_stats.txt
```

### Analýza dat
```bash
# Kolik unikátních IP adres přistoupilo na web?
cut -d' ' -f1 access.log | sort -u | wc -l

# Najdi 10 největších souborů
du -ah /home | sort -rh | head -10
```

> 💡 **Tip:** Nepoužívejte `cat soubor | prikaz`, když `prikaz` umí číst soubor
> přímo (`prikaz soubor`). Zbytečný `cat` navíc spouští další proces.
> Také `sort | uniq` lze zkrátit na `sort -u`.

### Filtrování výstupu
```bash
# Příliš mnoho výstupu? Najdi jen chyby:
make 2>&1 | grep -i error
```

## Přesměrování vizuálně
```
Bez přesměrování:           S přesměrováním:
┌─────────┐                 ┌─────────┐
│ příkaz  │──── stdout ───▶ │ příkaz  │──── > ───▶ soubor.txt
└─────────┘      │          └─────────┘
                 ▼
            obrazovka
```

## `>` vs `>>` vs `|`
```
>   přepíše soubor (pozor na ztrátu dat!)
>>  přidá na konec souboru
|   pošle výstup dalšímu příkazu (roura/pipe)
```

## Co se naučíte:
- Uložit výstup příkazu do souboru (`>`)
- Přidat výstup na konec souboru (`>>`)
- Propojovat příkazy pomocí rour (`|`)
- Zobrazovat části souborů (`head`, `tail`)
- Počítat řádky, slova a znaky (`wc`)
