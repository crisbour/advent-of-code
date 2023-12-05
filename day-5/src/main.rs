use std::fs;
use regex::Regex;
use env_logger;
use log::{info, debug};
use anyhow::Result;
use itertools::Itertools;
use std::ops::Range;
use std::fmt::Debug;
use std::cmp::Ordering;
use std::ops::Sub;
use std::mem;


#[derive(Debug, Clone, Ord, Copy)]
struct IdRange {
    start: u64,
    len: u64,
}

impl PartialEq for IdRange {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
    }
}

impl Eq for IdRange {}

impl PartialOrd for IdRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.start.cmp(&other.start))
    }
}

impl IdRange {
    fn new(start: u64, len: u64) -> Option<Self> {
        if len==0 {
            None
        } else {
            Some(Self { start, len})
        }
    }
    fn intersect(&self, other: &Self) -> Option<Self> {
        let start = std::cmp::max(self.start, other.start);
        let end = std::cmp::min(self.start + self.len, other.start + other.len);
        if start < end {
            let len = end - start;
            let intersect_range = IdRange{start, len};
            debug!("Intersection of {:?} and {:?} = {:?}", self, other, intersect_range);
            Some(intersect_range)
        } else {
            None
        }
    }
    fn sub(&self, other: &Self) -> Set {
        if let Some(to_sub) = self.intersect(other){
            let left = IdRange::new(self.start, to_sub.start.saturating_sub(self.start));
            let right = IdRange::new(to_sub.start+to_sub.len, (self.start + self.len).saturating_sub(to_sub.start + to_sub.len));
            debug!("Remained ranges: {:?}, {:?}", left, right);
            let new_set = if left.is_some() && right.is_some() {
                let left_range = left.unwrap();
                let right_range = right.unwrap();
                Set::Split(Box::new(Set::Interval(left_range)), Box::new(Set::Interval(right_range)))
            } else if let Some(left_range) = left {
                Set::Interval(left_range)
            } else if let Some(right_range) = right {
                Set::Interval(right_range)
            } else {
                Set::None
            };
            debug!("Subtracting intervals: {:?}\\{:?}={:?}", self, other, new_set);
            new_set
        } else {
            Set::Interval(*self)
        }

    }
}

#[derive(Clone, Debug)]
enum Set {
    None,
    Interval(IdRange),
    Split(Box<Set>, Box<Set>),
}

impl Set {
    fn new(init: IdRange) -> Self {
        Set::Interval(init)
    }
    fn sub(&mut self, other: IdRange) {
        let new_set = match self {
            Set::Interval(id) => id.sub(&other),
            Set::Split(left, right) => {
                left.sub(other);
                right.sub(other);
                Set::Split(left.clone(), right.clone())
            },
            Set::None => Set::None,
        };
        mem::replace(self, new_set);
    }
    fn yield_intervals(&self) -> Box<dyn Iterator<Item = IdRange>> {
        match self {
            Set::Interval(id) => Box::new(std::iter::once(*id)),
            Set::Split(left, right) => Box::new(left.yield_intervals().chain(right.yield_intervals())),
            Set::None => Box::new(std::iter::empty()),
        }
    }
}

#[derive(Debug)]
struct Seed {
    seed:       IdRange,
    soil:       Vec<IdRange>,
    fertilizer: Vec<IdRange>,
    water:      Vec<IdRange>,
    light:      Vec<IdRange>,
    temp:       Vec<IdRange>,
    humidity:   Vec<IdRange>,
    location:   Vec<IdRange>,
}

#[derive(Debug, Clone)]
struct Map {
    src: u64,
    dst: u64,
    len: u64,
}

impl Map {
    fn contains(&self, src: IdRange) -> Option<(IdRange, IdRange)> {
        debug!("Check mapping: {:?} in {:?}", src, self);
        let src_range = IdRange{start: self.src, len: self.len};
        if let Some(intersect) = src_range.intersect(&src) {
            let offset = intersect.start - self.src;
            let start = self.dst + offset;
            let len = intersect.len;
            let dst_range = IdRange{start, len};
            debug!("Found mapping: {:?} -> {:?}", intersect, dst_range);
            Some((intersect, dst_range))
        } else {
            None
        }
    }
}


impl Seed {
    fn new(start: u64, len: u64) -> Self {
        Self {
            seed: IdRange{start, len},
            soil:       Vec::new(), // don't care at init
            fertilizer: Vec::new(), // don't care at init
            water:      Vec::new(), // don't care at init
            light:      Vec::new(), // don't care at init
            temp:       Vec::new(), // don't care at init
            humidity:   Vec::new(), // don't care at init
            location:   Vec::new(), // don't care at init
        }
    }
}

