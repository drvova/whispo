use rdev::{listen, Event, EventType, Key};
use serde::Serialize;
use anyhow::Result;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardEvent {
    pub event_type: String,
    pub key: String,
    pub name: Option<String>,
    pub timestamp: u128,
}

static EVENT_SENDER: OnceLock<Arc<Mutex<Option<Sender<KeyboardEvent>>>>> = OnceLock::new();

pub fn init_keyboard_system() {
    EVENT_SENDER.get_or_init(|| Arc::new(Mutex::new(None)));
}

pub fn set_event_sender(sender: Sender<KeyboardEvent>) {
    if let Some(event_sender) = EVENT_SENDER.get() {
        *event_sender.lock().unwrap() = Some(sender);
    }
}

fn send_keyboard_event(event: KeyboardEvent) {
    if let Some(event_sender) = EVENT_SENDER.get() {
        if let Some(sender) = event_sender.lock().unwrap().as_ref() {
            let _ = sender.send(event);
        }
    }
}

pub fn start_keyboard_listener() -> Result<()> {
    listen(move |event| {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        match event.event_type {
            EventType::KeyPress(key) => {
                let keyboard_event = KeyboardEvent {
                    event_type: "KeyPress".to_string(),
                    key: format!("{:?}", key),
                    name: event.name.clone(),
                    timestamp,
                };

                send_keyboard_event(keyboard_event);
            }
            EventType::KeyRelease(key) => {
                let keyboard_event = KeyboardEvent {
                    event_type: "KeyRelease".to_string(),
                    key: format!("{:?}", key),
                    name: event.name.clone(),
                    timestamp,
                };

                send_keyboard_event(keyboard_event);
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

pub fn simulate_key_press(key_name: &str) -> Result<()> {
    use enigo::{Enigo, Key as EnigoKey, Keyboard, Settings};

    let mut enigo = Enigo::new(&Settings::default())?;

    let key = match key_name.to_lowercase().as_str() {
        "return" | "enter" => EnigoKey::Return,
        "escape" | "esc" => EnigoKey::Escape,
        "backspace" => EnigoKey::Backspace,
        "tab" => EnigoKey::Tab,
        "space" => EnigoKey::Space,
        "delete" => EnigoKey::Delete,
        "home" => EnigoKey::Home,
        "end" => EnigoKey::End,
        "pageup" => EnigoKey::PageUp,
        "pagedown" => EnigoKey::PageDown,
        "leftarrow" => EnigoKey::LeftArrow,
        "rightarrow" => EnigoKey::RightArrow,
        "uparrow" => EnigoKey::UpArrow,
        "downarrow" => EnigoKey::DownArrow,
        _ => return Err(anyhow::anyhow!("Unsupported key: {}", key_name)),
    };

    enigo.key(key, enigo::Direction::Click)?;

    Ok(())
}

pub fn is_modifier_key(key: &Key) -> bool {
    matches!(
        key,
        Key::ShiftLeft
            | Key::ShiftRight
            | Key::ControlLeft
            | Key::ControlRight
            | Key::Alt
            | Key::AltGr
            | Key::MetaLeft
            | Key::MetaRight
    )
}
