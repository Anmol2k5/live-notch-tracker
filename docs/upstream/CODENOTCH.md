# Codenotch integration

Source: https://github.com/vinzdg/codenotch
Pinned reference: 6522e9f3ee0fe35d46e428b441e6e80b586eacbb (Windows subtree).

Keep our React Desk, tasks, GitHub activity, quotes, launchers and metric contract. Adapt upstream Windows native behavior instead of replacing our checkout with its separate application.

Implementation scope:
- [x] Adapt native Codex `app-server` quota recovery and native executable discovery from `windows/codenotch/src/codex.rs`; HTTP remains first, server Retry-After remains authoritative, rollout remains the final fallback. Only initialize and quota queries; no login or inference.
- [x] Adapt per-user registry autostart from `windows/codenotch/src/autostart.rs` under our own registry value name, opt-in via the UI, hidden `--background` startup with tray recovery.
- [x] Adapt tray show/refresh/quit behavior from `windows/codenotch/src/tray.rs`; use single-instance plugin so launching again opens the existing Desk.
- [x] Preserve root and Windows MIT notices, list copied/adapted paths and local differences. No upstream signing keys, update feed or branding assets are copied.
- [x] Tests: Codex bucket parsing/missing usage/weekly limits/native path allowlist/backoff policy; autostart command quoting; frontend startup toggle errors and tray open event; existing suites/build/formatting. Windows native/installer checks remain a separate required validation on Windows.

Native integration runs on Tauri's main thread for tray/window operations. Closing the window hides to tray; Quit exits. Silent startup must never automatically enable autostart. Browser mode keeps these Windows-only controls unavailable.

## Adaptation map

| Upstream source | Local integration | Local differences |
| --- | --- | --- |
| `windows/codenotch/src/codex.rs` | `src-tauri/src/codex_rpc.rs`, `codex.rs` | Parse into our metric model; direct bin path supported; never execute shell wrappers; preserve backoff and 30-minute failed-native stand-down. |
| `windows/codenotch/src/autostart.rs` | `src-tauri/src/autostart.rs` | Windows-only; own `LiveNotchTracker` Run value, `--background`, invalid-path rejection, UI opt-in. |
| `windows/codenotch/src/tray.rs` | `src-tauri/src/tray.rs`, `lib.rs`, `src/App.tsx` | Smaller show/refresh/quit menu for our Desk, frontend mode event, hide-to-tray and single-instance activation. |

The single-instance plugin is pinned to upstream's lockfile version 2.4.4, preserving existing Tauri versions. Upstream root and Windows MIT notices are retained in `third-party/codenotch`.

Validation: 6 new native-core tests, frontend tests and browser startup/tray-event regression tests. Full native Cargo check remains blocked on this Linux host by missing GLib/GIO development packages; Windows execution and installers remain pending. No machine startup setting was changed during development.

## Final verification

- 35 frontend tests, 9 Playwright tests, and 12 platform-independent Rust core tests passed.
- TypeScript/Vite build, Rust formatting, core Clippy with warnings denied, and whitespace checks passed.
- Fresh review findings fixed: provider refresh no longer clears Claude's persisted Retry-After; compact placement no longer calls show(), preserving tray-hidden state. A provider-event regression test reproduced the latter before the fix. Full native refresh/lifecycle validation remains in the Windows checklist.
- Full native Cargo check was attempted with the pinned dependency lock and is blocked by this Linux host's missing GLib/GIO development libraries. This is not Windows compilation or installer evidence.
