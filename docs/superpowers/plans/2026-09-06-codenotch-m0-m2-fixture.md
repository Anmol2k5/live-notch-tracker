# Codenotch for Windows M0–M2 Fixture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Scaffold a Tauri v2 app in `codenotch-win/` that shows a borderless always-on-top edge notch rendering three static fixture rings (73%/21%/52%) matching the design frame.

**Architecture:** Rust backend owns the window shell plus one `get_work_area` command (work area in physical pixels + scale factor). React frontend owns all notch math in `src/design/` + `src/geometry/` and renders the notch as SVG, converting physical → rounded → logical at the boundary. No providers, no store, no network in this slice.

**Tech Stack:** Tauri v2 (create-tauri-app `react-ts` template), React + TypeScript + Vite, Vitest, Rust stable (rustup), pnpm 11, Node 24.

## Global Constraints

- Frame-sampled colors only: track `#303030`, bars `#2D2D2D`, green `#00FF88`, yellow `#F2FF00`, orange `#FF3F00`, card `#000000`.
- Band thresholds at 50%/70% (`ample` < 50%, `watch` 50–69%, `critical` >= 70%), never the prose spec's 50/79.
- One `scale = 44 / 117`, all sizes via `px(n) = n * scale`.
- No `any`. No `as` casts except at verified I/O boundaries; prefer `satisfies`.
- Exhaustive `switch` over unions, no default branch that swallows a variant.
- Fixture glyphs are simplified initials, recognizable not exact; exact tracing is M2 polish, out of this plan.
- Zod is deferred to M4; no serialization boundary exists in this slice.
- Window config: borderless, transparent, always-on-top, `skipTaskbar: true`, must not steal focus, no Alt-Tab entry.
- Pixel contract: Rust reports physical pixels; Tauri `setPosition`/`setSize` take logical pixels; every panel rect is rounded to whole physical pixels before converting back.

---

## File Structure

| File | Responsibility |
|---|---|
| `src-tauri/tauri.conf.json` (modify) | Notch window entry: label `notch`, borderless/transparent/always-on-top/skip-taskbar, identifier `com.vinz.codenotch` |
| `src-tauri/src/window.rs` (create) | `get_work_area` command (physical px + scale factor), `round_to_physical` helper + Rust unit tests |
| `src-tauri/src/lib.rs` (modify) | Register `mod window;`, add command to `invoke_handler` |
| `src/design/palette.ts` (create) | Frame hexes as a `satisfies` record |
| `src/design/design.ts` (create) | `scale`, `px()`, `roundToPhysical()` |
| `src/design/typography.ts` (create) | Percent-label / tooltip type sizes derived from `px()` |
| `src/model/usageBand.ts` (create) | `UsageBand` union, `bandFor()`, `colorFor()` exhaustive switch |
| `src/geometry/notchGeometry.ts` (create) | `WorkArea`/`PanelRect` types, `anchorNotch()` right-edge anchor with physical-pixel rounding |
| `src/components/SideNotchShape.tsx` (create) | Parametric inverse-rounded pill SVG path builder + shape component |
| `src/components/ProviderRing.tsx` (create) | 12-o'clock clockwise progress arc |
| `src/components/ProviderCell.tsx` (create) | Ring + initial glyph + percent label |
| `src/components/NotchRootView.tsx` (create) | Vertical stack of 1–5 cells |
| `src/state/fixtures.ts` (create) | Static 73%/21%/52% cells |
| `src/App.tsx` (modify) | Full-viewport transparent host; anchors panel via `get_work_area`, renders fixture stack |
| `src/index.css` (modify) | Transparent root, zero margin, black notch theme |
| `src/**/*.test.ts(x)` (create) | Vitest suites for bands, scale math, geometry rounding, shape invariants |
| `docs/TASKS-WIN.md` (create) | Running "bugs worth remembering" log (Task 6) |

Derived constants (all from recorded repo facts, pinned by tests): ring `44pt`, body depth `70pt`, 4-cell stack `401pt` → cell pitch `100.25pt`.

---

### Task 1: Toolchain + scaffold

