use std::process::Command;

use sysinfo::{Disks, System};
use chrono::Utc;

use crate::{AvgLoadStats, BatteryStats, DiskStats, EmbeddedDisplayStats, NetworkStats, RamStats, TempStats, VolumeObj, VolumeStats, WeatherStats, utils};



pub fn get_ram_info () -> Option<RamStats> {
    let mut sys = System::new();
    sys.refresh_memory();
    let tm = sys.total_memory();
    let um = sys.used_memory();
    let ts = sys.total_swap();
    let us = sys.used_swap();
    let mp = 100 * um / tm;

    if ts > 0 {
        let sp = 100 * us / ts;

        let mem_warn = utils::get_warn_level(60.0, 90.0, mp as f64, false);
        let swap_warn = utils::get_warn_level(60.0, 90.0, sp as f64, false);
        let warn = f64::max(mem_warn, swap_warn);
        Some(RamStats {
            total_memory: tm,
            used_memory: um,
            total_swap: ts,
            used_swap: us,
            mem_percent: mp,
            swap_percent: sp,
            mem_color: utils::get_color_gradient(60.0, 90.0, mp as f64, false),
            swap_color: utils::get_color_gradient(60.0, 90.0, sp as f64, false),
            mem_warn,
            swap_warn,
            warn
        })
    } else {
        let mem_warn = utils::get_warn_level(60.0, 90.0, mp as f64, false);
        Some(RamStats {
            total_memory: tm,
            used_memory: um,
            total_swap: 0,
            used_swap: 0,
            mem_percent: mp,
            swap_percent: 0,
            mem_color: utils::get_color_gradient(60.0, 90.0, mp as f64, false),
            swap_color: utils::get_color_gradient(0.0, 1.0, 0.0, false),
            mem_warn,
            swap_warn: 0.0,
            warn: mem_warn
        })
    }
}


pub fn get_disk_info () -> Option<DiskStats> {
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        if (disk as &sysinfo::Disk).mount_point() == std::path::Path::new("/") {
            if let Some(_name_str) = (disk as &sysinfo::Disk).name().to_str() {
                if let Some(_mount_str) = (disk as &sysinfo::Disk).mount_point().to_str() {
                    let tos = (disk as &sysinfo::Disk).total_space();
                    let avs = (disk as &sysinfo::Disk).available_space();
                    let up = 100 - (avs * 100 / tos);
                    return Some(DiskStats {
                        // name_str.to_string(),
                        // mount_str.to_string(),
                        total_size: tos,
                        used_size: tos - avs,
                        used_percent: up,
                        color: utils::get_color_gradient(60.0, 90.0, up as f64, false),
                        warn: utils::get_warn_level(60.0, 90.0, up as f64, false)
                    })
                }
            }
        }
    }
    // SysUpdate::Error("Disk not found".to_string())
    Some(DiskStats { total_size: 0, used_size: 0, used_percent: 100, color: "#FF0000".to_string(), warn: 1.0 })
}

pub fn get_sys_temperatures () -> Option<TempStats> {
    let components = sysinfo::Components::new_with_refreshed_list();
    for component in &components {
        if component.label() == "Tctl" {
            if let Some(temp) = component.temperature() {
                let icon = if temp < 80.0 { "" } else 
                                         if temp < 85.0 { "" } else
                                         if temp < 90.0 { "" } else
                                         if temp < 95.0 { "" } else { "" };
                let color = utils::get_color_gradient(80.0, 99.0, temp as f64, false);
                return Some(TempStats {
                    sensor: component.label().into(),
                    value: temp,
                    color: Some(color),
                    icon: icon.into(),
                    warn: utils::get_warn_level(80.0, 99.0, temp as f64, false)
                });
            } else {
                return Some(TempStats {
                    sensor: component.label().into(),
                    value: 0.0,
                    color: None,
                    icon: "󱔱".into(),
                    warn: 0.0
                });
            }
        }
    }
    Some(TempStats {
        sensor: "".into(),
        value: 0.0,
        color: None,
        icon: "󱔱".into(),
        warn: 0.0
    })
}


pub fn get_volume () -> Option<VolumeStats> {
    let output = Command::new("/home/vncnz/.config/eww/scripts/volume.sh").arg("json").output();
    let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
    // println!("\n{:?}", stdout);
    let vol: Result<VolumeObj, _> = serde_json::from_str(&stdout);
    if let Ok(volume) = vol {
        return Some(VolumeStats {
            color: utils::get_color_gradient(40.0, 100.0, volume.value as f64, false),
            icon: volume.icon,
            value: volume.value,
            clazz: volume.clazz,
            warn: 0.0
        })
    } else {
        Some(VolumeStats::default())
    }
}

