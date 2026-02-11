## MODIFIED Requirements

### Requirement: Extraction des métadonnées temporelles
Le système DOIT extraire la date de prise de vue la plus fiable à partir des métadonnées EXIF (priorité à `DateTimeOriginal`) lors de la pipeline du command `organize`.

#### Scenario: Extraction EXIF réussie dans organize
- **WHEN** une photo contient le tag EXIF `DateTimeOriginal` pendant l'analyse par la commande organize
- **THEN** cette date est extraite et utilisée comme référence pour le tri chronologique

#### Scenario: Fallback sur CreateDate lors de organize
- **WHEN** un fichier manque `DateTimeOriginal` mais contient `CreateDate`
- **THEN** le système utilise `CreateDate` comme date de référence

#### Scenario: Fallback sur le nom du fichier lors de organize
- **WHEN** aucune métadonnée EXIF n'est trouvée mais le fichier contient un pattern `YYYYMMDD` dans son nom
- **THEN** le système extrait la date du nom du fichier

#### Scenario: Fallback sur la date système lors de organize
- **WHEN** aucune métadonnée temporelle n'est trouvée dans le fichier
- **THEN** le système utilise la date de modification du fichier (`mtime`)

### Requirement: Structure de dossiers chronologique
Le système DOIT organiser les fichiers dans une hiérarchie de dossiers de type `/AAAA/MM/JJ/` lors de la commande `organize`.

#### Scenario: Création de la structure cible dans organize
- **WHEN** une photo prise le 15 Octobre 2024 est traitée par la commande organize
- **THEN** le fichier est placé dans le répertoire `2024/10/15/` sur la destination

#### Scenario: Création des répertoires parents lors de organize
- **WHEN** la structure chronologique n'existe pas encore
- **THEN** le système crée les répertoires parents (années, mois, jours) au besoin

#### Scenario: Réutilisation de répertoires existants
- **WHEN** un fichier avec la même date existe déjà (d'une exécution précédente de organize)
- **THEN** le système place le nouveau fichier dans le même répertoire chronologique sans le supprimer
