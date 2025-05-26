use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod utils;
use utils::*;

mod sysutils;
use sysutils::*;


#[derive(Default, Serialize)]
struct SystemStats {
    ram: RamStats,
    disk: DiskStats,
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

   { // Thread RAM
        let stats = Arc::clone(&stats);
        thread::spawn(move || {
            loop {
                let ram_val = get_ram_info();
                let mut data = stats.lock().unwrap();
                data.ram = ram_val;
                drop(data);
                thread::sleep(Duration::from_secs(2));
            }
        });
    }

    { // Thread Disk
        let stats = Arc::clone(&stats);
        thread::spawn(move || {
            loop {
                let disk_val = get_disk_info(); // sostituisci col vero valore
                let mut data = stats.lock().unwrap();
                data.disk = disk_val;
                drop(data);
                thread::sleep(Duration::from_secs(2));
            }
        });
    }

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