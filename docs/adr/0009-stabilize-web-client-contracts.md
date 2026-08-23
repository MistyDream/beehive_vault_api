# ADR-0009 — Stabiliser les contrats financiers du nouveau client Web

- Statut : Accepté
- Date : 2026-08-23
- Remplace partiellement : ADR-0006, absence de total dans les collections paginées

## Contexte

Les parcours du nouveau client Web sont conçus avant leur implémentation. L'API expose déjà les ressources financières principales, mais certaines réponses demandent encore au navigateur de retrouver des libellés, calculer des signes ou déduire l'existence d'une page suivante. Les mouvements d'un transfert apparaissent également comme deux transactions alors que l'interface les traite comme une seule opération.

Les contrats doivent être stabilisés avant la construction du proxy Nuxt et de la couche d'accès aux données afin de ne pas déplacer les règles comptables ou les dépendances historiques dans le navigateur.

## Décision

Les collections paginées utilisent toujours `page` et `limit`, mais retournent une enveloppe contenant `items`, `page`, `limit` et le nombre exact `total` d'éléments correspondant aux filtres. Le client déduit lui-même l'existence d'une page suivante. Le volume personnel attendu justifie le coût de ce total et ne nécessite pas encore de pagination par curseur.

La collection chronologique des transactions devient une collection consolidée d'opérations. Une transaction ordinaire apparaît une fois et un transfert apparaît une seule fois avec ses deux mouvements. Un discriminateur explicite permet au client de traiter les deux représentations sans déduire leur forme.

Les réponses de transaction incorporent des résumés compacts des comptes et catégories nécessaires à l'affichage. Ces résumés restent disponibles pour les ressources archivées et évitent les requêtes supplémentaires. Ils utilisent le nom actuel de la ressource et ne constituent pas un instantané historique.

La frontière HTTP distingue le montant nominal positif, son effet standard ou inverse, son effet économique signé et le montant brut signé du compte. L'API effectue les conversions propres aux comptes d'actif et de dette. Tous les montants restent des chaînes décimales et respectent la précision maximale de `NUMERIC(20, 4)`.

La collection des comptes fournit les sous-totaux nécessaires aux groupes de l'interface sans demander de calcul décimal au navigateur. Elle distingue les comptes actifs et archivés. L'API fournit également la restauration, la correction des soldes et les validations de date et de solde nécessaires au cycle de vie conçu.

Le catalogue global d'établissements décidé par l'ADR-0008 reste la référence stable des comptes. Les catégories ne reçoivent pas encore de métadonnée d'icône ; le client garantit un pictogramme neutre.

Le contrat détaillé est consigné dans [`docs/client-contracts.md`](../client-contracts.md). Sa documentation précède
son implémentation et distingue explicitement la cible du comportement réel de l'API.

## Conséquences positives

- le navigateur n'implémente aucune règle de signe propre aux actifs et dettes ;
- les transactions historiques restent lisibles avec des références archivées ;
- un transfert possède une représentation unique dans la chronologie ;
- la pagination progressive connaît sa fin et peut aussi annoncer le nombre de résultats ;
- les pages Comptes disposent de sous-totaux exacts sans arithmétique binaire ;
- le contrat cible peut être testé lot par lot avant l'implémentation Web.

## Conséquences négatives

- le calcul du total ajoute une requête ou un calcul SQL aux collections paginées ;
- les résumés incorporés répètent quelques informations dans une page JSON ;
- la collection consolidée demande une requête plus complexe et un comptage des opérations plutôt que des lignes de transaction ;
- la migration modifie des réponses HTTP existantes avant qu'un client externe stable ne les consomme ;
- plusieurs représentations monétaires doivent rester cohérentes et couvertes par des tests.

## Alternatives considérées

### Retourner uniquement `hasMore`

Écartée car le volume attendu permet de fournir un total exact, utile à l'accessibilité et aux résultats filtrés, sans exposer de complexité excessive.

### Retourner les références séparément dans une section `included`

Écartée car elle réduit la répétition au prix d'une normalisation systématique dans le client. Des résumés compacts incorporés restent suffisamment légers pour des pages de 50 opérations et simplifient chaque composant consommateur.

### Laisser le client calculer les signes et les sous-totaux

Écartée car elle dupliquerait les règles du domaine, imposerait une arithmétique décimale au navigateur et risquerait des divergences entre clients.

### Conserver deux lignes pour chaque transfert

Écartée car cette représentation fausse le nombre d'opérations, complique la pagination et ne correspond pas au parcours de consultation et de modification atomique du transfert.
