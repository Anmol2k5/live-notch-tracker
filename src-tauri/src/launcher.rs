use std::path::PathBuf;

#[cfg_attr(not(windows), allow(dead_code))]
pub fn app_candidates(
    id: &str,
    local: &str,
    programs: &str,
    programs86: &str,
) -> Result<Vec<PathBuf>, String> {
    let relative = match id {
        "chrome" => "Google/Chrome/Application/chrome.exe",
        "discord" => return Ok(vec![PathBuf::from(local).join("Discord/Update.exe")]),
        "spotify" => return Ok(vec![PathBuf::from(local).join("Spotify/Spotify.exe")]),
        "obs" => "obs-studio/bin/64bit/obs64.exe",
        _ => return Err("Unknown app shortcut.".into()),
    };
    Ok([programs, programs86, local]
        .iter()
        .map(|root| PathBuf::from(root).join(relative))
        .collect())
}

#[cfg(windows)]
pub fn launch(id: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let local =
        std::env::var("LOCALAPPDATA").map_err(|_| "Windows app directory is unavailable.")?;
    let programs = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    let programs86 =
        std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
    let roaming = std::env::var("APPDATA").unwrap_or_default();
    let mut candidates = app_candidates(id, &local, &programs, &programs86)?;
    if id == "spotify" && !roaming.is_empty() {
        candidates.insert(0, PathBuf::from(roaming).join("Spotify/Spotify.exe"));
    }
    let path = candidates.into_iter().find(|p| p.is_file())
        .ok_or("App not found. Install the desktop version in its default Windows location, then try again.")?;
    let mut command = std::process::Command::new(&path);
    if id == "discord" {
        command.args(["--processStart", "Discord.exe"]);
    }
    // OBS resolves its data directories relative to its executable directory.
    if let Some(parent) = path.parent() {
        command.current_dir(parent);
    }
    command
        .creation_flags(0x0800_0000)
        .spawn()
        .map_err(|_| "Windows could not open this app.".to_string())?;
    Ok(())
}

#[cfg(not(windows))]
pub fn launch(_id: &str) -> Result<(), String> {
    Err("App shortcuts are available on Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_ids_cannot_be_used_as_commands() {
        for id in [
            "cmd.exe",
            "chrome & calc",
            "../app.exe",
            "",
            "https://example.com",
        ] {
            assert!(app_candidates(
                id,
                "C:/Users/test",
                "C:/Program Files",
                "C:/Program Files (x86)"
            )
            .is_err());
        }
    }

    #[test]
    fn fixed_apps_resolve_only_to_known_executables() {
        let chrome = app_candidates("chrome", "C:/Local", "C:/Programs", "C:/Programs86").unwrap();
        assert!(chrome
            .iter()
            .all(|p| p.ends_with("Google/Chrome/Application/chrome.exe")));
        let discord =
            app_candidates("discord", "C:/Local", "C:/Programs", "C:/Programs86").unwrap();
        assert_eq!(
            discord,
            vec![std::path::PathBuf::from("C:/Local/Discord/Update.exe")]
        );
    }
}
