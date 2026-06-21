use ratatui::style::Color;

/// A swappable color palette. Roles are named so the renderer never mentions a
/// literal color — switching the whole look is a one-line change of theme.
pub struct Theme {
    pub wall: Color,
    pub floor: Color,
    pub outside: Color,
    pub player: Color,
    pub key: Color,
    pub door: Color,
}

impl Default for Theme {
    fn default() -> Self {
        // A green "hacker terminal" palette.
        Self {
            wall: Color::Rgb(0, 180, 0),
            floor: Color::Rgb(10, 30, 10),
            outside: Color::Black,
            player: Color::Rgb(0, 255, 0),
            key: Color::Rgb(180, 255, 120),
            door: Color::Rgb(120, 120, 120),
        }
    }
}
