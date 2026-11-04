Road to Alfendrof for Mac
========================

INSTALL
1. Open RoadToAlfendrof-Mac.dmg and drag "Road to Alfendrof" onto Applications.
   Alternatively, unzip RoadToAlfendrof-Mac.zip and copy its app to Applications.
   For a per-user install without administrator access, use ~/Applications instead.
2. Eject the disk image. Launch the installed copy, not the copy on the disk image.
3. This community build is ad hoc signed, NOT Developer ID signed or notarized.
   macOS may block its first launch because Apple cannot verify the developer.
   Only if you trust the release from
   https://github.com/thracian141/road-to-alfendrof/releases
   try opening the installed app, then visit System Settings > Privacy & Security.
   Scroll to Security, choose Open Anyway for this app, and confirm Open. On older
   macOS versions this is System Preferences > Security & Privacy > General.
   Do not disable Gatekeeper or remove quarantine attributes globally. If macOS
   reports malware or a damaged app, stop and download a fresh official release;
   do not bypass that warning.

The official release supports Apple Silicon and Intel Macs. Controls are in
Options > Controls. A locally built package supports the supplied binary's CPU
architecture only.
Windowed launches start maximized, with text scaled to the available window size.
Saved borderless/fullscreen preferences still apply; F11 toggles display mode.

AUTOMATIC UPDATES
Installed release builds check the GitHub project's latest release at startup.
When an update is ready on the title screen, the game closes, replaces the whole
app and relaunches it. Settings are kept separately and survive replacement.
Offline or unsuccessful update checks leave the current installation usable.
macOS security checks still apply to updated apps.

The app AND its containing folder must be writable by your account. Updating
never asks for administrator privileges. A disk image (/Volumes), read-only
volume, source/debug build, or installation owned by another user will not
self-update. Use ~/Applications if your /Applications folder is not writable.
Do not run directly from the download preview or disk image: install and eject
first. If automatic updating is unavailable, quit the game, download the latest
DMG and replace the installed copy manually. Do not store personal files inside
the app bundle; updates replace it in full.

SETTINGS AND TROUBLESHOOTING
Settings: ~/Library/Application Support/Road to Alfendrof/settings.ron
Finder > Go > Go to Folder can open that directory. Quit before editing settings.

No persistent game or updater log file is created by default. To capture game
output, quit the game and run this in Terminal (adjust the app path for a per-user
install):

  "/Applications/Road to Alfendrof.app/Contents/MacOS/t2f" 2>&1 | tee "$HOME/Desktop/RoadToAlfendrof.log"

Send the captured log, Mac model, CPU type, macOS version, game version, and a
description of the problem. Check the log for personal information before sharing.
Updater helper output is not included in that game log. On a failed update,
reopen the installed app or install the latest DMG manually.

BUILDING THIS INSTALLER
Requires macOS, Xcode Command Line Tools (Clang/AppKit and lipo), Apple's
codesign, ditto, hdiutil and osascript tools, and Finder in a logged-in graphical
session. Allow Terminal/your build runner to control Finder if macOS asks. No
external artwork downloads or third-party image tools are used: background.m
renders the existing assets/ui/logo.png and emblem.png into the DMG background.

Build a real release executable with the project's development environment, then
run from the repository root (replace VERSION with the Cargo.toml version):

  bash installer/mac/package.sh --binary target/release/t2f --output dist --version VERSION

This accepts a local arm64 executable. For a universal release, first build
--target aarch64-apple-darwin and --target x86_64-apple-darwin, then combine:

  lipo -create -output /tmp/t2f-universal target/aarch64-apple-darwin/release/t2f target/x86_64-apple-darwin/release/t2f
  bash installer/mac/package.sh --binary /tmp/t2f-universal --output dist --version VERSION

The same script creates RoadToAlfendrof-Mac.zip and RoadToAlfendrof-Mac.dmg.
The ZIP has the app at its root and this README inside Contents/Resources;
the DMG also exposes README.txt beside the app and Applications link. Both carry
the same ad hoc signed app. The Mac build workflow packages the universal binary
and uploads both artifacts; only a v* tag attaches them to a GitHub release.
