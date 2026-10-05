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

1. **Node.js**: v20 or higher.
2. **pnpm**: v10.x (`npm install -g pnpm`).
3. **Rust & Cargo**: v1.84+ with MSVC build tools.

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
