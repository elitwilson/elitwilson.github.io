---
number: 003
story: STR-001
status: ready
base_branch: main
depends_on: []
scope_files:
  - src/particles.rs
  - src/main.rs
---

# Feature: Particle System Core

## Summary
A standalone `ParticleSystem` that simulates particles with simple physics (gravity, position integration, lifetime fade) in `f32` cell-space and exposes a read-only surface for a renderer to iterate live particles. It is the reusable foundation for the effects (STR-002) and rendering (STR-003) layers and knows nothing about any specific effect, randomness, or the terminal grid. It mirrors the project's established single-mutator pattern: only the system mutates particle state; outside code observes through read accessors.

---

## Requirements
- A particle is modeled in `f32` cell-space with position `(x, y)`, velocity `(vx, vy)`, lifetime (remaining + total), and appearance (a `ratatui::style::Color` and a `char` glyph).
- Particles can be injected into the system through a single entry point that the effects layer will call.
- An injected particle appears in the system's read surface.
- `tick(dt: Duration)` advances every live particle: applies constant downward gravity to `vy`, integrates position by velocity against `dt.as_secs_f32()`, decrements remaining lifetime by `dt`, and culls particles whose lifetime has expired.
- After a tick, a particle's position reflects its velocity integrated over `dt`.
- A particle that starts with zero `vy` gains downward velocity after successive ticks (gravity is observable).
- A particle whose remaining lifetime reaches zero (or below) is removed and no longer appears in the read surface.
- Fade progress (a normalized value derivable from remaining vs. total lifetime) is observable through the read surface so the renderer can dim an aging particle.
- The read surface is read-only: a consumer can observe position, appearance, and fade progress but cannot mutate particle state.
- Given the same starting particles and the same `dt` sequence, `tick` is fully deterministic.

---

## Scope

### In Scope
- The `Particle` model (f32 cell-space pos/vel, lifetime, color + glyph).
- The `ParticleSystem` struct owning a `Vec<Particle>` of live particles.
- `tick(dt)` physics: gravity, position integration, lifetime decrement, cull.
- A spawn/emit entry point for injecting particles.
- A read-only iteration surface exposing position, appearance, and fade progress.
- A gravity constant for the downward acceleration.
- Inline `#[cfg(test)] mod tests` covering the acceptance criteria.

### Out of Scope
- Any `EffectKind`, effect parameters, or effect dispatch (STR-002).
- Randomness / PRNG — physics here is deterministic (STR-002).
- Rendering / projection of f32 positions onto the cell grid (STR-003).
- The sandbox loop, screen, or entry-point wiring (STR-004).
- Collision, drag, wind, or interaction with the map/entities.

---

## Technical Approach
- **New module:** `src/particles.rs`, declared with `mod particles;` in `src/main.rs` alongside the existing `mod app; mod render; mod theme; mod map;`. Tests live inline at the bottom as `#[cfg(test)] mod tests { use super::*; ... }`, matching every existing module in this project (app.rs, render.rs, map.rs all use inline test modules — the project does not use sibling `tests.rs` files).
- **`Particle` struct:** public fields are acceptable here since a particle is a plain data record with no invariants to protect (unlike `App`); however, mutation must only happen inside `ParticleSystem::tick`. Shape:
  - `pos: (f32, f32)`, `vel: (f32, f32)`
  - `remaining: Duration`, `total: Duration` (storing remaining + total makes fade `1.0 - remaining/total` cheap and keeps `dt` decrement in native `Duration` units; this is the recommended split over age/max_age, but either satisfies the contract)
  - `color: ratatui::style::Color`, `glyph: char`
