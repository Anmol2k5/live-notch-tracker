use crate::{github_calendar, launcher};

#[tauri::command]
pub async fn get_github_calendar(username: String) -> Result<serde_json::Value, String> {
    let username = username.trim().to_string();
    if !github_calendar::valid_username(&username) {
        return Err("Enter a valid public GitHub username.".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(15)).build();
        let html = agent.get(&format!("https://github.com/users/{username}/contributions"))
            .set("User-Agent", "Codenotch-Windows")
            .set("Accept-Language", "en-US")
            .call().map_err(|e| match e {
                ureq::Error::Status(404, _) => "GitHub user not found. Check the username.".to_string(),
                _ => "GitHub is unavailable or rate limited. Try refreshing in a moment.".to_string(),
            })?.into_string().map_err(|_| "Could not read GitHub's response.".to_string())?;
        let days = github_calendar::parse_calendar(&html)?;
        Ok(serde_json::json!({ "username": username, "days": days.iter().map(|d|
            serde_json::json!({"date": d.date, "count": d.count, "level": d.level})).collect::<Vec<_>>() }))
    }).await.map_err(|_| "GitHub request was interrupted.".to_string())?
}

#[tauri::command]
pub fn launch_app(id: String) -> Result<(), String> {
    launcher::launch(&id)
}
