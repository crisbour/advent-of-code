use std::fs;
use itertools::Itertools;
use anyhow::Result;
use regex::Regex;

enum Cubes {
    Red   { cnt: u32 },
    Green { cnt: u32 },
    Blue  { cnt: u32 },
}

impl From<&str> for Cubes {
    fn from(s: &str) -> Cubes {
        let pair = s.trim().split(' ').collect_tuple();
        let (cnt, color) = match pair {
                 Some((cnt_str, color)) => (cnt_str.parse::<u32>().unwrap(), color),
                 _ => panic!("Expected <u32 color_str> format, but received {:?}", pair),
                };
        match color {
            "red"   => Cubes::Red{cnt},
            "green" => Cubes::Green{cnt},
            "blue"  => Cubes::Blue{cnt},
            _ => panic!("Unknown cube type")
        }
    }
}

impl Cubes {
    fn valid(&self) -> bool {
        match self {
            Cubes::Red{cnt}   => *cnt <= 12,
            Cubes::Green{cnt} => *cnt <= 13,
            Cubes::Blue{cnt}  => *cnt <= 14,
        }
    }
}

struct Round {
    cubes: Vec<Cubes>,
}

impl From<&str> for Round {
    fn from(s: &str) -> Round {
        let cubes = s.split(',')
                     .map(|cubes_str| Cubes::from(cubes_str))
                     .collect::<Vec<Cubes>>();
        Round::new(cubes)
    }
}

impl Round {
    fn new(cubes: Vec<Cubes>) -> Self {
        Self { cubes }
    }
    fn valid(&self) -> bool {
        self.cubes.iter()
                  .map(|c| c.valid())
                  .fold(true, |acc, x| acc && x)
    }
}

struct Game {
    id:    u32,
    set:   Vec<Round>,
}

impl Game {
    fn new(id: u32, set: Vec<Round>) -> Self {
        Self { id, set }
    }
    fn valid(&self) -> bool {
        self.set.iter()
                  .map(|r| r.valid())
                  .fold(true, |acc, x| acc && x)
    }
    fn power(&self) -> u32 {
        let mut red   = 0;
        let mut green = 0;
        let mut blue  = 0;

        for round in &self.set {
            for cube in &round.cubes {
                match cube {
                    Cubes::Red{cnt}   => if red   < *cnt { red   = *cnt; },
                    Cubes::Green{cnt} => if green < *cnt { green = *cnt; },
                    Cubes::Blue{cnt}  => if blue  < *cnt { blue  = *cnt; },
                };
            }
        }
        red * green * blue
    }
}

fn main() -> Result<()> {
    let input: String = fs::read_to_string("input")?.parse()?;
    let re = Regex::new(r"(?m)Game\s(\d+):(.+)$")?;
    let mut sum_id = 0;
    let mut sum_power = 0;
    let games = re.captures_iter(&input)
                      .map(|c| c.extract())
                      .map(|(_, [id_str, set_str])| {
                          let id = id_str.parse().unwrap();
                          let set = set_str.split(';').map(|round_str| Round::from(round_str)).collect();
                          Game::new( id, set )
                      });
    for game in games {
        if game.valid() {
            sum_id += game.id;
        }
        sum_power += game.power()
    }
    println!("Part 1: {}", sum_id);
    println!("Part 2: {}", sum_power);
    Ok(())
}
