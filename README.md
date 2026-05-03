# ratatoskr - ᚱᚨᛏᚨᛏᛟᛊᚲᚱ

## About the name

In Norse mythology, Ratatoskr is a squirrel that lives on the world tree, Yggdrasil. He is known for running up and down the tree, carrying messages (often insults) between the eagle perched at the top of the tree and the serpent Níðhöggr who dwells beneath one of the tree's roots.

## About this project

The aim of this project is to collect some metrics and supply them to other processes. Every process can read this file, ensuring consistency and sync in data visualization between launcher and statusbar in my machine and keeping these processes free of a lot of code.

The project contains now two variants of Ratatoskr: legacy-ratatoskr and ratatoskr

## legacy-ratatoskr (deprecated)
It's the legacy version of Ratatoskr, who writes all information in the file /tmp/ratatoskr.json

## ratatoskr (ex sock-ratatoskr)
It's the new version of Ratatoskr, who sends data through a socket located in /tmp/ratatoskr.sock

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

```json
{
    "threshold_ram": [min, max] | null,
    "threshold_swap": [min, max] | null,
    "threshold_disk": [min, max] | null,
    "threshold_temperature": [min, max] | null,
    "threshold_avg_load": [min, max] | null,
    "threshold_battery": [min, max] | null,
    "threshold_volume_headphones": [min, max] | null,
    "threshold_volume_speakers": [min, max] | null,
    "threshold_bluetooth_battery": [min, max] | null,
    "write_json": true | false
}
```

As example and default values reference, check the following:
```json
{
    "threshold_ram": [60, 90],
    "threshold_swap": [60, 90],
    "threshold_disk": [60, 90],
    "threshold_temperature": [80, 99],
    "threshold_avg_load": [0.0, 1.0],
    "threshold_battery": [20, 70],
    "threshold_volume_headphones": [20, 90],
    "threshold_volume_speakers": [20, 90],
    "threshold_bluetooth_battery": [10, 30],
    "write_json": false
}
```

Every number can be represented as integer or decimal value.

If you set true as write_json, ratatoskr will write to disk /tmp/ratatoskr.json every 500 milliseconds, like legacy-ratatoskr was doing in the past. Socket sending will be always active, if a process is listening to.

## Tips
- You can check a socket output with following command: ```socat - UNIX-CONNECT:/tmp/ratatoskr.sock```

## Note

Please note that this is a personal project, for personal use, developed in my (not so much) free time. I'm learning Rust, so you'll not find clean code. I'm improving it over time. You've been warned.

## Known bugs
- ~~Bluetooth devices object is sent with warning 1.0 instead of 0.0~~ Solved!
- With bluetooth mouse, sometimes two icons appear (mouse and generic bt device)

## TODOs
- ~~Send only relevant updates (for example, ignore memory updates if less then 1% of change)~~ Done!
- ~~Send only relevant updates for display, network and disk, too~~ Done!
- ~~Add support for configurable alert thresholds for system resources~~ Done!
- ~~Make socket/json configurable~~ Done!
- ~~Make config optional~~ Done!
- Publish on AUR
- Create a GIF?