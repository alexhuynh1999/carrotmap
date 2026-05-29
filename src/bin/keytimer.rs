use std::collections::HashMap;
use std::time::Instant;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

fn main() {
    println!("Press keys to measure hold duration. Press Escape to quit.\n");

    let mut down_times: HashMap<u32, Instant> = HashMap::new();

    loop {
        for vk in 0x08u32..=0xFEu32 {
            let state = unsafe { GetAsyncKeyState(vk as i32) };
            let is_down = (state as u16) & 0x8000 != 0;
            let was_down = down_times.contains_key(&vk);

            if is_down && !was_down {
                down_times.insert(vk, Instant::now());
            } else if !is_down && was_down {
                if let Some(start) = down_times.remove(&vk) {
                    let duration_ms = start.elapsed().as_millis();
                    let name = vk_name(vk);
                    println!("{:<12} held for {:>5} ms", name, duration_ms);
                }
            }
        }

        // Escape to quit
        if unsafe { GetAsyncKeyState(VK_ESCAPE.0 as i32) } as u16 & 0x8000 != 0 {
            println!("\nDone.");
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn vk_name(vk: u32) -> String {
    match vk {
        0x08 => "Backspace".into(),
        0x09 => "Tab".into(),
        0x0D => "Enter".into(),
        0x10 => "Shift".into(),
        0x11 => "Ctrl".into(),
        0x12 => "Alt".into(),
        0x13 => "Pause".into(),
        0x14 => "CapsLock".into(),
        0x20 => "Space".into(),
        0x21 => "PageUp".into(),
        0x22 => "PageDown".into(),
        0x23 => "End".into(),
        0x24 => "Home".into(),
        0x25 => "Left".into(),
        0x26 => "Up".into(),
        0x27 => "Right".into(),
        0x28 => "Down".into(),
        0x2E => "Delete".into(),
        0x30..=0x39 => format!("{}", (vk as u8 - 0x30 + b'0') as char),
        0x41..=0x5A => format!("{}", (vk as u8 - 0x41 + b'A') as char),
        0x70..=0x7B => format!("F{}", vk - 0x6F),
        0xA0 => "LShift".into(),
        0xA1 => "RShift".into(),
        0xA2 => "LCtrl".into(),
        0xA3 => "RCtrl".into(),
        _ => format!("VK_0x{:02X}", vk),
    }
}