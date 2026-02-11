# Architecture Technique - Sift

Ce document détaille les choix de conception et l'architecture interne du moteur de tri Sift.

## 1. Choix du Langage : Rust

Le choix de **Rust** est dicté par trois impératifs :
- **Absence de Garbage Collector :** Garantit une consommation mémoire prévisible et constante lors du traitement de millions de fichiers.
- **Concurrence Sans Peur :** Utilisation de `Rayon` pour paralléliser le hachage et l'analyse EXIF sans risque de race conditions.
- **Performance I/O :** Contrôle granulaire sur les appels système et les buffers de lecture, crucial pour les protocoles SMB/NFS.

## 2. Pipeline de Traitement

Sift fonctionne comme un pipeline asynchrone composé de quatre étages :

1. **Walker :** Parcours récursif multi-threadé de la source (basé sur `walkdir`).
2. **Analyzer :** Extraction des métadonnées EXIF et calcul du hash **Blake3**. Blake3 a été choisi pour sa structure en arbre de Merkle permettant d'exploiter les instructions SIMD du CPU.
3. **Clusterer :** Regroupement spatial des photos.
4. **Writer :** Gestion des copies/déplacements vers la destination avec mécanismes de retry pour la résilience réseau.

## 3. Stratégie d'Idempotence et Indexation

Pour éviter de dépendre d'une base de données lourde (SQLite) peu performante sur réseau :
- **Index Local :** Un fichier binaire (sériallisation `Bincode`) est chargé intégralement en mémoire dans une `HashMap` au démarrage.
- **Vérification Flash :** La détection des doublons se fait en mémoire locale ($O(1)$), minimisant les accès réseau.
- **Mise à jour Atomique :** L'index est réécrit sur la destination uniquement en fin de processus ou par blocs, via un renommage atomique.

## 4. Organisation Spatiale et Temporelle

### Extraction de Dates
Sift suit une priorité stricte pour déterminer la date de prise de vue :
1. EXIF `DateTimeOriginal`
2. EXIF `CreateDate`
3. Regex sur le nom du fichier (`YYYYMMDD`)
4. Date de modification système (`mtime`)

### Clustering Géographique (DBSCAN)
L'algorithme **DBSCAN** est utilisé avec la métrique de **Haversine** pour regrouper les photos :
- **Epsilon ($\epsilon$) :** ~1km pour définir un voisinage.
- **MinPts :** 3-5 photos pour valider la création d'un sous-dossier de lieu.
- **Reverse Geocoding :** Utilisation de la base `cities1000.txt` de GeoNames embarquée, permettant une résolution de nom de ville hors ligne via un `k-d tree`.

## 5. Optimisations Réseau (SMB/NFS)

- **Buffered Reads :** Utilisation de tampons de 1 Mo pour optimiser le débit binaire.
- **Metadata Caching :** Phase d'inventaire initiale séparée pour réduire les allers-retours de métadonnées.
- **Résilience :** Backoff exponentiel sur les erreurs d'I/O pour supporter les micro-coupures réseau.

## 6. Comparaison avec l'Existant

| Caractéristique | Sift | Elodie / Phockup | PhotoSort |
| :--- | :--- | :--- | :--- |
| **Langage** | Rust | Python | Rust |
| **Hachage** | Blake3 (Rapide) | SHA/MD5 | MD5 |
| **Index** | HashMap Local | JSON/Aucun | Aucun |
| **Géo Offline** | Oui (DBSCAN) | Partiel | Non |
| **Binaire Unique**| Oui | Non | Oui |
