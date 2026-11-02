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
    let api = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let out = run_hidden(Command::new("curl.exe").args(["-s", "-f", "-m", "6", "-H", "User-Agent: road-to-alfendrof", &api]))?;
    Some(Release { version: json_field(&out, "tag_name")?, url: json_field(&out, "html_url")? })
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

/// The first `"key": "value"` in a JSON text.
fn json_field(json: &str, key: &str) -> Option<String> {
    let after = &json[json.find(&format!("\"{key}\""))? + key.len() + 2..];
    let after = &after[after.find('"')? + 1..];
    Some(after[..after.find('"')?].to_string())
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

    #[test]
    fn reads_fields() {
        let json = r#"{"url":"x","tag_name": "v1.2.3","html_url":"https://github.com/a/b/releases/tag/v1.2.3"}"#;
        assert_eq!(json_field(json, "tag_name").as_deref(), Some("v1.2.3"));
        assert_eq!(json_field(json, "html_url").as_deref(), Some("https://github.com/a/b/releases/tag/v1.2.3"));
    }
}
