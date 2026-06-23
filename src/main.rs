mod about;
mod app;
mod block_font;
mod effects;
mod map;
mod menu;
mod particle_render;
mod particles;
mod render;
mod rng;
mod sandbox;
mod sandbox_config;
mod theme;
mod victory;

use ratatui::DefaultTerminal;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Screen {
    Menu,
    Game,
    About,
    Sandbox,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Nav {
    To(Screen),
    Quit,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(run)?;
    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut current = Screen::Menu;
    loop {
        let nav = match current {
            Screen::Menu => menu::menu(terminal)?,
            Screen::Game => app::app(terminal)?,
            Screen::About => about::about(terminal)?,
            Screen::Sandbox => sandbox::sandbox(terminal)?,
        };
        match nav {
            Nav::Quit => return Ok(()),
            Nav::To(screen) => current = screen,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_quit_is_distinct_from_to() {
        let q = Nav::Quit;
        let t = Nav::To(Screen::Menu);
        assert_ne!(q, t);
    }

    #[test]
    fn screen_variants_are_distinct() {
        assert_ne!(Screen::Menu, Screen::Game);
        assert_ne!(Screen::Game, Screen::About);
        assert_ne!(Screen::About, Screen::Sandbox);
    }

    #[test]
    fn nav_to_carries_the_target_screen() {
        let nav = Nav::To(Screen::Game);
        assert_eq!(nav, Nav::To(Screen::Game));
    }
}