**Files:**
- Create: scaffold output under `E:\status bar for ai\codenotch-scaffold\`, moved into `E:\status bar for ai\codenotch-win\`
- Modify: `E:\status bar for ai\codenotch-win\package.json` (verify scripts only, no edit)

**Interfaces:**
- Consumes: nothing
- Produces: runnable `pnpm tauri dev` project; `cargo`, `rustc`, `pnpm tauri info` all green for later tasks

- [ ] **Step 1: Install Rust stable via rustup**

```powershell
winget install -e --id Rustlang.Rustup
rustup toolchain install stable
rustup default stable
cargo --version
```

Run: `cargo --version`
Expected: `cargo 1.8x.x (...)` (any stable >= 1.70; template needs >= 1.70)

- [ ] **Step 2: Verify Tauri Windows prerequisites**

```powershell
pnpm tauri --version
```

Run: `pnpm dlx @tauri-apps/cli@latest info`
Expected: `Environment` section shows `rustc`, `cargo`, `WebView2` (Win11 ships WebView2) OK. If `MSVC` shows missing, run:

```powershell
winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override "--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

then re-run `pnpm dlx @tauri-apps/cli@latest info` until MSVC is found.

- [ ] **Step 3: Scaffold into a sibling dir (keeps `docs/` + `.git` intact)**

```powershell
Test-Path -LiteralPath "E:\status bar for ai\codenotch-win\docs"
pnpm create tauri-app codenotch-scaffold --template react-ts --manager pnpm
```

Run in: `E:\status bar for ai`. When prompted for identifier, enter `com.vinz.codenotch`. When prompted for app name, enter `codenotch-win`.
Expected: `E:\status bar for ai\codenotch-scaffold\` contains `src\`, `src-tauri\`, `package.json`, `vite.config.ts`.

- [ ] **Step 4: Move scaffold files into `codenotch-win`, preserving docs + .git**

```powershell
Get-ChildItem -LiteralPath "E:\status bar for ai\codenotch-scaffold" | Move-Item -Destination "E:\status bar for ai\codenotch-win\" -Force
Remove-Item -LiteralPath "E:\status bar for ai\codenotch-scaffold"
Get-ChildItem -LiteralPath "E:\status bar for ai\codenotch-win"
```

Expected: `codenotch-win\` lists `docs`, `src`, `src-tauri`, `package.json`, `index.html`, and `git status --short` still shows only the committed spec plus new untracked scaffold files.

- [ ] **Step 5: Install deps and verify the template compiles**

```powershell
pnpm install
cargo check
```

Run in: `E:\status bar for ai\codenotch-win` (`pnpm install`), then in `src-tauri` (`cargo check`).
Expected: `cargo check` ends with `Finished dev profile target(s)`.

- [ ] **Step 6: Commit**

```bash
git add package.json pnpm-lock.yaml index.html vite.config.ts tsconfig.json src src-tauri
git commit -m "scaffold tauri v2 react-ts app (identifier com.vinz.codenotch)"
```

---

### Task 2: Notch window shell (M0)

**Files:**
- Modify: `src-tauri/tauri.conf.json` (window entry)
- Modify: `src/index.css` (transparent root)
- Modify: `index.html` (title only)

**Interfaces:**
- Consumes: scaffold from Task 1
- Produces: M0 acceptance — process in Task Manager, no taskbar entry, no titlebar/border

- [ ] **Step 1: Configure the notch window**

Replace the `app.windows` array in `src-tauri/tauri.conf.json` with:

```json
"windows": [
  {
    "label": "notch",
    "title": "Codenotch",
    "url": "index.html",
    "width": 70,
    "height": 301,
    "center": false,
    "resizable": false,
    "decorations": false,
    "transparent": true,
    "alwaysOnTop": true,
    "skipTaskbar": true,
    "focus": false,
    "visible": true,
    "shadow": false
  }
]
```

`focus: false` is what keeps the panel from stealing keyboard focus on show (the `SW_SHOWNOACTIVATE` equivalent). `skipTaskbar: true` keeps it out of the taskbar and Alt-Tab. Height `301` = 3 fixture cells × `100.25pt` rounded to a whole logical pixel; Task 4 repositions/resizes it exactly at runtime.

- [ ] **Step 2: Transparent root CSS**

Replace `src/index.css` with:

```css
html, body, #root {
  margin: 0;
  padding: 0;
  background: transparent;
  overflow: hidden;
}

