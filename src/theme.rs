use ratatui::style::Color;

pub struct Theme {
    wall: Color,
    floor: Color,
    outside: Color,
    player: Color,
    key: Color,
    door: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            wall: Color::Green,
            floor: Color::Black,
            outside: Color::Black,
            player: Color::Green,
            key: Color::Green,
            door: Color::White,
        }
    }
}