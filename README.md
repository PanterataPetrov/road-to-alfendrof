# Road to Alfendrof

A Bevy fighting game. Controls are available in Options > Controls.
Windowed launches start maximized; saved borderless/fullscreen choices still apply.

## Development

The pinned Rust environment lives in `nix/` so Nix does not copy build outputs
while Cargo is compiling. With Nix and direnv installed, review `.envrc`, then
run `direnv allow`. It activates tools without replacing your interactive shell.
Without direnv:

```sh
nix develop path:./nix
cargo run --release --bin t2f
cargo test --release --lib
rustc --edition=2021 --test src/update_install.rs -o /tmp/update-tests
/tmp/update-tests
```

macOS packaging additionally requires Apple's Command Line Tools and Finder.
Linux development requires a working graphics driver; native library dependencies
are included in the Nix shell.

## macOS installer

```sh
cargo build --locked --release --bin t2f
VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "t2f") | .version')
bash installer/mac/package.sh --binary target/release/t2f --output dist --version "$VERSION"
```

This creates `dist/RoadToAlfendrof-Mac.dmg` with game artwork, an Applications
shortcut and README, plus `dist/RoadToAlfendrof-Mac.zip` for the updater.
Local packages match the supplied executable's architecture. The Mac workflow
builds a universal Intel/Apple Silicon executable and packages both artifacts.
Apps are ad hoc signed, not Developer ID signed or notarized. See
[Mac installation instructions](installer/mac/README.txt) for Gatekeeper and
per-user installation guidance.

## Linux packages

On Linux, after building the release executable:

```sh
sh installer/linux/package.sh target/release/t2f dist x86_64
sh installer/linux/appimage.sh target/release/t2f /path/to/appimagetool dist
```

The first command makes the portable updater archive; it also accepts `aarch64`
when given an aarch64 executable. The second creates an x86_64 AppImage using
appimagetool 1.9.1. The Linux workflow builds both on Ubuntu 22.04 and verifies
the downloaded packaging tool's checksum. Runtime requirements and installation
instructions are in [the Linux README](installer/linux/README.txt).

## Automatic updates and publishing

Installed release builds check this project's latest GitHub release and download,
validate and stage the matching package in the background on startup. At the
title screen they exit only after the update helper acknowledges startup. The
helper waits for the game to exit, replaces the installation, restarts it and
restores the previous installation if replacement or initial launch fails.
Settings live outside the installation. Debug/source builds do not update.

Supported payloads:

- Windows: `RoadToAlfendrof-Setup.exe` (existing Inno Setup installer).
- macOS: `RoadToAlfendrof-Mac.zip`, with the signed app at its root.
- Linux portable: `RoadToAlfendrof-Linux-{x86_64|aarch64}.tar.gz`, with
  `RoadToAlfendrof/t2f` and `RoadToAlfendrof/assets`.
- AppImage: `RoadToAlfendrof-{version}-{architecture}.AppImage`.

The install and its parent must be writable. macOS disk images and Linux
system/package-manager installations do not self-replace. AppImage updates
replace the user-owned original file, never its mounted filesystem. Failed or
offline downloads leave the game running. Updates replace the entire package;
keep personal files outside it. Downloads require HTTPS and the platform's
curl/tar utilities. Version tags must be stable `vX.Y.Z` or `X.Y.Z`.

Manual GitHub workflow runs produce downloadable artifacts only. Pushing a
`v*` tag attaches the built packages to a GitHub release. Keep the tag version
and Cargo.toml version equal, and publish all platform assets for that release;
a missing asset is treated as an unsuccessful update, not a reason to quit.
