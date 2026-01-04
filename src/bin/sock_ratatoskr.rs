use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Utc;

use std::fs;

use ratatoskr::{RamStats, SystemStats};
use ratatoskr::sysutils::*;

use std::sync::{mpsc};
use std::os::unix::net::{UnixListener, UnixStream};
use std::io::Write;

/*
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
    pub metronome: bool */

fn send_burst (s: &SystemStats, tx: mpsc::Sender<String>) {
    println!("About to send burst");

    // Sending only resources with a pooling time longer than 1s
    let fields: [(&str, serde_json::Value); 9] = [
        ("ram", serde_json::json!(s.ram)),
        ("disk", serde_json::json!(s.disk)),
        ("temperature", serde_json::json!(s.temperature)),
        ("weather", serde_json::json!(s.weather)),
        ("loadavg", serde_json::json!(s.loadavg)),
        ("volume", serde_json::json!(s.volume)),
        ("battery", serde_json::json!(s.battery)),
        ("network", serde_json::json!(s.network)),
        ("display", serde_json::json!(s.display)),
    ];

    // send("burst start".to_string(), value::Value::Null, Some(tx.clone()));

    // println!("{:?}", fields);
    for (key, value) in fields {
        // println!("Burst - sending {}", key);
        // println!("sending {key}");
        let _ = send(key.to_string(), value, Some(tx.clone()));
        // println!("Burst - sent {}? {}", key, sent);
    }
    // send("burst end".to_string(), value::Value::Null, Some(tx.clone()));
    println!("Burst sent");
}

pub fn start_socket_dispatcher(
    sock_path: &str,
    s: Arc<Mutex<SystemStats>>
) -> std::io::Result<mpsc::Sender<String>> {
    let _ = fs::remove_file(sock_path);
    let listener = UnixListener::bind(sock_path)?;
    listener.set_nonblocking(true)?;
    let clients = Arc::new(Mutex::new(Vec::<UnixStream>::new()));

    let (tx, rx) = mpsc::channel::<String>();
    let clients_accept = Arc::clone(&clients);
    let tx_clone = tx.clone();

    // Thread che accetta nuovi client
    thread::spawn(move || {
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    println!("{} New client connected", chrono::Local::now().format("%H:%M:%S%.3f"));
                    stream.set_nonblocking(true).ok();
                    clients_accept.lock().unwrap().push(stream);
                    println!("About to lock s and send burst");
                    if let Ok(data) = s.lock() {
                        // thread::sleep(Duration::from_millis(2000));
                        send_burst(&data, tx_clone.clone());
                        // tx_clone.send("burst".into()).ok();
                    }
                    println!("Unlocked s");
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => eprintln!("Accept error: {e}"),
            }
        }
    });

    // Thread che invia i messaggi ai client
    let clients_send = Arc::clone(&clients);
    thread::spawn(move || {
        for msg in rx {
            // eprintln!("msg in rx {:?}", msg);
            let mut lock = clients_send.lock().unwrap();
            // eprintln!("rx len {}, sending msg {:?}", lock.len(), msg);
            lock.retain_mut(|c| {
                // eprintln!("lock.retain_mut");
                if let Err(e) = c.write_all(format!("{}\n", msg).as_bytes()) {
                    eprintln!("Disconnected client ({e})");
                    return false;
                }
                true
            });
        }
    });

    Ok(tx)
}

fn send (name: String, value: serde_json::Value, tx: Option<mpsc::Sender<String>>) -> bool {
    // println!("{} Sending {}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), name);
    match tx {
        Some(ttx) => {
            let json_val = serde_json::to_value(&value).unwrap_or_default();
            let warn = json_val.get("warn").and_then(|v| v.as_f64()).unwrap_or(1.0);
            let icon = json_val.get("icon").and_then(|v| v.as_str()).unwrap_or("");

            /* if name == "bt-batteries" {
                println!("\n\n{name} {warn} {icon} {json_val}\n\n");
            } */

            let msg = serde_json::json!({
                "resource": name,
                "warning": warn,
                "icon": icon,
                "data": serde_json::to_value(&value).unwrap()
            });
            let json = msg.to_string();
            // let json = serde_json::to_string(&value).unwrap();
    
            if ttx.clone().send(json).is_err() {
                false
            } else {
                true
            }
        },
        _ => {
            false
        }
    }

}


