use std::{fs, ops::Range};
use serde::Deserialize;
use once_cell::sync::OnceCell;

static CONFIG: OnceCell<Config> = OnceCell::new();


#[derive(Debug, Clone)]
pub struct Threshold {
    pub range: Option<Range<f64>>,
    high_is_better: bool
}

#[derive(Debug, Clone)]
pub struct Config {
    pub threshold_ram: Threshold
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    threshold_ram: Option<serde_json::Value>
}

impl Threshold {
    fn from_json(value: Option<serde_json::Value>, high_is_better: bool) -> Self {
        match value {
            Some(serde_json::Value::Null) => Threshold { range: None, high_is_better },
            None => Threshold { range: None, high_is_better },


            Some(serde_json::Value::Array(arr)) if arr.len() == 2 => {
                let vals: Vec<f64> = arr
                    .iter()
                    .filter_map(|v| v.as_f64().map(|x| x as f64))
                    .collect();
                Threshold { range: Some(vals[0]..vals[1]), high_is_better }
            },

            _ => {
                eprintln!("Invalid value in JSON configuration {:?}", value);
                Threshold { range: None, high_is_better }
            }
        }
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
}

impl Config {
    pub fn init(path: &str) -> &'static Config {
        CONFIG.get_or_init(|| Config::load_from_file(path))
    }

    pub fn global() -> &'static Config {
        CONFIG.get().expect("Config non inizializzata")
    }

    pub fn load_from_file(path: &str) -> Self {
        let expanded_path = shellexpand::tilde(path);
        let data = fs::read_to_string(expanded_path.as_ref())
            .unwrap_or_else(|_| {
                eprintln!("Cannot read configuration file: {}", path);
                "{}".to_string()
            });

        let raw: RawConfig = serde_json::from_str(&data).unwrap_or_else(|_| {
            eprintln!("Config JSON non valido, uso valori di default");
            RawConfig {
                threshold_ram: Some(serde_json::json!([60.0, 90.0]))
            }
        });

        Config { // TODO: if I explicitly write nulls in config I don't want defaults, If I don't write a specific parameter I want its default!
            threshold_ram: Threshold::from_json(raw.threshold_ram, false)
        }
    }
}
