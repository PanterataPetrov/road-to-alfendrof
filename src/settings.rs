//! Player settings from the Options menu: music volume, display mode and the fight keys. Saved to
//! `%APPDATA%\Road to Alfendrof\settings.ron` (`~/Library/Application Support/...` on a Mac) whenever they change. F11 toggles fullscreen anywhere.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, VideoModeSelection, WindowMode, WindowResized};
use serde::{Deserialize, Serialize};
use t2f::sim::Input;

/// The fight actions, in the order the Controls menu lists them, with their default keys.
pub const ACTIONS: [(&str, u16, KeyCode); 10] = [
    ("Left", Input::LEFT, KeyCode::KeyA),
    ("Right", Input::RIGHT, KeyCode::KeyD),
    ("Jump", Input::UP, KeyCode::KeyW),
    ("Crouch", Input::DOWN, KeyCode::KeyS),
    ("Light", Input::LIGHT, KeyCode::KeyF),
    ("Heavy", Input::HEAVY, KeyCode::KeyG),
    ("Special", Input::SPECIAL, KeyCode::KeyH),
    ("Block", Input::BLOCK, KeyCode::Space),
    ("Roll left", Input::ROLL_LEFT, KeyCode::KeyQ),
    ("Roll right", Input::ROLL_RIGHT, KeyCode::KeyE),
];

/// Keys a fight action can be bound to (Escape leaves a fight and F1/F11 are taken).
const BINDABLE: &[KeyCode] = {
    use KeyCode::*;
    &[
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR,
        KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ, Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6,
        Digit7, Digit8, Digit9, ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Space, Enter, Tab, Backspace,
        ShiftLeft, ShiftRight, ControlLeft, ControlRight, AltLeft, AltRight, Comma, Period, Slash, Semicolon,
        Quote, BracketLeft, BracketRight, Backslash, Minus, Equal, Backquote, Insert, Delete, Home, End, PageUp,
        PageDown, Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
        NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide, NumpadDecimal, NumpadEnter, F2, F3, F4, F5, F6,
        F7, F8, F9, F10, F12,
    ]
};

pub fn bindable(key: KeyCode) -> bool {
    BINDABLE.contains(&key)
}

/// Short name for menus and the skill bar: "F", "4", "Up", "Space", "ShiftLeft".
pub fn key_name(key: KeyCode) -> String {
    let name = format!("{key:?}");
    ["Key", "Digit", "Arrow"].iter().find_map(|p| name.strip_prefix(p)).unwrap_or(&name).to_string()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Display {
    #[default]
    Windowed,
    /// A borderless window covering the screen.
    Borderless,
    /// Exclusive fullscreen: the monitor switches to the game.
    Fullscreen,
}

impl Display {
    pub const ALL: [Display; 3] = [Display::Windowed, Display::Borderless, Display::Fullscreen];

    pub fn label(self) -> &'static str {
        match self {
            Display::Windowed => "Windowed",
            Display::Borderless => "Borderless",
            Display::Fullscreen => "Fullscreen",
        }
    }
}

#[derive(Resource, Clone, PartialEq, Debug)]
pub struct Settings {
    /// 0 to 10.
    pub music_volume: u8,
    pub display: Display,
    /// One key per entry of `ACTIONS`.
    pub keys: [KeyCode; ACTIONS.len()],
}

impl Default for Settings {
    fn default() -> Self {
        Settings { music_volume: 7, display: Display::Windowed, keys: ACTIONS.map(|(_, _, key)| key) }
    }
}

impl Settings {
    pub const MAX_VOLUME: u8 = 10;

    /// The volume to play music at (squared, so the steps sound even).
    pub fn music_gain(&self) -> f32 {
        (self.music_volume as f32 / Self::MAX_VOLUME as f32).powi(2)
    }

    pub fn key(&self, flag: u16) -> KeyCode {
        let i = ACTIONS.iter().position(|a| a.1 == flag).expect("action");
        self.keys[i]
    }

    /// Bind action `i` to `key`; an action that had that key gets this one's old key.
    pub fn bind(&mut self, i: usize, key: KeyCode) {
        if let Some(other) = self.keys.iter().position(|&k| k == key) {
            self.keys[other] = self.keys[i];
        }
        self.keys[i] = key;
    }

    /// The buttons held right now.
    pub fn read_input(&self, keys: &ButtonInput<KeyCode>) -> Input {
        let mut bits = 0;
        for (i, (_, flag, _)) in ACTIONS.iter().enumerate() {
            if keys.pressed(self.keys[i]) {
                bits |= flag;
            }
        }
        Input(bits)
    }
}

