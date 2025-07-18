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

-- Table users (avec hash de mot de passe)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet TEXT NOT NULL UNIQUE,
    wallet_short TEXT GENERATED ALWAYS AS (substring(wallet, 1, 8)) STORED,
    email TEXT UNIQUE,
    name TEXT,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Table roles (nouvelle table pour la gestion des rôles)
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_short TEXT NOT NULL UNIQUE,
    role TEXT CHECK (role IN ('admin', 'manager', 'investor', 'user')) NOT NULL DEFAULT 'user',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Table properties
CREATE TABLE properties (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    onchain_id INT NOT NULL,
    name TEXT NOT NULL,
    location TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('Residential', 'Commercial', 'Industrial')),
    description TEXT,
    total_price DECIMAL,
    token_price DECIMAL,
    annual_yield DECIMAL,
    image_url TEXT,
    documents JSONB, -- Liste de URLs de documents
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    is_validated BOOLEAN NOT NULL DEFAULT FALSE,
    validated_at TIMESTAMP,
    validated_by UUID REFERENCES users(id) ON DELETE SET NULL
);

-- Table investments
CREATE TABLE investments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    property_id UUID REFERENCES properties(id) ON DELETE CASCADE,
    amount_eth DOUBLE PRECISION NOT NULL,
    shares INT NOT NULL,
    tx_hash TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Table sessions
CREATE TABLE sessions (
    token UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);

-- Trigger pour mettre à jour le champ updated_at dans la table roles
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_roles_updated_at
BEFORE UPDATE ON roles
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Fonction pour obtenir le rôle d'un utilisateur à partir de son wallet
CREATE OR REPLACE FUNCTION get_user_role(wallet_address TEXT)
RETURNS TEXT AS $$
DECLARE
    user_role TEXT;
BEGIN
    SELECT role INTO user_role FROM roles
    WHERE wallet_short = substring(wallet_address, 1, 8);
    
    RETURN COALESCE(user_role, 'user');
END;
$$ LANGUAGE plpgsql;

-- Activer Row Level Security (RLS)
ALTER TABLE properties ENABLE ROW LEVEL SECURITY;
ALTER TABLE investments ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE roles ENABLE ROW LEVEL SECURITY;

-- Politiques RLS pour properties
CREATE POLICY "Tous peuvent voir les propriétés validées" 
    ON properties FOR SELECT 
    USING (is_validated = true);

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

-- Politiques RLS pour roles
CREATE POLICY "Seuls admin peuvent gérer les rôles" 
    ON roles FOR ALL 
    TO authenticated
    USING (
        get_user_role(auth.jwt()->>'wallet') = 'admin'
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
INSERT INTO users (wallet, email, name, password_hash) 
VALUES ('0xAdminWalletAddress', 'admin@example.com', 'Admin', crypt('admin_password', gen_salt('bf')));

-- Attribuer le rôle admin à cet utilisateur
INSERT INTO roles (wallet_short, role)
VALUES (substring('0xAdminWalletAddress', 1, 8), 'admin'); 