use std::time::Instant;

/// Authentic Minecraft 1.8.9 Game Timer (MCP `Timer.java`).
#[derive(Debug, Clone)]
pub struct GameTimer {
    pub ticks_per_second: f32,
    pub last_time: Instant,
    pub elapsed_ticks: u32,
    pub render_partial_ticks: f32,
    pub timer_speed: f32,
    pub elapsed_partial_ticks: f32,
}

impl GameTimer {
    pub fn new(tps: f32) -> Self {
        Self {
            ticks_per_second: tps,
            last_time: Instant::now(),
            elapsed_ticks: 0,
            render_partial_ticks: 0.0,
            timer_speed: 1.0,
            elapsed_partial_ticks: 0.0,
        }
    }

    /// Resets the base timer clock.
    pub fn reset(&mut self) {
        self.last_time = Instant::now();
        self.elapsed_ticks = 0;
        self.render_partial_ticks = 0.0;
        self.elapsed_partial_ticks = 0.0;
    }

    /// Advances the timer and calculates elapsed physics ticks and render partial ticks (MCP `Timer.updateTimer`).
    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_time).as_secs_f64();
        self.last_time = now;

        // Clamp dt to [0.0, 1.0] matching MCP MathHelper.clamp_double(d2, 0.0D, 1.0D)
        let clamped_dt = dt.clamp(0.0, 1.0);
        self.elapsed_partial_ticks +=
            (clamped_dt * (self.timer_speed as f64) * (self.ticks_per_second as f64)) as f32;
        self.elapsed_ticks = self.elapsed_partial_ticks as u32;
        self.elapsed_partial_ticks -= self.elapsed_ticks as f32;

        // MCP Timer.java line 103: clamp elapsedTicks to 10
        if self.elapsed_ticks > 10 {
            self.elapsed_ticks = 10;
        }

        self.render_partial_ticks = self.elapsed_partial_ticks;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_game_timer_monotonic() {
        let mut timer = GameTimer::new(20.0);
        assert_eq!(timer.elapsed_ticks, 0);
        assert_eq!(timer.render_partial_ticks, 0.0);

        // Simulate 25ms elapsed (half a tick)
        timer.last_time -= Duration::from_millis(25);
        timer.update();
        assert_eq!(timer.elapsed_ticks, 0);
        assert!((timer.render_partial_ticks - 0.5).abs() < 0.05);

        // Simulate another 30ms elapsed (total 55ms -> 1 full tick and 5ms)
        timer.last_time -= Duration::from_millis(30);
        timer.update();
        assert_eq!(timer.elapsed_ticks, 1);
        assert!((timer.render_partial_ticks - 0.1).abs() < 0.05);
    }
}