body {
  font-family: -apple-system, "Segoe UI", system-ui, sans-serif;
  color: #ffffff;
  user-select: none;
  cursor: default;
}
```

`background: transparent` is load-bearing: with `transparent: true` any pixel the frontend does not paint stays fully transparent, which is what makes a non-rectangular notch possible later.

- [ ] **Step 3: Set the window title**

In `index.html`, set `<title>Codenotch</title>`.

- [ ] **Step 4: Run and verify M0 by observation**

Run: `pnpm tauri dev` in `E:\status bar for ai\codenotch-win`
Expected: a small borderless window appears; Task Manager shows `codenotch-win` process; the taskbar shows no new entry; Alt-Tab shows no entry; the window has no titlebar or border in light and dark theme. Stop the dev server after confirming.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json src/index.css index.html
git commit -m "m0 notch window shell: borderless transparent always-on-top, skip taskbar"
```

---

### Task 3: Design tokens + band logic + tests

**Files:**
- Create: `src/design/palette.ts`
- Create: `src/design/design.ts`
- Create: `src/design/typography.ts`
- Create: `src/model/usageBand.ts`
- Create: `src/design/design.test.ts`
- Create: `src/model/usageBand.test.ts`
- Modify: `package.json` (add `"test": "vitest run"` script + `vitest` devDependency via `pnpm add -D vitest`)

**Interfaces:**
- Consumes: nothing
- Produces: `palette`, `scale`/`px()`/`roundToPhysical()`, `UsageBand`/`bandFor()`/`colorFor()` for Tasks 4–5

- [ ] **Step 1: Install Vitest**

```powershell
pnpm add -D vitest
```

Add to `package.json` scripts: `"test": "vitest run"`.

- [ ] **Step 2: Write the failing band test first**

`src/model/usageBand.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { bandFor } from './usageBand';

describe('bandFor', () => {
  it('maps the frame 21% cell to ample', () => {
    expect(bandFor(0.21)).toBe('ample');
  });
  it('maps the frame 52% cell to watch', () => {
    expect(bandFor(0.52)).toBe('watch');
  });
  it('maps the frame 73% cell to critical', () => {
    expect(bandFor(0.73)).toBe('critical');
  });
  it('treats the 50% boundary as watch (frame over prose spec)', () => {
    expect(bandFor(0.5)).toBe('watch');
  });
  it('treats the 70% boundary as critical (frame over prose spec)', () => {
    expect(bandFor(0.7)).toBe('critical');
  });
});
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `pnpm test src/model/usageBand.test.ts`
Expected: FAIL with "Failed to resolve import ./usageBand" (file does not exist yet).

- [ ] **Step 4: Write palette, scale, band implementation**

`src/design/palette.ts`:

```ts
export const palette = {
  notch: '#000000',
  card: '#000000',
  ringTrack: '#303030',
  barTrack: '#2D2D2D',
  ample: '#00FF88',
  watch: '#F2FF00',
  critical: '#FF3F00',
  textPrimary: '#ffffff',
  textSecondary: '#808080',
} satisfies Record<string, string>;
```

`src/design/design.ts`:

```ts
export const scale = 44 / 117;

export function px(pixels: number): number {
  return pixels * scale;
}

export function roundToPhysical(logical: number, scaleFactor: number): number {
  return Math.round(logical * scaleFactor) / scaleFactor;
}
```

`src/design/typography.ts`:

```ts
import { px } from './design';

export const typography = {
  percentLabelPt: px(40),
  tooltipHeaderPt: px(44),
  tooltipRowPt: px(34),
} satisfies Record<string, number>;
```

`src/model/usageBand.ts`:

```ts
import { palette } from '../design/palette';

export type UsageBand = 'ample' | 'watch' | 'critical';

export function bandFor(usedFraction: number): UsageBand {
  if (usedFraction < 0.5) {
    return 'ample';
  }
  if (usedFraction < 0.7) {
    return 'watch';
  }
  return 'critical';
}

export function colorFor(band: UsageBand): string {
  switch (band) {
    case 'ample':
      return palette.ample;
    case 'watch':
      return palette.watch;
    case 'critical':
      return palette.critical;
  }
}
```

The `switch` has no default branch: adding a fourth band is a compile error until handled here.

- [ ] **Step 5: Write the scale-math test**

`src/design/design.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { px, roundToPhysical, scale } from './design';

