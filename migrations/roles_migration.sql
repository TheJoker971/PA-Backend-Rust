-- Migration pour la gestion des rôles
-- Création des tables pour les utilisateurs avec rôles spéciaux et permissions sur propriétés

-- 1. Table des utilisateurs avec rôles spéciaux (admin/manager uniquement)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signature TEXT UNIQUE NOT NULL,
    name TEXT,
    role TEXT NOT NULL CHECK (role IN ('admin', 'manager')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 2. Table des propriétés avec contrats (enrichissement de la table existante)
-- Ajout de la colonne share_token_address si elle n'existe pas
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'properties' AND column_name = 'share_token_address') THEN
        ALTER TABLE properties ADD COLUMN share_token_address TEXT;
    END IF;
END $$;

-- 3. Table des permissions sur les propriétés
CREATE TABLE IF NOT EXISTS property_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_signature TEXT NOT NULL REFERENCES users(signature) ON DELETE CASCADE,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    role_type TEXT NOT NULL CHECK (role_type IN ('yield_manager', 'admin')),
    granted_by TEXT NOT NULL,
    granted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE,
    UNIQUE(user_signature, property_id, role_type)
);

-- 4. Index pour optimiser les performances
CREATE INDEX IF NOT EXISTS idx_users_signature ON users(signature);
CREATE INDEX IF NOT EXISTS idx_property_permissions_user ON property_permissions(user_signature);
CREATE INDEX IF NOT EXISTS idx_property_permissions_property ON property_permissions(property_id);
CREATE INDEX IF NOT EXISTS idx_property_permissions_active ON property_permissions(is_active);

-- 5. Insertion d'un utilisateur admin par défaut (à modifier selon vos besoins)
INSERT INTO users (signature, name, role) 
VALUES ('0xADMIN_SIGNATURE_PLACEHOLDER', 'Admin', 'admin')
ON CONFLICT (signature) DO NOTHING;

-- 6. Commentaires pour la documentation
COMMENT ON TABLE users IS 'Utilisateurs avec rôles spéciaux (admin/manager). Les investisseurs ne sont pas stockés ici.';
COMMENT ON TABLE property_permissions IS 'Permissions des utilisateurs sur les propriétés spécifiques';
COMMENT ON COLUMN properties.share_token_address IS 'Adresse du contrat PropertyShares pour cette propriété'; 
 