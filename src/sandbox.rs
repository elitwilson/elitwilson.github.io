use crate::effects::{EffectKind, spawn};
use crate::input::KeyCode;
use crate::particle_render::draw_particles;
use crate::particles::ParticleSystem;
use crate::rng::Rng;
use crate::sandbox_config::{ConfigField, FIELDS, SandboxConfig, step};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use std::time::Duration;

/// Fixed seed for the sandbox PRNG — reproducible run-to-run.
const SANDBOX_SEED: u64 = 0xdeadbeef_cafe1234;

/// All available effect kinds. When STR-002 exposes an ALL const this can be
/// replaced; for now we define the canonical list locally.
const EFFECT_KINDS: &[EffectKind] = &[EffectKind::Fireworks];

/// What a keypress resolves to in the sandbox (panel closed).
#[derive(Debug, PartialEq, Eq)]
pub enum SandboxCommand {
    Quit,
    CycleEffect,
    ToggleConfig,
    Switch,
    Ignore,
}

/// Pure mapping from a key to a sandbox command. No IO — testable seam.
pub fn map_sandbox_key(code: KeyCode) -> SandboxCommand {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => SandboxCommand::Quit,
        KeyCode::Tab => SandboxCommand::CycleEffect,
        KeyCode::Char('c') => SandboxCommand::ToggleConfig,
        KeyCode::Char('p') => SandboxCommand::Switch,
        _ => SandboxCommand::Ignore,
    }
}

/// What a keypress resolves to while the config panel is open.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigCommand {
    SelectPrev,
    SelectNext,
    Step(i32),
    Close,
    Ignore,
}

/// Pure mapping from a key to a config panel command. No IO — testable seam.
pub fn map_config_key(code: KeyCode) -> ConfigCommand {
    match code {
        KeyCode::Up => ConfigCommand::SelectPrev,
        KeyCode::Down => ConfigCommand::SelectNext,
        KeyCode::Left => ConfigCommand::Step(-1),
        KeyCode::Right => ConfigCommand::Step(1),
        KeyCode::Char('c') | KeyCode::Esc => ConfigCommand::Close,
        _ => ConfigCommand::Ignore,
    }
}

/// Compute the center of a `Rect` in f32 cell-space, including the area offset.
///
/// Particle coordinates are relative to the terminal buffer origin, so we must
/// add the area's top-left to the half-extents.
pub fn area_center(area: Rect) -> (f32, f32) {
    let cx = area.x as f32 + area.width as f32 / 2.0;
    let cy = area.y as f32 + area.height as f32 / 2.0;
    (cx, cy)
}

/// Choose the burst origin in body-relative cell-space.
///
/// Returns the mouse cell when known, otherwise the body center — both expressed
/// relative to `area`'s top-left, which is what `spawn` and `draw_particles`
/// expect (particle positions are body-relative; the renderer re-applies the
/// area offset when drawing).
pub fn spawn_origin(area: Rect, mouse: Option<(u16, u16)>) -> (f32, f32) {
    match mouse {
        Some((mx, my)) => (mx as f32 - area.x as f32, my as f32 - area.y as f32),
        None => {
            let (cx, cy) = area_center(area);
            (cx - area.x as f32, cy - area.y as f32)
        }
    }
}

/// Advance the effect-kind index by one, wrapping at the end of the list.
///
/// With `len == 1` this always returns `0` — no-op on the value but the
/// dispatch path (title line + spawn call) still exercises the current kind.
pub fn next_kind(idx: usize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (idx + 1) % len
}

/// How many spawns should fire this frame and what remainder carries over.
///
/// Given the accumulated `dt` and the `interval`, returns `(count, remainder)`
/// where `count` is the number of times `interval` fits in `accumulator + dt`
/// and `remainder` is what's left over.
pub fn cadence_step(accumulator: Duration, dt: Duration, interval: Duration) -> (u32, Duration) {
    let total = accumulator + dt;
    if interval.is_zero() {
        return (0, total);
    }
    let count = total.as_nanos() / interval.as_nanos();
    let remainder = total - interval * count as u32;
    (count as u32, remainder)
}

