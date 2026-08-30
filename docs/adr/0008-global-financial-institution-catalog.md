# ADR-0008 — Utiliser un catalogue global d'établissements financiers

- Statut : Accepté
- Date : 2026-08-20

## Contexte

Le modèle actuel rattache les établissements financiers à un foyer. Chaque utilisateur peut créer, renommer et archiver ses propres entrées. Cette approche permet de représenter rapidement n'importe quel organisme, mais produit des noms dupliqués ou incohérents et ne fournit aucune identité stable pour un logo, un pays ou une future intégration bancaire.

La conception du client web retient un établissement facultatif lors de la création d'un compte. L'absence d'une entrée ne doit pas empêcher de suivre le compte. La future synchronisation bancaire devra également distinguer l'établissement affiché, le prestataire d'agrégation et la connexion consentie par un utilisateur.

## Décision

Beehive Vault utilisera un catalogue global d'établissements financiers, configuré côté serveur et commun à tous les foyers.

Un compte référence facultativement une entrée de ce catalogue. Le client peut sélectionner « Aucun établissement », mais ne peut ni créer, ni renommer, ni archiver une entrée du catalogue.

La collection sera exposée hors du périmètre d'un foyer :

```text
GET /v1/institutions
```

Le catalogue conserve une identité stable et les métadonnées réellement utiles à l'affichage.
Les logos, pays et identifiants externes ne seront ajoutés que lorsqu'un besoin concret les exige.

Un établissement référencé ne signifie pas qu'il est synchronisable. Une future intégration modélisera séparément :

- la correspondance entre le catalogue et les prestataires d'agrégation ;
- la connexion bancaire consentie par un utilisateur ;
- les comptes externes découverts par cette connexion.

Un établissement absent ne produit pas automatiquement une nouvelle entrée. Une future fonction de signalement pourra enregistrer le besoin afin que le catalogue soit enrichi de manière contrôlée.

Le catalogue est administré hors de l'API HTTP avec un outil serveur explicite.
Cet outil permet de lister, ajouter et renommer une entrée, ainsi que d'importer un fichier JSON de manière additive et réentrante. L'import ne renomme ni ne supprime une entrée absente du fichier. Il n'est jamais exécuté implicitement au démarrage de l'API. La base reste ainsi la source de vérité après l'import et l'identifiant d'une entrée demeure stable lors d'un renommage.

La migration regroupe les anciennes entrées sans distinction de casse, rattache les comptes à leur identité globale et retire les routes d'établissements propres au foyer. Une installation neuve reçoit le catalogue initial lorsque l'administrateur déclenche son import.

## Conséquences positives

- un même établissement possède une identité et un nom cohérents ;
- les foyers ne maintiennent plus leur propre référentiel dupliqué ;
- l'établissement reste facultatif et ne bloque aucun compte manuel ;
- les futures métadonnées et correspondances de synchronisation ont un point d'ancrage stable ;
- la distinction entre établissement et connexion bancaire reste explicite.

## Conséquences négatives

- l'utilisateur ne peut pas ajouter immédiatement un établissement inconnu ;
- l'initialisation et la maintenance du catalogue demandent une opération serveur explicite ;
- les routes, contraintes et données actuellement rattachées aux foyers devront être migrées ;
- plusieurs établissements proches ou marques d'un même groupe demanderont une règle éditoriale cohérente ;
- une installation personnalisée devra modifier le catalogue côté serveur.

## Alternatives considérées

### Conserver un catalogue propre à chaque foyer

Rejetée car il duplique les mêmes organismes et ne fournit pas d'identité globale pour les métadonnées ou la synchronisation.

### Stocker un nom libre directement sur le compte

Rejetée car les variantes orthographiques empêcheraient tout rattachement fiable à des logos ou prestataires externes.

### Coder la liste dans le client web

Rejetée car les autres clients ne partageraient pas la même référence et parce que toute mise à jour imposerait une nouvelle version du client.

### Créer automatiquement une entrée lorsqu'elle manque

Rejetée car cette stratégie réintroduirait les doublons que le catalogue global doit éviter. Le compte reste utilisable sans établissement.
