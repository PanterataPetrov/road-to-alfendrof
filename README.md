 # Road to Alfendrof

A Bevy fighting game. Controls are available in Options > Controls.
Windowed launches start maximized; saved borderless/fullscreen choices still apply.

Version 0.1.10 adds rollback-capable Mac/Linux automatic updates and an artwork-backed
Mac DMG installer; the updater uses the companion ZIP.

Stages: Galata Tower and Assgrad. Select a stage after choosing both fighters.
Assgrad combines a 640px, 96-color pixel-art river landscape with grey paving,
a concrete bridge parapet, metal handrail and a decorative sightseeing viewer.
Its backdrop fits the visible arena without changing gameplay zoom; foreground
colors share the backdrop palette. All Assgrad art uses nearest-neighbor filtering.
Regenerate bridge art in the dev shell with `python tools/gen_assgrad.py`; pass
the original photograph as an optional argument to also regenerate the backdrop.

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
Local packages match the supplied executable's architecture. The official Mac
workflow builds Apple Silicon (ARM64) only and packages both artifacts; official
Intel builds are not provided.
Apps are ad hoc signed, not Developer ID signed or notarized. See
[Mac installation instructions](installer/mac/README.txt) for Gatekeeper and
per-user installation guidance.

## Linux packages

On Linux, after building the release executable:

```sh
LINUXDEPLOY=/path/to/linuxdeploy.AppImage bash installer/linux/package.sh
# Or build only the portable updater archive:
sh installer/linux/portable.sh target/release/t2f dist x86_64
```

The first command creates the AppImage, Debian `.deb`, Fedora `.rpm`, and portable
`.tar.gz` packages. See `.github/workflows/linux.yml` for native prerequisites and
the pinned, checksum-verified linuxdeploy download. The workflow runs on Ubuntu
22.04 to retain a glibc 2.35 baseline. The portable script also accepts `aarch64`
when supplied an aarch64 executable; official Linux releases are x86_64 only.
Runtime requirements and installation instructions are in
[the Linux README](installer/linux/README.txt).

## Automatic updates and publishing

Installed release builds check this project's latest GitHub release and download,
validate and stage the matching package in the background on startup. At the
title screen they exit only after the update helper acknowledges startup. On
Mac/Linux, the helper waits for the game to exit, replaces the installation,
restarts it and restores the previous installation if replacement or initial
launch fails. Windows uses the Inno Setup installer. Settings live outside the
installation. Debug/source builds do not update.

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

A push to `main` starts **Stable release**, which checks the Cargo version,
builds all platforms, and publishes one release only after every build succeeds.
Only its publisher has write permission. Platform workflows run manually or via
`workflow_call`, upload artifacts, and never publish independently. A manual
**Stable release** run on a non-main branch builds artifacts only; on `main` it
can publish. Do not create or push a tag to deploy: the publisher creates
`v<Cargo.toml version>` for the exact built main commit. Bump the version before
each new main release; existing tags are never moved.

All seven payloads must be present: AppImage, DEB, RPM, Windows installer, Mac
ZIP, Mac DMG, and Linux portable archive. A missing updater asset is treated as
an unsuccessful update, not a reason to quit. See [RELEASING.md](RELEASING.md)
for the canonical release and recovery procedure.
