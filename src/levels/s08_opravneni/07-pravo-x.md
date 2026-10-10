### Cíl
Zažijte rozdíl mezi čtením názvů a **průchodem** adresářem.

Začínáte v `parent/child`. Nadřazený adresář má běžná práva `755`.

### Předpověď a pozorování
**Nejdřív předpověď:** po `chmod a-x ..` si zapište nebo řekněte nahlas, co udělá `cd ..` - a proč.
Hlášku, kterou shell vypíše, čtěte pozorně: právě ona je odpovědí, ne překážkou.

1. Odeberte všem právo průchodu nadřazeným adresářem: `chmod a-x ..`.
2. Zkuste `cd ..` a porovnejte výsledek se svou předpovědí.
3. Vraťte vlastníkovi průchod: `chmod u+x ..`.
4. Zopakujte `cd ..`; nyní musí uspět.

### Vysvětlení
Které písmeno oprávnění umožňuje vstoupit do adresáře a procházet přes něj?

### Odevzdání
Z adresáře `parent` spusťte `shellgame submit x`.