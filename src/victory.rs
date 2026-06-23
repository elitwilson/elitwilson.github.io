use crate::effects::{EffectKind, FireworksParams, spawn};
use crate::particles::ParticleSystem;
use crate::rng::Rng;
use ratatui::layout::Rect;
use std::time::Duration;

/// Particles-per-burst range. Each burst rolls a random count in
/// `[COUNT_MIN, COUNT_MAX]` so bursts vary in density — some fat, some wispy.
const COUNT_MIN: usize = 45;
const COUNT_MAX: usize = 95;

/// Delay range between victory volleys. The next volley is scheduled after a
/// random delay drawn from `[INTERVAL_MIN, INTERVAL_MAX)` — a *range*, not a fixed
/// cadence, so the celebration feels organic rather than metronomic. Kept short
/// for a rapid grand-finale tempo.
const INTERVAL_MIN: Duration = Duration::from_millis(90);
const INTERVAL_MAX: Duration = Duration::from_millis(400);

/// Upper bound on bursts fired in a single volley. Each volley fires a random
/// `1..=MAX_SIMULTANEOUS_BURSTS`, so the show sometimes pops two or three at once.
const MAX_SIMULTANEOUS_BURSTS: u32 = 3;

/// Drives the celebratory fireworks shown behind the victory modal.
///
/// Owns its own particle system and PRNG. Each `tick` advances the simulation and,
/// when the randomized countdown elapses, spawns a burst at a random screen
/// position and reschedules the next one within the interval range.
pub struct VictoryFireworks {
    system: ParticleSystem,
    rng: Rng,
    params: FireworksParams,
    interval: (Duration, Duration),
    until_next: Duration,
}

impl VictoryFireworks {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let interval = (INTERVAL_MIN, INTERVAL_MAX);
        let until_next = random_interval(&mut rng, interval);
        Self {
            system: ParticleSystem::new(),
            rng,
            // `count` is rolled per burst in `tick`; the initial value is unused.
            params: FireworksParams::default(),
            interval,
            until_next,
        }
    }

    /// Advance the show by `dt`. Whenever the randomized countdown elapses, fire a
    /// volley of `1..=MAX_SIMULTANEOUS_BURSTS` bursts, each at its own random
    /// position inside `area` (body-relative cell-space), then reschedule.
    pub fn tick(&mut self, dt: Duration, area: Rect) {
        if self.until_next <= dt {
            let volley = random_burst_count(&mut self.rng);
            for _ in 0..volley {
                self.params.count = random_count(&mut self.rng);
                let origin = random_origin(&mut self.rng, area);
                spawn(
                    EffectKind::Fireworks,
                    origin,
                    &self.params,
                    &mut self.rng,
                    &mut self.system,
                );
            }
            self.until_next = random_interval(&mut self.rng, self.interval);
        } else {
            self.until_next -= dt;
        }
        self.system.tick(dt);
    }

    /// Read-only view of live particles for the renderer.
    pub fn particles(&self) -> &ParticleSystem {
        &self.system
    }
}

/// Pick a random delay within `[min, max)`.
fn random_interval(rng: &mut Rng, (min, max): (Duration, Duration)) -> Duration {
    Duration::from_secs_f32(rng.range_f32(min.as_secs_f32(), max.as_secs_f32()))
}

/// Pick how many bursts to fire this volley — `1..=MAX_SIMULTANEOUS_BURSTS`.
fn random_burst_count(rng: &mut Rng) -> u32 {
    1 + rng.next_u32() % MAX_SIMULTANEOUS_BURSTS
}

/// Pick a random particle count for a single burst — `[COUNT_MIN, COUNT_MAX]`.
fn random_count(rng: &mut Rng) -> usize {
    COUNT_MIN + (rng.next_u32() as usize) % (COUNT_MAX - COUNT_MIN + 1)
}

/// Pick a random burst origin inside `area`, in body-relative cell-space.
fn random_origin(rng: &mut Rng, area: Rect) -> (f32, f32) {
    (
        rng.range_f32(0.0, area.width as f32),
        rng.range_f32(0.0, area.height as f32),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect::new(0, 0, 80, 24)
    }

    // --- interval range ---

    #[test]
    fn random_interval_stays_within_range() {
        let mut rng = Rng::new(7);
        let range = (INTERVAL_MIN, INTERVAL_MAX);
        for _ in 0..1000 {
            let d = random_interval(&mut rng, range);
            assert!(d >= range.0 && d < range.1, "interval out of range: {d:?}");
        }
    }

    #[test]
    fn random_burst_count_stays_within_bounds() {
        let mut rng = Rng::new(13);
        let mut saw_multi = false;
        for _ in 0..1000 {
            let n = random_burst_count(&mut rng);
            assert!(
                (1..=MAX_SIMULTANEOUS_BURSTS).contains(&n),
                "burst count out of bounds: {n}"
            );
            if n > 1 {
                saw_multi = true;
            }
        }
        // The whole point of the finale: volleys sometimes fire more than one.
        assert!(saw_multi, "never produced a multi-burst volley");
    }

    #[test]
    fn random_count_stays_within_bounds() {
        let mut rng = Rng::new(17);
        let mut saw_min = false;
        let mut saw_high = false;
        for _ in 0..2000 {
            let n = random_count(&mut rng);
            assert!(
                (COUNT_MIN..=COUNT_MAX).contains(&n),
                "count out of bounds: {n}"
            );
            if n == COUNT_MIN {
                saw_min = true;
            }
            if n > COUNT_MIN {
                saw_high = true;
            }
        }
        // Sanity: the count actually varies across the range, not pinned to one value.
        assert!(saw_min && saw_high, "count did not vary across its range");
    }

    // --- random placement ---

    #[test]
    fn random_origin_stays_within_area() {
        let mut rng = Rng::new(11);
        let a = Rect::new(0, 0, 80, 24);
        for _ in 0..1000 {
            let (x, y) = random_origin(&mut rng, a);
            assert!(x >= 0.0 && x < 80.0, "x out of range: {x}");
            assert!(y >= 0.0 && y < 24.0, "y out of range: {y}");
        }
    }

    // --- scheduling ---

    #[test]
    fn no_burst_before_interval_elapses() {
        let mut fw = VictoryFireworks::new(1);
        // until_next is at least 250ms, so a single 10ms tick can't trigger a burst.
        fw.tick(Duration::from_millis(10), area());
        assert_eq!(fw.particles().particles().len(), 0);
    }

    #[test]
    fn burst_spawns_once_interval_elapses() {
        let mut fw = VictoryFireworks::new(1);
        // Max interval is < 900ms, so a single 1s tick guarantees a burst.
        fw.tick(Duration::from_millis(1000), area());
        assert!(fw.particles().particles().len() > 0);
    }

    #[test]
    fn spawns_repeatedly_over_time() {
        // Drive ~4s of 16ms frames and count bursts (a burst is a jump up in the
        // live count). The interval range should yield several over that span.
        let mut fw = VictoryFireworks::new(3);
        let a = area();
        let dt = Duration::from_millis(16);
        let mut bursts = 0;
        for _ in 0..250 {
            let before = fw.particles().particles().len();
            fw.tick(dt, a);
            if fw.particles().particles().len() > before {
                bursts += 1;
            }
        }
        assert!(
            bursts >= 2,
            "expected multiple bursts over time, got {bursts}"
        );
    }
}
