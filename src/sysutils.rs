use serde::Serialize;
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