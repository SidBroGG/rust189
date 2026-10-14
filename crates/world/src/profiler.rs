use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

pub const FRAME_SAMPLE_CAPACITY: usize = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrameStage {
    Input,
    Network,
    Gui,
    Render,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FrameStageTimings {
    pub input_ms: f64,
    pub network_ms: f64,
    pub gui_ms: f64,
    pub render_ms: f64,
}

impl FrameStageTimings {
    pub fn total_ms(&self) -> f64 {
        self.input_ms + self.network_ms + self.gui_ms + self.render_ms
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSnapshot {
    pub timestamp_ms: u64,
    pub current_fps: f64,
    pub avg_fps: f64,
    pub avg_frametime_ms: f64,
    pub low_1_percent_fps: f64,
    pub low_0_1_percent_fps: f64,
    pub low_0_01_percent_fps: f64,
    pub rtt_ms: f64,
    pub jitter_ms: f64,
    pub bytes_in_per_sec: f64,
    pub bytes_out_per_sec: f64,
    pub total_bytes_received: u64,
    pub total_bytes_sent: u64,
    pub packets_received: u64,
    pub packets_sent: u64,
    pub tps: f64,
    pub loaded_chunks: usize,
    pub stages: FrameStageTimings,
    pub recent_frametimes: Vec<f64>,
}

/// Comprehensive performance profiler and benchmark logger.
pub struct PerformanceProfiler {
    frame_samples: VecDeque<f64>, // Frame durations in seconds
    last_frame_instant: Option<Instant>,
    last_dump_instant: Instant,

    // Stage timings
    pub current_stages: FrameStageTimings,
    pub smoothed_stages: FrameStageTimings,

    // Network metrics
    pub rtt_ms: f64,
    pub jitter_ms: f64,
    last_rtt_ms: f64,
    pub total_bytes_received: u64,
    pub total_bytes_sent: u64,
    pub packets_received: u64,
    pub packets_sent: u64,

    last_net_calc_instant: Instant,
    last_bytes_received: u64,
    last_bytes_sent: u64,
    bytes_in_per_sec: f64,
    bytes_out_per_sec: f64,

    // Tick metrics
    pub tps: f64,
    pub loaded_chunks: usize,
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            frame_samples: VecDeque::with_capacity(FRAME_SAMPLE_CAPACITY),
            last_frame_instant: None,
            last_dump_instant: now,

            current_stages: FrameStageTimings::default(),
            smoothed_stages: FrameStageTimings::default(),

            rtt_ms: 0.0,
            jitter_ms: 0.0,
            last_rtt_ms: 0.0,
            total_bytes_received: 0,
            total_bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,

            last_net_calc_instant: now,
            last_bytes_received: 0,
            last_bytes_sent: 0,
            bytes_in_per_sec: 0.0,
            bytes_out_per_sec: 0.0,

            tps: 20.0,
            loaded_chunks: 0,
        }
    }

    /// Records execution duration for a specific frame pipeline stage.
    pub fn record_stage(&mut self, stage: FrameStage, duration: Duration) {
        self.record_stage_ms(stage, duration.as_secs_f64() * 1000.0);
    }

    /// Records execution duration in milliseconds for a frame stage with exponential smoothing.
    pub fn record_stage_ms(&mut self, stage: FrameStage, ms: f64) {
        let alpha = 0.15;
        match stage {
            FrameStage::Input => {
                self.current_stages.input_ms = ms;
                self.smoothed_stages.input_ms = self.smoothed_stages.input_ms * (1.0 - alpha) + ms * alpha;
            }
            FrameStage::Network => {
                self.current_stages.network_ms = ms;
                self.smoothed_stages.network_ms = self.smoothed_stages.network_ms * (1.0 - alpha) + ms * alpha;
            }
            FrameStage::Gui => {
                self.current_stages.gui_ms = ms;
                self.smoothed_stages.gui_ms = self.smoothed_stages.gui_ms * (1.0 - alpha) + ms * alpha;
            }
            FrameStage::Render => {
                self.current_stages.render_ms = ms;
                self.smoothed_stages.render_ms = self.smoothed_stages.render_ms * (1.0 - alpha) + ms * alpha;
            }
        }
    }

    /// Returns the recent frame durations in milliseconds (up to `count`).
    pub fn recent_frametimes(&self, count: usize) -> Vec<f64> {
        self.frame_samples
            .iter()
            .rev()
            .take(count)
            .rev()
            .map(|dt| *dt * 1000.0)
            .collect()
    }

    /// Records a new frame start/end time.
    pub fn on_frame(&mut self) {
        let now = Instant::now();
        if let Some(prev) = self.last_frame_instant {
            let dt = (now - prev).as_secs_f64();
            if dt > 0.0 {
                if self.frame_samples.len() >= FRAME_SAMPLE_CAPACITY {
                    self.frame_samples.pop_front();
                }
                self.frame_samples.push_back(dt);
            }
        }
        self.last_frame_instant = Some(now);

        // Update network throughput calculation periodically
        let net_elapsed = self.last_net_calc_instant.elapsed().as_secs_f64();
        if net_elapsed >= 0.5 {
            let bytes_in_diff = self.total_bytes_received.saturating_sub(self.last_bytes_received);
            let bytes_out_diff = self.total_bytes_sent.saturating_sub(self.last_bytes_sent);
            self.bytes_in_per_sec = (bytes_in_diff as f64) / net_elapsed;
            self.bytes_out_per_sec = (bytes_out_diff as f64) / net_elapsed;

            self.last_bytes_received = self.total_bytes_received;
            self.last_bytes_sent = self.total_bytes_sent;
            self.last_net_calc_instant = now;
        }
    }

    /// Records network latency measurement (from KeepAlive round trip).
    pub fn on_keep_alive_latency(&mut self, latency: Duration) {
        let current_rtt = latency.as_secs_f64() * 1000.0;
        let diff = (current_rtt - self.last_rtt_ms).abs();
        // Exponential moving average for jitter
        self.jitter_ms = self.jitter_ms * 0.8 + diff * 0.2;
        self.last_rtt_ms = current_rtt;
        self.rtt_ms = current_rtt;
    }

    /// Records incoming network packet stats.
    pub fn on_bytes_received(&mut self, byte_count: usize) {
        self.total_bytes_received += byte_count as u64;
        self.packets_received += 1;
    }

    /// Records outgoing network packet stats.
    pub fn on_bytes_sent(&mut self, byte_count: usize) {
        self.total_bytes_sent += byte_count as u64;
        self.packets_sent += 1;
    }

    /// Calculates current snapshot metrics.
    pub fn calculate_snapshot(&self) -> BenchmarkSnapshot {
        let sample_count = self.frame_samples.len();
        if sample_count == 0 {
            return BenchmarkSnapshot {
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                current_fps: 0.0,
                avg_fps: 0.0,
                avg_frametime_ms: 0.0,
                low_1_percent_fps: 0.0,
                low_0_1_percent_fps: 0.0,
                low_0_01_percent_fps: 0.0,
                rtt_ms: self.rtt_ms,
                jitter_ms: self.jitter_ms,
                bytes_in_per_sec: self.bytes_in_per_sec,
                bytes_out_per_sec: self.bytes_out_per_sec,
                total_bytes_received: self.total_bytes_received,
                total_bytes_sent: self.total_bytes_sent,
                packets_received: self.packets_received,
                packets_sent: self.packets_sent,
                tps: self.tps,
                loaded_chunks: self.loaded_chunks,
                stages: self.smoothed_stages,
                recent_frametimes: Vec::new(),
            };
        }

        let current_dt = *self.frame_samples.back().unwrap();
        let current_fps = if current_dt > 0.0 { 1.0 / current_dt } else { 0.0 };

        let sum_dt: f64 = self.frame_samples.iter().sum();
        let avg_dt = sum_dt / (sample_count as f64);
        let avg_fps = if avg_dt > 0.0 { 1.0 / avg_dt } else { 0.0 };
        let avg_frametime_ms = avg_dt * 1000.0;

        // Copy and sort frame times to calculate percentiles (worst frametimes = highest dt)
        let mut sorted: Vec<f64> = self.frame_samples.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // 1% Low is the 99th percentile frametime (1% slowest frames)
        let idx_1_pct = ((sample_count as f64) * 0.99).floor() as usize;
        let dt_1_pct = sorted[idx_1_pct.min(sample_count - 1)];
        let low_1_pct = if dt_1_pct > 0.0 { 1.0 / dt_1_pct } else { 0.0 };

        // 0.1% Low
        let idx_0_1_pct = ((sample_count as f64) * 0.999).floor() as usize;
        let dt_0_1_pct = sorted[idx_0_1_pct.min(sample_count - 1)];
        let low_0_1_pct = if dt_0_1_pct > 0.0 { 1.0 / dt_0_1_pct } else { 0.0 };

        // 0.01% Low
        let idx_0_01_pct = ((sample_count as f64) * 0.9999).floor() as usize;
        let dt_0_01_pct = sorted[idx_0_01_pct.min(sample_count - 1)];
        let low_0_01_pct = if dt_0_01_pct > 0.0 { 1.0 / dt_0_01_pct } else { 0.0 };

        let recent_frametimes = self.recent_frametimes(120);

        BenchmarkSnapshot {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            current_fps,
            avg_fps,
            avg_frametime_ms,
            low_1_percent_fps: low_1_pct,
            low_0_1_percent_fps: low_0_1_pct,
            low_0_01_percent_fps: low_0_01_pct,
            rtt_ms: self.rtt_ms,
            jitter_ms: self.jitter_ms,
            bytes_in_per_sec: self.bytes_in_per_sec,
            bytes_out_per_sec: self.bytes_out_per_sec,
            total_bytes_received: self.total_bytes_received,
            total_bytes_sent: self.total_bytes_sent,
            packets_received: self.packets_received,
            packets_sent: self.packets_sent,
            tps: self.tps,
            loaded_chunks: self.loaded_chunks,
            stages: self.smoothed_stages,
            recent_frametimes,
        }
    }

    /// Directly dumps the current benchmark snapshot to the target JSON file.
    pub fn dump_telemetry<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let snapshot = self.calculate_snapshot();
        if let Some(parent) = path.as_ref().parent() {
            create_dir_all(parent)?;
        }
        let mut file = File::create(path)?;
        let json_str = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        file.write_all(json_str.as_bytes())?;
        Ok(())
    }

    /// Throttled periodic dump to `logs/profile_benchmark.json` asynchronously off the main thread.
    pub fn maybe_dump_telemetry<P: AsRef<Path>>(&mut self, path: P) {
        if self.last_dump_instant.elapsed() >= Duration::from_secs(3) {
            self.last_dump_instant = Instant::now();
            let snapshot = self.calculate_snapshot();
            let target_path = path.as_ref().to_path_buf();
            std::thread::spawn(move || {
                if let Some(parent) = target_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Ok(mut file) = File::create(&target_path) {
                    if let Ok(json_str) = serde_json::to_string_pretty(&snapshot) {
                        let _ = file.write_all(json_str.as_bytes());
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_quantiles() {
        let mut profiler = PerformanceProfiler::new();
        // Simulate 100 frames at ~16.6ms (60 FPS) and 5 stutter frames at 50ms (20 FPS)
        for _ in 0..95 {
            profiler.frame_samples.push_back(0.0166);
        }
        for _ in 0..5 {
            profiler.frame_samples.push_back(0.050);
        }

        let snap = profiler.calculate_snapshot();
        assert!(snap.avg_fps > 50.0);
        assert!(snap.low_1_percent_fps < 30.0); // detects the stutters
    }

    #[test]
    fn test_stage_timings_and_recent_frametimes() {
        let mut profiler = PerformanceProfiler::new();
        profiler.record_stage(FrameStage::Network, Duration::from_micros(1200));
        profiler.record_stage(FrameStage::Gui, Duration::from_micros(2500));
        profiler.record_stage(FrameStage::Render, Duration::from_micros(4800));

        assert!(profiler.current_stages.network_ms > 1.0);
        assert!(profiler.current_stages.gui_ms > 2.0);
        assert!(profiler.current_stages.render_ms > 4.0);
        assert!(profiler.current_stages.total_ms() > 7.0);

        for i in 1..=10 {
            profiler.frame_samples.push_back(i as f64 * 0.001);
        }

        let recent = profiler.recent_frametimes(5);
        assert_eq!(recent.len(), 5);
        // The last 5 should be 6.0, 7.0, 8.0, 9.0, 10.0 ms
        assert!((recent[4] - 10.0).abs() < 1e-4);
        assert!((recent[0] - 6.0).abs() < 1e-4);

        let snap = profiler.calculate_snapshot();
        assert_eq!(snap.recent_frametimes.len(), 10);
        assert!(snap.stages.network_ms > 0.0);
    }
}
