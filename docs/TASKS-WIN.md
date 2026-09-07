# Codenotch for Windows — Task Log

## M0–M2 fixture acceptance (2026-09-07)

- [x] M0: process in Task Manager, no taskbar entry, no titlebar/border (light + dark)
- [ ] M1: flush against work-area edge at 100/125/150% DPI, no wallpaper hairline
- [ ] M1: clicks outside the flare pass through to the desktop
- [x] M2: fixture 73/21/52 matches frame-124 colors, bands, ring direction
- [x] Scale anchor verified: ring ~117px, body ~186px in frame-124-hover-tooltip.png
- [ ] Alt-Tab: app absent from switcher
- [x] Production build (`pnpm tauri build`) succeeds; installer NOT run (unsigned, M6 scope)

### Checkbox evidence (observed vs unverifiable, per box)

- M0 box — OBSERVED PASS, checked: release `codenotch-win.exe` (PID 23888, then
  PID 19380 after restart) present via `Get-Process` (same process list Task Manager
  reads); UIA search under `Shell_TrayWnd` finds no `Codenotch` element (method
  validated by finding `Start` in the same scope); composited-screen screenshots show
  no titlebar/border/chrome on the notch in dark theme (`task6-notch-dark.png`) and
  under light theme with `AppsUseLightTheme=1` (`task6-notch-light.png`, theme
  restored to 0 afterwards). Transparent margin shows the underlying window through
  (per-pixel transparency compositing works).
- M1 flush box — UNCHECKED (observed fail, logged not fixed): the production window
  never anchors. First launch rect `L=51 T=51 R=184 B=352` (133x301); after kill +
  relaunch `L=128 T=128 R=261 B=429` — OS default cascade positions, never flush to
  the work-area right edge (expected `R=1536`, vertically centered). Restart
  re-anchor therefore also fails deterministically. Only scale checked: 125%
  (`AppliedDPI=120`, window DPI 120); 100%/150% UNVERIFIABLE — changing scale needs
  logoff, not attempted on this live desktop. The notch-to-bezel hairline check is
  blocked by the anchor failure (window floats mid-desktop, no edge join exists to
  inspect). No clipped flare observed (all three rings fully visible). Follow-up
  through Tasks 4–5 (see Bugs).
- M1 click-through box — UNCHECKED (observed fail, logged not fixed): OS hit-test
  (`WindowFromPoint`, the exact mechanism routing clicks) at two points inside the
  window rect but outside the black silhouette — `(56,61)` top margin and `(56,201)`
  near-flare margin — both hit `Chrome_RenderWidgetHostHWND` rooted at the
  `Codenotch` window (PID 23888): margin clicks are swallowed by the app, not passed
  to the desktop. Control points outside the rect correctly hit the Telegram window
  behind (method validated). `WS_EX_TRANSPARENT` confirmed absent
  (exstyle `0x00040118`). No separate synthetic drag performed (no selection probe in
  the region); the mousedown hit-test above governs drag initiation too. M6a
  follow-up (full region toggling is M6a scope).
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
- Alt-Tab box — UNCHECKED, UNVERIFIABLE with reason: synthetic Alt+Tab
  (`keybd_event`, same method as Task 2) again fails to summon the Win11 switcher —
  foreground HWND unchanged (`198236` → `198236`), fullscreen screenshot shows no
  switcher UI. Static context only (not a verdict): unowned visible top-level,
  `WS_EX_APPWINDOW` set, `WS_EX_TOOLWINDOW` clear, taskbar entry removed via
  `DeleteTab` — membership cannot be inferred from styles. Needs a human Alt+Tab
  check on real hardware.
- Build box — OBSERVED PASS, checked: `pnpm tauri build` finished clean
  (`Finished release profile in 4m 01s`; frontend `32 modules transformed`,
  `built in 107ms`). Bundles produced (installer NOT run — M6 scope):
  `src-tauri\target\release\codenotch-win.exe` (portable, used for all
  observations),
  `src-tauri\target\release\bundle\msi\codenotch-win_0.1.0_x64_en-US.msi`
  (2056192 bytes),
  `src-tauri\target\release\bundle\nsis\codenotch-win_0.1.0_x64-setup.exe`
  (1402602 bytes).

## Bugs worth remembering

- Anchor never runs in the built app (M1 fail) → symptom: window sits at OS cascade
  `(51,51)` / `(128,128)` instead of flush right-edge + vertical center, on every
  fresh launch; restart does not recover. Suspect (not proven — no release-console
  access): `src-tauri/capabilities/default.json` grants `"windows": ["main"]` but
  the window label is `"notch"`, and no `set-size`/`set-position`-style window
  permissions are granted, so `App.tsx place()` (`invoke('get_work_area')` →
  `setSize` → `setPosition`) plausibly rejects before moving. What to check next
  time: run a dev build with the console open and read the `place()` rejection; fix
  goes back through Tasks 4–5 as a NEW commit (not in this task).
- Margin clicks swallowed (M1 fixture limit, M6a scope) → symptom: `WindowFromPoint`
  in the transparent margin returns the notch WebView. Cause: exact-fit panel with
  no `WS_EX_TRANSPARENT` region toggling (exstyle `0x00040118`, TRANSPARENT bit
  clear). What to check next time: after M6a region work, re-run the same
  five-point `WindowFromPoint` probe (`task6-clickroute.ps1` outside the repo) and
  expect margin points to resolve to the window behind.
- Width clamp (carried from Task 2, still present in release) → symptom: window is
  133px wide at 125% DPI vs configured 70 (`tauri.conf` width 70, height 301 exact).
  Cause: Windows minimum tracking-width clamp on the CAPTION-style window tao
  creates. What to check next time: whether runtime `setSize(70, …)` (once the
  anchor fix lands) narrows it or the clamp persists.

## Environment notes

1. `$env:CARGO_HOME="E:\.cargo"` was pre-set in this shell and `cargo 1.97.1`
   resolved from PATH — C: is critically full, so future sessions must keep Cargo
   registry/cache off C: (release build wrote under `E:\status bar for ai\…`
   `src-tauri\target\` as expected; nothing installed to C:).
2. Unsigned-build SmartScreen note: the locally built portable exe launched via
   `Start-Process` with NO SmartScreen prompt in this session; both installers
   (msi + nsis setup) were produced but NOT run — install flow + signing is M6
   scope.
3. What the environment prevented: display-scale change (stayed 125%,
   `AppliedDPI=120` before and after — 100%/150% need logoff, not attempted);
   taskbar move / auto-hide toggle (left at bottom, `StuckRects3` position byte 3,
   untouched — live re-anchor without restart not attempted, restart path observed
   instead); human Alt-Tab (synthetic key injection cannot summon the Win11
   switcher here). Light-theme check WAS possible via reversible registry flip
   (`AppsUseLightTheme` 1 → screenshot → restored 0, verified) — no setting was
   left changed: theme dark, scale 125%, taskbar bottom, test processes killed
   (no `codenotch-win` remains).
