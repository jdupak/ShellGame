Vítejte v první sekci ShellGame! Zde se naučíte základní dovednosti pro pohyb v příkazové řádce. Prvním krokem k ovládnutí příkazové řádky je pochopení, jak funguje souborový systém vašeho počítače.

### Stromová struktura

Souborový systém vašeho počítače je organizován jako stromová struktura adresářů (složek) a souborů. Strom začíná v kořenovém adresáři (root, označeném `/`). Každý adresář může obsahovat další podadresáře, což vytváří hierarchii.

```
/                          ← Kořen (root) - začátek všeho
├── home/                  ← Domovské adresáře uživatelů
│   └── student/           ← VÁŠ domovský adresář (~)
│       ├── dokumenty/
│       ├── stažené/
│       └── projekty/
├── tmp/                   ← Dočasné soubory
│   └── shellgame-<user>/  ← Váš pracovní prostor pro hru (např. shellgame-student/)
│       └── level-1/       ← ZDE ZAČÍNÁTE!
├── etc/                   ← Systémové konfigurace
└── usr/                   ← Programy a knihovny
```

### Working Directory

V každé chvíli se v terminálu nacházíte v určitém adresáři, který se nazývá "pracovní adresář" (working directory). Je to stejné, jako když máte v grafickém rozhraní otevřenou složku a vidíte její obsah. V terminálu však musíte vědět, kde jste, abyste mohli správně zadávat příkazy.

V této sekci si představujte, obrázek souborového stromu jako výše a na něm si představujte šipku stále ukazující na váš aktuální pracovní adresář. Pokaždé, když se někam přesunete pomocí příkazu `cd`, šipka vás bude následovat.

### Cesta (Path)

Cesta (path) je textový řetězec popisující umístění souboru nebo adresáře v systému. Cesta se skládá z jednotlivých adresářů oddělených lomítky (např. `/home/student/dokumenty`).

---

### Klíčové koncepty

V této sekci se zaměříme na tři klíčové dovednosti:
1. **Orientace**: Jak zjistit, kde se právě nacházíte. V grafickém rozhraní to vidíte v záhlaví okna, v terminálu se musíte zeptat.
2. **Průzkum**: Jak zjistit, co se nachází ve vašem okolí. Naučíte se vypisovat obsah adresářů a rozpoznávat různé typy položek.
3. **Pohyb**: Jak se přesouvat mezi adresáři. Naučíte se chodit "dovnitř" do podadresářů, vracet se "zpět" a skákat na konkrétní místa.

### Příkazy, které si osvojíte

- `pwd` (Print Working Directory) - „Kde jsem?“
- `ls` (List) - „Co tu je?“
- `<příkaz> --help` - stručná nápověda k příkazu
- `man <příkaz>` - podrobný manuál
- `cd` (Change Directory) - „Jdi tam!“

Jste připraveni začít svou cestu?
