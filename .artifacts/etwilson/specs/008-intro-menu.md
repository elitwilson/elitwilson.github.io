---
number: 008
story: null   # ad-hoc spec — intro menu, not decomposed from a story
status: ready
base_branch: main
depends_on: []
scope_files:
  - src/main.rs
  - src/block_font.rs
  - src/menu.rs
  - src/about.rs
  - src/app.rs
  - src/sandbox.rs
---

# Feature: Intro Menu

## Summary
The app currently boots straight into the castle game. This feature adds a front door: a traditional video-game-style opening menu that the binary launches into. The menu shows a large **CHARLO** title banner (rendered in an old-school block font with a diagonal drop shadow) above a vertical list of options — **Play**, **About**, **Quit**. The player moves a highlighted selection with the arrow keys (or w/s) and confirms with Enter. **Play** enters the castle game, **About** opens a placeholder personal-site screen, and **Quit** exits. From the game or the About screen, `Esc`/`q` now returns to the menu rather than quitting the whole app — the menu becomes the app's home base. The particle sandbox remains reachable via the existing `p` testing detour but is intentionally absent from the menu.

---

## Requirements
- On launch, the app displays the intro menu, not the game.
- The menu renders the title **CHARLO** as a multi-line block-font banner with a diagonal (`+1,+1`) drop shadow: bright-green glyphs over dark-green shadow.
- The menu lists three options in order: **Play**, **About**, **Quit**, with the current selection visibly highlighted.
- Up / `w` moves the selection up; Down / `s` moves it down; selection wraps around at both ends.
- Enter activates the selected option: **Play** → castle game, **About** → About screen, **Quit** → exit the app.
- `Esc`/`q` on the menu exits the app (equivalent to selecting Quit).
- A footer hint line shows the controls (e.g. `↑/↓ select · Enter · q quit`).
- From the **game**, `Esc`/`q` returns to the menu (no longer quits the process); `p` still detours to the sandbox.
- From the **About** screen, `Esc`/`q` returns to the menu. The About screen shows placeholder content plus a back hint — real personal-site content is out of scope.
- From the **sandbox**, `Esc`/`q` returns to the menu; `p` returns to the game (preserving the existing game↔sandbox toggle).
- The block-font renderer degrades gracefully: a character with no defined glyph renders as blank space of fixed width rather than panicking.

---

## Scope

### In Scope
- **Block-font module** (`src/block_font.rs`): a `char → 6-row art` glyph table covering the characters needed for the title (`C H A R L O` and space), a `compose` function that lays glyphs side-by-side into banner lines, and a buffer-drawing helper that paints the banner with a color + diagonal drop shadow. Color-agnostic — the caller supplies foreground and shadow colors.
- **Menu screen** (`src/menu.rs`): menu state (selected index), pure key→command mapping, selection movement with wraparound, the render (title banner + highlighted item list + footer hint), and the `menu()` screen loop.
- **About placeholder screen** (`src/about.rs`): a minimal centered placeholder with a back hint and its `about()` screen loop.
- **Screen router refactor** (`src/main.rs`): replace the two-state `ScreenExit`/`in_game` toggle with a small screen state machine that boots into the menu and routes between Menu / Game / About / Sandbox.
- **Game and sandbox exit remap** (`src/app.rs`, `src/sandbox.rs`): change their loop's exit translation so `Esc`/`q` returns to the menu instead of quitting; preserve the `p` toggle semantics.

### Out of Scope
- Real "about me" / personal-site content (this spec ships only a placeholder About screen).
- Win → about/victory content changes; the game's existing win/about modal is untouched.
- New particle effects or any change to the particle/effects/victory systems.
- Mixed-case or additional glyphs beyond the title characters (the table holds only what the title needs; unknown chars fall back to blank).
- Surfacing the sandbox on the menu (kept as a `p`-only dev affordance).

---

## Technical Approach

- **Entry point:** `main::run()` currently flips a `bool` between `app::app` and `sandbox::sandbox`, each returning `ScreenExit { Quit, Switch }`. Replace this with:
  ```rust
  pub enum Screen { Menu, Game, About, Sandbox }
  pub enum Nav { To(Screen), Quit }
  ```
  `run()` holds a `current: Screen` (starting `Screen::Menu`), dispatches to the matching screen loop, and matches the returned `Nav` — `Nav::Quit` exits, `Nav::To(s)` sets `current = s`. Each screen loop returns `io::Result<Nav>`.

