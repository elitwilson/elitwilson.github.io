use crate::{map::{self, demo_castle}, theme::Theme};
use ratatui::DefaultTerminal;
use crate::render::ui;

pub struct App {
    map: map::Map,
    player_pos: (u16, u16), // x, y position on the map.
    key_pos: (u16, u16),    // where the key sits on the map.
    has_key: bool,
    door_open: bool,
    show_about: bool,
    theme: Theme,
}

pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    ToggleAbout,
    // etc.
}

impl App {
    pub fn new() -> Self {
        Self {
            map: demo_castle(),
            player_pos: (3, 4), // bottom-middle of the interior floor
            key_pos: (2, 3),    // left side, like the original game
            has_key: false,
            door_open: false,
            show_about: false,
            theme: Theme::default(),
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::MoveUp => self.try_move(0, -1),
            Action::MoveDown => self.try_move(0, 1),
            Action::MoveLeft => self.try_move(-1, 0),
            Action::MoveRight => self.try_move(1, 0),
            Action::ToggleAbout => self.show_about = !self.show_about,
        }
    }

    /// Move the player by a signed delta, but only if the target is walkable.
    fn try_move(&mut self, dx: i32, dy: i32) {
        let nx = self.player_pos.0 as i32 + dx;
        let ny = self.player_pos.1 as i32 + dy;

        // walkable rejects negative and out-of-range coords, so if it passes
        // we know nx/ny are valid, non-negative, and safe to store as u16.
        if self.map.walkable(nx, ny) {
            self.player_pos = (nx as u16, ny as u16);

            // Picking up the key only makes sense on a tile we actually moved to.
            if self.player_pos == self.key_pos {
                self.has_key = true;
            }
        }
    }
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(ui)?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with(player: (u16, u16), key: (u16, u16)) -> App {
        App {
            map: demo_castle(),
            player_pos: player,
            key_pos: key,
            has_key: false,
            door_open: false,
            show_about: false,
            theme: Theme::default(),
        }
    }

    // Movement tests don't care about the key, so park it out of the way.
    fn app_at(pos: (u16, u16)) -> App {
        app_with(pos, (4, 4))
    }

    #[test]
    fn app_initial_state() {
        let app = App::new();
        assert_eq!(app.player_pos, (3, 4));
        assert!(!app.has_key);
        assert!(!app.door_open);
        assert!(!app.show_about);
    }

    #[test]
    fn app_update_toggles_about() {
        let mut app = App::new();
        app.update(Action::ToggleAbout);
        assert!(app.show_about);
    }

    // Moving onto a floor tile updates the player's position.
    #[test]
    fn move_onto_floor_updates_position() {
        let mut app = app_at((2, 2)); // floor; (3,2) is also floor
        app.update(Action::MoveRight);
        assert_eq!(app.player_pos, (3, 2));
    }

    // Moving into a wall leaves the player where they were.
    #[test]
    fn move_into_wall_is_blocked() {
        let mut app = app_at((2, 3)); // floor; (3,3) to the right is the center wall
        app.update(Action::MoveRight);
        assert_eq!(app.player_pos, (2, 3));
    }

    // Moving past the edge (coordinate 0) must not underflow or move.
    #[test]
    fn move_off_edge_is_blocked() {
        let mut app = app_at((0, 0));
        app.update(Action::MoveLeft); // would be (-1, 0) -> underflow if unguarded
        assert_eq!(app.player_pos, (0, 0));
    }

    // Stepping onto the key's tile picks it up.
    #[test]
    fn stepping_onto_key_picks_it_up() {
        // player at (2,2), key one step to the right at (3,2) (both floor)
        let mut app = app_with((2, 2), (3, 2));
        assert!(!app.has_key); // sanity: not collected yet
        app.update(Action::MoveRight);
        assert_eq!(app.player_pos, (3, 2)); // actually moved onto it
        assert!(app.has_key); // and picked it up
    }

    // Stepping onto an ordinary floor tile (not the key) doesn't grant the key.
    #[test]
    fn stepping_onto_floor_does_not_pick_up_key() {
        let mut app = app_with((2, 2), (4, 4)); // key is elsewhere
        app.update(Action::MoveRight); // moves onto (3,2), a plain floor tile
        assert!(!app.has_key);
    }

    // Once collected, the key stays collected after the player moves away.
    #[test]
    fn key_stays_collected_after_moving_away() {
        let mut app = app_with((2, 2), (3, 2));
        app.update(Action::MoveRight); // pick up key at (3,2)
        assert!(app.has_key);
        app.update(Action::MoveRight); // move on to (4,2)
        assert_eq!(app.player_pos, (4, 2));
        assert!(app.has_key); // still have it
    }
}
