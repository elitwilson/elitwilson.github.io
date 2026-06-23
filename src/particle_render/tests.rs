use super::*;
use crate::particles::{Particle, ParticleSystem};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::time::Duration;

// --- helpers ---

fn make_particle(pos: (f32, f32), color: Color, glyph: char, fade_progress: f32) -> Particle {
    // fade_progress: 0.0 = fresh (remaining == total), 1.0 = fully aged (remaining == 0)
    // We model it as: remaining = total * (1.0 - fade_progress)
    let total = Duration::from_secs_f32(2.0);
    let remaining_frac = (1.0 - fade_progress).clamp(0.0, 1.0);
    let remaining = Duration::from_secs_f32(total.as_secs_f32() * remaining_frac);
    Particle {
        pos,
        vel: (0.0, 0.0),
        remaining,
        total,
        color,
        glyph,
    }
}

fn system_with(particles: Vec<Particle>) -> ParticleSystem {
    let mut sys = ParticleSystem::new();
    for p in particles {
        sys.spawn(p);
    }
    sys
}

// ============================================================
// Task 1: Projection math
// ============================================================

#[test]
fn project_rounds_positive_coords_down() {
    assert_eq!(project((2.4, 3.4)), (2, 3));
}

#[test]
fn project_rounds_positive_coords_up_at_midpoint() {
    // f32::round is half-away-from-zero: 2.5 → 3
    assert_eq!(project((2.5, 3.5)), (3, 4));
}

#[test]
fn project_rounds_zero_coords() {
    assert_eq!(project((0.0, 0.0)), (0, 0));
}

#[test]
fn project_handles_negative_coords() {
    // Negative coordinates must survive as i32 (no underflow)
    assert_eq!(project((-1.4, -2.6)), (-1, -3));
}

#[test]
fn project_handles_negative_midpoint() {
    // -2.5 rounds away from zero → -3
    assert_eq!(project((-2.5, -0.5)), (-3, -1));
}

#[test]
fn project_large_coords() {
    assert_eq!(project((100.7, 200.3)), (101, 200));
}

// ============================================================
// Task 2: Fade-to-color mapping
// ============================================================

#[test]
fn fade_color_at_fresh_returns_base_color() {
    // fade=0.0 means fresh (full brightness), so the result should equal the base
    let base = Color::Rgb(200, 100, 50);
    let result = fade_color(base, 0.0);
    assert_eq!(result, base);
}

#[test]
fn fade_color_at_fully_aged_approaches_black() {
    let base = Color::Rgb(200, 100, 50);
    let result = fade_color(base, 1.0);
    assert_eq!(result, Color::Rgb(0, 0, 0));
}

#[test]
fn fade_color_at_half_dims_each_channel() {
    // fade=0.5 → brightness=0.5; each channel should be halved
    let base = Color::Rgb(200, 100, 50);
    let result = fade_color(base, 0.5);
    // 200 * 0.5 = 100, 100 * 0.5 = 50, 50 * 0.5 = 25
    assert_eq!(result, Color::Rgb(100, 50, 25));
}

#[test]
fn fade_color_non_rgb_passthrough() {
    // Named colors should be returned unchanged — no panic
    let base = Color::White;
    let result = fade_color(base, 0.5);
    assert_eq!(result, Color::White);
}

#[test]
fn fade_color_aged_is_strictly_dimmer_than_fresh() {
    let base = Color::Rgb(200, 100, 50);
    let fresh = fade_color(base, 0.0);
    let aged = fade_color(base, 0.7);
    // At least one channel of aged should be less than fresh
    match (fresh, aged) {
        (Color::Rgb(fr, fg, fb), Color::Rgb(ar, ag, ab)) => {
            assert!(
                ar < fr || ag < fg || ab < fb,
                "aged color should be strictly dimmer: fresh={fresh:?} aged={aged:?}"
            );
        }
        _ => panic!("expected Rgb colors"),
    }
}

// ============================================================
// Task 3: draw_particles — paint, bounds, overwrite
// ============================================================

#[test]
fn draw_particles_paints_glyph_at_projected_cell() {
    // Buffer 10x10, area=full buffer, origin=(0,0)
    // Particle at pos (3.0, 2.0) should appear at cell (3, 2)
    let area = Rect::new(0, 0, 10, 10);
    let mut buf = Buffer::empty(area);
    let sys = system_with(vec![make_particle(
        (3.0, 2.0),
        Color::Rgb(255, 0, 0),
        '*',
        0.0, // fresh
    )]);
    draw_particles(&sys, &mut buf, area, (0, 0));

    let cell = buf.cell((3, 2)).expect("cell should exist");
    assert_eq!(cell.symbol(), "*");
    // Fresh particle (fade=0.0) should keep the base color
    assert_eq!(cell.fg, Color::Rgb(255, 0, 0));
}

#[test]
fn draw_particles_paints_with_faded_color() {
    let area = Rect::new(0, 0, 10, 10);
    let mut buf = Buffer::empty(area);
    let sys = system_with(vec![make_particle(
        (5.0, 5.0),
        Color::Rgb(200, 100, 50),
        '+',
        1.0, // fully aged
    )]);
    draw_particles(&sys, &mut buf, area, (0, 0));

    let cell = buf.cell((5, 5)).expect("cell should exist");
    assert_eq!(cell.symbol(), "+");
    assert_eq!(cell.fg, Color::Rgb(0, 0, 0));
}

