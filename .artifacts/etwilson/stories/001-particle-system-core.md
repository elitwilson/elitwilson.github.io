---
id: STR-001
title: Particle system core
epic: EPIC-001
status: specced
priority: high
---

## Goal

A standalone `ParticleSystem` that simulates particles with simple physics and exposes a read-only surface for rendering. This is the reusable foundation the effects and rendering layers build on; it knows nothing about any specific effect.

---

## Scope

### In
- A particle model in `f32` cell-space: position `(x, y)`, velocity `(vx, vy)`, lifetime (remaining + total, for fade), and appearance (color + glyph).
- A `ParticleSystem` struct that owns a collection of live particles.
- `tick(dt: Duration)` that advances every particle: gravity (constant downward acceleration on `vy`), position integration against real `dt`, lifetime decrement, and culling of dead particles (lifetime expired).
- A way to inject particles into the system (e.g. `spawn_particle` / `emit`) — the entry the effects layer will call.
- A read-only surface for the renderer to iterate live particles (position + appearance + fade progress). Mirrors the existing `App`-as-sole-mutator pattern: only the system mutates particle state.

### Out
- Any notion of an "effect," `EffectKind`, or effect parameters (STR-002).
- Randomness / PRNG (STR-002 — physics here is deterministic).
- Rendering / projection to the cell grid (STR-003).
- The sandbox loop or entry-point changes (STR-004).
- Collision or interaction with map/entities.

---

## Acceptance Criteria

- [ ] A particle can be added to the system and appears in its read surface.
- [ ] After `tick(dt)`, a particle's position advances according to its velocity and `dt`.
- [ ] Gravity increases downward velocity over successive ticks (a particle with zero initial `vy` gains downward velocity).
- [ ] A particle whose lifetime elapses is culled and no longer appears in the read surface.
- [ ] Fade progress is derivable from the read surface (remaining vs. total lifetime) so the renderer can dim a particle as it ages.
- [ ] The renderer-facing surface is read-only; particle state is mutated only inside the system.

---

## Context & Decisions

- **`f32` cell-space (epic decision).** Particles simulate in floating-point *cell* coordinates, independent of the tile grid (`TILE_W = 2` in `render.rs`). This decouples the simulation from render resolution and preserves the future half-block/Braille upgrade path. Do not tie positions to tile coordinates.
- **Physics is gravity + fade only.** Constant downward acceleration plus lifetime-driven fade. No drag, wind, or collision — explicitly out of scope.
- **Deterministic core.** No randomness in the system itself; given the same particles and `dt`, `tick` is fully deterministic. Randomness lives in effect spawning (STR-002). This keeps the core trivially unit-testable.
- **Read-only surface, single mutator.** Follow the established `App` → `render.rs` boundary (`src/app.rs:121` "Read surface" comment): expose observation without letting callers mutate. The renderer (STR-003) consumes this; it never writes.
- **Standalone struct, not attached to `App` (epic decision (a)).** `ParticleSystem` is its own type so the sandbox can own one directly and the future game-integration epic can embed it in `App` unchanged.

---

## Dependencies

- **Depends on:** none
- **Blocks:** STR-002 (effects emit particles into this system), STR-003 (renderer reads this surface)

---

## Notes

- Mirror the existing module style: implementation in `src/particles.rs` (or similar) with tests in a `#[cfg(test)] mod tests;` sibling file per the project's Rust testing convention.
- `dt` comes from the existing poll loop (`src/app.rs:164-166`) as a `std::time::Duration`. Integrate physics against `dt.as_secs_f32()`.
- Glyph + color types: reuse Ratatui's `ratatui::style::Color`; glyph can be a `char`.
- Decide how a particle stores lifetime so fade is cheap to compute (e.g. `age` and `max_age`, or `remaining` and `total`) — either is fine; just make fade progress observable.
