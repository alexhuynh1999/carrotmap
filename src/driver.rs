use rusty_xinput::XInputHandle;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::keyboard::VirtualKeyboard;
use crate::profile::Profile;

pub struct Driver {
    profile: Profile,
    keyboard: VirtualKeyboard,
    xinput: XInputHandle,
    active_controller: Option<u32>,
    last_buttons: u16,
    stick_up: bool,
    stick_down: bool,
    stick_left: bool,
    stick_right: bool,
    lt_active: bool,
    rt_active: bool,
    lt_last_fire: Option<Instant>,
    rt_last_fire: Option<Instant>,
    last_repeat: HashMap<u16, Instant>,
}

impl Driver {
    pub fn new(profile: Profile) -> Self {
        let tap_duration = profile.tap_duration_ms;
        let xinput = XInputHandle::load_default().expect("Failed to load XInput DLL");
        Self {
            profile,
            keyboard: VirtualKeyboard::new(tap_duration),
            xinput,
            active_controller: None,
            last_buttons: 0,
            stick_up: false,
            stick_down: false,
            stick_left: false,
            stick_right: false,
            lt_active: false,
            rt_active: false,
            lt_last_fire: None,
            rt_last_fire: None,
            last_repeat: HashMap::new(),
        }
    }

    pub fn run(mut self) {
        println!("Running driver polling loop — press Ctrl+C to quit.\n");
        loop {
            if let Some(id) = self.active_controller {
                if let Ok(state) = self.xinput.get_state(id) {
                    self.process_state(&state);
                } else {
                    println!("Controller on port {} disconnected.", id);
                    self.keyboard.clear_all();
                    self.active_controller = None;
                }
            } else {
                // Hot-plugging scan: look for first connected controller
                for i in 0..4 {
                    if self.xinput.get_state(i).is_ok() {
                        self.active_controller = Some(i);
                        println!("Controller connected on port {}", i);
                        break;
                    }
                }
                if self.active_controller.is_none() {
                    // Rest while scanning to save CPU
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
            
            // Loop resolution: sleep 500us
            // (Precision is maintained via TimerResolutionGuard)
            std::thread::sleep(Duration::from_micros(500));
        }
    }

    fn process_state(&mut self, state: &rusty_xinput::XInputState) {
        let buttons = state.raw.Gamepad.wButtons;
        let just_pressed = buttons & !self.last_buttons;
        let just_released = !buttons & self.last_buttons;

        // --- Button mappings ---
        for (&mask, bound) in &self.profile.button_map {
            let pressed = just_pressed & mask != 0;
            let released = just_released & mask != 0;
            let held = buttons & mask != 0;

            if pressed {
                self.keyboard.key_down(bound.vk);
                if bound.repeat {
                    self.last_repeat.insert(mask, Instant::now());
                }
            }

            if released {
                self.keyboard.key_up(bound.vk);
                self.last_repeat.remove(&mask);
            }

            if held && bound.repeat && !pressed {
                let interval = Duration::from_nanos(
                    (bound.repeat_interval_ms * 1_000_000.0) as u64
                );
                let due = self.last_repeat
                    .get(&mask)
                    .map(|t| t.elapsed() >= interval)
                    .unwrap_or(false);

                if due {
                    self.keyboard.tap_key_for(bound.vk, bound.repeat_interval_ms / 2.0);
                    self.last_repeat.insert(mask, Instant::now());
                }
            }
        }

        self.last_buttons = buttons;

        // --- Left stick ---
        if let Some(stick) = self.profile.left_stick {
            let x = state.raw.Gamepad.sThumbLX as i32;
            let y = state.raw.Gamepad.sThumbLY as i32;
            let deadzone = stick.deadzone;
            let threshold = stick.threshold;
            
            let x = if x.abs() < deadzone { 0 } else { x };
            let y = if y.abs() < deadzone { 0 } else { y };

            self.update_stick_key(y > threshold, stick.up, StickDir::Up);
            self.update_stick_key(y < -threshold, stick.down, StickDir::Down);
            self.update_stick_key(x < -threshold, stick.left, StickDir::Left);
            self.update_stick_key(x > threshold, stick.right, StickDir::Right);
        }

        // --- Triggers ---
        if let Some(t) = self.profile.triggers {
            let lt = state.raw.Gamepad.bLeftTrigger;
            let rt = state.raw.Gamepad.bRightTrigger;

            let threshold = t.threshold;
            let left_vk = t.left.vk;
            let left_repeat = t.left.repeat;
            let left_interval = t.left.repeat_interval_ms;
            
            let right_vk = t.right.vk;
            let right_repeat = t.right.repeat;
            let right_interval = t.right.repeat_interval_ms;

            self.update_trigger(
                lt >= threshold,
                left_vk, left_repeat, left_interval,
                TriggerSide::Left,
            );
            self.update_trigger(
                rt >= threshold,
                right_vk, right_repeat, right_interval,
                TriggerSide::Right,
            );
        }
    }

    fn update_stick_key(&mut self, now_active: bool, vk: VIRTUAL_KEY, direction: StickDir) {
        let was_active = match direction {
            StickDir::Up => &mut self.stick_up,
            StickDir::Down => &mut self.stick_down,
            StickDir::Left => &mut self.stick_left,
            StickDir::Right => &mut self.stick_right,
        };

        if now_active && !*was_active {
            self.keyboard.key_down(vk);
        } else if !now_active && *was_active {
            self.keyboard.key_up(vk);
        }
        *was_active = now_active;
    }

    fn update_trigger(
        &mut self,
        now_active: bool,
        vk: VIRTUAL_KEY,
        repeat: bool,
        interval_ms: f64,
        side: TriggerSide,
    ) {
        let (was_active, last_fire) = match side {
            TriggerSide::Left => (&mut self.lt_active, &mut self.lt_last_fire),
            TriggerSide::Right => (&mut self.rt_active, &mut self.rt_last_fire),
        };

        if now_active && !*was_active {
            self.keyboard.key_down(vk);
            if repeat {
                *last_fire = Some(Instant::now());
            }
        } else if !now_active && *was_active {
            self.keyboard.key_up(vk);
            *last_fire = None;
        } else if now_active && repeat {
            let interval = Duration::from_nanos((interval_ms * 1_000_000.0) as u64);
            let due = last_fire
                .map(|t| t.elapsed() >= interval)
                .unwrap_or(false);

            if due {
                self.keyboard.tap_key_for(vk, interval_ms / 2.0);
                *last_fire = Some(Instant::now());
            }
        }

        *was_active = now_active;
    }
}

enum StickDir {
    Up,
    Down,
    Left,
    Right,
}

enum TriggerSide {
    Left,
    Right,
}
