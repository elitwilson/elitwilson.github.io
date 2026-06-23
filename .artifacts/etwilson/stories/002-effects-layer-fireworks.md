---
id: STR-002
title: Effects layer + fireworks (+ PRNG)
epic: EPIC-001
status: specced
priority: high
---

## Goal

A generic effects layer that turns a high-level "spawn this effect here" request into a configured burst of particles injected into the `ParticleSystem`, plus the first concrete effect (fireworks) and the hand-rolled PRNG it uses. This is where per-effect parameters live and where the system stays decoupled from any specific effect.

---

## Scope

### In
- An `EffectKind` enum (one variant to start: `Fireworks`) and a `spawn(kind, origin, …)` dispatch seam that routes to the right effect's emit logic.
- Per-effect parameter structs. Fireworks owns its own params (e.g. particle count, color palette, angular spread, initial speed range, gravity strength, lifetime range) with sensible hardcoded defaults.
- The fireworks effect: given an origin and its params, emit N particles into the system with randomized velocities (radial-ish burst), seeded colors/glyph, and randomized lifetimes.
- A small hand-rolled, **seedable** PRNG (xorshift or LCG) used for the randomization. No external `rand` dependency.

### Out
- The `ParticleSystem` itself and its physics (STR-001 — this layer only *emits into* it).
- Rendering (STR-003).
- The sandbox loop, repeat-spawn cadence, and the cycle-effect UI (STR-004).
- Additional effect types beyond fireworks (the enum exists; the list is length 1).
- External/config-file tuning of params (params are in-code only).

---

## Acceptance Criteria

- [ ] `spawn(EffectKind::Fireworks, origin)` injects a burst of multiple particles into the `ParticleSystem` at/around the origin.
- [ ] Emitted particles have varied velocities (not all identical) — i.e. randomization is actually applied.
- [ ] The PRNG is deterministic for a given seed: the same seed produces the same sequence, so a seeded fireworks spawn produces a reproducible burst (verifiable in tests).
- [ ] Fireworks parameters are expressed as a struct with defaults; changing a param (e.g. particle count) changes the emitted burst accordingly.
- [ ] Dispatch goes through `EffectKind` — adding a future variant is a matter of adding an enum arm + an emit function, with no change to the spawn call site's shape.

---

## Context & Decisions

- **Generic dispatch from day one (epic decision).** Build the `EffectKind` enum and `spawn(kind, …)` seam now even though fireworks is the only effect, so it never rots into a hardcoded single-effect path. STR-004's cycle control exercises this seam.
- **Per-effect parameters are the extension point (epic decision).** Each effect kind owns its parameter set. For now defaults are hardcoded in code; the structural seam to pass/override params per spawn must exist so the future "configurable fireworks" work is purely additive.
- **Hand-rolled, seedable PRNG (epic decision).** No `rand` crate — keeps deps minimal (`color-eyre`, `crossterm`, `ratatui` only). Seedability is a hard requirement: it's what makes effects deterministically testable. A ~10-line xorshift/LCG is sufficient.
- **Folded PRNG placement (story-scoping decision).** Randomness only enters at effect-spawn time; STR-001's physics is deterministic. So the PRNG lives here with its only consumer, not as a standalone story.
- **Effects depend on the system, never the reverse.** This layer calls into `ParticleSystem`'s emit entry; the system has no knowledge of effects. Preserves the two-layer architecture.

---

## Dependencies

- **Depends on:** STR-001 (emits particles into the `ParticleSystem`)
- **Blocks:** STR-004 (sandbox spawns effects via this layer)

---

## Notes

- Keep the PRNG tiny and self-contained (e.g. a `Rng` struct with `next_u32`/`next_f32`/range helpers). A single seedable instance owned by the caller (sandbox) and threaded into spawn is cleaner than global state.
- "Randomized velocities (radial-ish burst)": a fireworks burst typically picks a random angle + random speed per particle so they fan outward, then gravity pulls them down. Exact distribution is the architect's call; the observable requirement is *varied* outward motion.
- Follow the project's Rust testing convention: implementation + `#[cfg(test)] mod tests;` sibling file.
- Color palette can reuse `ratatui::style::Color`; consider a small fixed palette of bright colors for the burst.
