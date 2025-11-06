use crate::types::ActiveApplication;
use anyhow::{Context, Result};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "windows")]
use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};
#[cfg(target_os = "windows")]
use winapi::um::processthreadsapi::OpenProcess;
#[cfg(target_os = "windows")]
use winapi::um::psapi::{GetModuleFileNameExW, K32GetModuleFileNameExW};
#[cfg(target_os = "windows")]
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

/// Get active application on Windows
pub fn get_active_application() -> Result<ActiveApplication> {
    #[cfg(target_os = "windows")]
    {
        use std::ptr::null_mut;

        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                anyhow::bail!("No foreground window");
            }

            // Get window title
            let mut title_buf: [u16; 512] = [0; 512];
            let title_len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
            let title = if title_len > 0 {
                String::from_utf16_lossy(&title_buf[..title_len as usize])
            } else {
                String::new()
            };

            // Get process ID
            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut process_id);

            // Open process
            let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, process_id);
            if process_handle.is_null() {
                anyhow::bail!("Failed to open process");
            }

            // Get executable path
            let mut exe_buf: [u16; 512] = [0; 512];
            let exe_len = K32GetModuleFileNameExW(
                process_handle,
                null_mut(),
                exe_buf.as_mut_ptr(),
                512,
            );

            let exe_path = if exe_len > 0 {
                String::from_utf16_lossy(&exe_buf[..exe_len as usize])
            } else {
                String::from("unknown.exe")
            };

            let executable = exe_path
                .split('\\')
                .last()
                .unwrap_or("unknown.exe")
                .to_string();

            let app_name = executable
                .strip_suffix(".exe")
                .unwrap_or(&executable)
                .to_string();

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
    }

    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!("Windows-specific function called on non-Windows platform");
    }
}
