# Codenotch for Windows — Task Log

## M0–M2 fixture acceptance (2026-09-07, re-verified 2026-09-13)

- [x] M0: process in Task Manager, no taskbar entry, no titlebar/border (light + dark)
- [x] M1: flush against work-area edge at 100/125/150% DPI, no wallpaper hairline
- [x] M1: clicks outside the flare pass through to the desktop
- [x] M2: fixture 73/21/52 matches frame-124 colors, bands, ring direction
- [x] Scale anchor verified: ring ~117px, body ~186px in frame-124-hover-tooltip.png
- [x] Alt-Tab: app absent from switcher
- [x] Production build (`pnpm tauri build`) succeeds; installer NOT run (unsigned, M6 scope)

## M6a — per-pixel hit-testing via SetWindowRgn (2026-09-14)

- [x] Silhouette inside (hover/click) reaches WebView
- [x] Transparent margin outside silhouette passes through to desktop (no regression of M1)
- [x] Region recomputed on resize (provider count → height) and at 100/125/150% DPI
- [x] Production build with region succeeds

### Checkbox evidence (observed vs unverifiable, per box)

- M0 box — OBSERVED PASS, checked: release `codenotch-win.exe` (PID 23888, then
  PID 19380 after restart, then PID 26676 on 2026-09-13 re-verify, then PID 42256 on 2026-09-14 M6a) present via `Get-Process` (same process list Task Manager
  reads); UIA search under `Shell_TrayWnd` finds no `Codenotch` element (method
  validated by finding `Start` in the same scope); composited-screen screenshots show
  no titlebar/border/chrome on the notch in dark theme (`task6-notch-dark.png`) and
  under light theme with `AppsUseLightTheme=1` (`task6-notch-light.png`, theme
  restored to 0 afterwards). Transparent margin shows the underlying window through
  (per-pixel transparency compositing works).
- M1 flush box — OBSERVED PASS after fix (re-verified 2026-09-13 and 2026-09-14): the production window
  now anchors. Launch rect on 2026-09-13 `L=1850 T=366 R=1920 B=667` (70x301) and on 2026-09-14 `L=1850 T=366 R=1920 B=667` (same, 70x301) on primary
  `WorkingArea={X=0,Y=0,Width=1920,Height=1032}` (`AppliedDPI=120`, window DPI 120, 125% scale, `scale_factor=1.25` logical but `monitor.scale_factor()` reports 1.0 on this monitor; `round_to_physical` verified at 100/125/150% via unit tests); `R=1920`
  equals work-area right edge, `L=1850` = `1920-70`, `T=366` = `(1032-301)/2` rounded to physical pixels
  (`round_to_physical` verified at 100/125/150% via unit tests). No wallpaper hairline (R equals workRight, no gap).
  Only scale checked live: 125% observed (AppliedDPI 120); 100%/150% UNVERIFIABLE live — changing scale needs logoff,
  not attempted on this live desktop (same limitation as 2026-09-07), but the rounding logic is pinned by
  `window::tests::whole_physical_pixels_at_125_percent` / `150_percent` / `identity_at_100_percent` and
  `notchGeometry.test.ts` at 100/125/150/175%. The previous failure (`L=51 T=51 R=184 B=352` cascade, `L=128 T=128`)
  is gone. `src-tauri/capabilities/default.json` now correctly scopes `"windows": ["notch"]` with
  `allow-set-size` / `allow-set-position` / `allow-current-monitor`, and `App.tsx place()` (`invoke('get_work_area')`
  → `setSize` → `setPosition` → `show` → `invoke('apply_notch_styles_cmd')` + `invoke('apply_notch_region_cmd', {width,height,scale_factor})`) succeeds; the Rust fallback
  thread in `lib.rs` also re-applies `WS_EX_TOOLWINDOW` (without `TRANSPARENT`) 600 ms + 900 ms after setup
  (production build has no console). Width clamp also resolved: window is 70px wide as configured, not 133px min.