- **`ParticleSystem` struct:** owns `particles: Vec<Particle>`. Constructed via `ParticleSystem::new()` (empty). Reuse `ratatui::style::Color` as the existing palette type (see `src/theme.rs`).
- **Spawn entry point:** `fn spawn(&mut self, particle: Particle)` (or `emit`) pushes onto the vec. Keep it a single, simple injection point — the effects layer (STR-002) owns the logic of *what* to spawn.
- **`tick(dt: Duration)`:** convert once to `let secs = dt.as_secs_f32();`. For each particle: `vel.1 += GRAVITY * secs;` then `pos.0 += vel.0 * secs; pos.1 += vel.1 * secs;` then decrement remaining lifetime. Use `Duration::saturating_sub(dt)` for the lifetime decrement so it floors at zero without panicking. Cull with `Vec::retain` keeping particles whose `remaining` is greater than zero. Mirrors the dt seam in `src/app.rs:155` / `src/app.rs:164-166` and integrates against real elapsed time, not a fixed step.
- **Read surface:** follow the `App` read-surface pattern (`src/app.rs:121`). Expose an iterator over `&Particle` (e.g. `fn particles(&self) -> impl Iterator<Item = &Particle>` or `fn particles(&self) -> &[Particle]`) returning shared references so the renderer can read but not mutate. Expose fade progress as a method on `Particle`, e.g. `fn fade(&self) -> f32` returning `1.0 - remaining/total` clamped to `[0.0, 1.0]` (0.0 = fresh, 1.0 = fully faded) — pick a direction and document it so STR-003 dims consistently.
- **Gravity constant:** a module-level `const GRAVITY: f32` in cell-units-per-second-squared (downward = positive y, since screen y grows downward — consistent with the render module's coordinate sense). Value is a tuning choice, not load-bearing for this story; a modest positive value (e.g. on the order of tens) is fine and STR-002/effects can revisit.
- **Key design decisions:** `Duration`-based lifetime keeps `dt` arithmetic exact and unit-clean; the `Vec` + `retain` cull is the idiomatic O(n) approach with no ordering guarantees needed; standalone struct (not on `App`) per epic decision (a) — nothing here references `App`.

---

## Success Criteria
- [ ] `cargo build` and `cargo clippy` pass clean with the new module wired into `src/main.rs`.
- [ ] A spawned particle appears when iterating the read surface; an empty system yields none.
- [ ] After `tick(dt)` a particle with nonzero velocity has moved by `vel * dt.as_secs_f32()` (assert with an f32 epsilon tolerance).
- [ ] A particle starting at `vy == 0` has `vy > 0` (downward) after one or more ticks.
- [ ] A particle whose remaining lifetime is consumed by a tick is absent from the read surface afterward; one with lifetime left remains.
- [ ] `fade()` returns ~0.0 at spawn and approaches 1.0 as remaining lifetime drains (documented direction), clamped to `[0.0, 1.0]`.
- [ ] The read surface returns shared references only — no public method hands out `&mut` to a particle.
- [ ] `cargo test` passes; tests are deterministic across runs.

---

## Tasks
Ordered by dependency. TDD per the project workflow — scaffold, fail, implement.

- [ ] **Module scaffold + wiring:** Create `src/particles.rs` with the `Particle` and `ParticleSystem` struct definitions, `ParticleSystem::new`, the `GRAVITY` const, and an empty `#[cfg(test)] mod tests`. Add `mod particles;` to `src/main.rs`. Confirm it compiles (`cargo build`) before moving on.
- [ ] **Spawn + read surface (RED→GREEN):** Write failing tests that a spawned particle appears in the read surface and an empty system is empty. Implement `spawn`/`emit` and the read accessor (`particles()`), plus `Particle::fade()`. Verify read surface hands out only shared references.
- [ ] **Physics tick (RED→GREEN):** Write failing tests for position integration against `dt`, gravity accumulating downward `vy` over ticks, and fade progress derived from lifetime. Implement `tick(dt)` (gravity → integrate → lifetime decrement). Use f32 epsilon comparisons in assertions.
- [ ] **Lifetime cull (RED→GREEN):** Write failing tests that an expired particle is removed and a still-living one survives a tick. Implement the `retain`-based cull at the end of `tick`. Confirm full `cargo test` + `cargo clippy` are green.

---

## Considerations
- **Coordinate sense:** the render module treats screen y as growing downward (`sy = oy + ty`). Gravity is therefore *positive* on `vy`. Keep cell-space y consistent with this so STR-003's projection is a straightforward scale, not a flip.
- **f32 in tests:** never assert exact f32 equality for integrated positions/velocities — use an epsilon (e.g. `(a - b).abs() < 1e-4`). Choose round `dt` values (e.g. `Duration::from_secs_f32(0.5)`) and simple velocities to keep expected values clean.
- **Lifetime decrement underflow:** `Duration` panics on overflowing subtraction; use `saturating_sub` so a `dt` larger than the remaining lifetime floors at zero and the particle is culled rather than panicking.
- **Cull predicate boundary:** decide and test the exact-zero case — a particle whose remaining hits exactly `0` should be culled (treat `remaining == 0` as dead), so a particle can never render with a 100% fade artifact.
- **f32 cell-space is intentional** (epic decision): the sim resolution is decoupled from `TILE_W = 2` in `src/render.rs`. Do not introduce any tile/cell quantization here — that belongs to the renderer (STR-003) and preserves the future half-block/Braille upgrade path.
- **Public-field mutation:** if `Particle` fields are public for ergonomic construction by the effects layer, that is acceptable, but no method on `ParticleSystem` may return a `&mut Particle` — the single-mutator invariant lives at the system boundary, mutating only inside `tick`.
