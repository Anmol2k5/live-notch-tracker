# Windows Desk

Implement the user's screenshot as a usable Windows desktop companion, using the existing Tauri 2/Rust and React stack. The user requested implementation immediately; the layout and interaction choices below are implementation defaults.

## Approach

Keep the existing thin screen-edge notch and add a floating Desk window mode, rather than a marketing page or five separate native windows. A single dashboard makes all five features discoverable and keeps Windows startup simple. A compact-mode button returns to the existing notch; the notch has an accessible return action.

## Behavior and visual design

Dark rounded, subtly green-tinted cards with fine borders, restrained shadows, mint accents, Segoe UI, compact labels and generous spacing. The dashboard contains project task lists, selectable AI provider usage, a GitHub contribution heatmap, a rotating offline quote, and four Windows app shortcuts. It remains responsive and keyboard accessible, with reduced-motion support.

Tasks support projects, add/toggle/delete, persisted locally with validated data and visible storage failures. No initial tasks or usage are fabricated. The GitHub card accepts a public username and fetches its real public calendar through Rust with strict username validation, request timeout and explicit errors. Private activity is limited to what GitHub publishes. App launching accepts only fixed IDs for Chrome, Discord, Spotify and OBS; no arbitrary commands or shell interpolation. Missing apps produce useful errors.

AI usage uses the existing local provider adapters; preserve statuses, reset times and count-only metrics. The frontend must also render in a browser for development, explaining which actions require the desktop app without throwing Tauri IPC errors.

The Desk is draggable, supports pin/unpin and close, and starts visible. Native mode switching must clear the old Win32 silhouette before showing the rectangular dashboard and restore the notch mask when compact. A smaller display must keep the dashboard within its work area.

## Validation and limits

Unit-test persisted input, project/task behavior, usage states, GitHub parsing/validation and launcher routing. Exercise browser UI interactions and build. Install local Rust tooling for pure backend tests and formatting; Windows CI remains authoritative for MSVC compilation, backend integration tests and NSIS/MSI artifacts. No claim of a locally tested Windows installer from this Linux host. Do not push or publish without instruction.