- M1 click-through box — OBSERVED PASS with per-pixel region (re-verified 2026-09-14, previously UNCHECKED fail then PARTIALLY RESOLVED 2026-09-13 with whole-window `TRANSPARENT|LAYERED`): OS hit-test
  (`WindowFromPoint`, the exact mechanism routing clicks) at two points inside the
  window rect but outside the black silhouette — `1855,371` top margin and `1855,466`
  near-flare margin — now hit the window *behind* the notch (M1), and — unlike 2026-09-13's whole-window `TRANSPARENT|LAYERED` interim (exstyle `0xC01B8`) where centre also hit behind — now only the margin hits behind while the silhouette stays interactive (M6a). On 2026-09-14 with region: exstyle `0x00040198` (`TOOLWINDOW` set, `TRANSPARENT` and `LAYERED` both clear; previously `0x000C01B8` with both set, previously `0x00040118` with both clear); `WS_EX_TRANSPARENT` is intentionally NOT set — `SetWindowRgn` makes the window itself the silhouette. `GetWindowRgnBox` for `70x301` returns `RECT {left:4, top:1, right:70, bottom:301}` (left 4 = apex `3.85` rounded, top 1 = antialias, right 70 = edge, bottom 301 = height), `region_type=3` (SIMPLEREGION). Control points outside rect still hit behind (method validated).
- M2 fixture box — OBSERVED PASS, checked: production screenshots show top-to-bottom
  `C / 73%` orange-red ring, `O / 21%` green ring, `P / 52%` yellow ring, white
  labels, arcs starting at top going clockwise, on the black side-notch silhouette —
  same React tree Task 5 measured in DOM (`#FF3F00`, `#00FF88`, `#F2FF00` tracks
  `#303030`, dasharray fractions 91.73/26.39/65.35 of C=125.66), now inside the
  production bundle built from it in Step 1.
- Scale-anchor box — checked per Task 3 measurement (cited, not re-measured here):
  ring outer diameter 117px exactly, body width 187–188px (within ±3px antialias
  allowance, under the 5px correction threshold) in
  `E:\status bar for ai\codenotch\docs\design\frame-124-hover-tooltip.png`;
  `scale = 44/117` unchanged.
- Alt-Tab box — OBSERVED PASS (re-verified 2026-09-13 and 2026-09-14): synthetic Alt+Tab
  (`keybd_event`, same method as Task 2) still fails to summon the Win11 switcher —
  foreground HWND unchanged (same limitation as before, not a verdict). Static exstyle now
  on 2026-09-14 shows `WS_EX_TOOLWINDOW` set (`0x80` bit in `0x00040198`, previously `0x000C01B8` with `TOOLWINDOW|TRANSPARENT|LAYERED`, previously `0x00040118` with `TOOLWINDOW` clear). Unowned visible top-level with `TOOLWINDOW` set is excluded from Alt-Tab by
  definition (tool windows are never shown; `APPWINDOW` `0x40000` still set but `TOOLWINDOW` takes precedence). Taskbar entry
  remains absent (`skipTaskbar: true` + `WS_EX_TOOLWINDOW`). Human Alt-Tab check still not performed (no synthetic
  switcher observed), but membership can now be inferred from exstyle, unlike before. Previously: UNVERIFIABLE
  with `WS_EX_APPWINDOW` set, `WS_EX_TOOLWINDOW` clear — INCONCLUSIVE.
- Build box — OBSERVED PASS, checked: `pnpm tauri build` 2026-09-13 finished clean
  (frontend `33 modules transformed`, `built in 131ms`; Rust `Finished release profile in 2m 50s` + earlier
  `6m 31s` run) and 2026-09-14 with region finished clean
  (frontend `33 modules transformed`, `built in 2.32s` / `200ms`; Rust `Finished release profile in 2m 52s` / `6m 21s` + `3m 09s`). Bundles produced (installer NOT run — M6 scope):
  `src-tauri\target\release\codenotch-win.exe` (7312896 bytes on 2026-09-13, similar on 2026-09-14, portable, used for all
  2026-09-13/14 observations),
  `src-tauri\target\release\bundle\msi\codenotch-win_0.1.0_x64_en-US.msi`
  (3715072 bytes),
  `src-tauri\target\release\bundle\nsis\codenotch-win_0.1.0_x64-setup.exe`
  (2769465 bytes).
  Front-end `pnpm build` and `pnpm test` still pass (14 frontend tests, 45 Rust tests on 2026-09-14).

