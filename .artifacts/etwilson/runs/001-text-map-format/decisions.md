# Decisions — 001-text-map-format

## Inline tests vs separate test files

The project-level rust-testing.md rule says to use separate test files (`src/map/tests.rs`),
but the spec explicitly says "Add new parser unit tests, added **inline** in map.rs's existing
`#[cfg(test)] mod tests` block (matching the codebase's current convention)." The spec takes
precedence since it describes the exact pattern to match (existing inline tests in map.rs).

## Trailing newline handling

Spec says to handle trailing `\n` deterministically. Decision: strip a trailing newline before
splitting into rows (using `trim_end_matches('\n')`). This means a file with or without a trailing
newline parses identically. Tested implicitly by verifying the castle parses to the expected height.

## Map::new width derivation

Spec notes: padding must happen *before* `Map::new` so the first row already equals max width
(since `Map::new` derives width from the first row). Parser pads all rows to `max_width` before
building the Map.

## ParseError shape

Spec says "Exact error variant shape is the implementer's call as long as the three failure modes
are distinguishable and tested." Using the suggested shape from the spec exactly:
- `UnknownChar(char)` 
- `MissingEntity(&'static str)` — "player" | "key" | "door"
- `DuplicateEntity(&'static str)`
