//! Checks GitHub releases in the background and hands validated updates to a platform installer.

use std::process::Command;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::AppState;

#[path = "update_install.rs"]
mod install;

/// Where releases are published.
const REPO: &str = "thracian141/road-to-alfendrof";
#[cfg(windows)]
const INSTALLER: &str = "RoadToAlfendrof-Setup.exe";

pub struct UpdatePlugin;

impl Plugin for UpdatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Progress>()
            .add_systems(Startup, start_update)
            .add_systems(Update, update_notice.run_if(in_state(AppState::Title)));
    }
}

enum Step {
    Idle,
    Downloading(String),
    Ready(ReadyUpdate),
}

enum ReadyUpdate {
    #[cfg(windows)]
    Windows(install::Workspace),
    #[cfg(unix)]
    Unix(install::Prepared),
}

#[derive(Resource)]
struct Progress(Arc<Mutex<Step>>);

impl Default for Progress {
    fn default() -> Self {
        Progress(Arc::new(Mutex::new(Step::Idle)))
    }
}

#[derive(Component)]
struct UpdateUi;

#[derive(Component)]
struct SpinnerDot(usize);

/// Debug/source builds never update themselves; package detection also checks write access.
fn start_update(progress: Res<Progress>) {
    if cfg!(debug_assertions) { return; }
    #[cfg(windows)]
    if !std::env::current_exe().ok().and_then(|exe| exe.parent().map(|dir| dir.join("assets").is_dir())).unwrap_or(false) {
        return;
    }
    #[cfg(unix)]
    let installation = {
        #[cfg(target_os = "macos")]
        let platform = install::Platform::Mac;
        #[cfg(not(target_os = "macos"))]
        let platform = install::Platform::Linux;
        let Ok(executable) = std::env::current_exe() else { return };
        let Ok(installation) = install::Installation::detect(&executable, platform) else { return };
        installation
    };
    let step = progress.0.clone();
    std::thread::spawn(move || {
        let Some(release) = fetch_latest() else { return };
        if !install::is_newer(&release.version, env!("CARGO_PKG_VERSION")) { return; }
        #[cfg(windows)]
        let (workspace, asset) = {
            let Ok(workspace) = install::Workspace::new(&std::env::temp_dir()) else { return };
            (workspace, INSTALLER.to_string())
        };
        #[cfg(unix)]
        let (workspace, asset) = {
            let Ok(workspace) = installation.workspace() else { return };
            let Some(asset) = installation.platform.asset(&release.version) else { return };
            (workspace, asset)
        };
        *step.lock().unwrap() = Step::Downloading(release.version.clone());
        let file = workspace.path.join(&asset);
        let url = format!("https://github.com/{REPO}/releases/download/{}/{asset}", release.version);
        let downloaded = run_hidden(Command::new(curl()).args([
            "--silent", "--show-error", "--fail", "--location", "--proto", "=https",
            "--proto-redir", "=https", "--connect-timeout", "10", "--max-time", "600", "--output",
        ]).arg(&file).arg(&url)).is_some();
        let ready = if downloaded {
            #[cfg(windows)]
            {
                use std::io::Read;
                let mut magic = [0; 2];
                let valid = std::fs::File::open(&file).and_then(|mut f| f.read_exact(&mut magic)).is_ok()
                    && magic == *b"MZ"
                    && std::fs::metadata(&file).map(|m| m.len() > 1_000_000).unwrap_or(false);
                if valid { Some(ReadyUpdate::Windows(workspace)) } else { None }
            }
            #[cfg(unix)]
            { installation.prepare(workspace, &file).ok().map(ReadyUpdate::Unix) }
        } else { None };
        *step.lock().unwrap() = ready.map(Step::Ready).unwrap_or(Step::Idle);
    });
}

fn curl() -> &'static str {
    if cfg!(windows) { "curl.exe" } else if cfg!(target_os = "macos") { "/usr/bin/curl" } else { "curl" }
}

struct Release {
    version: String,
}

