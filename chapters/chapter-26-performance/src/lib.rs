// Capítulo 26. Performance — Metrics, frame budgeting, profiling
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Frame timing data for performance analysis
#[derive(Clone, Debug, Default)]
pub struct FrameMetrics {
    pub frame_time_ms: f64,
    pub fps: f64,
    pub update_ms: f64,
    pub render_ms: f64,
    pub entity_count: usize,
    pub draw_call_count: usize,
}

impl FrameMetrics {
    pub fn from_frame_time(frame_time_ms: f64) -> Self {
        let fps = if frame_time_ms > 0.0 {
            1000.0 / frame_time_ms
        } else {
            0.0
        };
        Self {
            frame_time_ms,
            fps,
            ..Default::default()
        }
    }
}

/// Rolling average of frame times for stable FPS display
pub struct FrameTimeHistory {
    samples: VecDeque<f64>,
    max_samples: usize,
}

impl FrameTimeHistory {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn push(&mut self, frame_time_ms: f64) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(frame_time_ms);
    }

    pub fn average_ms(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.samples.iter().sum();
        sum / self.samples.len() as f64
    }

    pub fn average_fps(&self) -> f64 {
        let avg = self.average_ms();
        if avg > 0.0 {
            1000.0 / avg
        } else {
            0.0
        }
    }

    pub fn percentile(&self, pct: f64) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f64> = self.samples.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = (((pct / 100.0) * sorted.len() as f64).ceil() as usize).min(sorted.len()) - 1;
        sorted[idx]
    }

    /// Check if the game is missing its target frame rate
    pub fn is_frame_budget_exceeded(&self, target_fps: f64) -> bool {
        let target_ms = 1000.0 / target_fps;
        let p99 = self.percentile(99.0);
        p99 > target_ms * 1.5 // 50% over budget at 99th percentile
    }
}

/// Performance budget tracker for different subsystems
pub struct PerformanceBudget {
    pub target_frame_time_ms: f64,
    pub subsystems: Vec<SubsystemBudget>,
}

#[derive(Clone, Debug)]
pub struct SubsystemBudget {
    pub name: String,
    pub budget_ms: f64,
    pub actual_ms: f64,
}

impl PerformanceBudget {
    pub fn new(target_fps: f64) -> Self {
        Self {
            target_frame_time_ms: 1000.0 / target_fps,
            subsystems: Vec::new(),
        }
    }

    pub fn allocate(&mut self, name: &str, budget_ms: f64) {
        self.subsystems.push(SubsystemBudget {
            name: name.to_string(),
            budget_ms,
            actual_ms: 0.0,
        });
    }

    pub fn record(&mut self, name: &str, actual_ms: f64) {
        if let Some(sub) = self.subsystems.iter_mut().find(|s| s.name == name) {
            sub.actual_ms = actual_ms;
        }
    }

    pub fn total_used(&self) -> f64 {
        self.subsystems.iter().map(|s| s.actual_ms).sum()
    }

    pub fn is_over_budget(&self) -> bool {
        self.total_used() > self.target_frame_time_ms
    }

    /// Find subsystems that exceed their individual budget
    pub fn over_budget_subsystems(&self) -> Vec<&SubsystemBudget> {
        self.subsystems
            .iter()
            .filter(|s| s.actual_ms > s.budget_ms)
            .collect()
    }
}

/// Simple scope timer for profiling code sections
pub struct ScopeTimer {
    start: Instant,
    label: String,
}

impl ScopeTimer {
    pub fn start(label: &str) -> Self {
        Self {
            start: Instant::now(),
            label: label.to_string(),
        }
    }

    pub fn elapsed_ms(&self) -> f64 {
        let elapsed = self.start.elapsed();
        elapsed.as_secs_f64() * 1000.0
    }

    pub fn elapsed_micros(&self) -> u128 {
        self.start.elapsed().as_micros()
    }
}

/// Entity count profiler: tracks spawn/despawn rates
pub struct EntityProfiler {
    pub current_count: usize,
    pub peak_count: usize,
    pub total_spawned: u64,
    pub total_despawned: u64,
}

impl EntityProfiler {
    pub fn new() -> Self {
        Self {
            current_count: 0,
            peak_count: 0,
            total_spawned: 0,
            total_despawned: 0,
        }
    }

    pub fn on_spawn(&mut self, count: usize) {
        self.current_count += count;
        self.total_spawned += count as u64;
        self.peak_count = self.peak_count.max(self.current_count);
    }

