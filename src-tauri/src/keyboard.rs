use rdev::{listen, Event, EventType};
use serde::Serialize;
use anyhow::Result;

#[derive(Serialize, Clone)]
pub struct KeyboardEvent {
    pub event_type: String,
    pub name: Option<String>,
    pub time: std::time::SystemTime,
    pub data: String,
}

pub fn start_keyboard_listener() -> Result<()> {
    listen(move |event| {
        match event.event_type {
            EventType::KeyPress(key) => {
                let json_event = KeyboardEvent {
                    event_type: "KeyPress".to_string(),
                    name: event.name.clone(),
                    time: event.time,
                    data: serde_json::json!({
                        "key": format!("{:?}", key)
                    }).to_string(),
                };

                // In Electron version, this would emit to the main process
                // In Tauri, we could emit events to frontend windows
                println!("{}", serde_json::to_string(&json_event).unwrap_or_default());
            }
            EventType::KeyRelease(key) => {
                let json_event = KeyboardEvent {
                    event_type: "KeyRelease".to_string(),
                    name: event.name.clone(),
                    time: event.time,
                    data: serde_json::json!({
                        "key": format!("{:?}", key)
                    }).to_string(),
                };

                println!("{}", serde_json::to_string(&json_event).unwrap_or_default());
            }
            _ => {}
        }
    })?;

    Ok(())
}

pub fn write_text(text: &str) -> Result<()> {
    use enigo::{Enigo, Keyboard, Settings};

    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.text(text)?;

    Ok(())
}
