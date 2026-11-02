//! Adapted from vinzdg/codenotch windows/codenotch/src/codex.rs at
//! 6522e9f3ee0fe35d46e428b441e6e80b586eacbb. MIT; see third-party/codenotch.

#[derive(Debug)]
pub struct RpcWindow {
    pub id: &'static str,
    pub used: f64,
    pub minutes: Option<f64>,
    pub resets_at: Option<u64>,
}

pub fn may_query_native(now: u64, http_backoff: u64, native_retry: u64) -> bool {
    http_backoff <= now && native_retry <= now
}

pub fn is_native_executable(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("exe"))
}

pub fn parse_windows(result: &serde_json::Value) -> Option<Vec<RpcWindow>> {
    let core = match result.get("rateLimitsByLimitId").filter(|v| !v.is_null()) {
        Some(buckets) => buckets.get("codex")?,
        None => result.get("rateLimits")?,
    };
    if core
        .get("limitId")
        .and_then(|x| x.as_str())
        .is_some_and(|id| id != "codex")
    {
        return None;
    }
    let mut windows = Vec::new();
    for id in ["primary", "secondary"] {
        let Some(w) = core.get(id).filter(|v| v.is_object()) else {
            continue;
        };
        let Some(used) = w
            .get("usedPercent")
            .and_then(|x| x.as_f64())
            .filter(|n| n.is_finite())
        else {
            continue;
        };
        windows.push(RpcWindow {
            id,
            used: (used / 100.0).clamp(0.0, 1.0),
            minutes: w
                .get("windowDurationMins")
                .and_then(|x| x.as_f64())
                .filter(|n| n.is_finite() && *n > 0.0),
            resets_at: w
                .get("resetsAt")
                .and_then(|x| x.as_u64())
                .map(|s| s.saturating_mul(1000)),
        });
    }
    if windows.is_empty() {
        None
    } else {
        Some(windows)
    }
}

#[cfg(windows)]
pub fn query(executable: &std::path::Path) -> Option<Vec<RpcWindow>> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    let mut command = Command::new(executable);
    command
        .arg("app-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let mut child = command.spawn().ok()?;
    let result = (|| {
        let mut input = child.stdin.take()?;
        let output = child.stdout.take()?;
        let (tx, rx) = std::sync::mpsc::sync_channel(16);
        std::thread::spawn(move || {
            for line in BufReader::new(output).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        writeln!(input, "{}", serde_json::json!({"id":1,"method":"initialize","params":{"clientInfo":{"name":"codenotch","version":env!("CARGO_PKG_VERSION")}}})).ok()?;
        input.flush().ok()?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            let line = rx
                .recv_timeout(deadline.checked_duration_since(std::time::Instant::now())?)
                .ok()?;
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue;
            };
            match v.get("id").and_then(|x| x.as_u64()) {
                Some(1) => {
                    v.get("result")?;
                    writeln!(
                        input,
                        "{}",
                        serde_json::json!({"method":"initialized","params":{}})
                    )
                    .ok()?;
                    writeln!(
                        input,
                        "{}",
                        serde_json::json!({"id":2,"method":"account/rateLimits/read"})
                    )
                    .ok()?;
                    input.flush().ok()?;
                }
                Some(2) => return parse_windows(v.get("result")?),
                _ => {}
            }
        }
    })();
    // This is a directly launched native executable, never a cmd/node wrapper or a running app.
    let _ = child.kill();
    let _ = child.wait();
    result
}

#[cfg(not(windows))]
pub fn query(_executable: &std::path::Path) -> Option<Vec<RpcWindow>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn uses_authoritative_core_bucket_not_spark() {
        let windows = parse_windows(&json!({"rateLimits":{"primary":{"usedPercent":99}},
            "rateLimitsByLimitId":{"codex":{"limitId":"codex","primary":{"usedPercent":32,"windowDurationMins":10080,"resetsAt":1789878630}}}})).unwrap();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].used, 0.32);
        assert_eq!(windows[0].minutes, Some(10080.0));
        assert_eq!(windows[0].resets_at, Some(1789878630000));
    }
    #[test]
    fn missing_or_noncore_usage_never_becomes_a_zero_reading() {
        for value in [
            json!({}),
            json!({"rateLimitsByLimitId":{},"rateLimits":{"primary":{"usedPercent":7}}}),
            json!({"rateLimits":{"limitId":"codex_bengalfox","primary":{"usedPercent":0}}}),
            json!({"rateLimits":{"primary":{"usedPercent":null}}}),
        ] {
            assert!(parse_windows(&value).is_none());
        }
    }
    #[test]
    fn accepts_legacy_core_and_clamps_percentages() {
        let windows = parse_windows(
            &json!({"rateLimits":{"primary":{"usedPercent":-4},"secondary":{"usedPercent":150}}}),
        )
        .unwrap();
        assert_eq!(windows[0].used, 0.0);
        assert_eq!(windows[1].used, 1.0);
    }
    #[test]
    fn recovery_respects_http_backoff_and_native_stand_down() {
        assert!(!may_query_native(100, 200, 0));
        assert!(!may_query_native(100, 0, 200));
        assert!(may_query_native(100, 100, 100));
    }
    #[test]
    fn only_native_executables_are_accepted() {
        assert!(is_native_executable(std::path::Path::new("Codex.EXE")));
        for path in ["codex.cmd", "codex.bat", "node", "codex"] {
            assert!(!is_native_executable(std::path::Path::new(path)));
        }
    }
}