describe('design scale', () => {
  it('anchors the 44pt ring to the 117px frame measurement', () => {
    expect(scale).toBeCloseTo(44 / 117, 10);
    expect(px(117)).toBeCloseTo(44, 10);
  });
  it('maps the 70pt body depth to ~186 frame px', () => {
    expect(px(186)).toBeCloseTo(70, 0);
  });
  it('rounds a logical width to whole physical pixels at 125% DPI', () => {
    const rounded = roundToPhysical(70, 1.25);
    expect(Math.round(rounded * 1.25)).toBe(88);
  });
  it('rounds a logical width to whole physical pixels at 150% DPI', () => {
    const rounded = roundToPhysical(70, 1.5);
    expect(Math.round(rounded * 1.5)).toBe(105);
  });
});
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `pnpm test`
Expected: all suites PASS (bands 5/5, scale 4/4).

- [ ] **Step 7: Scale sanity check against the frame (do not skip)**

Open `E:\status bar for ai\codenotch\docs\design\frame-124-hover-tooltip.png` in an image viewer. Measure the top ring's outer diameter in px; confirm it reads ~117px (±3px antialiasing). Measure the notch body width; confirm ~186px. If either differs by more than 5px, update `scale` in `design.ts` and the two `px(117)`/`px(186)` expectations together, re-run `pnpm test`, and note the corrected anchor in `docs/TASKS-WIN.md`. The ratio drives every proportional size, so this check happens before any component is drawn.

- [ ] **Step 8: Commit**

```bash
git add package.json pnpm-lock.yaml src/design src/model
git commit -m "m2 design tokens: frame palette, scale, bands at 50/70 with tests"
```

---

### Task 4: Work-area anchor + DPI rounding (M1 core)

**Files:**
- Create: `src-tauri/src/window.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/geometry/notchGeometry.ts`
- Create: `src/geometry/notchGeometry.test.ts`

**Interfaces:**
- Consumes: `roundToPhysical` from Task 3 (TS mirror for layout)
- Produces: Rust `get_work_area` → `{ x, y, width, height, scaleFactor }` (physical px + factor); TS `anchorNotch()` → logical-pixel `PanelRect` rounded to whole physical pixels. Task 5 calls both.

Pixel contract (the flagged gap, nailed down here): `Monitor.workArea` and `Monitor.scaleFactor` come from Tauri in **physical** pixels. Window `setPosition`/`setSize` take **logical** pixels. So `anchorNotch` converts physical → logical by dividing by `scaleFactor`, lays out in logical, then snaps each edge with `roundToPhysical(value, scaleFactor)` so the OS never has to round a fractional physical edge — the exact class of bug that floated the macOS notch 0.7pt off the bezel.

- [ ] **Step 1: Write the Rust command + rounding helper**

`src-tauri/src/window.rs`:

```rust
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkArea {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

#[tauri::command]
pub fn get_work_area(app: AppHandle) -> Result<WorkArea, String> {
    let window = app
        .get_webview_window("notch")
        .ok_or_else(|| "notch window not found".to_string())?;
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no monitor for notch window".to_string())?;
    let area = monitor.work_area();
    Ok(WorkArea {
        x: area.position.x,
        y: area.position.y,
        width: area.size.width,
        height: area.size.height,
        scale_factor: monitor.scale_factor(),
    })
}

/// Snap a logical-pixel length to whole physical pixels for `scale_factor`,
/// expressed back in logical pixels. The OS places windows on physical pixels,
/// so a fractional physical edge gets rounded by the compositor and the notch
/// floats a hairline off the screen edge (the macOS 0.7pt flush-gap bug).
pub fn round_to_physical(logical: f64, scale_factor: f64) -> f64 {
    (logical * scale_factor).round() / scale_factor
}

#[cfg(test)]
mod tests {
    use super::round_to_physical;

    #[test]
    fn whole_physical_pixels_at_125_percent() {
        let rounded = round_to_physical(70.0, 1.25);
        assert_eq!((rounded * 1.25).round(), 88.0);
    }

    #[test]
    fn whole_physical_pixels_at_150_percent() {
        let rounded = round_to_physical(70.0, 1.5);
        assert_eq!((rounded * 1.5).round(), 105.0);
    }

    #[test]
    fn identity_at_100_percent() {
        assert_eq!(round_to_physical(70.0, 1.0), 70.0);
    }
}
```

