## ADDED Requirements

### Requirement: Hachage Blake3 des fichiers
Le système DOIT calculer une empreinte unique pour chaque fichier source en utilisant l'algorithme Blake3.

#### Scenario: Calcul de hash réussi
- **WHEN** un fichier est lu par le moteur
- **THEN** le système génère un hash Blake3 unique basé sur le contenu intégral du fichier

### Requirement: Indexation locale pour l'idempotence
Le système DOIT maintenir un index local des fichiers déjà traités pour éviter les doublons.

#### Scenario: Détection de doublon existant
- **WHEN** un fichier avec un hash déjà présent dans l'index est rencontré
- **THEN** le système ignore le fichier et ne procède à aucune copie ou déplacement

#### Scenario: Mise à jour de l'index
- **WHEN** un nouveau fichier est traité avec succès
- **THEN** son hash est ajouté à l'index local et sauvegardé de manière persistante
