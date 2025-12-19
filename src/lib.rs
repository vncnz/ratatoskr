use serde::{Deserialize, Serialize};

pub mod utils;
pub mod sysutils;

#[derive(Default, Serialize)]
pub struct SystemStats {
    pub ram: Option<RamStats>,
    pub disk: Option<DiskStats>,
    pub temperature: Option<TempStats>,
    pub weather: Option<WeatherStats>,
    pub loadavg: Option<AvgLoadStats>,
    pub volume: Option<VolumeStats>,
    pub battery: Option<BatteryStats>,
    pub network: Option<NetworkStats>,
    pub display: Option<EmbeddedDisplayStats>,
    pub written_at: u64,
    pub metronome: bool
}

#[derive(Default, Serialize)]
pub struct RamStats {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub mem_percent: u64,
    pub swap_percent: u64,
    pub mem_color: String,
    pub swap_color: String,
    pub mem_warn: f64,
    pub swap_warn: f64,
    pub warn: f64
}

#[derive(Default, Serialize)]
pub struct DiskStats {
    pub total_size: u64,
    pub used_size: u64,
    pub used_percent: u64,
    pub color: String,
    pub warn: f64
}

#[derive(Default, Serialize)]
pub struct TempStats {
    pub sensor: String,
    pub value: f32,
    pub color: Option<String>,
    pub icon: String,
    pub warn: f64
}

#[derive(Default, Serialize, Deserialize)]
pub struct WeatherStats {
    pub icon: String,
    pub icon_name: String,
    pub temp: i8,
    pub temp_real: i8,
    pub temp_unit: String,
    pub text: String,
    pub day: String,
    pub sunrise: String,
    pub sunset: String,
    pub sunrise_mins: u64,
    pub sunset_mins: u64,
    pub daylight: f64,
    pub locality: String,
    pub humidity: u8,
    pub updated: Option<String>,
    pub warn: Option<f64>
}

#[derive(Default, Serialize)]
pub struct AvgLoadStats {
    pub m1: f64,
    pub m5: f64,
    pub m15: f64,
    pub ncpu: usize,
    pub warn: f64,
    pub color: String
}

#[derive(Default, Serialize)]
pub struct VolumeStats {
    pub value: i64,
    pub icon: String,
    pub color: String,
    pub clazz: String,
    pub warn: f64
}
#[derive(Deserialize)]
pub struct VolumeObj {
    pub value: i64,
    pub icon: String,
    pub clazz: String,
    pub headphones: i8
}

#[derive(Default, Serialize)]
pub struct BatteryStats {
    pub percentage: i32,
    pub capacity: f32,
    pub eta: Option<f32>,
    pub state: String,
    pub icon: String,
    pub color: Option<String>,
    pub watt: f32,
    pub warn: f64
}

#[derive(Default,Serialize,Debug)]
pub struct NetworkStats {
    pub iface: String,
    pub conn_type: String,
    pub ssid: Option<String>,
    pub signal: Option<u8>,
    pub ip: Option<String>,
    pub icon: String,
    pub color: Option<String>,
    pub warn: f64
}

#[derive(Default, Serialize)]
pub struct EmbeddedDisplayStats {
    pub brightness_current: u32,
    pub brightness_max: u32,
    pub perc: u8,
    pub icon: String,
    pub warn: f64
}