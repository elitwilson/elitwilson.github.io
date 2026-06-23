---
id: STR-004
title: Sandbox screen + entry swap
epic: EPIC-001
status: specced
priority: high
---

## Goal

A runnable sandbox screen that ties the particle system, effects, and rendering into something you can see immediately — the binary temporarily auto-launches into it, bypassing the game. This is how the whole epic gets exercised in isolation, without playing the game to trigger an effect.

---

## Scope

### In
- A standalone `sandbox(terminal)` function with its own poll loop, reusing the existing `dt`/tick pattern (`event::poll(FRAME_TIME)` → compute `dt` → advance → draw).
- It owns a `ParticleSystem` and a seeded PRNG, advances the system via `tick(dt)`, and draws particles via the STR-003 projection over a cleared/background scene.
- **Auto-spawn on repeat:** fireworks fire automatically at screen center on a recurring cadence so motion is always visible with no input.
- A **cycle-effect control** (e.g. Tab or number keys) that cycles the selected `EffectKind` — functional even though the list is length 1, to keep the dispatch seam honest.
- `Esc` / `q` to quit the sandbox.
- The **entry swap:** `main` (or the entry in `src/app.rs`) calls `sandbox(terminal)` instead of `app(terminal)`. The game entry point is **parked, not deleted** — restoring it is a one-line swap.

### Out
- The particle system, effects, PRNG, and rendering internals (STR-001/002/003 — this story composes them).
- Win → fireworks integration and any game-mode interaction (deferred to a future epic; the game code stays intact but unused).
- Manual click/cursor placement of effects (spawn is auto-at-center per the epic).
- Persisting sandbox state or any config surface.

---

## Acceptance Criteria

- [ ] Running the binary launches directly into the sandbox (the game does not run).
- [ ] Fireworks spawn automatically at screen center on repeat — the screen shows continuous bursting motion with no key input.
- [ ] Particles visibly rise/fan out, fall under gravity, fade, and disappear (end-to-end: system + effects + rendering working together).
- [ ] The cycle-effect control changes the selected effect kind (observable even with a single kind — e.g. a label/indicator updates or selection wraps).
- [ ] `Esc` and `q` exit the sandbox cleanly (terminal restored).
- [ ] Restoring the game is a one-line change at the entry point (the game code is intact and unreferenced-but-present, not removed).

---

## Context & Decisions

- **Standalone loop, not an `App` mode (epic decision (a)).** The sandbox owns its own state and loop; it does not attach the particle system to `App`. The entry swap (`sandbox` vs `app`) is the deliberate, fully-reversible "park the game" mechanism. Reuse the *pattern* of the existing loop (`src/app.rs:158-181`), not `App::tick` itself.
- **Auto-spawn at center on repeat (epic decision).** No manual spawn key — the sandbox re-fires fireworks on a fixed cadence at the center of the draw area so there's always something to watch. The exact interval is the architect's call (e.g. every N seconds, or when the previous burst dies); the observable requirement is continuous visible motion.
- **Cycle control keeps dispatch honest (epic decision).** Wiring a real `EffectKind` cycle now — even over a length-1 list — proves the generic seam end-to-end and makes adding a second effect trivial later.
- **Temporary detour, recorded as such (epic decision).** This auto-launch is deliberate and reversible. Do not delete or gut `app()` / the game state — leave it compiling and one swap away from being the entry point again.

---

## Dependencies

- **Depends on:** STR-002 (spawns effects), STR-003 (draws particles). Transitively STR-001.
- **Blocks:** none (top of the dependency tree for this epic)

---

## Notes

- Entry point: today `src/main.rs` runs the terminal and `app::app(terminal)` is the loop (`src/app.rs:158`). The swap should be as small and obvious as possible — ideally one call-site line — so reverting is trivial.
- Reuse `FRAME_TIME` (`src/app.rs:10`) and the `Instant`/`dt` computation pattern rather than inventing new timing.
- Screen center: compute from the draw `Rect` (mirror `map_origin`'s centering math in `render.rs` if useful). Particles are in cell-space, so center is roughly `(area.width/2, area.height/2)` as `f32`.
- Consider a minimal title/hint line (like the game's) showing the current effect + controls, so the cycle control is observable — but keep it lightweight.
- Terminal teardown is handled by the existing harness around the loop; ensure the sandbox returns cleanly on quit the way `app()` does (`Ok(())`).
