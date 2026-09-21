### Sekce 2: Práce se soubory

V této sekci se naučíte pracovat se soubory a pokročilejší navigaci.

### Proč je to důležité?
Skutečná práce v terminálu je stálé přebíhání mezi několika místy:
zdrojový kód a testy, konfigurace a log. Kdo umí jen `cd jmeno`,
vypisuje pokaždé celou cestu znovu. Po této sekci se mezi dvěma místy
přepnete jedním krátkým příkazem — a obsah souboru si zobrazíte,
aniž byste otevírali editor.

### Navigace mezi sourozeneckými adresáři
```
level-2/
├── start/      ← Jste tady
├── finish/     ← Chcete se sem dostat
└── other/

Jak se dostat ze 'start' do 'finish'?
1. Krok zpět: cd ..        (jste v level-2)
2. Krok dovnitř: cd finish (jste ve finish)

Nebo jedním příkazem: cd ../finish
```

### Co se naučíte
- Navigace mezi sousedními adresáři (`cd ../jiný`)
- Rychlý návrat do předchozího adresáře (`cd -`)
- Čtení obsahu souborů (`cat`)
- Přerušení běžícího příkazu klávesovou zkratkou (**Ctrl+C**)
- Vytváření souborů (`touch`)

### Proč je `cd -` užitečný?
Když často přepínáte mezi dvěma místy (např. zdrojový kód a testy),
`cd -` vás vrátí tam, kde jste byli naposledy - bez psaní celé cesty.

### Pokračování
Pro zahájení prvního levelu této sekce stiskněte Enter.
