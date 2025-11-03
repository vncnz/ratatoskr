use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use chrono::Utc;

use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::fs;

use ratatoskr::{SystemStats, utils::*};
use ratatoskr::sysutils::*;

const SOCK_PATH: &str = "/tmp/ratatoskr.sock";

macro_rules! stat_updater { // New version, standby-proof!
    ($stats:expr, $interval:expr, $getter:expr, $field:ident, $check_sleep:expr, $sock:expr, $name:expr) => {
        {
            let stats = Arc::clone(&$stats);
            let sock: Arc<Mutex<UnixDatagram>> = Arc::clone($sock);
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
                            let mut warn = 0.0;
                            if value.is_some() {
                                last_update = Utc::now();
                                warn = value.as_ref().unwrap().warn;
                            }
                            let msg = serde_json::json!({
                                "resource": $name,
                                "warning": warn,
                                "data": serde_json::to_value(&value).unwrap()
                            });
                            let json = msg.to_string();
                            // let json = serde_json::to_string(&value).unwrap();
                            let sent = sock.lock().unwrap().send_to(json.as_bytes(), SOCK_PATH);

                            data.$field = value;
                        }
                    }
                    std::thread::sleep(sleep_time);
                }
            });
        }
    };
}

fn main() {
    let output_path = "/tmp/ratatoskr.json";
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

    if Path::new(SOCK_PATH).exists() {
        fs::remove_file(SOCK_PATH).ok();
    }

    let sock = UnixDatagram::unbound().unwrap();
    let mut was_disconnected = false;

    let msock = Arc::new(Mutex::new(UnixDatagram::unbound().expect("Error msock")));

    stat_updater!(stats, Duration::from_secs(1), get_ram_info, ram, false, &msock, "memory");
    stat_updater!(stats, Duration::from_secs(5), get_disk_info, disk, false, &msock, "disk");
    stat_updater!(stats, Duration::from_secs(1), get_sys_temperatures, temperature, false, &msock, "temperature");
    stat_updater!(stats, Duration::from_secs(600), get_weather, weather, true, &msock, "weather");
    stat_updater!(stats, Duration::from_millis(500), get_load_avg, loadavg, false, &msock, "loadavg");
    stat_updater!(stats, Duration::from_secs(1), get_volume, volume, false, &msock, "volume");
    stat_updater!(stats, Duration::from_secs(1), get_battery, battery, false, &msock, "battery");
    stat_updater!(stats, Duration::from_secs(1), get_network_stats, network, false, &msock, "network");
    stat_updater!(stats, Duration::from_secs(1), get_brightness_stats, display, false, &msock, "brightness");

    loop {
        {
            if let Ok(mut data) = stats.lock() {
                data.written_at = get_unix_time();
                data.metronome = !data.metronome;
            }
            let data = stats.lock().unwrap();
            /* if let Err(e) = write_json_atomic(output_path, &*data) {
                eprintln!("Failed to write sysinfo JSON: {e}");
            } */

            let json = serde_json::to_string(&*data).unwrap();
            // sock.send_to(json.as_bytes(), sock_path).ok();
            let sent = sock.send_to(json.as_bytes(), SOCK_PATH);
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