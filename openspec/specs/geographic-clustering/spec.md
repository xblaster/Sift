# Capability: Geographic Clustering

## Purpose
TBD - Grouping of photos based on geographic location using DBSCAN and GeoNames.

## Requirements

### Requirement: Regroupement spatial via DBSCAN
Le système DOIT identifier des clusters de photos prises dans une même zone géographique en utilisant l'algorithme DBSCAN, optionnellement activé dans la commande `organize` via le flag `--with-clustering`.

#### Scenario: Clustering désactivé (comportement par défaut)
- **WHEN** la commande `organize` est exécutée sans le flag `--with-clustering`
- **THEN** le système ignore le clustering spatial et organise uniquement par date chronologique

#### Scenario: Clustering activé lors de organize
- **WHEN** la commande `organize` est exécutée avec `--with-clustering` et les photos contiennent des coordonnées GPS
- **THEN** le système identifie les clusters de photos dans un rayon de 1km (ε ≈ 1km, MinPts = 3-5)

#### Scenario: Identification d'un cluster dans organize
- **WHEN** un groupe de plus de 3 photos est détecté dans un rayon de 1km avec le clustering activé
- **THEN** ces photos sont marquées comme appartenant à un même cluster géographique

#### Scenario: Photos sans GPS lors du clustering
- **WHEN** une photo manque de coordonnées GPS durant le clustering
- **THEN** le système la place dans le dossier chronologique sans sous-dossier géographique

### Requirement: Reverse Geocoding Offline
Le système DOIT attribuer un nom de lieu à chaque cluster en utilisant une base de données GeoNames locale, lorsque le clustering est activé dans `organize`.

#### Scenario: Nommage de dossier par lieu lors de organize
- **WHEN** un cluster est identifié près de "San Francisco" avec clustering activé
- **THEN** un sous-dossier nommé "San_Francisco" est créé à l'intérieur de la structure chronologique `{dest}/YYYY/MM/DD/San_Francisco/`

#### Scenario: Reverse geocoding échoue, utilisation de coordonnées
- **WHEN** le reverse geocoding ne peut pas identifier le nom de la ville
- **THEN** le système utilise les coordonnées GPS (lat/lon) ou un identifiant générique pour le sous-dossier

#### Scenario: Cohérence des noms de lieux à travers les exécutions
- **WHEN** la commande `organize` est exécutée plusieurs fois avec le même clustering
- **THEN** les mêmes clusters sont nommés de manière cohérente, assurant l'idempotence
