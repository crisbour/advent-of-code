use std::fs;
use regex::Regex;
use anyhow::{anyhow, Result};
use std::collections::{VecDeque, BTreeSet, BTreeMap};
use log::{info, debug};
use env_logger;

fn str_list_parse(str_list: &str) -> Vec<u32> {
    str_list.trim().split_whitespace().map(|val_str| val_str.parse::<u32>().unwrap()).collect()
}

#[derive(Debug)]
struct Card {
    id: u32,
    winning: BTreeSet<u32>,
    own: Vec<u32>,
}

impl Card {
    fn new(id: u32, winning_vec: Vec<u32>, own: Vec<u32>) -> Self {
        let winning = BTreeSet::from_iter(winning_vec.iter().map(|x| *x));
        debug!("Create card: {}, {:?}, {:?}", id, winning, own);
        Self { id, winning, own }
    }
    fn get_prize(&self) -> u32 {
        let mut prize = 0;
        for number in self.own.iter() {
            if self.winning.contains(&number) {
                if prize == 0 {
                    prize = 1;
                } else {
                    prize = prize << 1;
                }
            }
        }
        debug!("Search {:?} in {:?}", self.own, self.winning);
        debug!("Prize for Card {}: {}", self.id, prize);
        prize
    }

    fn get_new_cards(&self) -> Vec<u32> {
        let new_cards_cnt = self.own.iter().fold(0, |acc, x| if self.winning.contains(&x) {acc + 1} else {acc});
        return ((self.id+1)..=(self.id + new_cards_cnt)).collect()
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let input: String = fs::read_to_string("input")?.parse()?;
    let re = Regex::new(r"(?m)Card\s+(\d+):\s+(.+)\s+\|\s+(.+)$")?;
    let cards: Vec<_> = re.captures_iter(&input)
                          .map(|c| c.extract())
                          .map(|(_, [id_str, winning_str, own_str])| {
                              let id: u32 = id_str.parse().unwrap();
                              let winning = str_list_parse(winning_str);
                              let own     = str_list_parse(own_str);
                              Card::new(id, winning, own)
                          })
                          .collect();

    let mut total_prize = 0;
    for card in cards.iter() {
        total_prize += card.get_prize();
    }

    let mut count = 0;
    let card_map = BTreeMap::from_iter(cards.iter().map(|card| (card.id, card)));
    let mut queue: BTreeMap<u32,usize> = cards.iter().map(|card| (card.id, 1 as usize)).collect();
    for card_id in cards.iter().map(|card| card.id) {
        let cnt = *queue.get(&card_id).ok_or(anyhow!("Did not find {}", card_id))?;
        count += cnt;
        let card = card_map.get(&card_id).unwrap();
        let new_cards = card.get_new_cards();
        debug!("{} scratchcards of {:?} producing: {:?}", cnt, card.id, new_cards);
        for new_card in new_cards {
            queue.entry(new_card).and_modify(|curr| *curr += cnt).or_insert(1);
        }
        queue.remove(&card_id);
    }
    println!("Part 1: {}", total_prize);
    println!("Part 2: {}", count);
    Ok(())
}
