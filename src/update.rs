//! Updates the game by itself: on start it looks on GitHub for a newer release, downloads the installer in the
//! background (a spinner shows on the title screen) and then runs it quietly and restarts the game.
//! The work runs on its own thread through the curl that ships with Windows, so being offline costs nothing.

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::AppState;

/// Where releases are published.
const REPO: &str = "thracian141/road-to-alfendrof";
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
    Ready(PathBuf),
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

/// Only an installed game updates itself: the assets sit beside its exe, unlike a build run from the project.
fn is_installed() -> bool {
    // (the updater downloads the Windows installer, so other systems never update themselves)
    cfg!(windows)
        && !cfg!(debug_assertions)
        && std::env::current_exe().ok().and_then(|exe| exe.parent().map(|dir| dir.join("assets").is_dir())).unwrap_or(false)
}

fn start_update(progress: Res<Progress>) {
    if !is_installed() {
        return;
    }
    let step = progress.0.clone();
    std::thread::spawn(move || {
        let Some(release) = fetch_latest() else { return };
        if !is_newer(&release.version, env!("CARGO_PKG_VERSION")) {
            return;
        }
        *step.lock().unwrap() = Step::Downloading(release.version.clone());
        let file = std::env::temp_dir().join(format!("RoadToAlfendrof-Setup-{}.exe", release.version));
        let url = format!("https://github.com/{REPO}/releases/download/{}/{INSTALLER}", release.version);
        let done = run_hidden(Command::new("curl.exe").args(["-s", "-f", "-L", "-m", "600", "-o"]).arg(&file).arg(&url)).is_some()
            && std::fs::metadata(&file).map(|m| m.len() > 1_000_000).unwrap_or(false);
        *step.lock().unwrap() = if done { Step::Ready(file) } else { Step::Idle };
    });
}

struct Release {
    version: String,
}

fn fetch_latest() -> Option<Release> {
    // /releases/latest redirects to the newest tag's page, which costs no API quota
    let page = format!("https://github.com/{REPO}/releases/latest");
    let url = run_hidden(Command::new("curl.exe").args(["-s", "-L", "-m", "6", "-o", "NUL", "-w", "%{url_effective}", &page]))?;
    let version = url.rsplit("/tag/").next().filter(|_| url.contains("/tag/"))?.trim().to_string();
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

fn parse_version(v: &str) -> Vec<u32> {
    v.trim_start_matches('v').split('.').map(|part| part.parse().unwrap_or(0)).collect()
}

fn is_newer(candidate: &str, current: &str) -> bool {
    parse_version(candidate) > parse_version(current)
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
    if let Step::Ready(file) = &*step {
        let started = run_installer(file);
        *step = Step::Idle; // once only, even if it failed to start
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

/// Quiet install over the current one, then the installer starts the game again.
fn run_installer(file: &std::path::Path) -> bool {
    let mut cmd = Command::new(file);
    cmd.args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/CLOSEAPPLICATIONS", "/relaunch=1"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd.spawn().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer("v0.2.0", "0.1.0"));
        assert!(is_newer("0.1.10", "0.1.9"));
        assert!(!is_newer("v0.1.0", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }
}
