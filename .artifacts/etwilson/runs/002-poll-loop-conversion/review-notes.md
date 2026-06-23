# Review Notes — 002-poll-loop-conversion

## Add the `tick` seam

## Verdict: APPROVED

**Task:** Add the `tick` seam
**Spec:** .artifacts/etwilson/specs/002-poll-loop-conversion.md

**Scope issues:** none

**Coverage gaps:** none

The proposed approach exactly matches the spec's Technical Approach section:
- `use std::time::{Duration, Instant};` added to imports
- `const FRAME_TIME: Duration = Duration::from_millis(16);` at module level
- `pub fn tick(&mut self, _dt: Duration) {}` on `impl App` with the specified doc comment
- No new fields added to `App`
- No tests written — correct per the spec's explicit "Testing approach" directive

Current `src/app.rs` has no existing `Duration`/`Instant` imports and no `tick` method, so all three additions are net-new with no conflicts. The existing test suite (lines 168–331) exercises `map_key`, `update`, movement, key/door logic — none of which are affected by adding a method and two declarations.

---

## Convert the loop in `app()`

## Verdict: APPROVED

**Task:** Convert the loop in `app()`
**Spec:** .artifacts/etwilson/specs/002-poll-loop-conversion.md

**Scope issues:** none

**Coverage gaps:** none

The proposed loop shape satisfies every spec requirement:
- `event::poll(FRAME_TIME)` replaces blocking `event::read()` — correct.
- `last` initialized to `Instant::now()` before the loop; `dt` computed each iteration and passed to `app.tick(dt)` — correct.
- `continue` guard eliminated; replaced with positive `key.kind == KeyEventKind::Press` condition so non-Press events fall through to `app.tick(dt)` — the spec's load-bearing detail is handled correctly.
- `app.tick(dt)` called unconditionally at the bottom of every iteration — correct.
- Loop ordering (draw → dt → poll/read → tick) matches the spec's Technical Approach exactly.
- No new `App` fields. Only `src/app.rs` modified.
- No unit tests — correct per the spec's explicit testing directive.

`return Ok(())` vs `break Ok(())` for Quit is semantically equivalent; both satisfy the spec.
