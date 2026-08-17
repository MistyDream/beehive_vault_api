# Note — Chantiers de documentation

Cette note conserve les améliorations identifiées pendant la reconstruction du
MVP. Elle ne remplace ni la feuille de route produit ni les ADR.

## Contrat OpenAPI

- choisir une intégration OpenAPI compatible avec Axum ;
- générer le schéma au plus près des routes et DTO afin d'éviter une seconde
  description manuelle du contrat ;
- documenter les corps de requête, réponses, paramètres, statuts et erreurs ;
- représenter correctement les montants décimaux transmis sous forme de chaînes ;
- documenter le comportement des champs facultatifs et nullable dans les
  requêtes `PATCH` ;
- décider si une interface interactive doit être exposée uniquement en
  développement ;
- conserver `docs/api.md` comme vue d'ensemble et point d'entrée vers le schéma
  généré.

## Documentation à resynchroniser

- distinguer dans le modèle métier les invariants déjà implémentés de ceux
  prévus pour l'import CSV, notamment la référence d'import et la clé de
  déduplication ;
- mettre à jour l'arborescence de l'architecture avec les modules `categories`
  et `transactions` ;
- mettre à jour l'état du projet dans le README.

## Position de sécurité actuelle

Créer une courte documentation qui précise :

- que l'API ne possède actuellement aucune authentification ni autorisation ;
- qu'elle est destinée à un usage local et ne doit pas être exposée publiquement
  dans cet état ;
- que les fichiers `.env`, URL PostgreSQL et futures données d'import sont
  sensibles ;
- que les données financières et secrets ne doivent pas apparaître dans les
  journaux applicatifs ;
- que l'authentification, l'autorisation par foyer, la configuration CORS et la
  terminaison TLS devront être définies avant un accès distant ou familial.

## Avant un usage réel durable

- documenter la sauvegarde et la restauration de PostgreSQL ;
- documenter le déploiement et la gestion des secrets ;
- définir la journalisation, les métriques et la supervision nécessaires ;
- documenter la stratégie de migration et de reprise des données ;
- préparer des données de démonstration non sensibles pour vérifier le parcours
  complet.
