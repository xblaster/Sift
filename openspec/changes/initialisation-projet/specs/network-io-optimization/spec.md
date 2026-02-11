## ADDED Requirements

### Requirement: Lectures bufferisées pour SMB/NFS
Le système DOIT utiliser des tampons de lecture optimisés (typiquement 1 Mo) pour saturer la bande passante réseau.

#### Scenario: Lecture de gros fichier
- **WHEN** un fichier volumineux est lu sur un partage SMB
- **THEN** le système effectue des lectures séquentielles par blocs de 1 Mo pour minimiser les appels réseau

### Requirement: Résilience aux coupures réseau
Le système DOIT implémenter un mécanisme de retry avec backoff exponentiel pour les opérations d'I/O réseau.

#### Scenario: Micro-coupure réseau
- **WHEN** une opération de lecture échoue à cause d'un timeout réseau
- **THEN** le système retente l'opération après un délai croissant avant de signaler une erreur fatale
