---
number: 005
story: STR-003
status: ready
base_branch: main
depends_on: [STR-001]
scope_files:
  - src/particle_render.rs
  - src/particle_render/tests.rs
  - src/main.rs
---

# Feature: Particle rendering (glyph projection)

## Summary
This is the projection layer that turns the particle system's f32 cell-space positions into concrete cell-grid draws. For each live particle exposed by the `ParticleSystem` read surface, it rounds the f32 position to an integer cell, computes a faded foreground color from the particle's lifetime progress, and paints the particle's glyph into that cell of the Ratatui buffer — layered as a foreground symbol over whatever scene is already there. It is a pure, isolated, unit-testable seam: a future epic can replace it with half-block/Braille sub-cell rendering without touching the simulation. The visual payoff only appears once STR-004's sandbox calls this function, which is expected.

---

## Requirements
- A live particle at f32 position `(x, y)` projects to integer cell `(round(x), round(y))` — rounding is half-away-from-zero via `f32::round`, defined and tested.
- A particle is drawn as a single foreground glyph (its `char`) with a faded color at its projected cell, relative to a caller-supplied origin offset within the target `Rect`.
- An aged particle (lower fade progress / more lifetime elapsed) renders dimmer than a fresh one; the fade-to-color mapping is pure and tested independently of rendering.
- A particle whose projected cell falls outside the target `Rect` (or outside the buffer) is skipped — no panic, no out-of-bounds write.
- Multiple particles projecting to the same cell do not corrupt the buffer; last-write precedence in iteration order is acceptable.
- The draw entry point is a free function taking the system's read surface, a `&mut Buffer` (or `Frame`), the target `Rect`, and an origin offset — it does not couple to `App`.

---

## Scope

### In Scope
- Pure projection: f32 particle position → integer cell coordinate.
- Pure fade mapping: fade progress (0.0 = dead/elapsed, 1.0 = fresh) + base `Color` → dimmed `Color`.
- A `draw_particles` free function that iterates the read surface and paints foreground glyphs into the buffer via `cell_mut`, skipping off-area / off-buffer cells.
- Bounds discipline against both the target `Rect` and the buffer (`cell_mut` returns `None` off-buffer).
- Unit tests for the projection and fade math, plus tests for bounds-skipping behavior against a real `Buffer`.

### Out of Scope
- The `ParticleSystem`, particle model, and effects (STR-001 / STR-002).
- The sandbox screen, loop, and input handling (STR-004) — this story provides the draw function the sandbox calls; it owns no screen.
- Sub-cell rendering (half-block, Braille). This layer is exactly the seam a future epic would swap for those; not built now.
- Background / scene composition beyond painting particles.
- Wiring this into the existing castle `render::ui` — the sandbox (STR-004) is the only caller.

---

## Technical Approach
- **Entry point / interface:** A free function in a new module `src/particle_render.rs`:
  ```rust
  pub fn draw_particles(
      particles: &ParticleSystem,   // STR-001 read surface
      buf: &mut Buffer,
      area: Rect,
      origin: (u16, u16),           // top-left cell to anchor particle (0,0) at
  )
  ```
  Taking `&mut Buffer` (obtainable from `Frame::buffer_mut()`) rather than `&mut Frame` keeps it trivially testable with a standalone `Buffer` and matches how `fill_tile` already reaches `frame.buffer_mut()`. The sandbox passes its body `Rect` and the origin it wants particles anchored to.

