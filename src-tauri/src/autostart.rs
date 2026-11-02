//! Adapted from vinzdg/codenotch windows/codenotch/src/autostart.rs at
//! 6522e9f3ee0fe35d46e428b441e6e80b586eacbb. MIT; see third-party/codenotch.
//! Own registry name and --background mode; never modifies upstream Codenotch's Run entry.

//! Start at sign-in: an HKCU\...\Run registry value (per user, no administrator needed).
//! The command carries --background: start hidden; the system tray restores the Desk.
//! Implemented with reg.exe, so no new dependency.

#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(windows)]
const NAME: &str = "LiveNotchTracker";

#[cfg(windows)]
fn reg(args: &[&str]) -> Option<(bool, String)> {
    let mut c = Command::new("reg");
    c.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    c.output().ok().map(|o| {
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        (o.status.success(), text)
    })
}

#[cfg(windows)]
pub fn is_enabled() -> Result<bool, String> {
    reg(&["query", RUN_KEY, "/v", NAME])
        .map(|(ok, out)| ok && out.contains(NAME))
        .ok_or_else(|| "Could not query Windows startup settings.".into())
}

#[cfg(windows)]
pub fn enable() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let val = startup_command(&exe)?;
    match reg(&["add", RUN_KEY, "/v", NAME, "/t", "REG_SZ", "/d", &val, "/f"]) {
        Some((true, _)) => Ok("Start with Windows enabled.".into()),
        Some((false, out)) => Err(out),
        None => Err("reg.exe failed to run".into()),
    }
}

#[cfg(windows)]
pub fn disable() -> Result<String, String> {
    match reg(&["delete", RUN_KEY, "/v", NAME, "/f"]) {
        Some((true, _)) => Ok("start at sign-in disabled".into()),
        Some((false, out)) => {
            if out.to_lowercase().contains("unable to find") || out.contains("找不到") {
                // reg.exe answers in the OS language; "找不到" is the Chinese "unable to find"
                Ok("start at sign-in was not enabled".into())
            } else {
                Err(out)
            }
        }
        None => Err("reg.exe failed to run".into()),
    }
}

#[cfg_attr(not(windows), allow(dead_code))]
pub fn startup_command(exe: &std::path::Path) -> Result<String, String> {
    let text = exe
        .to_str()
        .ok_or("Executable path is not valid Unicode.")?;
    if text.is_empty() || text.contains(['"', '\n', '\r']) {
        return Err("Invalid executable path.".into());
    }
    Ok(format!("\"{text}\" --background"))
}

#[cfg(not(windows))]
pub fn is_enabled() -> Result<bool, String> {
    Err("Start with Windows is only available on Windows.".into())
}
#[cfg(not(windows))]
pub fn enable() -> Result<String, String> {
    Err("Start with Windows is only available on Windows.".into())
}
#[cfg(not(windows))]
pub fn disable() -> Result<String, String> {
    Err("Start with Windows is only available on Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quotes_the_executable_and_only_requests_background_start() {
        assert_eq!(
            startup_command(std::path::Path::new(r"C:\Program Files\Desk\app.exe")).unwrap(),
            r#""C:\Program Files\Desk\app.exe" --background"#
        );
        assert!(startup_command(std::path::Path::new("bad\"path.exe")).is_err());
    }
}
