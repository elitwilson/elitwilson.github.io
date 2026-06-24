---
id: EPIC-002
title: About Screen — BIOS Layout
status: ready
created: 2026-06-23
---

## Goal
Replace the "About — coming soon" placeholder in `src/about.rs` with a fully rendered BIOS-aesthetic terminal layout matching the Claude Design mockup. The screen establishes visual identity through layout and color — real content (bio, jobs) is a follow-up; this epic is about getting the structure right.

---

## Scope In
- **BIOS header bar** — single row: `WILSON-BIOS (C) 1981  v2.6.0` left, `SYS: ATARI 2600 / RATATUI WASM` right; secondary color (`#7f8fd9`)
- **POST boot log** — 4–5 lines of POST output in dim green (`#2aa84a`), final `> READY.` in bright green, blinking cursor block via `Modifier::SLOW_BLINK`
- **Title section** — `ELI WILSON` (prominent, bright green) + subtitle `// SOFTWARE DEVELOPER  ·  PLAYER 1` in secondary color
- **PROFILE.TXT panel** — bordered box, inverted-green header title bar (`PROFILE.TXT`), placeholder bio text in panel-text color (`#bfe9bd`)
- **Two-column section** — equal-width `SKILLS.SYS` and `CAREER.LOG` panels side by side:
  - Skills: hardcoded placeholder entries with `█`/`░` bar, name, and `n/10` label
  - Career: hardcoded placeholder job entries (span, role, org)
- **Footer hints row** — `↑/↓  w/s  scroll  ·  Esc  back to menu  ·  q  quit` in secondary color
- **Color palette** — all via `Color::Rgb`: indigo bg `(30, 27, 75)`, bright green `(61, 224, 53)`, dim green `(42, 168, 74)`, secondary blue `(127, 143, 217)`, panel text `(191, 233, 189)`
- **Bordered panels** — `Block::bordered()` with bright-green border; header title bar rendered as an inverted row (green bg, dark text) inside each panel's inner rect
- **Single-screen layout** — designed to fit one terminal screen; sections are abbreviated before scrolling is considered

## Scope Out
- Scrolling (follow-up epic)
- Real bio text and real job history (content fill-in is a follow-up)
- Scanline or flicker visual effects (not achievable in terminal)
- Title font changes (terminal font is fixed; no pixel/bitmap font)
- Routing or key-handling changes — existing `Esc`/`q` → back is already wired

---

## Key Decisions
- **Layout-first delivery.** All dynamic content (bio, skills, jobs) uses hardcoded placeholder data in code. The structural seam for later content replacement exists; populating it is out of scope.
- **No scrolling.** Layout must fit one screen. If content is too tall, sections are trimmed or abbreviated rather than made scrollable.
- **Blinking cursor via `Modifier::SLOW_BLINK`.** The POST boot log's trailing cursor block uses Ratatui's built-in blink modifier. No tick-based animation state is needed — the terminal renderer handles it.
- **Inverted panel headers.** Ratatui's `Block::title()` doesn't support background inversion. Each panel's header bar (`PROFILE.TXT`, `SKILLS.SYS`, `CAREER.LOG`) is rendered as a separate `Paragraph` over the top row of the panel's inner area, styled with green bg and dark text, to replicate the design's solid-header-inside-border look.
- **Two-column split via `Layout`.** Skills and career panels use two `Constraint::Percentage(50)` horizontal constraints.
- **`About` struct and key-handling preserved.** Only `About::render` is replaced. `map_about_key`, `AboutCommand`, and the existing tests are untouched.