fn parse_vec<'a>(seeds_str: &'a str) -> impl Iterator<Item = u64> + 'a {
    seeds_str.trim().split_whitespace().map(|x| x.parse::<u64>().unwrap())
}

fn iter_map(map_ranges: &str) -> Vec<Map> {
    map_ranges.split('\n')
        .map(|map_range| match map_range.trim().split_whitespace().collect_tuple() {
             Some((dst, src, len)) => (src.parse::<u64>().unwrap(), dst.parse::<u64>().unwrap(), len.parse::<u64>().unwrap()),
             _ => panic!("Unsuported map syntax: {}", map_range),
             })
        .map(|(src, dst, len)| Map{ src, dst, len})
        .collect()
}

fn iter_dst(map_iter: &Vec<Map>, src: IdRange) -> Vec<IdRange> {
    debug!("Find dsts for {:?} in {:?}", src, map_iter);
    let mut set = Box::new(Set::new(src));
    let mut dsts: Vec<IdRange> = Vec::new();
    map_iter.iter()
            .filter_map(|map| map.contains(src))
            .for_each(|(src, dst)| {
                set.sub(src);
                debug!("Remained src_set = {:?}", set);
                dsts.push(dst)
            });

    let unmapped = set.yield_intervals().collect();
    debug!("Append unmapped intervals from {:?}: {:?}", set, unmapped);
    push_iter(&mut dsts, unmapped);

    if dsts.is_empty() {
        vec![src]
    } else {
        dsts
    }
}

fn vec_iter_dst(map_vec: Vec<Map>, srcs: &Vec<IdRange>) -> Vec<IdRange> {
    let mut new_dsts = Vec::new();
    for src in srcs {
        push_iter(&mut new_dsts, iter_dst(&map_vec, *src));
    }
    new_dsts
}

fn push_iter<T>(vec: &mut Vec<T>, concat_vec: Vec<T>) {
    for elem in concat_vec {
        vec.push(elem);
    }
}


fn main() -> Result<()> {
    env_logger::init();
    let input: String = fs::read_to_string("input")?.parse()?;
    let args: Vec<String> = std::env::args().collect();
    let part: u32 = args[1].parse()?;
    let blocks = input.split("\n\n");
    let mut seeds: Vec<Seed> = Vec::new();
    blocks.map(|block| match block.split(':').collect_tuple() {

          Some((name, map)) => (name, map.trim()),
          _ => panic!("Unsuported map format: {}", block),
    })
          .for_each(|(name, map_str)| {
              match name {
                "seeds"                   => {
                    match part {
                        1 => seeds = Vec::from_iter(parse_vec(map_str).map(|seed_id| Seed::new(seed_id, 1))),
                        2 => {
                            for (start, len) in parse_vec(map_str).tuples() {
                                seeds.push(Seed::new(start, len));
                            }
                        },
                        _ => panic!("Part {} not defined", part),
                    }
                }
                "seed-to-soil map"            => seeds.iter_mut().for_each(|seed| push_iter(&mut seed.soil,       iter_dst(&iter_map(map_str), seed.seed))),
                "soil-to-fertilizer map"      => seeds.iter_mut().for_each(|seed| push_iter(&mut seed.fertilizer, vec_iter_dst(iter_map(map_str), &seed.soil))),
                "fertilizer-to-water map"     => seeds.iter_mut().for_each(|seed| push_iter(&mut seed.water,      vec_iter_dst(iter_map(map_str), &seed.fertilizer))),
                "water-to-light map"          => seeds.iter_mut().for_each(|seed| push_iter(&mut seed.light,      vec_iter_dst(iter_map(map_str), &seed.water))),
                "light-to-temperature map"    => seeds.iter_mut().for_each(|seed| push_iter(&mut seed.temp,       vec_iter_dst(iter_map(map_str), &seed.light))),
                "temperature-to-humidity map" => seeds.iter_mut().for_each(|seed| push_iter(&mut seed.humidity,   vec_iter_dst(iter_map(map_str), &seed.temp))),
                "humidity-to-location map"    => seeds.iter_mut().for_each(|seed| push_iter(&mut seed.location,   vec_iter_dst(iter_map(map_str), &seed.humidity))),
                _ => panic!("Unsuported map name: {}", name),
              }
          });

    info!("Part1: seeds={:?}", seeds);
    let mut locations: Vec<IdRange> = seeds.iter().flat_map(|seed| seed.location.iter()).cloned().collect();
    locations.sort();
    println!("Part1: location={}", locations[0].start);

    Ok(())
}