macro_rules! stat_updater { // New version, standby-proof!
    ($stats:expr, $interval:expr, $getter:expr, $field:ident, $check_sleep:expr, $comparator:expr, $tx:expr, $name:expr) => {
        {
            let stats = Arc::clone(&$stats);
            let tx = $tx.clone();
            thread::spawn(move || {
                let mut last_update = Utc::now() - $interval;
                let sleep_time = if $check_sleep { std::cmp::min($interval, Duration::from_secs(1)) } else { $interval };
                loop {
                    let run_now = if $check_sleep { Utc::now() >= last_update + $interval } else { true };
                    /* if $check_sleep {
                        println!("{:?} {:?}", Utc::now(), last_update + $interval);
                    } */
                    if run_now {
                        let new_value = $getter();

                        if let Ok(mut data) = stats.lock() {

                            if new_value.is_some() {
                                last_update = Utc::now();

                                let should_send = match (&data.$field, &new_value) {
                                    (Some(old), Some(new)) => $comparator(old, new),
                                    _ => true
                                };

                                if should_send {
                                    let json_val = serde_json::to_value(&new_value).unwrap_or_default();
                                    if !send($name.to_string(), json_val, tx.clone()) {
                                        eprintln!("Dispatcher terminato, chiudo thread di {}", $name);
                                        break;
                                    }
                                }
                            }
                            data.$field = new_value;
                        }




                        /* let value = $getter();
                        if let Ok(mut data) = stats.lock() {
                            if value.is_some() {
                                last_update = Utc::now();
                                // warn = value.as_ref().unwrap().warn;
                                // icon = value.as_ref().unwrap().icon;
                                let json_val = serde_json::to_value(&value).unwrap_or_default();
                                if !send($name.to_string(), json_val, tx.clone()) {
                                    eprintln!("Dispatcher terminato, chiudo thread di {}", $name);
                                    break;
                                }
                            }

                            data.$field = value;
                        } */
                    }
                    std::thread::sleep(sleep_time);
                }
            });
        }
    };
}

fn always_changed<T>(_: &T, _: &T) -> bool {
    true
}

fn main() {
    // let output_path = "/tmp/ratatoskr.json";
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

    let tx = start_socket_dispatcher("/tmp/ratatoskr.sock", stats.clone()).ok();

    /*if Path::new(SOCK_PATH).exists() {
        fs::remove_file(SOCK_PATH).ok();
    }

    let sock = UnixDatagram::unbound().unwrap(); */
    // let mut was_disconnected = false;

    // let msock = Arc::new(Mutex::new(UnixDatagram::unbound().expect("Error msock")));

    stat_updater!(stats, Duration::from_secs(1), get_ram_info, ram, false, |old: &RamStats, new: &RamStats| old.mem_percent.abs_diff(new.mem_percent) >= 1 || old.swap_percent.abs_diff(new.swap_percent) >= 1, &tx, "ram");
    stat_updater!(stats, Duration::from_secs(5), get_disk_info, disk, false, always_changed, &tx, "disk");
    stat_updater!(stats, Duration::from_secs(1), get_sys_temperatures, temperature, false, always_changed, &tx, "temperature");
    stat_updater!(stats, Duration::from_secs(600), get_weather, weather, true, always_changed, &tx, "weather");
    stat_updater!(stats, Duration::from_millis(500), get_load_avg, loadavg, false, always_changed, &tx, "loadavg");
    // stat_updater!(stats, Duration::from_secs(1), get_volume, volume, false, &tx, "volume");
    stat_updater!(stats, Duration::from_secs(1), get_battery, battery, false, always_changed, &tx, "battery");
    stat_updater!(stats, Duration::from_secs(1), get_network_stats, network, false, always_changed, &tx, "network");
    stat_updater!(stats, Duration::from_secs(1), get_brightness_stats, display, false, always_changed, &tx, "display");


    let (tx_audio, rx_audio) = std::sync::mpsc::channel();
    spawn_volume_listener(tx_audio);

    let (tx_bluetooth, rx_bluetooth) = std::sync::mpsc::channel();
    spawn_upower_listener(tx_bluetooth);
    // print_bt_batteries();

    loop {
        if let Ok(mut data) = stats.lock() {
            data.written_at = get_unix_time();
            data.metronome = !data.metronome;

            // send_burst(&data, tx.clone().unwrap());
        }
        while let Ok(volume_obj) = rx_audio.try_recv() {
            if let Ok(mut data) = stats.lock() {
                let json_val = serde_json::to_value(&volume_obj).unwrap_or_default();
                if !send("volume".to_string(), json_val, tx.clone()) {
                    // eprintln!("Dispatcher terminato, chiudo thread di {}", $name);
                    // break;
                }
                data.volume = Some(volume_obj);
            }
        }

        while let Ok(batterydevice_obj) = rx_bluetooth.try_recv() {
            // println!("Bluetooth update {:?}", batterydevice_obj);
            if let Ok(mut data) = stats.lock() {
                let json_val = serde_json::to_value(&batterydevice_obj).unwrap_or_default();
                if !send("bt-batteries".to_string(), json_val, tx.clone()) {
                    // eprintln!("Dispatcher terminato, chiudo thread di {}", $name);
                    // break;
                }
                data.bluetooth_batteries = Some(batterydevice_obj);
            }
        }

        // let data = stats.lock().unwrap();
        /* if let Err(e) = write_json_atomic(output_path, &*data) {
            eprintln!("Failed to write sysinfo JSON: {e}");
        } */

        // let json = serde_json::to_string(&*data).unwrap();
        

        /* if let Some(st) = &niristate {
            let niridata = st.lock().unwrap();
            if let Err(e) = write_niri_json_atomic(output_niri_path, &*niridata) {
                eprintln!("Failed to write niri JSON: {e}");
            }
        } */
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