use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod utils;
use utils::*;

mod sysutils;
use sysutils::*;

macro_rules! stat_updater {
    ($stats:expr, $interval:expr, $getter:expr, $field:ident) => {
        {
            let stats = Arc::clone(&$stats);
            thread::spawn(move || {
                loop {
                    loop {
                    let value = $getter();
                    if let Ok(mut data) = stats.lock() {
                        data.$field = value;
                    }
                    std::thread::sleep($interval);
                }
                }
            });
        }
    };
}


#[derive(Default, Serialize)]
struct SystemStats {
    ram: RamStats,
    disk: DiskStats,
    temperature: TempStats,
    weather: WeatherStats,
    loadavg: AvgLoadStats,
    volume: VolumeStats,
    battery: BatteryStats,
    written_at: u64,
    metronome: bool
}

fn main() {
    let output_path = "/tmp/ratatoskr.json";
    let output_niri_path = "/tmp/windows.json";
    let stats = Arc::new(Mutex::new(SystemStats::default()));

    let niristate_result = get_niri_situation();
    let niristate: Option<Arc<Mutex<niri_ipc::state::EventStreamState>>>;
    match niristate_result {
        Ok(l) => {
            println!("line: {:?}", &l);
            niristate = Some(l);
        },
        Err(e) => {
            eprintln!("Read error: {e}");
            niristate = None;
        }
    };

    stat_updater!(stats, Duration::from_secs(1), get_ram_info, ram);
    stat_updater!(stats, Duration::from_secs(5), get_disk_info, disk);
    stat_updater!(stats, Duration::from_secs(1), get_sys_temperatures, temperature);
    stat_updater!(stats, Duration::from_secs(600), get_weather, weather);
    stat_updater!(stats, Duration::from_millis(500), get_load_avg, loadavg);
    stat_updater!(stats, Duration::from_secs(1), get_volume, volume);
    stat_updater!(stats, Duration::from_secs(1), get_battery, battery);
    // stat_updater!(stats, Duration::from_secs(2), get_load_avg, network);

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

            if let Some(st) = &niristate {
                let niridata = st.lock().unwrap();
                if let Err(e) = write_niri_json_atomic(output_niri_path, &*niridata) {
                    eprintln!("Failed to write niri JSON: {e}");
                }
            }
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