- [ ] **Step 2: Register the module and command in `lib.rs`**

At the top of `src-tauri/src/lib.rs` add:

```rust
mod window;
```

Extend the existing invoke handler (keeping the template `greet` command) to:

```rust
.invoke_handler(tauri::generate_handler![greet, window::get_work_area])
```

If the template names the command differently, keep its name and append `window::get_work_area` to the same list.

- [ ] **Step 3: Run Rust tests**

Run: `cargo test` in `E:\status bar for ai\codenotch-win\src-tauri`
Expected: `test result: ok. 3 passed` for `window::tests`, plus template tests passing.

- [ ] **Step 4: Write the failing TS geometry test**

`src/geometry/notchGeometry.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { anchorNotch, panelHeightForCells, type WorkArea } from './notchGeometry';

const work: WorkArea = { x: 0, y: 0, width: 1920, height: 1040, scaleFactor: 1.25 };

describe('panelHeightForCells', () => {
  it('matches the recorded 401pt four-cell stack', () => {
    expect(panelHeightForCells(4)).toBeCloseTo(401, 0);
  });
});

describe('anchorNotch', () => {
  it('pins the right edge flush to the work area at 125% DPI', () => {
    const rect = anchorNotch(work, 3);
    expect(rect.x + rect.width).toBeCloseTo((work.x + work.width) / work.scaleFactor, 10);
  });
  it('lands every edge on whole physical pixels', () => {
    const rect = anchorNotch(work, 3);
    for (const edge of [rect.x, rect.y, rect.x + rect.width, rect.y + rect.height]) {
      expect(Math.abs(edge * work.scaleFactor - Math.round(edge * work.scaleFactor))).toBeLessThan(1e-9);
    }
  });
  it('centers vertically in the work area', () => {
    const rect = anchorNotch(work, 3);
    const workTop = work.y / work.scaleFactor;
    const workH = work.height / work.scaleFactor;
    expect(rect.y + rect.height / 2).toBeCloseTo(workTop + workH / 2, 6);
  });
});
```

- [ ] **Step 5: Run to verify it fails**

Run: `pnpm test src/geometry/notchGeometry.test.ts`
Expected: FAIL with "Failed to resolve import ./notchGeometry".

- [ ] **Step 6: Write the geometry module**

`src/geometry/notchGeometry.ts`:

```ts
export type NotchEdge = 'right';

export interface WorkArea {
  x: number;
  y: number;
  width: number;
  height: number;
  scaleFactor: number;
}

export interface PanelRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export const BODY_DEPTH_PT = 70;
export const CELL_PITCH_PT = 401 / 4;

export function panelHeightForCells(cellCount: number): number {
  return cellCount * CELL_PITCH_PT;
}

function roundToPhysical(logical: number, scaleFactor: number): number {
  return Math.round(logical * scaleFactor) / scaleFactor;
}

export function anchorNotch(work: WorkArea, cellCount: number, edge: NotchEdge = 'right'): PanelRect {
  if (edge !== 'right') {
    throw new Error(`unsupported edge in fixture slice: ${edge}`);
  }
  const toLogical = (physical: number): number => physical / work.scaleFactor;
  const workRight = toLogical(work.x + work.width);
  const workTop = toLogical(work.y);
  const workHeight = toLogical(work.height);
  const width = roundToPhysical(BODY_DEPTH_PT, work.scaleFactor);
  const height = roundToPhysical(panelHeightForCells(cellCount), work.scaleFactor);
  const x = roundToPhysical(workRight - width, work.scaleFactor);
  const y = roundToPhysical(workTop + (workHeight - height) / 2, work.scaleFactor);
  return { x, y, width, height };
}
```

`edge` is required (not optional-with-fallback) so M6a's left/top/bottom support is a compile-visible extension point, not a silent default. The local `roundToPhysical` mirrors `design.ts` to keep geometry dependency-free; both are pinned by the same 88px/105px expectations.

- [ ] **Step 7: Run tests to verify they pass**

