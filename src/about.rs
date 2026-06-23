use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
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

pub fn about(terminal: &mut DefaultTerminal) -> std::io::Result<crate::Nav> {
    loop {
        terminal.draw(|frame| {
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
        })?;

        if event::poll(std::time::Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match map_about_key(key.code) {
                AboutCommand::Back => return Ok(crate::Nav::To(crate::Screen::Menu)),
                AboutCommand::Ignore => {}
            }
        }
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
