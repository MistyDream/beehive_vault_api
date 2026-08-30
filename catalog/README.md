# Catalogue d'établissements

`institutions.json` fournit le catalogue initial d'une installation française.
Il est appliqué explicitement avec le binaire d'administration décrit dans
`docs/development.md`.

L'import est additif et réentrant. Les noms sont nettoyés et comparés sans tenir
compte de la casse. Une entrée déjà présente reste inchangée, tandis qu'une
entrée absente reçoit un nouvel UUID stable. Retirer ou renommer une entrée dans
ce fichier ne modifie pas la base. La commande explicite de renommage couvre ce
dernier besoin.

La base reste la source de vérité après l'import. L'API n'importe pas ce fichier
automatiquement au démarrage et l'outil d'administration ne supprime aucun
établissement.
