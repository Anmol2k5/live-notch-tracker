# Audit Report — Live Notch Tracker (Codenotch for Windows)

**Audit date:** 2026-10-10  
**Repository:** https://github.com/Anmol2k5/live-notch-tracker  
**Audited ref:** `main`, commit `6c073742e57eed2bc828bee87107652de31829cd`  
**Method and limitations:** Full recursive GitHub tree enumerated; selected source files read through GitHub connector, and the Windows GitHub Actions job and raw logs inspected. Attempted a local `git clone --depth 1`, which failed because the container could not resolve `github.com`. Consequently **no local build, execution, dependency sweep, or complete repository-wide grep could be performed**. GitHub CI is the independently executed evidence; none of its results are misrepresented as a local run. Only claims substantiated by checked file/line locations are made. UI observations are source review, not screenshot or usability testing.

## 1. Summary

**Verdict: not production-ready; frontend builds and tests, but the latest Windows CI is red and Windows installer creation has not been exercised.** A working installed application has **not** been independently verified. A precise completion percentage cannot responsibly be inferred from source or these checks.

Stack: Tauri 2 + Rust/Win32 backend; React 19 + TypeScript 6 + Vite 8 frontend; pnpm and Cargo; Windows-native overlay (`README.md:1-20`, `package.json:1-27`, `src-tauri/Cargo.toml:1-37`). Four tracked integrations are Claude Code, Cursor, Codex and Antigravity (`README.md:14-18`, `src-tauri/src/lib.rs:1-6,142-146`).

Entrypoints: `src/main.tsx` → `src/App.tsx:30-111` → `src/components/NotchRootView.tsx:13-108` → `ProviderCell`; frontend subscriptions and IPC are in `src/state/usageStore.ts:6-65`. Tauri entrypoint is `src-tauri/src/main.rs` → `src-tauri/src/lib.rs:104-152`; it registers IPC commands at `lib.rs:115-124` and launches four Rust pollers at `lib.rs:142-146`. Window operations in `src-tauri/src/window.rs`; provider adapters in `usage.rs`, `cursor.rs`, `codex.rs`, `antigravity.rs`; snapshots persisted via `storage.rs`.

Primary risks are failing CI; an undocumented release validation gap; silently swallowed persistence failures; initial-fetch failures indistinguishable from true absence; and responsiveness/multi-monitor behavior that still needs on-device testing.

## 2. Build/Run/Test Results

**Attempted local clone**:
```text
git clone --depth 1 https://github.com/Anmol2k5/live-notch-tracker.git /mnt/data/live-notch-tracker
fatal: unable to access 'https://github.com/Anmol2k5/live-notch-tracker.git/': Could not resolve host: github.com
```
This is **an audit-container network limitation**, not a defect proved in the repository.

**Windows CI evidence**: https://github.com/Anmol2k5/live-notch-tracker/actions/runs/37345655388 , job `111883449780`, run 2026-10-05, same commit as audited tree. Jobs are defined in `.github/workflows/ci.yml:9-52`.

| Check | Observed result |
|---|---|
| pnpm install --frozen-lockfile | PASS (GitHub Actions) |
| pnpm build (tsc + Vite) | PASS (GitHub Actions; `package.json:8`) |
| pnpm test | PASS: 5 test files, 28 tests (GitHub Actions; `package.json:11`) |
| cargo fmt --check | PASS (GitHub Actions) |
| cargo clippy -- -D warnings | **FAIL**, exit code 1 |
| cargo test | SKIPPED after Clippy failure |
| pnpm tauri build | SKIPPED after Clippy failure |
| Desktop runtime and installer install/uninstall | NOT RUN / NOT VERIFIED |
| Dedicated eslint / mypy / other lint | No script declared in `package.json:6-12`; no execution claimed |

