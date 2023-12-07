use log::debug;
use env_logger;
use std::fs;
use anyhow::{anyhow, Result};
use itertools::Itertools;

fn field_vec(vec_str: &str) -> Vec<u64> {
    vec_str.split(':')
           .skip(1)
           .next()
           .unwrap()
           .trim()
           .split_whitespace()
           .map(|x| x.parse().unwrap())
           .collect()
}

fn field_concat(vec_str: &str) -> u64 {
    let field_str = vec_str.split(':')
                          .skip(1)
                          .next()
                          .unwrap()
                          .trim()
                          .split_whitespace()
                          .fold(String::new(), |acc, x| acc+x);
    debug!("Field: {}", field_str);
    field_str.parse().unwrap()
}

fn ways_to_win(time: u64, dist: u64) -> u64 {
        let delta            = time*time - 4*dist;
        let delta_sqrt       = (delta as f64).sqrt() as u64;
        let exact            = delta == delta_sqrt*delta_sqrt;
        let root_left_float  = ((time as f64) - (delta as f64).sqrt())/2.0;
        let root_right_float = ((time as f64) + (delta as f64).sqrt())/2.0;
        let root_left        = root_left_float.ceil() as u64;
        let root_right       = root_right_float.floor() as u64;
        let mut size = root_right - root_left + 1;
        if exact {
            size = size - 2;
        }
        size
}

fn main() -> Result<()> {
    env_logger::init();
    let input: String = fs::read_to_string("input")?.parse()?;
    let args: Vec<String> = std::env::args().collect();
    let part: u32 = args[1].parse()?;
    let lines: Vec<&str> = input.split('\n').filter(|line| !line.is_empty()).take(2).collect();
    let mut result = 1;

    match part {
        1 => {
            let times = field_vec(lines[0]);
            let dists = field_vec(lines[1]);
            for (time, dist) in times.iter().zip(dists.iter()) {
                result *= ways_to_win(*time, *dist);
            }
        },
        2 => {
            let time = field_concat(lines[0]);
            let dist = field_concat(lines[1]);
            result = ways_to_win(time, dist);

        },
        _ => panic!("Part {} no supported", part),
    };

    println!("Part {}: {}", part, result);

    Ok(())
}
