mod app;
mod effects;
mod map;
mod particle_render;
mod particles;
mod render;
mod rng;
mod sandbox;
mod sandbox_config;
mod theme;
mod victory;

use ratatui::DefaultTerminal;

/// Why a screen's loop returned: quit the whole app, or switch to the other screen.
pub enum ScreenExit {
    Quit,
    Switch,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(run)?;
    Ok(())
}

/// Top-level screen driver. The game is the default screen; pressing `p` detours
/// into the particle sandbox (and `p` again returns), so the particle epic can be
/// exercised by hand without changing the entry point. Temporary testing affordance.
fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut in_game = true;
    loop {
        let exit = if in_game {
            app::app(terminal)?
        } else {
            sandbox::sandbox(terminal)?
        };
        match exit {
            ScreenExit::Quit => return Ok(()),
            ScreenExit::Switch => in_game = !in_game,
        }
    }
}