**Verbatim failing diagnostics (CI):**
```text
error: using `chunks_exact` with a constant chunk size
   --> src\antigravity.rs:316:14
316 |             .chunks_exact(2)
    |              ^^^^^^^^^^^^ help: consider using `as_chunks` instead: `as_chunks::<2>().0.iter()`
= note: `-D clippy::chunks-exact-to-as-chunks` implied by `-D warnings`
error: could not compile `codenotch-win` (lib) due to 1 previous error
Process completed with exit code 1.
```

## 3. Critical Issues (breaks core functionality or release verification)

- [ ] **[CRITICAL / confirmed]** `src-tauri/src/antigravity.rs:316` — new Rust Clippy lint errors on `.chunks_exact(2)` with `-D warnings`, causing the Windows CI workflow to fail and skipping Rust tests and package build. Replace with an equivalent chunks API where compatible, or document narrowly justified lint handling; run the entire pipeline again. Evidence: CI job above.
- [ ] **[HIGH / unverified release gate]** `.github/workflows/ci.yml:45-52` — Windows backend tests and package generation are gated behind the failed Clippy step. Fix failure, then ensure all subsequent stages actually complete and upload a real installer artifact. This is a verification gap, not evidence that runtime/installers are broken.
- [ ] **[HIGH / resilience risk]** `src-tauri/src/storage.rs:85-97` — `persist_provider_snapshot` ignores serialization and filesystem write errors (`if let Ok`, `let _ = std::fs::write`). A disk permission/space issue can silently prevent state persistence. Return a `Result` and log a sanitized error; test readonly filesystem, missing directory and disk errors.
- [ ] **[HIGH / visibility risk]** `src/state/usageStore.ts:12-17,45-59` and `src/components/NotchRootView.tsx:39-54` — failed initial IPC requests are logged but leave provider values null; the view then labels the situation as idle/no active sessions. Distinguish `loading`, `failed`, and `no providers` states with retry controls. Whether this occurs for a particular user is unverified.

## 4. Broken / Non-Working Features

- [ ] **[unverified]** `README.md:11-20`, `src-tauri/src/window.rs:188-224` — real Win32 hit-test silhouette, mixed DPI, taskbar relocation and Alt-Tab behavior need a Windows runtime interaction matrix. Source presence does not prove mouse hit-testing works across scale changes.
- [ ] **[unverified]** `README.md:15-18`, `src-tauri/src/lib.rs:142-146` — integration adapters are wired, but live authenticated Claude/Cursor/Codex/Antigravity accounts and outage/rate-limit responses were not exercised in this audit.
- [ ] **[UX defect by design]** `src/components/ProviderCell.tsx:74,81` and `src/state/usageStore.ts:41-43` — clicking a single provider refreshes **all** providers, potentially surprising users and causing unnecessary polling. Either label it “Refresh all” or expose provider-specific refresh IPC.
- [ ] **[unverified]** `src-tauri/tauri.conf.json:35-44`, `.github/workflows/ci.yml:51-52` — installer output and upgrades cannot be confirmed until the pipeline reaches packaging.

## 5. Hardcoded Values & Mock Data

- [ ] `src-tauri/src/window.rs:35-43` — monitor discovery fallback always returns `1920×1040`, origin 0, scale 1.0. Avoid pretending this is a real work area; report unavailable-monitor state or query the OS again and recover.
- [ ] `src/geometry/notchGeometry.ts:18-23` — fixed width 70 and pitch `401/4` logical units. Centralize as layout tokens and cap final height to monitor work area (`notchGeometry.ts:29-41`).
- [ ] `src/App.tsx:33` — initial placeholder geometry uses 3 cells. Replace with a loading layout aligned to actual provider count; avoid a visible jump once work area resolves.
- [ ] `src-tauri/tauri.conf.json:18-21` — initial hardcoded `70×301` window (runtime does resize at `src/App.tsx:35-45`). Ensure the initial invisible window never becomes visible at an old size.
- [ ] `src-tauri/tauri.conf.json:8`, `vite.config.ts:15-18` — `localhost:1420` is the expected development-only Vite endpoint, **not** a leaked production host; maintain consistency.
- [ ] `src-tauri/src/antigravity.rs:197` — `127.0.0.1` is intended local language-server IPC, not an externally hardcoded server; retain loopback-only semantics.
- [ ] `src-tauri/src/usage.rs:18`, `src-tauri/src/cursor.rs:30`, `src-tauri/src/codex.rs:39`, `src-tauri/src/antigravity.rs:46-48` — fixed provider HTTP endpoints; centralize configuration/constants and document API change/failure behavior rather than treating each URL as an exposed secret.
- [ ] `src/App.tsx:17-19` — provider with zero windows is forced to `0%`, which looks like genuine usage. Display “—”/“No reading” instead of inventing zero utilization.