/// Render the config overlay onto `frame`.
///
/// Draws a centered bordered panel listing all adjustable fields with their
/// current values. The selected row is highlighted in yellow+bold. The sandbox
/// keeps rendering behind the panel (caller draws particles first, then calls
/// this to overlay).
pub fn draw_config_panel(frame: &mut Frame, config: &SandboxConfig) {
    let full = frame.area();

    // Centre a panel that is 36 columns wide and has one row per field plus
    // 2 border rows and 1 header padding row = FIELDS.len() + 3 rows tall.
    let panel_w = 36u16;
    let panel_h = (FIELDS.len() as u16) + 3;
    let x = full.width.saturating_sub(panel_w) / 2;
    let y = full.height.saturating_sub(panel_h) / 2;
    let panel = Rect::new(x, y, panel_w.min(full.width), panel_h.min(full.height));

    // Clear the cells behind the panel so particle glyphs don't show through.
    frame.render_widget(Clear, panel);

    let block = Block::default()
        .title(" Config ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::Black));
    frame.render_widget(block, panel);

    // Inner area: inside the border (1-cell inset on all sides).
    let inner = Rect::new(
        panel.x + 1,
        panel.y + 1,
        panel.width.saturating_sub(2),
        panel.height.saturating_sub(2),
    );

    for (i, field) in FIELDS.iter().enumerate() {
        let label = field_label(*field);
        let value = field_value(config, *field);
        let text = format!("{label}: {value}");

        let style = if i == config.selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let row = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
        if row.y < panel.y + panel.height - 1 {
            frame.render_widget(Paragraph::new(Line::from(Span::styled(text, style))), row);
        }
    }
}

/// Human-readable label for a config field.
fn field_label(field: ConfigField) -> &'static str {
    match field {
        ConfigField::Count => "count",
        ConfigField::Spread => "spread (rad)",
        ConfigField::SpeedMin => "speed_min",
        ConfigField::SpeedMax => "speed_max",
        ConfigField::LifetimeMin => "lifetime_min",
        ConfigField::LifetimeMax => "lifetime_max",
        ConfigField::SpawnInterval => "spawn_interval",
    }
}

/// Formatted current value of a config field.
fn field_value(config: &SandboxConfig, field: ConfigField) -> String {
    match field {
        ConfigField::Count => format!("{}", config.params.count),
        ConfigField::Spread => format!("{:.2}", config.params.spread),
        ConfigField::SpeedMin => format!("{:.1}", config.params.speed.0),
        ConfigField::SpeedMax => format!("{:.1}", config.params.speed.1),
        ConfigField::LifetimeMin => format!("{:.1}", config.params.lifetime.0),
        ConfigField::LifetimeMax => format!("{:.1}", config.params.lifetime.1),
        ConfigField::SpawnInterval => format!("{}ms", config.spawn_interval.as_millis()),
    }
}

/// The particle sandbox screen.
///
/// Owns its own `ParticleSystem` and a seeded `Rng`. Auto-spawns fireworks on a
/// recurring cadence at the mouse position (or body center until the mouse first
/// moves). Tab cycles the effect kind; `c` toggles the config panel; `p` returns
/// to the game; Esc/q goes back to the menu.
///
/// Mouse position is fed in via [`set_mouse`](Self::set_mouse) — each runner is
/// responsible for sourcing it (crossterm mouse capture natively, ratzilla's
/// `on_mouse_event` on the web).
pub struct SandboxScreen {
    system: ParticleSystem,
    rng: Rng,
    config: SandboxConfig,
    kind_idx: usize,
    spawn_acc: Duration,
    config_open: bool,
    last_area: Option<Rect>,
    last_mouse: Option<(u16, u16)>,
}

impl SandboxScreen {
    pub fn new() -> Self {
        Self {
            system: ParticleSystem::new(),
            rng: Rng::new(SANDBOX_SEED),
            config: SandboxConfig::default(),
            kind_idx: 0,
            spawn_acc: Duration::ZERO,
            config_open: false,
            last_area: None,
            last_mouse: None,
        }
    }

    /// Record the latest cursor cell. Bursts spawn here; mouse-follow continues
    /// regardless of whether the config panel is open.
    pub fn set_mouse(&mut self, pos: (u16, u16)) {
        self.last_mouse = Some(pos);
    }

    pub fn handle_key(&mut self, code: KeyCode) -> Option<crate::Nav> {
        if self.config_open {
            // Config panel captures all keys while open.
            match map_config_key(code) {
                ConfigCommand::SelectPrev => self.config.select_prev(),
                ConfigCommand::SelectNext => self.config.select_next(),
                ConfigCommand::Step(dir) => {
                    let field = self.config.selected_field();
                    step(&mut self.config, field, dir);
                }
                ConfigCommand::Close => self.config_open = false,
                ConfigCommand::Ignore => {}
            }
        } else {
            match map_sandbox_key(code) {
                SandboxCommand::Quit => return Some(crate::Nav::To(crate::Screen::Menu)),
                SandboxCommand::Switch => return Some(crate::Nav::To(crate::Screen::Game)),
                SandboxCommand::CycleEffect => {
                    self.kind_idx = next_kind(self.kind_idx, EFFECT_KINDS.len());
                }
                SandboxCommand::ToggleConfig => self.config_open = true,
                SandboxCommand::Ignore => {}
            }
        }
        None
    }

