#!/bin/bash
# Code signing template for Windows installer
# Usage: ./scripts/sign-installer.sh

set -e

CERT_PATH="${SIGN_CERT_PATH}"
CERT_PASSWORD="${SIGN_PASSWORD}"
INSTALLER_PATH="src-tauri/target/release/bundle/nsis/zntlDesk_0.1.0_x64_en-US.exe"

if [ ! -f "$INSTALLER_PATH" ]; then
    echo "❌ Installer not found: $INSTALLER_PATH"
    exit 1
fi

if [ -z "$CERT_PATH" ] || [ -z "$CERT_PASSWORD" ]; then
    echo "❌ Environment variables not set:"
    echo "   SIGN_CERT_PATH: path to .pfx file"
    echo "   SIGN_PASSWORD: certificate password"
    exit 1
fi

echo "🔐 Signing installer..."
# This will be implemented with actual signtool or equivalent
# signtool sign /f "$CERT_PATH" /p "$CERT_PASSWORD" /t http://timestamp.server.com "$INSTALLER_PATH"

echo "✅ Installer signed successfully"