- **Screen exit mapping (per screen):**
  - `menu()`: Play → `To(Game)`, About → `To(About)`, Quit / `Esc` / `q` → `Quit`.
  - `app::app()`: `Esc`/`q` → `To(Menu)` (was `Quit`); `p` → `To(Sandbox)`. Keep the pure `map_key` returning the existing `Command` enum — only the loop's translation of `Command::Quit`/`Command::Switch` into `Nav` changes, so existing `map_key` tests stay valid.
  - `sandbox::sandbox()`: `Esc`/`q` → `To(Menu)` (was `Quit`); `p` → `To(Game)`. Same approach — `map_sandbox_key` is unchanged; only the loop's translation of `SandboxCommand::Quit`/`Switch` into `Nav` changes.
  - `about()`: `Esc`/`q` → `To(Menu)`.

- **Block font (`block_font.rs`):**
  - `const HEIGHT: usize = 6;`
  - `fn glyph(c: char) -> [&'static str; HEIGHT]` via a `match`, returning a fixed-width blank glyph for unknown characters. Glyph art uses `█` for filled cells and spaces for empty (the same art validated during drafting).
  - `fn compose(text: &str, gap: usize) -> Vec<String>` — produces `HEIGHT` lines, each the row-wise concatenation of glyph rows separated by `gap` spaces between letters.
  - `fn draw_banner(buf: &mut Buffer, top_left: (u16, u16), lines: &[String], fg: Color, shadow: Color)` — two-pass paint: first the shadow layer (every non-space cell at offset `+1,+1` painted `█` in `shadow`), then the main layer (every non-space cell at `top_left` painted `█` in `fg`) on top. Bounds-checked via `buf.cell_mut`, consistent with `particle_render::draw_particles`.

- **Menu (`menu.rs`):**
  - `enum MenuItem { Play, About, Quit }` with an ordered `ITEMS` slice and a `label()`.
  - `struct Menu { selected: usize }` with `up()`/`down()` doing wraparound over `ITEMS.len()`.
  - `enum MenuCommand { Up, Down, Select, Quit, Ignore }` from a pure `map_menu_key(KeyCode)` (Up: `Up`/`w`; Down: `Down`/`s`; Select: `Enter`; Quit: `Esc`/`q`).
  - A pure `fn activate(item: MenuItem) -> Nav` mapping the selected item to a `Nav` (unit-testable without a terminal).
  - Rendering: vertical layout — a title band sized for the banner (`HEIGHT + 1` shadow row + padding), the centered item list with the selected row highlighted (bright/ reversed), and a centered footer hint. Title colors: `fg = Color::Rgb(0, 255, 0)` (matches `Theme::player`), `shadow = Color::Rgb(0, 80, 0)`, declared as named consts in `menu.rs`.

- **About (`about.rs`):** a centered placeholder paragraph (e.g. "About — coming soon") with an `Esc: back to menu` hint; pure `map_about_key` (`Esc`/`q` → back) and the `about()` loop returning `Nav`.

- **Key design decisions:**
  - **Menu is home.** The navigation model is a hub-and-spoke: every sub-screen's `Esc`/`q` returns to the menu; the process only exits from the menu. This is the change in meaning behind the existing `Quit` commands, localized to each loop's `Nav` translation.
  - **Pure seams preserved.** Following the existing pattern (`map_key`, `map_sandbox_key`), every screen exposes pure key-mapping and item-resolution functions so behavior is unit-testable; the IO loops stay thin.
  - **Font module is color-agnostic and minimal.** It renders any string from a known glyph set with a caller-chosen shadow; it is not a figlet engine and holds only the glyphs the title uses, with a blank fallback so adding the future About/Victory big text is a glyph-table addition, not a rewrite.

---

