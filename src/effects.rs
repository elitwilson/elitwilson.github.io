use crate::particles::{Particle, ParticleSystem};
use crate::rng::Rng;
use ratatui::style::Color;
use std::time::Duration;

/// Supported effect types. Adding a new effect means adding an arm here
/// plus a private emit function — the `spawn` call site shape does not change.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    Fireworks,
}

/// Parameters that control a fireworks burst.
///
/// All fields have sensible defaults via `FireworksParams::default()`.
/// Callers override individual fields; the rest stay at defaults.
pub struct FireworksParams {
    /// Number of particles emitted per burst.
    pub count: usize,
    /// Color palette: one color is chosen at random per particle.
    pub palette: &'static [Color],
    /// Angular spread in radians. `2π` gives a full circular burst.
    pub spread: f32,
    /// Speed range `(min, max)` in cells per second.
    pub speed: (f32, f32),
    /// Gravity hint (cells/s²). Stored for future per-effect override; the
    /// physics system (STR-001) owns actual gravity for now.
    #[allow(dead_code)]
    pub gravity: f32,
    /// Lifetime range `(min, max)` in seconds.
    pub lifetime: (f32, f32),
}

/// A small bright palette reusing `ratatui::style::Color::Rgb`.
static DEFAULT_PALETTE: &[Color] = &[
    Color::Rgb(255, 80, 80),   // red-orange
    Color::Rgb(255, 200, 50),  // golden yellow
    Color::Rgb(80, 255, 80),   // lime green
    Color::Rgb(80, 200, 255),  // sky blue
    Color::Rgb(200, 80, 255),  // violet
    Color::Rgb(255, 255, 255), // white
];

impl Default for FireworksParams {
    fn default() -> Self {
        Self {
            count: 40,
            palette: DEFAULT_PALETTE,
            spread: std::f32::consts::TAU, // full 2π circle
            speed: (8.0, 20.0),
            gravity: 30.0,
            lifetime: (0.8, 2.0),
        }
    }
}

/// Dispatch an effect into the particle system.
///
/// The caller owns the `Rng`; threading it by `&mut` keeps state deterministic
/// and avoids global PRNG state.
pub fn spawn(
    kind: EffectKind,
    origin: (f32, f32),
    params: &FireworksParams,
    rng: &mut Rng,
    system: &mut ParticleSystem,
) {
    match kind {
        EffectKind::Fireworks => emit_fireworks(origin, params, rng, system),
    }
}

/// Emit a fireworks burst: `params.count` particles fanning outward from `origin`.
///
/// Each particle gets a random angle within `spread`, a random speed within
/// `params.speed`, a random lifetime within `params.lifetime`, and a random
/// color from `params.palette`. Glyph is a fixed spark character.
fn emit_fireworks(
    origin: (f32, f32),
    params: &FireworksParams,
    rng: &mut Rng,
    system: &mut ParticleSystem,
) {
    // Spread is centered symmetrically: angles run from -spread/2 to +spread/2.
    // Full TAU spread gives a complete circle; π spread gives a hemisphere.
    let half_spread = params.spread / 2.0;

    for _ in 0..params.count {
        let angle = rng.range_f32(-half_spread, half_spread);
        let speed = rng.range_f32(params.speed.0, params.speed.1);
        let lifetime_secs = rng.range_f32(params.lifetime.0, params.lifetime.1);
        let palette_idx = (rng.next_u32() as usize) % params.palette.len();

        let vx = angle.cos() * speed;
        let vy = angle.sin() * speed;
        let lifetime = Duration::from_secs_f32(lifetime_secs);

        system.spawn(Particle {
            pos: origin,
            vel: (vx, vy),
            remaining: lifetime,
            total: lifetime,
            color: params.palette[palette_idx],
            glyph: '*',
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::particles::ParticleSystem;
    use crate::rng::Rng;

    // --- burst count ---

    #[test]
    fn spawn_fireworks_injects_exactly_params_count_particles() {
        let mut sys = ParticleSystem::new();
        let params = FireworksParams::default(); // count == 40
        let mut rng = Rng::new(1);
        spawn(
            EffectKind::Fireworks,
            (10.0, 10.0),
            &params,
            &mut rng,
            &mut sys,
        );
        assert_eq!(
            sys.particles().len(),
            params.count,
            "burst should inject exactly params.count particles"
        );
    }

    #[test]
    fn spawn_fireworks_respects_custom_count() {
        let mut sys = ParticleSystem::new();
        let params = FireworksParams {
            count: 5,
            ..FireworksParams::default()
        };
        let mut rng = Rng::new(2);
        spawn(
            EffectKind::Fireworks,
            (5.0, 5.0),
            &params,
            &mut rng,
            &mut sys,
        );
        assert_eq!(sys.particles().len(), 5);
    }

    // --- velocity variation ---

    #[test]
    fn emitted_particles_have_varied_velocities() {
        let mut sys = ParticleSystem::new();
        let params = FireworksParams {
            count: 20,
            ..FireworksParams::default()
        };
        let mut rng = Rng::new(3);
        spawn(
            EffectKind::Fireworks,
            (0.0, 0.0),
            &params,
            &mut rng,
            &mut sys,
        );

        let particles = sys.particles();
        // At least two distinct velocity vectors must exist.
        let first = particles[0].vel;
        let all_same = particles
            .iter()
            .all(|p| (p.vel.0 - first.0).abs() < 1e-6 && (p.vel.1 - first.1).abs() < 1e-6);
        assert!(
            !all_same,
            "all particles had identical velocity — randomization is not applied"
        );
    }

    // --- seed reproducibility ---

    #[test]
    fn same_seed_same_origin_same_params_yields_identical_burst() {
        let make_burst = || {
            let mut sys = ParticleSystem::new();
            let params = FireworksParams::default();
            let mut rng = Rng::new(999);
            spawn(
                EffectKind::Fireworks,
                (8.0, 12.0),
                &params,
                &mut rng,
                &mut sys,
            );
            sys
        };

        let sys_a = make_burst();
        let sys_b = make_burst();

        let pa = sys_a.particles();
        let pb = sys_b.particles();
        assert_eq!(pa.len(), pb.len(), "burst sizes must match");

        for (i, (a, b)) in pa.iter().zip(pb.iter()).enumerate() {
            assert!(
                (a.vel.0 - b.vel.0).abs() < 1e-6 && (a.vel.1 - b.vel.1).abs() < 1e-6,
                "particle {i}: velocity mismatch between identical seeds"
            );
            let diff = a.remaining.as_secs_f32() - b.remaining.as_secs_f32();
            assert!(
                diff.abs() < 1e-6,
                "particle {i}: lifetime mismatch between identical seeds"
            );
        }
    }

    // --- origin placement ---

    #[test]
    fn emitted_particles_start_at_origin() {
        let mut sys = ParticleSystem::new();
        let params = FireworksParams {
            count: 10,
            ..FireworksParams::default()
        };
        let origin = (15.0_f32, 7.0_f32);
        let mut rng = Rng::new(55);
        spawn(EffectKind::Fireworks, origin, &params, &mut rng, &mut sys);

        for p in sys.particles() {
            assert!(
                (p.pos.0 - origin.0).abs() < 1e-6 && (p.pos.1 - origin.1).abs() < 1e-6,
                "particle does not start at origin: {:?}",
                p.pos
            );
        }
    }
}
