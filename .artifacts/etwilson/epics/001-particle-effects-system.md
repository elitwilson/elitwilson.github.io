---
id: EPIC-001
title: Particle Effects System
status: ready
created: 2026-06-22
---

## Goal
Build a reusable particle effects system for the Ratatui castle game, exercised through a dedicated sandbox screen rather than the game itself. The system simulates particles with simple physics (gravity, fade, lifetime) and renders them over the terminal grid; "effects" are configured emitters that spawn particles into the system. The first concrete effect is a fireworks burst. The aim is a clean, extensible foundation — generic particle simulation plus a per-effect parameter seam — without over-engineering ahead of need.

---

## Scope In
- **Particle system** — owns live particles, advances them each `tick(dt)` (gravity, fade, lifetime expiry, dead-particle culling), and exposes a read-only surface for the renderer. Knows nothing about specific effects.
- **Particle model** — `f32` cell-space position and velocity, lifetime, and color/glyph. Physics is integrated against real `dt` from the existing `App::tick(dt)` seam.
- **Effects layer** — effect definitions that, when spawned, emit a configured burst of particles into the system. Dispatched through an `EffectKind` enum so the generic spawn seam is real from day one. Each effect owns its own parameters (per-effect params); fireworks ships with sensible hardcoded defaults.
- **Fireworks effect** — the first and only effect implemented: a burst of particles from an origin with randomized velocities, affected by gravity, fading over their lifetime.
- **Hand-rolled PRNG** — a tiny seedable generator (xorshift/LCG) for particle randomization. No external `rand` dependency. Seedable so effects are deterministically testable.
- **Cell-glyph rendering** — particles rendered as single foreground glyphs (with color) layered over the scene, as a projection of their `f32` positions onto the cell grid.
- **Sandbox screen** — a dedicated screen the binary auto-launches into (temporarily bypassing the game). It auto-spawns the fireworks effect on repeat at screen center so motion is always visible without input. Includes a cycle-effect control (functional even with a single effect, to keep the `EffectKind` dispatch honest) and `Esc`/`q` to quit.

## Scope Out
- Win → fireworks integration. The game's win path (`enter_tile`) is **not** wired to the effect system in this epic. The game entry point is parked (reversibly), not deleted; re-wiring fireworks into the real win is a follow-up epic.
- Additional effect types beyond fireworks (the `EffectKind` dispatch exists, but the list is length 1).
- Sub-cell rendering (half-block characters, Braille via Ratatui Canvas). The `f32` position model preserves this as a future projection swap; it is not built now.
- Config-file or runtime tuning of effect parameters. Per-effect params exist in code as the extension seam; surfacing them externally is deferred.
- Continuous/streaming emitters (the sandbox's repeat-spawn is a sandbox loop, not a system-level emitter abstraction).
- Gameplay-affecting particles (collision, interaction with map/entities). Particles are purely cosmetic.

---

## Key Decisions

- **Two-layer architecture.** A generic *particle system* (simulation + read surface) is separate from *effects* (configured emitters that inject particles). The system never references a specific effect; effects depend on the system. This keeps the simulation reusable and the per-effect parameters localized.
- **Coordinate space: `f32` cell-space.** Particles simulate in floating-point cell coordinates, independent of the tile grid (`TILE_W = 2`). Physics: constant downward acceleration (gravity) plus lifetime-driven fade. This decouples simulation from render resolution and is what preserves the half-block/Braille upgrade path.
- **Rendering: cell glyphs as a projection.** v1 draws one foreground glyph per occupied cell by rounding float positions to the grid. The renderer remains a pure observer (consistent with the existing `App` → `render.rs` boundary): only the system mutates particle state. Higher-resolution projections can replace this layer later without touching the simulation.
- **Generic dispatch from day one.** An `EffectKind` enum and a `spawn(kind, origin)` seam are built immediately, even though fireworks is the only variant. The sandbox's cycle control exercises this seam so it doesn't rot into a hardcoded single-effect path.
- **Per-effect parameters.** Each effect kind owns its parameter set (e.g. fireworks: particle count, color palette, spread, initial speed, gravity strength). For now these are hardcoded defaults; the structural seam to pass/override them per spawn exists from the start.
- **Randomness: hand-rolled, seedable PRNG.** A small xorshift/LCG generator, no `rand` dependency. Seedability is a hard requirement so particle behavior is deterministically testable.
- **Sandbox-first delivery.** The system is built and validated through a sandbox screen, not the game. The binary temporarily auto-launches the sandbox, bypassing the game. This detour is deliberate and reversible — the game entry point is parked, not removed, so restoring it is a one-line swap.
- **Driven by the existing tick seam.** The sandbox advances particles via the already-present `App::tick(dt: Duration)` hook (real elapsed time, ~60fps). No new timing infrastructure.
