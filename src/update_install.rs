//! Dependency-free release selection and transactional Unix installation support.
//! Run independently with `rustc --edition=2021 --test src/update_install.rs -o /tmp/update-install-tests`.
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

pub fn version(value: &str) -> Option<[u32; 3]> {
    let value = value.strip_prefix('v').unwrap_or(value);
    let mut parts = value.split('.');
    let mut result = [0; 3];
    for slot in &mut result {
        let part = parts.next()?;
        if part.is_empty() || !part.bytes().all(|c| c.is_ascii_digit())
            || (part.len() > 1 && part.starts_with('0')) { return None; }
        *slot = part.parse().ok()?;
    }
    if parts.next().is_some() { return None; }
    Some(result)
}

pub fn is_newer(candidate: &str, current: &str) -> bool {
    match (version(candidate), version(current)) {
        (Some(candidate), Some(current)) => candidate > current,
        _ => false,
    }
}

/// Created atomically; Unix permissions are private from the moment of creation.
pub struct Workspace { pub path: PathBuf, retained: bool }
impl Workspace {
    pub fn new(parent: &Path) -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?.as_nanos();
        for _ in 0..100 {
            let path = parent.join(format!(".road-to-alfendrof-update-{}-{now}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)] {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => return Ok(Self { path, retained: false }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(io::ErrorKind::AlreadyExists, "cannot create private update workspace"))
    }
}
impl Drop for Workspace {
    fn drop(&mut self) { if !self.retained { let _ = fs::remove_dir_all(&self.path); } }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[cfg(target_os = "linux")]
    use std::os::unix::fs::MetadataExt;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Platform { Mac, Linux, AppImage }
    impl Platform {
        pub fn root(self) -> &'static str {
            match self { Self::Mac => "Road to Alfendrof.app", Self::Linux => "RoadToAlfendrof", Self::AppImage => "" }
        }
        pub fn asset(self, tag: &str) -> Option<String> {
            match self {
                Self::Mac => Some("RoadToAlfendrof-Mac.zip".into()),
                Self::Linux if matches!(std::env::consts::ARCH, "x86_64" | "aarch64") =>
                    Some(format!("RoadToAlfendrof-Linux-{}.tar.gz", std::env::consts::ARCH)),
                Self::AppImage if matches!(std::env::consts::ARCH, "x86_64" | "aarch64") && version(tag).is_some() =>
                    Some(format!("RoadToAlfendrof-{}-{}.AppImage", tag.strip_prefix('v').unwrap_or(tag), std::env::consts::ARCH)),
                _ => None,
            }
        }
        fn executable(self, root: &Path) -> PathBuf {
            if self == Self::AppImage { return root.to_path_buf(); }
            root.join(match self { Self::Mac => "Contents/MacOS/t2f", _ => "t2f" })
        }
        fn assets(self, root: &Path) -> PathBuf {
            root.join(match self { Self::Mac => "Contents/Resources/assets", _ => "assets" })
        }
    }

    pub struct Installation { pub root: PathBuf, pub platform: Platform }
    impl Installation {
        pub fn detect(executable: &Path, platform: Platform) -> io::Result<Self> {
            #[cfg(target_os = "linux")]
            if let Some(image) = std::env::var_os("APPIMAGE") {
                let appdir = std::env::var_os("APPDIR").ok_or_else(|| invalid("missing AppImage mount path"))?;
                if !fs::canonicalize(executable)?.starts_with(fs::canonicalize(appdir)?) {
                    return Err(invalid("executable is not inside its AppImage"));
                }
                let root = fs::canonicalize(image)?;
                let uid = Command::new("id").arg("-u").output()?;
                let uid: u32 = std::str::from_utf8(&uid.stdout).ok().and_then(|s| s.trim().parse().ok())
                    .ok_or_else(|| invalid("cannot determine user ownership"))?;
                let metadata = fs::metadata(&root)?;
                if uid != metadata.uid() || !metadata.is_file() || metadata.permissions().readonly() {
                    return Err(invalid("AppImage is not user-owned"));
                }
                validate_binary(&root, Platform::AppImage)?;
                let result = Self { root, platform: Platform::AppImage };
                drop(result.workspace()?);
                return Ok(result);
            }
            let executable = fs::canonicalize(executable)?;
            let root = match platform {
                Platform::Mac => executable.parent().and_then(Path::parent).and_then(Path::parent),
                Platform::Linux => executable.parent(),
                Platform::AppImage => return Err(invalid("AppImage requires APPIMAGE environment")),
            }.ok_or_else(|| invalid("missing installation directory"))?.to_path_buf();
            if root.file_name() != Some(std::ffi::OsStr::new(platform.root()))
                || platform.executable(&root) != executable || !platform.assets(&root).is_dir()
                || fs::metadata(&root)?.permissions().readonly()
                || (platform == Platform::Mac && (!root.join("Contents/Info.plist").is_file()
                    || root.starts_with("/Volumes"))) {
                return Err(invalid("not an installed writable package"));
            }
            // A real write probe rejects read-only mounts and unwritable installation parents.
            let probe = Workspace::new(root.parent().ok_or_else(|| invalid("missing installation parent"))?)?;
            drop(probe);
            Ok(Self { root, platform })
        }
        pub fn workspace(&self) -> io::Result<Workspace> {
            Workspace::new(self.root.parent().ok_or_else(|| invalid("missing installation parent"))?)
        }
        pub fn prepare(self, workspace: Workspace, archive: &Path) -> io::Result<Prepared> {
            let staged = if self.platform == Platform::AppImage {
                validate_binary(archive, self.platform)?;
                fs::set_permissions(archive, fs::Permissions::from_mode(0o755))?;
                archive.to_path_buf()
            } else {
                validate_listing(archive, self.platform)?;
                let unpack = workspace.path.join("unpack");
                fs::create_dir(&unpack)?;
                let result = Command::new(tar()).arg("-xf").arg(archive)
                    .arg("-C").arg(&unpack).arg("--no-same-owner")
                    .stdin(Stdio::null()).output()?;
                if !result.status.success() { return Err(invalid("cannot unpack update")); }
                let staged = unpack.join(self.platform.root());
                reject_links(&unpack)?;
                validate_payload(&staged, self.platform)?;
                staged
            };
            // Preflight rename permissions, including replacement of the existing package itself.
            if fs::symlink_metadata(&self.root)?.permissions().readonly() {
                return Err(invalid("installation is read-only"));
            }
            let script = helper_script(&self.root, &staged, &workspace.path, self.platform, std::process::id())?;
            fs::write(workspace.path.join("install.sh"), script)?;
            let status = Command::new("/bin/sh").arg("-n").arg(workspace.path.join("install.sh")).status()?;
            if !status.success() { return Err(invalid("invalid update helper")); }
            Ok(Prepared { workspace })
        }
    }

    pub struct Prepared { workspace: Workspace }
    impl Prepared {
        /// Only request game exit after the helper has started successfully and acknowledged readiness.
        pub fn launch(mut self) -> bool {
            let child = Command::new("/bin/sh").arg(self.workspace.path.join("install.sh"))
                .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
            let Ok(mut child) = child else { return false; };
            for _ in 0..100 {
                if self.workspace.path.join("ready").is_file() {
                    self.workspace.retained = true;
                    // Reap the detached helper if this game remains alive long enough.
                    std::thread::spawn(move || { let _ = child.wait(); });
                    return true;
                }
                if !matches!(child.try_wait(), Ok(None)) { return false; }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            let _ = child.kill();
            let _ = child.wait();
            false
        }
    }

    fn invalid(message: &str) -> io::Error { io::Error::new(io::ErrorKind::InvalidData, message) }
    fn tar() -> &'static str { if cfg!(target_os = "macos") { "/usr/bin/tar" } else { "tar" } }
    fn valid_entry(name: &str, platform: Platform) -> bool {
        let path = Path::new(name);
        let mut components = path.components();
        components.next() == Some(Component::Normal(std::ffi::OsStr::new(platform.root())))
            && components.all(|part| matches!(part, Component::Normal(_)))
            && !name.contains(['\\', '\r', '\n'])
            && !name.split('/').any(|part| part == "." || part == "..")
    }
    fn validate_listing(archive: &Path, platform: Platform) -> io::Result<()> {
        let output = Command::new(tar()).arg("-tf").arg(archive).output()?;
        let names = std::str::from_utf8(&output.stdout).map_err(|_| invalid("invalid archive names"))?;
        if !output.status.success() || names.is_empty() || !names.lines().all(|name| valid_entry(name, platform)) {
            return Err(invalid("unexpected archive layout"));
        }
        // Reject links and special files BEFORE extraction, including symlink traversal attacks.
        let output = Command::new(tar()).arg("-tvf").arg(archive).output()?;
        let listing = std::str::from_utf8(&output.stdout).map_err(|_| invalid("invalid archive entries"))?;
        if !output.status.success() || listing.lines().count() != names.lines().count()
            || !listing.lines().all(|line| matches!(line.as_bytes().first(), Some(b'-' | b'd'))) {
            return Err(invalid("archive contains links or special files"));
        }
        Ok(())
    }
    fn reject_links(path: &Path) -> io::Result<()> {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.is_dir() {
            for entry in fs::read_dir(path)? { reject_links(&entry?.path())?; }
        } else if !metadata.is_file() { return Err(invalid("unsafe extracted entry")); }
        Ok(())
    }
    fn validate_binary(executable: &Path, platform: Platform) -> io::Result<()> {
        use std::io::{Read, Seek, SeekFrom};
        let metadata = fs::metadata(executable)?;
        if !metadata.is_file() || metadata.len() < 1024 { return Err(invalid("truncated game executable")); }
        let mut header = [0u8; 64];
        let mut file = fs::File::open(executable)?;
        file.read_exact(&mut header)?;
        let binary_ok = match platform {
            Platform::Mac => matches!(&header[..4],
                [0xca, 0xfe, 0xba, 0xbe] | [0xca, 0xfe, 0xba, 0xbf] | [0xcf, 0xfa, 0xed, 0xfe]),
            Platform::Linux | Platform::AppImage => header[..4] == *b"\x7fELF" && header[4] == 2 && header[5] == 1
                && u16::from_le_bytes([header[18], header[19]]) == match std::env::consts::ARCH {
                    "x86_64" => 62, "aarch64" => 183, _ => 0,
                },
        };
        if !binary_ok || (platform == Platform::AppImage && header[8..11] != *b"AI\x02") {
            return Err(invalid("wrong executable architecture or format"));
        }
        #[cfg(target_os = "macos")]
        if platform == Platform::Mac {
            let architecture = if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" };
            if !Command::new("/usr/bin/lipo").arg(executable)
                .args(["-verify_arch", architecture]).output()?.status.success() {
                return Err(invalid("app does not support this CPU"));
            }
        }
        if platform != Platform::Mac {
            let phoff = u64::from_le_bytes(header[32..40].try_into().unwrap());
            let entry_size = u16::from_le_bytes(header[54..56].try_into().unwrap()) as u64;
            let entries = u16::from_le_bytes(header[56..58].try_into().unwrap()) as u64;
            if entry_size != 56 || entries == 0 || entries > 1024 || phoff < 64
                || phoff.checked_add(entry_size * entries).is_none_or(|end| end > metadata.len()) {
                return Err(invalid("invalid ELF program headers"));
            }
            let mut loadable = false;
            for index in 0..entries {
                file.seek(SeekFrom::Start(phoff + index * entry_size))?;
                let mut entry = [0u8; 56]; file.read_exact(&mut entry)?;
                if u32::from_le_bytes(entry[..4].try_into().unwrap()) == 1 {
                    let offset = u64::from_le_bytes(entry[8..16].try_into().unwrap());
                    let size = u64::from_le_bytes(entry[32..40].try_into().unwrap());
                    if offset.checked_add(size).is_none_or(|end| end > metadata.len()) {
                        return Err(invalid("truncated ELF load segment"));
                    }
                    loadable |= size > 0;
                }
            }
            if !loadable { return Err(invalid("ELF contains no loadable segments")); }
            if platform == Platform::AppImage {
                file.seek(SeekFrom::Start(0))?;
                let mut buffer = [0u8; 65539];
                let mut carry = 0;
                let mut found = false;
                loop {
                    let count = file.read(&mut buffer[carry..])?;
                    if count == 0 { break; }
                    let length = carry + count;
                    if buffer[..length].windows(4).any(|bytes| bytes == b"hsqs") { found = true; break; }
                    carry = length.min(3);
                    buffer.copy_within(length - carry..length, 0);
                }
                if !found { return Err(invalid("AppImage is missing its filesystem")); }
            }
        }
        Ok(())
    }
    fn validate_payload(root: &Path, platform: Platform) -> io::Result<()> {
        let executable = platform.executable(root);
        validate_binary(&executable, platform)?;
        if fs::metadata(&executable)?.permissions().mode() & 0o111 == 0
            || !platform.assets(root).is_dir() || fs::read_dir(platform.assets(root))?.next().is_none()
            || (platform == Platform::Mac && !root.join("Contents/Info.plist").is_file()) {
            return Err(invalid("update does not contain an executable game and assets"));
        }
        #[cfg(target_os = "macos")]
        if platform == Platform::Mac && !Command::new("/usr/bin/codesign")
            .args(["--verify", "--deep", "--strict"]).arg(root).output()?.status.success() {
            return Err(invalid("app signature verification failed"));
        }
        Ok(())
    }
    fn quote(path: &Path) -> io::Result<String> {
        let value = path.to_str().ok_or_else(|| invalid("non-UTF-8 installation path"))?;
        Ok(format!("'{}'", value.replace('\'', "'\\''")))
    }
    fn helper_script(target: &Path, staged: &Path, work: &Path, platform: Platform, pid: u32) -> io::Result<String> {
        let launcher = match platform {
            Platform::Mac => "/usr/bin/open -n -W -a \"$target\"",
            Platform::Linux => "(cd \"$target\" && exec \"$target/t2f\")",
            Platform::AppImage => "(cd \"$(dirname \"$target\")\" && exec \"$target\")",
        };
        Ok(format!(r#"#!/bin/sh
# All paths are passed as literal shell words, never release tag fragments.
umask 077
target={target}
staged={staged}
work={work}
backup="$work/backup"
committed=0
launch() {{
    {launcher} </dev/null >/dev/null 2>&1 &
    launched=$!
    sleep 1
    kill -0 "$launched" 2>/dev/null
}}
cleanup() {{
    trap '' HUP INT TERM
    if [ "$committed" = 0 ] && [ -e "$backup" ]; then
        if [ -e "$target" ]; then
            kill "$launched" 2>/dev/null || :
            # Move aside rather than deleting until the original is restored.
            mv "$target" "$work/failed" || exit 1
        fi
        if mv "$backup" "$target"; then
            launch || :
        else
            # Preserve the backup for manual recovery if restoration is impossible.
            exit 1
        fi
    fi
    rm -rf "$work"
}}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
[ -e "$target" ] && [ -e "$staged" ] && [ -w "$(dirname "$target")" ] || exit 1
: > "$work/ready" || exit 1
remaining=120
while kill -0 {pid} 2>/dev/null; do
    [ "$remaining" -gt 0 ] || exit 1
    remaining=$((remaining - 1))
    sleep 1
done
# Both renames stay on the installation filesystem.
if ! mv "$target" "$backup"; then launch || :; exit 1; fi
if ! mv "$staged" "$target"; then exit 1; fi
if ! launch; then exit 1; fi
committed=1
exit 0
"#, target = quote(target)?, staged = quote(staged)?, work = quote(work)?))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn rejects_archive_escape_and_foreign_roots() {
            for entry in ["/RoadToAlfendrof/t2f", "RoadToAlfendrof/../bad", "RoadToAlfendrof/./t2f", "other/t2f", "RoadToAlfendrof/a\\b"] {
                assert!(!valid_entry(entry, Platform::Linux), "{entry}");
            }
            assert!(valid_entry("RoadToAlfendrof/assets/font.ttf", Platform::Linux));
            assert!(valid_entry("Road to Alfendrof.app/Contents/MacOS/t2f", Platform::Mac));
        }
        #[test]
        fn workspace_is_private_unique_and_removed() {
            let a = Workspace::new(&std::env::temp_dir()).unwrap();
            let b = Workspace::new(&std::env::temp_dir()).unwrap();
            assert_ne!(a.path, b.path);
            assert_eq!(fs::metadata(&a.path).unwrap().permissions().mode() & 0o777, 0o700);
            let path = a.path.clone(); drop(a); assert!(!path.exists());
        }
        #[test]
        fn detects_only_packaged_installations() {
            let work = Workspace::new(&std::env::temp_dir()).unwrap();
            let root = work.path.join("RoadToAlfendrof");
            fs::create_dir_all(root.join("assets")).unwrap();
            fs::write(root.join("t2f"), b"test").unwrap();
            assert!(Installation::detect(&root.join("t2f"), Platform::Linux).is_ok());
            fs::set_permissions(&root, fs::Permissions::from_mode(0o555)).unwrap();
            assert!(Installation::detect(&root.join("t2f"), Platform::Linux).is_err());
            fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
            fs::rename(&root, work.path.join("source")).unwrap();
            assert!(Installation::detect(&work.path.join("source/t2f"), Platform::Linux).is_err());
        }
        #[test]
        fn helper_rolls_back_failed_launch_with_quoted_paths() {
            let workspace = Workspace::new(&std::env::temp_dir()).unwrap();
            let target = workspace.path.join("game's installation");
            let stage = workspace.path.join("new game");
            let work = workspace.path.join("helper");
            fs::create_dir(&target).unwrap(); fs::create_dir(&stage).unwrap(); fs::create_dir(&work).unwrap();
            fs::write(target.join("marker"), "original").unwrap();
            // The old game deliberately exits too; the important guarantee is restoration.
            fs::write(target.join("t2f"), "#!/bin/sh\nexit 0\n").unwrap();
            fs::set_permissions(target.join("t2f"), fs::Permissions::from_mode(0o755)).unwrap();
            fs::write(stage.join("t2f"), "#!/bin/sh\nexit 1\n").unwrap();
            fs::set_permissions(stage.join("t2f"), fs::Permissions::from_mode(0o755)).unwrap();
            // Above the supported kernels' PID limits, but still a positive signed PID.
            let script = helper_script(&target, &stage, &work, Platform::Linux, i32::MAX as u32).unwrap();
            let helper = workspace.path.join("helper.sh"); fs::write(&helper, script).unwrap();
            assert!(!Command::new("/bin/sh").arg(helper).status().unwrap().success());
            assert_eq!(fs::read_to_string(target.join("marker")).unwrap(), "original");
            assert!(!work.exists());
        }
        #[test]
        fn helper_waits_then_replaces_and_relaunches() {
            let workspace = Workspace::new(&std::env::temp_dir()).unwrap();
            let target = workspace.path.join("RoadToAlfendrof");
            let stage = workspace.path.join("new");
            let work = workspace.path.join("helper");
            fs::create_dir(&target).unwrap(); fs::create_dir(&stage).unwrap(); fs::create_dir(&work).unwrap();
            fs::write(target.join("marker"), "original").unwrap();
            fs::write(stage.join("marker"), "new").unwrap();
            fs::write(stage.join("t2f"), "#!/bin/sh\nprintf '%s' \"$$\" > launched.pid\nexec sleep 30\n").unwrap();
            fs::set_permissions(stage.join("t2f"), fs::Permissions::from_mode(0o755)).unwrap();
            let mut original = Command::new("/bin/sleep").arg("30").spawn().unwrap();
            let script = helper_script(&target, &stage, &work, Platform::Linux, original.id()).unwrap();
            let helper = workspace.path.join("helper.sh"); fs::write(&helper, script).unwrap();
            let mut child = Command::new("/bin/sh").arg(helper).spawn().unwrap();
            for _ in 0..100 {
                if work.join("ready").exists() { break; }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            assert!(work.join("ready").exists());
            assert_eq!(fs::read_to_string(target.join("marker")).unwrap(), "original");
            original.kill().unwrap(); original.wait().unwrap();
            assert!(child.wait().unwrap().success());
            assert_eq!(fs::read_to_string(target.join("marker")).unwrap(), "new");
            let pid = fs::read_to_string(target.join("launched.pid")).unwrap();
            let _ = Command::new("/bin/kill").arg(pid).status();
            assert!(!work.exists());
        }
        #[test]
        fn helper_replaces_single_appimage_file() {
            let workspace = Workspace::new(&std::env::temp_dir()).unwrap();
            let target = workspace.path.join("game.AppImage");
            let stage = workspace.path.join("new.AppImage");
            let work = workspace.path.join("helper"); fs::create_dir(&work).unwrap();
            fs::write(&target, "old image").unwrap();
            fs::write(&stage, "#!/bin/sh\nprintf '%s' \"$$\" > launched.pid\nexec sleep 30\n").unwrap();
            fs::set_permissions(&stage, fs::Permissions::from_mode(0o755)).unwrap();
            let script = helper_script(&target, &stage, &work, Platform::AppImage, i32::MAX as u32).unwrap();
            let helper = workspace.path.join("helper.sh"); fs::write(&helper, script).unwrap();
            assert!(Command::new("/bin/sh").arg(helper).status().unwrap().success());
            assert!(fs::read_to_string(&target).unwrap().starts_with("#!/bin/sh"));
            let pid = fs::read_to_string(workspace.path.join("launched.pid")).unwrap();
            let _ = Command::new("/bin/kill").arg(pid).status();
            assert!(!work.exists());
        }
        #[test]
        fn rejects_invalid_binary_before_handoff() {
            let workspace = Workspace::new(&std::env::temp_dir()).unwrap();
            let file = workspace.path.join("bad");
            let mut data = [0u8; 2048];
            data[..4].copy_from_slice(b"\x7fELF");
            data[4] = 2; data[5] = 1;
            data[18..20].copy_from_slice(&(if std::env::consts::ARCH == "aarch64" { 183u16 } else { 62u16 }).to_le_bytes());
            fs::write(&file, data).unwrap();
            assert!(validate_binary(&file, Platform::Linux).is_err());
            assert!(validate_binary(&file, Platform::AppImage).is_err());
            assert!(validate_binary(&file, Platform::Mac).is_err());
        }
        #[test]
        fn selects_architecture_specific_release_assets() {
            assert_eq!(Platform::Mac.asset("v0.1.9").unwrap(), "RoadToAlfendrof-Mac.zip");
            assert_eq!(Platform::Linux.asset("v0.1.9").unwrap(),
                format!("RoadToAlfendrof-Linux-{}.tar.gz", std::env::consts::ARCH));
            assert_eq!(Platform::AppImage.asset("v0.1.9").unwrap(),
                format!("RoadToAlfendrof-0.1.9-{}.AppImage", std::env::consts::ARCH));
            assert!(Platform::AppImage.asset("v0.1.9/evil").is_none());
        }
        #[cfg(target_os = "macos")]
        #[test]
        fn stages_signed_native_mac_zip_and_rejects_tampering() {
            let parent = Workspace::new(&std::env::temp_dir()).unwrap();
            let root = parent.path.join(Platform::Mac.root());
            fs::create_dir_all(root.join("Contents/MacOS")).unwrap();
            fs::create_dir_all(root.join("Contents/Resources/assets")).unwrap();
            fs::write(root.join("Contents/Resources/assets/marker"), "original").unwrap();
            fs::copy(std::env::current_exe().unwrap(), root.join("Contents/MacOS/t2f")).unwrap();
            fs::write(root.join("Contents/Info.plist"), r#"<?xml version="1.0"?><plist version="1.0"><dict>
                <key>CFBundleExecutable</key><string>t2f</string>
                <key>CFBundleIdentifier</key><string>com.thracian141.updater-test</string>
                <key>CFBundlePackageType</key><string>APPL</string>
                </dict></plist>"#).unwrap();
            assert!(Command::new("/usr/bin/codesign").args(["--force", "--deep", "--sign", "-"])
                .arg(&root).output().unwrap().status.success());
            let installation = Installation::detect(&root.join("Contents/MacOS/t2f"), Platform::Mac).unwrap();
            let workspace = installation.workspace().unwrap();
            let archive = workspace.path.join("update.zip");
            assert!(Command::new("/usr/bin/ditto").args(["-c", "-k", "--keepParent", "--norsrc"])
                .arg(&root).arg(&archive).status().unwrap().success());
            let prepared = installation.prepare(workspace, &archive).unwrap();
            assert_eq!(fs::read_to_string(root.join("Contents/Resources/assets/marker")).unwrap(), "original");
            drop(prepared);
            fs::write(root.join("Contents/Resources/assets/marker"), "tampered").unwrap();
            assert!(validate_payload(&root, Platform::Mac).is_err());
        }
        #[cfg(target_os = "linux")]
        #[test]
        fn stages_valid_portable_package_without_touching_installation() {
            let parent = Workspace::new(&std::env::temp_dir()).unwrap();
            let root = parent.path.join("RoadToAlfendrof");
            fs::create_dir_all(root.join("assets")).unwrap();
            fs::write(root.join("assets/marker"), "original").unwrap();
            fs::copy("/bin/sleep", root.join("t2f")).unwrap();
            let installation = Installation::detect(&root.join("t2f"), Platform::Linux).unwrap();
            let workspace = installation.workspace().unwrap();
            let archive = workspace.path.join("update.tar.gz");
            assert!(Command::new("/usr/bin/tar").arg("-czf").arg(&archive).arg("-C").arg(&parent.path)
                .arg("RoadToAlfendrof").status().unwrap().success());
            let prepared = installation.prepare(workspace, &archive).unwrap();
            assert!(prepared.workspace.path.join("unpack/RoadToAlfendrof/t2f").is_file());
            assert_eq!(fs::read_to_string(root.join("assets/marker")).unwrap(), "original");
            let temporary = prepared.workspace.path.clone();
            drop(prepared);
            assert!(!temporary.exists());
        }
        #[test]
        fn rejects_garbage_and_symlink_archives() {
            use std::os::unix::fs::symlink;
            let work = Workspace::new(&std::env::temp_dir()).unwrap();
            let archive = work.path.join("update.tar.gz");
            fs::write(&archive, "not an archive").unwrap();
            assert!(validate_listing(&archive, Platform::Linux).is_err());
            let root = work.path.join("RoadToAlfendrof"); fs::create_dir(&root).unwrap();
            symlink("/tmp", root.join("assets")).unwrap();
            assert!(Command::new("/usr/bin/tar").arg("-czf").arg(&archive).arg("-C").arg(&work.path)
                .arg("RoadToAlfendrof").status().unwrap().success());
            assert!(validate_listing(&archive, Platform::Linux).is_err());
        }
    }
}
#[cfg(unix)]
pub use unix::{Installation, Platform, Prepared};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compares_only_stable_versions() {
        assert!(is_newer("v0.2.0", "0.1.3"));
        assert!(is_newer("0.1.10", "0.1.9"));
        for value in ["v0.1.3", "0.0.9", "v1.2", "v1.2.3-beta", "v1.2.3/evil", "vv1.2.3", "1.02.3", "1.2.4294967296", "1.2.+3", "1.2.3\n"] {
            assert!(!is_newer(value, "0.1.3"), "{value}");
        }
    }
}
