// Capítulo 18. Audio — Volume mixing, pitch, spatial audio

/// Audio channel for managing multiple audio streams
#[derive(Clone, Debug)]
pub struct AudioChannel {
    pub name: String,
    pub volume: f32,
    pub pitch: f32,
    pub paused: bool,
}

impl AudioChannel {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            volume: 1.0,
            pitch: 1.0,
            paused: false,
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(0.1, 4.0);
    }
}

/// Audio mixer: manages multiple channels and computes final output
pub struct AudioMixer {
    pub master: f32,
    pub channels: std::collections::HashMap<String, AudioChannel>,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            master: 1.0,
            channels: std::collections::HashMap::new(),
        }
    }

    pub fn add_channel(&mut self, name: &str) {
        self.channels.insert(name.to_string(), AudioChannel::new(name));
    }

    pub fn get_channel(&self, name: &str) -> Option<&AudioChannel> {
        self.channels.get(name)
    }

    pub fn get_channel_mut(&mut self, name: &str) -> Option<&mut AudioChannel> {
        self.channels.get_mut(name)
    }

    /// Compute effective volume for a channel: master * channel_volume
    pub fn effective_volume(&self, channel_name: &str) -> f32 {
        match self.channels.get(channel_name) {
            Some(ch) if !ch.paused => self.master * ch.volume,
            _ => 0.0,
        }
    }

    pub fn set_master(&mut self, volume: f32) {
        self.master = volume.clamp(0.0, 1.0);
    }
}

impl Default for AudioMixer {
    fn default() -> Self { Self::new() }
}

/// Spatial audio: attenuate volume based on distance
pub fn spatial_volume(
    listener_pos: (f32, f32),
    source_pos: (f32, f32),
    max_distance: f32,
    base_volume: f32,
) -> f32 {
    let dx = source_pos.0 - listener_pos.0;
    let dy = source_pos.1 - listener_pos.1;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist >= max_distance {
        return 0.0;
    }

    // Linear attenuation
    let attenuation = 1.0 - dist / max_distance;
    base_volume * attenuation
}

/// Stereo panning: compute left/right channel volumes
pub fn stereo_pan(pan: f32) -> (f32, f32) {
    // pan: -1.0 = full left, 0.0 = center, 1.0 = full right
    let pan = pan.clamp(-1.0, 1.0);
    let left = (1.0 - pan) * 0.5;
    let right = (1.0 + pan) * 0.5;
    (left, right)
}

/// Random pitch variation for repetitive sounds
pub fn random_pitch(base: f32, variation: f32, rng_value: f32) -> f32 {
    // rng_value: 0.0 to 1.0
    let offset = (rng_value - 0.5) * 2.0 * variation;
    (base + offset).max(0.1)
}

/// Crossfade between two audio tracks
pub fn crossfade(progress: f32) -> (f32, f32) {
    let p = progress.clamp(0.0, 1.0);
    // Smoothstep curve for natural crossfade
    let smooth = p * p * (3.0 - 2.0 * p);
    (1.0 - smooth, smooth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_channel_default() {
        let ch = AudioChannel::new("music");
        assert_eq!(ch.volume, 1.0);
        assert!(!ch.paused);
    }

    #[test]
    fn audio_channel_volume_clamped() {
        let mut ch = AudioChannel::new("sfx");
        ch.set_volume(2.0);
        assert_eq!(ch.volume, 1.0);

        ch.set_volume(-0.5);
        assert_eq!(ch.volume, 0.0);
    }

    #[test]
    fn mixer_effective_volume() {
        let mut mixer = AudioMixer::new();
        mixer.set_master(0.8);
        mixer.add_channel("music");
        mixer.get_channel_mut("music").unwrap().set_volume(0.5);

        assert!((mixer.effective_volume("music") - 0.4).abs() < 0.001);
    }

    #[test]
    fn mixer_paused_channel() {
        let mut mixer = AudioMixer::new();
        mixer.add_channel("music");
        mixer.get_channel_mut("music").unwrap().paused = true;

        assert_eq!(mixer.effective_volume("music"), 0.0);
    }

    #[test]
    fn spatial_volume_at_listener() {
        let vol = spatial_volume((0.0, 0.0), (0.0, 0.0), 100.0, 1.0);
        assert!((vol - 1.0).abs() < 0.001);
    }

    #[test]
    fn spatial_volume_at_max_distance() {
        let vol = spatial_volume((0.0, 0.0), (100.0, 0.0), 100.0, 1.0);
        assert!(vol < 0.01);
    }

    #[test]
    fn spatial_volume_beyond_range() {
        let vol = spatial_volume((0.0, 0.0), (200.0, 0.0), 100.0, 1.0);
        assert_eq!(vol, 0.0);
    }

    #[test]
    fn stereo_pan_center() {
        let (left, right) = stereo_pan(0.0);
        assert!((left - right).abs() < 0.001);
    }

    #[test]
    fn stereo_pan_full_left() {
        let (left, right) = stereo_pan(-1.0);
        assert!(left > right, "Left should be louder when panned left");
    }

    #[test]
    fn stereo_pan_full_right() {
        let (left, right) = stereo_pan(1.0);
        assert!(right > left, "Right should be louder when panned right");
    }

    #[test]
    fn random_pitch_within_bounds() {
        for i in 0..100 {
            let rng = i as f32 / 100.0;
            let pitch = random_pitch(1.0, 0.1, rng);
            assert!(pitch >= 0.9 && pitch <= 1.1, "Pitch {} out of bounds", pitch);
        }
    }

    #[test]
    fn crossfade_start() {
        let (a, b) = crossfade(0.0);
        assert!((a - 1.0).abs() < 0.001);
        assert!(b < 0.001);
    }

    #[test]
    fn crossfade_end() {
        let (a, b) = crossfade(1.0);
        assert!(a < 0.001);
        assert!((b - 1.0).abs() < 0.001);
    }

    #[test]
    fn crossfade_midpoint() {
        let (a, b) = crossfade(0.5);
        assert!((a - b).abs() < 0.01, "At midpoint both should be ~equal");
    }
}
