#![allow(dead_code)]
use rand::distributions::{Distribution, Uniform};
use rand::{distributions::Alphanumeric, Rng};

pub fn rnd_buffer(length : usize) -> Vec<u8> {
    rnd_buff(length, false)
}

pub fn rnd_string_buffer(length : usize) -> Vec<u8> {
    rnd_buff(length, true)
}

fn rnd_buff(length: usize, string : bool) -> Vec<u8> {
    let mut buffer:Vec<u8> = vec![];

    let between = Uniform::from(32..=126);
    let mut rng = rand::thread_rng();

    for _ in 0..length {
        buffer.push(between.sample(&mut rng));
    }

    if string {
        buffer.pop();
        buffer.push(0_u8);
    }

    buffer

}


pub fn random_data() -> (f32, bool, String) {
    let mut rng = rand::thread_rng();
    let length_usize = rng.gen_range(1..=12);

    let level:f32 = rng.gen_range(0.0..1.0);

    (
        level,
        rng.gen_bool(2.0 / 3.0),
        rng.sample_iter(&Alphanumeric)
            .take(length_usize)
            .map(char::from)
            .collect(),
    )
}

pub fn random_data_node() -> (f32, bool, String) {
    let mut rng = rand::thread_rng();
    let length_usize = rng.gen_range(1..=12);

    let level:f32 = rng.gen_range(-90.0..10.0);

    (
        (level * 10.0).round() / 10.0,
        rng.gen_bool(2.0 / 3.0),
        rng.sample_iter(&Alphanumeric)
            .take(length_usize)
            .map(char::from)
            .collect(),
    )
}