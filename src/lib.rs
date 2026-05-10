use serde::{Deserialize, Serialize};

pub mod utils;
pub mod sysutils;
pub mod config;

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
    pub bluetooth_batteries: Option<BluetoothStats>,
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
    pub icon: String, // legacy
    pub color: String, // legacy
    pub clazz: String, // legacy
    pub headphones: i8,
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
    pub capacity_design: f32,
    pub cycles: Option<u32>,
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

#[derive(Debug, Clone, Serialize)]
pub struct BatteryDevice {
    pub name: String,
    pub kind: UPowerDeviceKind,
    pub percentage: f64,
    pub warn: f64
}

#[derive(Debug, Clone, Serialize)]
pub enum UPowerDeviceKind {
    Unknown,
    LinePower,
    Battery,
    Ups,
    Monitor,
    Mouse,
    Keyboard,
    Pda,
    Phone,
    MediaPlayer,
    Tablet,
    Computer,
    GamingInput,
    Pen,
    Touchpad,
    Modem,
    Network,
    Headset,
    Speakers,
    Headphones,
    Video,
    OtherAudio,
    RemoteControl,
    Printer,
    Scanner,
    Camera,
    Wearable,
    Toy,
    BluetoothGeneric,
}

impl From<u32> for UPowerDeviceKind {
    fn from(value: u32) -> Self {
        match value {
            0 => UPowerDeviceKind::Unknown,
            1 => UPowerDeviceKind::LinePower,
            2 => UPowerDeviceKind::Battery,
            3 => UPowerDeviceKind::Ups,
            4 => UPowerDeviceKind::Monitor,
            5 => UPowerDeviceKind::Mouse,
            6 => UPowerDeviceKind::Keyboard,
            7 => UPowerDeviceKind::Pda,
            8 => UPowerDeviceKind::Phone,
            9 => UPowerDeviceKind::MediaPlayer,
            10 => UPowerDeviceKind::Tablet,
            11 => UPowerDeviceKind::Computer,
            12 => UPowerDeviceKind::GamingInput,
            13 => UPowerDeviceKind::Pen,
            14 => UPowerDeviceKind::Touchpad,
            15 => UPowerDeviceKind::Modem,
            16 => UPowerDeviceKind::Network,
            17 => UPowerDeviceKind::Headset,
            18 => UPowerDeviceKind::Speakers,
            19 => UPowerDeviceKind::Headphones,
            20 => UPowerDeviceKind::Video,
            21 => UPowerDeviceKind::OtherAudio,
            22 => UPowerDeviceKind::RemoteControl,
            23 => UPowerDeviceKind::Printer,
            24 => UPowerDeviceKind::Scanner,
            25 => UPowerDeviceKind::Camera,
            26 => UPowerDeviceKind::Wearable,
            27 => UPowerDeviceKind::Toy,
            28 => UPowerDeviceKind::BluetoothGeneric,
            _ => UPowerDeviceKind::Unknown,
        }
    }
}

#[derive(Default, Serialize)]
pub struct BluetoothStats {
    pub devices: Vec<BatteryDevice>,
    pub icon: String,
    pub warn: f64
}