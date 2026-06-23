//! Web (WASM) runner.
//!
//! Drives the same [`Router`](crate::router::Router) as the native loop, but
//! through ratzilla's callback model: input arrives via `on_key_event` /
//! `on_mouse_event`, and `draw_web` invokes the render closure every animation
//! frame (driven by `requestAnimationFrame`). The frame hook supplies no delta
//! time, so we derive it from `performance.now()`.
//!
//! There's no process to exit on the web, so `Nav::Quit` is remapped to the menu.

use crate::router::Router;
use crate::{Nav, Screen};
use ratzilla::ratatui::Terminal;
use ratzilla::{DomBackend, WebRenderer};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub fn run() -> std::io::Result<()> {
    // Route Rust panics to the browser console with a readable stack trace.
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let backend = DomBackend::new()?;
    let mut terminal = Terminal::new(backend)?;

    // Shared across the key, mouse, and draw callbacks. Single-threaded in the
    // browser, and the callbacks never run reentrantly, so the borrows never
    // overlap.
    let router = Rc::new(RefCell::new(Router::new()));

    terminal.on_key_event({
        let router = router.clone();
        move |key_event| {
            let mut router = router.borrow_mut();
            if let Some(nav) = router.handle_key(key_event.code) {
                match nav {
                    // No process to quit on the web — fall back to the menu.
                    Nav::Quit => router.goto(Screen::Menu),
                    Nav::To(screen) => router.goto(screen),
                }
            }
        }
    })?;

    terminal.on_mouse_event({
        let router = router.clone();
        move |mouse_event| {
            router
                .borrow_mut()
                .set_mouse((mouse_event.col, mouse_event.row));
        }
    })?;

    let mut last = now_ms();
    terminal.draw_web(move |frame| {
        let now = now_ms();
        // Clamp against clock anomalies so a bad sample can't rewind animation.
        let dt = Duration::from_secs_f64((now - last).max(0.0) / 1000.0);
        last = now;

        let mut router = router.borrow_mut();
        router.render(frame);
        router.tick(dt);
    });

    Ok(())
}

/// Milliseconds from a monotonic high-resolution clock.
fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}
