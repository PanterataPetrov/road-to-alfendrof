//! Menu music: `assets/music/menu.wav` plays from the start once (the buildup), then loops `LOOP_START` to
//! `LOOP_END` seamlessly; the echo after `LOOP_END` is mixed over the start of each repeat instead of played as
//! a gap. It stops when a fight starts and comes back, without the buildup, in the menus after it.

use std::num::NonZero;
use std::sync::Arc;
use std::time::Duration;

use bevy::audio::{AddAudioSource, AudioSinkPlayback, ChannelCount, Decodable, Sample, SampleRate, Source, Volume};
use bevy::prelude::*;

use crate::settings::Settings;
use crate::AppState;

/// Where the loop starts: everything before it is the buildup, heard only once.
const LOOP_START: Duration = Duration::from_secs(2);
/// Where the music ends and jumps back to `LOOP_START`; after it the file only has the fading echo.
const LOOP_END: Duration = Duration::from_secs(42);

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<Music>()
            .add_systems(Startup, load_menu_music)
            .add_systems(OnEnter(AppState::Fight), stop_menu_music)
            .add_systems(OnExit(AppState::Fight), resume_menu_music)
            .add_systems(Update, set_music_volume);
    }
}

/// Decoded PCM that loops from `loop_start` to `loop_end` (sample indices) and can begin past the buildup.
#[derive(Asset, TypePath, Clone)]
pub struct Music {
    /// 16-bit samples as stored in the file, turned into floats as they play (half the memory of holding floats)
    samples: Arc<[i16]>,
    channels: ChannelCount,
    rate: SampleRate,
    loop_start: usize,
    loop_end: usize,
    start: usize,
}

impl Decodable for Music {
    type Decoder = MusicDecoder;

    fn decoder(&self) -> MusicDecoder {
        MusicDecoder { music: self.clone(), pos: self.start, tail: None }
    }
}

pub struct MusicDecoder {
    music: Music,
    pos: usize,
    /// Where in the echo after `loop_end` we are, while it plays over a repeat.
    tail: Option<usize>,
}

impl Iterator for MusicDecoder {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        let samples = &self.music.samples;
        if self.pos >= self.music.loop_end {
            self.pos = self.music.loop_start;
            self.tail = Some(self.music.loop_end);
        }
        let mut sample = *samples.get(self.pos)? as Sample / 32768.0;
        self.pos += 1;
        if let Some(t) = self.tail {
            sample = (sample + samples[t] as Sample / 32768.0).clamp(-1.0, 1.0);
            self.tail = (t + 1 < samples.len()).then_some(t + 1);
        }
        Some(sample)
    }
}

impl Source for MusicDecoder {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        self.music.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.music.rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// The decoded menu song, kept so it can restart after a fight.
#[derive(Resource)]
struct MenuMusic(Music);

#[derive(Component)]
struct MenuMusicPlayer;

fn load_menu_music(mut commands: Commands, mut songs: ResMut<Assets<Music>>, settings: Res<Settings>) {
    let path = crate::assets_dir().join("music/menu.wav");
    let music = match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|bytes| parse_wav(&bytes)) {
        Ok(music) => music,
        Err(e) => {
            warn!("no menu music ({}): {e}", path.display());
            return;
        }
    };
    commands.insert_resource(MenuMusic(music.clone()));
    commands.spawn((MenuMusicPlayer, AudioPlayer(songs.add(music)), playback(&settings)));
}

/// Back in the menus after a fight: straight into the loop, no buildup.
fn resume_menu_music(mut commands: Commands, music: Option<Res<MenuMusic>>, mut songs: ResMut<Assets<Music>>, settings: Res<Settings>) {
    let Some(music) = music else { return };
    let song = Music { start: music.0.loop_start, ..music.0.clone() };
    commands.spawn((MenuMusicPlayer, AudioPlayer(songs.add(song)), playback(&settings)));
}

fn playback(settings: &Settings) -> PlaybackSettings {
    PlaybackSettings::LOOP.with_volume(Volume::Linear(settings.music_gain()))
}

/// Options > Music volume, heard while it is changed.
fn set_music_volume(settings: Res<Settings>, mut sinks: Query<&mut AudioSink, With<MenuMusicPlayer>>) {
    if settings.is_changed() {
        for mut sink in &mut sinks {
            sink.set_volume(Volume::Linear(settings.music_gain()));
        }
    }
}

fn stop_menu_music(mut commands: Commands, players: Query<Entity, With<MenuMusicPlayer>>) {
    for player in &players {
        commands.entity(player).despawn();
    }
}

/// Reads an uncompressed 16-bit PCM WAV.
fn parse_wav(bytes: &[u8]) -> Result<Music, String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("not a WAV file".into());
    }
    let u16_at = |i: usize| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
    let u32_at = |i: usize| u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
    let mut format = None;
    let mut data = None;
    let mut at = 12;
    while at + 8 <= bytes.len() {
        let len = u32_at(at + 4) as usize;
        let body = at + 8..(at + 8 + len).min(bytes.len());
        match &bytes[at..at + 4] {
            b"fmt " if len >= 16 => format = Some((u16_at(at + 8), u16_at(at + 10), u32_at(at + 12), u16_at(at + 22))),
            b"data" => data = Some(&bytes[body]),
            _ => {}
        }
        // chunks are padded to an even length
        at += 8 + len + (len & 1);
    }
    let (Some((kind, channels, rate, bits)), Some(data)) = (format, data) else {
        return Err("missing fmt or data chunk".into());
    };
    if kind != 1 || bits != 16 {
        return Err(format!("only 16-bit PCM is supported (format {kind}, {bits} bits)"));
    }
    let channels = NonZero::new(channels).ok_or("no channels")?;
    let rate = NonZero::new(rate).ok_or("no sample rate")?;
    let samples: Arc<[i16]> = data.chunks_exact(2).map(|s| i16::from_le_bytes([s[0], s[1]])).collect();
    // on a whole frame (one sample per channel)
    let frame_index = |at: Duration| (at.as_secs_f64() * rate.get() as f64).round() as usize * channels.get() as usize;
    let loop_end = frame_index(LOOP_END).min(samples.len());
    let loop_start = frame_index(LOOP_START).min(loop_end.saturating_sub(channels.get() as usize));
    Ok(Music { samples, channels, rate, loop_start, loop_end, start: 0 })
}
