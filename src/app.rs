use crate::theme::Theme;
use ratatui::DefaultTerminal;
use crate::render::ui;

pub struct App {
    player_pos: (u16, u16), // x, y position on the map.
    has_key: bool,
    door_open: bool,
    show_about: bool,
    theme: Theme,
}

impl App {
    pub fn new() -> Self {
        Self {
            player_pos: (0, 0),
            has_key: false,
            door_open: false,
            show_about: false,
            theme: Theme::default(),
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