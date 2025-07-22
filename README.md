# API Backend Rust - Gestion Immobilière

Cette API REST est construite avec Rust (Axum) et utilise PostgreSQL pour la gestion de propriétés immobilières et d'investissements avec un système de rôles basé sur les signatures de wallet.

## 🚀 Fonctionnalités

- **Authentification Bearer Token** avec signatures de wallet
- **Gestion des rôles** : Admin, Manager, User avec permissions granulaires
- **Gestion des propriétés** : CRUD complet avec validation et contrôles de statut
- **Gestion des investissements** : Système d'investissement dans les propriétés validées
- **Sécurité avancée** : Protection des propriétés validées, contrôles d'accès par rôle
- **Routes publiques** : Accès aux propriétés validées sans authentification

## 🏗️ Architecture

### Rôles et Permissions

| Rôle | Properties | Investments | Permissions spéciales |
|------|------------|-------------|----------------------|
| **Admin** | Voit tout, peut tout modifier | Voit tout, peut tout modifier | Seul à pouvoir changer les statuts, supprimer les propriétés validées |
| **Manager** | Voit ses créations uniquement | Voit les investissements sur ses propriétés | Peut créer/modifier des propriétés (sauf validées) |
| **User** | Voit ses investissements | Voit/modifie ses investissements | Peut investir dans les propriétés validées |

### Statuts des Propriétés

- **`pending`** : En attente de validation (défaut à la création)
- **`validated`** : Validée par l'admin, peut recevoir des investissements, protégée contre les modifications
- **`rejected`** : Rejetée par l'admin

## 📋 Prérequis

- Rust 1.70+
- PostgreSQL 13+
- Base de données configurée avec les migrations

## ⚙️ Configuration

### 1. Variables d'environnement

Créez un fichier `.env` à la racine du projet :

```env
DATABASE_URL=postgres://username:password@localhost:5432/database_name
PORT=3000
```

### 2. Migration de la base de données

Exécutez le script de migration pour créer les tables :

```sql
-- Copiez et exécutez le contenu de migrations/supabase_migration.sql
-- dans votre base de données PostgreSQL
```

### 3. Création d'un utilisateur admin

```sql
INSERT INTO users (signature, name, role) 
VALUES ('0xVOTRE_SIGNATURE_ADMIN', 'Admin', 'admin');
```

## 🚀 Démarrage

```bash
# Installation des dépendances
cargo build

# Lancement du serveur de développement
cargo run

# Lancement avec logs détaillés
RUST_LOG=debug cargo run
```

Le serveur sera accessible à `http://localhost:3000`

## 📚 Documentation de l'API

### Authentification

Toutes les routes protégées nécessitent un header `Authorization` :

```bash
Authorization: Bearer <signature_wallet>
```

### Routes disponibles

#### 🔓 Routes Publiques

- `GET /health` - Santé de l'API
- `GET /properties/public` - Liste des propriétés validées
- `POST /users` - Création d'utilisateur
- `POST /auth/login` - Connexion (retourne les infos utilisateur)
- `POST /auth/logout` - Déconnexion

#### 🔐 Routes Protégées (Bearer Token requis)

##### Propriétés
- `GET /api/properties` - Liste filtrée par rôle
- `POST /api/properties` - Créer (Manager/Admin)
- `GET /api/properties/:id` - Détail
- `PUT /api/properties/:id` - Modifier (Manager/Admin, sauf validées)
- `PUT /api/properties/:id/status` - Changer statut (Admin uniquement)
- `DELETE /api/properties/:id` - Supprimer (Admin, sauf validées)

##### Investissements
- `GET /api/investments` - Liste filtrée par rôle
- `POST /api/investments` - Créer (propriétés validées uniquement)
- `GET /api/investments/:id` - Détail
- `PUT /api/investments/:id` - Modifier (Admin/Propriétaire)
- `DELETE /api/investments/:id` - Supprimer (Admin/Propriétaire)

## 🔧 Exemples d'utilisation

### Créer une propriété (Manager/Admin)

```bash
curl -X POST http://localhost:3000/api/properties \
  -H "Authorization: Bearer 0xVOTRE_SIGNATURE" \
  -H "Content-Type: application/json" \
  -d '{
    "onchain_id": "prop_001",
    "name": "Appartement Paris 16e",
    "location": "Paris, France",
    "property_type": "Appartement",
    "description": "Bel appartement 3 pièces",
    "total_price": 500000,
    "token_price": 100,
    "annual_yield": 4.5
  }'
```

### Valider une propriété (Admin uniquement)

```bash
curl -X PUT http://localhost:3000/api/properties/{property_id}/status \
  -H "Authorization: Bearer 0xSIGNATURE_ADMIN" \
  -H "Content-Type: application/json" \
  -d '{"status": "validated"}'
```

### Investir dans une propriété

```bash
curl -X POST http://localhost:3000/api/investments \
  -H "Authorization: Bearer 0xVOTRE_SIGNATURE" \
  -H "Content-Type: application/json" \
  -d '{
    "property_id": "uuid-de-la-propriete",
    "amount_eth": 1.5,
    "shares": 15,
    "tx_hash": "0x..."
  }'
```

## 📁 Structure du projet

```
src/
├── main.rs          # Point d'entrée et configuration des routes
├── auth.rs          # Authentification Bearer Token et gestion des rôles
├── models.rs        # Modèles de données (Property, Investment, User)
├── routes.rs        # Handlers des routes API
└── db.rs           # Configuration base de données

migrations/
├── schema.sql           # Schéma initial
└── supabase_migration.sql # Migration pour Supabase

API_DOCUMENTATION.md     # Documentation détaillée de l'API
```

## 🛡️ Sécurité

### Contrôles d'accès
- **Authentification Bearer Token** obligatoire pour toutes les routes sensibles
- **Vérification des rôles** sur chaque endpoint
- **Protection des propriétés validées** contre les modifications non autorisées
- **Isolation des données** selon le rôle (users ne voient que leurs investissements)

### Règles métier
- Seul l'admin peut modifier les statuts des propriétés
- Les propriétés validées ne peuvent plus être modifiées (sauf par l'admin)
- Les propriétés validées ne peuvent pas être supprimées
- Impossible d'investir dans une propriété non validée

## 📖 Documentation complète

Pour une documentation détaillée de toutes les routes avec exemples de requêtes et réponses, consultez [`API_DOCUMENTATION.md`](./API_DOCUMENTATION.md).

## 🤝 Contribution

1. Fork le projet
2. Créez une branche pour votre fonctionnalité
3. Committez vos changements
4. Poussez vers la branche
5. Ouvrez une Pull Request

## 📄 Licence

Ce projet est sous licence MIT. Voir le fichier `LICENSE` pour plus de détails.
