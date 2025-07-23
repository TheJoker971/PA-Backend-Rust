# Documentation de l'API

## Authentification

Toutes les routes de l'API, sauf `POST /auth/login`, `POST /auth/logout`, `GET /health` et `GET /properties/public`, nécessitent une authentification via un **Bearer Token** dans le header `Authorization`.

- **Header** : `Authorization`
- **Format** : `Bearer <adresse_wallet_utilisateur>`

### Exemple

```bash
curl -H "Authorization: Bearer VOTRE_ADRESSE_WALLET" [URL_DE_LA_ROUTE]
```

---

## Routes

### Authentification

#### `POST /auth/login`

Permet à un utilisateur de se connecter en fournissant son adresse wallet. Retourne les informations de l'utilisateur.

- **Méthode** : `POST`
- **Headers** : `Content-Type: application/json`
- **Body** :
  ```json
  {
    "wallet": "string"
  }
  ```
- **Réponse (200 OK)** :
  ```json
  {
    "id": "uuid",
    "wallet": "string",
    "name": "string",
    "role": "string ('user', 'manager', 'admin')",
    "created_at": "string (timestamp)"
  }
  ```

#### `POST /auth/logout`

Déconnecte l'utilisateur.

- **Méthode** : `POST`
- **Body** : Aucun
- **Réponse (200 OK)** :
  ```json
  {
    "message": "Déconnecté avec succès"
  }
  ```

### Santé de l'API

#### `GET /health`

Vérifie si l'API est en cours d'exécution.

- **Méthode** : `GET`
- **Body** : Aucun
- **Réponse (200 OK)** :
  ```json
  {
    "status": "ok",
    "message": "API is running"
  }
  ```

### Utilisateurs

#### `POST /users`

Crée un nouvel utilisateur.

- **Méthode** : `POST`
- **Headers** : `Content-Type: application/json`
- **Body** :
  ```json
  {
    "wallet": "string",
    "name": "string",
    "role": "string (optionnel, 'user' par défaut)"
  }
  ```
- **Réponse (201 Created)** :
  ```json
  {
    "id": "uuid",
    "message": "Utilisateur créé avec succès"
  }
  ```

#### Routes Admin

##### `GET /api/users`

Retourne la liste de tous les utilisateurs.

- **Méthode** : `GET`
- **Headers** : `Authorization: Bearer <wallet_admin>`
- **Body** : Aucun
- **Rôle requis** : `admin`

##### `PUT /api/users/:id/role`

Met à jour le rôle d'un utilisateur.

