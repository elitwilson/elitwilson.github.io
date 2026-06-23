use crate::input::KeyCode;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

#[derive(Debug, PartialEq, Eq)]
pub enum AboutCommand {
    Back,
    Ignore,
}

pub fn map_about_key(code: KeyCode) -> AboutCommand {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => AboutCommand::Back,
        _ => AboutCommand::Ignore,
    }
}

/// The About screen carries no state — it's a static page until a key sends it
/// back to the menu. A unit struct keeps it uniform with the other screens so
/// the router can drive all four the same way.
pub struct About;

impl About {
    pub fn handle_key(&mut self, code: KeyCode) -> Option<crate::Nav> {
        match map_about_key(code) {
            AboutCommand::Back => Some(crate::Nav::To(crate::Screen::Menu)),
            AboutCommand::Ignore => None,
        }
    }

    pub fn render(&self, frame: &mut Frame) -> ratatui::layout::Rect {
        let area = frame.area();
        let text = vec![
            Line::from(Span::styled(
                "About — coming soon",
                Style::default().fg(Color::Rgb(0, 200, 0)),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Esc / q  back to menu",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        frame.render_widget(
            Paragraph::new(text)
                .alignment(Alignment::Center)
                .block(Block::default()),
            area,
        );
        area
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esc_maps_to_back() {
        assert_eq!(map_about_key(KeyCode::Esc), AboutCommand::Back);
    }

    #[test]
    fn q_maps_to_back() {
        assert_eq!(map_about_key(KeyCode::Char('q')), AboutCommand::Back);
    }

    #[test]
    fn unknown_key_maps_to_ignore() {
        assert_eq!(map_about_key(KeyCode::Enter), AboutCommand::Ignore);
        assert_eq!(map_about_key(KeyCode::Char('z')), AboutCommand::Ignore);
    }
}
