# Review Notes — 001-text-map-format

## Parser + Level/ParseError (RED phase) — initial

## Verdict: FLAGGED

**Task:** Parser + Level/ParseError (RED phase) — initial
**Spec:** .artifacts/etwilson/specs/001-text-map-format.md

**Scope issues:** none

**Coverage gaps:**

1. `parse_wall_tile`, `parse_floor_tile`, `parse_outside_tile` — single-char inputs fail at parse due to missing entities, not tile logic. Flagged for rewrite.
2. `parse_outside_tile` — Outside tile had no valid coverage in the suite.
3. `entity_chars_place_floor_underneath` — two `let src` bindings in same scope; would not compile.

---

## Parser + Level/ParseError (RED phase) — revised

## Verdict: APPROVED

**Task:** Parser + Level/ParseError (RED phase) — revised
**Spec:** .artifacts/etwilson/specs/001-text-map-format.md

**Scope issues:** none

**Coverage gaps:** none

All three flagged issues are resolved:
- `parse_wall_tile`, `parse_floor_tile`, `parse_outside_tile` now use complete maps with all three required entities.
- `parse_outside_tile` uses `" @kD"` — space at (0,0) is exercised in a valid parse.
- `entity_chars_place_floor_underneath` dead binding removed.

All spec requirements are covered: three tile chars, three entity chars (position recording + Floor underneath), ragged-row padding, unknown char error, all three missing-entity errors, all three duplicate-entity errors, trailing-newline handling.

---

## Author assets/castle.map (RED phase)

## Verdict: APPROVED

**Task:** Author assets/castle.map (RED phase)
**Spec:** .artifacts/etwilson/specs/001-text-map-format.md

**Scope issues:** none

**Coverage gaps:** none

The single test `castle_map_parses_to_expected_state` satisfies the spec's requirement for a guard test. The asserted values were verified against the `castle()` grid in `src/map.rs`:
- width 21 and height 11 match the grid dimensions exactly.
- Player (10, 8): row 8 col 10 is Floor in the source grid.
- Key (3, 4): row 4 col 3 is Floor in the source grid.
- Door (10, 9): row 9 col 10 is Floor in the source grid.

The test will correctly fail RED (compile error on missing `assets/castle.map`) until the file is created.

---

## Rewire App::new and remove castle() (RED phase)

## Verdict: APPROVED

**Task:** Rewire App::new and remove castle() (RED phase)
**Spec:** .artifacts/etwilson/specs/001-text-map-format.md

**Scope issues:** none

**Coverage gaps:** none

The spec explicitly states the guard for this task is the existing `app_initial_state` test passing unchanged. No new tests are required. The two existing tests together (`app_initial_state` asserting player (10,8)/no key/door closed + `castle_map_parses_to_expected_state` asserting entity positions and dimensions) fully cover the observable requirements of this task. Proceed to GREEN.
