#!/usr/bin/env bash
# Install the pinned official mj release without a Go toolchain.
set -euo pipefail

version=0.11.1
case "$(uname -s)" in
  Linux) platform=linux ;;
  Darwin) platform=darwin ;;
  *) echo 'mj supports Linux and macOS.' >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64) arch=amd64 ;;
  arm64|aarch64) arch=arm64 ;;
  *) echo 'mj supports amd64 and arm64.' >&2; exit 1 ;;
esac

destination=${1:-"$HOME/.local/bin"}
mkdir -p "$destination"
destination=$(cd "$destination" && pwd)
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
archive="mj_${version}_${platform}_${arch}.tar.gz"
release="https://github.com/ehmo/mj/releases/download/v${version}"
curl -fsSL "$release/$archive" -o "$temporary/$archive"
curl -fsSL "$release/checksums.txt" -o "$temporary/checksums.txt"
python3 - "$temporary" "$archive" <<'PY'
import hashlib
import sys
from pathlib import Path

directory, archive = Path(sys.argv[1]), sys.argv[2]
expected = next(line.split()[0] for line in (directory / "checksums.txt").read_text().splitlines()
                if line.split()[-1] == archive)
actual = hashlib.sha256((directory / archive).read_bytes()).hexdigest()
if actual != expected:
    raise SystemExit("mj archive checksum mismatch")
PY
tar -xzf "$temporary/$archive" -C "$temporary" mj mj-mcp
install -m 755 "$temporary/mj" "$temporary/mj-mcp" "$destination/"
"$destination/mj" version
echo "Installed mj and mj-mcp in $destination"
