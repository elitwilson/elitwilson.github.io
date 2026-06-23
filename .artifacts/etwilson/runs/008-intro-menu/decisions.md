# Decisions — 008-intro-menu

## Assumptions

1. The `Screen` and `Nav` enums are defined in `main.rs` (not in a dedicated module) since `main.rs` is the router.
2. The `pub enum` visibility for `Screen`/`Nav` is `pub(crate)` since they're only used within the crate (same pattern as `ScreenExit`).
3. The glyph art for block_font uses `█` (U+2588 FULL BLOCK) as filled cells and space for empty, as specified in the spec.
4. The `compose` function uses a `gap` parameter of spaces between letters.
5. For the About screen stub during the router refactor task, a simple placeholder function is sufficient to keep compilation green.
6. The existing `show_about` field in `App` is unrelated to the new `About` screen — the game's in-game "about" modal is a different thing and is untouched per the spec.
7. The `map_key` tests in `app.rs` that test `q`/`Esc` → `Command::Quit` remain valid — only the loop's translation of `Command::Quit` into `Nav` changes.
8. The `draw_banner_main_overdrawes_shadow_at_overlap` test was corrected after RED: the original checked all "█" cells within the glyph bounding box, but shadow cells from row N can land in the main bounding box at row N+1 without being overdrawn (if row N+1 has a space at that column). Fixed to check only positions where the compose output has a non-space character (the actual main-layer cells).
