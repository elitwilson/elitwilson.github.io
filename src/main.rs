mod app;
mod effects;
mod map;
mod particle_render;
mod particles;
mod render;
mod rng;
mod theme;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app::app)?;
    Ok(())
}
