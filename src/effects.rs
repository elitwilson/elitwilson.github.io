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
mod tests;
