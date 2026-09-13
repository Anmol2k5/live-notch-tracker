//! Antigravity (Google's IDE, Gemini quota) usage adapter, implemented from the upstream
//! Codenotch's documented behaviour.
//!
//! Google publishes no usage numbers to third parties: `cloudcode-pa`'s `retrieveUserQuotaSummary`
//! answers a personal account with 403 #3501 "no valid license" (it looks at *who* is asking, and
//! impersonating Antigravity is off the table). Upstream's answer is Antigravity's own answer: ask
//! the language_server running on this machine — it holds the credential and the client identity
//! and asks Google itself.
//!
//! Data paths, most honest first (same as upstream):
//!   1. Local bridge: find the `language_server*` process (its command line carries
//!      `--csrf_token <t>`; the port is `--https_server_port 0`, i.e. random at runtime, and can
//!      only be found in the listening table; it opens two ports and only one answers this RPC,
//!      so both are tried),
//!      POST `https://127.0.0.1:<port>/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary`
//!      with header `x-codeium-csrf-token: <t>` (Antigravity sits on the Codeium stack; the header
//!      name never changed) and body `{"forceRefresh":true}` (otherwise the server answers from
//!      QuotaSummaryCache). Self-signed certificate → verification is relaxed for 127.0.0.1 only.
//!      Reply `{response:{groups:[{displayName, buckets:[{bucketId, displayName, remainingFraction, resetTime}]}]}}`
//!      — it reports what **remains**, so used = 1 - remainingFraction; the label is
//!      group.displayName (buckets only ever say "Weekly Limit Remaining").
//!   2. Bridge answered before and does not now = Antigravity is closed (the port changes on every
//!      launch): keep the last percentage marked stale rather than switching to a count.
//!   3. Credential path (when a Google token exists): Windows Credential Manager target
//!      `gemini:antigravity` (Go keyring: service:user), value JSON
//!      `{auth_method, token:{access_token, expiry (RFC3339 with offset)}}`; on macOS it carries a
//!      `go-keyring-base64:` prefix, and both forms are accepted. POST `:loadCodeAssist`
//!      (`{"metadata":{"pluginType":"GEMINI"}}`, not ANTIGRAVITY) for the tier name; then try
//!      `:retrieveUserQuotaSummary` (empty body `{}`), which is 200 only for licensed accounts, and
//!      parse it defensively (no positive limit, or used > 1.5×limit → discard).
//!   4. Fallback: count today's `source=="MODEL"` steps in
//!      `~/.gemini/antigravity/brain/*/.system_generated/logs/transcript.jsonl` (created_at is UTC,
//!      compared by local day). This is a **count, not a percentage** — there is no published
//!      denominator, so the ring draws only its track.
//!
//! Read only; token values are never cached and never appear in any log.

use crate::usage::{LimitWindow, UsageSnapshot};
use crate::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

const POLL_SECS: u64 = 300;
const LOAD_CODE_ASSIST: &str = "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";
const QUOTA_SUMMARY: &str = "https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary";
const LS_SERVICE: &str = "/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary";
const CSRF_HEADER: &str = "x-codeium-csrf-token";

static REFRESH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn request_refresh() {
    REFRESH.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn state_root() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".gemini").join("antigravity"))
}

fn store_path() -> PathBuf {
    crate::data_dir().with_file_name("antigravity.json")
}