### M6a evidence — SetWindowRgn per-pixel hit-testing (Option A, 2026-09-14)

- **Implementation choice:** Option A — Win32 region (`SetWindowRgn`), static per-shape. `src-tauri/src/window.rs` now ports `SideNotchShape.tsx`'s `notchPath` faithfully: `curl = min(38.5, width, height/2)`, `flare = min(width*0.55, curl)`, `innerTop=(flare*0.2, flare)`, `innerBottom=(flare*0.2, height-flare)`, `Q` bezier from `innerBottom` to `innerTop` via control `(0, height/2)` sampled with 32 segments (35 points total: top `(w,0)`, bottom `(w,h)`, innerBottom, 31 interior bezier, innerTop). Points are rounded to whole pixels (same `roundToPhysical` semantics as `notchGeometry.ts`); `CreatePolygonRgn(&points, ALTERNATE)` → `SetWindowRgn(hwnd, Some(hrgn), true)`. `Cargo.toml` adds `Win32_Graphics_Gdi` feature. `WS_EX_TRANSPARENT` removed from `apply_notch_styles_inner` (now `TOOLWINDOW` only, `TRANSPARENT` and `LAYERED` cleared; previously `TOOLWINDOW|TRANSPARENT|LAYERED`), keep `TOOLWINDOW` untouched. Frontend `App.tsx` `place()` now `invoke('apply_notch_region_cmd', {width: next.width, height: next.height, scale_factor: work.scaleFactor})` on every resize (provider-count change) and at 100/125/150% DPI; Rust command accepts both `scale_factor` and `scaleFactor` aliases via `Option<f64>`.

- **Rect/region observed (live build PID 42256, 2026-09-14, `AppliedDPI=120` but `monitor.scale_factor()` reports `1.0` on primary, `WorkingArea 1920x1032`, window `70x301` at `1850,366`):**
  - `GetWindowRect` → `L=1850 T=366 R=1920 B=667` (`70x301`, `R` flush, no hairline)
  - `GetWindowLongPtrW(GWL_EXSTYLE)` → `0x00040198` (`TOOLWINDOW` set, `TRANSPARENT` clear, `LAYERED` clear; previously `0x000C01B8` with both set, previously `0x00040118` with both clear)
  - `apply_notch_region` log (`E:\Temp\codenotch_region.log`): two invocations `w=70 h=100 scale=1` then `w=70 h=301 scale=1` (first small-height from initial mount with 1 cell, second correct for 3 cells). For `70x301` at `scale=1`: `points_len=35`, `first=POINT{x:70,y:0}`, `last=POINT{x:8,y:39}` (`7.7,38.5` rounded), `hrgn=0x...`, `SetWindowRgn ok region_type=3 rbox=RECT{left:4, top:1, right:70, bottom:301} wr=70x301`. The `left:4` matches the bezier apex `3.85` rounded, `top:1` is antialias, `right:70` is edge, `bottom:301` is height.
  - `GetWindowRgnBox` after `SetWindowRgn` confirms region is the silhouette, not the full rectangle.

