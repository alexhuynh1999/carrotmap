use std::collections::HashMap;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

pub struct BoundButton {
    pub vk: VIRTUAL_KEY,
    pub repeat: bool,
    pub repeat_interval_ms: f64,
}

#[derive(Copy, Clone)]
pub struct LeftStick {
    pub deadzone: i32,
    pub threshold: i32,
    pub up: VIRTUAL_KEY,
    pub down: VIRTUAL_KEY,
    pub left: VIRTUAL_KEY,
    pub right: VIRTUAL_KEY,
}

#[derive(Copy, Clone)]
pub struct ResolvedTriggerBinding {
    pub vk: VIRTUAL_KEY,
    pub repeat: bool,
    pub repeat_interval_ms: f64,
}

#[derive(Copy, Clone)]
pub struct Triggers {
    pub threshold: u8,
    pub left: ResolvedTriggerBinding,
    pub right: ResolvedTriggerBinding,
}

pub struct Profile {
    pub name: String,
    pub tap_duration_ms: u64,
    pub triggers: Option<Triggers>,
    pub left_stick: Option<LeftStick>,
    pub button_map: HashMap<u16, BoundButton>,
}
