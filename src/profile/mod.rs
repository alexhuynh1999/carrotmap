pub mod config;
pub mod model;

use config::*;
pub use model::*;

use std::collections::HashMap;
use std::fmt;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::Input::XboxController::*;

#[derive(Debug)]
pub enum ProfileError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidKey(String, String), // (context, key_name)
    InvalidButton(String),      // (button_name)
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileError::Io(e) => write!(f, "IO error reading profile: {}", e),
            ProfileError::Json(e) => write!(f, "JSON syntax error in profile: {}", e),
            ProfileError::InvalidKey(ctx, key) => write!(f, "Invalid virtual key '{}' in {}", key, ctx),
            ProfileError::InvalidButton(btn) => write!(f, "Invalid gamepad button '{}'", btn),
        }
    }
}

impl std::error::Error for ProfileError {}

impl From<std::io::Error> for ProfileError {
    fn from(err: std::io::Error) -> Self {
        ProfileError::Io(err)
    }
}

impl From<serde_json::Error> for ProfileError {
    fn from(err: serde_json::Error) -> Self {
        ProfileError::Json(err)
    }
}

pub fn load_profile(path: &str) -> Result<Profile, ProfileError> {
    let file_content = std::fs::read_to_string(path)?;
    let config: ProfileConfig = serde_json::from_str(&file_content)?;

    let mut button_map = HashMap::new();
    for mapping in config.mappings {
        let mask = button_name_to_mask(&mapping.button)
            .ok_or_else(|| ProfileError::InvalidButton(mapping.button.clone()))?;
        
        let vk = key_name_to_vk(&mapping.key)
            .ok_or_else(|| ProfileError::InvalidKey(format!("button mapping '{}'", mapping.button), mapping.key.clone()))?;

        button_map.insert(mask, BoundButton {
            vk,
            repeat: mapping.repeat,
            repeat_interval_ms: mapping.repeat_interval_ms,
        });
    }

    let left_stick = if let Some(stick) = config.left_stick {
        let up = key_name_to_vk(&stick.up)
            .ok_or_else(|| ProfileError::InvalidKey("left_stick.up".into(), stick.up.clone()))?;
        let down = key_name_to_vk(&stick.down)
            .ok_or_else(|| ProfileError::InvalidKey("left_stick.down".into(), stick.down.clone()))?;
        let left = key_name_to_vk(&stick.left)
            .ok_or_else(|| ProfileError::InvalidKey("left_stick.left".into(), stick.left.clone()))?;
        let right = key_name_to_vk(&stick.right)
            .ok_or_else(|| ProfileError::InvalidKey("left_stick.right".into(), stick.right.clone()))?;

        Some(LeftStick {
            deadzone: stick.deadzone,
            threshold: stick.threshold,
            up,
            down,
            left,
            right,
        })
    } else {
        None
    };

    let triggers = if let Some(trig) = config.triggers {
        let left_vk = key_name_to_vk(&trig.left.key)
            .ok_or_else(|| ProfileError::InvalidKey("triggers.left.key".into(), trig.left.key.clone()))?;
        let right_vk = key_name_to_vk(&trig.right.key)
            .ok_or_else(|| ProfileError::InvalidKey("triggers.right.key".into(), trig.right.key.clone()))?;

        Some(Triggers {
            threshold: trig.threshold,
            left: ResolvedTriggerBinding {
                vk: left_vk,
                repeat: trig.left.repeat,
                repeat_interval_ms: trig.left.repeat_interval_ms,
            },
            right: ResolvedTriggerBinding {
                vk: right_vk,
                repeat: trig.right.repeat,
                repeat_interval_ms: trig.right.repeat_interval_ms,
            },
        })
    } else {
        None
    };

    Ok(Profile {
        name: config.name,
        tap_duration_ms: config.tap_duration_ms,
        triggers,
        left_stick,
        button_map,
    })
}