pub fn load_persisted() -> UsageSnapshot {
    std::fs::read_to_string(store_path())
        .ok()
        .and_then(|t| serde_json::from_str::<UsageSnapshot>(&t).ok())
        .map(|mut s| {
            if !s.windows.is_empty() {
                s.status = "stale".into();
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

/// Is Antigravity installed: the state directory exists, or Credential Manager holds its token
pub fn present() -> bool {
    state_root().map(|p| p.is_dir()).unwrap_or(false) || read_credential_raw().is_some()
}

// ---------------- 1. Local bridge ----------------

#[derive(Clone, Debug, PartialEq)]
struct Endpoint {
    ports: Vec<u16>,
    csrf: String,
}

fn run_hidden(program: &str, args: &[&str]) -> String {
    let mut cmd = std::process::Command::new(program);
    cmd.args(args).stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd.output().map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default()
}

/// The process table is the only source of truth: the token is on the command line and the port is written nowhere
#[cfg(windows)]
fn discover() -> Option<Endpoint> {
    // PowerShell CIM query: one "pid<TAB>commandline" per line
    let table = run_hidden(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-CimInstance Win32_Process -Filter \"Name LIKE '%language_server%'\" | ForEach-Object { \"$($_.ProcessId)`t$($_.CommandLine)\" }",
        ],
    );
    let line = table.lines().find(|l| l.contains("--csrf_token"))?;
    let (pid_s, cmdline) = line.split_once('\t')?;
    let pid: u32 = pid_s.trim().parse().ok()?;
    let csrf = flag_value(cmdline, "--csrf_token")?;
    let ports = listening_ports(pid);
    if ports.is_empty() {
        return None;
    }
    Some(Endpoint { ports, csrf })
}
#[cfg(not(windows))]
fn discover() -> Option<Endpoint> {
    let table = run_hidden("ps", &["-Ao", "pid,command"]);
    let line = table.lines().find(|l| l.contains("language_server") && l.contains("--csrf_token"))?;
    let pid: u32 = line.trim().split_whitespace().next()?.parse().ok()?;
    let csrf = flag_value(line, "--csrf_token")?;
    let out = run_hidden("lsof", &["-nP", "-a", "-p", &pid.to_string(), "-iTCP", "-sTCP:LISTEN"]);
    let ports: Vec<u16> = out
        .lines()
        .filter_map(|l| l.split_whitespace().rev().find(|w| w.contains(':')))
        .filter_map(|a| a.rsplit(':').next()?.parse().ok())
        .collect();
    if ports.is_empty() {
        return None;
    }
    Some(Endpoint { ports, csrf })
}

fn flag_value(line: &str, flag: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let i = parts.iter().position(|p| *p == flag)?;
    parts.get(i + 1).map(|s| s.trim_matches('"').to_string())
}

/// netstat -ano: `TCP 127.0.0.1:PORT 0.0.0.0:0 LISTENING PID`
#[cfg(windows)]
fn listening_ports(pid: u32) -> Vec<u16> {
    let out = run_hidden("netstat", &["-ano", "-p", "TCP"]);
    let pid_s = pid.to_string();
    let mut ports: Vec<u16> = out
        .lines()
        .filter(|l| l.contains("LISTENING"))
        .filter_map(|l| {
            let cols: Vec<&str> = l.split_whitespace().collect();
            if cols.len() < 5 || cols[4] != pid_s {
                return None;
            }
            cols[1].rsplit(':').next()?.parse::<u16>().ok()
        })
        .collect();
    ports.sort_unstable();
    ports.dedup();
    ports
}

/// Loopback only: the self-signed certificate is accepted for 127.0.0.1 alone (never used for any public request)
fn local_agent() -> Option<ureq::Agent> {
    let tls = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .ok()?;
    Some(ureq::AgentBuilder::new().tls_connector(Arc::new(tls)).timeout(Duration::from_secs(10)).build())
}

fn bridge_quota(ep: &Endpoint) -> Result<Vec<LimitWindow>, String> {
    let agent = local_agent().ok_or("TLS setup failed")?;
    let mut last = String::from("no port answered");
    for port in &ep.ports {
        let url = format!("https://127.0.0.1:{port}{LS_SERVICE}");
        match agent
            .post(&url)
            .set("Content-Type", "application/json")
            .set(CSRF_HEADER, &ep.csrf)
            .send_string(r#"{"forceRefresh":true}"#)
        {
            Ok(r) => match r.into_json::<serde_json::Value>() {
                Ok(v) => {
                    let w = windows_from_bridge(&v);
                    if !w.is_empty() {
                        return Ok(w);
                    }
                    last = format!("port {port}: no recognisable groups");
                }
                Err(e) => last = format!("port {port}: {e}"),
            },
            Err(ureq::Error::Status(code, _)) => last = format!("port {port}: HTTP {code}"),
            Err(e) => last = format!("port {port}: {e}"),
        }
    }
    Err(last)
}

fn parse_iso(v: Option<&serde_json::Value>) -> Option<u64> {
    v.and_then(|x| x.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.timestamp_millis().max(0) as u64)
}

/// The server reports what remains and the notch shows what is used: flip it here so the view never learns about provider differences
pub fn windows_from_bridge(v: &serde_json::Value) -> Vec<LimitWindow> {
    let mut out = Vec::new();
    let Some(groups) = v.pointer("/response/groups").and_then(|g| g.as_array()) else { return out };
    for g in groups {
        let gname = g.get("displayName").and_then(|x| x.as_str());
        let Some(buckets) = g.get("buckets").and_then(|b| b.as_array()) else { continue };
        for b in buckets {
            let Some(rem) = b.get("remainingFraction").and_then(|x| x.as_f64()) else { continue };
            if !(0.0..=1.0).contains(&rem) {
                continue;
            }
            let bname = b.get("displayName").and_then(|x| x.as_str());
            out.push(LimitWindow {
                id: b.get("bucketId").and_then(|x| x.as_str()).or(gname).unwrap_or("quota").to_string(),
                label: gname.or(bname).unwrap_or("Usage").to_string(),
                used: (1.0 - rem).clamp(0.0, 1.0),
                resets_at: parse_iso(b.get("resetTime")),
                ..Default::default()
            });
        }
    }
    out
}

// ---------------- 3. Credential path ----------------

struct Creds {
    access_token: String,
    expired: bool,
    auth_method: String,
}

/// Windows Credential Manager: generic credential with target = "gemini:antigravity" (Go keyring's service:user naming)
#[cfg(windows)]
fn read_credential_raw() -> Option<Vec<u8>> {
    use windows::core::PCWSTR;
    use windows::Win32::Security::Credentials::{CredFree, CredReadW, CREDENTIALW, CRED_TYPE_GENERIC};
    let target: Vec<u16> = "gemini:antigravity".encode_utf16().chain(std::iter::once(0)).collect();
    let mut pcred: *mut CREDENTIALW = std::ptr::null_mut();
    unsafe {
        if CredReadW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, Some(0), &mut pcred).is_err() || pcred.is_null() {
            return None;
        }
        let c = &*pcred;
        let blob = if c.CredentialBlobSize > 0 && !c.CredentialBlob.is_null() {
            std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize).to_vec()
        } else {
            Vec::new()
        };
        CredFree(pcred as *const core::ffi::c_void);
        if blob.is_empty() {
            None
        } else {
            Some(blob)
        }
    }
}
#[cfg(not(windows))]
fn read_credential_raw() -> Option<Vec<u8>> {
    None
}

/// Raw JSON, or base64 with a `go-keyring-base64:` prefix (UTF-16 storage is accepted too)
fn decode_credential(raw: &[u8]) -> Option<Creds> {
    let mut text = String::from_utf8(raw.to_vec()).unwrap_or_else(|_| {
        // Some writers store the blob as UTF-16LE
        let u16s: Vec<u16> = raw.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&u16s)
    });
    text = text.trim_matches('\0').trim().to_string();
    if let Some(rest) = text.strip_prefix("go-keyring-base64:") {
        let bytes = b64_decode(rest.trim())?;
        text = String::from_utf8(bytes).ok()?;
    }
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let access = v.pointer("/token/access_token")?.as_str()?.to_string();
    let expiry = v.pointer("/token/expiry").and_then(|x| x.as_str()).unwrap_or("");
    let expired = chrono::DateTime::parse_from_rfc3339(expiry)
        .map(|d| (d.timestamp_millis().max(0) as u64) <= now_ms())
        .unwrap_or(false);
    let auth_method = v.get("auth_method").and_then(|x| x.as_str()).unwrap_or("").to_string();
    Some(Creds { access_token: access, expired, auth_method })
}

/// Dependency-free base64 (standard alphabet, tolerant of URL-safe characters and missing padding)
pub(crate) fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0u8;
    for c in s.bytes() {
        let sextet: u8 = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\n' | b'\r' | b' ' => continue,
            _ => return None,
        };
        let v = sextet as u32;
        buf = (buf << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

fn read_credentials() -> Option<Creds> {
    decode_credential(&read_credential_raw()?)
}

/// Tier name ("Personal"/"Pro"…); 401/403 → NeedsAuth
fn load_tier(token: &str) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).build();
    match agent
        .post(LOAD_CODE_ASSIST)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/json")
        .send_string(r#"{"metadata":{"pluginType":"GEMINI"}}"#)
    {
        Ok(r) => {
            let v: serde_json::Value = r.into_json().map_err(|e| e.to_string())?;
            let tier = v
                .get("currentTier")
                .or_else(|| {
                    v.get("allowedTiers").and_then(|a| a.as_array()).and_then(|a| {
                        a.iter().find(|t| t.get("isDefault").and_then(|x| x.as_bool()) == Some(true)).or(a.first())
                    })
                })
                .and_then(|t| t.get("name"))
                .and_then(|x| x.as_str())
                .unwrap_or("Gemini");
            Ok(tier.to_string())
        }
        Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => Err("needsAuth".into()),
        Err(ureq::Error::Status(code, _)) => Err(format!("HTTP {code}")),
        Err(e) => Err(e.to_string()),
    }
}

/// Direct quota for licensed accounts; a personal account gets 403 → None (not an error)
fn direct_quota(token: &str) -> Option<Vec<LimitWindow>> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).build();
    let r = agent
        .post(QUOTA_SUMMARY)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/json")
        .send_string("{}")
        .ok()?;
    let v: serde_json::Value = r.into_json().ok()?;
    let mut buckets: Vec<serde_json::Value> = Vec::new();
    if let Some(groups) = v.get("quotaGroups").and_then(|g| g.as_array()) {
        for g in groups {
            if let Some(bs) = g.get("buckets").and_then(|b| b.as_array()) {
                buckets.extend(bs.iter().cloned());
            }
        }
    }
    if let Some(bs) = v.get("buckets").and_then(|b| b.as_array()) {
        buckets.extend(bs.iter().cloned());
    }
    let out: Vec<LimitWindow> = buckets
        .iter()
        .filter_map(|b| {
            let limit = b.get("limit").and_then(|x| x.as_f64())?;
            let used = b.get("used").and_then(|x| x.as_f64())?;
            if limit <= 0.0 || used < 0.0 || used > limit * 1.5 {
                return None; // defensive: a reply of the wrong shape draws no ring
            }
            let label = b
                .get("displayName")
                .or_else(|| b.get("name"))
                .and_then(|x| x.as_str())
                .unwrap_or("Usage")
                .to_string();
            Some(LimitWindow {
                id: b.get("name").and_then(|x| x.as_str()).unwrap_or(&label).to_string(),
                label,
                used: (used / limit).clamp(0.0, 1.0),
                resets_at: parse_iso(b.get("resetTime")),
                ..Default::default()
            })
        })
        .collect();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

