-- migrations/schema.sql

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

-- Création de la table users
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet TEXT NOT NULL UNIQUE,
    name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Création de la table roles
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Création de la table sessions
CREATE TABLE IF NOT EXISTS sessions (
    token UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL
);

-- Création de la table properties
CREATE TABLE IF NOT EXISTS properties (
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

-- Création de la table investments
CREATE TABLE IF NOT EXISTS investments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    property_id UUID NOT NULL REFERENCES properties(id),
    amount_eth NUMERIC NOT NULL,
    shares INTEGER NOT NULL,
    tx_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

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
    WHERE wallet = wallet_address;
    
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
