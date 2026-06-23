use ratatui::DefaultTerminal;

pub fn menu(_terminal: &mut DefaultTerminal) -> std::io::Result<crate::Nav> {
    Ok(crate::Nav::Quit)
}
