# Sift

**Sift** est un moteur de tri de photos ultra-léger et idempotent, conçu spécifiquement pour gérer des volumes massifs de données (To, millions de fichiers) sur des infrastructures de stockage réseau (SMB, NFS).

Développé en Rust pour une performance brute et une sécurité mémoire optimale, Sift automatise l'organisation de vos bibliothèques photographiques tout en garantissant l'absence de doublons et une empreinte système minimale.

## Points Forts

- **Performance Extrême :** Écrit en Rust, utilisant le hachage **Blake3** (parallélisé via SIMD) pour une intégrité des données ultra-rapide.
- **Idempotence Stricte :** Détection intelligente des doublons grâce à un index local persistant, évitant les re-traitements inutiles sur le réseau.
- **Organisation Intelligente :**
  - Hiérarchie chronologique précise (`/AAAA/MM/JJ/`).
  - **Clustering Géographique :** Regroupement automatique des photos par lieu via l'algorithme **DBSCAN**.
  - **Reverse Geocoding Offline :** Identification des noms de lieux sans dépendance au cloud (via GeoNames).
- **Optimisé pour le Réseau :** Stratégie de cache local et lectures bufferisées pour saturer la bande passante SMB/NFS sans latence excessive.
- **Zéro Dépendance :** Livré sous forme d'un binaire unique, sans besoin de Python, ExifTool ou base de données externe.

## Installation

```bash
# Exemple de compilation (nécessite Rust)
cargo build --release
```

## Utilisation Rapide

```bash
sift --source /chemin/vers/import --dest /chemin/vers/bibliotheque
```

## Philosophie

Sift est conçu pour les architectes systèmes et les photographes exigeants qui recherchent un outil "set and forget" capable de traiter des archives gigantesques avec la fiabilité d'un outil système de bas niveau.
