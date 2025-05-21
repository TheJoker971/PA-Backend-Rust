# 🧱 Real Estate Web3 API

API REST en Rust (Axum + SQLx) pour gérer une plateforme Web3 de gestion d'utilisateurs, propriétés immobilières tokenisées et investissements liés à la blockchain.

---

## 🚀 Technologies

- [Rust](https://www.rust-lang.org/)
- [Axum](https://docs.rs/axum/latest/axum/)
- [Tokio](https://tokio.rs/)
- [SQLx](https://docs.rs/sqlx/)
- [PostgreSQL (Neon)](https://neon.tech/)
- [dotenvy](https://docs.rs/dotenvy)
- [uuid](https://docs.rs/uuid)
- [chrono](https://docs.rs/chrono)

---

## 📁 Structure

.
├── src/
│ ├── main.rs # Démarrage serveur + DB init
│ ├── db.rs # Connexion à la DB PostgreSQL
│ ├── routes.rs # Définition des routes API
│ └── models.rs # Structs de données (User, Property, Investment)
├── migrations/
│ └── schema.sql # Script SQL de création des tables
├── .env # Variables d'environnement (DB, port)
├── Cargo.toml # Dépendances Rust
└── README.md # Ce fichier


---

## ⚙️ Variables d'environnement `.env`

```env
DATABASE_URL=postgres://<user>:<pass>@<host>/<dbname>?sslmode=require
PORT=3000
```


# Installer les dépendances Rust (si ce n'est pas fait)
rustup update
cargo install sqlx-cli

# Lancer le serveur
```bash
cargo run
```


# 🏗️ Installation
```bash
# Installer les dépendances Rust (si ce n'est pas fait)
rustup update
cargo install sqlx-cli

# Lancer le serveur
cargo run
```

# 🔧 Initialiser la base de données
Les tables sont créées automatiquement au démarrage (via le fichier migrations/schema.sql), ou vous pouvez les exécuter manuellement :

```bash
psql "$DATABASE_URL" -f migrations/schema.sql
```

# 📚 Routes API
## ✅ Health Check

```http
GET /health
```

## 👤 Utilisateurs
```http
POST /users
GET  /users
```

### Body JSON POST :

```json

{
  "wallet": "0xabc123...",
  "email": "bob@example.com",
  "name": "Bob",
  "role": "user"
}
```

## 🏘️ Propriétés
```http
POST /properties
GET  /properties
```

### Body JSON POST :

```json
{
  "onchain_id": 1,
  "name": "Villa Sunset",
  "description": "Maison tokenisée dans le sud",
  "image_url": "https://ipfs.io/ipfs/...",
  "category": "villa",
  "created_by": "uuid-user"
}
```

## 💸 Investissements
```http
POST /investments
GET  /investments
```

### Body JSON POST :

```json
{
  "user_id": "uuid-user",
  "property_id": "uuid-property",
  "amount_eth": 1.5,
  "shares": 10,
  "tx_hash": "0xhash"
}
```

## 📄 Licence

Ce projet est distribué sous la licence **GNU Affero General Public License v3.0 (AGPL-3.0)**.

Cela signifie que :
- Vous êtes libre d’utiliser, modifier et redistribuer ce projet.
- Toute modification ou utilisation sur un serveur (même sans distribution) doit également être rendue accessible sous la même licence.
- Le code source modifié doit être publié si vous fournissez ce service publiquement.

👉 [Consulter la licence complète](https://www.gnu.org/licenses/agpl-3.0.html)
