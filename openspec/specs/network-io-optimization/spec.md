# Capability: Network IO Optimization

## Purpose
TBD - Optimization of network I/O operations for performance and resilience.

## Requirements

### Requirement: Lectures bufferisées pour SMB/NFS
Le système DOIT utiliser des tampons de lecture optimisés (typiquement 1 Mo) pour saturer la bande passante réseau, appliqués lors de la phase d'analyse et de copie de la commande `organize`.

#### Scenario: Lecture de gros fichier pendant organize
- **WHEN** la commande `organize` analyse un fichier volumineux sur un partage SMB/NFS
- **THEN** le système effectue des lectures séquentielles par blocs de 1 Mo pour minimiser les appels réseau et maximiser le débit

#### Scenario: Copie avec lecture bufferisée
- **WHEN** un fichier est copié vers la destination pendant organize
- **THEN** le système utilise des buffers de 1 Mo pour les opérations de lecture source

#### Scenario: Hash computation avec buffering optimisé
- **WHEN** la phase d'analyse (Analyzer) calcule le hash Blake3 d'un fichier réseau
- **THEN** le système lit le fichier par blocs bufferisés pour maximiser le parallélisme de hachage

### Requirement: Résilience aux coupures réseau
Le système DOIT implémenter un mécanisme de retry avec backoff exponentiel pour les opérations d'I/O réseau, appliqué à toutes les opérations de lecture/écriture sur SMB/NFS dans la commande `organize`.

#### Scenario: Micro-coupure réseau lors de analyze
- **WHEN** une opération de lecture échoue à cause d'un timeout réseau pendant l'analyse
- **THEN** le système retente l'opération après un délai croissant (ex: 100ms, 200ms, 400ms) avant de signaler une erreur fatale

#### Scenario: Résilience lors de la copie vers destination
- **WHEN** une opération de copie échoue à cause d'une erreur réseau temporaire
- **THEN** le système applique le backoff exponentiel et retente jusqu'à N fois avant de marquer le fichier comme échoué

#### Scenario: Límite de tentatives et log d'erreur
- **WHEN** une opération I/O échoue persistemment après max retries (typiquement 3-5)
- **THEN** le système log l'erreur avec contexte et continue le traitement des autres fichiers

#### Scenario: Absence de retry-storm sur réseau partagé
- **WHEN** plusieurs threads tentent de re-lire le même fichier réseau
- **WHEN** le backoff exponentiel décale les tentatives de manière pseudo-aléatoire
- **THEN** le système évite une thundering herd de requêtes simultanées
