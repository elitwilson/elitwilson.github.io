---
number: 009
story: STR-005
status: complete
base_branch: main
depends_on: []
scope_files:
  - src/about.rs
---

# Feature: BIOS About Screen Layout

## Summary
Replace the "About — coming soon" placeholder rendered by `About::render` in `src/about.rs` with a full BIOS-aesthetic terminal layout matching the EPIC-002 mockup. The screen presents a vertical stack of sections — a BIOS header bar, a POST boot log with a blinking cursor, an ELI WILSON title block, a bordered PROFILE.TXT panel, a two-column SKILLS.SYS + CAREER.LOG row, and a footer hint row — all in a fixed BIOS color palette. A visitor reaching the About screen (from the menu or via game completion) sees a stylized terminal "boot" page. All content is hardcoded placeholder data; only the render path changes. Key handling, navigation, and existing tests are untouched.

---

## Requirements
- `About::render` draws a single-screen BIOS layout that fits within an 80×24 terminal with no scrolling.
- A BIOS header bar spans the top row: `WILSON-BIOS (C) 1981  v2.6.0` left-aligned and `SYS: ATARI 2600 / RATATUI WASM` right-aligned, in the secondary-blue color.
- A POST boot log shows 4–5 dim-green lines, the final line `> READY.` in bright green, followed by a cursor block that blinks via `Modifier::SLOW_BLINK`.
- A title block renders `ELI WILSON` prominently in bright green, with the subtitle `// SOFTWARE DEVELOPER  ·  PLAYER 1` in secondary blue.
- A PROFILE.TXT panel has a bright-green border and an inverted header bar (green background, dark text) reading `PROFILE.TXT`, with placeholder bio text in panel-text color inside.
- SKILLS.SYS and CAREER.LOG render as two equal-width side-by-side bordered panels, each with the same inverted-header style.
- Each skill entry shows a `█`/`░` bar, a skill name, and an `n/10` label using hardcoded placeholder values.
- Each career entry shows a hardcoded placeholder span, role, and org.
- A footer hint row reads `↑/↓  w/s  scroll  ·  Esc  back to menu  ·  q  quit` in secondary blue.
- The whole screen uses the indigo background; all colors come from `Color::Rgb`.
- `Esc` and `q` still navigate back to the menu (no regression in `map_about_key` / `handle_key`).
- All existing tests in `src/about.rs` continue to pass unchanged.

---

## Scope

### In Scope
- Full replacement of the `About::render` body in `src/about.rs`.
- BIOS header bar, POST boot log w/ blinking cursor, title block, PROFILE.TXT panel, two-column SKILLS.SYS + CAREER.LOG row, footer hint row.
- BIOS color palette via `Color::Rgb` constants.
- Inverted panel header bars rendered as separate `Paragraph`s over each panel's top inner row.
- Hardcoded placeholder data for bio, skills, and jobs.
- Private render helpers within `src/about.rs` as needed to keep `render` readable.

### Out of Scope
- Scrolling of any kind.
- Real bio text or real job history.
- Scanline / flicker / CRT effects.
- Title font / bitmap-font changes.
- Any change to `map_about_key`, `AboutCommand`, `handle_key`, the `About` struct shape, or existing tests.
- `render_modal` in `src/render.rs` (the in-game About overlay — separate, untouched).

---