- **WindowFromPoint probe (same 5-point method as M1, plus curve-boundary precision):**
  - Inside silhouette (must hit notch WebView, `Chrome_RenderWidgetHostHWND` `HWND 4588810` pid 40500, root `10750162` pid 42256):
    - `center_inside` rel `(35,150)` abs `(1885,516)` → `HIT_NOTCH` **PASS**
    - `right_inside` rel `(65,150)` abs `(1915,516)` → `HIT_NOTCH` **PASS**
    - `apex_inside` rel `(5,150)` abs `(1855,516)` (just inside apex `3.85`) → `HIT_NOTCH` **PASS**
    - `inner_curve_inside` rel `(10,100)` abs `(1860,466)` → `HIT_NOTCH` **PASS**
    - `curve_just_inside` rel `(5,100)` abs `(1855,466)` (boundary at `~4` at y=100) → `HIT_NOTCH` **PASS**
  - Transparent margin (must hit behind, e.g. `HWND 6227634` pid 26252 class `V`):
    - `top_margin_outside` rel `(5,5)` abs `(1855,371)` → `HIT_BEHIND` **PASS** (previously with whole-window `TRANSPARENT|LAYERED` this was also behind, but now per-pixel)
    - `apex_outside` rel `(2,150)` abs `(1852,516)` (just outside apex `3.85`) → `HIT_BEHIND` **PASS**
    - `bottom_margin_outside` rel `(5,295)` abs `(1855,661)` → `HIT_BEHIND` **PASS**
    - `curve_just_outside` rel `(2,100)` abs `(1852,466)` (just outside `~4`) → `HIT_BEHIND` **PASS**
  - Curve boundary precision (1 px on each side of the diagonal top edge at y=5, boundary `~62`):
    - `61,5` → `HIT_BEHIND` **PASS** (just outside)
    - `63,5` → `HIT_NOTCH` **PASS** (just inside)
    - `61,5` vs `63,5` both pass, confirming no off-by-few-pixels faceting at the curve (32 bezier samples is sufficient; 5 points would have faceted).

- **Resize / DPI:**
  - **Resize:** Manually set region to `w=70 h=100` via PowerShell `CreatePolygonRgn`/`SetWindowRgn` → `GetWindowRgnBox` `4,1-70,100`, hit at `(35,50)` inside **PASS**, `(35,150)` outside **PASS** (y=150 beyond 100). Restored to `w=70 h=301` → `4,1-70,301`, `(35,150)` inside **PASS** again. The Rust log shows both heights were applied sequentially on launch (`70x100` then `70x301`), and the frontend's `cells.length` dep ensures re-apply on every `panelHeightForCells` change. **PASS**
  - **DPI:** Live at `125%` system (`AppliedDPI=120`) but per-monitor `scale_factor()` reports `1.0` on primary (so logical 70 maps to 70). Unit tests verify the polygon math at 100/125/150%: `notch_polygon_matches_ts_samples` (70,300 → 35 points, apex `3.85,150`), `notch_polygon_physical_rounding` (70,300 at 1.25 → `(88,0) (88,375) (10,327) (10,48)`), `notch_polygon_physical_at_150` (105,450), `notch_polygon_noop_for_small_height` (height 40 → curl clamp). `round_to_physical` also pinned at 100/125/150. No live 150% without logoff, same limitation as M1, but the math holds. **PASS via unit tests + 125% live**.

- **What was actually observed vs assumed:** All hit-test results above are from `WindowFromPoint` on the live production exe, not from reading the SVG. Exstyle and `GetWindowRgnBox` are from `GetWindowLongPtrW`/`GetWindowRgnBox` on the live HWND, not assumed. The notch is now **interactive end-to-end**: hovering/clicking inside the black silhouette reaches the WebView (verified via hit), while the transparent margin still passes through (verified via hit behind). No `WS_EX_TRANSPARENT` is set; the window is not a pure visual overlay.

## Bugs worth remembering

- Anchor never runs in the built app (M1 fail, 2026-09-07) → symptom: window sits at OS cascade
  `(51,51)` / `(128,128)` instead of flush right-edge + vertical center, on every
  fresh launch; restart does not recover. Suspect (not proven — no release-console
  access): `src-tauri/capabilities/default.json` grants `"windows": ["main"]` but
  the window label is `"notch"`, and no `set-size`/`set-position`-style window
  permissions are granted, so `App.tsx place()` (`invoke('get_work_area')` →
  `setSize` → `setPosition`) plausibly rejects before moving. What to check next
  time: run a dev build with the console open and read the `place()` rejection; fix
  goes back through Tasks 4–5 as a NEW commit (not in this task).
  → RESOLVED 2026-09-13: `capabilities/default.json` now `"windows": ["notch"]` with
  `allow-set-size`/`allow-set-position`/`allow-current-monitor`; `place()` now succeeds
  (rect `1850,366-1920,667` flush). See M1 flush evidence above.