// ---------------- 4. Fallback count ----------------

/// Pure parsing/counting of transcript JSONL text — extracted for testing.
/// Returns (today_count, latest_ms) where latest is the newest timestamp of any MODEL line.
/// `today` is the Local date to compare against (UTC timestamps are converted to Local).
pub(crate) fn count_requests_in_text(text: &str, today: chrono::NaiveDate) -> (u64, Option<u64>) {
    use chrono::{Datelike, Local, TimeZone};
    let mut count = 0u64;
    let mut latest: Option<u64> = None;
    for line in text.lines() {
        if !line.contains("\"MODEL\"") {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        if v.get("source").and_then(|x| x.as_str()) != Some("MODEL") {
            continue;
        }
        let Some(ts) = v.get("created_at").and_then(|x| x.as_str()) else { continue };
        let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) else { continue };
        let ms = dt.timestamp_millis().max(0) as u64;
        latest = Some(latest.map_or(ms, |l| l.max(ms)));
        if let Some(local) = Local.timestamp_millis_opt(ms as i64).single() {
            if local.date_naive() == today {
                count += 1;
            }
        }
        let _ = today.year();
    }
    (count, latest)
}

/// Today's MODEL steps (UTC timestamps compared by local day)
pub fn requests_today() -> (u64, Option<u64>) {
    use chrono::Local;
    let Some(root) = state_root().map(|r| r.join("brain")) else { return (0, None) };
    let Ok(rd) = std::fs::read_dir(&root) else { return (0, None) };
    let today = Local::now().date_naive();
    let mut total = 0u64;
    let mut latest: Option<u64> = None;
    for e in rd.flatten() {
        let p = e.path().join(".system_generated").join("logs").join("transcript.jsonl");
        let Ok(text) = std::fs::read_to_string(&p) else { continue };
        let (c, l) = count_requests_in_text(&text, today);
        total += c;
        if let Some(ms) = l {
            latest = Some(latest.map_or(ms, |prev| prev.max(ms)));
        }
    }
    (total, latest)
}

