# Windows Desk Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement inline, task by task.

**Goal:** Deliver the screenshot-inspired widgets in a functioning Windows desktop companion.

**Architecture:** A React dashboard shares existing usage adapters with compact notch mode. Local tasks live in versioned localStorage. Rust commands own public GitHub fetches, allowlisted app launches and native window transitions.

**Tech Stack:** React 19, TypeScript, Tauri 2, Rust, existing pnpm/Vitest.

**Spec:** docs/superpowers/specs/2026-11-02-windows-desk-design.md

## Global Constraints

- Preserve existing provider adapters, metric meanings and compact notch geometry.
- No fabricated tasks, activity or live usage; show honest empty and error states.
- Use fixed launcher IDs; no arbitrary shell commands or credentials.
- Work in existing isolated cloud checkout on a feature branch; no extra worktree.
- Windows packaging and interaction require Windows validation.

## Review Focus

- Corrupt/full local storage must not silently discard or pretend to save tasks.
- Missing/stale provider data must not show a false zero-percent quota.
- GitHub errors, changed HTML and rapid username changes must not show another user's data.
- A missing app or non-Windows runtime must produce an actionable error.
- Mode switching and DPI changes must not leave the Desk clipped by the notch region.

### Task 1: State and native boundaries

Files: `src/desk/model.ts`, `src/desk/model.test.ts`, `src/desk/native.ts`, `src-tauri/src/github_calendar.rs`, `src-tauri/src/launcher.rs`, `src-tauri/src/desktop.rs`, `src-tauri/src/lib.rs`.

- [x] Write failing tests for task/project operations, corrupt saved state, contribution parsing and launcher allowlist.
- [x] Implement versioned local state, typed desktop API, pure contribution parser and fixed Windows launch targets.
- [x] Wire timeout-bounded asynchronous Rust GitHub command and launcher into Tauri.
- [x] Verify Vitest plus pure Rust tests; retain explicit platform limits.

Interfaces: `DeskState`, `readDeskState`, `addTask`, `toggleTask`, `removeTask`, `addProject`, `removeProject`; `GitHubCalendar { username, days: {date,count,level}[] }`; `getCalendar(username)`, `launchApp(id)`.

### Task 2: Dashboard and Windows integration

Files: `src/desk/*View.tsx`, `src/desk/desk.css`, `src/desk/useDeskState.ts`, `src/desk/useProviders.ts`, `src/App.tsx`, `src/CompactNotch.tsx`, `src/index.css`, `src-tauri/src/window.rs`, Tauri config/capabilities.

- [x] Add tests for usage empty/auth/stale/count behavior and dashboard content.
- [x] Implement all five widgets, local task controls, errors/loading, accessible controls and responsive visual design.
- [x] Add browser-safe provider subscriptions with cleanup; keep old notch mode and add return button.
- [x] Implement native mode resizing/mask reset, drag/pin/close controls; keep the window reachable on small displays.
- [x] Run full tests/build and a real browser interaction smoke test including reload persistence.

### Task 3: Windows release checks and review

Files: `.github/workflows/ci.yml`, `README.md`, `docs/WINDOWS-VALIDATION.md`.

- [x] Align Node CI with Vite requirements; publish build artifacts within the CI job and allow manual dispatch.
- [x] Fix the already documented Clippy chunks lint with the supported array-chunk API.
- [x] Document Windows setup, five features, data storage and manual native verification.
- [x] Run final tests/build/formatting, get a fresh whole-change review and address important findings.
- [x] Leave reviewable local changes; no push, release or installer claims without Windows evidence.

## Execution outcome

All implementation tasks and locally supported validation are complete. The full Linux Rust build is blocked by missing GLib development libraries, and Windows native/installer validation remains explicitly pending. Playwright tests and its dev dependency were added so browser regression checks are reproducible in Windows CI. Review findings for clipped return controls, failed expansion recovery and lost pin state were reproduced and fixed.
