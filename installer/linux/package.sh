#!/bin/sh
# Usage: sh installer/linux/package.sh /path/to/t2f [output-directory] [x86_64|aarch64]
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
BINARY=${1:?provide a release t2f executable}
OUTPUT=${2:-"$ROOT/dist"}
ARCH=${3:-$(uname -m)}
case "$ARCH" in x86_64|aarch64) ;; *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;; esac
[ -x "$BINARY" ] || { echo "Not an executable: $BINARY" >&2; exit 1; }
mkdir -p "$OUTPUT"
OUTPUT=$(CDPATH= cd -- "$OUTPUT" && pwd)
STAGE=$(mktemp -d "$OUTPUT/.linux-package.XXXXXXXX")
trap 'rm -rf "$STAGE"' EXIT HUP INT TERM
mkdir "$STAGE/RoadToAlfendrof"
cp "$BINARY" "$STAGE/RoadToAlfendrof/t2f"
chmod 755 "$STAGE/RoadToAlfendrof/t2f"
cp -R "$ROOT/assets" "$STAGE/RoadToAlfendrof/assets"
cp "$ROOT/installer/linux/README.txt" "$STAGE/RoadToAlfendrof/README.txt"
tar -czf "$STAGE/package.tar.gz" -C "$STAGE" RoadToAlfendrof
mv "$STAGE/package.tar.gz" "$OUTPUT/RoadToAlfendrof-Linux-$ARCH.tar.gz"
printf '%s\n' "$OUTPUT/RoadToAlfendrof-Linux-$ARCH.tar.gz"
