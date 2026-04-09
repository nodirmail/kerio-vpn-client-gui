#!/bin/bash
# install.sh
# Installation script for Kerio VPN Client macOS-like GUI (Ubuntu 24.04)

echo "Building the application..."
npm run tauri build

echo "Installing the application..."
# Tauri creates a .deb package that we can install
DEB_FILE=$(find src-tauri/target/release/bundle/deb -name "*.deb" | head -n 1)

if [ -z "$DEB_FILE" ]; then
    echo "Build failed. Could not find .deb package."
    exit 1
fi

sudo dpkg -i "$DEB_FILE"

echo "Installation complete!"
echo "If you want to allow the VPN to connect/disconnect WITHOUT asking for your password every time, you can configure sudoers or Polkit."
echo "Application icon should now be available in your application launcher."
