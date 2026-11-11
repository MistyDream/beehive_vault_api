# Jeu de démonstration

Le jeu contient uniquement des données synthétiques en EUR. Ses dates sont fixes, du 31 juillet au 31 octobre 2026, et son foyer utilise `Europe/Paris`. Les dates sont donc utilisables avec l’API à partir du 31 octobre 2026. Le mois courant de la vue d’ensemble reste celui du calendrier réel. Choisir août, septembre ou octobre 2026 dans le rapport mensuel pour retrouver les valeurs ci-dessous.

## Charger et rejouer le jeu

Utiliser une base de développement ou de test isolée, par exemple `beehive_vault_demo`, puis définir explicitement sa connexion dans `DATABASE_URL`.

```bash
DATABASE_URL=postgres://beehive_vault@127.0.0.1:5432/beehive_vault_demo \
  cargo run --bin beehive_vault_admin -- demo load
```

La commande applique les migrations puis charge tout le jeu dans une transaction. Le foyer, les comptes et les écritures ont des UUID fixes. Deux chargements simultanés sont sérialisés. Une nouvelle exécution conserve le foyer existant, ses éventuelles modifications et les transactions importées. La sortie `changed=false` signifie que le foyer était déjà présent. Elle ne certifie pas que ses données correspondent encore au jeu initial.

Les données bancaires sont réinitialisées seulement par une demande explicite de remise à zéro.

```bash
DATABASE_URL=postgres://beehive_vault@127.0.0.1:5432/beehive_vault_demo \
  cargo run --bin beehive_vault_admin -- demo reset
```

La remise à zéro supprime le foyer de démonstration et toutes ses données, y compris les modifications ou imports faits dans ce foyer. Les deux établissements globaux de démonstration sont supprimés seulement lorsqu’aucun autre compte ne les utilise. Les autres foyers et les comptes qui réutilisent ces établissements sont préservés. Relancer `demo load` recrée le jeu initial avec les mêmes identifiants.

Les identifiants de démonstration utilisent le préfixe `019d9000-0000-7000-8000-`. La commande refuse de réutiliser un identifiant réservé de foyer ou d’établissement portant des attributs différents. Si le foyer a été renommé, rétablir son nom `Demo Household`, sa devise `EUR` et son fuseau `Europe/Paris` avant la remise à zéro.

## Comptes et résultats attendus

Le foyer `Demo Household` a l’identifiant `019d9000-0000-7000-8000-000000000001`. Il contient trois catégories explicites, `Salary`, `Housing` et `Groceries`, ainsi que des dépenses sans catégorie. Les établissements sont `Demo Everyday Bank` et `Demo Investment Bank`.

| Compte | Suffixe UUID | Rapprochement de référence | Solde courant avant import |
|---|---|---|---|
| Everyday account | `000000000015` | 2 010 EUR au 31/08/2026 | 3 945 EUR |
| Savings account | `000000000016` | 2 900 EUR au 30/09/2026 | 3 500 EUR |
| Investment account | `000000000017` | 3 300 EUR au 31/10/2026 | 3 300 EUR |
| Home loan | `000000000018` | 9 700 EUR au 31/10/2026 | 9 700 EUR dus |

Les soldes initiaux du 31 juillet sont respectivement 1 000, 2 000, 3 000 et 10 000 EUR. Les rapprochements suivants incluent déjà les opérations de leur journée et des journées antérieures. Le jeu contient 15 transactions ordinaires, dont trois remboursements et trois dépenses sans catégorie, et six transferts représentés par 12 mouvements.

Le résumé `GET /v1/households/{household_id}/summary` doit renvoyer 10 745 EUR d’actifs, 9 700 EUR de dettes et **1 045 EUR de patrimoine net**.

| Rapport mensuel | Revenus | Dépenses nettes des remboursements | Flux net |
|---|---|---|---|
| `2026-08` | 2 500 EUR | 990 EUR | 1 510 EUR |
| `2026-09` | 2 600 EUR | 1 025 EUR | 1 575 EUR |
| `2026-10` | 2 700 EUR | 1 040 EUR | 1 660 EUR |

Ces valeurs sont consultables par `GET /v1/households/{household_id}/monthly-flows/{month}`. Les virements d’épargne et remboursements du capital du prêt sont des transferts entre comptes du foyer et sont exclus des revenus et dépenses.

## Relevé à importer

Le fichier [`demo/statement.csv`](../demo/statement.csv) contient trois lignes supplémentaires pour `Everyday account`. Le chargement initial laisse ce relevé à importer pour permettre de jouer tout le parcours.

Configurer le séparateur `semicolon`, le format `day_month_year`, le séparateur décimal `comma` et les colonnes `Booking`, `Description`, `Amount`. Après la prévisualisation, confirmer les lignes 2 et 3 avec `nature=expense`, puis la ligne 4 avec `nature=income`. Utiliser `effect=standard` pour les trois lignes. Le [contrat CSV](api.md#confirmation-dun-relevé-csv) décrit les requêtes.

Après confirmation, le compte courant vaut **3 980 EUR**, les actifs **10 780 EUR**, les dettes **9 700 EUR** et le patrimoine net **1 080 EUR**. Pour octobre, les revenus valent **2 775,25 EUR**, les dépenses **1 080,25 EUR** et le flux net **1 695 EUR**. Les trois lignes sont initialement sans catégorie.

Réimporter le même fichier sur ce compte doit renvoyer trois résultats `duplicate` avec les mêmes identifiants. Les montants et les totaux restent identiques. Modifier les octets du fichier constitue un autre relevé selon le contrat actuel.

## Contrôles reproductibles

Le test d’intégration contrôle le chargement concurrent, la relance sans doublon, les résumés et rapports par les routes HTTP normales, l’import réel et sa déduplication, la remise à zéro, la préservation d’un autre foyer et d’un établissement partagé, ainsi que les collisions d’identifiants.

```bash
TEST_DATABASE_URL=postgres://beehive_vault@127.0.0.1:5432/beehive_vault_test \
  cargo test --test demo_dataset -- --ignored
```

Ce test utilise et nettoie les identifiants réservés du jeu dans la base de test. Pour vérifier une démonstration modifiée manuellement, consulter les routes ci-dessus et comparer les montants aux valeurs attendues avant ou après import.