- **Méthode** : `PUT`
- **Headers** : `Authorization: Bearer <wallet_admin>`, `Content-Type: application/json`
- **URL Paramètre** : `id` (UUID de l'utilisateur)
- **Body** :
  ```json
  {
    "role": "string ('user', 'manager', 'admin')"
  }
  ```
- **Rôle requis** : `admin`
- **Restriction** : Un admin ne peut pas modifier son propre rôle.

### Propriétés (Properties)

#### Route Publique

##### `GET /properties/public`

Retourne la liste de toutes les propriétés dont le statut est **validé**.

- **Méthode** : `GET`
- **Body** : Aucun
- **Réponse (200 OK)** :
  ```json
  {
    "properties": [
      {
        "id": "uuid",
        "onchain_id": "string",
        "name": "string",
        "location": "string",
        "type": "string",
        "description": "string",
        "total_price": "number",
        "token_price": "number",
        "annual_yield": "number",
        "image_url": "string",
        "documents": ["string"],
        "created_at": "string (timestamp)"
      }
    ],
    "count": "integer",
    "message": "Propriétés validées uniquement"
  }
  ```

#### Routes Authentifiées

##### `GET /api/properties`

Retourne une liste de propriétés en fonction du rôle de l'utilisateur.

- **Méthode** : `GET`
- **Headers** : `Authorization: Bearer <wallet>`
- **Body** : Aucun
- **Comportement par rôle** :
  - `admin` : Voit toutes les propriétés, sans filtre.
  - `manager` : Ne voit que les propriétés qu'il a créées.
  - `user` : Ne voit que les propriétés dans lesquelles il a investi.

##### `POST /api/properties`

Crée une nouvelle propriété avec le statut `pending`.

- **Méthode** : `POST`
- **Headers** : `Authorization: Bearer <wallet>`, `Content-Type: application/json`
- **Body** :
  ```json
  {
    "onchain_id": "string",
    "name": "string",
    "location": "string",
    "property_type": "string",
    "description": "string (optionnel)",
    "total_price": "number",
    "token_price": "number",
    "annual_yield": "number",
    "image_url": "string (optionnel)",
    "documents": "array (optionnel)"
  }
  ```
- **Rôle requis** : `manager`, `admin`

##### `GET /api/properties/:id`

Retourne les détails d'une propriété spécifique.

- **Méthode** : `GET`
- **Headers** : `Authorization: Bearer <wallet>`
- **URL Paramètre** : `id` (UUID de la propriété)
- **Body** : Aucun
- **Rôle requis** : `user`, `manager`, `admin`

##### `PUT /api/properties/:id`

Met à jour une propriété.

- **Méthode** : `PUT`
- **Headers** : `Authorization: Bearer <wallet>`, `Content-Type: application/json`
- **URL Paramètre** : `id` (UUID de la propriété)
- **Body** : Identique à `POST /api/properties`
- **Rôle requis** : `manager`, `admin`
- **Restriction** : Un `manager` ne peut pas modifier une propriété si son statut est `validated`. Seul un `admin` le peut.

##### `PUT /api/properties/:id/status`

Met à jour le statut d'une propriété.

- **Méthode** : `PUT`
- **Headers** : `Authorization: Bearer <wallet>`, `Content-Type: application/json`
- **URL Paramètre** : `id` (UUID de la propriété)
- **Body** :
  ```json
  {
    "status": "string ('pending', 'validated', 'rejected')"
  }
  ```
- **Rôle requis** : `admin`

##### `DELETE /api/properties/:id`

Supprime une propriété.

- **Méthode** : `DELETE`
- **Headers** : `Authorization: Bearer <wallet>`
- **URL Paramètre** : `id` (UUID de la propriété)
- **Body** : Aucun
- **Rôle requis** : `admin`
- **Restriction** : Ne peut pas supprimer une propriété si son statut est `validated`.

### Investissements (Investments)

#### Routes Authentifiées

##### `GET /api/investments`

Retourne une liste d'investissements en fonction du rôle de l'utilisateur.

- **Méthode** : `GET`
- **Headers** : `Authorization: Bearer <wallet>`
- **Body** : Aucun
- **Comportement par rôle** :
  - `admin` : Voit tous les investissements.
  - `manager` : Voit les investissements liés aux propriétés qu'il a créées.
  - `user` : Voit uniquement ses propres investissements.
- **Réponse (200 OK)** :
  ```json
  {
    "investments": [
      {
        "id": "uuid",
        "user_id": "uuid",
        "property_id": "uuid",
        "amount_eth": "number",
        "shares": "integer",
        "tx_hash": "string",
        "created_at": "string (timestamp)"
      }
    ],
    "count": "integer"
  }
  ```

##### `POST /api/investments`

Crée un nouvel investissement.

- **Méthode** : `POST`
- **Headers** : `Authorization: Bearer <wallet>`, `Content-Type: application/json`
- **Body** :
  ```json
  {
    "property_id": "uuid",
    "amount_eth": "number",
    "shares": "integer",
    "tx_hash": "string"
  }
  ```
- **Rôle requis** : `user`, `manager`, `admin`
- **Restriction** : Seules les propriétés avec le statut `validated` peuvent recevoir des investissements.
- **Note** : Le `user_id` est automatiquement assigné à l'utilisateur authentifié.

##### `GET /api/investments/:id`

Retourne les détails d'un investissement spécifique.

- **Méthode** : `GET`
- **Headers** : `Authorization: Bearer <wallet>`
- **URL Paramètre** : `id` (UUID de l'investissement)
- **Body** : Aucun
- **Contrôle d'accès** :
  - `admin` : Peut voir n'importe quel investissement.
  - `manager` : Peut voir les investissements liés à ses propriétés.
  - `user` : Peut voir uniquement ses propres investissements.

##### `PUT /api/investments/:id`

Met à jour un investissement.

- **Méthode** : `PUT`
- **Headers** : `Authorization: Bearer <wallet>`, `Content-Type: application/json`
- **URL Paramètre** : `id` (UUID de l'investissement)
- **Body** :
  ```json
  {
    "amount_eth": "number",
    "shares": "integer",
    "tx_hash": "string"
  }
  ```
- **Contrôle d'accès** : Seul l'`admin` ou le propriétaire de l'investissement peut le modifier.

##### `DELETE /api/investments/:id`

Supprime un investissement.

- **Méthode** : `DELETE`
- **Headers** : `Authorization: Bearer <wallet>`
- **URL Paramètre** : `id` (UUID de l'investissement)
- **Body** : Aucun
- **Contrôle d'accès** : Seul l'`admin` ou le propriétaire de l'investissement peut le supprimer.

--- 