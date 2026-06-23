use ratatui::DefaultTerminal;

pub fn about(_terminal: &mut DefaultTerminal) -> std::io::Result<crate::Nav> {
    Ok(crate::Nav::To(crate::Screen::Menu))
}
