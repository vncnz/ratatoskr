use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use chrono::Utc;

use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::fs;

mod utils;
use utils::*;

mod sysutils;
use sysutils::*;

macro_rules! stat_updater { // New version, standby-proof!
    ($stats:expr, $interval:expr, $getter:expr, $field:ident, $check_sleep:expr) => {
        {
            let stats = Arc::clone(&$stats);
            thread::spawn(move || {
                let mut last_update = Utc::now() - $interval;
                let sleep_time = if $check_sleep { std::cmp::min($interval, Duration::from_secs(1)) } else { $interval };
                loop {
                    let run_now = if $check_sleep { Utc::now() >= last_update + $interval } else { true };
                    /* if $check_sleep {
                        println!("{:?} {:?}", Utc::now(), last_update + $interval);
                    } */
                    if run_now {
                        let value = $getter();
                        if let Ok(mut data) = stats.lock() {
                            if value.is_some() { last_update = Utc::now(); }
                            data.$field = value;
                        }
                    }
                    std::thread::sleep(sleep_time);
                }
            });
        }
    };
}

#[derive(Default, Serialize)]
struct SystemStats {
    ram: Option<RamStats>,
    disk: Option<DiskStats>,
    temperature: Option<TempStats>,
    weather: Option<WeatherStats>,
    loadavg: Option<AvgLoadStats>,
    volume: Option<VolumeStats>,
    battery: Option<BatteryStats>,
    network: Option<NetworkStats>,
    display: Option<EmbeddedDisplayStats>,
    written_at: u64,
    metronome: bool
}

fn main() {
    let output_path = "/tmp/ratatoskr.json";
    let sock_path = "/tmp/ratatoskr.sock";
    // let output_niri_path = "/tmp/windows.json";
    let stats = Arc::new(Mutex::new(SystemStats::default()));

    /* let niristate_result = get_niri_situation();
    let niristate: Option<Arc<Mutex<niri_ipc::state::EventStreamState>>>;
    match niristate_result {
        Ok(l) => {
            // println!("line: {:?}", &l);
            niristate = Some(l);
        },
        Err(e) => {
            eprintln!("Read error: {e}");
            niristate = None;
        }
    }; */

    stat_updater!(stats, Duration::from_secs(1), get_ram_info, ram, false);
    stat_updater!(stats, Duration::from_secs(5), get_disk_info, disk, false);
    stat_updater!(stats, Duration::from_secs(1), get_sys_temperatures, temperature, false);
    stat_updater!(stats, Duration::from_secs(600), get_weather, weather, true);
    stat_updater!(stats, Duration::from_millis(500), get_load_avg, loadavg, false);
    stat_updater!(stats, Duration::from_secs(1), get_volume, volume, false);
    stat_updater!(stats, Duration::from_secs(1), get_battery, battery, false);
    stat_updater!(stats, Duration::from_secs(1), get_network_stats, network, false);
    stat_updater!(stats, Duration::from_secs(1), get_brightness_stats, display, false);

    if Path::new(sock_path).exists() {
        fs::remove_file(sock_path).ok();
    }

    let sock = UnixDatagram::unbound().unwrap();
    let mut was_disconnected = false;

    loop {
        {
            if let Ok(mut data) = stats.lock() {
                data.written_at = get_unix_time();
                data.metronome = !data.metronome;
            }
            let data = stats.lock().unwrap();
            if let Err(e) = write_json_atomic(output_path, &*data) {
                eprintln!("Failed to write sysinfo JSON: {e}");
            }

            let json = serde_json::to_string(&*data).unwrap();
            // sock.send_to(json.as_bytes(), sock_path).ok();
            let sent = sock.send_to(json.as_bytes(), sock_path);
            match sent {
                Ok(_) => {
                    if was_disconnected {
                        println!("Reconnected!");
                    }
                    was_disconnected = false;
                },
                Err(_) => {

                    if !was_disconnected {
                        println!("Disconnected!");
                    }
                    was_disconnected = true;
                }
                
            }

            /* if let Some(st) = &niristate {
                let niridata = st.lock().unwrap();
                if let Err(e) = write_niri_json_atomic(output_niri_path, &*niridata) {
                    eprintln!("Failed to write niri JSON: {e}");
                }
            } */
        }
        thread::sleep(Duration::from_millis(500));
    }
}



/* In other processes we can register a listener for changes in the file, when a file is changed the kernel notifies us:

RUST:

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event};
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(500))?;

    watcher.watch("/tmp/ratatoskr.json", RecursiveMode::NonRecursive)?;

    for event in rx {
        match event {
            Ok(Event { kind, .. }) => println!("File changed: {kind:?}"),
            Err(e) => eprintln!("Watch error: {e:?}"),
        }
    }

    Ok(())
}


BASH:

#!/bin/bash
inotifywait -m -e close_write /tmp/ratatoskr.json |
while read -r _; do
    echo "update"  # o un valore da usare in EWW
done

*/