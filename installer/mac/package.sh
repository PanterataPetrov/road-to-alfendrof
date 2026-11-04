#!/bin/bash
# Usage: bash installer/mac/package.sh --binary PATH --output DIR --version VERSION
# Requires macOS, Xcode command-line tools (Clang/AppKit), and a logged-in Finder session.
set -euo pipefail

BINARY=""
OUTPUT=""
VERSION=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --binary|--output|--version)
            [[ $# -ge 2 ]] || { echo "Missing value for $1" >&2; exit 1; }
            case "$1" in
                --binary) BINARY="$2" ;;
                --output) OUTPUT="$2" ;;
                --version) VERSION="$2" ;;
            esac
            shift 2
            ;;
        *) echo "Usage: $0 --binary PATH --output DIR --version VERSION" >&2; exit 1 ;;
    esac
done
[[ -n "$BINARY" && -n "$OUTPUT" && -n "$VERSION" ]] || {
    echo "Usage: $0 --binary PATH --output DIR --version VERSION" >&2; exit 1;
}
[[ "$(uname -s)" == Darwin ]] || { echo "Packaging requires macOS." >&2; exit 1; }
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
    echo "Version must have three numeric components (for example 0.1.3)." >&2; exit 1;
}
[[ -f "$BINARY" && -x "$BINARY" ]] || { echo "Binary must be an executable file: $BINARY" >&2; exit 1; }
case "$(/usr/bin/file -b "$BINARY")" in
    *Mach-O*executable*|*Mach-O*universal*) ;;
    *) echo "Expected a macOS Mach-O executable: $BINARY" >&2; exit 1 ;;
esac
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BINARY="$(cd "$(dirname "$BINARY")" && pwd)/$(basename "$BINARY")"
mkdir -p "$OUTPUT"
OUTPUT="$(cd "$OUTPUT" && pwd)"
WORK="$(mktemp -d "$OUTPUT/.mac-package.XXXXXX")"
MOUNT="$WORK/mount"
cleanup() {
    if /sbin/mount | /usr/bin/grep -Fq " on $MOUNT ("; then
        /usr/bin/hdiutil detach "$MOUNT" -force >/dev/null || true
    fi
    rm -rf "$WORK"
}
trap cleanup EXIT
STAGE="$WORK/stage"
APP="$STAGE/Road to Alfendrof.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" "$STAGE/.background" "$MOUNT"
cp "$BINARY" "$APP/Contents/MacOS/t2f"
chmod 755 "$APP/Contents/MacOS/t2f"
# Nix's Darwin toolchain can link its copy of Apple's libiconv. Use the macOS
# system library before signing so the distributable never depends on /nix/store.
while IFS= read -r library; do
    case "$library" in
        /nix/store/*/lib/libiconv.2.dylib)
            /usr/bin/install_name_tool -change "$library" /usr/lib/libiconv.2.dylib "$APP/Contents/MacOS/t2f"
            ;;
        /nix/*)
            echo "Unsupported non-portable library dependency: $library" >&2
            exit 1
            ;;
    esac
done < <(/usr/bin/otool -L "$APP/Contents/MacOS/t2f" | /usr/bin/awk 'NR > 1 {print $1}')
cp "$SCRIPT_DIR/Info.plist" "$APP/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $VERSION" "$APP/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" "$APP/Contents/Info.plist"
cp "$SCRIPT_DIR/AppIcon.icns" "$APP/Contents/Resources/AppIcon.icns"
/usr/bin/ditto "$ROOT/assets" "$APP/Contents/Resources/assets"
# Keep the update ZIP restricted to the app's root, including its bundled README.
cp "$SCRIPT_DIR/README.txt" "$APP/Contents/Resources/README.txt"
/usr/bin/xattr -cr "$APP"
/usr/bin/codesign --force --deep --sign - "$APP"
/usr/bin/ditto -c -k --keepParent --norsrc "$APP" "$WORK/RoadToAlfendrof-Mac.zip"

cp "$SCRIPT_DIR/README.txt" "$STAGE/README.txt"
ln -s /Applications "$STAGE/Applications"
/usr/bin/clang -fobjc-arc -framework AppKit "$SCRIPT_DIR/background.m" -o "$WORK/background"
"$WORK/background" \
    "$ROOT/assets/ui/logo.png" "$ROOT/assets/ui/emblem.png" "$STAGE/.background/background.png"
/usr/bin/hdiutil create -quiet -srcfolder "$STAGE" -volname "Road to Alfendrof" \
    -fs HFS+ -format UDRW "$WORK/installer.dmg"
/usr/bin/hdiutil attach -quiet -readwrite -noverify -noautoopen \
    -mountpoint "$MOUNT" "$WORK/installer.dmg"
# Finder writes .DS_Store window geometry, artwork and icon positions onto the image.
/usr/bin/osascript - "$MOUNT" <<'APPLESCRIPT'
on run arguments
    set diskFolder to POSIX file (item 1 of arguments) as alias
    tell application "Finder"
        set installerWindow to make new Finder window to diskFolder
        set installerFolder to target of installerWindow
        tell installerWindow
            set current view to icon view
            delay 1
            set toolbar visible to false
            set statusbar visible to false
            set bounds to {100, 100, 900, 648}
            set viewOptions to icon view options of installerWindow
            set arrangement of viewOptions to not arranged
            set icon size of viewOptions to 88
            set text size of viewOptions to 13
            set background picture of viewOptions to POSIX file ((item 1 of arguments) & "/.background/background.png")
            set position of item "Road to Alfendrof.app" of installerFolder to {200, 254}
            set position of item "Applications" of installerFolder to {600, 254}
            set position of item "README.txt" of installerFolder to {400, 350}
            close
        end tell
        update diskFolder without registering applications
        delay 2
    end tell
end run
APPLESCRIPT
/bin/sync
/usr/bin/hdiutil detach -quiet "$MOUNT"
/usr/bin/hdiutil convert -quiet "$WORK/installer.dmg" -format UDZO \
    -imagekey zlib-level=9 -o "$WORK/RoadToAlfendrof-Mac.dmg"
# Publish only complete artifacts, leaving any previous build intact until this point.
mv -f "$WORK/RoadToAlfendrof-Mac.zip" "$OUTPUT/RoadToAlfendrof-Mac.zip"
mv -f "$WORK/RoadToAlfendrof-Mac.dmg" "$OUTPUT/RoadToAlfendrof-Mac.dmg"
printf 'Created %s\n' "$OUTPUT/RoadToAlfendrof-Mac.zip" "$OUTPUT/RoadToAlfendrof-Mac.dmg"
