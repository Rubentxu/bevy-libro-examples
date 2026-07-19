// Capítulo 19. Animación — Timers, frame cycling, sprite sheets
use std::collections::HashMap;

/// Animation clip: sequence of frame indices with a duration per frame
#[derive(Clone, Debug)]
pub struct AnimationClip {
    pub name: String,
    pub frames: Vec<usize>,
    pub frame_duration: f32,
    pub looping: bool,
}

impl AnimationClip {
    pub fn new(name: &str, frames: Vec<usize>, frame_duration: f32, looping: bool) -> Self {
        Self {
            name: name.to_string(),
            frames,
            frame_duration,
            looping,
        }
    }

    pub fn total_duration(&self) -> f32 {
        self.frames.len() as f32 * self.frame_duration
    }
}

/// Animation player: cycles through frames of the current clip
#[derive(Clone, Debug)]
pub struct AnimationPlayer {
    pub current_clip: Option<String>,
    pub elapsed: f32,
    pub current_frame: usize,
    clips: HashMap<String, AnimationClip>,
}

impl AnimationPlayer {
    pub fn new() -> Self {
        Self {
            current_clip: None,
            elapsed: 0.0,
            current_frame: 0,
            clips: HashMap::new(),
        }
    }

    pub fn add_clip(&mut self, clip: AnimationClip) {
        self.clips.insert(clip.name.clone(), clip);
    }

    pub fn play(&mut self, name: &str) {
        if self.current_clip.as_deref() == Some(name) {
            return; // Already playing
        }
        self.current_clip = Some(name.to_string());
        self.elapsed = 0.0;
        self.current_frame = 0;
    }

    pub fn stop(&mut self) {
        self.current_clip = None;
        self.elapsed = 0.0;
        self.current_frame = 0;
    }

    /// Advance animation by dt seconds. Returns the current frame index.
    pub fn update(&mut self, dt: f32) -> Option<usize> {
        let clip_name = self.current_clip.clone()?;
        let clip = self.clips.get(&clip_name)?;

        self.elapsed += dt;

        let frame_count = clip.frames.len();
        if frame_count == 0 {
            return None;
        }

        let total_time = frame_count as f32 * clip.frame_duration;

        if self.elapsed >= total_time {
            if clip.looping {
                self.elapsed %= total_time;
            } else {
                self.current_frame = frame_count - 1;
                return Some(clip.frames[self.current_frame]);
            }
        }

        self.current_frame = (self.elapsed / clip.frame_duration) as usize;
        if self.current_frame >= frame_count {
            self.current_frame = frame_count - 1;
        }

        Some(clip.frames[self.current_frame])
    }

    pub fn is_playing(&self) -> bool {
        self.current_clip.is_some()
    }

    pub fn is_finished(&self) -> bool {
        if let Some(ref name) = self.current_clip {
            if let Some(clip) = self.clips.get(name) {
                return !clip.looping && self.elapsed >= clip.total_duration();
            }
        }
        true
    }
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Blend tree for smooth transitions between animations
#[derive(Clone, Debug)]
pub struct BlendTree {
    pub input_speed: f32,
    pub idle_clip: String,
    pub walk_clip: String,
    pub run_clip: String,
    pub walk_threshold: f32,
    pub run_threshold: f32,
}

impl BlendTree {
    pub fn decide_clip(&self) -> String {
        if self.input_speed < self.walk_threshold {
            self.idle_clip.clone()
        } else if self.input_speed < self.run_threshold {
            self.walk_clip.clone()
        } else {
            self.run_clip.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_clip_duration() {
        let clip = AnimationClip::new("walk", vec![0, 1, 2, 3], 0.1, true);
        assert!((clip.total_duration() - 0.4).abs() < 0.001);
    }

    #[test]
    fn player_starts_idle() {
        let player = AnimationPlayer::new();
        assert!(!player.is_playing());
        assert!(player.is_finished());
    }

    #[test]
    fn player_play_advances_frames() {
        let mut player = AnimationPlayer::new();
        player.add_clip(AnimationClip::new("walk", vec![0, 1, 2, 3], 0.1, true));
        player.play("walk");

        // Frame 0 at t=0
        let frame = player.update(0.0);
        assert_eq!(frame, Some(0));

        // Frame 1 at t=0.1
        let frame = player.update(0.1);
        assert_eq!(frame, Some(1));

        // Frame 3 at t=0.3
        let frame = player.update(0.1);
        assert_eq!(frame, Some(2));
        let frame = player.update(0.1);
        assert_eq!(frame, Some(3));
    }

    #[test]
    fn player_loops_back_to_frame_0() {
        let mut player = AnimationPlayer::new();
        player.add_clip(AnimationClip::new("walk", vec![0, 1, 2, 3], 0.1, true));
        player.play("walk");

        // Advance past total duration (0.4s)
        player.update(0.45);
        let frame = player.update(0.0);

        // Should have looped back
        assert_eq!(frame, Some(0));
    }

    #[test]
    fn player_non_looping_stops_at_last_frame() {
        let mut player = AnimationPlayer::new();
        player.add_clip(AnimationClip::new("attack", vec![0, 1, 2], 0.1, false));
        player.play("attack");

        // Advance past end
        player.update(0.5);
        let frame = player.update(0.0);

        assert_eq!(frame, Some(2), "Should stay on last frame");
        assert!(player.is_finished());
    }

    #[test]
    fn player_play_same_clip_no_restart() {
        let mut player = AnimationPlayer::new();
        player.add_clip(AnimationClip::new("walk", vec![0, 1, 2], 0.1, true));
        player.play("walk");
        player.update(0.15); // Now at frame 1

        let elapsed_before = player.elapsed;
        player.play("walk"); // Should NOT restart
        assert_eq!(player.elapsed, elapsed_before, "Should not restart");
    }

    #[test]
    fn player_stop_resets() {
        let mut player = AnimationPlayer::new();
        player.add_clip(AnimationClip::new("walk", vec![0, 1, 2], 0.1, true));
        player.play("walk");
        player.update(0.2);

        player.stop();
        assert!(!player.is_playing());
        assert_eq!(player.elapsed, 0.0);
    }

    #[test]
    fn blend_tree_idle() {
        let tree = BlendTree {
            input_speed: 0.5,
            idle_clip: "idle".to_string(),
            walk_clip: "walk".to_string(),
            run_clip: "run".to_string(),
            walk_threshold: 1.0,
            run_threshold: 5.0,
        };
        assert_eq!(tree.decide_clip(), "idle");
    }

    #[test]
    fn blend_tree_walk() {
        let tree = BlendTree {
            input_speed: 3.0,
            idle_clip: "idle".to_string(),
            walk_clip: "walk".to_string(),
            run_clip: "run".to_string(),
            walk_threshold: 1.0,
            run_threshold: 5.0,
        };
        assert_eq!(tree.decide_clip(), "walk");
    }

    #[test]
    fn blend_tree_run() {
        let tree = BlendTree {
            input_speed: 8.0,
            idle_clip: "idle".to_string(),
            walk_clip: "walk".to_string(),
            run_clip: "run".to_string(),
            walk_threshold: 1.0,
            run_threshold: 5.0,
        };
        assert_eq!(tree.decide_clip(), "run");
    }
}