use std::time::{SystemTime, UNIX_EPOCH};
pub fn get_unix_time () -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => 0,
    }
}
/*
fn get_brightness () -> SysUpdate {
    let output = Command::new("/home/vncnz/.config/eww/scripts/brightness.sh").arg("json").output();
    let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
    // println!("\n{:?}", stdout);
    if let Ok(brightness) = serde_json::from_str(&stdout) {
        SysUpdate::Brightness(brightness)
    } else {
        SysUpdate::Error("Error with serde and brightness data".to_string())
    }
}
*/

pub fn get_weather () -> Option<WeatherStats> {
    let output = Command::new("/home/vncnz/.config/eww/scripts/meteo.sh").arg("Desenzano Del Garda").arg("45.457692").arg("10.570684").output();
    let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
    // println!("\n{:?}", stdout);
    // let weather: WeatherObj;
    if let Ok(mut weather) = serde_json::from_str::<WeatherStats>(&stdout) {
        weather.updated = Some(format!("{}", Utc::now().to_rfc3339()));
        let mut temp_warn: f64 = 0.0;
        if weather.temp > 28 {
            temp_warn = f64::clamp(weather.temp as f64 / 36.0, 0.0, 1.0);
        } else if weather.temp < 10 {
            temp_warn = f64::clamp((10.0 - weather.temp as f64) / 10.0 , 0.0, 1.0);
        }
        match weather.text.as_str() {
            "Overcast" => { temp_warn += 0.2; },
            "Fog" => { temp_warn += 0.3; },
            "Depositing rime fog" => { temp_warn += 0.3; },
            "Rain (slight)" => { temp_warn += 0.2; },
            "Rain (moderate)" => { temp_warn += 0.4; },
            "Rain (heavy)" => { temp_warn += 0.6; },
            "Thunderstorm" => { temp_warn += 0.8; },

            _ => {}
        }
        weather.warn = Some(temp_warn);
        return Some(weather)
    } else {
        eprintln!("Error weather");
        eprintln!("{}", stdout);
    }
    // WeatherStats::default()
    return None
}

// static mut N_CPU: usize = 0;

use once_cell::sync::Lazy;
static CORE_COUNT: Lazy<usize> = Lazy::new(|| {
    std::fs::read_to_string("/proc/cpuinfo")
        .map(|contents| {
            contents
                .lines()
                .filter(|line| line.starts_with("processor"))
                .count()
        })
        .unwrap_or(1)
});

pub fn get_load_avg() -> Option<AvgLoadStats> {
    /* if unsafe { N_CPU } == 0 {
        unsafe { N_CPU = std::fs::read_to_string("/proc/cpuinfo")
            .map(|contents| {
                contents
                    .lines()
                    .filter(|line| line.starts_with("processor"))
                    .count()
            })
            .unwrap_or(1) } // fallback: almeno 1 core
    } */
    if let Ok(output) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = output.split_whitespace().collect();
        let ncpu = *CORE_COUNT as f64;

        // let T = clamp((load1 / load5 - 1.0) / 1.0, 0.0, 1.0);
        // let I = clamp(load1 / custom_max_load, 0.0, 1.0);
        // let S = 0.5 * T + 0.5 * I;
        let m1 = parts[0].parse().expect("Error 1m");
        let m5 = parts[1].parse().expect("Error 5m");
        let m15 = parts[2].parse().expect("Error 15m");

        let incrementing_factor = ((m1 / m5 - 1.0) as f64).clamp(-0.5, 1.0);
        let absolute_factor = (((m1 - 1.0) / (ncpu - 1.0)) as f64).clamp(0.0, 1.0);
        let overall_factor = ((0.5 * incrementing_factor as f64) + 1.0 * absolute_factor).clamp(0.0, 1.0);
        // println!("0.5*{incrementing_factor} + 1.0*{absolute_factor} = {overall_factor}");
        let color = utils::get_color_gradient(0.0, 1.0, overall_factor, false);

        Some(AvgLoadStats {
            m1: m1,
            m5: m5,
            m15: m15,
            ncpu: *CORE_COUNT,
            warn: overall_factor,
            color: color
        })
    } else {
        Some(AvgLoadStats::default())
    }
}

use battery::{Manager, State};