## Success Criteria
- [ ] Launching the binary shows the intro menu with the CHARLO drop-shadow banner and the Play / About / Quit list — not the game.
- [ ] Up/Down (and w/s) move the highlight; the selection wraps at both ends.
- [ ] Enter on **Play** enters the castle game; Enter on **About** opens the About screen; Enter on **Quit** (or `Esc`/`q` on the menu) exits the process.
- [ ] `Esc`/`q` from the game returns to the menu (process keeps running); `p` from the game still opens the sandbox.
- [ ] `Esc`/`q` from the About screen returns to the menu.
- [ ] `Esc`/`q` from the sandbox returns to the menu; `p` from the sandbox returns to the game.
- [ ] `block_font::compose("CHARLO", 1)` returns exactly 6 lines of equal length; an unknown character composes to blank width without panicking.
- [ ] `block_font::draw_banner` paints shadow cells offset `+1,+1` in the shadow color and main cells in the fg color, with main overdrawing shadow on overlap.
- [ ] All existing tests (game `map_key`, sandbox `map_sandbox_key`, etc.) still pass after the router refactor.

---

## Tasks
Ordered by dependency.

- [ ] **Block-font module:** Create `src/block_font.rs` with `HEIGHT`, the `glyph` table for `C H A R L O` + space + blank fallback, `compose`, and `draw_banner`. Unit-test first (RED): `compose` line count/equal-width, unknown-char fallback is blank (no panic), and `draw_banner` shadow/main cell placement + colors against a `Buffer`. Add `mod block_font;` to `main.rs`. Must be unit-tested before the menu task uses it.
- [ ] **Screen router refactor:** In `src/main.rs`, replace `ScreenExit` with `Screen` and `Nav` and rewrite `run()` to boot into `Screen::Menu` and dispatch by `current`. Update `src/app.rs` and `src/sandbox.rs` loops to return `Nav`, translating their existing `Quit` commands to `Nav::To(Screen::Menu)` and `Switch` to `To(Sandbox)`/`To(Game)` respectively. Existing `map_key`/`map_sandbox_key` tests must remain green. (Menu/About loops are stubbed until their tasks land — keep the project compiling.)
- [ ] **Menu screen:** Create `src/menu.rs` with `MenuItem`/`ITEMS`, `Menu` selection state with wraparound, pure `map_menu_key`, pure `activate(item) -> Nav`, the render (banner via `block_font` + highlighted list + footer), and the `menu()` loop. RED first: tests for `map_menu_key`, wraparound `up()`/`down()`, and `activate` routing each item to the right `Nav`. Wire `menu()` into the router as the boot screen.
- [ ] **About placeholder screen:** Create `src/about.rs` with `map_about_key`, the placeholder render, and the `about()` loop returning `Nav`. RED first: `map_about_key` returns the back command for `Esc`/`q`. Wire into the router.
- [ ] **Integration smoke test:** Verify end-to-end wiring. Add a router-level test exercising the pure transition seams (`activate(Play)→To(Game)`, `activate(About)→To(About)`, `activate(Quit)→Quit`; game/about/sandbox `Esc` → `To(Menu)`). Then a manual smoke run: launch the binary, confirm the menu renders the banner, navigate to Play (enter game) → `Esc` (back to menu) → About (placeholder) → `Esc` → Quit. Confirm `cargo test` and `cargo clippy` are clean.

---

## Considerations
- **Banner sizing on small terminals:** the composed CHARLO banner is ~40 cells wide. `draw_banner` must bounds-check every cell (via `buf.cell_mut`) so a narrow terminal clips the banner instead of panicking — mirror the off-buffer handling in `particle_render::draw_particles`.
- **Don't widen `map_key` semantics:** the game/sandbox key-mapping functions and their tests stay as-is; only the loops' translation into `Nav` changes. Resist editing the `Command`/`SandboxCommand` enums.
- **Sandbox terminal setup:** `sandbox()` enables/disables mouse capture around its loop; returning to the menu must go through its existing teardown path so the terminal state is restored before the menu draws.
- **Screen loops are IO-bound** and not directly unit-testable; correctness rests on the pure seams (key maps, `compose`, `activate`, wraparound) plus the manual smoke run. Keep logic out of the loop bodies and in those testable functions.
- **Highlight legibility:** the selected menu item must be visually distinct in the green palette (e.g. reversed video or a bright-on-dark swap) — avoid a highlight that only differs by a hue the terminal may render faintly.
