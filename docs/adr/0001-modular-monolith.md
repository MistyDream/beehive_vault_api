# ADR-0001 — Utiliser un monolithe modulaire organisé par fonctionnalité

- Statut : Accepté
- Date : 2026-08-10

## Contexte

La version `0.1.0` utilisait une architecture hexagonale séparée en couches `domain`, `application` et `infrastructure`. Chaque ressource possédait souvent un service, un port sous forme de trait et une implémentation de repository.

Cette structure isolait les technologies, mais introduisait de nombreux fichiers, conversions et abstractions pour des opérations ne contenant aucune logique métier. Le projet est actuellement développé par une seule personne, utilise une seule API HTTP et cible durablement PostgreSQL.

Le redémarrage doit privilégier l'apprentissage de Rust et la livraison du socle financier plutôt que la maintenance d'abstractions anticipées.

## Décision

Beehive Vault est développé comme un monolithe modulaire organisé par fonctionnalité.

Chaque module fonctionnel contient uniquement les routes, types, requêtes et services dont il a besoin. Un service est introduit lorsqu'une opération possède une véritable orchestration ou règle métier, pas comme intermédiaire obligatoire entre une route et une requête.

Les règles financières restent indépendantes des types HTTP. Les intégrations externes peuvent être isolées derrière un trait lorsqu'une substitution réelle ou un faux de test le justifie.

## Conséquences positives

- le parcours d'une fonctionnalité utilise moins de fichiers et de conversions ;
- la complexité visible correspond davantage au domaine financier ;
- les fonctionnalités peuvent être développées et testées verticalement ;
- les abstractions apparaissent lorsque leur besoin est observable ;
- le déploiement reste simple avec un seul exécutable.

## Conséquences négatives

- les modules peuvent être plus directement couplés à SQLx et PostgreSQL ;
- la discipline des frontières repose sur l'organisation et les revues de code ;
- extraire ultérieurement un service indépendant nécessitera un travail explicite ;
- remplacer PostgreSQL demanderait davantage qu'une nouvelle implémentation de
  repository.

Ces coûts sont acceptés car aucun remplacement de PostgreSQL ni découpage en
services indépendants n'est prévu dans le périmètre actuel.

## Alternatives considérées

### Conserver l'architecture hexagonale existante

Rejetée car son coût est immédiat alors que ses principaux bénéfices restent
hypothétiques pour la taille et le contexte du projet.

### Utiliser une architecture entièrement plate

Rejetée car elle mélangerait rapidement les routes, les règles financières et
les requêtes. L'organisation modulaire conserve des frontières sans imposer une
couche identique à chaque fonctionnalité.