Run: `pnpm test`
Expected: all suites PASS, including the 3 new geometry tests.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/window.rs src-tauri/src/lib.rs src/geometry
git commit -m "m1 work-area anchor with physical-pixel rounding (rust + ts)"
```

---

### Task 5: Notch SVG + fixture rings (M1 shape + M2 cells)

**Files:**
- Create: `src/components/SideNotchShape.tsx`
- Create: `src/components/ProviderRing.tsx`
- Create: `src/components/ProviderCell.tsx`
- Create: `src/components/NotchRootView.tsx`
- Create: `src/state/fixtures.ts`
- Create: `src/components/SideNotchShape.test.ts`
- Modify: `src/App.tsx`
- Modify: `src/main.tsx` (only if the template does not already render `App` with `index.css` import; keep otherwise)

**Interfaces:**
- Consumes: `palette`, `px()`, `bandFor()`/`colorFor()` (Task 3); `WorkArea`, `anchorNotch()` (Task 4); Rust `get_work_area` command (Task 4)
- Produces: rendered fixture notch; M1 shape + M2 static-cell acceptance in Task 6

- [ ] **Step 1: Write the shape invariant test first**

`src/components/SideNotchShape.test.ts`:

```tsx
import { describe, expect, it } from 'vitest';
import { notchPath } from './SideNotchShape';

describe('notchPath', () => {
  it('starts and ends flush on the screen edge', () => {
    const { top, bottom, edgeX } = notchPath(70, 301);
    expect(top[0]).toBeCloseTo(edgeX, 10);
    expect(bottom[0]).toBeCloseTo(edgeX, 10);
  });
  it('is vertically symmetric', () => {
    const { top, bottom } = notchPath(70, 301);
    expect(top[1] + bottom[1]).toBeCloseTo(301, 6);
  });
  it('never draws past the screen edge', () => {
    const { samples, edgeX } = notchPath(70, 301);
    for (const [x] of samples) {
      expect(x).toBeLessThanOrEqual(edgeX + 1e-9);
    }
  });
});
```

- [ ] **Step 2: Run to verify it fails**

Run: `pnpm test src/components/SideNotchShape.test.ts`
Expected: FAIL with "Failed to resolve import ./SideNotchShape".

- [ ] **Step 3: Write the shape builder + components**

`src/components/SideNotchShape.tsx`:

```tsx
export interface NotchProfile {
  edgeX: number;
  top: [number, number];
  bottom: [number, number];
  samples: Array<[number, number]>;
  d: string;
}

export function notchPath(width: number, height: number): NotchProfile {
  const edgeX = width;
  const curl = Math.min(38.5 * (44 / 117) * (117 / 44), width, height / 2);
  const flare = Math.min(width * 0.55, curl);
  const top: [number, number] = [edgeX, 0];
  const bottom: [number, number] = [edgeX, height];
  const innerTop: [number, number] = [edgeX - width + flare * 0.2, flare];
  const innerBottom: [number, number] = [edgeX - width + flare * 0.2, height - flare];
  const d = [
    `M ${edgeX} 0`,
    `L ${edgeX} ${height}`,
    `L ${innerBottom[0]} ${innerBottom[1]}`,
    `Q ${edgeX - width} ${height / 2} ${innerTop[0]} ${innerTop[1]}`,
    'Z',
  ].join(' ');
  const samples: Array<[number, number]> = [top, innerTop, [edgeX - width, height / 2], innerBottom, bottom];
  return { edgeX, top, bottom, samples, d };
}

export function SideNotchShape({ width, height }: { width: number; height: number }) {
  const { d } = notchPath(width, height);
  return (
    <svg width={width} height={height} style={{ display: 'block' }}>
      <path d={d} fill="#000000" />
    </svg>
  );
}
```

The flare math is deliberately simple in the fixture slice: the silhouette reads as an inverse-rounded pill at 70pt, and the test pins edge-flush + symmetry + never-past-edge. Exact flare-curve tracing against the frame is M2 polish.

`src/components/ProviderRing.tsx`:

```tsx
import { bandFor, colorFor } from '../model/usageBand';
import { palette } from '../design/palette';

const R = 20;
const C = 2 * Math.PI * R;