fn fetch_latest() -> Option<Release> {
    // /releases/latest redirects to the newest tag's page, which costs no API quota
    let page = format!("https://github.com/{REPO}/releases/latest");
    let url = run_hidden(Command::new(curl()).args([
        "--silent", "--fail", "--location", "--proto", "=https", "--proto-redir", "=https",
        "--max-time", "6", "--output", if cfg!(windows) { "NUL" } else { "/dev/null" },
        "--write-out", "%{url_effective}", &page,
    ]))?;
    let prefix = format!("https://github.com/{REPO}/releases/tag/");
    let version = url.strip_prefix(&prefix)?.to_string();
    install::version(&version)?;
    Some(Release { version })
}

#[cfg(windows)]
fn run_hidden(cmd: &mut Command) -> Option<String> {
    use std::os::windows::process::CommandExt;
    let out = cmd.creation_flags(0x0800_0000).output().ok()?; // no console window
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(not(windows))]
fn run_hidden(cmd: &mut Command) -> Option<String> {
    let out = cmd.output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}


/// A ring of dots with one bright dot going round, plus a line saying what is happening. Once the installer is
/// downloaded it is started quietly and the game closes; the installer restarts it.
fn update_notice(
    mut commands: Commands,
    progress: Res<Progress>,
    time: Res<Time>,
    ui: Query<Entity, With<UpdateUi>>,
    mut dots: Query<(&SpinnerDot, &mut Sprite)>,
    mut exit: MessageWriter<AppExit>,
) {
    let mut step = progress.0.lock().unwrap();
    if matches!(&*step, Step::Ready(_)) {
        let Step::Ready(ready) = std::mem::replace(&mut *step, Step::Idle) else { unreachable!() };
        let started = run_installer(ready);
        if started {
            exit.write(AppExit::Success);
        }
        return;
    }
    match &*step {
        Step::Idle => {
            for e in &ui {
                commands.entity(e).despawn();
            }
        }
        Step::Downloading(version) => {
            if ui.is_empty() {
                commands.spawn((
                    DespawnOnExit(AppState::Title),
                    UpdateUi,
                    Text2d::new(format!("Updating to {version}...")),
                    crate::pixel_font(8.0),
                    TextColor(Color::srgb(0.6, 1.0, 0.6)),
                    Transform::from_xyz(8.0, -103.0, 6.0),
                ));
                for i in 0..8 {
                    let angle = i as f32 / 8.0 * std::f32::consts::TAU;
                    commands.spawn((
                        DespawnOnExit(AppState::Title),
                        UpdateUi,
                        SpinnerDot(i),
                        Sprite::from_color(Color::srgb(0.6, 1.0, 0.6), Vec2::splat(2.0)),
                        Transform::from_xyz(-60.0 + angle.sin() * 4.0, -103.0 + angle.cos() * 4.0, 6.0),
                    ));
                }
            }
            let head = (time.elapsed_secs() * 10.0) as usize % 8;
            for (dot, mut sprite) in &mut dots {
                let behind = (head + 8 - dot.0) % 8;
                sprite.color = Color::srgba(0.6, 1.0, 0.6, 1.0 - behind as f32 / 8.0 * 0.85);
            }
        }
        Step::Ready(_) => {}
    }
}

/// The Unix helper acknowledges startup before this process is asked to exit.
fn run_installer(ready: ReadyUpdate) -> bool {
    match ready {
        #[cfg(unix)]
        ReadyUpdate::Unix(prepared) => prepared.launch(),
        #[cfg(windows)]
        ReadyUpdate::Windows(workspace) => {
            use std::os::windows::process::CommandExt;
            let mut cmd = Command::new(workspace.path.join(INSTALLER));
            cmd.args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/CLOSEAPPLICATIONS", "/relaunch=1"]);
            cmd.creation_flags(0x0800_0000);
            match cmd.spawn() {
                Ok(mut child) => {
                    std::thread::spawn(move || { let _ = child.wait(); drop(workspace); });
                    true
                }
                Err(_) => false,
            }
        }
    }
}
