use crate::particles::ParticleSystem;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

/// Project a f32 cell-space position to an integer cell coordinate.
///
/// Uses half-away-from-zero rounding (`f32::round`) so `2.5 → 3` and `2.4 → 2`.
/// Returns `(i32, i32)` so negative coordinates survive the bounds check without
/// underflowing a `u16`.
pub fn project(pos: (f32, f32)) -> (i32, i32) {
    (pos.0.round() as i32, pos.1.round() as i32)
}

/// Scale a base `Color::Rgb` toward black by the fade factor.
///
/// `fade` is the raw `Particle::fade()` value: `0.0` = fresh (full brightness),
/// `1.0` = fully aged (black). Non-`Rgb` colors are returned unchanged.
///
/// # Assumed STR-001 surface
/// Particles carry `Color::Rgb` appearance; the non-`Rgb` arm is a safe fallback.
pub fn fade_color(base: Color, fade: f32) -> Color {
    // brightness = 1.0 when fresh (fade=0.0), approaches 0.0 as particle ages
    let brightness = (1.0 - fade).clamp(0.0, 1.0);
    match base {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * brightness) as u8,
            (g as f32 * brightness) as u8,
            (b as f32 * brightness) as u8,
        ),
        // Non-Rgb colors pass through unchanged — no panic.
        other => other,
    }
}

/// Draw all live particles from `particles` into `buf` within `area`.
///
/// Each particle's f32 position is rounded to the nearest cell, offset by
/// `origin` (the top-left cell anchor for particle coordinate (0,0)), bounds-
/// checked against both `area` and the buffer, then painted as a foreground
/// glyph with a faded color over whatever scene already occupies the cell.
///
/// # Assumed STR-001 surface
/// Depends on: `ParticleSystem::particles() -> &[Particle]`, `Particle::pos`,
/// `Particle::color`, `Particle::glyph`, `Particle::fade()`. If STR-001's
/// final surface differs (e.g. fields vs. accessors), adapt the call sites
/// in this function — the math is independent.
pub fn draw_particles(
    particles: &ParticleSystem,
    buf: &mut Buffer,
    area: Rect,
    origin: (u16, u16),
) {
    for p in particles.particles() {
        let (px, py) = project(p.pos);

        // Offset by caller-supplied origin to anchor particle (0,0) at `origin`.
        let cx = px + origin.0 as i32;
        let cy = py + origin.1 as i32;

        // Skip negative coordinates before any u16 cast (avoids underflow).
        if cx < 0 || cy < 0 {
            continue;
        }

        let cx = cx as u16;
        let cy = cy as u16;

        // Skip cells outside the target area.
        if cx < area.left() || cx >= area.right() || cy < area.top() || cy >= area.bottom() {
            continue;
        }

        // `cell_mut` returns None if the coordinate is off the buffer.
        if let Some(cell) = buf.cell_mut((cx, cy)) {
            let faded = fade_color(p.color, p.fade());
            cell.set_symbol(&p.glyph.to_string());
            cell.set_fg(faded);
        }
    }
}

#[cfg(test)]
mod tests;
