use serde::{Serialize,Deserialize};
use std::process::{Command};

use sysinfo::{Disks, System};

use crate::utils;

#[derive(Default, Serialize)]
pub struct RamStats {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub mem_percent: u64,
    pub swap_percent: u64,
    pub mem_color: String,
    pub swap_color: String
}

#[derive(Default, Serialize)]
pub struct DiskStats {
    pub total_size: u64,
    pub used_size: u64,
    pub used_percent: u64,
    pub color: String
}

#[derive(Default, Serialize)]
pub struct TempStats {
    pub sensor: String,
    pub temperature: f32
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
    pub humidity: u8
}

#[derive(Default, Serialize)]
pub struct AvgLoadStats {
    pub m1: f64,
    pub m5: f64,
    pub m15: f64,
    pub ncpu: usize,
    pub critical_factor: f64,
    pub color: String
}

/* fn get_load_avg() -> SysUpdate {
    if let Ok(output) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = output.split_whitespace().collect();
        SysUpdate::LoadAvg(parts[0].parse().expect("Error 1m"), parts[1].parse().expect("Error 5m"), parts[2].parse().expect("Error 15m"))
    } else {
        SysUpdate::Error("Errore".into())
    }
} */

pub fn get_ram_info () -> RamStats {
    let mut sys = System::new();
    sys.refresh_memory();
    let tm = sys.total_memory();
    let um = sys.used_memory();
    let ts = sys.total_swap();
    let us = sys.used_swap();
    let mp = 100 * um / tm;
    let sp = 100 * us / ts;

    RamStats {
        total_memory: tm,
        used_memory: um,
        total_swap: ts,
        used_swap: us,
        mem_percent: mp,
        swap_percent: sp,
        mem_color: utils::get_color_gradient(60.0, 90.0, mp as f64, false),
        swap_color: utils::get_color_gradient(60.0, 90.0, sp as f64, false)
    }
}


pub fn get_disk_info () -> DiskStats {
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        if (disk as &sysinfo::Disk).mount_point() == std::path::Path::new("/") {
            if let Some(_name_str) = (disk as &sysinfo::Disk).name().to_str() {
                if let Some(_mount_str) = (disk as &sysinfo::Disk).mount_point().to_str() {
                    let tos = (disk as &sysinfo::Disk).total_space();
                    let avs = (disk as &sysinfo::Disk).available_space();
                    let up = 100 - (avs * 100 / tos);
                    return DiskStats {
                        // name_str.to_string(),
                        // mount_str.to_string(),
                        total_size: tos,
                        used_size: tos - avs,
                        used_percent: up,
                        color: utils::get_color_gradient(60.0, 90.0, up as f64, false)
                    }
                }
            }
        }
    }
    // SysUpdate::Error("Disk not found".to_string())
    DiskStats { total_size: 0, used_size: 0, used_percent: 100, color: "#FF0000".to_string() }
}

pub fn get_sys_temperatures () -> TempStats {
    let components = sysinfo::Components::new_with_refreshed_list();
    for component in &components {
        if component.label() == "Tctl" {
            if let Some(temp) = component.temperature() {
                return TempStats {
                    sensor: component.label().into(),
                    temperature: temp
                };
            } else {
                return TempStats {
                    sensor: component.label().into(),
                    temperature: 0.0
                };
            }
        }
    }
    TempStats {
        sensor: "".into(),
        temperature: 0.0
    }
}

pub fn get_weather () -> WeatherStats {
    let output = Command::new("/home/vncnz/.config/eww/scripts/meteo.sh").arg("'Desenzano Del Garda'").arg("45.457692").arg("10.570684").output();
    let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
    // println!("\n{:?}", stdout);
    // let weather: WeatherObj;
    if let Ok(weather) = serde_json::from_str(&stdout) {
        return weather
    }
    WeatherStats::default()
}

static mut N_CPU: usize = 0;

use once_cell::sync::Lazy;
/* static CORE_COUNT: Lazy<usize> = Lazy::new(|| {
    std::fs::read_to_string("/proc/cpuinfo")
        .map(|contents| {
            contents
                .lines()
                .filter(|line| line.starts_with("processor"))
                .count()
        })
        .unwrap_or(1)
}); */

pub fn get_load_avg() -> AvgLoadStats {
    if unsafe { N_CPU } == 0 {
        unsafe { N_CPU = std::fs::read_to_string("/proc/cpuinfo")
            .map(|contents| {
                contents
                    .lines()
                    .filter(|line| line.starts_with("processor"))
                    .count()
            })
            .unwrap_or(1) } // fallback: almeno 1 core
    }
    if let Ok(output) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = output.split_whitespace().collect();

        // let T = clamp((load1 / load5 - 1.0) / 1.0, 0.0, 1.0);
        // let I = clamp(load1 / custom_max_load, 0.0, 1.0);
        // let S = 0.5 * T + 0.5 * I;
        let m1 = parts[0].parse().expect("Error 1m");
        let m5 = parts[1].parse().expect("Error 5m");
        let m15 = parts[2].parse().expect("Error 15m");

        let incrementing_factor = m1 / m5 - 1.0;
        let ncpu = unsafe { N_CPU } as f64;
        let absolute_factor = ((m1 / ncpu) as f64).clamp(0.0, 1.0);
        let overall_factor = ((0.6 * incrementing_factor as f64) + 0.4 * absolute_factor).clamp(0.0, 1.0);
        let color = utils::get_color_gradient(0.0, 1.0, overall_factor, false);

        AvgLoadStats {
            m1: m1,
            m5: m5,
            m15: m15,
            ncpu: unsafe { N_CPU },
            critical_factor: overall_factor,
            color: color
        }
    } else {
        AvgLoadStats::default()
    }
}