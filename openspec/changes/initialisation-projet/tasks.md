## 1. Setup & Core Engine

- [x] 1.1 Initialiser le projet Rust avec Cargo
- [x] 1.2 Ajouter les dépendances de base (blake3, bincode, serde, walkdir, rayon)
- [x] 1.3 Implémenter le calcul de hash Blake3 parallélisé
- [x] 1.4 Créer la structure de données de l'index et sa persistance binaire

## 2. Métadonnées & Organisation Chronologique

- [x] 2.1 Intégrer une bibliothèque de lecture EXIF (ex: kamadak-exif)
- [x] 2.2 Implémenter l'extraction de date avec fallback sur mtime
- [x] 2.3 Créer le module de gestion de la structure de dossiers /AAAA/MM/JJ/

## 3. Clustering Géographique

- [x] 3.1 Implémenter l'algorithme DBSCAN avec la métrique Haversine
- [x] 3.2 Intégrer les données GeoNames (cities1000.txt) dans le binaire
- [x] 3.3 Implémenter la recherche de lieu le plus proche (Reverse Geocoding Offline)

## 4. Optimisation & CLI

- [x] 4.1 Implémenter le buffering de lecture pour les fichiers distants
- [x] 4.2 Ajouter la gestion des erreurs réseau avec retries
- [x] 4.3 Créer l'interface CLI avec Clap (source, destination, options)
- [x] 4.4 Effectuer des tests de performance sur un montage SMB/NFS
