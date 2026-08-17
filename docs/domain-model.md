# Modèle métier initial

Ce modèle décrit les concepts nécessaires au MVP, indépendamment de la base de
données et de l'API.

## Foyer financier

Le foyer est la frontière de consolidation et de propriété des données.

Informations minimales :

- identifiant ;
- nom ;
- devise principale ;
- fuseau horaire.

Le MVP crée un seul foyer et un seul membre. Le rattachement des données au foyer
évite une migration structurelle lorsque le partage sera introduit.

## Compte

Un compte représente un emplacement où une valeur est détenue ou due.

Types initiaux :

- compte courant ;
- épargne ;
- espèces ;
- investissement ;
- carte de crédit ;
- prêt ;
- autre actif ;
- autre dette.

Informations minimales : nom, type, devise et état actif ou archivé.

Dans le MVP, les comptes consolidés utilisent la devise principale du foyer. La conversion entre devises sera ajoutée lorsque le besoin sera réellement traité.

Un solde positif augmente le patrimoine pour un actif et représente une somme due pour une dette. Cette convention devra être rendue explicite dans le code et l'interface.

Le type d'un compte peut évoluer au sein de la même famille, par exemple de
compte courant vers épargne ou de carte de crédit vers prêt. Le passage entre
un actif et une dette reste possible tant qu'aucune transaction n'existe. Il est
ensuite interdit, y compris lorsque les transactions sont supprimées
logiquement, car il inverserait leur interprétation économique historique.

## Établissement financier

Un établissement regroupe les comptes ouverts auprès d'un même organisme. Il
appartient au foyer, mais son rattachement à un compte reste facultatif afin de
représenter des espèces ou des actifs détenus sans intermédiaire.

## Solde de rapprochement

Un solde de rapprochement associe un montant à un compte et à une date. Il sert
de point d'ancrage au calcul transactionnel et de valorisation manuelle
temporaire pour les investissements. Il ne constitue jamais un revenu ou une
dépense.

## Transaction

Une transaction représente un mouvement financier sur un compte.

Informations minimales :

- compte ;
- date de comptabilisation ;
- libellé ;
- montant signé ;
- catégorie éventuelle ;
- note éventuelle ;
- origine manuelle ou importée ;
- référence d'import éventuelle.

Un montant positif augmente le solde du compte et un montant négatif le diminue.
Cette règle est identique pour toutes les transactions stockées.

## Transfert

Un transfert relie deux transactions opposées appartenant à deux comptes du même foyer. Il déplace la valeur sans créer de revenu ni de dépense au niveau consolidé.

Les deux mouvements restent visibles dans chaque compte. Leur lien garantit que les tableaux de bord peuvent les exclure des flux de consommation.

Le montant nominal d'un transfert et le montant d'une transaction possèdent des
sémantiques différentes :

- `TransferAmount` est strictement positif et représente la valeur absolue
  saisie par l'utilisateur ;
- `TransactionAmount` est signé, positif ou négatif mais jamais nul, et
  représente l'effet brut d'un mouvement sur le solde de son compte.

Un `TransferAmount` produit donc deux `TransactionAmount`. Leur signe est dérivé
du rôle source ou destination et du fait que le compte représente un actif ou
une dette. Les montants bruts ne sont pas nécessairement opposés lorsque des
comptes de dette sont impliqués ; ce sont leurs effets économiques qui doivent
s'annuler.

## Catégorie

Une catégorie qualifie un revenu ou une dépense. Elle appartient au foyer afin de permettre une personnalisation ultérieure.

Le MVP fournit un petit catalogue initial et autorise la création, le renommage et l'archivage. Une catégorie archivée reste attachée aux transactions passées.

## Import

Un import représente le traitement d'un fichier externe pour un compte donné.
Il conserve au minimum le nom du fichier, sa date, son résultat et les clés servant à empêcher les doublons.

L'import doit proposer un aperçu avant validation. Une ligne invalide ne doit pas rendre les autres lignes incompréhensibles ou silencieusement incorrectes.

## Valeurs calculées

Les valeurs suivantes sont dérivées des données précédentes et ne constituent pas des sources concurrentes de vérité :

- solde courant d'un compte ;
- total des actifs ;
- total des dettes ;
- patrimoine net ;
- revenus mensuels ;
- dépenses mensuelles ;
- ventilation des dépenses par catégorie.

## Invariants essentiels

- toute donnée financière appartient à exactement un foyer ;
- les deux comptes d'un transfert appartiennent au même foyer ;
- les deux mouvements d'un transfert ont une valeur opposée ;
- une transaction importée possède une clé de déduplication stable ;
- une donnée archivée reste disponible dans l'historique ;
- les calculs consolidés excluent les transferts internes ;
- seules les transactions postérieures au solde de rapprochement retenu
  modifient ce solde ;
- les montants utilisent une représentation décimale, jamais un flottant.

## Décisions reportées

- taux de change et consolidation multidevise ;
- comptes partagés entre plusieurs foyers ;
- transactions ventilées entre plusieurs catégories ;
- rapprochement bancaire ;
- gestion détaillée des titres et lots d'investissement ;
- règles automatiques de catégorisation.