    pub fn on_despawn(&mut self, count: usize) {
        self.current_count = self.current_count.saturating_sub(count);
        self.total_despawned += count as u64;
    }
}

impl Default for EntityProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_metrics_calculates_fps() {
        let m = FrameMetrics::from_frame_time(16.67);
        assert!((m.fps - 60.0).abs() < 0.5, "16.67ms should be ~60 FPS");
    }

    #[test]
    fn frame_metrics_zero_time() {
        let m = FrameMetrics::from_frame_time(0.0);
        assert_eq!(m.fps, 0.0);
    }

    #[test]
    fn frame_time_history_average() {
        let mut history = FrameTimeHistory::new(100);
        for _ in 0..10 {
            history.push(16.0);
        }
        assert!((history.average_ms() - 16.0).abs() < 0.001);
        assert!((history.average_fps() - 62.5).abs() < 0.5);
    }

    #[test]
    fn frame_time_history_rolling_window() {
        let mut history = FrameTimeHistory::new(5);
        for i in 0..10 {
            history.push(i as f64); // 0, 1, 2, ... 9
        }
        // Should only have last 5 samples: 5, 6, 7, 8, 9
        assert!((history.average_ms() - 7.0).abs() < 0.001);
    }

    #[test]
    fn frame_time_percentile() {
        let mut history = FrameTimeHistory::new(100);
        // Add frames: 10 at 16ms, 1 spike at 50ms
        for _ in 0..10 {
            history.push(16.0);
        }
        history.push(50.0);

        let median = history.percentile(50.0);
        assert!((median - 16.0).abs() < 0.5, "Median should be ~16ms");

        let p99 = history.percentile(99.0);
        assert!((p99 - 50.0).abs() < 0.5, "99th percentile should catch spike");
    }

    #[test]
    fn frame_budget_exceeded_on_spikes() {
        let mut history = FrameTimeHistory::new(100);
        for _ in 0..100 {
            history.push(16.0); // 60 FPS target
        }
        assert!(!history.is_frame_budget_exceeded(60.0));

        // Add spikes
        for _ in 0..5 {
            history.push(50.0); // Spike
        }
        assert!(history.is_frame_budget_exceeded(60.0));
    }

    #[test]
    fn performance_budget_tracks() {
        let mut budget = PerformanceBudget::new(60.0); // 16.67ms target
        budget.allocate("physics", 5.0);
        budget.allocate("render", 8.0);
        budget.allocate("ai", 3.0);

        budget.record("physics", 4.5);
        budget.record("render", 7.0);
        budget.record("ai", 2.5);

        assert!(!budget.is_over_budget(), "Total 14ms should be under 16.67ms");
    }

    #[test]
    fn performance_budget_over() {
        let mut budget = PerformanceBudget::new(60.0);
        budget.allocate("physics", 5.0);
        budget.allocate("render", 8.0);

        budget.record("physics", 10.0); // Over!
        budget.record("render", 8.0);

        assert!(budget.is_over_budget());
        let over = budget.over_budget_subsystems();
        assert_eq!(over.len(), 1);
        assert_eq!(over[0].name, "physics");
    }

    #[test]
    fn scope_timer_measures() {
        let timer = ScopeTimer::start("test");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 8.0 && elapsed <= 50.0, "Should measure ~10ms, got {}", elapsed);
    }

    #[test]
    fn entity_profiler_tracks_spawn() {
        let mut profiler = EntityProfiler::new();
        profiler.on_spawn(10);
        assert_eq!(profiler.current_count, 10);
        assert_eq!(profiler.peak_count, 10);
        assert_eq!(profiler.total_spawned, 10);
    }

    #[test]
    fn entity_profiler_tracks_despawn() {
        let mut profiler = EntityProfiler::new();
        profiler.on_spawn(100);
        profiler.on_despawn(30);
        assert_eq!(profiler.current_count, 70);
        assert_eq!(profiler.total_despawned, 30);
        assert_eq!(profiler.peak_count, 100, "Peak should not decrease");
    }

    #[test]
    fn entity_profiler_peak_tracks_correctly() {
        let mut profiler = EntityProfiler::new();
        profiler.on_spawn(50);
        profiler.on_despawn(20); // 30 left
        profiler.on_spawn(10);   // 40 left
        assert_eq!(profiler.peak_count, 50, "Peak should be 50 from first spawn");
    }
}
