//! Claude usage adapter (official), implemented from the upstream Codenotch's documented behaviour.
//! Endpoint: GET https://api.anthropic.com/api/oauth/usage
//! Headers: Authorization: Bearer <token>; anthropic-beta: oauth-2025-04-20; 15 s timeout
//! Rules (upstream's discipline):
//!   - the credential comes from Claude Code's own store (Windows: ~/.claude/.credentials.json), read only
//!   - 401/403 → re-read the credential once and retry (Claude Code may have just refreshed the token) → still failing means needsAuth
//!   - 429 → back off 60 s × 2^n capped at 15 min, Retry-After only raises it; the deadline is persisted
//!   - never invent a percentage on failure: keep the last reading marked stale, and the UI shows how old it is
//! Reply (snake_case): { limits:[{kind,percent,resets_at}], five_hour:{utilization,resets_at}, seven_day:{...} }
//! limits is the forward-compatible main shape; five_hour/seven_day are merged in as a fallback (a window that just rolled over disappears from limits).

use crate::AppState;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

const ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";
const POLL_ACTIVE_SECS: u64 = 60;
const POLL_IDLE_SECS: u64 = 300;
const BACKOFF_BASE_SECS: u64 = 60;
const BACKOFF_CAP_SECS: u64 = 900;

static REFRESH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Immediate refresh from the tray or a command
pub fn request_refresh() {
    REFRESH.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Sleep in slices so request_refresh can interrupt it
fn sleep_interruptible(total_secs: u64) {
    for _ in 0..total_secs {
        if REFRESH.swap(false, std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LimitWindow {
    pub id: String,
    pub label: String,
    /// 0.0–1.0 (fraction used)
    pub used: f64,
    /// Reset time, ms epoch (None = unknown)
    pub resets_at: Option<u64>,
    /// Pure count window (no published denominator, e.g. Antigravity's requests today) — the cell shows ~N and the ring draws only its track
    #[serde(default)]
    pub count: Option<i64>,
    /// The number is ours, not the vendor's (upstream fidelity=.derived) — the card adds a ~ prefix
    #[serde(default)]
    pub derived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageSnapshot {
    /// ok | stale | needsAuth | backoff | error
    pub status: String,
    pub windows: Vec<LimitWindow>,
    pub fetched_at: u64,
    pub note: String,
    #[serde(default)]
    pub backoff_until: u64,
}

fn store_path() -> std::path::PathBuf {
    crate::data_dir().with_file_name("usage.json")
}

pub fn load_persisted() -> UsageSnapshot {
    std::fs::read_to_string(store_path())
        .ok()
        .and_then(|t| serde_json::from_str::<UsageSnapshot>(&t).ok())
        .map(|mut s| {
            if !s.windows.is_empty() {
                s.status = "stale".into(); // an old reading after a restart is labelled as such
            }
            s
        })
        .unwrap_or_default()
}

fn persist(s: &UsageSnapshot) {
    if let Ok(t) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(store_path(), t);
    }
}

fn parse_credentials_value(v: &serde_json::Value, now: u64) -> Option<(String, bool)> {
    let oauth = v.get("claudeAiOauth").unwrap_or(v);
    let tok = oauth.get("accessToken").and_then(|x| x.as_str())?;
    if tok.is_empty() {
        return None;
    }
    let expired = oauth
        .get("expiresAt")
        .and_then(|x| x.as_f64())
        .map(|ms| (ms as u64) <= now)
        .unwrap_or(false);
    Some((tok.to_string(), expired))
}

fn credentials_from_text(text: &str, now: u64) -> Option<(String, bool)> {
    let v = serde_json::from_str::<serde_json::Value>(text).ok()?;
    parse_credentials_value(&v, now)
}

/// Reads Claude Code's OAuth credential. Returns (token, expired hint).
fn read_credentials() -> Option<(String, bool)> {
    let home = dirs::home_dir()?;
    let now = now_ms();
    for name in [".credentials.json", "credentials.json"] {
        let p = home.join(".claude").join(name);
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        if let Some(creds) = credentials_from_text(&text, now) {
            return Some(creds);
        }
    }
    None
}

/// For doctor: credential probe report (prints no secret values)
pub fn probe_credentials() -> String {
    match read_credentials() {
        Some((tok, expired)) => format!(
            "credential: found (token {} chars, {})",
            tok.len(),
            if expired { "expired — Claude Code refreshes it on its next use" } else { "valid" }
        ),
        None => "credential: ~/.claude/.credentials.json not found (needsAuth; the desktop app may use another store — signing in once with the Claude Code CLI creates it)".into(),
    }
}

fn parse_reset(v: &serde_json::Value) -> Option<u64> {
    v.as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.timestamp_millis().max(0) as u64)
}

fn label_for(kind: &str) -> String {
    match kind {
        "session" => "Current session".into(),
        "seven_day" | "weekly_all" => "Weekly (all models)".into(),
        "seven_day_opus" | "weekly_opus" => "Weekly (Opus)".into(),
        "weekly_scoped" => "Weekly (model-scoped)".into(),
        other => {
            // Forward compatibility: an unknown kind gets a readable label
            let mut s = other.replace('_', " ");
            if let Some(c) = s.get_mut(0..1) {
                c.make_ascii_uppercase();
            }
            s
        }
    }
}

fn parse_response(v: &serde_json::Value) -> Vec<LimitWindow> {
    let mut out: Vec<LimitWindow> = Vec::new();
    if let Some(arr) = v.get("limits").and_then(|x| x.as_array()) {
        for l in arr {
            let Some(kind) = l.get("kind").and_then(|x| x.as_str()) else {
                continue;
            };
            let Some(pct) = l.get("percent").and_then(|x| x.as_f64()) else {
                continue;
            };
            let resets = l.get("resets_at").and_then(parse_reset);
            if resets.is_none() {
                continue; // upstream rule: a window without a reset time is not shown
            }
            out.push(LimitWindow {
                id: kind.to_string(),
                label: label_for(kind),
                used: (pct / 100.0).clamp(0.0, 1.0),
                resets_at: resets, ..Default::default()
            });
        }
    }
    // Fallback merge: a window that just rolled over disappears from limits while the named field remains.
    // In practice the kinds in limits are weekly_all/weekly_scoped, not seven_day — deduplicating by id
    // alone would add the seven_day fallback a second time (the card showed "Weekly all" and
    // "Weekly (all models)" as twins). Three dedupe rules: id alias / same resets_at and percentage / same label.
    let aliases: [(&str, &str, &[&str]); 2] = [
        ("five_hour", "session", &["session", "five_hour"]),
        ("seven_day", "seven_day", &["seven_day", "weekly_all", "weekly"]),
    ];
    for (field, id, alias) in aliases {
        let Some(w) = v.get(field) else { continue };
        let Some(u) = w.get("utilization").and_then(|x| x.as_f64()) else { continue };
        let used = (u / 100.0).clamp(0.0, 1.0);
        let resets_at = w.get("resets_at").and_then(parse_reset);
        let label = label_for(id);
        let dup = out.iter().any(|x| {
            alias.contains(&x.id.as_str())
                || x.label == label
                || (resets_at.is_some()
                    && x.resets_at.map(|r| r / 1000) == resets_at.map(|r| r / 1000)
                    && (x.used - used).abs() < 0.005)
        });
        if dup {
            continue;
        }
        out.push(LimitWindow { id: id.into(), label, used, resets_at, ..Default::default() });
    }
    // session always comes first (upstream display order)
    out.sort_by_key(|w| if w.id == "session" { 0 } else { 1 });
    out
}

enum FetchErr {
    NeedsAuth,
    RateLimited(u64), // suggested wait in seconds (the Retry-After before the floor is applied)
    Other(String),
}

fn fetch_once(token: &str) -> Result<Vec<LimitWindow>, FetchErr> {
    let resp = ureq::get(ENDPOINT)
        .set("Authorization", &format!("Bearer {token}"))
        .set("anthropic-beta", "oauth-2025-04-20")
        .timeout(Duration::from_secs(15))
        .call();
    match resp {
        Ok(r) => {
            let v: serde_json::Value = r
                .into_json()
                .map_err(|e| FetchErr::Other(format!("parse: {e}")))?;
            Ok(parse_response(&v))
        }
        Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
            Err(FetchErr::NeedsAuth)
        }
        Err(ureq::Error::Status(429, r)) => {
            let ra = r
                .header("retry-after")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            Err(FetchErr::RateLimited(ra))
        }
        Err(ureq::Error::Status(code, _)) => Err(FetchErr::Other(format!("HTTP {code}"))),
        Err(e) => Err(FetchErr::Other(format!("{e}"))),
    }
}

fn backoff_secs(consecutive: u32, retry_after_floor: u64) -> u64 {
    let exp = BACKOFF_BASE_SECS.saturating_mul(1u64 << consecutive.min(4));
    exp.clamp(BACKOFF_BASE_SECS, BACKOFF_CAP_SECS).max(retry_after_floor)
}

fn set_and_broadcast(app: &AppHandle, mutate: impl FnOnce(&mut UsageSnapshot)) {
    let st = app.state::<AppState>();
    let snap = {
        let mut u = st.usage.lock().unwrap_or_else(|e| e.into_inner());
        mutate(&mut u);
        u.clone()
    };
    persist(&snap);
    let _ = app.emit("usage", &snap);
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        // Broadcast the persisted old reading at startup (stale beats blank)
        {
            let st = app.state::<AppState>();
            let snap = st.usage.lock().unwrap_or_else(|e| e.into_inner()).clone();
            let _ = app.emit("usage", &snap);
        }
        let mut consecutive_429: u32 = 0;
        loop {
            // No requests inside the backoff window
            let bu = {
                let st = app.state::<AppState>();
                let u = st.usage.lock().unwrap_or_else(|e| e.into_inner());
                u.backoff_until
            };
            let now = now_ms();
            if bu > now {
                sleep_interruptible(((bu - now) / 1000).clamp(1, 30));
                continue;
            }
            match read_credentials() {
                None => set_and_broadcast(&app, |u| {
                    u.status = "needsAuth".into();
                    u.note = "No Claude Code credential found".into();
                }),
                Some((token, expired)) => {
                    // On 401/403 re-read the credential and retry once (Claude Code may have just refreshed it)
                    let result = match fetch_once(&token) {
                        Err(FetchErr::NeedsAuth) => match read_credentials() {
                            Some((t2, _)) if t2 != token => fetch_once(&t2),
                            _ => Err(FetchErr::NeedsAuth),
                        },
                        other => other,
                    };
                    let auth_note = if expired {
                        "Credential expired — run any claude command (or chat with Claude) to refresh it"
                    } else {
                        "Credential rejected (switched accounts?)"
                    };
                    match result {
                        Ok(windows) => {
                            consecutive_429 = 0;
                            set_and_broadcast(&app, |u| {
                                u.status = "ok".into();
                                u.windows = windows;
                                u.fetched_at = now_ms();
                                u.note.clear();
                                u.backoff_until = 0;
                            });
                        }
                        Err(FetchErr::NeedsAuth) => set_and_broadcast(&app, |u| {
                            u.status = "needsAuth".into();
                            u.note = auth_note.into();
                        }),
                        Err(FetchErr::RateLimited(ra)) => {
                            consecutive_429 += 1;
                            let wait = backoff_secs(consecutive_429 - 1, ra);
                            set_and_broadcast(&app, |u| {
                                if !u.windows.is_empty() {
                                    u.status = "stale".into();
                                }
                                u.note = format!("Rate limited, retrying in {wait}s");
                                u.backoff_until = now_ms() + wait * 1000;
                            });
                        }
                        Err(FetchErr::Other(msg)) => set_and_broadcast(&app, |u| {
                            if u.windows.is_empty() {
                                u.status = "error".into();
                            } else {
                                u.status = "stale".into();
                            }
                            u.note = msg;
                        }),
                    }
                }
            }
            // 60 s while a session is active, 300 s otherwise (upstream throttling discipline)
            let active = crate::is_claude_running();
            sleep_interruptible(if active {
                POLL_ACTIVE_SECS
            } else {
                POLL_IDLE_SECS
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---- parse_response: normal successful response ----

    #[test]
    fn normal_limits_response() {
        let v = json!({
            "limits": [
                { "kind": "session", "percent": 42.5, "resets_at": "2026-09-13T12:00:00Z" },
                { "kind": "seven_day", "percent": 10.0, "resets_at": "2026-09-20T00:00:00Z" }
            ]
        });
        let w = parse_response(&v);
        assert_eq!(w.len(), 2);
        // session sorts first
        assert_eq!(w[0].id, "session");
        assert!((w[0].used - 0.425).abs() < 1e-9);
        assert_eq!(w[0].label, "Current session");
        assert!(w[0].resets_at.is_some());
        assert_eq!(w[1].id, "seven_day");
        assert!((w[1].used - 0.10).abs() < 1e-9);
    }

    #[test]
    fn normal_limits_with_fallback_dedup() {
        // limits has weekly_all at 25%; seven_day fallback at same percent+time must not duplicate
        let v = json!({
            "limits": [
                { "kind": "weekly_all", "percent": 25.0, "resets_at": "2026-09-20T00:00:00Z" }
            ],
            "seven_day": { "utilization": 25.0, "resets_at": "2026-09-20T00:00:00Z" },
            "five_hour": { "utilization": 73.0, "resets_at": "2026-09-13T15:00:00Z" }
        });
        let w = parse_response(&v);
        // weekly_all from limits, five_hour from fallback, seven_day fallback deduped
        assert_eq!(w.len(), 2);
        let ids: Vec<&str> = w.iter().map(|x| x.id.as_str()).collect();
        assert!(ids.contains(&"weekly_all"));
        assert!(ids.contains(&"session")); // five_hour maps to session id
        // seven_day should not appear as duplicate
        assert!(!ids.contains(&"seven_day"));
    }

    #[test]
    fn fallback_only_when_limits_empty() {
        let v = json!({
            "five_hour": { "utilization": 50.0, "resets_at": "2026-09-13T15:00:00Z" },
            "seven_day": { "utilization": 20.0, "resets_at": "2026-09-20T00:00:00Z" }
        });
        let w = parse_response(&v);
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].id, "session");
        assert!((w[0].used - 0.5).abs() < 1e-9);
        assert_eq!(w[1].id, "seven_day");
    }

    #[test]
    fn limits_missing_reset_is_skipped() {
        let v = json!({
            "limits": [
                { "kind": "session", "percent": 42.0, "resets_at": "not-a-date" },
                { "kind": "session", "percent": 42.0, "resets_at": "2026-09-13T12:00:00Z" }
            ]
        });
        let w = parse_response(&v);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].id, "session");
    }

    // ---- backoff_secs: 429 handling including Retry-After ----

    #[test]
    fn backoff_base_and_exponential() {
        assert_eq!(backoff_secs(0, 0), 60);
        assert_eq!(backoff_secs(1, 0), 120);
        assert_eq!(backoff_secs(2, 0), 240);
        assert_eq!(backoff_secs(3, 0), 480);
        assert_eq!(backoff_secs(4, 0), 900); // 960 capped to 900
        assert_eq!(backoff_secs(10, 0), 900); // capped even beyond 4
    }

    #[test]
    fn backoff_honors_retry_after_floor() {
        // Retry-After raises the wait, never lowers it
        assert_eq!(backoff_secs(0, 120), 120);
        assert_eq!(backoff_secs(0, 30), 60); // floor 60 still wins when Retry-After is small
        assert_eq!(backoff_secs(1, 300), 300); // exp 120 < 300 so 300 wins
        assert_eq!(backoff_secs(4, 1000), 1000); // Retry-After can exceed cap
        assert_eq!(backoff_secs(4, 0), 900);
    }

    #[test]
    fn backoff_retry_after_string_parse_simulation() {
        // Simulate fetch Once behaviour: header missing -> 0, valid -> parsed value
        let ra_missing: u64 = "".parse::<u64>().unwrap_or(0);
        assert_eq!(backoff_secs(0, ra_missing), 60);
        let ra_valid: u64 = "180".parse::<u64>().unwrap_or(0);
        assert_eq!(backoff_secs(0, ra_valid), 180);
        let ra_invalid: u64 = "not-a-number".parse::<u64>().unwrap_or(0);
        assert_eq!(backoff_secs(0, ra_invalid), 60);
    }

    // ---- malformed / partial JSON degrades gracefully ----

    #[test]
    fn malformed_empty_object_yields_no_windows() {
        let v = json!({});
        let w = parse_response(&v);
        assert!(w.is_empty(), "empty object should not panic and should yield no windows");
    }

    #[test]
    fn malformed_limits_not_array() {
        let v = json!({ "limits": "not-an-array" });
        let w = parse_response(&v);
        assert!(w.is_empty());
    }

    #[test]
    fn malformed_partial_limits_missing_fields() {
        let v = json!({
            "limits": [
                { "kind": "session" }, // missing percent
                { "percent": 50.0 },   // missing kind
                { "kind": "session", "percent": "not-a-number", "resets_at": "2026-09-13T12:00:00Z" },
                { "kind": "session", "percent": 50.0, "resets_at": "2026-09-13T12:00:00Z" } // only valid one
            ]
        });
        let w = parse_response(&v);
        assert_eq!(w.len(), 1);
        assert!((w[0].used - 0.5).abs() < 1e-9);
    }

    #[test]
    fn malformed_percent_clamped() {
        let v = json!({
            "limits": [
                { "kind": "session", "percent": 150.0, "resets_at": "2026-09-13T12:00:00Z" },
                { "kind": "seven_day", "percent": -10.0, "resets_at": "2026-09-13T12:00:00Z" }
            ]
        });
        let w = parse_response(&v);
        assert_eq!(w.len(), 2);
        assert!((w[0].used - 1.0).abs() < 1e-9);
        assert!((w[1].used - 0.0).abs() < 1e-9);
    }

    #[test]
    fn malformed_json_does_not_panic_on_invalid_reset() {
        let v: serde_json::Value = serde_json::from_str(r#"{"limits":[{"kind":"session","percent":42.5,"resets_at":null}]}"#).unwrap();
        let w = std::panic::catch_unwind(|| parse_response(&v));
        assert!(w.is_ok());
        assert!(w.unwrap().is_empty());
    }

    // ---- missing / invalid credentials file ----

    #[test]
    fn credentials_missing_file_returns_none() {
        let result = credentials_from_text("", 0);
        assert!(result.is_none());
    }

    #[test]
    fn credentials_invalid_json_returns_none() {
        assert!(credentials_from_text("not json at all", 0).is_none());
        assert!(credentials_from_text(r#"{"claudeAiOauth": "not an object"}"#, 0).is_none());
    }

    #[test]
    fn credentials_missing_access_token_returns_none() {
        let j = r#"{"claudeAiOauth":{"expiresAt": 9999999999999}}"#;
        assert!(credentials_from_text(j, 0).is_none());
        let j2 = r#"{"otherKey": 123}"#;
        assert!(credentials_from_text(j2, 0).is_none());
    }

    #[test]
    fn credentials_valid_token_future_expiry() {
        let now: u64 = 1_700_000_000_000;
        let future = now + 3600_000;
        let j = format!(r#"{{"claudeAiOauth":{{"accessToken":"tok123","expiresAt":{future}}}}}"#);
        let res = credentials_from_text(&j, now).unwrap();
        assert_eq!(res.0, "tok123");
        assert!(!res.1, "future expiry should be not-expired");
    }

    #[test]
    fn credentials_expired_token() {
        let now: u64 = 1_700_000_000_000;
        let past = now - 1000;
        let j = format!(r#"{{"claudeAiOauth":{{"accessToken":"tok123","expiresAt":{past}}}}}"#);
        let res = credentials_from_text(&j, now).unwrap();
        assert!(res.1, "past expiry should be marked expired");
    }

    #[test]
    fn credentials_top_level_token_no_wrapper() {
        // Credentials file may store accessToken at top level (oauth field missing)
        let now: u64 = 1_700_000_000_000;
        let j = r#"{"accessToken":"top-level-token","expiresAt": 9999999999999}"#;
        let res = credentials_from_text(j, now).unwrap();
        assert_eq!(res.0, "top-level-token");
    }

    #[test]
    fn credentials_empty_token_is_invalid() {
        let j = r#"{"claudeAiOauth":{"accessToken":"","expiresAt": 9999999999999}}"#;
        assert!(credentials_from_text(j, 0).is_none());
    }
}
