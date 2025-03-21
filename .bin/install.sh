#!/bin/bash
# Script d'installation du service SMX Power

set -e

# Vérification des privilèges root
if [ "$EUID" -ne 0 ]; then
  echo "Ce script doit être exécuté en tant que root"
  exit 1
fi

# Répertoire d'installation
INSTALL_DIR="/usr/local/bin"
SERVICE_NAME="sme-pwrsvc"
BINARY_NAME="pwrsvc"

# Copier l'exécutable
echo "Installation de l'exécutable..."
if [ -f "./$BINARY_NAME" ]; then
  cp "./$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
  chmod 755 "$INSTALL_DIR/$BINARY_NAME"
else
  echo "Erreur: Exécutable '$BINARY_NAME' non trouvé dans le répertoire courant"
  exit 1
fi

# Installer le fichier de service systemd
echo "Installation du service systemd..."
if [ -f "./$SERVICE_NAME.service" ]; then
  cp "./$SERVICE_NAME.service" "/etc/systemd/system/"
  chmod 644 "/etc/systemd/system/$SERVICE_NAME.service"
else
  echo "Erreur: Fichier de service '$SERVICE_NAME.service' non trouvé dans le répertoire courant"
  exit 1
fi

# Créer le répertoire runtime si utilisation de /run
mkdir -p /run/sme-pwrsvc
chmod 755 /run/sme-pwrsvc

# Recharger systemd et activer le service
echo "Activation du service..."
systemctl daemon-reload
systemctl enable "$SERVICE_NAME.service"

echo "Installation terminée. Démarrer le service avec: systemctl start $SERVICE_NAME"