    pub fn render(&mut self, frame: &mut Frame) -> Rect {
        let full = frame.area();

        // Split off a 1-row title at the top, mirroring render::ui's layout.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(full);

        let title_area = chunks[0];
        let body = chunks[1];

        let kind_name = match EFFECT_KINDS[self.kind_idx] {
            EffectKind::Fireworks => "Fireworks",
        };
        let panel_hint = if self.config_open {
            " c/Esc: close panel "
        } else {
            " c: config "
        };
        let hint = format!(
            " Sandbox | Effect: {kind_name} | Follows mouse | Tab: cycle | p: game | q/Esc: quit |{panel_hint}"
        );
        let title = Paragraph::new(Span::styled(hint, Style::default().fg(Color::Yellow)))
            .block(Block::default());
        frame.render_widget(title, title_area);

        // Clear the body with a dark background so particles read clearly.
        let bg = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(bg, body);

        // Draw live particles. Origin (0,0) is the top-left of the body area;
        // particle positions are body-relative because center is computed from body.
        draw_particles(&self.system, frame.buffer_mut(), body, (body.x, body.y));

        // Overlay the config panel when open — drawn after particles so it
        // appears on top of the simulation.
        if self.config_open {
            draw_config_panel(frame, &self.config);
        }

        // Track the live body area so center is always current after a resize.
        self.last_area = Some(body);
        body
    }

