## ADDED Requirements

### Requirement: Extraction des métadonnées temporelles
Le système DOIT extraire la date de prise de vue la plus fiable à partir des métadonnées EXIF (priorité à `DateTimeOriginal`).

#### Scenario: Extraction EXIF réussie
- **WHEN** une photo contient le tag EXIF `DateTimeOriginal`
- **THEN** cette date est utilisée comme référence pour le tri

#### Scenario: Fallback sur la date système
- **WHEN** aucune métadonnée temporelle n'est trouvée dans le fichier
- **THEN** le système utilise la date de modification du fichier (`mtime`)

### Requirement: Structure de dossiers chronologique
Le système DOIT organiser les fichiers dans une hiérarchie de dossiers de type `/AAAA/MM/JJ/`.

#### Scenario: Création de la structure cible
- **WHEN** une photo prise le 15 Octobre 2023 est traitée
- **THEN** le fichier est placé dans le répertoire `2023/10/15/` sur la destination
