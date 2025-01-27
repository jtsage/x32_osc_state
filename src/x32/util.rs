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