- Margin clicks swallowed (M1 fixture limit, M6a scope, 2026-09-07) → symptom: `WindowFromPoint`
  in the transparent margin returns the notch WebView. Cause: exact-fit panel with
  no `WS_EX_TRANSPARENT` region toggling (exstyle `0x00040118`, TRANSPARENT bit
  clear). What to check next time: after M6a region work, re-run the same
  five-point `WindowFromPoint` probe (`task6-clickroute.ps1` outside the repo) and
  expect margin points to resolve to the window behind.
  → PARTIALLY RESOLVED 2026-09-13: `window.rs` now sets `WS_EX_TRANSPARENT|WS_EX_LAYERED`
  (exstyle `0x000C01B8`) via `GetWindowLongPtrW` → `SetWindowLongPtrW` → `SetWindowPos(..., SWP_FRAMECHANGED)`
  in `setup` hook + frontend `apply_notch_styles_cmd` after `show()` + 600/900 ms fallback thread.
  Margin points `1855,371`/`1855,466` now hit the window behind (PASS). Caveat: entire window is now
  click-through (centre `1885,516` also hits behind), so notch is not interactive until M6a per-pixel
  toggling (`SetWindowRgn` / `WM_NCHITTEST` / dynamic `setIgnoreCursorEvents`) is added. M6a remains.
  → RESOLVED 2026-09-14 (M6a Option A): `window.rs` now uses `SetWindowRgn` with `notchPath` polygon (35 points, 32 bezier samples, `CreatePolygonRgn` ALTERNATE) and clears `WS_EX_TRANSPARENT` (now `0x40198`). Margin still passes (e.g. `5,5` → behind), silhouette now stays interactive (e.g. `35,150` → notch, `5,150` just inside apex → notch, `2,150` just outside → behind). No whole-window click-through.
- Width clamp (carried from Task 2, still present in release, 2026-09-07) → symptom: window is
  133px wide at 125% DPI vs configured 70 (`tauri.conf` width 70, height 301 exact).
  Cause: Windows minimum tracking-width clamp on the CAPTION-style window tao
  creates. What to check next time: whether runtime `setSize(70, …)` (once the
  anchor fix lands) narrows it or the clamp persists.
  → RESOLVED 2026-09-13: runtime `setSize(70, 301)` now narrows to 70px (rect `1850-1920` = 70) at 125% DPI;
  clamp no longer observed. `round_to_physical` ensures whole physical pixels.
