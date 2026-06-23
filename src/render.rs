use crate::app::App;
use crate::map::Tile;
use crate::particle_render::draw_particles;
use crate::particles::ParticleSystem;
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};

/// Tiles are drawn this many cells wide (and one cell tall) so they look square,
/// since terminal cells are about twice as tall as they are wide.
const TILE_W: u16 = 2;

/// Render the game and return the body `Rect` (the area below the title), which
/// the caller uses to place the victory fireworks.
pub fn ui(frame: &mut Frame, app: &App, particles: &ParticleSystem) -> Rect {
    let [title_area, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(frame.area());

    let hint = if app.has_key() {
        "You have the key — reach the door!   (WASD/arrows · i: about · p: particles · q: quit)"
    } else {
        "Find the key, then open the door.   (WASD/arrows · i: about · p: particles · q: quit)"
    };
    frame.render_widget(Line::from(hint).centered(), title_area);

    let map = app.map();
    let theme = app.theme();
    let (ox, oy) = map_origin(body, map.width(), map.height());

    // The castle itself.
    for ty in 0..map.height() {
        for tx in 0..map.width() {
            let color = match map.tile(tx, ty) {
                Tile::Wall => theme.wall,
                Tile::Floor => theme.floor,
                Tile::Outside => theme.outside,
            };
            fill_tile(frame, ox, oy, tx, ty, color);
        }
    }

    // Entities on top of the floor. The key disappears once collected.
    if !app.has_key() {
        let (kx, ky) = app.key_pos();
        fill_tile(frame, ox, oy, kx, ky, theme.key);
    }
    let (dx, dy) = app.door_pos();
    fill_tile(frame, ox, oy, dx, dy, theme.door);
    let (px, py) = app.player_pos();
    fill_tile(frame, ox, oy, px, py, theme.player);

    // Victory fireworks sit above the scene but below the modal. Drawn last so
    // they overlay the castle; `render_about` then clears its own rect on top,
    // so the bursts read as being *behind* the modal.
    draw_particles(particles, frame.buffer_mut(), body, (body.x, body.y));

    if app.show_about() {
        render_modal(frame, body, app.won());
    }

    body
}

/// Paint a single map tile (TILE_W cells wide) with a background color.
fn fill_tile(frame: &mut Frame, ox: u16, oy: u16, tx: u16, ty: u16, color: Color) {
    let sx = ox + tx * TILE_W;
    let sy = oy + ty;
    let buf = frame.buffer_mut();
    for i in 0..TILE_W {
        if let Some(cell) = buf.cell_mut((sx + i, sy)) {
            cell.set_symbol(" ");
            cell.set_bg(color);
        }
    }
}

/// Centered modal painted over the game. On a win it's the victory banner;
/// otherwise it's the plain About page (toggled with `i`).
fn render_modal(frame: &mut Frame, area: Rect, won: bool) {
    let modal = centered_rect(area, 40, 12);
    frame.render_widget(Clear, modal); // wipe whatever's underneath
    let (title, text) = if won {
        ("Victory", "You won!")
    } else {
        ("About", "about page")
    };
    let block = Block::bordered().title(title);
    frame.render_widget(Paragraph::new(text).centered().block(block), modal);
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let [h] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [v] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(h);
    v
}

/// Top-left cell at which to draw the map so it sits centered in `area`.
/// Clamps to the area origin if the map is larger than the space available.
fn map_origin(area: Rect, map_w: u16, map_h: u16) -> (u16, u16) {
    let pixel_w = map_w * TILE_W;
    let pixel_h = map_h;
    // saturating_sub keeps us pinned to the origin (no underflow) if the map
    // is bigger than the area.
    let x = area.x + area.width.saturating_sub(pixel_w) / 2;
    let y = area.y + area.height.saturating_sub(pixel_h) / 2;
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_map_within_area() {
        let area = Rect::new(0, 0, 20, 10);
        // 5x5 map -> 10 cells wide, 5 tall -> centered origin (5, 2)
        assert_eq!(map_origin(area, 5, 5), (5, 2));
    }

    #[test]
    fn respects_area_offset() {
        // body sits below a 1-row title (area starts at y = 1)
        let area = Rect::new(0, 1, 20, 9);
        assert_eq!(map_origin(area, 5, 5), (5, 3));
    }

    #[test]
    fn clamps_when_map_larger_than_area() {
        let area = Rect::new(0, 0, 8, 4); // smaller than the 10x5 map
        assert_eq!(map_origin(area, 5, 5), (0, 0)); // pinned to origin, no underflow
    }
}
