#![allow(dead_code)]
use crate::osc::{Message, Buffer};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref LVL_STRING: Regex = Regex::new(r"^(?<level>[+\-0-9.]+)").expect("unable to compile pattern");
    static ref NODE_STRING: Regex = Regex::new(r#"[^\s"]+|"([^"]*)""#).expect("unable to compile pattern");
}

/// Split an node message string argument into it's parts
#[must_use]
pub fn split_node_msg(s : &str) -> (String, Vec<String>) {
    let mut address = String::new();
    let mut args:Vec<String> = vec![];

    for (i, cap) in NODE_STRING.captures_iter(s).enumerate() {
        if let Some(v) = cap.get(1) {
            args.push(v.as_str().to_owned());
        } else if let Some(v) = cap.get(0) {
            if i == 0 {
                v.as_str().clone_into(&mut address);
            } else {
                args.push(v.as_str().to_owned());
            }
        }
    }
    (address, args)
}

/// Split an address, or address-like string into 4-String tuple
#[must_use]
pub fn split_address(s : &str) -> (String, String, String, String) {
    let mut s = s.to_owned();
    let s = if s.starts_with('/') { s.split_off(1)} else { s };

    let mut sp = s.split('/');
    (
        sp.next().unwrap_or("").to_owned(),
        sp.next().unwrap_or("").to_owned(),
        sp.next().unwrap_or("").to_owned(),
        sp.next().unwrap_or("").to_owned(),
    )
}

/// Get new buffer of a "/node" message
#[must_use]
pub fn new_node_buffer(s : String) -> Buffer {
    Message::new("/node").add_item(s).clone().try_into().unwrap_or_default()
}

/// Convert fader number (String) to proper index (zero-based)
#[must_use]
pub fn fader_num_to_idx(v: &str) -> usize {
    match v {
        "st" => 0,
        "m" => 1,
        _ => v.parse::<usize>().unwrap_or(1) - 1
    }
}

/// Get is on property from ON/OFF
#[must_use]
pub fn is_on_from_string(v : &str) -> bool { v == "ON" }

/// Get string level from float
#[must_use]
pub fn level_to_string(v : f32) -> String {
    let c_value = match v {
        d if d >= 0.5 => v * 40_f32 - 30_f32,
        d if d >= 0.25 => v * 80_f32 - 50_f32,
        d if d >= 0.0625 => v * 160_f32 - 70_f32,
        _ => v * 480_f32 - 90_f32
    };

    match c_value {
        d if (-0.05..=0.05).contains(&d)  => String::from("+0.0 dB"),
        d if d <= -89.9 => String::from("-oo dB"),
        d if d < 0_f32   => format!("{c_value:.1} dB"),
        _ => format!("+{c_value:.1} dB")
    }
}

/// get level as float from String
#[must_use]
pub fn level_from_string(input : &str) -> f32 {
    if input.starts_with("-oo") {
        0_f32
    } else if let Some(caps) = LVL_STRING.captures(input) {
        let lvl = match caps["level"].parse::<f32>() {
            Ok(d) if d < -60.0_f32 => (d + 90.0_f32) / 480.0_f32,
            Ok(d) if d < -30.0_f32 => (d + 70.0_f32) / 160.0_f32,
            Ok(d) if d < -10.0_f32 => (d + 50.0_f32) / 80.0_f32,
            Ok(d) => (d + 30.0_f32) / 40.0_f32,
            Err(_) => 0_f32
        };
        let f_lvl = (lvl * 1023.5).trunc() / 1023.0;
        (f_lvl * 10000.0).round() / 10000.0
    } else {
        0_f32
    }
}