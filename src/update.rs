//! Looks on GitHub for a newer release when the game starts and says so on the title screen.
//! The request runs on its own thread through the curl that ships with Windows, so being offline costs nothing.

use std::process::Command;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::AppState;

/// Where releases are published.
const REPO: &str = "thracian141/road-to-alfendrof";

pub struct UpdatePlugin;

impl Plugin for UpdatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Latest>()
            .add_systems(Startup, check_for_update)
            .add_systems(Update, update_notice.run_if(in_state(AppState::Title)));
    }
}

struct Release {
    version: String,
    url: String,
}

#[derive(Resource, Default)]
struct Latest(Arc<Mutex<Option<Release>>>);

#[derive(Component)]
struct UpdateText;

fn check_for_update(latest: Res<Latest>) {
    let slot = latest.0.clone();
    std::thread::spawn(move || {
        if let Some(release) = fetch_latest() {
            if is_newer(&release.version, env!("CARGO_PKG_VERSION")) {
                *slot.lock().unwrap() = Some(release);
            }
        }
    });
}

fn fetch_latest() -> Option<Release> {
    // /releases/latest redirects to the newest tag's page, which costs no API quota
    let page = format!("https://github.com/{REPO}/releases/latest");
    let url = run_hidden(Command::new("curl.exe").args(["-s", "-L", "-m", "6", "-o", "NUL", "-w", "%{url_effective}", &page]))?;
    let version = url.rsplit("/tag/").next().filter(|_| url.contains("/tag/"))?.trim().to_string();
    Some(Release { version, url })
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

/// Shows the notice once the answer is in; U opens the download page.
fn update_notice(
    mut commands: Commands,
    latest: Res<Latest>,
    keys: Res<ButtonInput<KeyCode>>,
    shown: Query<(), With<UpdateText>>,
) {
    let guard = latest.0.lock().unwrap();
    let Some(release) = guard.as_ref() else { return };
    if shown.is_empty() {
        commands.spawn((
            DespawnOnExit(AppState::Title),
            UpdateText,
            Text2d::new(format!("New version {} is out - press U to download", release.version)),
            crate::pixel_font(8.0),
            TextColor(Color::srgb(0.6, 1.0, 0.6)),
            Transform::from_xyz(0.0, -100.0, 6.0),
        ));
    }
    if keys.just_pressed(KeyCode::KeyU) {
        open_page(&release.url);
    }
}

#[cfg(windows)]
fn open_page(url: &str) {
    use std::os::windows::process::CommandExt;
    let _ = Command::new("cmd").args(["/c", "start", "", url]).creation_flags(0x0800_0000).spawn();
}

#[cfg(not(windows))]
fn open_page(url: &str) {
    let _ = Command::new("xdg-open").arg(url).spawn();
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
