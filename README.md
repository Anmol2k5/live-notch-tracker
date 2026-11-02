# Live Notch Tracker (Codenotch for Windows)

A lightweight, hardware-accelerated Windows desktop status overlay for real-time AI quota tracking. Sits flush against the screen margin as an elegant side-notch, displaying live usage statistics for Claude Code, Cursor, Codex, and Antigravity.

Built with **Tauri 2**, **Rust**, **React 19**, **TypeScript**, and native **Win32 APIs**.

---

## Features

- **Side-Notch Silhouette**: Precision Bezier side-notch anchored to the right work-area edge of your primary or secondary display.
- **Pixel-Accurate Click-Through (`SetWindowRgn`)**: Clicks and hovers within the notch silhouette interact directly with the app, while transparent margins let clicks pass completely through to underlying desktop windows.
- **Multi-Monitor & DPI Aware**: Fully responsive across mixed display scaling factors (100%, 125%, 150%, 175%, 200%) with dynamic re-anchoring when displays or taskbars shift.
- **Accurate Domain Metrics**:
  - **Claude Code**: Live 5-hour and 7-day quota utilization tracked via official OAuth usage tokens.
  - **Cursor**: Allowance percentage tracked from local session database.
  - **Codex**: Real-time rate limit tracking via OpenAI endpoint with rollout log failover.
  - **Antigravity**: Native local IPC bridge to the IDE language server with request-count tracking (strictly avoiding fake percentages where no total limit exists).
- **Privacy & Security**: Zero telemetry. Credentials remain strictly local and read-only.
- **Keyboard & Motion Accessible**: Full keyboard navigation support and adherence to Windows `prefers-reduced-motion` settings.

---

## Architecture Overview

```
                                      +------------------------+
                                      |  Windows Desktop Edge  |
                                      +------------------------+
                                                  |
           +--------------------------------------+--------------------------------------+
           |                                                                             |
           v                                                                             v
+------------------------+                                                    +--------------------+
|  Win32 Native Layer    |                                                    | React 19 Frontend  |
| - SetWindowRgn mask    | <====== IPC (work area, DPI, snap broadcast) ===== | - SideNotchShape   |
| - WS_EX_TOOLWINDOW     |                                                    | - ProviderCell/Ring|
| - Fallback anchor logic|                                                    | - Metric formatting|
+------------------------+                                                    +--------------------+
           ^
           | (background polling threads)
+----------------------------------------------------------------------------------------+
|                                    Provider Adapters                                   |
|  - Claude (~/.claude)   - Cursor (state.vscdb)   - Codex (auth.json)   - Antigravity   |
+----------------------------------------------------------------------------------------+
```

---

## Getting Started

### Prerequisites

1. **Node.js**: v22.12 or higher (v24 LTS recommended).
2. **pnpm**: v10.x (`npm install -g pnpm`).
3. **Rust & Cargo**: v1.88+ with Visual Studio C++ Build Tools (MSVC) and a Windows SDK.
4. **Windows**: Windows 10/11 with Microsoft Edge WebView2 Runtime.

### Windows Desk

The app opens as a floating dark dashboard inspired by the widget reference:

- **Project tasks:** create projects, add/complete/delete tasks, and remove a project with confirmation. Tasks are saved locally in the WebView's application storage, with visible save errors.
- **AI usage:** choose Codex, Claude Code, Cursor or Antigravity. Existing local adapters provide real readings, reset times and authentication/stale states. Counts without published limits stay counts.
- **GitHub activity:** enter a public GitHub username to load its most recent 18 weeks of public contributions. Only the username is saved; no token is needed. Private contribution visibility follows the public profile. The calendar is parsed from GitHub's public HTML; a future markup change produces an error rather than fake activity.
- **Quotes:** cycle through an offline collection.
- **App shortcuts:** launch Chrome, Discord, Spotify and OBS Studio from their default Windows installation locations. Missing apps produce an error; custom install locations and Microsoft Store-only installations are not currently resolved.

Drag the dashboard by its header. Use the pin to change always-on-top behavior, **Compact notch** for the original screen-edge overlay, and **Desk ↗** to return. On Windows, closing the window hides it to the system tray. Left-click the tray icon or use **Open Desk** to return; **Quit Live Notch Tracker** exits. Launching a second copy opens the existing app. **Start with Windows** is optional and starts hidden to tray at sign-in. Turn it off before uninstalling to remove its per-user startup entry.

Tasks and the GitHub username stay on the device and are not uploaded by Codenotch. Provider credential files are still read only by the existing adapters. The GitHub card contacts `github.com`; launcher buttons only accept the four fixed app IDs.

For browser-only frontend development, run `pnpm dev`. Tasks, quotes and widget navigation work; native provider sessions, GitHub requests and Windows launching require `pnpm tauri dev`. Browser mode displays these limitations explicitly.

See [Windows validation](docs/WINDOWS-VALIDATION.md) for the release checklist. The Windows GitHub Actions workflow builds and uploads MSI/NSIS artifacts after all checks pass and can be run manually. A successful frontend build alone is not an installer validation.

### Development

Install dependencies and start the Tauri dev server:

```bash
# Install frontend packages
pnpm install

# Run application in development mode
pnpm tauri dev
```

### Verification & Testing

Run all unit tests and quality checks across frontend and backend:

```bash
# Frontend tests (Vitest)
pnpm test

# TypeScript type check & build
pnpm build

# Browser interaction tests (first install Chromium)
pnpm exec playwright install chromium
pnpm test:smoke

# Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Rust formatting & Clippy linter
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

---

## Production Build & Packaging

To generate portable binaries and Windows installers:

```bash
pnpm tauri build
```

This generates:
- **Portable Executable**: `src-tauri/target/release/codenotch-win.exe`
- **NSIS Installer**: `src-tauri/target/release/bundle/nsis/codenotch-win_*_x64-setup.exe`
- **MSI Installer**: `src-tauri/target/release/bundle/msi/codenotch-win_*_x64_en-US.msi`

---

## License

MIT License.

### Upstream Codenotch code

This app integrates adapted Windows code from [vinzdg/codenotch](https://github.com/vinzdg/codenotch), pinned to commit `6522e9f3ee0fe35d46e428b441e6e80b586eacbb`: native Codex quota recovery, native executable discovery, registry autostart and tray behavior. Our React Desk and metric schema remain separate. Source paths, adaptations and validation are documented in [the integration record](docs/upstream/CODENOTCH.md); MIT notices are retained under [third-party/codenotch](third-party/codenotch/LICENSE). Upstream's auto-updater/release identity, account manager and taskbar rendering are not integrated.

Platform-independent integration tests can run without GTK/WebKit:

```sh
cargo test --locked --manifest-path src-tauri/core-tests/Cargo.toml
```
