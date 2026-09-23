#!/usr/bin/env bash
# Rebuilds adapter/ from Cortex-Debug sources + patches/*.patch.
# Usage: scripts/build-adapter.sh [path-to-existing-cortex-debug-checkout]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${1:-$ROOT/.build/cortex-debug}"
BASE="$(cat "$ROOT/patches/BASE_COMMIT")"

if [ ! -d "$SRC/.git" ]; then
  git clone https://github.com/Marus/cortex-debug.git "$SRC"
  git -C "$SRC" checkout -q "$BASE"
  git -C "$SRC" -c user.name=build -c user.email=build@localhost am -q "$ROOT"/patches/*.patch
fi

cd "$SRC"
npm ci --ignore-scripts
npx webpack --config-name zedAdapter --mode production

mkdir -p "$ROOT/adapter/dist" "$ROOT/adapter/support"
cp dist/zedadapter.js "$ROOT/adapter/dist/"
cp support/gdbsupport.init support/gdb-swo.init "$ROOT/adapter/support/"
cp LICENSE "$ROOT/adapter/LICENSE-cortex-debug"
echo "adapter/ updated. Rebuild the extension in Zed (Extensions -> Rebuild)."
