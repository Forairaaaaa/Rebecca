#!/bin/bash

set -e

URL="https://github.com/microsoft/cascadia-code/releases/download/v2407.24/CascadiaCode-2407.24.zip"
TMP_DIR="/tmp/cascadia_install"
ZIP_NAME="CascadiaCode.zip"
EXTRACT_DIR="$TMP_DIR/CascadiaCode"
FONT_DIR="/usr/local/share/fonts"

echo "Creating temp directory $EXTRACT_DIR..."
mkdir -p "$EXTRACT_DIR"

echo "Downloading Cascadia Code..."
curl -L "$URL" -o "$TMP_DIR/$ZIP_NAME"

echo "Extracting zip to $EXTRACT_DIR..."
unzip -q "$TMP_DIR/$ZIP_NAME" -d "$EXTRACT_DIR"

echo "Installing fonts to $FONT_DIR..."
sudo mkdir -p "$FONT_DIR"
sudo cp -r "$EXTRACT_DIR" "$FONT_DIR"

echo "Updating font cache..."
sudo fc-cache -f -v

echo "Cleaning up..."
rm -rf "$TMP_DIR"

echo "Cascadia Code installed successfully!"
