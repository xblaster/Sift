## Why

La gestion de volumes massifs de photos (To, millions de fichiers) sur des stockages réseau (SMB/NFS) nécessite une solution haute performance, ultra-légère et résiliente à la latence réseau. Sift répond à ce besoin en offrant un moteur de tri idempotent sans dépendance externe.

## What Changes

- Création d'un utilitaire CLI en Rust pour une performance et une sécurité mémoire optimales.
- Implémentation du hachage Blake3 pour une détection de doublons ultra-rapide.
- Organisation automatique des photos en hiérarchie chronologique (`/AAAA/MM/JJ/`).
- Regroupement géographique des photos via l'algorithme DBSCAN.
- Reverse geocoding offline utilisant les données GeoNames.
- Gestion d'un index local persistant pour garantir l'idempotence sur stockage distant.

## Capabilities

### New Capabilities
- `core-deduplication`: Moteur de hachage Blake3 et gestion de l'index local pour l'idempotence.
- `chronological-organization`: Extraction des métadonnées EXIF et création de la structure de dossiers par date.
- `geographic-clustering`: Algorithme DBSCAN pour le regroupement spatial et reverse geocoding offline.
- `network-io-optimization`: Stratégie de cache et buffers de lecture optimisés pour SMB/NFS.

### Modified Capabilities
<!-- Aucune capacité existante n'est modifiée car il s'agit de l'initialisation du projet. -->

## Impact

- Création d'une nouvelle base de code en Rust.
- Optimisation des flux de données sur les montages réseau.
- Autonomie totale sans dépendance à des API cloud ou des runtimes lourds (Python).
