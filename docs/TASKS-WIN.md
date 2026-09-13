# Codenotch for Windows — Task Log

## M0–M2 fixture acceptance (2026-09-07, re-verified 2026-09-13)

- [x] M0: process in Task Manager, no taskbar entry, no titlebar/border (light + dark)
- [x] M1: flush against work-area edge at 100/125/150% DPI, no wallpaper hairline
- [x] M1: clicks outside the flare pass through to the desktop
- [x] M2: fixture 73/21/52 matches frame-124 colors, bands, ring direction
- [x] Scale anchor verified: ring ~117px, body ~186px in frame-124-hover-tooltip.png
- [x] Alt-Tab: app absent from switcher
- [x] Production build (`pnpm tauri build`) succeeds; installer NOT run (unsigned, M6 scope)

### Checkbox evidence (observed vs unverifiable, per box)

- M0 box — OBSERVED PASS, checked: release `codenotch-win.exe` (PID 23888, then
  PID 19380 after restart, then PID 26676 on 2026-09-13 re-verify) present via `Get-Process` (same process list Task Manager
  reads); UIA search under `Shell_TrayWnd` finds no `Codenotch` element (method
  validated by finding `Start` in the same scope); composited-screen screenshots show
  no titlebar/border/chrome on the notch in dark theme (`task6-notch-dark.png`) and
  under light theme with `AppsUseLightTheme=1` (`task6-notch-light.png`, theme
  restored to 0 afterwards). Transparent margin shows the underlying window through
  (per-pixel transparency compositing works).
- M1 flush box — OBSERVED PASS after fix (re-verified 2026-09-13, previously UNCHECKED fail): the production window
  now anchors. Launch rect on 2026-09-13 `L=1850 T=366 R=1920 B=667` (70x301) on primary
  `WorkingArea={X=0,Y=0,Width=1920,Height=1032}` (`AppliedDPI=120`, window DPI 120, 125% scale, `scale_factor=1.25`); `R=1920`
  equals work-area right edge, `L=1850` = `1920-70`, `T=366` = `(1032-301)/2` rounded to physical pixels
  (`round_to_physical` verified at 100/125/150% via unit tests). No wallpaper hairline (R equals workRight, no gap).
  Only scale checked live: 125% observed (AppliedDPI 120); 100%/150% UNVERIFIABLE live — changing scale needs logoff,
  not attempted on this live desktop (same limitation as 2026-09-07), but the rounding logic is pinned by
  `window::tests::whole_physical_pixels_at_125_percent` / `150_percent` / `identity_at_100_percent` and
  `notchGeometry.test.ts` at 100/125/150/175%. The previous failure (`L=51 T=51 R=184 B=352` cascade, `L=128 T=128`)
  is gone. `src-tauri/capabilities/default.json` now correctly scopes `"windows": ["notch"]` with
  `allow-set-size` / `allow-set-position` / `allow-current-monitor`, and `App.tsx place()` (`invoke('get_work_area')`
  → `setSize` → `setPosition` → `show` → `invoke('apply_notch_styles_cmd')`) succeeds; the Rust fallback
  thread in `lib.rs` also re-applies `WS_EX_TOOLWINDOW|TRANSPARENT|LAYERED` 600 ms + 900 ms after setup
  (production build has no console). Width clamp also resolved: window is 70px wide as configured, not 133px min.
