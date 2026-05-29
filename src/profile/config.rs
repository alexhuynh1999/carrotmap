use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProfileConfig {
    pub name: String,
    pub tap_duration_ms: u64,
    pub triggers: Option<TriggersConfig>,
    pub left_stick: Option<LeftStickConfig>,
    pub mappings: Vec<MappingConfig>,
}

#[derive(Deserialize, Debug)]
pub struct LeftStickConfig {
    pub deadzone: i32,
    pub threshold: i32,
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
}

#[derive(Deserialize, Debug)]
pub struct MappingConfig {
    pub button: String,
    pub key: String,
    #[serde(default)]
    pub repeat: bool,
    #[serde(default = "default_repeat_interval")]
    pub repeat_interval_ms: f64,
}

fn default_repeat_interval() -> f64 {
    200.0
}

#[derive(Deserialize, Debug)]
pub struct TriggerBindingConfig {
    pub key: String,
    #[serde(default)]
    pub repeat: bool,
    #[serde(default = "default_repeat_interval")]
    pub repeat_interval_ms: f64,
}

#[derive(Deserialize, Debug)]
pub struct TriggersConfig {
    pub threshold: u8,
    pub left: TriggerBindingConfig,
    pub right: TriggerBindingConfig,
}
