use std::fs;
use std::io::Write;
use std::path::Path;

use crate::SystemStats;

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r1 + m) * 255.0).round() as u8;
    let g = ((g1 + m) * 255.0).round() as u8;
    let b = ((b1 + m) * 255.0).round() as u8;

    (r, g, b)
}

pub fn get_color_gradient(min: f64, max: f64, value: f64, reversed: bool) -> String {
    let clamped = value.clamp(min, max);
    let mut ratio = if (max - min).abs() < f64::EPSILON {
        0.5
    } else {
        (clamped - min) / (max - min)
    };

    if !reversed { ratio = 1.0 - ratio; }
    let sat = f64::max(1.0 - (ratio * ratio * ratio), 0.0);
    let hue = 60.0 * ratio; // 120 -> 0
    let (r, g, b) = hsv_to_rgb(hue, sat, 1.0);

    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

pub fn write_json_atomic<P: AsRef<Path>>(path: P, stats: &SystemStats) -> std::io::Result<()> {
    let tmp_path = path.as_ref().with_extension("tmp");

    let json = serde_json::to_string(stats).unwrap();

    // Scrive su file temporaneo
    let mut tmp_file = fs::File::create(&tmp_path)?;
    tmp_file.write_all(json.as_bytes())?;
    tmp_file.flush()?; // Assicura che i dati siano effettivamente scritti

    // Rinomina atomica
    fs::rename(tmp_path, path)?;

    Ok(())
}