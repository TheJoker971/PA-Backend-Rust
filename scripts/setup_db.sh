#!/bin/bash

# Script pour configurer la base de données

echo "🔧 Configuration de la base de données..."

# Vérifier si les variables d'environnement sont définies
if [ -z "$DATABASE_URL" ]; then
    echo "❌ Erreur: DATABASE_URL n'est pas définie"
    echo "Créez un fichier .env avec DATABASE_URL=postgresql://..."
    exit 1
fi

# Appliquer le schéma
echo "📋 Application du schéma..."
psql "$DATABASE_URL" -f migrations/schema.sql

# Appliquer les migrations supplémentaires
echo "📋 Application des migrations supplémentaires..."
psql "$DATABASE_URL" -f migrations/roles_migration.sql

echo "✅ Base de données configurée avec succès!" 