// Platform-specific functionality

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::types::ActiveApplication;
use anyhow::Result;

/// Get the currently active application
pub fn get_active_application() -> Result<ActiveApplication> {
    #[cfg(target_os = "macos")]
    return macos::get_active_application();

    #[cfg(target_os = "windows")]
    return windows::get_active_application();

    #[cfg(target_os = "linux")]
    return linux::get_active_application();
}

/// Check if accessibility permissions are granted
pub fn is_accessibility_granted() -> bool {
    #[cfg(target_os = "macos")]
    return macos::is_accessibility_granted();

    #[cfg(target_os = "windows")]
    return true; // Windows doesn't require special accessibility permission

    #[cfg(target_os = "linux")]
    return true; // Linux doesn't require special accessibility permission
}

/// Request accessibility permissions (macOS only)
pub fn request_accessibility_access() -> bool {
    #[cfg(target_os = "macos")]
    return macos::request_accessibility_access();

    #[cfg(not(target_os = "macos"))]
    return true; // Other platforms don't need this
}

/// Get microphone permission status
pub fn get_microphone_status() -> String {
    #[cfg(target_os = "macos")]
    return macos::get_microphone_status();

    #[cfg(not(target_os = "macos"))]
    return "granted".to_string(); // Other platforms handled by browser/webview
}

/// Request microphone access
pub async fn request_microphone_access() -> bool {
    #[cfg(target_os = "macos")]
    return macos::request_microphone_access().await;

    #[cfg(not(target_os = "macos"))]
    return true; // Other platforms handled by browser/webview
}
