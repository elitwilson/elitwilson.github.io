use crate::input::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::block_font;

const TITLE_FG: Color = Color::Rgb(0, 255, 0);
const TITLE_SHADOW: Color = Color::Rgb(0, 80, 0);

/// External link opened by the GitHub menu item.
const GITHUB_URL: &str = "https://github.com/elitwilson";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Play,
    GitHub,
    About,
    Quit,
}

pub(crate) const ITEMS: &[MenuItem] = &[
    MenuItem::Play,
    MenuItem::GitHub,
    MenuItem::About,
    MenuItem::Quit,
];

impl MenuItem {
    fn label(self) -> &'static str {
        match self {
            MenuItem::Play => "Play",
            MenuItem::GitHub => "GitHub",
            MenuItem::About => "About",
            MenuItem::Quit => "Quit",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MenuCommand {
    Up,
    Down,
    Select,
    Quit,
    Ignore,
}

pub fn map_menu_key(code: KeyCode) -> MenuCommand {
    match code {
        KeyCode::Up | KeyCode::Char('w') => MenuCommand::Up,
        KeyCode::Down | KeyCode::Char('s') => MenuCommand::Down,
        KeyCode::Enter => MenuCommand::Select,
        KeyCode::Esc | KeyCode::Char('q') => MenuCommand::Quit,
        _ => MenuCommand::Ignore,
    }
}

pub struct Menu {
    pub selected: usize,
}

impl Menu {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn up(&mut self) {
        if self.selected == 0 {
            self.selected = ITEMS.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn down(&mut self) {
        self.selected = (self.selected + 1) % ITEMS.len();
    }

    pub fn selected_item(&self) -> MenuItem {
        ITEMS[self.selected]
    }
}

pub fn activate(item: MenuItem) -> crate::Nav {
    match item {
        MenuItem::Play => crate::Nav::To(crate::Screen::Game),
        MenuItem::GitHub => crate::Nav::OpenUrl(GITHUB_URL),
        MenuItem::About => crate::Nav::To(crate::Screen::About),
        MenuItem::Quit => crate::Nav::Quit,
    }
}

/// Draw one block-font line, horizontally centered within `area`, with the
/// title colors and drop shadow.
fn draw_centered_banner(frame: &mut Frame, area: Rect, text: &str) {
    let lines = block_font::compose(text, 1);
    let banner_w = lines[0].chars().count() as u16;
    let banner_x = area.x + area.width.saturating_sub(banner_w) / 2;
    block_font::draw_banner(
        frame.buffer_mut(),
        (banner_x, area.y),
        &lines,
        TITLE_FG,
        TITLE_SHADOW,
    );
}

fn render_menu(frame: &mut Frame, menu: &Menu) {
    let area = frame.area();

    // Each block-font line occupies HEIGHT rows (glyph rows + the drop shadow,
    // which falls into the blank padding row that `compose` already includes).
    let banner_h = block_font::HEIGHT as u16;
    let item_h = ITEMS.len() as u16;

    // Two stacked name lines, a smaller subtitle, the menu items, and the footer
    // — with single-row gaps separating the groups.
    let total_content = banner_h    // ELI
        + banner_h                  // WILSON
        + 1                         // gap
        + 1                         // subtitle
        + 1                         // gap
        + item_h                    // menu items
        + 1                         // gap
        + 1; // footer

    let top_pad = area.height.saturating_sub(total_content) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad),
            Constraint::Length(banner_h), // 1: ELI
            Constraint::Length(banner_h), // 2: WILSON
            Constraint::Length(1),        // 3: gap
            Constraint::Length(1),        // 4: subtitle
            Constraint::Length(1),        // 5: gap
            Constraint::Length(item_h),   // 6: items
            Constraint::Length(1),        // 7: gap
            Constraint::Length(1),        // 8: footer
            Constraint::Min(0),
        ])
        .split(area);

    let items_area = chunks[6];
    let footer_area = chunks[8];

    // Name in big block letters with a drop shadow, each line horizontally
    // centered in its row. The font is uppercase-only, so it reads as all-caps.
    draw_centered_banner(frame, chunks[1], "ELI");
    draw_centered_banner(frame, chunks[2], "WILSON");

    // Smaller subtitle under the name, in normal terminal-size text.
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Software Developer",
            Style::default().fg(Color::Rgb(0, 200, 0)),
        )))
        .alignment(Alignment::Center)
        .block(Block::default()),
        chunks[4],
    );

    // Item list
    for (i, item) in ITEMS.iter().enumerate() {
        let row = Rect::new(items_area.x, items_area.y + i as u16, items_area.width, 1);
        let style = if i == menu.selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(0, 255, 0))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(0, 200, 0))
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("  {}  ", item.label()),
                style,
            )))
            .alignment(Alignment::Center)
            .block(Block::default()),
            row,
        );
    }

    // Footer hint
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "↑/↓  w/s  select · Enter · q quit",
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center)
        .block(Block::default()),
        footer_area,
    );
}

