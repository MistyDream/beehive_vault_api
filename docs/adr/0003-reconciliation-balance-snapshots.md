# ADR-0003 — Utiliser des soldes de rapprochement comme points d'ancrage

- Statut : Accepté
- Date : 2026-08-10

## Contexte

Additionner les transactions disponibles ne suffit pas toujours à calculer le solde d'un compte. Un import peut ne couvrir que quelques mois et omettre le solde antérieur. À l'inverse, stocker uniquement un solde courant mutable ferait perdre l'historique et empêcherait d'expliquer les écarts.

Les comptes d'investissement ont une difficulté supplémentaire : leurs versements et retraits ne représentent pas leur valeur de marché.

## Décision

Un compte possède des soldes de rapprochement datés. Pour un compte bancaire, le solde calculé est le dernier solde de rapprochement auquel sont ajoutées les transactions postérieures à sa date.

Les transactions restent la source des revenus, dépenses et mouvements. Un solde de rapprochement ne crée jamais de flux financier.

Tant que les positions d'investissement ne sont pas gérées, la valeur d'un compte d'investissement est fournie par sa dernière valorisation manuelle. À terme, elle sera calculée depuis les positions et les prix de marché, les valorisations manuelles restant des points de contrôle.

## Conséquences positives

- un historique partiel de transactions peut produire un solde fiable ;
- les écarts avec un relevé bancaire peuvent être détectés et rapprochés ;
- l'évolution du patrimoine peut être reconstruite ;
- les comptes d'investissement sont utilisables avant le suivi des positions ;
- les snapshots ne polluent pas les revenus et dépenses.

## Conséquences négatives

- le calcul doit éviter de compter deux fois les transactions antérieures au solde de référence ;
- les dates de solde et de transaction doivent avoir une sémantique précise ;
- les comptes bancaires et d'investissement n'utilisent pas toujours la même stratégie de valorisation ;
- les rapprochements ajoutent davantage d'états qu'un champ `current_balance`.

## Alternatives considérées

### Calculer le solde uniquement depuis les transactions

Rejetée car elle exige un historique complet depuis l'ouverture de chaque compte.

### Stocker uniquement le solde courant sur le compte

Rejetée car chaque mise à jour écraserait la valeur précédente et rendrait le
solde difficile à expliquer ou à auditer.
