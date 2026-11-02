# Windows validation

## Build

Use Windows 10/11, Node 24 LTS, pnpm 10, stable Rust (at least 1.88), Visual Studio C++ Build Tools with Windows SDK, and WebView2 Runtime. In the repository root:

```powershell
pnpm install --frozen-lockfile
pnpm test
pnpm build
pnpm exec playwright install chromium
pnpm test:smoke
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

The GitHub Actions **CI & Build Verification** workflow performs these checks on `windows-latest`, supports manual dispatch, and uploads `codenotch-windows-installers` containing NSIS and MSI installers. It must be run on the updated branch before treating native compilation or installers as verified. Installers are not code signed by this workflow.

## Interactive release checks

- Open the Desk and verify all five widgets render. Drag by the header; pin/unpin; close and reopen.
- Add a project and task, complete it, restart, and verify persistence. Delete a task; confirm and cancel project deletion. Unreadable stored data must remain untouched until the user chooses to discard it.
- With each supported provider signed in locally, check actual usage and reset timestamps. Test absent/expired credentials, refresh errors and saved stale readings. Antigravity request counts must not become percentages.
- Load a valid public GitHub username, refresh, switch usernames, and try an invalid/missing user and offline network. Show errors rather than zero or another account's data. No GitHub token should be needed.
- Launch installed Chrome, Discord, Spotify and OBS. Test missing apps. Confirm OBS starts with its executable directory as its working directory.
- Switch Desk → compact notch → Desk repeatedly, including while usage events arrive. The expanded Desk must never keep the notch-shaped region.
- Check single and multiple monitors at 100%, 125%, 150%, 175% and 200% scale, negative monitor origins, small work areas and moved taskbars. The Desk must fit the selected monitor. Recheck compact click-through, interactive silhouette and edge placement.
- Use only the keyboard for task forms, project/provider selection, quotes and window controls. Check visible focus and reduced motion.
- Install each MSI/NSIS artifact in a clean Windows user session, start it, repeat a basic task/launch/mode smoke check, and uninstall.

## Current evidence

Development was performed on Linux. Frontend tests/build, browser interactions and standalone pure Rust parser/launcher-routing/window-bounds tests can run here. These do not validate Win32 APIs, Tauri/MSVC integration, actual app launching, provider credential access or Windows installers. Record a Windows CI run and the manual results before release.

## Results from this implementation

- Vitest: 35 tests passed across 7 files.
- Playwright/Chromium: 7 tests passed, covering task persistence/project isolation, native-only action messaging, corrupt/full storage, silhouette hit geometry, failed expansion recovery, and pin-state preservation. Native-command failure tests simulate only the IPC boundary; they do not execute Win32 APIs.
- TypeScript and Vite production build: passed.
- `cargo fmt --check`: passed.
- Standalone Rust core tests: 6 passed (3 calendar, 2 launcher routing, 1 window-bounds test).
- The calendar parser also read a real public GitHub response: 366 days and 1,075 contributions for the requested repository owner's public profile at test time.
- Full Linux `cargo test --locked --no-default-features`: blocked before application compilation by missing system `glib-2.0` development files. This is an environment blocker, not a passing backend integration run.
- Windows/MSVC CI, installers and interactive native checks: pending.

## Upstream native integration checks

- Close the Desk: the process remains in the tray. Tray left-click and Open Desk restore it from compact mode; tray Refresh requests provider reads; tray Quit exits.
- Launch a second copy while the first is hidden or compact: one process/Desk remains.
- Enable Start with Windows, confirm the per-user `LiveNotchTracker` Run entry points to this executable with `--background`, and confirm sign-in starts hidden with a recoverable tray icon. Disable the setting and verify its entry is removed. Disable it before uninstalling; it uses its own name and does not alter upstream Codenotch's startup entry.
- With a managed Codex desktop sign-in and an unusable HTTP credential file, native quota recovery queries only `initialize`, `initialized`, `account/rateLimits/read`, with a 20-second deadline and owned-process cleanup. Verify a 429 does not launch recovery; failed native reads stand down for 30 minutes.
- Standalone core tests verify quota bucket selection, malformed/missing values, bounds, backoff eligibility, native executable extensions and startup command quoting without reading local credentials or changing startup settings.
- While Claude is under a real Retry-After, tray and UI Refresh must wake the poller without issuing a request before the persisted deadline.
- Close a compact window to tray; change provider availability or DPI and verify it stays hidden until explicitly opened.

Latest local integration checks: 35 frontend tests, 9 browser regressions and 12 Rust core tests pass. The core harness also passes Clippy with warnings denied. These do not replace the native Windows checklist above.