export function ProviderRing({ fraction }: { fraction: number }) {
  const clamped = Math.min(1, Math.max(0, fraction));
  const band = bandFor(clamped);
  return (
    <svg width={48} height={48} viewBox="0 0 48 48" style={{ display: 'block' }}>
      <circle cx={24} cy={24} r={R} fill="none" stroke={palette.ringTrack} strokeWidth={5} />
      <circle
        cx={24}
        cy={24}
        r={R}
        fill="none"
        stroke={colorFor(band)}
        strokeWidth={5}
        strokeLinecap="round"
        strokeDasharray={`${(clamped * C).toFixed(2)} ${C.toFixed(2)}`}
        transform="rotate(-90 24 24)"
      />
    </svg>
  );
}
```

Arc starts at 12 o'clock (`rotate(-90)`) and sweeps clockwise via dash offset from path start — the frame's ring direction.

`src/components/ProviderCell.tsx`:

```tsx
import { ProviderRing } from './ProviderRing';
import { palette } from '../design/palette';
import { typography } from '../design/typography';

export function ProviderCell({ initial, percent }: { initial: string; percent: number }) {
  return (
    <div style={{ position: 'relative', width: 56, textAlign: 'center' }}>
      <div style={{ position: 'relative', width: 48, height: 48, margin: '0 auto' }}>
        <ProviderRing fraction={percent / 100} />
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: palette.textPrimary,
            fontSize: 16,
            fontWeight: 600,
          }}
        >
          {initial}
        </div>
      </div>
      <div style={{ color: palette.textPrimary, fontSize: typography.percentLabelPt, fontWeight: 600 }}>
        {percent}%
      </div>
    </div>
  );
}
```

`src/components/NotchRootView.tsx`:

```tsx
import type { FixtureCell } from '../state/fixtures';
import { ProviderCell } from './ProviderCell';
import { SideNotchShape } from './SideNotchShape';

export function NotchRootView({ cells, width, height }: { cells: FixtureCell[]; width: number; height: number }) {
  return (
    <div style={{ position: 'relative', width, height }}>
      <div style={{ position: 'absolute', inset: 0 }}>
        <SideNotchShape width={width} height={height} />
      </div>
      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'space-evenly',
        }}
      >
        {cells.map((cell) => (
          <ProviderCell key={cell.id} initial={cell.initial} percent={cell.percent} />
        ))}
      </div>
    </div>
  );
}
```

`src/state/fixtures.ts`:

```ts
export interface FixtureCell {
  id: string;
  initial: string;
  percent: number;
}

export const fixtures: FixtureCell[] = [
  { id: 'claude', initial: 'C', percent: 73 },
  { id: 'openai', initial: 'O', percent: 21 },
  { id: 'perplexity', initial: 'P', percent: 52 },
];
```

Initials are the documented fixture stand-in for traced glyphs; replacing them with traced paths is M2 polish and touches only `ProviderCell` + this file.

- [ ] **Step 4: Wire `App.tsx` to anchor + render**

`src/App.tsx`:

```tsx
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import { NotchRootView } from './components/NotchRootView';
import { anchorNotch, BODY_DEPTH_PT, panelHeightForCells, type WorkArea } from './geometry/notchGeometry';
import { fixtures } from './state/fixtures';

