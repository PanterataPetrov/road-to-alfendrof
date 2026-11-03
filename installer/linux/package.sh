#!/usr/bin/env bash
# Run from the repository root after `cargo build --locked --release --bin t2f`.
# Requires the Ubuntu packages listed in .github/workflows/linux.yml and LINUXDEPLOY's AppImage path.
# Outputs x86-64 packages in dist/. Native installs launch via `road-to-alfendrof` or the desktop menu.
# AppImage: chmod +x RoadToAlfendrof-*.AppImage, then run it (or use --appimage-extract-and-run without FUSE).
# Debian: sudo apt install ./road-to-alfendrof_*.deb; Fedora: sudo dnf install ./road-to-alfendrof-*.rpm.
# All formats need a host Vulkan graphics driver; AppImage targets glibc 2.35 or newer.
set -euo pipefail

: "${LINUXDEPLOY:?Set LINUXDEPLOY to the linuxdeploy AppImage path}"
LINUXDEPLOY=$(realpath "$LINUXDEPLOY")
VERSION=$(cargo metadata --locked --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "t2f") | .version')
ROOT=$PWD
mkdir -p dist
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

# Keep assets beside the real executable, matching assets_dir() without changing the game.
STAGING="$WORK/native"
GAME="$STAGING/usr/lib/road-to-alfendrof"
mkdir -p "$GAME" "$STAGING/usr/bin" "$STAGING/usr/share/applications" \
  "$STAGING/usr/share/icons/hicolor/256x256/apps"
install -m755 target/release/t2f "$GAME/t2f"
cp -a assets "$GAME/assets"
ln -s ../lib/road-to-alfendrof/t2f "$STAGING/usr/bin/road-to-alfendrof"
install -m644 installer/linux/road-to-alfendrof.desktop "$STAGING/usr/share/applications/"
convert assets/ui/emblem.png -filter point -resize 256x256 -gravity center -background none -extent 256x256 \
  "$WORK/road-to-alfendrof.png"
install -m644 "$WORK/road-to-alfendrof.png" "$STAGING/usr/share/icons/hicolor/256x256/apps/"

# Derive linked-library requirements from the actual executable. Add runtime-loaded window/GPU libs.
mkdir -p "$WORK/debian"
printf 'Source: road-to-alfendrof\n\nPackage: road-to-alfendrof\nArchitecture: amd64\n' > "$WORK/debian/control"
DEPS=$(cd "$WORK" && dpkg-shlibdeps -O -e"$GAME/t2f")
DEPS=${DEPS#shlibs:Depends=}
mkdir -p "$STAGING/DEBIAN"
cat > "$STAGING/DEBIAN/control" <<EOF
Package: road-to-alfendrof
Version: $VERSION
Architecture: amd64
Maintainer: zerw0 <zerw0@users.noreply.github.com>
Section: games
Priority: optional
Homepage: https://github.com/zerw0/road-to-alfendrof
Depends: $DEPS, libx11-6, libxcursor1, libxi6, libxrandr2, libxkbcommon0, libwayland-client0, libwayland-cursor0, libwayland-egl1, libvulkan1
Description: Road to Alfendrof fighting game
 Includes character, stage, music and artwork assets.
 Requires a Vulkan-capable graphics driver.
EOF
# Package managers install these files as root even if this script runs as an unprivileged CI user.
dpkg-deb --root-owner-group --build "$STAGING" "dist/road-to-alfendrof_${VERSION}_amd64.deb"

# RPM derives versioned ELF dependencies itself (including the build's glibc baseline).
rpmbuild -bb installer/linux/road-to-alfendrof.spec \
  --define "_topdir $WORK/rpm" --define "staging $STAGING" --define "game_version $VERSION"
cp "$WORK/rpm/RPMS/x86_64/road-to-alfendrof-${VERSION}-1.x86_64.rpm" dist/

APPDIR="$WORK/AppDir"
mkdir -p "$APPDIR/usr/bin"
install -m755 target/release/t2f "$APPDIR/usr/bin/road-to-alfendrof"
cp -a assets "$APPDIR/usr/bin/assets"
# Extract-and-run avoids requiring a FUSE mount in CI; linuxdeploy also bundles linked shared libraries.
export APPIMAGE_EXTRACT_AND_RUN=1
export ARCH=x86_64
export LINUXDEPLOY_OUTPUT_VERSION="$VERSION"
export LDAI_OUTPUT="$ROOT/dist/RoadToAlfendrof-${VERSION}-x86_64.AppImage"
"$LINUXDEPLOY" --appdir "$APPDIR" \
  --executable "$APPDIR/usr/bin/road-to-alfendrof" \
  --desktop-file installer/linux/road-to-alfendrof.desktop \
  --icon-file "$WORK/road-to-alfendrof.png" --output appimage
