# Codenotch for Windows — M0–M2 Fixture Design

Date: 2026-09-06
Scope: M0 (skeleton) + M1 (notch surface) + M2 (provider cells, static fixture only)
Approach: A — Tauri v2 + React + SVG, window shell first
Reference: `E:\status bar for ai\codenotch` (macOS Swift build), design frame
`docs/design/frame-124-hover-tooltip.png` is the source of truth for geometry and color.

## Goal

A borderless always-on-top Tauri panel pinned to the screen edge, rendering three
static provider rings (73% / 21% / 52%) that visually match the design frame.
No live providers, no network, no credentials in this slice.

## Architecture

```
codenotch-win/
  src-tauri/          Rust: window shell, work-area anchor, updater stub
    src/
      window/         edge placement, work-area query, fold state (stub)
      ipc/            typed commands (get_work_area only in this slice)
  src/                TypeScript/React: notch SVG, rings, fixture root
    design/           palette.ts, typography.ts, design.ts (one `scale`)
    components/       SideNotchShape, ProviderRing, ProviderCell, NotchRootView
    state/            fixtures.ts (73/21/52, CODENOTCH_DEMO=1 semantics)
```

- Rust backend does window shell + one `get_work_area` command via
  `GetMonitorInfo` (work area, not raw monitor rect — respects taskbar).
- All notch math in TS `src/design/`, mirroring Swift `DesignSystem/`.
- No `PlatformAdapter` abstraction, no provider trait, no store in this slice
  (Ponytail: smallest diff for M0–M2 acceptance).

## Design tokens (frame wins over prose spec)

- Track `#303030`, bars `#2D2D2D`, green `#00FF88`, yellow `#F2FF00`,
  orange `#FF3F00`, card `#000000` at 18.5pt-equivalent radius.
- Bands: 0–49% ample, 50–69% watch, 70–100% critical (frame renders 21% green /
  52% yellow / 73% orange; prose spec's 50/79 table is wrong per TASKS.md
  "Corrections to the spec").
- `scale = 44 / 117` (44pt ring = 117px in frame), `px(n) = n * scale`.
  MUST sanity-check against the 70x401pt notch body anchor while looking at
  frame-124-hover-tooltip.png before locking `design.ts` — the ratio drives
  every proportional size.

## Components

- `tauri.conf.json`: borderless, transparent, always-on-top, `skip_taskbar: true`,
  no decorations. Must not steal focus, no Alt-Tab entry, no taskbar entry.
- `SideNotchShape`: SVG inverse-rounded pill, right edge default.
- `UsageBand`: `usedFraction` → `ample | watch | critical`. Exhaustive switch,
  no default swallow.
- `ProviderRing`: arc from 12 o'clock clockwise, `trim` = fraction.
- `ProviderCell`: ring + glyph + percent label (`used`, not remaining).
- `NotchRootView`: vertical stack, 1–5 cells (3 in fixture).
- Glyphs (fixture): single-path simplified marks or text initials inside the ring
  (e.g. distinct abstract glyph per cell, recognizable at a glance). Exact tracing of
  Claude/OpenAI/Perplexity marks and Cursor-SVG-flatten / Gemini-four-arc
  generation are deferred to M2 polish — scope creep if done now.

## Data flow

Fixture only: `fixtures.ts` returns three static snapshots. No fetch, no polling,
no backoff, no archive, no activity monitors. `CODENOTCH_DEMO=1` semantics
preserved so later live providers slot behind the same render path.

Type discipline (every `.ts`/`.tsx`): no `any`, no `as` except at verified I/O
boundaries (none in this slice), discriminated unions for status, exhaustive
switches. Zod deferred to M4 (first serialization boundary).

## Errors and platform gotchas

- Re-anchor on display/taskbar change (Tauri scale/window events; `WM_DISPLAYCHANGE`
  hookup lands with the Rust shell, poll fallback acceptable in fixture).
- DPI: verify no flush-gap at 125%/150% (macOS bug class: fractional-vs-rounded
  frame math). Round panel frame to whole physical pixels.
- Click-through outside the silhouette: `WS_EX_TRANSPARENT` toggling tracked
  against the actual shape; manual test clicking just outside the curved flare.
- Hardware-notch merge: dropped, no Windows equivalent.

## Acceptance (state before checking off, verify against real build)

- M0: process in Task Manager, not in taskbar; no titlebar/border in any theme.
- M1: notch flush against work-area edge, no visible gap at 100/125/150%;
  pass-through outside flare; re-anchors on taskbar move/monitor change.
- M2: fixture 73%/21%/52% with frame colors/bands visually matches
  `frame-124-hover-tooltip.png` (rings + labels; glyphs recognizable, not exact).

## Out of scope (this slice)

Providers, Tooltip (M3), store/scheduler/backoff/archive (M4), settings (M5),
folding/motion/edges (M6a), packaging/signing/updater (M6). Perplexity stays
unregistered per upstream.