impl Menu {
    /// Handle a keypress. Returns `Some(Nav)` when the press resolves to a
    /// navigation (selecting an item or quitting), `None` to stay on the menu.
    pub fn handle_key(&mut self, code: KeyCode) -> Option<crate::Nav> {
        match map_menu_key(code) {
            MenuCommand::Up => self.up(),
            MenuCommand::Down => self.down(),
            MenuCommand::Select => return Some(activate(self.selected_item())),
            MenuCommand::Quit => return Some(crate::Nav::Quit),
            MenuCommand::Ignore => {}
        }
        None
    }

    /// Draw the menu. Returns the full area (the menu has no separate body the
    /// animation layer cares about).
    pub fn render(&self, frame: &mut Frame) -> ratatui::layout::Rect {
        render_menu(frame, self);
        frame.area()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- map_menu_key ---

    #[test]
    fn arrow_up_maps_to_up() {
        assert_eq!(map_menu_key(KeyCode::Up), MenuCommand::Up);
    }

    #[test]
    fn w_maps_to_up() {
        assert_eq!(map_menu_key(KeyCode::Char('w')), MenuCommand::Up);
    }

    #[test]
    fn arrow_down_maps_to_down() {
        assert_eq!(map_menu_key(KeyCode::Down), MenuCommand::Down);
    }

    #[test]
    fn s_maps_to_down() {
        assert_eq!(map_menu_key(KeyCode::Char('s')), MenuCommand::Down);
    }

    #[test]
    fn enter_maps_to_select() {
        assert_eq!(map_menu_key(KeyCode::Enter), MenuCommand::Select);
    }

    #[test]
    fn esc_maps_to_quit() {
        assert_eq!(map_menu_key(KeyCode::Esc), MenuCommand::Quit);
    }

    #[test]
    fn q_maps_to_quit() {
        assert_eq!(map_menu_key(KeyCode::Char('q')), MenuCommand::Quit);
    }

    #[test]
    fn unknown_key_maps_to_ignore() {
        assert_eq!(map_menu_key(KeyCode::Char('z')), MenuCommand::Ignore);
        assert_eq!(map_menu_key(KeyCode::Tab), MenuCommand::Ignore);
    }

    // --- Menu::up() / down() wraparound ---

    #[test]
    fn down_advances_selection() {
        let mut m = Menu::new();
        assert_eq!(m.selected, 0);
        m.down();
        assert_eq!(m.selected, 1);
        m.down();
        assert_eq!(m.selected, 2);
    }

    #[test]
    fn down_wraps_from_last_to_first() {
        let mut m = Menu::new();
        for _ in 0..(ITEMS.len() - 1) {
            m.down();
        }
        assert_eq!(m.selected, ITEMS.len() - 1);
        m.down();
        assert_eq!(m.selected, 0);
    }

    #[test]
    fn up_wraps_from_first_to_last() {
        let mut m = Menu::new();
        assert_eq!(m.selected, 0);
        m.up();
        assert_eq!(m.selected, ITEMS.len() - 1);
    }

    #[test]
    fn up_decrements_selection() {
        let mut m = Menu::new();
        m.down();
        m.down(); // at index 2
        m.up();
        assert_eq!(m.selected, 1);
    }

    // --- activate: item → Nav ---

    #[test]
    fn activate_play_goes_to_game() {
        assert_eq!(
            activate(MenuItem::Play),
            crate::Nav::To(crate::Screen::Game)
        );
    }

    #[test]
    fn activate_about_goes_to_about() {
        assert_eq!(
            activate(MenuItem::About),
            crate::Nav::To(crate::Screen::About)
        );
    }

    #[test]
    fn activate_github_opens_the_link() {
        assert_eq!(activate(MenuItem::GitHub), crate::Nav::OpenUrl(GITHUB_URL));
    }

    #[test]
    fn activate_quit_returns_quit() {
        assert_eq!(activate(MenuItem::Quit), crate::Nav::Quit);
    }
}