**Scope caution:** This is a verified **selected-file** sweep, not the user's requested exhaustive grep across every file.

## 6. TODO/FIXME Inventory

No exhaustive inventory is asserted because a local recursive checkout/search was not possible. Specific outstanding markers and unfinished language identified in inspected files:
- [ ] `src/geometry/notchGeometry.ts:31` — runtime error string says `unsupported edge in fixture slice`; replace fixture-era wording and explicitly document right-edge-only support.
- [ ] `vite.config.ts:3` — `@ts-expect-error type error without @types/node package`; add `@types/node`, remove suppression if no longer needed, and rerun TypeScript.
- [ ] `src-tauri/src/window.rs:19,35-43` — monitor fallback comments explicitly describe guessed geometry; address before multi-monitor release.
- [ ] `src/components/NotchRootView.tsx:40-53` — “Sensible empty / listening state” comment currently conflates load errors with no sessions (as above).

## 7. Security & Config Issues

- [ ] **[HIGH]** `.gitignore:1-25` — no explicit `.env`, `.env.*`, local credentials directories or test artifact secret patterns (except `*.local`). Add explicit excludes while whitelisting safe `.env.example` and audit tracked history before shipping.
- [ ] **[MEDIUM]** `src-tauri/src/antigravity.rs:178-189` — `danger_accept_invalid_certs(true)` and `danger_accept_invalid_hostnames(true)` permit insecure TLS **within the dedicated local agent**. Code currently constructs a loopback URL at `antigravity.rs:197`; enforce host validation in a narrowly scoped helper and never reuse this agent with a remote endpoint.
- [ ] **[MEDIUM]** `src-tauri/src/lib.rs:19-21,104-106` and `src-tauri/src/storage.rs:45-50` — application data directory failures can fall back to the process working directory and creation errors are ignored in startup. Fail closed or use a predictable writable OS config directory.
- [ ] **[MEDIUM]** `src-tauri/tauri.conf.json:31-33` — CSP permits unsafe inline styles to support current inline React styling. Prefer CSS classes/nonces where practical; avoid adding remote script origins.
- [ ] **[status: not confirmed]** `src-tauri/src/usage.rs:210-231`, `src-tauri/src/codex.rs:139-163`, `src-tauri/src/cursor.rs:121-130` — provider credentials are read from local source locations. Need an actual log redaction and permissions test before certifying “zero credential exposure” in `README.md:19`; no real credential leak was demonstrated.
- [ ] **[status: no positive finding]** No live committed secret was identified in examined files. This is **not** a complete git-history secret scan.

## 8. UI/UX Issues & Fixes

