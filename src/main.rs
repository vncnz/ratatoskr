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
    weather: WeatherStats
}

fn main() {
    let output_path = "/tmp/ratatoskr.json";
    let stats = Arc::new(Mutex::new(SystemStats::default() /*{
        ram: RamStats {
            total_memory: 0,
            used_memory: 0,
            total_swap: 0,
            used_swap: 0,
            mem_percent: 0,
            swap_percent: 0,
        },
        disk: DiskStats {
            total_size: 0,
            used_size: 0,
            used_percent: 0,
        },
    }*/));

    /* loop {
        let stats = SystemStats {
            ram: get_ram_info(),
            disk: get_disk_info()
        };

        if let Err(e) = write_json_atomic(output_path, &stats) {
            eprintln!("Failed to write stats: {}", e);
        }

        thread::sleep(Duration::from_secs(2));
    } */

    stat_updater!(stats, Duration::from_secs(2), get_ram_info, ram);
    stat_updater!(stats, Duration::from_secs(2), get_disk_info, disk);
    stat_updater!(stats, Duration::from_secs(2), get_sys_temperatures, temperature);
    stat_updater!(stats, Duration::from_secs(600), get_weather, weather);

    loop {
            {
        let data = stats.lock().unwrap();
            if let Err(e) = write_json_atomic(output_path, &*data) {
                eprintln!("Failed to write JSON: {e}");
            }
        }
        thread::sleep(Duration::from_secs(2));
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