pub fn get_battery() -> Option<BatteryStats> {
    let manager = match Manager::new() {
        Ok(m) => m,
        Err(_) => {
            return Some(BatteryStats {
                percentage: 0,
                capacity: 0.0,
                eta: None,
                state: "no_manager".to_string(),
                icon: "󰂑".to_string(),
                color: None,
                watt: 0.0,
                warn: 0.0
            })
        }
    };

    let batteries = manager.batteries();

    if batteries.is_err() {
        return Some(BatteryStats {
            percentage: 0,
            capacity: 0.0,
            eta: None,
            state: "no_battery".to_string(),
            icon: "".to_string(),
            color: None,
            watt: 0.0,
            warn: 0.0
        });
    }

    if let Some(Ok(battery)) = batteries.unwrap().next() {

        let percentage = ((battery.state_of_charge().value * 100.0) as f32).round() as i32;
        let capacity = battery.energy_full().value as f32;
        let watt = battery.energy_rate().value as f32;

        let eta = match battery.time_to_empty().or(battery.time_to_full()) {
            Some(t) => Some((t.value as f32) / 60.0),  // Converte da secondi a minuti
            _ => None,
        };

        let state = match battery.state() {
            State::Charging => "Charging",
            State::Discharging => "Discharging",
            State::Full => "Full",
            State::Empty => "Empty",
            State::Unknown => "Unknown",
            _ => "Strage"
        }
        .to_string();

        let icon = (match battery.state() {
            State::Charging => "󰂄",
            State::Discharging => {
                if percentage < 15 { "󰁺" }
                else if percentage < 25 { "󰁻" }
                else if percentage < 35 { "󰁼" }
                else if percentage < 45 { "󰁽" }
                else if percentage < 55 { "󰁾" }
                else if percentage < 65 { "󰁿" }
                else if percentage < 75 { "󰂀" }
                else if percentage < 85 { "󰂁" }
                else if percentage < 95 { "󰂂" }
                else { "󰁹" }
            },
            State::Full | State::Unknown => "󱟢",
            State::Empty => "Empty",
            // State::Unknown => "󰂑",
            _ => "󱧥"
        }).to_string();

        let color = Some(utils::get_color_gradient(20.0, 70.0, percentage as f64, true));

        Some(BatteryStats {
            percentage,
            capacity,
            eta,
            state,
            icon,
            color,
            watt,
            warn: (100.0 - (percentage as f64)) / 100.0
        })
    } else {
        Some(BatteryStats {
            percentage: 0,
            capacity: 0.0,
            eta: None,
            state: "no_battery".to_string(),
            icon: "".to_string(),
            color: None,
            watt: 0.0,
            warn: 0.0
        })
    }
}

// use std::process::Command;

pub fn get_network_stats() -> Option<NetworkStats> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,TYPE,STATE,CONNECTION", "device"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 4 || parts[2] != "connected" {
            continue;
        }

        let iface = parts[0].to_string();
        let ip = get_ip(&iface);
        let conn_type = parts[1].to_string();
        // let mut icon = if conn_type == "ethernet" { "󰈀" } else { "󰞃" };
        let mut color: Option<String> = None;
        let mut warn = 0.0;
        if conn_type == "wifi" {
            // SSID
            let out = Command::new("nmcli")
                .args(["-t", "-f", "ACTIVE,SSID,SIGNAL", "dev", "wifi"])
                .output()
                .ok()?;
            let lines = String::from_utf8_lossy(&out.stdout);
            for wifi_line in lines.lines() {
                let wifi_parts: Vec<&str> = wifi_line.split(':').collect();
                let signal = wifi_parts[2].parse().ok();
                let mut icon = "󰞃";
                if let Some(sig) = signal {
                    if sig < 15 { icon = "󰢿"; }
                    else if sig < 30 { icon = "󰢼"; }
                    else if sig < 60 { icon = "󰢽"; }
                    else { icon = "󰢾"; }
                    color = Some(utils::get_color_gradient(20.0, 60.0, sig as f64, true));
                    warn = utils::get_warn_level(20.0, 60.0, sig as f64, true)
                }

                if wifi_parts.len() >= 3 && wifi_parts[0] == "yes" {
                    return Some(NetworkStats {
                        iface,
                        conn_type,
                        ssid: Some(wifi_parts[1].to_string()),
                        signal,
                        ip,
                        icon: icon.to_string(),
                        color,
                        warn
                    });
                }
            }
        } else if conn_type == "ethernet" {
            return Some(NetworkStats {
                iface,
                conn_type,
                ssid: None,
                signal: None,
                ip,
                icon: "󰈀".to_string(),
                color,
                warn
            })
        };
    }
    None
}

fn get_ip(iface: &str) -> Option<String> {
    let out = Command::new("nmcli")
        .args(["-g", "IP4.ADDRESS", "device", "show", iface])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines().next().map(|s| s.split('/').next().unwrap_or("").to_string())
}

pub fn get_brightness_stats() -> Option<EmbeddedDisplayStats> {
    
    let output = Command::new("sh")
        .arg("-c")
        .arg("brightnessctl g; brightnessctl m")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();

    let brightness_current = lines.next()?.trim().parse::<u32>().ok()?;
    let brightness_max = lines.next()?.trim().parse::<u32>().ok()?;
    let perc = (100.0 * (brightness_current as f32) / (brightness_max as f32)).round() as u8;

    let icons = ["", "", "", "", "", "", "", "", "", "", "", "", "", ""];
    let icon_idx = ((brightness_current as f32) / (brightness_max as f32) * (icons.len() as f32)).round() as usize;
    let icon = icons[icon_idx].into();

    Some(EmbeddedDisplayStats {
        brightness_current,
        brightness_max,
        perc,
        icon,
        warn: 0.0
    })
}


