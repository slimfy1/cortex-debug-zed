#!/usr/bin/env bash
# Fills in the repository owner, author name and email.
# Usage: scripts/set-owner.sh <github-user> "<Your Name>" <you@example.com>
set -euo pipefail
[ $# -eq 3 ] || { echo "usage: $0 <github-user> \"<Your Name>\" <email>"; exit 1; }
cd "$(dirname "$0")/.."
user="$1"; name="$2"; email="$3"
sed -i.bak \
  -e "s|<your-username>|$user|g" \
  -e "s|<your-name> <<your-email>>|$name <$email>|g" \
  extension.toml README.md README.ru.md
sed -i.bak -e "s|YOUR_GITHUB_USERNAME|$user|g" src/lib.rs
rm -f extension.toml.bak README.md.bak README.ru.md.bak src/lib.rs.bak
grep -n "authors\|repository" extension.toml
grep -n "GITHUB_REPO: &str" src/lib.rs
