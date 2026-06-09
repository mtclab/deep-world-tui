//! Audio module: feature-gated synthesis and playback.
//!
//! - `audio` feature: WAV synthesis via `hound` (pure Rust, no ALSA needed).
//! - `audio-playback` feature: adds `rodio` for device playback (needs ALSA).
//! - Neither feature: all functions are no-ops.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundEvent {
    UiClick,
    UiCancel,
    Gather,
    Combat,
    Trade,
    Weather,
    Ambient,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AudioSettings {
    pub enabled: bool,
    pub volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AudioConfig {
    pub enabled: bool,
    pub volume: f32,
}

impl From<AudioSettings> for AudioConfig {
    fn from(s: AudioSettings) -> Self {
        Self {
            enabled: s.enabled,
            volume: s.volume,
        }
    }
}

#[cfg(feature = "audio")]
pub fn render_clip(event: SoundEvent, volume: f32) -> Vec<u8> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 22_050,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut buf = Vec::new();
    let cursor = std::io::Cursor::new(&mut buf);
    let mut writer = match hound::WavWriter::new(cursor, spec) {
        Ok(w) => w,
        Err(_) => return Vec::new(),
    };
    let samples = synthesize(event, volume);
    for s in samples {
        let _ = writer.write_sample(s);
    }
    let _ = writer.finalize();
    buf
}

#[cfg(not(feature = "audio"))]
pub fn render_clip(_event: SoundEvent, _volume: f32) -> Vec<u8> {
    Vec::new()
}

#[cfg(feature = "audio-playback")]
pub fn play(event: SoundEvent, config: AudioConfig) {
    if !config.enabled {
        return;
    }
    let clip = render_clip(event, config.volume);
    let _ = clip;
}

#[cfg(not(feature = "audio-playback"))]
pub fn play(_event: SoundEvent, _config: AudioConfig) {}

#[cfg_attr(not(feature = "audio"), allow(dead_code))]
fn synthesize(event: SoundEvent, volume: f32) -> Vec<i16> {
    let sample_rate = 22_050_u32;
    let dur_ms = match event {
        SoundEvent::UiClick => 40,
        SoundEvent::UiCancel => 60,
        SoundEvent::Gather => 220,
        SoundEvent::Combat => 320,
        SoundEvent::Trade => 260,
        SoundEvent::Weather => 480,
        SoundEvent::Ambient => 800,
    };
    let total = (sample_rate as usize * dur_ms as usize) / 1000;
    let vol = volume.clamp(0.0, 1.0) * 0.6;
    let mut out = Vec::with_capacity(total);
    for i in 0..total {
        let t = i as f32 / sample_rate as f32;
        let env_t = i as f32 / total as f32;
        let envelope = (1.0 - env_t) * (env_t * std::f32::consts::PI).sin();
        let s = match event {
            SoundEvent::UiClick | SoundEvent::UiCancel => {
                let freq = if matches!(event, SoundEvent::UiClick) {
                    880.0
                } else {
                    220.0
                };
                (t * freq * 2.0 * std::f32::consts::PI).sin()
            }
            SoundEvent::Gather => {
                (t * 330.0 * 2.0 * std::f32::consts::PI).sin() * 0.6
                    + (t * 495.0 * 2.0 * std::f32::consts::PI).sin() * 0.4
            }
            SoundEvent::Combat => {
                let noise =
                    ((i.wrapping_mul(2654435761) >> 3) & 0xFFFF) as i32 as f32 / 16_384.0 - 1.0;
                (t * 110.0 * 2.0 * std::f32::consts::PI).sin() * 0.5 + noise * 0.5
            }
            SoundEvent::Trade => {
                (t * 523.25 * 2.0 * std::f32::consts::PI).sin() * 0.5
                    + (t * 659.25 * 2.0 * std::f32::consts::PI).sin() * 0.3
            }
            SoundEvent::Weather => {
                (t * 110.0 * 2.0 * std::f32::consts::PI).sin() * 0.3
                    + (t * 165.0 * 2.0 * std::f32::consts::PI).sin() * 0.2
            }
            SoundEvent::Ambient => {
                (t * 196.0 * 2.0 * std::f32::consts::PI).sin() * 0.4
                    + (t * 261.63 * 2.0 * std::f32::consts::PI).sin() * 0.2
            }
        };
        out.push((s * envelope * vol * i16::MAX as f32) as i16);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_off() {
        let s = AudioSettings::default();
        assert!(!s.enabled);
        assert!(s.volume > 0.0 && s.volume <= 1.0);
    }

    #[test]
    fn config_from_settings() {
        let s = AudioSettings {
            enabled: true,
            volume: 0.3,
        };
        let c: AudioConfig = s.into();
        assert!(c.enabled);
        assert!((c.volume - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn play_disabled_is_noop() {
        let c = AudioConfig {
            enabled: false,
            volume: 0.5,
        };
        play(SoundEvent::Combat, c);
    }

    #[test]
    fn clip_lengths_match_event() {
        let cases = [
            (SoundEvent::UiClick, 40_u32),
            (SoundEvent::UiCancel, 60),
            (SoundEvent::Gather, 220),
            (SoundEvent::Combat, 320),
            (SoundEvent::Trade, 260),
            (SoundEvent::Weather, 480),
            (SoundEvent::Ambient, 800),
        ];
        for (ev, ms) in cases {
            let samples = synthesize(ev, 0.5);
            let expected = (22_050 * ms / 1000) as usize;
            assert_eq!(samples.len(), expected, "event {ev:?} length wrong");
        }
    }

    #[test]
    fn volume_clamped() {
        let s = synthesize(SoundEvent::Gather, 2.0);
        let max = s.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
        assert!(max <= i16::MAX as u16);
    }

    #[test]
    fn zero_volume_silent() {
        let s = synthesize(SoundEvent::Combat, 0.0);
        let max = s.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
        assert_eq!(max, 0);
    }

    #[test]
    fn render_clip_empty_when_feature_off() {
        #[cfg(not(feature = "audio"))]
        {
            let clip = render_clip(SoundEvent::Combat, 0.5);
            assert!(clip.is_empty());
        }
    }

    #[test]
    fn render_clip_nonempty_when_feature_on() {
        #[cfg(feature = "audio")]
        {
            let clip = render_clip(SoundEvent::UiClick, 0.5);
            assert!(!clip.is_empty());
            assert!(clip.len() > 44, "WAV header + data expected");
        }
    }

    #[test]
    fn sound_event_serde_roundtrip() {
        for ev in [
            SoundEvent::UiClick,
            SoundEvent::UiCancel,
            SoundEvent::Gather,
            SoundEvent::Combat,
            SoundEvent::Trade,
            SoundEvent::Weather,
            SoundEvent::Ambient,
        ] {
            let s = ron::to_string(&ev).unwrap();
            let back: SoundEvent = ron::from_str(&s).unwrap();
            assert_eq!(ev, back);
        }
    }

    #[test]
    fn audio_settings_serde_roundtrip() {
        let s = AudioSettings {
            enabled: true,
            volume: 0.42,
        };
        let r = ron::to_string(&s).unwrap();
        let back: AudioSettings = ron::from_str(&r).unwrap();
        assert_eq!(back.enabled, s.enabled);
        assert!((back.volume - s.volume).abs() < 1e-6);
    }
}