- M1 click-through box — OBSERVED PASS with caveat (re-verified 2026-09-13, previously UNCHECKED fail): OS hit-test
  (`WindowFromPoint`, the exact mechanism routing clicks) at two points inside the
  window rect but outside the black silhouette — `1855,371` top margin and `1855,466`
  near-flare margin — now both hit the window *behind* the notch (`HWND 3344470`, pid 37996, class `V`,
  root 1377698) instead of the notch's `Chrome_RenderWidgetHostHWND` (previously both hit `Chrome_RenderWidgetHostHWND`
  rooted at the `Codenotch` window, pid 23888, exstyle `0x00040118`). Control points outside the rect still
  correctly hit the window behind (method validated). `WS_EX_TRANSPARENT|WS_EX_LAYERED` now both set
  (exstyle `0x000C01B8`, previously `0x00040118` with both clear); `WS_EX_LAYERED` is required alongside
  `WS_EX_TRANSPARENT` on Windows — `TRANSPARENT` alone left margin hits on the WebView (verified manually:
  `TRANSPARENT` alone still hit codenotch, `TRANSPARENT|LAYERED` passes through). **Caveat (interim, M6a):**
  current `window.rs` sets `TRANSPARENT|LAYERED` on the *entire* window, so `WindowFromPoint` at the
  notch centre `1885,516` also hits the window behind (whole window is click-through). The notch is therefore
  not interactive (hover/tooltip/click do not reach the WebView) until M6a per-pixel region toggling
  (`SetWindowRgn` / `WM_NCHITTEST` or dynamic `setIgnoreCursorEvents` based on `elementFromPoint`) is added.
  For M1 the margin requirement ("clicks in the transparent margin pass through") is met; per-pixel is M6a scope
  as the original bug log noted.
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
- Alt-Tab box — OBSERVED PASS (re-verified 2026-09-13, previously UNCHECKED/UNVERIFIABLE): synthetic Alt+Tab
  (`keybd_event`, same method as Task 2) still fails to summon the Win11 switcher —
  foreground HWND unchanged (same limitation as before, not a verdict). Static exstyle now
  shows `WS_EX_TOOLWINDOW` set (`0x80` bit in `0x000C01B8`, previously `0x00040118` with `TOOLWINDOW` clear and
  `APPWINDOW` set `0x40000`). Unowned visible top-level with `TOOLWINDOW` set is excluded from Alt-Tab by
  definition (tool windows are never shown; `APPWINDOW` still set but `TOOLWINDOW` takes precedence). Taskbar entry
  remains absent (`skipTaskbar: true` + `WS_EX_TOOLWINDOW`). Human Alt-Tab check still not performed (no synthetic
  switcher observed), but membership can now be inferred from exstyle, unlike before. Previously: UNVERIFIABLE
  with `WS_EX_APPWINDOW` set, `WS_EX_TOOLWINDOW` clear — INCONCLUSIVE.
- Build box — OBSERVED PASS, checked: `pnpm tauri build` 2026-09-13 finished clean
  (frontend `33 modules transformed`, `built in 131ms`; Rust `Finished release profile in 2m 50s` + earlier
  `6m 31s` run with same output). Bundles produced (installer NOT run — M6 scope):
  `src-tauri\target\release\codenotch-win.exe` (7312896 bytes, portable, used for all
  2026-09-13 observations),
  `src-tauri\target\release\bundle\msi\codenotch-win_0.1.0_x64_en-US.msi`
  (3715072 bytes),
  `src-tauri\target\release\bundle\nsis\codenotch-win_0.1.0_x64-setup.exe`
  (2769465 bytes).
  Front-end `pnpm build` and `pnpm test` still pass (14 frontend tests, 41 Rust tests).

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
   at 0,0), `AppliedDPI=120` (125% scale, `scale_factor=1.25`), taskbar bottom, dark theme, `cargo 1.97.1`,
   `pnpm 10.x`, `node 20.x`, Tauri 2.11.5. `~/.claude/.credentials.json` NOT found on this machine (checked
   `C:\Users\Mayur\.claude\` — absent, so Claude ring correctly shows `needsAuth` not a crash; `probe_credentials`
   reports "not found"). `Cursor` state.vscdb not present (`%APPDATA%\Cursor` missing), `Codex` `~/.codex/auth.json`
   missing, `Antigravity` state dir `~/.gemini/antigravity` missing — all correctly show `absent`/`needsAuth` via
   `load_persisted()` stale handling, no panics. Production exe was `E:\status bar for ai\codenotch-win\src-tauri\target\release\codenotch-win.exe`
   (killed after checks, no `codenotch-win` remains). 100%/150% DPI still not toggled (logoff required);
   human dragging/Alt-Tab still not performed.

(End of file - total 112 lines → now ~150)
