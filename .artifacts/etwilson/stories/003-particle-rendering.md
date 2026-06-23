---
id: STR-003
title: Particle rendering (glyph projection)
epic: EPIC-001
status: specced
priority: high
---

## Goal

The projection layer that turns the particle system's `f32` positions into cell-grid draws: each live particle is rounded to a cell and drawn as a foreground glyph (with color, dimmed by fade) over the scene. This is the explicit, swappable seam that a future half-block/Braille upgrade replaces without touching the simulation.

---

## Scope

### In
- A pure projection from the `ParticleSystem` read surface to cell draws: for each live particle, round its `f32` position to integer cell `(x, y)` and produce the glyph + color to paint.
- Applying fade: a particle's color/intensity reflects its lifetime progress (older → dimmer), using the fade progress exposed by STR-001.
- Drawing those glyphs into the Ratatui buffer as **foreground** symbols layered over whatever is beneath (consistent with the existing cell-writing approach in `render.rs`).
- Handling off-area / out-of-bounds particles gracefully (skip cells outside the target `Rect`).

### Out
- The `ParticleSystem` and effects (STR-001 / STR-002).
- The sandbox screen, loop, and input handling (STR-004) — this story provides the draw function the sandbox calls; it does not own a screen.
- Sub-cell rendering (half-block, Braille). This layer is exactly the seam that would later be swapped for those; not built now.
- Background/scene composition beyond drawing particles (the sandbox decides what's underneath).

---

## Acceptance Criteria

- [ ] Given a set of particles at known `f32` positions, the projection maps each to the expected integer cell (rounding behavior is defined and tested).
- [ ] A particle is drawn as a single foreground glyph with its color at the projected cell.
- [ ] An aged particle renders dimmer than a fresh one (fade is visibly applied).
- [ ] Particles whose projected cell falls outside the target area are skipped without panicking or writing out of bounds.
- [ ] Multiple particles projecting to the same cell don't corrupt the buffer (last-write or defined precedence is acceptable; no panic).

---

## Context & Decisions

- **Rendering is a projection, not the simulation (epic decision).** Because particles live in `f32` cell-space, the renderer just rounds-and-draws. This is the swappable layer: a future epic can replace glyph projection with half-block (2× vertical) or Braille (2×4 via Ratatui Canvas) without changing STR-001/002. Keep this layer thin and isolated so that swap stays cheap.
- **Foreground glyphs, layered over the scene.** Unlike the existing `fill_tile` (`src/render.rs:56`), which sets a background color on `" "`, particles set a foreground *symbol* + color so they read as objects on top of the scene. Reuse the same `buffer_mut().cell_mut((x, y))` cell-writing mechanism; only the bounds-checked `cell_mut` is used (returns `None` off-buffer — skip those).
- **Pure and unit-testable.** The position→cell projection and fade→color mapping should be testable without a running terminal (assert on the produced draws, or on a test buffer). Visible payoff only lands when STR-004 runs the sandbox — that's expected and acceptable.
- **Bounds discipline.** The project already guards against out-of-bounds/underflow (`map_origin`'s `saturating_sub`, `walkable`'s coord checks). Match that care: never index a cell outside the target `Rect`.

---

## Dependencies

- **Depends on:** STR-001 (consumes the read surface)
- **Blocks:** STR-004 (sandbox calls this to draw particles). Can be built in parallel with STR-002.

---

## Notes

- Look at `src/render.rs` for the existing cell-writing pattern (`fill_tile`, `buffer_mut`, `cell_mut`) and match its style. `cell.set_symbol(&glyph.to_string())` + `cell.set_fg(color)` is the foreground analogue of the existing `set_bg`.
- Fade-to-color: options include scaling toward black, or using `Color` + a dimming step. The architect picks; the observable requirement is "older = dimmer."
- Decide the draw signature so the sandbox can pass the target `Rect` and an origin offset if needed — keep it a free function taking the system's read surface + a `&mut Buffer`/`Frame` + area, rather than coupling to `App`.
