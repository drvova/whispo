use crate::types::ActiveApplication;
use anyhow::{Context, Result};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get active application on Linux using xdotool/xprop
pub fn get_active_application() -> Result<ActiveApplication> {
    // Try xdotool first (more reliable)
    if let Ok(app) = get_active_app_xdotool() {
        return Ok(app);
    }

    // Fallback to xprop
    get_active_app_xprop()
}

fn get_active_app_xdotool() -> Result<ActiveApplication> {
    // Get window ID
    let window_output = Command::new("xdotool")
        .args(&["getactivewindow"])
        .output()
        .context("Failed to run xdotool")?;

    if !window_output.status.success() {
        anyhow::bail!("xdotool failed");
    }

    let window_id = String::from_utf8_lossy(&window_output.stdout).trim().to_string();

    // Get window name
    let name_output = Command::new("xdotool")
        .args(&["getwindowname", &window_id])
        .output()
        .context("Failed to get window name")?;

    let title = if name_output.status.success() {
        String::from_utf8_lossy(&name_output.stdout).trim().to_string()
    } else {
        String::new()
    };

    // Get PID
    let pid_output = Command::new("xdotool")
        .args(&["getwindowpid", &window_id])
        .output()
        .context("Failed to get window PID")?;

    if !pid_output.status.success() {
        anyhow::bail!("Failed to get PID");
    }

    let pid = String::from_utf8_lossy(&pid_output.stdout).trim().to_string();

    // Get process name from PID
    let exe_path = std::fs::read_link(format!("/proc/{}/exe", pid))
        .unwrap_or_else(|_| std::path::PathBuf::from("unknown"));

    let executable = exe_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let app_name = executable.clone();

    Ok(ActiveApplication {
        name: app_name,
        executable,
        title,
        last_updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    })
}

fn get_active_app_xprop() -> Result<ActiveApplication> {
    let output = Command::new("xprop")
        .args(&["-root", "_NET_ACTIVE_WINDOW"])
        .output()
        .context("Failed to run xprop")?;

    if !output.status.success() {
        anyhow::bail!("xprop failed");
    }

    let result = String::from_utf8_lossy(&output.stdout);
    let window_id = result
        .split_whitespace()
        .last()
        .context("No window ID")?;

    // Get window properties
    let prop_output = Command::new("xprop")
        .args(&["-id", window_id])
        .output()
        .context("Failed to get window properties")?;

    let props = String::from_utf8_lossy(&prop_output.stdout);

    let title = props
        .lines()
        .find(|line| line.starts_with("WM_NAME"))
        .and_then(|line| line.split('"').nth(1))
        .unwrap_or("")
        .to_string();

    let executable = props
        .lines()
        .find(|line| line.starts_with("WM_CLASS"))
        .and_then(|line| line.split('"').nth(1))
        .unwrap_or("unknown")
        .to_string();

    Ok(ActiveApplication {
        name: executable.clone(),
        executable,
        title,
        last_updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    })
}