fn button_name_to_mask(name: &str) -> Option<u16> {
    match name.to_lowercase().as_str() {
        "dpad_up"    => Some(XINPUT_GAMEPAD_DPAD_UP.0),
        "dpad_down"  => Some(XINPUT_GAMEPAD_DPAD_DOWN.0),
        "dpad_left"  => Some(XINPUT_GAMEPAD_DPAD_LEFT.0),
        "dpad_right" => Some(XINPUT_GAMEPAD_DPAD_RIGHT.0),
        "start"      => Some(XINPUT_GAMEPAD_START.0),
        "back"       => Some(XINPUT_GAMEPAD_BACK.0),
        "l3" | "left_thumb" => Some(XINPUT_GAMEPAD_LEFT_THUMB.0),
        "r3" | "right_thumb" => Some(XINPUT_GAMEPAD_RIGHT_THUMB.0),
        "lb" | "left_shoulder" => Some(XINPUT_GAMEPAD_LEFT_SHOULDER.0),
        "rb" | "right_shoulder" => Some(XINPUT_GAMEPAD_RIGHT_SHOULDER.0),
        "a"          => Some(XINPUT_GAMEPAD_A.0),
        "b"          => Some(XINPUT_GAMEPAD_B.0),
        "x"          => Some(XINPUT_GAMEPAD_X.0),
        "y"          => Some(XINPUT_GAMEPAD_Y.0),
        _ => None,
    }
}

pub fn key_name_to_vk(name: &str) -> Option<VIRTUAL_KEY> {
    match name.to_lowercase().as_str() {
        "space"     => Some(VK_SPACE),
        "enter" | "return" => Some(VK_RETURN),
        "escape" | "esc" => Some(VK_ESCAPE),
        "tab"       => Some(VK_TAB),
        "shift"     => Some(VK_SHIFT),
        "ctrl" | "control" => Some(VK_CONTROL),
        "alt" | "menu" => Some(VK_MENU),
        "backspace" => Some(VK_BACK),
        "delete" | "del" => Some(VK_DELETE),
        "insert" | "ins" => Some(VK_INSERT),
        "up"        => Some(VK_UP),
        "down"      => Some(VK_DOWN),
        "left"      => Some(VK_LEFT),
        "right"     => Some(VK_RIGHT),
        "pageup" | "pgup" => Some(VK_PRIOR),
        "pagedown" | "pgdn" => Some(VK_NEXT),
        "home"      => Some(VK_HOME),
        "end"       => Some(VK_END),
        "f1"  => Some(VK_F1),  "f2"  => Some(VK_F2),
        "f3"  => Some(VK_F3),  "f4"  => Some(VK_F4),
        "f5"  => Some(VK_F5),  "f6"  => Some(VK_F6),
        "f7"  => Some(VK_F7),  "f8"  => Some(VK_F8),
        "f9"  => Some(VK_F9),  "f10" => Some(VK_F10),
        "f11" => Some(VK_F11), "f12" => Some(VK_F12),
        // OEM Symbols
        ";" | "semicolon" => Some(VK_OEM_1),
        "/" | "slash" => Some(VK_OEM_2),
        "`" | "grave" | "tilde" => Some(VK_OEM_3),
        "[" | "leftbracket" => Some(VK_OEM_4),
        "\\" | "backslash" => Some(VK_OEM_5),
        "]" | "rightbracket" => Some(VK_OEM_6),
        "'" | "quote" | "apostrophe" => Some(VK_OEM_7),
        "," | "comma" => Some(VK_OEM_COMMA),
        "." | "period" => Some(VK_OEM_PERIOD),
        "-" | "minus" => Some(VK_OEM_MINUS),
        "+" | "=" | "plus" | "equal" => Some(VK_OEM_PLUS),
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap().to_ascii_uppercase();
            if c.is_ascii_alphanumeric() {
                Some(VIRTUAL_KEY(c as u16))
            } else {
                None
            }
        }
        _ => None,
    }
}
