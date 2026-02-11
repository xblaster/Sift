## ADDED Requirements

### Requirement: Regroupement spatial via DBSCAN
Le système DOIT identifier des clusters de photos prises dans une même zone géographique en utilisant l'algorithme DBSCAN.

#### Scenario: Identification d'un cluster
- **WHEN** un groupe de plus de 3 photos est détecté dans un rayon de 1km
- **THEN** ces photos sont marquées comme appartenant à un même cluster géographique

### Requirement: Reverse Geocoding Offline
Le système DOIT attribuer un nom de lieu à chaque cluster en utilisant une base de données GeoNames locale.

#### Scenario: Nommage de dossier par lieu
- **WHEN** un cluster est identifié près de "Paris"
- **THEN** un sous-dossier nommé "Paris" est créé à l'intérieur de la structure chronologique