## Technical Approach
- **Entry point:** `About::render(&self, frame: &mut Frame) -> ratatui::layout::Rect`. Signature is unchanged — `router.rs:62` calls `s.render(frame)` and expects the screen's `Rect` returned. Continue returning `frame.area()`.
- **Outer layout:** Paint the indigo background over `frame.area()` first (render a full-area `Block`/`Paragraph` with the indigo bg style, or set bg on each section's style), then split `frame.area()` vertically with `Layout::vertical([...])` (or `Layout::default().direction(Vertical).constraints([...])`, matching the `src/menu.rs:132` idiom) into fixed-height rows:
  1. `Length(1)` — BIOS header bar
  2. `Length(1)` — gap
  3. `Length(~5)` — POST boot log (with trailing blinking cursor)
  4. `Length(1)` — gap
  5. `Length(2)` — ELI WILSON title + subtitle
  6. `Length(1)` — gap
  7. `Length(~5)` — PROFILE.TXT panel (bordered)
  8. `Length(1)` — gap
  9. `Min(0)` / `Length(~7)` — two-column SKILLS.SYS + CAREER.LOG row (bordered)
  10. `Length(1)` — footer hint row
  Exact lengths are tuned to fit 24 rows; trim/abbreviate placeholder content before exceeding the screen (per the no-scroll decision).
- **Header bar:** A single `Paragraph` (or two — left `Alignment::Left`, right `Alignment::Right` over the same row, or a horizontal split) styled secondary blue.
- **POST boot log:** A `Paragraph` of `Line`s; dim-green body lines, bright-green `> READY.`, and a trailing cursor `Span` styled with `Modifier::SLOW_BLINK` (e.g. a `█` or inverted space). No tick state — the renderer drives the blink.
- **Inverted panel headers:** For each bordered panel, render `Block::bordered()` with bright-green border into the panel area, then compute its `.inner(area)` rect, take the top row of that inner rect, and render a `Paragraph` with the title text styled `bg = green, fg = dark` (e.g. indigo or near-black) to produce the solid inverted header bar. Body content renders into the inner rect below the header row.
- **Two-column row:** Split the row rect with `Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])`; render SKILLS.SYS into the left, CAREER.LOG into the right, each via the bordered-panel + inverted-header helper.
- **Skill bars:** Build each bar string from `█` × filled and `░` × empty for `n/10`, append name + `n/10` label. Hardcoded entries.
- **Career entries:** Hardcoded `Line`s of span / role / org placeholder strings.
- **Color palette:** Define `Color::Rgb` consts at module top: indigo bg `(30, 27, 75)`, bright green `(61, 224, 53)`, dim green `(42, 168, 74)`, secondary blue `(127, 143, 217)`, panel text `(191, 233, 189)`. Use one dark fg (indigo bg or near-black) for inverted-header text.
- **Key design decisions:**
  - No state added — `About` stays a unit struct; blink is handled by `Modifier::SLOW_BLINK`, not a tick counter.
  - Inverted headers are separate `Paragraph`s over the inner top row because `Block::title()` cannot invert background.
  - Placeholder data is inlined in `render` (or small private `fn`s) — no content abstraction / data model yet.

---

## Success Criteria
- [ ] `cargo run` shows the About screen with all sections visible at 80×24.
- [ ] BIOS header bar appears at the top in secondary-blue with the left/right text from the design.
- [ ] POST boot log shows 4–5 lines ending in `> READY.` with a visibly blinking cursor block.
- [ ] `ELI WILSON` renders in bright green; subtitle in secondary blue.
- [ ] PROFILE.TXT panel has a bright-green border and an inverted (green bg, dark text) header bar.
- [ ] SKILLS.SYS and CAREER.LOG render side-by-side with matching inverted headers.
- [ ] Skill bars use `█`/`░` with a name and `n/10` label.
- [ ] Footer hint row matches `↑/↓  w/s  scroll  ·  Esc  back to menu  ·  q  quit`.
- [ ] `Esc` and `q` still return to the menu.
- [ ] `cargo test` passes (existing `map_about_key` tests unchanged); `cargo clippy` is clean.

---

## Tasks
Ordered by dependency.

- [ ] **Palette + layout skeleton:** In `src/about.rs`, add the `Color::Rgb` palette consts and rewrite `About::render` to paint the indigo background and split `frame.area()` into the fixed vertical row layout. Render placeholder text into each row so the overall structure is visible end-to-end. Keep the returned `Rect` as `frame.area()`.
- [ ] **Header, boot log, title, footer:** Fill in the BIOS header bar (left/right text, secondary blue), the POST boot log (dim-green lines + bright-green `> READY.` + `SLOW_BLINK` cursor), the ELI WILSON title + subtitle, and the footer hint row.
- [ ] **Bordered-panel + inverted-header helper:** Add a private helper that, given an area and a title, renders a bright-green `Block::bordered()` and an inverted (green bg / dark fg) header `Paragraph` over the top inner row, returning the remaining inner body rect. Use it for the PROFILE.TXT panel with placeholder bio text.
- [ ] **Two-column SKILLS.SYS + CAREER.LOG:** Split the two-column row 50/50 horizontally, render both panels via the helper, with `█`/`░` skill bars + `n/10` labels and placeholder career entries. Verify the full layout fits 80×24 via `cargo run` and trim lengths if it overflows.

---

## Considerations
- **Testing:** This is a pure rendering change with no new business logic, so no new unit tests are warranted — do not invent assertions over rendered output. The only pure logic in the file is `map_about_key`, already covered by the three existing tests, which must keep passing. There is no `tests/` integration dir and no existing `TestBackend` usage in this project; a `TestBackend` smoke test would assert structure that the manual `cargo run` check already covers more meaningfully, so it is not included. Verification is: `cargo test` (regression on existing tests) + `cargo run` visual inspection at 80×24.
- **Signature stability:** `router.rs:62` and the `ScreenState::About` arm depend on `render(&self, &mut Frame) -> Rect`. Do not change the signature or the returned `Rect` semantics (return `frame.area()`).
- **`render` borrow flow:** Compute layout rects from `frame.area()` before borrowing `frame.buffer_mut()`; if any direct buffer writes are used (e.g. for the inverted header), follow the `src/render.rs` pattern of finishing layout math first.
- **Background painting:** Ratatui does not clear to a custom bg by default — explicitly paint the indigo background over the full area first, and set bg on panel/inner styles so borders and inverted headers read correctly against it.
- **Fit budget:** The summed fixed `Length`s plus borders must stay ≤ 24 rows. The two-column panels and boot log are the easiest to abbreviate; trim placeholder lines rather than adding scroll.
- **Unicode width:** `█`/`░` and `·`/`↑`/`↓` are used in the design strings — they are single-width and render fine in the terminal/WASM target already used elsewhere (the game uses block fills), so no width special-casing is needed.