/// What goes in the file; keys by name so the file stays readable.
#[derive(Serialize, Deserialize)]
struct Saved {
    music_volume: u8,
    display: Display,
    keys: Vec<(String, String)>,
}

fn settings_path() -> PathBuf {
    // Windows: %APPDATA%; macOS: ~/Library/Application Support; elsewhere: ~/.config
    let dir = std::env::var_os("APPDATA").map(PathBuf::from).or_else(|| {
        let home = PathBuf::from(std::env::var_os("HOME")?);
        Some(if cfg!(target_os = "macos") { home.join("Library").join("Application Support") } else { home.join(".config") })
    });
    match dir {
        Some(dir) => dir.join("Road to Alfendrof").join("settings.ron"),
        None => PathBuf::from("settings.ron"),
    }
}

pub fn load() -> Settings {
    let mut settings = Settings::default();
    let Ok(text) = std::fs::read_to_string(settings_path()) else { return settings };
    let saved: Saved = match ron::from_str(&text) {
        Ok(saved) => saved,
        Err(e) => {
            warn!("ignoring unreadable settings: {e}");
            return settings;
        }
    };
    settings.music_volume = saved.music_volume.min(Settings::MAX_VOLUME);
    settings.display = saved.display;
    for (action, key) in saved.keys {
        let i = ACTIONS.iter().position(|a| a.0 == action);
        let key = BINDABLE.iter().copied().find(|k| format!("{k:?}") == key);
        if let (Some(i), Some(key)) = (i, key) {
            settings.bind(i, key);
        }
    }
    settings
}

fn save(settings: &Settings) {
    let saved = Saved {
        music_volume: settings.music_volume,
        display: settings.display,
        keys: ACTIONS.iter().zip(settings.keys).map(|(a, k)| (a.0.to_string(), format!("{k:?}"))).collect(),
    };
    let path = settings_path();
    let result = ron::ser::to_string_pretty(&saved, ron::ser::PrettyConfig::default())
        .map_err(|e| e.to_string())
        .and_then(|text| {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            std::fs::write(&path, text).map_err(|e| e.to_string())
        });
    if let Err(e) = result {
        warn!("could not save settings to {}: {e}", path.display());
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load())
            .add_systems(Update, (toggle_fullscreen, apply_settings, fit_fullscreen_scale).chain());
    }
}

fn toggle_fullscreen(keys: Res<ButtonInput<KeyCode>>, mut settings: ResMut<Settings>) {
    if keys.just_pressed(KeyCode::F11) {
        settings.display = if settings.display == Display::Windowed { Display::Borderless } else { Display::Windowed };
    }
}

/// Save on every change and put the window in the chosen mode (the music volume is applied in `music`).
fn apply_settings(settings: Res<Settings>, mut window: Single<&mut Window, With<PrimaryWindow>>, mut last: Local<Option<Settings>>) {
    if !settings.is_changed() || last.as_ref() == Some(&*settings) {
        return;
    }
    // the first run is the loaded file, nothing new to save
    if last.is_some() {
        save(&settings);
    }
    let mode = match settings.display {
        Display::Windowed => WindowMode::Windowed,
        Display::Borderless => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        Display::Fullscreen => WindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current),
    };
    if window.mode != mode {
        window.mode = mode;
        if mode == WindowMode::Windowed {
            // back to the usual 3x window
            window.resolution.set_scale_factor_override(Some(crate::SCREEN_SCALE));
            window.resolution.set(crate::WINDOW_W, crate::WINDOW_H);
        }
    }
    *last = Some(settings.clone());
}

/// Fullscreen, text is drawn at the screen's own scale (the pixel-art view times however much it is
/// enlarged) instead of being made for the 3x window and stretched.
fn fit_fullscreen_scale(mut resized: MessageReader<WindowResized>, mut window: Single<&mut Window, With<PrimaryWindow>>) {
    if resized.read().last().is_none() || window.mode == WindowMode::Windowed {
        return;
    }
    let (w, h) = (window.physical_width() as f32, window.physical_height() as f32);
    let scale = (w / crate::VIEW_W).min(h / crate::VIEW_H).max(1.0);
    if window.resolution.scale_factor_override() != Some(scale) {
        window.resolution.set_scale_factor_override(Some(scale));
    }
}
