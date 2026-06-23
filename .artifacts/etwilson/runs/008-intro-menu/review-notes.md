# Review Notes — 008-intro-menu

## Block-font module (RED phase)

## Verdict: APPROVED

**Task:** Block-font module (RED phase)
**Spec:** .artifacts/etwilson/specs/008-intro-menu.md

**Scope issues:** none

**Coverage gaps:** none

All expected requirements are covered:
- `compose` returns exactly `HEIGHT` lines — covered (`compose_returns_exactly_height_lines`, `compose_single_char_returns_height_lines`)
- All lines have equal width — covered (`compose_all_lines_equal_width`)
- Unknown character does not panic and renders as blank space of fixed width — covered (`compose_unknown_char_does_not_panic`, `compose_unknown_char_width_matches_known_char_width`, `compose_unknown_char_rows_contain_only_spaces`)
- `draw_banner` paints shadow cells at `+1,+1` offset in shadow color — covered (`draw_banner_paints_shadow_cells_at_offset_plus_one`)
- `draw_banner` paints main cells in fg color — covered (`draw_banner_paints_main_cells_in_fg_color`)
- Main cells overdraw shadow on overlap — covered (`draw_banner_main_overdrawes_shadow_at_overlap`)
- `draw_banner` is bounds-safe on a small buffer (no panic) — covered (`draw_banner_bounds_checked_no_panic_on_small_buffer`)

## Screen router refactor

## Verdict: APPROVED

**Task:** Screen router refactor
**Spec:** .artifacts/etwilson/specs/008-intro-menu.md

**Scope issues:** none

**Coverage gaps:** none

The spec explicitly states that screen loops are IO-bound and not directly unit-testable — correctness for loop-level Nav translations rests on the pure seams plus the manual smoke run in the integration task. The three enum-shape tests are the appropriate RED coverage for this task's testable surface. Existing `map_key`/`map_sandbox_key` tests confirmed green (159 total). Project compiles with stubs in place. Loop-level Nav translation behaviors (game/sandbox Quit→Menu, Switch→Sandbox/Game) are properly deferred to Task 5 (integration smoke test).

## Menu screen (RED phase)

## Verdict: APPROVED

**Task:** Menu screen (RED phase)
**Spec:** .artifacts/etwilson/specs/008-intro-menu.md

**Scope issues:** none

**Coverage gaps:** none

All 15 expected tests present and confirmed failing via `todo!` panics:
- `map_menu_key`: Up, w, Down, s, Enter, Esc, q, unknown — 8 tests
- Wraparound `up()`/`down()`: advance, wrap-last→first, wrap-first→last, decrement — 4 tests
- `activate` routing: Play→Game, About→About, Quit→Quit — 3 tests

## About placeholder screen (RED phase)

## Verdict: APPROVED

**Task:** About placeholder screen (RED phase)
**Spec:** .artifacts/etwilson/specs/008-intro-menu.md

**Scope issues:** none

**Coverage gaps:** none

Both required cases covered — `Esc`→Back and `q`→Back. The third test (unknown key→Ignore) is a non-spec-required addition consistent with the pure-seam pattern; not a violation. All 3 failing via `todo!` panics. The `about()` loop is IO-bound and correctly not unit-tested here.
