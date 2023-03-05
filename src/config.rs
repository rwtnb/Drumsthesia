use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::output_manager::OutputDescriptor;

#[derive(Serialize, Deserialize, Default)]
pub struct ColorSchema {
    pub cyan: (u8, u8, u8),
    pub dark: (u8, u8, u8),
    pub gray: (u8, u8, u8),
    pub red: (u8, u8, u8),
    pub green: (u8, u8, u8),
    pub yellow: (u8, u8, u8),
    pub blue: (u8, u8, u8),
    pub purple1: (u8, u8, u8),
    pub purple2: (u8, u8, u8),
    pub purple3: (u8, u8, u8),
    pub aqua: (u8, u8, u8),
    pub orange1: (u8, u8, u8),
    pub orange2: (u8, u8, u8),
    pub orange3: (u8, u8, u8),
    pub beige: (u8, u8, u8),
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_speed_multiplier")]
    pub speed_multiplier: f32,

    #[serde(default = "default_playback_offset")]
    pub playback_offset: f32,

    #[serde(default = "default_play_along")]
    #[serde(skip_serializing)]
    pub play_along: bool,

    #[serde(default = "default_mute_drums")]
    #[serde(skip_serializing)]
    pub mute_drums: bool,

    #[serde(default = "default_color_schema")]
    pub color_schema: ColorSchema,

    #[serde(default)]
    pub background_color: (u8, u8, u8),

    #[serde(default = "default_output")]
    pub output: Option<String>,
    pub input: Option<String>,

    pub soundfont_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let config: Option<Config> = if let Some(path) = crate::utils::resources::settings_ron() {
            if let Ok(file) = std::fs::read_to_string(path) {
                match ron::from_str(&file) {
                    Ok(config) => Some(config),
                    Err(err) => {
                        log::error!("{:#?}", err);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        config.unwrap_or_else(|| Self {
            speed_multiplier: default_speed_multiplier(),
            playback_offset: default_playback_offset(),
            play_along: default_play_along(),
            mute_drums: default_mute_drums(),
            color_schema: default_color_schema(),
            background_color: Default::default(),
            output: default_output(),
            input: None,
            soundfont_path: None,
        })
    }

    pub fn set_output(&mut self, v: &OutputDescriptor) {
        if let OutputDescriptor::DummyOutput = v {
            self.output = None;
        } else {
            self.output = Some(v.to_string());
        }
    }

    pub fn set_input<D: std::fmt::Display>(&mut self, v: Option<D>) {
        self.input = v.map(|v| v.to_string());
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        if let Ok(s) = ron::ser::to_string_pretty(self, Default::default()) {
            if let Some(path) = crate::utils::resources::settings_ron() {
                std::fs::create_dir_all(path.parent().unwrap()).ok();
                std::fs::write(path, &s).ok();
            }
        }
    }
}

fn default_speed_multiplier() -> f32 {
    1.0
}

fn default_playback_offset() -> f32 {
    0.0
}

fn default_play_along() -> bool {
    false
}

fn default_mute_drums() -> bool {
    true
}

pub const fn default_color_schema() -> ColorSchema {
    ColorSchema {
        cyan: (93, 188, 255),
        dark: (48, 124, 255),
        gray: (146, 131, 116),
        red: (204, 36, 29),
        green: (152, 151, 26),
        yellow: (215, 153, 33),
        blue: (69, 133, 136),
        purple1: (211, 134, 155),
        purple2: (177, 98, 134),
        purple3: (143, 63, 113),
        aqua: (104, 157, 106),
        orange1: (254, 128, 25),
        orange2: (214, 93, 14),
        orange3: (175, 58, 3),
        beige: (235, 219, 178),
    }
}

fn default_output() -> Option<String> {
    Some("Buildin Synth".into())
}
