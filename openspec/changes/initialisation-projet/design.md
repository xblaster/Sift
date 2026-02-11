## Context

Sift est un nouveau projet visant à organiser des bibliothèques de photos massives sur stockage réseau. Le système doit être extrêmement rapide, léger et ne pas dépendre de services cloud ou de bases de données externes lourdes.

## Goals / Non-Goals

**Goals:**
- Performance brute pour le hachage et l'I/O réseau.
- Idempotence garantie via un index local persistant.
- Organisation chronologique et géographique automatisée.
- Binaire unique sans dépendances de runtime.

**Non-Goals:**
- Pas d'interface graphique (GUI) dans cette phase.
- Pas de modification des fichiers sources (lecture seule de la source).
- Pas de reconnaissance faciale ou d'analyse de contenu par IA.

## Decisions

### Langage de programmation : Rust
- **Rationale** : Sécurité mémoire sans Garbage Collector, performance comparable au C++, et excellent support de la concurrence (Rayon, Tokio).
- **Alternatives** : Go (rejeté à cause du runtime/GC), Python (rejeté pour les performances et les dépendances).

### Algorithme de hachage : Blake3
- **Rationale** : Plus rapide que SHA-256/MD5 grâce à sa parallélisation native via SIMD.
- **Alternatives** : SHA-256 (plus lent), MD5 (risques de collisions, moins performant sur CPU moderne).

### Gestion de l'index : HashMap en mémoire + Sériallisation Bincode
- **Rationale** : Chargement intégral au démarrage pour des accès $O(1)$ sans latence réseau (contrairement à SQLite sur SMB/NFS). Bincode offre une sériallisation binaire compacte et rapide.
- **Alternatives** : SQLite (problèmes de verrous sur réseau), JSON (trop lent pour des millions d'entrées).

### Clustering Géographique : DBSCAN avec Haversine
- **Rationale** : Détecte des clusters de forme arbitraire sans connaître le nombre de clusters à l'avance. La métrique Haversine est nécessaire pour la précision sur une sphère.
- **Alternatives** : K-Means (nécessite de connaître $K$).

## Risks / Trade-offs

- [Risk] Latence réseau extrême sur SMB → [Mitigation] Utilisation de buffers de lecture larges (1 Mo) et parallélisation des requêtes de métadonnées.
- [Risk] Consommation mémoire pour l'index → [Mitigation] Pour 1 million de fichiers, l'index occupera ~100-200 Mo, ce qui reste acceptable pour un utilitaire système.
