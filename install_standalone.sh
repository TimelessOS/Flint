#!/bin/bash

# This script installs flint into a temporary location, adds a repository, and then installs flint from there into it's final location.

set -eu

TMP_LOCATION="/tmp/flint_installer.x86_64"
INITIAL_REPOSITORY_NAME="$1"
INITIAL_REPOSITORY_URL="$2"

echo "[INFO] Using inital repository: '$INITIAL_REPOSITORY_NAME' with URL: '$INITIAL_REPOSITORY_URL'"

echo "[INFO] Installing latest flintpkg from github releases..."
# TODO: Rename to flint.x86_64 on next release
curl --follow https://github.com/TimelessOS/Flint/releases/latest/download/flint -o $TMP_LOCATION

echo "[INFO] Beginning installation..."
chmod 700 $TMP_LOCATION

echo "[INFO] Adding initial repository..."
$TMP_LOCATION repo add $INITIAL_REPOSITORY_NAME $INITIAL_REPOSITORY_URL 

echo "[INFO] Installing 'flint' from initial repository..."
$TMP_LOCATION install flint
# This is needed to update the quicklaunch scripts.
$TMP_LOCATION run flint -- flint update

rm $TMP_LOCATION


echo "====================================================="
echo "Installed successfully!"
