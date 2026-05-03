# ratatoskr - ᚱᚨᛏᚨᛏᛟᛊᚲᚱ

Ratatoskr is a high-performance system monitoring daemon written in Rust. It aggregates and evaluates real-time data on system resources (including RAM, swap, disk usage, battery status, and temperatures) to serve them directly to other processes.

Designed to provide "ready-to-use" metrics, Ratatoskr eliminates the need for individual applications to implement low-level monitoring logic. It broadcasts system states through Unix sockets and an optional, continuously updated JSON interface, making it an ideal backend for status bars, desktop widgets, or custom system tools.

On my laptop, Ratatoskr consumes ~5 MB of RAM.
The impact on average load is less than 0.001, so virtually zero. I measured the impact on average load as the ratio between the time spent with the Ratatoskr process in "Running" or "disk-sleep" status and the total measurement time.

## About the name

In Norse mythology, Ratatoskr is a squirrel that lives on the world tree, Yggdrasil. He is known for running up and down the tree, carrying messages (often insults) between the eagle perched at the top of the tree and the serpent Níðhöggr who dwells beneath one of the tree's roots.

## About this project

The aim of this project is to collect some metrics and supply them to other processes. Every process can read this file, ensuring consistency and sync in data visualization between launcher and statusbar in my machine and keeping these processes free of a lot of code.

The project contains now two variants of Ratatoskr: legacy-ratatoskr and ratatoskr

## legacy-ratatoskr (deprecated)
It's the legacy version of Ratatoskr, who writes all information in the file /tmp/ratatoskr.json

## ratatoskr (ex sock-ratatoskr)
It's the new version of Ratatoskr, who sends data through a socket located in /tmp/ratatoskr.sock (but can write a JSON too, in /tmp/ratatoskr.json).

Each message through the socket is send in following (JSON) format:
```
{
    resource: String,
    warning: f64,
    icon: Option<String>,
    data: {...stuff}
}
```

The key `data' holds the original message collected for a specific resource, for example:

```
{
    "resource":"temperature",
    "warning":0.0,
    "icon":"",
    "data": {
        "color":"#55FF00",
        "icon":"",
        "sensor":"Tctl",
        "value":74.5,
        "warn":0.0
    }
}
```

Keys `resource` and `warning` are always present, while icon and data are optional and depends on resource type.

## Configuration

You can configure warning ranges with a json file in ```~/.config/ratatoskr/config.json```:

```js
{
    "threshold_ram": [min, max] | null,
    "threshold_swap": [min, max] | null,
    "threshold_disk": [min, max] | null,
    "threshold_temperature": [min, max] | null,
    "threshold_avg_load": [min, max] | null,
    "threshold_battery": [min, max] | null,
    "threshold_wlan_signal": [min, max] | null,
    "threshold_volume_headphones": [min, max] | null,
    "threshold_volume_speakers": [min, max] | null,
    "threshold_bluetooth_battery": [min, max] | null,
    "write_json": true | false
}
```

As example and default values reference, check the following:
```js
{
    "threshold_ram": [60, 90],
    "threshold_swap": [60, 90],
    "threshold_disk": [60, 90],
    "threshold_temperature": [80, 99],
    "threshold_avg_load": [0.0, 1.0],
    "threshold_battery": [20, 70],
    "threshold_wlan_signal": [20, 60],
    "threshold_volume_headphones": [20, 90],
    "threshold_volume_speakers": [20, 90],
    "threshold_bluetooth_battery": [10, 30],
    "write_json": false
}
```

Every number can be represented as integer or decimal value.

Warning value is:
- 0.0 if resource value is less than the first threshold value
- between 0.0 and 1.0 if resource value is between the two threshold values
- 1.0 if resource value is greater than the second threshold value

For some resources, the logic is the opposite. These resources are battery, bluetooth_battery, wlan_signal.

### Load AVG

For warning computation, load average is normalized by the number of cpu and is kept in consideration if the 1m value is greater or less than the 5m value. The exact computation is the following:

```rust
let incrementing_factor = ((m1 / m5 - 1.0) as f64).clamp(-0.5, 1.0);
let absolute_factor = (((m1 - 1.0) / (ncpu - 1.0)) as f64).clamp(0.0, 1.0);
let overall_factor = ((0.5 * incrementing_factor as f64) + 1.0 * absolute_factor).clamp(0.0, 1.0);
```

### JSON output

If you set true as write_json, ratatoskr will write to disk /tmp/ratatoskr.json every 500 milliseconds, like legacy-ratatoskr was doing in the past. Socket sending will be always active, if a process is listening to.

## Tips
- You can check a socket output with following command: ```socat - UNIX-CONNECT:/tmp/ratatoskr.sock```

## Note

Please note that this is a personal project, for personal use, developed in my (not so much) free time. I'm learning Rust, so you'll not find clean code. I'm improving it over time. You've been warned.

## Known bugs
- ~~Bluetooth devices object is sent with warning 1.0 instead of 0.0~~ Solved!
- Sometimes, on bluetooth mouse connection, two icons appear (mouse and generic bt device)

## TODOs
- ~~Send only relevant updates (for example, ignore memory updates if less then 1% of change)~~ Done!
- ~~Send only relevant updates for display, network and disk, too~~ Done!
- ~~Add support for configurable alert thresholds for system resources~~ Done!
- ~~Make socket/json configurable~~ Done!
- ~~Make config optional~~ Done!
- Publish on AUR
- Create a GIF?
- Show again last notification on cmd retrieving?