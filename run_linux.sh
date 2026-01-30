#!/bin/bash

# Se déplacer dans le dossier du script
cd "$(dirname "$0")"

# Vérifier si le binaire existe
BINARY="./target/release/pixelvault"

if [ ! -f "$BINARY" ]; then
    echo "⚠️  Le binaire n'a pas été trouvé !"
    echo "Tentative de compilation en cours..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "❌ Erreur lors de la compilation."
        echo "Appuyez sur Entrée pour fermer..."
        read
        exit 1
    fi
fi

# Ajouter le dossier release au PATH pour cette session
export PATH="$PATH:$(pwd)/target/release"

echo "========================================================"
echo "   🔐  PIXELVAULT - Terminal Interactif"
echo "========================================================"
echo ""
echo "Le programme est prêt."
echo "Commandes disponibles :"
echo "  pixelvault init     -> Créer un nouveau coffre"
echo "  pixelvault add      -> Ajouter un mot de passe"
echo "  pixelvault get      -> Récupérer un mot de passe"
echo "  pixelvault list     -> Voir les entrées"
echo "  pixelvault --help   -> Voir toute l'aide"
echo ""
echo "Tapez vos commandes ci-dessous."
echo "========================================================"
echo ""

# Lancer un shell interactif pour garder la fenêtre ouverte
# On utilise $SHELL ou bash par défaut
if [ -z "$SHELL" ]; then
    bash
else
    "$SHELL"
fi
