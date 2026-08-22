# ADR-0007 — Standardiser les erreurs HTTP avec RFC 9457

- Statut : Accepté
- Date : 2026-08-18

## Contexte

L'API retournait un objet JSON propre au projet contenant `code` et `message`. Ce format suffisait aux premiers usages, mais obligeait chaque client à connaître un contrat spécifique et ne distinguait pas clairement le type stable du problème, son résumé et le détail propre à une occurrence.

La reconstruction du client web introduira un proxy Nuxt et une nouvelle couche d'accès à l'API. Le contrat d'erreur doit être stabilisé avant cette intégration afin que les clients puissent traiter les erreurs de manière uniforme sans reproduire la logique interne du serveur.

## Décision

Les réponses d'erreur HTTP de l'API adopteront le format Problem Details défini par la [RFC 9457](https://www.rfc-editor.org/rfc/rfc9457.html) et utiliseront le type de contenu `application/problem+json`.

Une réponse contient les membres standards applicables :

- `type`, une URI stable identifiant la famille du problème ;
- `title`, un résumé court et stable ;
- `status`, identique au statut de la réponse HTTP ;
- `detail`, une explication propre à l'occurrence lorsqu'elle apporte une information sûre et utile ;
- `instance`, uniquement lorsqu'un identifiant d'occurrence exploitable existe ; il est donc omis dans l'implémentation actuelle.

`type` est l'identité canonique du problème. Tant qu'aucun nom de domaine stable n'est disponible, ses valeurs utilisent des URN sous la forme `urn:beehive-vault:problem:<type>`, par exemple `urn:beehive-vault:problem:household-not-found`.

L'extension `code` conserve en parallèle un identifiant métier court en `snake_case`, par exemple `household_not_found`. Elle simplifie le routage dans le client sans remplacer `type`. Les clients n'interprètent jamais `title` ou `detail` comme des identifiants.

Les erreurs de validation ajoutent une extension `errors`. Chaque élément contient `location` (`body`, `path` ou `query`), `pointer` sous forme de fragment JSON Pointer, `code` et `detail`. Le code de l'élément identifie la règle enfreinte indépendamment du code général `validation_error`.

La même représentation doit couvrir les erreurs applicatives et, autant que possible, les rejets produits par Axum pour les chemins, paramètres et corps JSON invalides. Les détails SQL, secrets, données financières et autres informations sensibles ne sont jamais retournés au client.

Les textes humains sont initialement produits en anglais par l'API. Le client web traduit les erreurs connues à partir de `code` et utilise `detail` comme solution de repli. Une négociation par `Accept-Language` ne sera ajoutée que si un client en a réellement besoin.

Un document JSON mal formé ou un paramètre de chemin illisible relève de `400 Bad Request`. Une valeur bien formée qui échoue à une règle de validation relève de `422 Unprocessable Content`. Les conflits métier relèvent de `409 Conflict`. Une collection imbriquée retourne `404 Not Found` lorsque son foyer parent n'existe pas, même si elle aurait autrement été vide.

Les membres JSON inconnus sont ignorés afin que l'ajout d'un champ par un client plus récent ne rende pas automatiquement sa requête invalide. Les routes métier ainsi que les réponses de route inconnue et de méthode non prise en charge partagent ce contrat. Les routes techniques de vivacité et de disponibilité conservent leur réponse dédiée.

Une seule définition Rust associe chaque famille de problème à son `type`, son `code`, son `title` et son statut afin d'empêcher leur divergence entre modules.

## Conséquences positives

- les clients reposent sur une représentation standard des erreurs HTTP ;
- le proxy Nuxt peut transmettre les erreurs sans inventer un second format ;
- les types de problèmes et codes métier deviennent documentables et testables ;
- les traductions du client ne dépendent pas des messages du serveur ;
- les erreurs de validation fournissent des détails structurés.

## Conséquences négatives

- la migration modifie le contrat consommé par les clients existants ;
- les rejets natifs d'Axum demandent une adaptation pour partager le même format ;
- les URI de types de problèmes et les codes deviennent des identifiants publics à maintenir ;
- le contenu de `title`, `detail` et `code` doit rester cohérent dans tous les modules.

## Alternatives considérées

### Conserver `{ code, message }`

Rejetée car cette représentation propre au projet reporte sur chaque client la compréhension et la normalisation des erreurs.

### Normaliser les erreurs uniquement dans le proxy Nuxt

Rejetée car les autres clients de l'API conserveraient un contrat différent et parce que le proxy devrait déduire des informations absentes de la réponse source.

### Retourner uniquement des statuts HTTP

Rejetée car un statut ne suffit pas à identifier précisément une erreur métier, à afficher une explication utile ou à rattacher une erreur à un champ.