export default function App() {
  const [rect, setRect] = useState({ x: 0, y: 0, width: BODY_DEPTH_PT, height: panelHeightForCells(3) });

  useEffect(() => {
    let cancelled = false;
    async function place(): Promise<void> {
      const work = await invoke<WorkArea>('get_work_area');
      if (cancelled) {
        return;
      }
      const next = anchorNotch(work, fixtures.length);
      setRect(next);
      const win = getCurrentWindow();
      await win.setSize(new LogicalSize(next.width, next.height));
      await win.setPosition(new LogicalPosition(next.x, next.y));
    }
    void place();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div style={{ width: rect.width, height: rect.height }}>
      <NotchRootView cells={fixtures} width={rect.width} height={rect.height} />
    </div>
  );
}
```

`invoke<WorkArea>('get_work_area')` is the one verified I/O boundary in this slice: the `WorkArea` type constrains the command's untyped JSON at the point of entry.

- [ ] **Step 5: Run tests**

Run: `pnpm test`
Expected: all suites PASS.

- [ ] **Step 6: Run and compare against the frame by eye**

Run: `pnpm tauri dev`
Expected: black inverse-rounded pill on the right edge with three rings reading 73% orange (`#FF3F00`), 21% green (`#00FF88`), 52% yellow (`#F2FF00`) top to bottom with white percent labels — the frame's arrangement minus the tooltip. Stop the dev server after confirming.

- [ ] **Step 7: Commit**

```bash
git add src/components src/state src/App.tsx src/main.tsx
git commit -m "m1/m2 notch shape + fixture rings 73/21/52 anchored to work area"
```

---

### Task 6: M0–M2 acceptance pass

**Files:**
- Create: `docs/TASKS-WIN.md`
- Modify: none (fix tasks go back through Tasks 2–5 as new commits)

**Interfaces:**
- Consumes: built app from Task 5
- Produces: verified M0/M1/M2 acceptance checklist + bugs log

- [ ] **Step 1: Production build**

Run: `pnpm tauri build` in `E:\status bar for ai\codenotch-win`
Expected: `Finished` + a signed-ability-noted unsigned installer under `src-tauri\target\release\bundle\` (SmartScreen will flag it; signing is M6, note it in TASKS-WIN.md rather than fixing it here).

- [ ] **Step 2: DPI sweep (the flush-gap check)**

Set Windows display scale to 100%, launch the built app, screenshot the notch-to-bezel join at 400% zoom; repeat at 125% and 150%.
Expected: no wallpaper hairline between notch edge and screen edge at any scale; no clipped flare. If a gap appears, the fix goes into `round_to_physical`/`anchorNotch` (Task 4), never into CSS fudge — record the scale factor and measured gap in `docs/TASKS-WIN.md`.

- [ ] **Step 3: Click-throughLibraries manual test**

With the app running, click on the desktop 2px outside the notch's curved flare, then drag-select across it.
Expected: clicks pass through to the desktop/icons behind (full `WS_EX_TRANSPARENT` region toggling is M6a; in this slice the transparent window plus exact-fit panel size must already avoid swallowing clicks outside the silhouette — if it does, shrink the panel to the drawn shape in `anchorNotch` and note it).

- [ ] **Step 4: Taskbar re-anchor**

Move the taskbar to another edge (or toggle auto-hide), wait 2s, observe the notch.
Expected (fixture slice): after app restart the notch sits flush on the new work area. Live re-anchor without restart is M1 polish; record the actual behavior observed in `docs/TASKS-WIN.md` rather than claiming more.

- [ ] **Step 5: Write the bugs log**

`docs/TASKS-WIN.md`:

```md
# Codenotch for Windows — Task Log

## M0–M2 fixture acceptance (2026-09-06)

- [ ] M0: process in Task Manager, no taskbar entry, no titlebar/border (light + dark)
- [ ] M1: flush against work-area edge at 100/125/150% DPI, no wallpaper hairline
- [ ] M1: clicks outside the flare pass through to the desktop
- [ ] M2: fixture 73/21/52 matches frame-124 colors, bands, ring direction
- [ ] Scale anchor verified: ring ~117px, body ~186px in frame-124-hover-tooltip.png

## Bugs worth remembering

- (append each fix with symptom → cause → what to check next time)
```

Check each box only against the observed build, never against an agent summary.

- [ ] **Step 6: Commit**

```bash
git add docs/TASKS-WIN.md
git commit -m "m0-m2 acceptance checklist + windows task log"
```

---

## Self-Review

1. **Spec coverage:** M0 shell → Task 2. M1 geometry/hit-region/re-anchor → Task 4 (+ Task 6 steps 2–4). M2 palette/scale/bands/rings/cells/fixtures → Tasks 3 + 5. Glyph deferral, 50/70 bands, scale sanity check, Zod deferral, no PlatformAdapter — all explicit in tasks or constraints. Rust install prerequisite → Task 1.
2. **Placeholder scan:** every step has exact file paths, complete code blocks, exact commands with expected output. The two deliberate observation steps (M0 run, visual frame compare) state exactly what to look at. `shadow: false` in Task 2: if the Tauri schema rejects it, drop that single key and note it in TASKS-WIN.md.
3. **Type consistency:** `WorkArea` (camelCase `scaleFactor` via serde rename) matches on Rust and TS sides. `round_to_physical` (Rust) and `roundToPhysical` (TS) share the 88px@125%/105px@150% expectations. `anchorNotch(work, cellCount, edge)` signature matches App usage and tests. `notchPath(width, height)` return shape matches its test. `FixtureCell{ id, initial, percent }` matches all three consuming components.
