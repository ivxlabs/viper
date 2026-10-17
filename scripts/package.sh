#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
# Build the engine and lay it out as Difflicit installs it: dist/<id>/engine.json and dist/<id>/bin/<id>.
# In Difflicit: Analyze → Manage engines… → Install engine…, and pick dist/<id>.
set -eu
cd "$(dirname "$0")/.."
id=$(sed -n 's/^ *"id": *"\([^"]*\)".*/\1/p' engine.json | head -n 1)
cargo build --release --quiet
rm -rf "dist/$id"
mkdir -p "dist/$id/bin"
cp engine.json "dist/$id/"
cp "target/release/$id" "dist/$id/bin/$id"
echo "dist/$id"
