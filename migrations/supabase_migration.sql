-- Script de migration pour Supabase
-- À exécuter dans l'interface SQL de Supabase ou via l'API

-- Extension pour UUID
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Activer l'extension pgcrypto pour les fonctions de hachage
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Supprimer les tables existantes si présentes (ordre dépendant des contraintes)
DROP TABLE IF EXISTS sessions CASCADE;
DROP TABLE IF EXISTS investments CASCADE;
DROP TABLE IF EXISTS properties CASCADE;
DROP TABLE IF EXISTS roles CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- Table users avec role intégré
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signature TEXT NOT NULL UNIQUE,
    name TEXT,
    role TEXT NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table properties
CREATE TABLE properties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    onchain_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    location TEXT NOT NULL,
    type TEXT NOT NULL,
    description TEXT,
    total_price NUMERIC NOT NULL,
    token_price NUMERIC NOT NULL,
    annual_yield NUMERIC NOT NULL,
    image_url TEXT,
    documents TEXT[],
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_validated BOOLEAN NOT NULL DEFAULT FALSE,
    validated_at TIMESTAMPTZ,
    validated_by UUID REFERENCES users(id)
);

-- Table investments
CREATE TABLE investments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    property_id UUID NOT NULL REFERENCES properties(id),
    amount_eth NUMERIC NOT NULL,
    shares INTEGER NOT NULL,
    tx_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table sessions
CREATE TABLE sessions (
    token UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);

-- Création d'un utilisateur administrateur par défaut (à modifier avec vos propres valeurs)
INSERT INTO users (signature, name, role) 
VALUES ('0xAdminSignature', 'Admin', 'admin'); 