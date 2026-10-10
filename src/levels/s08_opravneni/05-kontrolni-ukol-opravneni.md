### Kontrolní úkol: Oprávnění souborů

Ukažte, že rozumíte oprávněním běžných souborů předtím, než přejdeme k adresářům!

### Úkol
V `level-8/challenge` jsou 4 soubory:

1. `script.sh` - potřebuje být **spustitelný** vlastníkem
2. `secret.txt` - odeberte **právo zápisu všem**
3. `shared.txt` - nastavte oprávnění **644** (rw-r--r--)
4. `team.txt` - přidejte skupině právo čtení, ostatní práva neměňte

### Shrnutí příkazů Sekce 8
```
ls -l            → Zobrazí oprávnění
chmod u+x soubor → Přidá vlastníkovi právo spuštění
chmod a-w soubor → Odebere všem právo zápisu
chmod g+r soubor → Přidá skupině právo čtení
chmod 755 soubor → Nastaví rwxr-xr-x
chmod 644 soubor → Nastaví rw-r--r--
```

### Oktalové oprávnění
```
4 = read    2 = write    1 = execute
7 = rwx     6 = rw-      5 = r-x     4 = r--
```

### Odevzdání
Po splnění všech bodů spusťte `shellgame submit`.