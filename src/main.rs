use ratatui::DefaultTerminal;

mod app;
mod render;
mod theme;
mod map;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app::app)?;
    Ok(())
}