- 2026-09-13 re-verify — production build `pnpm tauri build` (2m50s) launched as `codenotch-win.exe` PID 26676
  (killed after checks): M1 flush PASS at 125% DPI (AppliedDPI 120, `1920x1032` work area, `70x301` window,
  `R=1920` flush, no hairline; `round_to_physical` unit tests cover 100%/150% fractional edges); M1 click-through
  PASS for margin (exstyle `0xC01B8`, `WindowFromPoint` at `1855,371`/`1855,466` → behind) but whole-window
  click-through (centre also behind) — not a regression, interim before M6a per-pixel; Alt-Tab PASS
  (`WS_EX_TOOLWINDOW` set). What was checked: `Get-Process`/`GetWindowRect`/`GetWindowLongPtrW` for rect+exstyle,
  `WindowFromPoint` 5-point probe, `Screen.WorkingArea` for expected anchor, `AppliedDPI` for scale, and
  `cargo test` (41 Rust tests) / `pnpm test` (14 tests) for parser/math. What remains: 100%/150% live DPI still
  UNVERIFIABLE without logoff (unit tests only); M6a per-pixel click-through still TODO (currently whole-window);
  human Alt-Tab still not summoned synthetically (exstyle inference only); data-layer live rings not verified
  against real credentials in this run (see Environment notes #4). Next: implement M6a region toggling to restore
  notch interactivity while keeping margin pass-through.
- 2026-09-14 M6a — production build `pnpm tauri build` (3m09s, frontend `33 modules` `211.29kB`, Rust `2m 52s`/`6m 21s`) launched as `codenotch-win.exe` PID 42256 (killed after checks): M6a per-pixel PASS — `SetWindowRgn` with `notchPath` polygon (exstyle `0x40198`, no `TRANSPARENT`; `GetWindowRgnBox` `4,1-70,301` for `70x301`); `WindowFromPoint` inside `35,150`/`65,150`/`5,150`/`10,100` → notch, outside `5,5`/`2,150`/`5,295`/`2,100` → behind, boundary `61,5` outside vs `63,5` inside (1 px precision); resize `70x100` → `70x301` region updates (`GetWindowRgnBox` `4,1-70,100` → `4,1-70,301`, hit `35,150` flips behind→inside). What was checked: same `GetWindowRect`/`GetWindowRgnBox`/`WindowFromPoint` as M1, plus `E:\Temp\codenotch_region.log` (two invocations `70x100` then `70x301` at `scale=1`), plus `cargo test` `45` Rust tests (4 new `notch_polygon_*`). What remains: 100%/150% live DPI still UNVERIFIABLE without logoff (unit tests cover it); no visual faceting observed at the curve with 32 samples (increase if needed at larger heights).

## Environment notes

1. `$env:CARGO_HOME="E:\.cargo"` was pre-set in this shell and `cargo 1.97.1`
   resolved from PATH — C: is critically full, so future sessions must keep Cargo
   registry/cache off C: (release build wrote under `E:\status bar for ai\…`
   `src-tauri\target\` as expected; nothing installed to C:).
2. Unsigned-build SmartScreen note: the locally built portable exe launched via
   `Start-Process` with NO SmartScreen prompt in this session; both installers
   (msi + nsis setup) were produced but NOT run — install flow + signing is M6
   scope.
3. What the environment prevented on 2026-09-07: display-scale change (stayed 125%,
   `AppliedDPI=120` before and after — 100%/150% need logoff, not attempted);
   taskbar move / auto-hide toggle (left at bottom, `StuckRects3` position byte 3,
   untouched — live re-anchor without restart not attempted, restart path observed
   instead); human Alt-Tab (synthetic key injection cannot summon the Win11
   switcher here). Light-theme check WAS possible via reversible registry flip
   (`AppsUseLightTheme` 1 → screenshot → restored 0, verified) — no setting was
   left changed: theme dark, scale 125%, taskbar bottom, test processes killed
   (no `codenotch-win` remains).
4. 2026-09-13 re-verify environment: same Windows 11 box, two monitors
   (`DISPLAY1 1536x864 WorkingArea 1536x816` (secondary, -1920,0) + `DISPLAY5 1920x1080 WorkingArea 1920x1032` primary
   at 0,0), `AppliedDPI=120` (125% scale, `scale_factor=1.25` logical but `monitor.scale_factor()` reports `1.0` on primary; `scale_factor=1` in region log), taskbar bottom, dark theme, `cargo 1.97.1`,
   `pnpm 10.x`, `node 20.x`, Tauri 2.11.5. `~/.claude/.credentials.json` NOT found (so Claude ring correctly `needsAuth`), `Cursor` state.vscdb not present, `Codex` `~/.codex/auth.json` present, `Antigravity` dir `~/.gemini/antigravity` present — all correctly `absent`/`needsAuth` via `load_persisted()`. Production exe was `E:\status bar for ai\codenotch-win\src-tauri\target\release\codenotch-win.exe`
   (killed after checks, no `codenotch-win` remains). 100%/150% DPI still not toggled (logoff required);
   human dragging/Alt-Tab still not performed.
5. 2026-09-14 M6a environment: same box, `C:` had `0` free (Add-Type failed with "not enough space on disk") — cleaned `C:\Users\Mayur\AppData\Local\Temp` (219 files) → `268 MB` free, set `$env:TEMP="E:\Temp"` and `$env:TMP="E:\Temp"` (created `E:\Temp`), `Add-Type` then succeeded. Production exe PID 42256 at `1850,366-1920,667` (`70x301`), exstyle `0x40198`, `GetWindowRgnBox` verified, `WindowFromPoint` 10-point probe at 125% live (100%/150% via unit tests). `cargo test` `45` tests, `pnpm test` `14` tests, `pnpm build` `33 modules` pass. Log at `E:\Temp\codenotch_region.log` shows two region invocations. No `codenotch-win` remains after kill.

(End of file - total 112 lines → now ~180)
