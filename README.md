# API Backend Rust avec Supabase

Cette API est construite avec Rust (Axum) et utilise Supabase comme base de données PostgreSQL.

## Fonctionnalités

- Système de gestion des rôles basé sur les adresses de wallet
- Gestion des propriétés immobilières avec validation par les administrateurs
- Gestion des investissements
- Authentification sécurisée

## Configuration de Supabase

1. Créez un compte sur [Supabase](https://supabase.com/) si ce n'est pas déjà fait
2. Créez un nouveau projet
3. Notez les informations de connexion à la base de données dans la section "Settings > Database"

## Migration de la base de données

1. Accédez à l'interface SQL de Supabase (Table Editor > SQL)
2. Copiez le contenu du fichier `migrations/supabase_migration.sql`
3. Collez-le dans l'éditeur SQL de Supabase et exécutez-le
4. Modifiez les valeurs par défaut pour l'utilisateur administrateur dans le script

## Configuration de l'environnement

1. Créez un fichier `.env` à la racine du projet en vous basant sur `.env.example`
2. Remplissez la variable `DATABASE_URL` avec l'URL de connexion à votre base de données Supabase

```
DATABASE_URL=postgres://postgres:your-password-here@db.xxxxxxxxxxxx.supabase.co:5432/postgres
PORT=3000
```

## Exécution de l'application

```bash
# Installation des dépendances
cargo build

# Lancement du serveur
cargo run
```

Le serveur sera accessible à l'adresse `http://localhost:3000`.

## Structure du projet

- `src/main.rs` : Point d'entrée de l'application
- `src/db.rs` : Configuration de la connexion à la base de données
- `src/models.rs` : Modèles de données
- `src/routes.rs` : Définition des routes de l'API
- `src/auth.rs` : Gestion de l'authentification et des rôles

## API Endpoints

### Authentification

- `POST /auth/login` : Connexion avec email/mot de passe
- `POST /auth/logout` : Déconnexion

### Utilisateurs

- `POST /users` : Création d'un utilisateur
- `GET /users` : Liste des utilisateurs (admin/manager uniquement)

### Rôles

- `POST /roles` : Création d'un rôle (admin uniquement)
- `GET /roles` : Liste des rôles (admin uniquement)
- `DELETE /roles/:role_id` : Suppression d'un rôle (admin uniquement)
- `GET /roles/wallet/:wallet` : Récupération du rôle d'un wallet

### Propriétés

- `GET /properties` : Liste des propriétés validées (public)
- `GET /properties/:property_id` : Détails d'une propriété validée (public)
- `GET /properties/all` : Liste de toutes les propriétés (admin/manager uniquement)
- `GET /properties/admin/:property_id` : Détails d'une propriété (admin/manager uniquement)
- `POST /properties` : Création d'une propriété (admin/manager uniquement)
- `PUT /properties/:property_id/validate` : Validation d'une propriété (admin uniquement)

### Investissements

- `POST /investments` : Création d'un investissement (uniquement pour les propriétés validées)
- `GET /investments` : Liste des investissements de l'utilisateur connecté
- `GET /investments/user/:user_id` : Liste des investissements d'un utilisateur spécifique

## Gestion des rôles

Le système utilise les 8 premiers caractères de l'adresse du wallet pour associer un rôle à un utilisateur. Les rôles disponibles sont :

- `admin` : Accès complet à toutes les fonctionnalités
- `manager` : Peut créer et gérer des propriétés, mais ne peut pas valider
- `investor` : Peut investir dans des propriétés validées
- `user` : Accès limité (par défaut)

## Sécurité

L'API utilise :
- Cookies sécurisés pour les sessions
- Protection CSRF
- Row Level Security (RLS) dans Supabase
- Hachage des mots de passe avec bcrypt
