use std::collections::HashMap;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub struct VirtualKeyboard {
    tap_duration_ms: u64,
    pressed_counts: HashMap<u16, usize>,
}

impl VirtualKeyboard {
    pub fn new(tap_duration_ms: u64) -> Self {
        Self {
            tap_duration_ms,
            pressed_counts: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn tap_key(&self, vk: VIRTUAL_KEY) {
        self.tap_key_for(vk, self.tap_duration_ms as f64);
    }

    /// Tap with an explicit duration in milliseconds (supports sub-ms via spin-wait)
    pub fn tap_key_for(&self, vk: VIRTUAL_KEY, duration_ms: f64) {
        let scan = vk_to_scan(vk);
        send_inputs(&[make_key_input(vk, scan, false)]);
        precise_sleep(duration_ms);
        send_inputs(&[make_key_input(vk, scan, true)]);
    }

    pub fn key_down(&mut self, vk: VIRTUAL_KEY) {
        let count = self.pressed_counts.entry(vk.0).or_insert(0);
        if *count == 0 {
            let scan = vk_to_scan(vk);
            send_inputs(&[make_key_input(vk, scan, false)]);
        }
        *count += 1;
    }

    pub fn key_up(&mut self, vk: VIRTUAL_KEY) {
        if let Some(count) = self.pressed_counts.get_mut(&vk.0) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    let scan = vk_to_scan(vk);
                    send_inputs(&[make_key_input(vk, scan, true)]);
                }
            }
        }
    }

    /// Release all currently held keys (useful on disconnection/quit)
    pub fn clear_all(&mut self) {
        for (&vk_val, &count) in &self.pressed_counts {
            if count > 0 {
                let vk = VIRTUAL_KEY(vk_val);
                let scan = vk_to_scan(vk);
                send_inputs(&[make_key_input(vk, scan, true)]);
            }
        }
        self.pressed_counts.clear();
    }
}

/// Sleep precisely for `ms` milliseconds.
/// Uses thread::sleep for anything above 2ms (efficient),
/// and spin-waits for anything at or below 2ms (precise).
pub fn precise_sleep(ms: f64) {
    let duration = Duration::from_nanos((ms * 1_000_000.0) as u64);
    let start = Instant::now();

    if ms > 2.0 {
        // Sleep for most of the duration, leaving 2ms to spin
        let sleep_for = duration.saturating_sub(Duration::from_millis(2));
        if !sleep_for.is_zero() {
            std::thread::sleep(sleep_for);
        }
    }

    // Spin for the remaining time
    while start.elapsed() < duration {
        std::hint::spin_loop();
    }
}

fn make_key_input(vk: VIRTUAL_KEY, scan: u16, is_up: bool) -> INPUT {
    let mut flags = KEYEVENTF_SCANCODE;
    let extended = matches!(vk,
        VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT |
        VK_INSERT | VK_DELETE | VK_HOME | VK_END |
        VK_PRIOR | VK_NEXT
    );
    if extended { flags |= KEYEVENTF_EXTENDEDKEY; }
    if is_up    { flags |= KEYEVENTF_KEYUP; }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_inputs(inputs: &[INPUT]) {
    unsafe {
        SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

fn vk_to_scan(vk: VIRTUAL_KEY) -> u16 {
    unsafe { MapVirtualKeyW(vk.0 as u32, MAPVK_VK_TO_VSC) as u16 }
}