use crate::types::ActiveApplication;
use anyhow::{Context, Result};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get active application on macOS using AppleScript
pub fn get_active_application() -> Result<ActiveApplication> {
    let script = r#"
        tell application "System Events"
            set frontApp to first application process whose frontmost is true
            set appName to name of frontApp
            set appPath to POSIX path of (application file of frontApp as text)
            set windowTitle to ""
            try
                set windowTitle to name of front window of frontApp
            end try
            return appName & "|" & appPath & "|" & windowTitle
        end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("Failed to execute osascript")?;

    if !output.status.success() {
        anyhow::bail!("osascript failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let result = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = result.trim().split('|').collect();

    if parts.len() < 3 {
        anyhow::bail!("Invalid osascript output");
    }

    let executable = parts[1]
        .split('/')
        .last()
        .unwrap_or("unknown")
        .to_string();

    Ok(ActiveApplication {
        name: parts[0].to_string(),
        executable,
        title: parts[2].to_string(),
        last_updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    })
}

/// Check accessibility permissions on macOS
pub fn is_accessibility_granted() -> bool {
    let script = r#"
        tell application "System Events"
            try
                set frontApp to first application process whose frontmost is true
                return "true"
            on error
                return "false"
            end try
        end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok();

    if let Some(output) = output {
        let result = String::from_utf8_lossy(&output.stdout);
        result.trim() == "true"
    } else {
        false
    }
}

/// Request accessibility access on macOS
pub fn request_accessibility_access() -> bool {
    // On macOS, we can't programmatically grant permissions
    // We can only check and direct user to System Preferences
    // Return current status
    is_accessibility_granted()
}

/// Get microphone permission status on macOS
pub fn get_microphone_status() -> String {
    // Check microphone permission using tccutil
    let output = Command::new("sqlite3")
        .arg(format!(
            "{}/Library/Application Support/com.apple.TCC/TCC.db",
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        ))
        .arg("SELECT service FROM access WHERE service='kTCCServiceMicrophone' AND allowed=1;")
        .output();

    match output {
        Ok(output) if output.status.success() && !output.stdout.is_empty() => {
            "granted".to_string()
        }
        _ => "denied".to_string(),
    }
}

/// Request microphone access on macOS
pub async fn request_microphone_access() -> bool {
    // macOS handles microphone permissions automatically when accessed
    // We return true to indicate the request was made
    // Actual permission will be shown by system dialog
    true
}