- [ ] `src/geometry/notchGeometry.ts:18-23,29-41` — height scales linearly with detected providers and does not clamp to available screen height. Limit to e.g. `min(calculatedHeight, workAreaHeight - 32px)`; set `overflow-y: auto` for the interior on small displays.
- [ ] `src/components/ProviderCell.tsx:94-101` — interactive hit area effectively 56px wide with a 48px ring. Keep at least a 44×44px pointer/keyboard target, with a >=6px visible focus margin and no clipping by the mask.
- [ ] `src/components/ProviderCell.tsx:56,119-121`, `src/index.css:29-37` — overlapping hover scaling and staggered entrance animation could feel janky. Prefer 160–200ms easing, a single transform owner and use 40–60ms staggering. This is a visual source recommendation, not observed jank.
- [ ] `src/index.css:71-79` — reduced motion disables provider/ring effects, but also verify nested SVG and any future animation. Include automated reduced-motion assertions and Windows accessibility check.
- [ ] `src/components/ProviderCell.tsx:16-27,58` — reset countdown is computed only on render; without rerenders its text ages. Update a time state on a 30–60s interval while visible, stop the interval on unmount; no render-frequency guarantee was proven.
- [ ] `src/components/NotchRootView.tsx:39-54`, `src/state/usageStore.ts:12-17` — add explicit states: “Checking providers…”, “No providers connected”, “Unable to load · Retry”. Never show network/IPC failure as ordinary idle.
- [ ] `src-tauri/tauri.conf.json:20-27` — intentionally non-resizable/focusless overlay. Validate keyboard traversal, screen-reader handling and Windows keyboard navigation on real hardware rather than assuming conventional desktop window behavior.

## 9. Prioritized Fix Plan

| Priority | Task | Effort | Success criterion |
|---|---|---|---|
| Critical | Repair Clippy error (`antigravity.rs:316`) | Quick | Windows CI Clippy green |
| High | Run Rust tests + Tauri Windows packaging (`ci.yml:45-52`) | Moderate | MSI/NSIS built; smoke-installed on Windows |
| High | Make storage errors observable (`storage.rs:85-97`) | Moderate | Error-injection tests pass |
| High | Explicit loading/error/empty states (`usageStore.ts:12-17`, `NotchRootView.tsx:39-54`) | Moderate | Failed IPC clearly surfaced |
| High | Monitor/DPI/Win32 manual QA (`window.rs:13-44,188-224`) | Moderate | All targeted DPI and display cases pass |
| Medium | Protect configs and credentials (`.gitignore:1-25`) | Quick | Ignore checks + secret scanner clean |
| Medium | Replace fake 0% empty readings (`App.tsx:13-19`) | Quick | Missing data shows unknown, not zero |
| Medium | Fit on small monitors (`notchGeometry.ts:18-41`) | Moderate | No clipped cells at supported resolutions |
| Medium | Improve refresh semantics (`ProviderCell.tsx:74-81`) | Quick/Moderate | Label accurate or targeted IPC exists |
| Low | Remove fixture labels and TS suppression (`notchGeometry.ts:31`, `vite.config.ts:3`) | Quick | Clear names + clean TypeScript |

## 10. Nice-to-haves

- [ ] `src-tauri/src/lib.rs:142-146` — introduce a unified provider polling coordinator/health diagnostics without losing adapter isolation.
- [ ] `src/components/ProviderCell.tsx:61-75` — replace native multiline title tooltips with an accessible, keyboard-activated detail panel for richer status explanations.
- [ ] `src/index.css:29-60` — visual regression tests for ring, hover, idle, stale, 125–200% DPI and reduced motion.
- [ ] `.github/workflows/ci.yml:9-52` — upload Windows build artifacts; add end-to-end installer smoke tests and a published release checklist.
- [ ] `README.md:91-103` — specify signing strategy, installer testing process, actual supported Windows versions and rollback instructions.

### Audit's verification boundary

The GitHub CI run is conclusive evidence of the listed frontend passes and the one Clippy failure. It is **not** evidence that Rust tests, Windows packaging, live provider polling, user interaction, or security are clean. Some audit requirements (full grep, unused-dependency resolution, full history secret scan, runtime UI tests) remain **incomplete** due to local clone/network access and are explicitly not presented as completed. Repeat from a Windows checkout with `pnpm install --frozen-lockfile && pnpm build && pnpm test && cargo fmt --manifest-path src-tauri/Cargo.toml --check && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml && pnpm tauri build`, plus secret scanning and on-device verification.