// ---------------- Putting it together ----------------

struct Runtime {
    endpoint: Option<Endpoint>,
    ever_bridged: bool,
}

fn read_once(rt: &mut Runtime, prev: &UsageSnapshot) -> UsageSnapshot {
    let mut snap = UsageSnapshot::default();
    // 1. Local bridge (the cached endpoint first; the port changes on every launch, so a miss is normal)
    let mut bridge_err = String::new();
    let mut tried = false;
    if let Some(ep) = rt.endpoint.clone() {
        tried = true;
        match bridge_quota(&ep) {
            Ok(w) => {
                rt.ever_bridged = true;
                snap.status = "ok".into();
                snap.windows = w;
                snap.fetched_at = now_ms();
                snap.note = "via Antigravity".into();
                return snap;
            }
            Err(e) => {
                bridge_err = e;
                rt.endpoint = None;
            }
        }
    }
    if let Some(ep) = discover() {
        tried = true;
        match bridge_quota(&ep) {
            Ok(w) => {
                rt.endpoint = Some(ep);
                rt.ever_bridged = true;
                snap.status = "ok".into();
                snap.windows = w;
                snap.fetched_at = now_ms();
                snap.note = "via Antigravity".into();
                return snap;
            }
            Err(e) => bridge_err = e,
        }
    }
    if tried && !bridge_err.is_empty() {
        println!("LOG: {}", &format!("antigravity: local bridge failed ({bridge_err})"));
    }
    // 2. The bridge worked before: keep the last percentage marked stale instead of degrading to a count (8% → 31 looks broken)
    if rt.ever_bridged && !prev.windows.is_empty() {
        snap = prev.clone();
        snap.status = "stale".into();
        snap.note = "Antigravity is closed — last reading kept".into();
        return snap;
    }
    // 3. Credential path
    let mut tier: Option<String> = None;
    match read_credentials() {
        Some(c) if !c.expired => match load_tier(&c.access_token) {
            Ok(t) => {
                tier = Some(t);
                if let Some(w) = direct_quota(&c.access_token) {
                    snap.status = "ok".into();
                    snap.windows = w;
                    snap.fetched_at = now_ms();
                    snap.note = format!("{} · via Google", tier.clone().unwrap_or_default());
                    return snap;
                }
            }
            Err(e) if e == "needsAuth" => {
                snap.status = "needsAuth".into();
                snap.note = "Antigravity's Google session was rejected — sign in again in Antigravity".into();
                return snap;
            }
            Err(e) => println!("LOG: {}", &format!("antigravity: loadCodeAssist {e}")),
        },
        Some(c) => {
            // Expired ≠ signed out: Antigravity refreshes it on its next run; auth_method stands in for the tier
            tier = Some(if c.auth_method == "consumer" { "Personal".into() } else { c.auth_method.clone() });
        }
        None => {}
    }
    // 4. Count fallback (derived: the card gets a ~ prefix and the ring draws only its track)
    let (n, latest) = requests_today();
    snap.status = "ok".into();
    snap.fetched_at = latest.unwrap_or_else(now_ms);
    snap.windows = vec![LimitWindow {
        id: "requests".into(),
        label: "Requests today · no limit published".into(),
        used: 0.0,
        resets_at: None,
        count: Some(n as i64),
        derived: true,
    }];
    snap.note = match tier {
        Some(t) => format!("{t} · Google publishes no quota for this account"),
        None => "Open Antigravity to read its quota".into(),
    };
    snap
}

