use std::time::{Duration, Instant};
use anyhow::Result;

const TICK_RATE: u64 = 20; // 20 ticks per second (50ms per tick)
const TARGET_TICK_DURATION: Duration = Duration::from_millis(50);

pub struct GameLoop {
    tick_count: u64,
    last_tick: Instant,
}

impl GameLoop {
    pub fn new() -> Self {
        Self {
            tick_count: 0,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);

        if elapsed >= TARGET_TICK_DURATION {
            self.tick_count += 1;
            self.last_tick = now;

            // Perform tick updates
            self.update_players();
            self.update_entities();
            self.update_physics();

            tracing::debug!("Tick {}", self.tick_count);
        }

        Ok(())
    }

    fn update_players(&mut self) {
        // Update player positions, health, etc.
    }

    fn update_entities(&mut self) {
        // Update mobs, projectiles, etc.
    }

    fn update_physics(&mut self) {
        // Apply gravity, collisions, etc.
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }
}
