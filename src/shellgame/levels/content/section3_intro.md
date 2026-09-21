### Sekce 3: Skryté soubory

V této sekci odhalíte tajemství skrytých souborů a adresářů.

### Proč existují skryté soubory?
V Linuxu soubory začínající tečkou (`.`) jsou "skryté" - nezobrazí se při běžném `ls`.
Používají se pro konfigurační soubory, které nechcete mít na očích při běžné práci.

### Běžné skryté soubory
```
~/                          ← Váš domovský adresář
├── dokumenty/              ← Běžný adresář (viditelný)
├── .bashrc                 ← Konfigurace shellu (SKRYTÝ)
├── .config/                ← Konfigurační adresář (SKRYTÝ)
│   └── shellgame/
│       └── state.json
└── .ssh/                   ← SSH klíče (SKRYTÝ)

Příkaz 'ls' zobrazí pouze: dokumenty/
Příkaz 'ls -a' zobrazí VŠE včetně skrytých!
Příkaz 'ls -la' přidá podrobnosti; adresář poznáte podle 'd' na začátku řádku.
```

### Co se naučíte
- Co jsou skryté soubory (začínají tečkou)
- Jak je zobrazit a rozlišit jejich typ (`ls -a`, `ls -la`)
- Jak s nimi pracovat (čtení, navigace)

### Pokračování
Pro zahájení prvního levelu této sekce stiskněte Enter.
