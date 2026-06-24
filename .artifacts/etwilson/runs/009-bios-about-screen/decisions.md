# Decisions — 009-bios-about-screen

## Task 1: Palette + layout skeleton
- Spec says testing is verification via `cargo test` (existing) + `cargo run` visual — no new unit tests for rendering.
- The spec's Considerations section explicitly states: "no new unit tests are warranted — do not invent assertions over rendered output."
- Per spec, Tasks 1-4 are all implementation tasks; RED phase means writing failing tests only where there is testable logic. Since all tasks are pure rendering with no new business logic, tests are limited to confirming existing tests pass.
- Will confirm `cargo test` passes after each task as the regression gate.
