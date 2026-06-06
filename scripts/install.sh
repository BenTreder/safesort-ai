#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "Building SafeSort AI release binary..."
cargo build --release

mkdir -p "$HOME/.local/bin"
install -m 755 target/release/safesort "$HOME/.local/bin/safesort"

echo
echo "Installed:"
echo "  $HOME/.local/bin/safesort"

case ":$PATH:" in
  *":$HOME/.local/bin:"*) ;;
  *)
    echo
    echo "Note: ~/.local/bin is not currently in your PATH."
    echo "Add this to your shell config:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
    ;;
esac

echo
echo "Try:"
echo "  safesort"
echo "  safesort -scan"
echo "  safesort -run"
echo "  safesort -status"
echo "  safesort -rollback"
