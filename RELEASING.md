# Deploying a stable version

## Release policy

A push to `main` starts **Stable release** (`.github/workflows/release.yml`). It checks the version, runs the Linux, Windows and Mac build workflows, and publishes one stable GitHub release only after all three builds succeed.

- The tag is `v<package.version>` from `Cargo.toml`; it identifies the exact main commit that was built.
- Only the `publish` job has `contents: write`, and it is restricted to `refs/heads/main`.
- Platform workflows run manually or through `workflow_call`; they upload Actions artifacts and never publish independently.
- Manually running **Stable release** on a non-main branch builds artifacts without publishing. Running it on `main` can publish.
- An existing version tag may be reused only by a rerun of the same commit. A different main commit with the same version fails the version check. Existing tags are never moved.

Every main push is a release candidate. Once a version is published, bump the version before merging another change to main, including documentation or CI changes. Keep work that is not ready for release on a feature branch.

## Prepare the next version

1. Create a feature branch from the target repository's current `main`.
2. Choose a new `MAJOR.MINOR.PATCH` version greater than the latest published stable release. Do not use a prerelease suffix for this workflow.
3. Update `[package].version` in `Cargo.toml` and the default `AppVersion` in `installer/road-to-alfendrof.iss`. CI passes the manifest version to Inno Setup automatically; the default keeps manual installer builds consistent.
4. Refresh the workspace package version in the lockfile:

   ```sh
   cargo update --package t2f --offline
   ```

   If dependencies are not cached, omit `--offline`. Commit `Cargo.toml`, `Cargo.lock`, and the installer version change together.

5. Run `cargo test --locked --release --lib` in the appropriate development environment. For packaging prerequisites, use the platform workflows as the source of truth.
6. Commit and push the feature branch, then open a pull request against `main` in the repository that should publish the release. Do not create the release tag manually.

The original repository is `thracian141/road-to-alfendrof`; `zerw0/road-to-alfendrof` is a fork. Workflows publish into the repository where they run, not into its parent. The built-in Windows updater currently checks the original repository, so releases made in a fork do not redirect existing clients to that fork.

## Build and smoke-test before deployment

Once the workflows exist on the repository's default branch, open **Actions** and run **Linux build**, **Windows build**, or **Mac build**, selecting the candidate branch. These runs do not publish releases. Alternatively, run **Stable release** on the candidate branch to build all three platforms together without publishing.

The platform jobs run in parallel and cache Rust dependencies. Tests use the release profile so the subsequent game build can reuse compiled dependencies instead of compiling a separate development profile. The Mac workflow uses `aarch64-apple-darwin` for both tests and the app on GitHub's ARM64 `macos-15` runner. Linux's three package formats share one executable; Windows and macOS require different binaries. The Mac package is Apple Silicon-only; there is no Intel build.

Download and extract the resulting Actions artifacts:

| Artifact | Package files |
| --- | --- |
| `RoadToAlfendrof-Linux-x86_64` | `.AppImage`, Debian `.deb`, Fedora `.rpm` |
| `RoadToAlfendrof-Windows-x86_64` | `RoadToAlfendrof-Setup.exe` |
| `RoadToAlfendrof-Mac` | `RoadToAlfendrof-Mac.zip` containing an Apple Silicon (ARM64) app |

Install and launch on the intended platforms. Check the title screen, music, character selection, and a fight on a stage. Launch from the desktop or another working directory to confirm assets are found independently of the source checkout.

- **Linux:** x86-64, glibc 2.35 or newer (Debian 12+ and current Fedora), with a working Vulkan graphics driver. Install DEB with `sudo apt install ./road-to-alfendrof_*.deb` or RPM with `sudo dnf install ./road-to-alfendrof-*.rpm`. For AppImage, run `chmod +x RoadToAlfendrof-*.AppImage`; if FUSE is unavailable, use `./RoadToAlfendrof-<version>-x86_64.AppImage --appimage-extract-and-run`.
- **Windows:** the x64 installer includes the game executable, all assets, the icon, and `dxcompiler.dll`/`dxil.dll`. It does not install the user's graphics driver. The installer is unsigned; check the repository and release origin before trusting it.
- **Mac:** Apple Silicon (M1 or later) only; Intel Macs are not supported. The app has an ad-hoc signature, not Apple notarization; see `installer/mac/README.txt` for the installation instructions.

A passing build does not prove graphical gameplay works on the user's GPU. Keep native runtime checks separate from compile and packaging checks.

## Deploy

1. Merge the approved, version-bumped pull request into the publishing repository's `main`. This is the deployment trigger; do not merge merely to test the pipeline.
2. In **Actions → Stable release**, wait for the version check and all three platform jobs, then the `publish` job.
3. In **Releases**, verify that `v<version>` points to the merged main commit and contains all five assets: AppImage, DEB, RPM, `RoadToAlfendrof-Setup.exe`, and `RoadToAlfendrof-Mac.zip`.
4. Review the automatically generated release notes and repeat the native installation/launch smoke checks using the files downloaded from the release itself.

The workflows use the repository's `GITHUB_TOKEN`; no personal access token is needed. Repository or organization policy must permit the publisher's explicitly declared `contents: write` permission. Do not grant write permissions to the platform build jobs.

## Failed runs and recovery

- A failed platform build prevents the publication job from running. Fix the cause before deploying; Actions artifacts from successful platforms may still be available for diagnosis.
- For a transient failure, use **Re-run all jobs** on the original main run. The same commit may complete or refresh its own versioned release.
- If the version guard reports that a tag belongs to another commit, bump the version and merge the correction. Do not delete or force-move a stable tag to bypass the guard.
- If the upload fails partway through, inspect the release for partial assets and rerun the same commit to complete it. Verify all five files before announcing the release.
- To roll back a bad published version, mark the previous known-good stable release as latest and reinstall its packages where necessary. The Windows updater does not automatically downgrade clients already on a newer version. Ship the permanent fix under a new, higher version; never replace the historical tag with different code.

### Action runtimes and runner notices

The JavaScript actions use Node 24-compatible versions: `actions/checkout@v7`, `actions/upload-artifact@v7`, `actions/download-artifact@v8`, and `softprops/action-gh-release@v3`. Resolve runtime deprecation warnings by updating the action references, not by suppressing the warnings or forcing an unsupported runtime. When upgrading artifact actions, preserve named archive uploads and automatic download extraction so the publisher still receives the expected package files.

A macOS ARM64 capacity notice concerns GitHub's runner scheduling. A job may wait in the queue before any compilation starts; changing Rust build flags does not shorten that wait. It is a scheduler notice, not a game compiler error.
