#!/bin/sh
# Usage: sh installer/linux/appimage.sh /path/to/t2f /path/to/appimagetool [output-directory]
# Run on x86_64 Linux. appimagetool must support --appimage-extract-and-run.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
BINARY=${1:?provide a release t2f executable}
TOOL=${2:?provide appimagetool}
OUTPUT=${3:-"$ROOT/dist"}
BINARY=$(realpath "$BINARY")
TOOL=$(realpath "$TOOL")
VERSION=$(cargo metadata --no-deps --format-version 1 --manifest-path "$ROOT/Cargo.toml" | jq -r '.packages[] | select(.name == "t2f") | .version')
mkdir -p "$OUTPUT"
OUTPUT=$(CDPATH= cd -- "$OUTPUT" && pwd)
STAGE=$(mktemp -d "$OUTPUT/.appimage-package.XXXXXXXX")
trap 'rm -rf "$STAGE"' EXIT HUP INT TERM
APP="$STAGE/RoadToAlfendrof.AppDir"
mkdir -p "$APP/usr/bin"
cp "$BINARY" "$APP/usr/bin/t2f"
chmod 755 "$APP/usr/bin/t2f"
cp -R "$ROOT/assets" "$APP/usr/bin/assets"
cp "$ROOT/installer/linux/README.txt" "$APP/README.txt"
cp "$ROOT/assets/ui/emblem.png" "$APP/road-to-alfendrof.png"
cp "$ROOT/installer/linux/road-to-alfendrof.desktop" "$APP/"
cp "$ROOT/installer/linux/AppRun" "$APP/AppRun"
chmod 755 "$APP/AppRun"
ARCH=x86_64 "$TOOL" --appimage-extract-and-run "$APP" "$OUTPUT/RoadToAlfendrof-$VERSION-x86_64.AppImage"