- **Assumed STR-001 read surface (specced in parallel; does not exist yet):** This spec designs against the read surface STR-001's story describes. The assumed contract is:
  - `ParticleSystem` exposes an iterator over live particles, e.g. `fn particles(&self) -> impl Iterator<Item = &Particle>` (exact method name is STR-001's call; the implementer aligns to whatever STR-001 ships).
  - Each `Particle` exposes, by accessor or public field: f32 position `pos: (f32, f32)` (cell-space), appearance `color: Color` and `glyph: char`, and a fade value. Fade is taken as a `fade(&self) -> f32` in `[0.0, 1.0]` where `1.0` = fresh and `0.0` = fully aged (derived from remaining/total lifetime).
  - **If STR-001's actual surface differs** (e.g. it exposes `remaining`/`total` rather than a computed `fade()`, or fields vs. accessors), the implementer adapts the call sites only — the projection and fade math below are independent of that shape.

- **Key modules / responsibilities:**
  - `src/particle_render.rs` — owns three things: `project(pos: (f32, f32)) -> (i32, i32)` (pure rounding), `fade_color(base: Color, fade: f32) -> Color` (pure dimming), and `draw_particles(...)` (iterate, project, fade, bounds-check, paint).
  - `src/particle_render/tests.rs` — `#[cfg(test)] mod tests;` sibling per the project's Rust testing convention.

- **Projection:** `project` rounds each f32 coordinate with `f32::round` (half-away-from-zero) to the nearest cell, returning `(i32, i32)` so negative results survive for the bounds check rather than underflowing a `u16`. `draw_particles` then offsets by `origin` and validates against `area` before casting to `u16`.

- **Fade-to-color:** scale the base RGB toward black by the fade factor: `Color::Rgb((r as f32 * fade) as u8, …)`. Fresh (`fade = 1.0`) yields the base color; aged (`fade → 0`) approaches black. Non-`Rgb` colors (e.g. named) are returned unchanged or mapped through a documented fallback — particles in this project carry `Color::Rgb` appearances, so the `Rgb` path is the real one; handle the non-`Rgb` arm without panicking.

- **Bounds discipline:** for a projected, origin-offset cell `(cx, cy)`:
  1. Skip if `cx < 0 || cy < 0`.
  2. Skip if the cell is outside `area` (`cx < area.left() || cx >= area.right() || cy < area.top() || cy >= area.bottom()`).
  3. `buf.cell_mut((cx as u16, cy as u16))` returns `None` off-buffer — skip those too.
  Then `cell.set_symbol(&glyph.to_string()); cell.set_fg(faded);` — the foreground analogue of `fill_tile`'s `set_symbol(" ") + set_bg`.

- **Key design decisions:**
  - Free function + pure helpers, no `App` coupling — keeps the seam thin and swappable per the epic decision.
  - `&mut Buffer` over `&mut Frame` for testability.
  - `i32` intermediate for projected coords so out-of-bounds (including negative) is a clean comparison, never a `u16` underflow — mirrors the `saturating_sub` care in `map_origin`.

---

## Success Criteria
- [ ] `project` maps known f32 positions to the expected integer cells, including the rounding boundary (e.g. `2.5 → 3`, `2.4 → 2`) — tested.
- [ ] `fade_color` returns the base color at `fade = 1.0` and a strictly darker color at lower fade — tested on a known `Color::Rgb`.
- [ ] `draw_particles` paints a particle's glyph + faded fg color at its projected cell in a real `Buffer` — asserted via `buf.cell((x, y))`.
- [ ] A particle projecting outside `area` or off-buffer is skipped: no panic, and no cell outside the area is written — tested.
- [ ] Two particles on the same cell leave the buffer in a valid state (last-write), no panic — tested.
- [ ] `cargo test`, `cargo clippy`, and `cargo fmt --check` pass.

---

## Tasks
Ordered by dependency.

- [ ] **Module + projection math:** Create `src/particle_render.rs` and register it in `src/main.rs` (`mod particle_render;`). Implement `project((f32, f32)) -> (i32, i32)` using `f32::round`. Add `src/particle_render/tests.rs` with `#[cfg(test)] mod tests;` wired at the bottom of `particle_render.rs`. Fully test projection (including rounding boundaries) before moving on.
- [ ] **Fade-to-color mapping:** Implement `fade_color(Color, f32) -> Color` scaling `Color::Rgb` toward black by the fade factor, with a non-panicking fallback for non-`Rgb`. Test base-at-1.0 and dimmer-at-lower-fade.
- [ ] **`draw_particles` against the read surface:** Implement the iterate→project→offset→bounds-check→paint pipeline taking `&ParticleSystem`, `&mut Buffer`, `Rect`, and `origin`. Code against the assumed STR-001 surface (`particles()` iterator yielding particles with `pos`, `color`, `glyph`, `fade()`); leave a brief comment noting the assumed surface so alignment with STR-001 is obvious. Test paint-at-cell, out-of-bounds skip (negative and beyond-area), off-buffer skip, and same-cell overwrite using a standalone `Buffer`.
- [ ] **Lint + format pass:** Run `cargo clippy` and `cargo fmt`; resolve findings.

---

## Considerations
- **STR-001 is not built yet.** The read-surface contract above is an assumption stated explicitly. The pure helpers (`project`, `fade_color`) are independent of it and can be written and tested immediately; only `draw_particles`'s call sites depend on STR-001's final shape. If STR-001 ships a different surface (fields vs. accessors, `remaining`/`total` vs. `fade()`), adapt the call sites — not the math.
- **Cell-space vs. tile-space.** Particles live in f32 *cell* coordinates (epic decision), decoupled from the castle's `TILE_W = 2` tile width. This projection does **not** multiply by `TILE_W` — one particle maps to one terminal cell, not one 2-cell tile. The `origin` offset is in cell coordinates.
- **Foreground, not background.** Unlike `fill_tile` (which sets `bg` on a `" "`), particles set a foreground `symbol` + `fg`, layered over whatever scene already occupies the cell. The existing cell's `bg` is left untouched.
- **Negative projected coordinates** must be handled before any `u16` cast — using `i32` intermediates avoids the underflow that a naive `as u16` would cause.
- **Glyph as a single cell.** `set_symbol(&glyph.to_string())` writes one grapheme; multi-width glyphs are out of scope — particles use simple single-width chars.
- No new dependencies; everything is in `ratatui` already in use (`Buffer`, `Rect`, `Color`).
