use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::usage::UsageSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    Claude,
    Cursor,
    Codex,
    Antigravity,
}

impl ProviderId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderId::Claude => "claude",
            ProviderId::Cursor => "cursor",
            ProviderId::Codex => "codex",
            ProviderId::Antigravity => "antigravity",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderId::Claude => "Claude Code",
            ProviderId::Cursor => "Cursor",
            ProviderId::Codex => "Codex / ChatGPT",
            ProviderId::Antigravity => "Antigravity",
        }
    }

    pub fn file_name(&self) -> &'static str {
        match self {
            ProviderId::Claude => "usage.json",
            ProviderId::Cursor => "cursor.json",
            ProviderId::Codex => "codex.json",
            ProviderId::Antigravity => "antigravity.json",
        }
    }
}

/// Central application data directory (%APPDATA%/codenotch-win).
/// Ensures the directory exists before returning.
pub fn app_data_dir() -> std::io::Result<PathBuf> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("codenotch-win");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Central provider storage file path: %APPDATA%/codenotch-win/<file_name>.
/// Uses explicit PathBuf::join so the directory path is never replaced.
pub fn provider_data_path(provider: ProviderId) -> std::io::Result<PathBuf> {
    let dir = app_data_dir()?;
    Ok(dir.join(provider.file_name()))
}

/// Load a persisted provider snapshot from disk.
/// Any restored snapshot with windows is labelled "stale" until refreshed.
pub fn load_provider_snapshot(provider: ProviderId) -> UsageSnapshot {
    let Ok(path) = provider_data_path(provider) else {
        return UsageSnapshot::default();
    };

    std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str::<UsageSnapshot>(&t).ok())
        .map(|mut s| {
            if !s.windows.is_empty() {
                s.status = "stale".into();
            }
            // Ensure metrics are populated on all windows
            for w in &mut s.windows {
                if w.metric.is_none() {
                    w.metric = Some(w.resolved_metric());
                }
            }
            s
        })
        .unwrap_or_default()
}

/// Persist a provider snapshot to disk.
pub fn persist_provider_snapshot(provider: ProviderId, s: &UsageSnapshot) {
    if let Ok(path) = provider_data_path(provider) {
        let mut to_save = s.clone();
        for w in &mut to_save.windows {
            if w.metric.is_none() {
                w.metric = Some(w.resolved_metric());
            }
        }
        if let Ok(t) = serde_json::to_string_pretty(&to_save) {
            let _ = std::fs::write(path, t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage::{LimitWindow, UsageMetric};

    #[test]
    fn test_provider_file_names() {
        assert_eq!(ProviderId::Claude.file_name(), "usage.json");
        assert_eq!(ProviderId::Cursor.file_name(), "cursor.json");
        assert_eq!(ProviderId::Codex.file_name(), "codex.json");
        assert_eq!(ProviderId::Antigravity.file_name(), "antigravity.json");
    }

    #[test]
    fn test_provider_data_path_is_inside_dir() {
        let path = provider_data_path(ProviderId::Claude).expect("valid path");
        assert!(path.ends_with("usage.json"));
        let parent = path.parent().expect("has parent");
        assert!(parent.ends_with("codenotch-win"));
    }

    #[test]
    fn test_provider_display_names() {
        assert_eq!(ProviderId::Claude.display_name(), "Claude Code");
        assert_eq!(ProviderId::Cursor.display_name(), "Cursor");
        assert_eq!(ProviderId::Codex.display_name(), "Codex / ChatGPT");
        assert_eq!(ProviderId::Antigravity.display_name(), "Antigravity");
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let temp_dir = std::env::temp_dir().join("codenotch_test_roundtrip");
        let _ = std::fs::create_dir_all(&temp_dir);
        let path = temp_dir.join("test_snap.json");

        let snap = UsageSnapshot {
            status: "ok".into(),
            windows: vec![LimitWindow::count("test", "Test Label", 42, None, None, true)],
            fetched_at: 12345678,
            note: "test note".into(),
            backoff_until: 0,
        };

        let json = serde_json::to_string_pretty(&snap).expect("serialize");
        std::fs::write(&path, &json).expect("write");

        let read_back: UsageSnapshot = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(read_back.status, "ok");
        assert_eq!(read_back.windows.len(), 1);
        let metric = read_back.windows[0].resolved_metric();
        match metric {
            UsageMetric::Count { used, limit, derived, .. } => {
                assert_eq!(used, 42);
                assert_eq!(limit, None);
                assert!(derived);
            }
            _ => panic!("Expected count metric"),
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
