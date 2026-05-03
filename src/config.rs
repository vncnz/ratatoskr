use std::{fs, ops::Range};
use serde::Deserialize;
use once_cell::sync::OnceCell;

use crate::utils::{DEFAULT_WHITE, hsv_to_rgb};

static CONFIG: OnceCell<Config> = OnceCell::new();

const DEFAULT_RAM_RANGE: [f64; 2] = [60.0, 90.0];
const DEFAULT_SWAP_RANGE: [f64; 2] = [60.0, 90.0];
const DEFAULT_DISK_RANGE: [f64; 2] = [60.0, 90.0];
const DEFAULT_TEMPERATURE_RANGE: [f64; 2] = [80.0, 99.0];
const DEFAULT_AVG_LOAD_RANGE: [f64; 2] = [0.0, 1.0];
const DEFAULT_BATTERY_RANGE: [f64; 2] = [20.0, 70.0];

#[derive(Debug, Clone)]
pub struct Threshold {
    pub range: Option<Range<f64>>,
    pub high_is_better: bool
}

#[derive(Debug, Clone)]
pub struct Config {
    pub threshold_ram: Threshold,
    pub threshold_swap: Threshold,
    pub threshold_disk: Threshold,
    pub threshold_temperature: Threshold,
    pub threshold_avg_load: Threshold,
    pub threshold_battery: Threshold
}

#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    threshold_ram: Option<serde_json::Value>,
    threshold_swap: Option<serde_json::Value>,
    threshold_disk: Option<serde_json::Value>,
    threshold_temperature: Option<serde_json::Value>,
    threshold_avg_load: Option<serde_json::Value>,
    threshold_battery: Option<serde_json::Value>
}

impl Threshold {
    /// Cases:
    /// - Some(Value::Array)    -> use value
    /// - Some(Value::Null)     -> use None (explicitly disabled by user)
    /// - None                  -> use default
    /// - Other                 -> log and use default
    fn from_json_with_default(
        value: Option<serde_json::Value>, 
        default_range: Option<[f64; 2]>, 
        high_is_better: bool
    ) -> Self {
        match value {
            Some(serde_json::Value::Array(arr)) if arr.len() == 2 => {
                let vals: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();
                if vals.len() == 2 {
                    Threshold { range: Some(vals[0]..vals[1]), high_is_better }
                } else {
                    Self::fallback_to_default(default_range, high_is_better, "formato numerico errato")
                }
            }
            Some(serde_json::Value::Null) => { Threshold { range: None, high_is_better } }
            None => { Self::fallback_to_default(default_range, high_is_better, "Missed parameter") }
            _ => { Self::fallback_to_default(default_range, high_is_better, "Invalid parameter") }
        }
    }

    fn fallback_to_default(default: Option<[f64; 2]>, high_is_better: bool, reason: &str) -> Self {
        if !reason.contains("assente") {
            eprintln!("Config Warning: {}, uso il default.", reason);
        }
        let range = default.map(|d| d[0]..d[1]);
        Threshold { range, high_is_better }
    }

    pub fn get_warn_level(&self, value: f64) -> f64 {
        if let Some(r) = &self.range {
            let warn_level = if value < r.start { 0.0 }
                            else if value < r.end { (value - r.start) / (r.end - r.start) }
                            else { 1.0 };
            if self.high_is_better { 1.0 - warn_level } else { warn_level }
        } else {
            0.0
        }
    }

    pub fn get_color(&self, value: f64) -> String {
        if let Some(r) = &self.range {
            let min = r.start;
            let max = r.end;
            let clamped = value.clamp(min, max);
            let mut ratio = if (max - min).abs() < f64::EPSILON {
                0.5
            } else {
                (clamped - min) / (max - min)
            };

            if self.high_is_better { ratio = 1.0 - ratio; }
            let sat;
            let hue;
            if DEFAULT_WHITE {
                sat = f64::max(1.0 - (ratio * ratio * ratio), 0.0);
                hue = 60.0 * ratio; // 60 -> 0
            } else {
                sat = 1.0;
                hue = 100.0 * ratio; // 100 -> 0
            }
            let (r, g, b) = hsv_to_rgb(hue, sat, 1.0);

            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            let (r,g,b) = (128,128,128);
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        }
    }
}

impl Config {
    pub fn init(path: &str) -> &'static Config {
        CONFIG.get_or_init(|| Config::load_from_file(path))
    }

    pub fn global() -> &'static Config {
        CONFIG.get().expect("Config not initialized")
    }

    pub fn load_from_file(path: &str) -> Self {
        let expanded_path = shellexpand::tilde(path);
        
        let raw: RawConfig = fs::read_to_string(expanded_path.as_ref())
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_else(|| {
                RawConfig::default()
            });

        Config {
            threshold_ram: Threshold::from_json_with_default(raw.threshold_ram, Some(DEFAULT_RAM_RANGE), false),
            threshold_swap: Threshold::from_json_with_default(raw.threshold_swap, Some(DEFAULT_SWAP_RANGE), false),
            threshold_disk: Threshold::from_json_with_default(raw.threshold_disk, Some(DEFAULT_DISK_RANGE), false),
            threshold_temperature: Threshold::from_json_with_default(raw.threshold_temperature, Some(DEFAULT_TEMPERATURE_RANGE), false),
            threshold_avg_load: Threshold::from_json_with_default(raw.threshold_avg_load, Some(DEFAULT_AVG_LOAD_RANGE), false),
            threshold_battery: Threshold::from_json_with_default(raw.threshold_battery, Some(DEFAULT_BATTERY_RANGE), true),
        }
    }
}