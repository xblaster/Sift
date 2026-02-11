## MODIFIED Requirements

### Requirement: Hachage Blake3 des fichiers
Le système DOIT calculer une empreinte unique pour chaque fichier source en utilisant l'algorithme Blake3, intégré dans la pipeline du command `organize`.

#### Scenario: Calcul de hash réussi dans la pipeline organize
- **WHEN** la commande `organize` traite un fichier source
- **THEN** le système génère un hash Blake3 unique basé sur le contenu intégral du fichier en parallèle via Rayon

#### Scenario: Hash utilisé pour déduplication dans organize
- **WHEN** un fichier est haché et son hash est vérifié contre l'index chargé
- **THEN** si le hash existe déjà, le fichier est marqué comme doublon et ignoré par les étapes suivantes

### Requirement: Indexation locale pour l'idempotence
Le système DOIT maintenir un index local des fichiers déjà traités pour éviter les doublons, chargé au démarrage de la commande `organize` et sauvegardé atomiquement à la fin.

#### Scenario: Chargement de l'index au démarrage de organize
- **WHEN** la commande `organize` est exécutée
- **THEN** le système charge l'index existant de la destination dans une `HashMap` en mémoire

#### Scenario: Détection de doublon dans organize
- **WHEN** un fichier avec un hash déjà présent dans l'index est rencontré pendant la pipeline organize
- **THEN** le système ignore le fichier et ne procède à aucune copie ou déplacement

#### Scenario: Mise à jour de l'index après organize
- **WHEN** la commande `organize` complète son traitement avec succès
- **THEN** son hash de tous les fichiers traités est ajouté à l'index local et sauvegardé de manière persistante et atomique via renommage de fichier

#### Scenario: Réexécution de organize avec idempotence
- **WHEN** la commande `organize` est exécutée une deuxième fois sur la même source
- **THEN** le système détecte tous les fichiers de la première exécution comme doublons et produit une structure identique
