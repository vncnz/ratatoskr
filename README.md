# ratatoskr - ᚱᚨᛏᚨᛏᛟᛊᚲᚱ

## About the name

In Norse mythology, Ratatoskr is a squirrel that lives on the world tree, Yggdrasil. He is known for running up and down the tree, carrying messages (often insults) between the eagle perched at the top of the tree and the serpent Níðhöggr who dwells beneath one of the tree's roots.

## About this project

The aim of this project is to collect some metrics and supply them to other processes. Every process can read this file, ensuring consistency and sync in data visualization between launcher and statusbar in my machine and keeping these processes free of a lot of code.

The project contains now two variants of Ratatoskr: legacy-ratatoskr and sock-ratatoskr

## legacy-ratatoskr
It's the legacy version of Ratatoskr, who writes all information in the file /tmp/ratatoskr.json

## sock-ratatoskr
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

## Tips
- You can check a socket output with following command: ```socat - UNIX-CONNECT:/tmp/ratatoskr.sock```

## Note

Please note that this is a personal project, for personal use, developed in my (not so much) free time. You'll not find clean code or a flexible, modular system here. You'll find lots of experiments, abandoned ideas, dead code, temporary hacks and workarounds. Oh, and last but not least, I'm just learning both Rust and RataTUI. You've been warned.

## Known bugs
- ~~Bluetooth devices object is sent with warning 1.0 instead of 0.0~~ Solved!

## TODOs
- ~~Send only relevant updates (for example, ignore memory updates if less then 1% of change)~~ Done!
- Send only relevant updates for ~~display~~ and network, too