use niri_ipc::{
    socket::SOCKET_PATH_ENV,
    state::{EventStreamState, EventStreamStatePart},
    Event, Request,
};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

pub fn get_niri_situation () -> std::io::Result<Arc<Mutex<EventStreamState>>> {
    let socket_path = env::var(SOCKET_PATH_ENV).expect("Variabile d'ambiente NIRI_SOCKET non impostata");

    // Connetti al socket
    let stream = UnixStream::connect(socket_path)?;
    let reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    // Invia la richiesta per avviare il flusso di eventi
    let request = serde_json::to_string(&Request::EventStream).unwrap();
    writeln!(writer, "{}", request)?;
    writer.flush()?;

    // Inizializza lo stato degli eventi
    let state = Arc::new(Mutex::new(EventStreamState::default()));
    let state_clone = Arc::clone(&state);

    std::thread::spawn(move || {
        // Leggi e gestisci gli eventi in arrivo
        for line in reader.lines() {
            let line = match line {
                Ok(l) => {
                    // println!("line: {:?}", &l);
                    l
                },
                Err(e) => {
                    eprintln!("Read error: {e}");
                    continue;
                }
            };

            if line.trim() == r#"{"Ok":"Handled"}"# {
                continue;
            }

            /*
            let event: Event = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Errore nel parsing dell'evento: {e}\nContenuto: {line}");
                    continue;
                }
            };
            */

            let evt = serde_json::from_str(&line);
            // println!("evt: {:?}", &evt);
            let event: Event = evt.unwrap();
            let mut s = state_clone.lock().unwrap();
            s.apply(event);

            // Stampa la lista aggiornata delle finestre
            /* println!("Finestre attuali:");
            for window in s.windows.windows.values() {
                println!(
                    "- ID: {}, Titolo: {:?}, App: {:?}",
                    window.id, window.title, window.app_id
                );
            } */
        }
    });

    Ok(state)
}

#[derive(Debug, Clone, Default)]
pub struct AudioState {
    pub volume: u32,
    pub muted: bool,
    pub port_name: Option<String> // Es: "analog-output-headphones"
}

/*pub fn get_audio_situation() -> std::io::Result<Arc<Mutex<AudioState>>> {
    let audio_state = Arc::new(Mutex::new(AudioState::default()));
    let state_clone = Arc::clone(&audio_state);

    thread::spawn(move || {
        let mut proplist = Proplist::new().unwrap();
        proplist.set_str(pulse::proplist::properties::APPLICATION_NAME, "ratatoskr-audio").unwrap();

        let mut mainloop = Mainloop::new().unwrap();
        let mut context = Context::new_with_proplist(&mainloop, "ratatoskr-context", &proplist).unwrap();

        context.connect(None, pulse::context::FlagSet::NOFLAGS, None).unwrap();
        while let IterateResult::Success(_) = mainloop.iterate(false) {
            if let pulse::context::State::Ready = context.get_state() {
                break;
            }
        }

        let introspect = context.introspect();
        introspect.get_sink_info_list(move |result| {
            if let ListResult::Item(sink) = result {
                let mut data = state_clone.lock().unwrap();
                data.volume = sink.volume.avg().0;
                data.muted = sink.mute;
            }
        });

        // Subscribing to events
        let context = Arc::new(Mutex::new(context));
        let state_clone = Arc::new(Mutex::new(state_clone));
        let context_clone = Arc::clone(&context);
        let state_clone_2 = Arc::clone(&state_clone);
        context.lock().unwrap().set_subscribe_callback(Some(Box::new(move |facility, _op, _idx| {
            if facility == Some(pulse::context::subscribe::Facility::Sink) {
                let introspect = context_clone.lock().unwrap().introspect();
                
                introspect.get_sink_info_list(move |result| {
                    if let ListResult::Item(sink) = result {
                        let mut data = state_clone_2.lock().unwrap();
                        data.volume = sink.volume.avg().0;
                        data.muted = sink.mute;
                        data.port_name = sink
                            .active_port
                            .as_ref()
                            .and_then(|p| p.name.as_ref().map(|name| name.to_string()));
                    }
                });
            }
        })));


        context.lock().unwrap().subscribe(pulse::context::subscribe::InterestMaskSet::SINK, |_| {});
        loop {
            mainloop.iterate(true);
        }
    });

    Ok(audio_state)
}*/