    pub fn tick(&mut self, dt: Duration) {
        // --- auto-spawn at the mouse (or center until the mouse first moves) ---
        // Reads live from config so cadence and burst shape update immediately.
        if let Some(area) = self.last_area {
            let (count, remainder) = cadence_step(self.spawn_acc, dt, self.config.spawn_interval);
            self.spawn_acc = remainder;
            let origin = spawn_origin(area, self.last_mouse);
            for _ in 0..count {
                spawn(
                    EFFECT_KINDS[self.kind_idx],
                    origin,
                    &self.config.params,
                    &mut self.rng,
                    &mut self.system,
                );
            }
        }

        self.system.tick(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    // --- area_center ---

    #[test]
    fn area_center_of_simple_rect() {
        let area = Rect::new(0, 0, 80, 24);
        let (cx, cy) = area_center(area);
        assert!((cx - 40.0).abs() < 1e-4, "cx: {cx}");
        assert!((cy - 12.0).abs() < 1e-4, "cy: {cy}");
    }

    #[test]
    fn area_center_includes_area_offset() {
        // Area starts at (10, 5) with size 20x10 → center at (10+10, 5+5) = (20, 10)
        let area = Rect::new(10, 5, 20, 10);
        let (cx, cy) = area_center(area);
        assert!((cx - 20.0).abs() < 1e-4, "cx: {cx}");
        assert!((cy - 10.0).abs() < 1e-4, "cy: {cy}");
    }

    #[test]
    fn area_center_odd_dimensions() {
        // 81x25 → center at 40.5, 12.5
        let area = Rect::new(0, 0, 81, 25);
        let (cx, cy) = area_center(area);
        assert!((cx - 40.5).abs() < 1e-4, "cx: {cx}");
        assert!((cy - 12.5).abs() < 1e-4, "cy: {cy}");
    }

    // --- spawn_origin ---

    #[test]
    fn spawn_origin_uses_mouse_relative_to_area() {
        // Body starts at (0, 1); cursor at absolute (30, 11) is (30, 10) body-relative.
        let area = Rect::new(0, 1, 80, 23);
        let (x, y) = spawn_origin(area, Some((30, 11)));
        assert!((x - 30.0).abs() < 1e-4, "x: {x}");
        assert!((y - 10.0).abs() < 1e-4, "y: {y}");
    }

    #[test]
    fn spawn_origin_falls_back_to_center_without_mouse() {
        // No mouse yet → body center, expressed relative to the body origin.
        // area center abs = (40, 13); relative = (40, 12).
        let area = Rect::new(0, 1, 80, 24);
        let (x, y) = spawn_origin(area, None);
        assert!((x - 40.0).abs() < 1e-4, "x: {x}");
        assert!((y - 12.0).abs() < 1e-4, "y: {y}");
    }

    // --- next_kind ---

    #[test]
    fn next_kind_wraps_at_end() {
        // With len=3: 0→1→2→0
        assert_eq!(next_kind(0, 3), 1);
        assert_eq!(next_kind(1, 3), 2);
        assert_eq!(next_kind(2, 3), 0);
    }

    #[test]
    fn next_kind_len_one_stays_zero() {
        // With only one kind the index always stays at 0.
        assert_eq!(next_kind(0, 1), 0);
    }

    #[test]
    fn next_kind_zero_len_returns_zero() {
        // Guard against empty list.
        assert_eq!(next_kind(0, 0), 0);
    }

    // --- map_sandbox_key ---

    #[test]
    fn esc_maps_to_quit() {
        assert_eq!(map_sandbox_key(KeyCode::Esc), SandboxCommand::Quit);
    }

    #[test]
    fn q_maps_to_quit() {
        assert_eq!(map_sandbox_key(KeyCode::Char('q')), SandboxCommand::Quit);
    }

    #[test]
    fn tab_maps_to_cycle_effect() {
        assert_eq!(map_sandbox_key(KeyCode::Tab), SandboxCommand::CycleEffect);
    }

    #[test]
    fn unknown_key_maps_to_ignore() {
        assert_eq!(map_sandbox_key(KeyCode::Char('z')), SandboxCommand::Ignore);
        assert_eq!(map_sandbox_key(KeyCode::Enter), SandboxCommand::Ignore);
    }

    // --- map_sandbox_key: ToggleConfig ---

    #[test]
    fn c_maps_to_toggle_config() {
        assert_eq!(
            map_sandbox_key(KeyCode::Char('c')),
            SandboxCommand::ToggleConfig
        );
    }

    #[test]
    fn p_maps_to_switch() {
        assert_eq!(map_sandbox_key(KeyCode::Char('p')), SandboxCommand::Switch);
    }

    // --- map_config_key ---

    #[test]
    fn config_up_maps_to_select_prev() {
        assert_eq!(map_config_key(KeyCode::Up), ConfigCommand::SelectPrev);
    }

    #[test]
    fn config_down_maps_to_select_next() {
        assert_eq!(map_config_key(KeyCode::Down), ConfigCommand::SelectNext);
    }

    #[test]
    fn config_left_maps_to_step_dec() {
        assert_eq!(map_config_key(KeyCode::Left), ConfigCommand::Step(-1));
    }

    #[test]
    fn config_right_maps_to_step_inc() {
        assert_eq!(map_config_key(KeyCode::Right), ConfigCommand::Step(1));
    }

    #[test]
    fn config_c_maps_to_close() {
        assert_eq!(map_config_key(KeyCode::Char('c')), ConfigCommand::Close);
    }

    #[test]
    fn config_esc_maps_to_close() {
        assert_eq!(map_config_key(KeyCode::Esc), ConfigCommand::Close);
    }

    #[test]
    fn config_q_is_inert() {
        assert_eq!(map_config_key(KeyCode::Char('q')), ConfigCommand::Ignore);
    }

    #[test]
    fn config_tab_is_inert() {
        assert_eq!(map_config_key(KeyCode::Tab), ConfigCommand::Ignore);
    }

    #[test]
    fn config_unknown_key_is_inert() {
        assert_eq!(map_config_key(KeyCode::Enter), ConfigCommand::Ignore);
    }

    // --- cadence_step ---

    #[test]
    fn cadence_no_spawn_below_interval() {
        let interval = Duration::from_millis(800);
        let (count, remainder) = cadence_step(Duration::ZERO, Duration::from_millis(16), interval);
        assert_eq!(count, 0);
        assert_eq!(remainder, Duration::from_millis(16));
    }

    #[test]
    fn cadence_one_spawn_when_interval_crossed() {
        let interval = Duration::from_millis(800);
        // accumulator at 790ms + dt 16ms = 806ms → 1 spawn, 6ms remainder
        let (count, remainder) = cadence_step(
            Duration::from_millis(790),
            Duration::from_millis(16),
            interval,
        );
        assert_eq!(count, 1);
        assert_eq!(remainder, Duration::from_millis(6));
    }

    #[test]
    fn cadence_two_spawns_when_two_intervals_crossed() {
        let interval = Duration::from_millis(100);
        // accumulator 0 + dt 250ms = 250ms → 2 full intervals, 50ms remainder
        let (count, remainder) = cadence_step(Duration::ZERO, Duration::from_millis(250), interval);
        assert_eq!(count, 2);
        assert_eq!(remainder, Duration::from_millis(50));
    }

    #[test]
    fn cadence_carries_remainder_forward() {
        let interval = Duration::from_millis(800);
        // Step 1: 0 + 600ms → 0 spawns, 600ms left
        let (c1, r1) = cadence_step(Duration::ZERO, Duration::from_millis(600), interval);
        assert_eq!(c1, 0);
        // Step 2: 600ms + 400ms = 1000ms → 1 spawn, 200ms left
        let (c2, r2) = cadence_step(r1, Duration::from_millis(400), interval);
        assert_eq!(c2, 1);
        assert_eq!(r2, Duration::from_millis(200));
    }
}
