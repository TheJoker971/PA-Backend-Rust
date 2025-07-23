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

-- Supprimer les fonctions existantes si elles existent
DROP FUNCTION IF EXISTS get_user_role(TEXT);

-- Supprimer les types existants si ils existent
DROP TYPE IF EXISTS property_status CASCADE;
DROP TYPE IF EXISTS user_role CASCADE;

-- Créer l'enum pour les statuts de propriété
CREATE TYPE property_status AS ENUM ('pending', 'validated', 'rejected');

-- Créer l'enum pour les rôles utilisateur
CREATE TYPE user_role AS ENUM ('user', 'manager', 'admin');

-- Table users avec role intégré et enum strict
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet TEXT NOT NULL UNIQUE,
    name TEXT,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table properties avec le nouveau système de status
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
    status property_status NOT NULL DEFAULT 'pending',
    status_updated_at TIMESTAMPTZ,
    status_updated_by UUID REFERENCES users(id)
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

-- Fonction pour obtenir le rôle d'un utilisateur à partir de son wallet
CREATE OR REPLACE FUNCTION get_user_role(wallet_address TEXT)
RETURNS TEXT AS $$
DECLARE
    user_role TEXT;
BEGIN
    SELECT role INTO user_role FROM users
    WHERE wallet = wallet_address;
    
    RETURN COALESCE(user_role, 'user');
END;
$$ LANGUAGE plpgsql;

-- Activer Row Level Security (RLS)
ALTER TABLE properties ENABLE ROW LEVEL SECURITY;
ALTER TABLE investments ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- Politiques RLS pour properties
CREATE POLICY "Tous peuvent voir les propriétés validées" 
    ON properties FOR SELECT 
    USING (status = 'validated');

CREATE POLICY "Admin et manager peuvent voir toutes les propriétés" 
    ON properties FOR SELECT 
    TO authenticated
    USING (
        get_user_role(auth.jwt()->>'wallet') IN ('admin', 'manager')
    );

CREATE POLICY "Seuls admin et manager peuvent créer des propriétés" 
    ON properties FOR INSERT 
    TO authenticated
    WITH CHECK (
        get_user_role(auth.jwt()->>'wallet') IN ('admin', 'manager')
    );

CREATE POLICY "Seuls admin et manager peuvent modifier des propriétés" 
    ON properties FOR UPDATE 
    TO authenticated
    USING (
        get_user_role(auth.jwt()->>'wallet') IN ('admin', 'manager')
    );

CREATE POLICY "Seuls admin peuvent supprimer des propriétés" 
    ON properties FOR DELETE 
    TO authenticated
    USING (
        get_user_role(auth.jwt()->>'wallet') = 'admin'
    );

-- Politiques RLS pour investments
CREATE POLICY "Utilisateurs peuvent voir leurs propres investissements" 
    ON investments FOR SELECT 
    TO authenticated
    USING (
        user_id = auth.uid() OR
        get_user_role(auth.jwt()->>'wallet') IN ('admin', 'manager')
    );

CREATE POLICY "Utilisateurs peuvent créer leurs propres investissements" 
    ON investments FOR INSERT 
    TO authenticated
    WITH CHECK (
        user_id = auth.uid()
    );

-- Politiques RLS pour users
CREATE POLICY "Utilisateurs peuvent voir leur propre profil" 
    ON users FOR SELECT 
    TO authenticated
    USING (
        id = auth.uid() OR
        get_user_role(auth.jwt()->>'wallet') IN ('admin', 'manager')
    );

CREATE POLICY "Utilisateurs peuvent modifier leur propre profil" 
    ON users FOR UPDATE 
    TO authenticated
    USING (
        id = auth.uid() OR
        get_user_role(auth.jwt()->>'wallet') = 'admin'
    );

-- Création d'un utilisateur administrateur par défaut (à modifier avec vos propres valeurs)
INSERT INTO users (wallet, name, role) 
VALUES ('0xAdminWalletAddress', 'Admin', 'admin'); 