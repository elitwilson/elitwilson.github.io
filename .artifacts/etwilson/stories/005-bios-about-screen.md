---
id: STR-005
title: BIOS About screen layout
epic: EPIC-002
status: specced
priority: high
---

## Goal

Replace the "About — coming soon" placeholder in `src/about.rs` with a fully rendered BIOS-aesthetic terminal layout. This is the visual identity of the site — the first real content screen.

---

## Scope

### In
- Full replacement of `About::render()` with the BIOS layout
- BIOS header bar, POST boot log with blinking cursor, ELI WILSON title + subtitle, PROFILE.TXT panel, two-column SKILLS.SYS + CAREER.LOG, footer hints
- BIOS color palette via `Color::Rgb` (indigo bg, phosphor green, dim green, secondary blue, panel text)
- Bordered panels with inverted-color header title bars
- Placeholder data for bio, skills, and jobs — hardcoded in code
- Layout fits one terminal screen (no scrolling)

### Out
- Scrolling
- Real bio text or real job history (follow-up)
- Scanline / flicker effects
- Changes to `map_about_key`, `AboutCommand`, or existing tests

---

## Acceptance Criteria

- [ ] `cargo run` shows the About screen with all sections visible at a standard terminal size (80×24 minimum)
- [ ] BIOS header bar appears at the top with secondary-color text
- [ ] POST boot log shows 4–5 lines ending with `> READY.` and a blinking cursor block
- [ ] ELI WILSON title is prominent and bright green; subtitle is in secondary color
- [ ] PROFILE.TXT panel has a green border and an inverted (green bg, dark text) header bar
- [ ] SKILLS.SYS and CAREER.LOG panels render side-by-side with the same inverted-header style
- [ ] Skill bars use `█`/`░` Unicode blocks with name and `n/10` label
- [ ] Footer hint row matches the design: `↑/↓  w/s  scroll  ·  Esc  back to menu  ·  q  quit`
- [ ] `Esc` and `q` still navigate back to the menu (no regression)
- [ ] `cargo test` passes

---

## Context & Decisions

- `About` is a unit struct — no state needed. Blinking cursor uses `Modifier::SLOW_BLINK`; no tick-based animation.
- `Block::title()` does not support background inversion. Each panel's header bar (`PROFILE.TXT`, `SKILLS.SYS`, `CAREER.LOG`) must be rendered as a separate `Paragraph` over the top row of the panel's inner area, styled with green bg and dark text.
- Two-column section uses `Constraint::Percentage(50)` horizontal split.
- Color palette from the design: indigo bg `(30, 27, 75)`, bright green `(61, 224, 53)`, dim green `(42, 168, 74)`, secondary blue `(127, 143, 217)`, panel text `(191, 233, 189)`.
- All dynamic content (bio, skills, jobs) is hardcoded placeholder data. No content abstraction needed yet.
- `map_about_key`, `AboutCommand`, and all existing tests in `src/about.rs` are untouched.

---

## Dependencies

- **Depends on:** none
- **Blocks:** none

---

## Notes

- `src/about.rs` is the only file in scope. The game-level `render_modal` in `src/render.rs` is a separate in-game overlay — not touched.
- The design source is at `.artifacts/etwilson/epics/002-about-screen.md` and the original Claude Design file (`About.dc.html`) in project `190a7d49-b570-40ad-a84e-43e660799464`.
- Existing layout patterns to follow: `src/render.rs` uses `Layout::vertical` / `Layout::horizontal` with `Constraint` — same approach here.
