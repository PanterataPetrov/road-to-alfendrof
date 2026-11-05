//! Music. In the menus `assets/music/menu.wav` plays from the start once (the buildup), then loops `loop_start` to
//! `loop_end` seamlessly; the echo after `loop_end` is mixed over the start of each repeat instead of played as
//! a gap. From the countdown of a fight until it is left, `assets/music/generic.wav` loops instead; the menu song
//! comes back, without the buildup, in the menus after it.

use std::num::NonZero;
use std::sync::Arc;
use std::time::Duration;

use bevy::audio::{AddAudioSource, AudioSinkPlayback, ChannelCount, Decodable, Sample, SampleRate, Source, Volume};
use bevy::prelude::*;

use t2f::sim::Phase;

use crate::settings::Settings;
use crate::{AppState, Sim};

/// A song file and where it loops.
struct Track {
    file: &'static str,
    /// Where the loop starts: everything before it is the buildup, heard only once.
    loop_start: Duration,
    /// Where the music ends and jumps back to `loop_start`; after it the file only has the fading echo.
    loop_end: Duration,
}

const MENU: Track = Track { file: "music/menu.wav", loop_start: Duration::from_secs(2), loop_end: Duration::from_secs(42) };
/// (Loops at 1:28; the fade-out that starts at 1:29 carries on over the beginning of the next pass.)
const FIGHT: Track = Track { file: "music/generic.wav", loop_start: Duration::ZERO, loop_end: Duration::from_secs(88) };

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<Music>()
            .add_systems(Startup, load_songs)
            .add_systems(Update, (choose_music, set_music_volume).chain());
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

/// The decoded songs, kept so they can restart.
#[derive(Resource, Default)]
struct Songs {
    menu: Option<Music>,
    fight: Option<Music>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Which {
    Menu,
    Fight,
}

#[derive(Component)]
struct MusicPlayer(Which);

fn load_songs(mut commands: Commands) {
    let load = |track: &Track| {
        let path = crate::assets_dir().join(track.file);
        match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|bytes| parse_wav(&bytes, track)) {
            Ok(music) => Some(music),
            Err(e) => {
                warn!("no music ({}): {e}", path.display());
                None
            }
        }
    };
    commands.insert_resource(Songs { menu: load(&MENU), fight: load(&FIGHT) });
}

/// Menu song in the menus (and on the skills screen before a match), fight song from the countdown on.
fn choose_music(
    mut commands: Commands,
    state: Res<State<AppState>>,
    sim: Res<Sim>,
    songs: Res<Songs>,
    mut assets: ResMut<Assets<Music>>,
    settings: Res<Settings>,
    players: Query<(Entity, &MusicPlayer)>,
    mut heard_buildup: Local<bool>,
) {
    let fighting = *state.get() == AppState::Fight && !matches!(sim.0.phase, Phase::Briefing(_));
    let want = if fighting { Which::Fight } else { Which::Menu };
    for (player, MusicPlayer(which)) in &players {
        if *which != want {
            commands.entity(player).despawn();
        }
    }
    if players.iter().any(|(_, MusicPlayer(which))| *which == want) {
        return;
    }
    let Some(song) = (match want {
        Which::Menu => songs.menu.clone(),
        Which::Fight => songs.fight.clone(),
    }) else {
        return;
    };
    // the menu song's buildup is only for the first time it plays
    let song = if want == Which::Menu && *heard_buildup { Music { start: song.loop_start, ..song } } else { song };
    *heard_buildup |= want == Which::Menu;
    commands.spawn((MusicPlayer(want), AudioPlayer(assets.add(song)), playback(&settings)));
}

fn playback(settings: &Settings) -> PlaybackSettings {
    PlaybackSettings::LOOP.with_volume(Volume::Linear(settings.music_gain()))
}

/// Options > Music volume, heard while it is changed.
fn set_music_volume(settings: Res<Settings>, mut sinks: Query<&mut AudioSink, With<MusicPlayer>>) {
    if settings.is_changed() {
        for mut sink in &mut sinks {
            sink.set_volume(Volume::Linear(settings.music_gain()));
        }
    }
}

/// Reads an uncompressed 16-bit PCM WAV.
fn parse_wav(bytes: &[u8], track: &Track) -> Result<Music, String> {
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
    let loop_end = frame_index(track.loop_end).min(samples.len());
    let loop_start = frame_index(track.loop_start).min(loop_end.saturating_sub(channels.get() as usize));
    Ok(Music { samples, channels, rate, loop_start, loop_end, start: 0 })
}