#[test]
fn draw_particles_skips_particle_with_negative_projected_x() {
    let area = Rect::new(0, 0, 10, 10);
    let mut buf = Buffer::empty(area);
    // pos=(-1.0, 5.0) + origin=(0,0) → projected cx=-1 → skip
    let sys = system_with(vec![make_particle(
        (-1.0, 5.0),
        Color::Rgb(255, 0, 0),
        '*',
        0.0,
    )]);
    draw_particles(&sys, &mut buf, area, (0, 0));

    // Nothing in the buffer should be changed from the default reset state
    // Check that cell (0, 5) is still empty (default symbol is space)
    let cell = buf.cell((0, 5)).expect("cell should exist");
    assert_eq!(cell.symbol(), " ");
}

#[test]
fn draw_particles_skips_particle_with_negative_projected_y() {
    let area = Rect::new(0, 0, 10, 10);
    let mut buf = Buffer::empty(area);
    let sys = system_with(vec![make_particle(
        (5.0, -1.0),
        Color::Rgb(255, 0, 0),
        '*',
        0.0,
    )]);
    draw_particles(&sys, &mut buf, area, (0, 0));

    let cell = buf.cell((5, 0)).expect("cell should exist");
    assert_eq!(cell.symbol(), " ");
}

#[test]
fn draw_particles_skips_particle_beyond_area_right() {
    let area = Rect::new(0, 0, 10, 10);
    let mut buf = Buffer::empty(area);
    // pos=(10.0, 5.0) → cx=10, area.right()=10, so 10 >= 10 → skip
    let sys = system_with(vec![make_particle(
        (10.0, 5.0),
        Color::Rgb(255, 0, 0),
        '*',
        0.0,
    )]);
    draw_particles(&sys, &mut buf, area, (0, 0));

    // Should not panic; buffer last column (9) should be untouched
    let cell = buf.cell((9, 5)).expect("cell should exist");
    assert_eq!(cell.symbol(), " ");
}

#[test]
fn draw_particles_skips_particle_beyond_area_bottom() {
    let area = Rect::new(0, 0, 10, 10);
    let mut buf = Buffer::empty(area);
    let sys = system_with(vec![make_particle(
        (5.0, 10.0),
        Color::Rgb(255, 0, 0),
        '*',
        0.0,
    )]);
    draw_particles(&sys, &mut buf, area, (0, 0));

    let cell = buf.cell((5, 9)).expect("cell should exist");
    assert_eq!(cell.symbol(), " ");
}

#[test]
fn draw_particles_respects_area_offset() {
    // Buffer is 20x10; area is the right half: Rect::new(10, 0, 10, 10)
    // Particle at pos (0.0, 0.0) + origin=(10, 0) → cx=10 which is area.left() → paint
    let buf_area = Rect::new(0, 0, 20, 10);
    let area = Rect::new(10, 0, 10, 10);
    let mut buf = Buffer::empty(buf_area);
    let sys = system_with(vec![make_particle(
        (0.0, 0.0),
        Color::Rgb(255, 255, 0),
        '@',
        0.0,
    )]);
    draw_particles(&sys, &mut buf, area, (10, 0));

    let cell = buf.cell((10, 0)).expect("cell should exist");
    assert_eq!(cell.symbol(), "@");
}

#[test]
fn draw_particles_same_cell_last_write_wins() {
    // Two particles at the same position — second one should win
    let area = Rect::new(0, 0, 10, 10);
    let mut buf = Buffer::empty(area);
    let p1 = make_particle((3.0, 3.0), Color::Rgb(255, 0, 0), 'A', 0.0);
    let p2 = make_particle((3.0, 3.0), Color::Rgb(0, 255, 0), 'B', 0.0);
    let sys = system_with(vec![p1, p2]);
    draw_particles(&sys, &mut buf, area, (0, 0));

    let cell = buf.cell((3, 3)).expect("cell should exist");
    // Last write should win — p2 was added second so it's iterated last
    assert_eq!(cell.symbol(), "B");
}

#[test]
fn draw_particles_empty_system_no_panic() {
    let area = Rect::new(0, 0, 10, 10);
    let mut buf = Buffer::empty(area);
    let sys = ParticleSystem::new();
    draw_particles(&sys, &mut buf, area, (0, 0));
    // No panic; buffer is unchanged
    let cell = buf.cell((0, 0)).expect("cell should exist");
    assert_eq!(cell.symbol(), " ");
}

#[test]
fn draw_particles_origin_offset_shifts_all_particles() {
    // Particle at pos (1.0, 1.0) with origin (5, 3) should land at (6, 4)
    let area = Rect::new(0, 0, 20, 20);
    let mut buf = Buffer::empty(area);
    let sys = system_with(vec![make_particle(
        (1.0, 1.0),
        Color::Rgb(100, 200, 50),
        '#',
        0.0,
    )]);
    draw_particles(&sys, &mut buf, area, (5, 3));

    let cell = buf.cell((6, 4)).expect("cell should exist");
    assert_eq!(cell.symbol(), "#");
    // Cell at (1,1) should be untouched
    let other = buf.cell((1, 1)).expect("cell should exist");
    assert_eq!(other.symbol(), " ");
}
