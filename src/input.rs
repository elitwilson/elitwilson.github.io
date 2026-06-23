//! Target-agnostic key code.
//!
//! The native build reads keys through crossterm; the web build reads them
//! through ratzilla. Both expose a `KeyCode` enum whose variants we match on
//! (`Char`, the arrows, `Enter`, `Esc`, `Tab`) share identical shapes, so a
//! re-export alias is all the abstraction the input layer needs — every pure
//! `map_*` function and its tests compile unchanged against whichever backend
//! the current target pulls in.
#[cfg(not(target_arch = "wasm32"))]
pub use crossterm::event::KeyCode;

#[cfg(target_arch = "wasm32")]
pub use ratzilla::event::KeyCode;