fn broadcast(app: &AppHandle, snap: UsageSnapshot) {
    let st = app.state::<AppState>();
    *st.antigravity.lock().unwrap_or_else(|e| e.into_inner()) = snap.clone();
    persist(&snap);
    let _ = app.emit("antigravity", &snap);
}

fn sleep_interruptible(secs: u64) {
    for _ in 0..secs {
        if REFRESH.swap(false, std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        {
            let st = app.state::<AppState>();
            let snap = st.antigravity.lock().unwrap_or_else(|e| e.into_inner()).clone();
            let _ = app.emit("antigravity", &snap);
        }
        if !present() {
            broadcast(&app, UsageSnapshot { status: "absent".into(), ..Default::default() });
            loop {
                sleep_interruptible(600);
                if present() {
                    break;
                }
            }
        }
        let mut rt = Runtime { endpoint: None, ever_bridged: false };
        loop {
            let prev = {
                let st = app.state::<AppState>();
                let s = st.antigravity.lock().unwrap_or_else(|e| e.into_inner()).clone();
                s
            };
            let snap = read_once(&mut rt, &prev);
            broadcast(&app, snap);
            sleep_interruptible(POLL_SECS);
        }
    });
}

/// For doctor: contains no secrets
pub fn probe() -> String {
    let root = state_root().map(|p| p.display().to_string()).unwrap_or_default();
    let has_root = state_root().map(|p| p.is_dir()).unwrap_or(false);
    let cred = read_credentials();
    let ep = discover();
    format!(
        "Antigravity: state dir {} ({}) | Credential Manager gemini:antigravity {} | language_server {}",
        root,
        if has_root { "present" } else { "missing" },
        match cred {
            Some(c) => format!("found ({}, {})", c.auth_method, if c.expired { "expired" } else { "valid" }),
            None => "not found".into(),
        },
        match ep {
            Some(e) => format!("running, ports {:?}", e.ports),
            None => "not running".into(),
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---- windows_from_bridge: remainingFraction → used% conversion ----

    #[test]
    fn bridge_normal_remaining_to_used() {
        let v = json!({
            "response": {
                "groups": [
                    {
                        "displayName": "Gemini",
                        "buckets": [
                            { "bucketId": "weekly", "displayName": "Weekly Limit Remaining", "remainingFraction": 0.75, "resetTime": "2026-09-20T00:00:00Z" },
                            { "bucketId": "daily", "displayName": "Daily Limit Remaining", "remainingFraction": 0.20, "resetTime": "2026-09-13T00:00:00Z" }
                        ]
                    }
                ]
            }
        });
        let w = windows_from_bridge(&v);
        assert_eq!(w.len(), 2);
        // used = 1 - remaining
        assert!((w[0].used - 0.25).abs() < 1e-9, "remaining 0.75 -> used 0.25");
        assert!((w[1].used - 0.80).abs() < 1e-9, "remaining 0.20 -> used 0.80");
        assert_eq!(w[0].id, "weekly");
        assert_eq!(w[0].label, "Gemini"); // group displayName wins
        assert!(w[0].resets_at.is_some());
    }

    #[test]
    fn bridge_remaining_out_of_range_skipped() {
        let v = json!({
            "response": {
                "groups": [
                    {
                        "displayName": "Gemini",
                        "buckets": [
                            { "bucketId": "bad_high", "remainingFraction": 1.5, "resetTime": "2026-09-20T00:00:00Z" },
                            { "bucketId": "bad_low", "remainingFraction": -0.1, "resetTime": "2026-09-20T00:00:00Z" },
                            { "bucketId": "ok", "remainingFraction": 0.5, "resetTime": "2026-09-20T00:00:00Z" }
                        ]
                    }
                ]
            }
        });
        let w = windows_from_bridge(&v);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].id, "ok");
        assert!((w[0].used - 0.5).abs() < 1e-9);
    }

    #[test]
    fn bridge_missing_fields_no_panic() {
        let v = json!({});
        assert!(windows_from_bridge(&v).is_empty());
        let v2 = json!({ "response": {} });
        assert!(windows_from_bridge(&v2).is_empty());
        let v3 = json!({ "response": { "groups": "not-an-array" } });
        assert!(windows_from_bridge(&v3).is_empty());
        let v4 = json!({
            "response": { "groups": [{ "displayName": "G", "buckets": [{ "remainingFraction": 0.5 }] }] }
        });
        let w = windows_from_bridge(&v4);
        assert_eq!(w.len(), 1);
        assert!(w[0].resets_at.is_none());
    }

    #[test]
    fn bridge_clamps_used_and_handles_zero_one() {
        let v = json!({
            "response": {
                "groups": [
                    {
                        "displayName": "X",
                        "buckets": [
                            { "bucketId": "full", "remainingFraction": 0.0, "resetTime": "2026-09-13T00:00:00Z" },
                            { "bucketId": "empty", "remainingFraction": 1.0, "resetTime": "2026-09-13T00:00:00Z" }
                        ]
                    }
                ]
            }
        });
        let w = windows_from_bridge(&v);
        assert!((w[0].used - 1.0).abs() < 1e-9);
        assert!((w[1].used - 0.0).abs() < 1e-9);
    }

    // ---- Credential Manager JSON value parser (plain + base64) ----

    #[test]
    fn credential_plain_valid() {
        let raw = br#"{"auth_method":"consumer","token":{"access_token":"ya29.testtoken123","expiry":"2099-09-13T12:00:00+02:00"}}"#;
        let c = decode_credential(raw).expect("should parse plain JSON");
        assert_eq!(c.access_token, "ya29.testtoken123");
        assert_eq!(c.auth_method, "consumer");
        assert!(!c.expired, "far-future expiry should not be expired");
    }

    #[test]
    fn credential_plain_expired() {
        let raw = br#"{"auth_method":"api_key","token":{"access_token":"tok","expiry":"2000-01-01T00:00:00Z"}}"#;
        let c = decode_credential(raw).expect("should parse");
        assert!(c.expired);
        assert_eq!(c.auth_method, "api_key");
    }

    #[test]
    fn credential_base64_prefixed() {
        let json = r#"{"auth_method":"consumer","token":{"access_token":"base64token","expiry":"2099-01-01T00:00:00Z"}}"#;
        let b64 = {
            let bytes = json.as_bytes();
            // Use our own b64_decode's counterpart: encode via simple base64 for test
            let mut out = String::new();
            const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let mut buf: u32 = 0;
            let mut bits: u8 = 0;
            let mut count = 0;
            for &b in bytes {
                buf = (buf << 8) | b as u32;
                bits += 8;
                while bits >= 6 {
                    bits -= 6;
                    out.push(TABLE[((buf >> bits) & 0x3F) as usize] as char);
                    count += 1;
                }
            }
            if bits > 0 {
                out.push(TABLE[((buf << (6 - bits)) & 0x3F) as usize] as char);
                count += 1;
            }
            while count % 4 != 0 {
                out.push('=');
                count += 1;
            }
            out
        };
        let prefixed = format!("go-keyring-base64:{b64}");
        let c = decode_credential(prefixed.as_bytes()).expect("should parse base64 prefixed");
        assert_eq!(c.access_token, "base64token");
        assert_eq!(c.auth_method, "consumer");
    }

    #[test]
    fn credential_invalid_returns_none() {
        assert!(decode_credential(b"not json").is_none());
        assert!(decode_credential(b"{}").is_none());
        assert!(decode_credential(b"").is_none());
        // missing access_token
        assert!(decode_credential(br#"{"auth_method":"x","token":{"expiry":"2099-01-01T00:00:00Z"}}"#).is_none());
    }

    #[test]
    fn b64_decode_tolerates_url_safe_and_missing_padding() {
        // "Hello" -> "SGVsbG8=" ; without padding "SGVsbG8" and url-safe variant
        let decoded = b64_decode("SGVsbG8").expect("missing padding should still decode");
        assert_eq!(decoded, b"Hello");
        let decoded2 = b64_decode("SGVsbG8=").expect("with padding");
        assert_eq!(decoded2, b"Hello");
        // url-safe: '+' -> '-', '/' -> '_' — our decoder accepts both
        let hello_url = b64_decode("SGVs-b8=").or_else(|| b64_decode("SGVs/b8="));
        assert!(hello_url.is_some());
        assert!(b64_decode("!!!invalid!!!").is_none());
    }

    #[test]
    fn b64_decode_empty() {
        assert_eq!(b64_decode("").unwrap(), b"");
    }

    // ---- transcript JSONL line-counting logic ----

    fn today() -> chrono::NaiveDate {
        use chrono::Local;
        Local::now().date_naive()
    }

    #[test]
    fn transcript_counts_model_today() {
        use chrono::{Local, Duration};
        let today = today();
        let now = Local::now();
        let today_iso = now.to_rfc3339();
        let yesterday_iso = (now - Duration::days(1)).to_rfc3339();
        let text = format!(
            "{}\n{}\n{}\n{}\n",
            // today MODEL counted
            serde_json::json!({"source":"MODEL","created_at": today_iso}).to_string(),
            // yesterday MODEL not counted (but contributes to latest)
            serde_json::json!({"source":"MODEL","created_at": yesterday_iso}).to_string(),
            // non-MODEL ignored
            serde_json::json!({"source":"USER","created_at": today_iso}).to_string(),
            // malformed line ignored
            "not json at all"
        );
        let (count, latest) = count_requests_in_text(&text, today);
        assert_eq!(count, 1, "only today's MODEL should be counted");
        assert!(latest.is_some());
    }

    #[test]
    fn transcript_counts_multiple_today() {
        use chrono::Local;
        let today = today();
        let iso = Local::now().to_rfc3339();
        let line = serde_json::json!({"source":"MODEL","created_at": iso}).to_string();
        let text = format!("{line}\n{line}\n{line}\n");
        let (count, latest) = count_requests_in_text(&text, today);
        assert_eq!(count, 3);
        assert!(latest.is_some());
    }

    #[test]
    fn transcript_empty_yields_zero() {
        let today = today();
        let (count, latest) = count_requests_in_text("", today);
        assert_eq!(count, 0);
        assert!(latest.is_none());
    }

    #[test]
    fn transcript_malformed_lines_ignored() {
        use chrono::Local;
        let today = today();
        let iso = Local::now().to_rfc3339();
        let text = format!(
            "{}\n{}\n{}\n",
            r#"{"source":"MODEL","created_at":"not-a-date"}"#,
            r#"{"source":"MODEL"}"#, // missing created_at
            serde_json::json!({"source":"MODEL","created_at": iso}).to_string()
        );
        let (count, latest) = count_requests_in_text(&text, today);
        assert_eq!(count, 1);
        assert!(latest.is_some());
    }

    #[test]
    fn transcript_latest_is_max() {
        use chrono::{Local, Duration};
        let today = today();
        let now = Local::now();
        let iso_old = (now - Duration::days(2)).to_rfc3339();
        let iso_new = now.to_rfc3339();
        let text = format!(
            "{}\n{}\n",
            serde_json::json!({"source":"MODEL","created_at": iso_old}).to_string(),
            serde_json::json!({"source":"MODEL","created_at": iso_new}).to_string()
        );
        let (_count, latest) = count_requests_in_text(&text, today);
        let expected = chrono::DateTime::parse_from_rfc3339(&iso_new).unwrap().timestamp_millis().max(0) as u64;
        assert_eq!(latest, Some(expected));
    }

    #[test]
    fn transcript_no_model_yields_none_latest() {
        use chrono::Local;
        let today = today();
        let iso = Local::now().to_rfc3339();
        let text = format!(
            "{}\n",
            serde_json::json!({"source":"USER","created_at": iso}).to_string()
        );
        let (count, latest) = count_requests_in_text(&text, today);
        assert_eq!(count, 0);
        assert!(latest.is_none());
    }
}
