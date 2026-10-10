### Cíl
Přejděte z aktuálního podadresáře do sousedního (sourozeneckého) adresáře.

### Struktura adresářů
```
level-2/
├── start/       ← Zde se nacházíte (start)
└── finish/      ← Váš cíl
```

### Navigace mezi sourozenci
Adresáře `start` a `finish` leží vedle sebe ve stejném rodičovském adresáři (`level-2`).
Protože `finish` neleží uvnitř `start`, nelze použít přímý příkaz `cd finish`.
Musíte se nejprve vrátit k rodiči (`..`) a odtud vstoupit do cíle:
- Ve dvou krocích: nejprve o úroveň výše (`cd ..`) a potom do cíle.
- Nebo v jednom kroku spojenou relativní cestou: `cd ../<cílový_adresář>`.

### Úkol
1. Začínáte v adresáři `level-2/start`
2. Přejděte do sousedního adresáře `finish`
3. V cíli ověřte polohu příkazem `pwd` a odešlete řešení

### Odevzdání
V cílovém adresáři spusťte:
`shellgame submit`