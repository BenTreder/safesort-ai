#!/usr/bin/env bash
set -euo pipefail

BIN="$HOME/.local/bin/safesort"
DATA="$HOME/.local/share/safesort"

if [[ -f "$BIN" ]]; then
  rm -f "$BIN"
  echo "Removed: $BIN"
else
  echo "No binary found at: $BIN"
fi

echo
echo "SafeSort organized folders such as ./safesort/ are NEVER deleted by this script."
echo

if [[ -d "$DATA" ]]; then
  read -r -p "Delete SafeSort metadata under $DATA? This includes manifests, backups, and rollback receipts. Type DELETE to confirm: " answer
  if [[ "$answer" == "DELETE" ]]; then
    rm -rf "$DATA"
    echo "Deleted metadata: $DATA"
  else
    echo "Kept metadata: $DATA"
  fi
else
  echo "No metadata found at: $DATA"
fi
