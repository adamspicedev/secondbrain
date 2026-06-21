#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

echo "[doctor:deps] Removing node_modules"
rm -rf node_modules

echo "[doctor:deps] Reinstalling dependencies with lockfile"
bun install --frozen-lockfile

echo "[doctor:deps] Running TypeScript check"
bun run type-check

echo "[doctor:deps] Running Biome check"
bun run biome:check

echo "[doctor:deps] Done. If VS Code still shows stale errors, run:"
echo "  1) TypeScript: Restart TS Server"
echo "  2) Developer: Reload Window"
