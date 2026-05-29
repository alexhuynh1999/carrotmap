mod keyboard;
mod profile;
mod timer;
mod driver;

use timer::TimerResolutionGuard;
use driver::Driver;

fn main() {
    // Enable Windows high-resolution scheduler resolution (1ms) for this process
    let _timer_guard = TimerResolutionGuard::new();

    // Load the profile. Propagate loading errors gracefully.
    let profile_path = "profile.json";
    let profile = match profile::load_profile(profile_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error loading profile '{}': {}", profile_path, e);
            std::process::exit(1);
        }
    };

    println!("Loaded profile: {}", profile.name);
    println!("Tap duration: {}ms", profile.tap_duration_ms);
    println!("Mappings: {} buttons bound", profile.button_map.len());
    if profile.left_stick.is_some() { println!("Left stick: enabled"); }
    if profile.triggers.is_some()   { println!("Triggers: enabled"); }

    let driver = Driver::new(profile);
    driver.run();
}