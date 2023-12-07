use std::fs;
use anyhow::{anyhow, Result};
use std::cmp::Ordering;
use itertools::Itertools;
use env_logger;
use std::io::Write;
use log::debug;

#[derive(Debug, Clone, Copy)]
struct CardType {
    card: char,
}

impl CardType {
    fn into(&self, joker: bool) -> u32 {
        if self.card.is_ascii_digit() {
            self.card.to_digit(10).unwrap()
        } else {
            match self.card {
                'T' => 10,
                'J' => if joker { 1 } else { 11 },
                'Q' => 12,
                'K' => 13,
                'A' => 14,
                _ => panic!("Unrecognized card type: {}", self.card),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandType {
    Five,
    Four,
    FullHouse,
    Three,
    TwoPairs,
    OnePair,
    HighCard,
}

impl HandType {
    fn new(cards: &Vec<CardType>, joker: bool) -> Self {
        let mut cards_sorted: Vec<_> = cards.clone();
        cards_sorted.sort_by(|a, b| a.into(joker).cmp(&b.into(joker)));
        cards_sorted.reverse();
        let mut hand: HandType = HandType::HighCard;
        let mut hands: Vec<HandType> = Vec::new();
        let mut jokers: usize = 4;
        for (prev, ch) in cards_sorted.iter().tuple_windows() {
            if joker && (prev.card == 'J' || ch.card=='J') {
                break;
            } else {
                jokers -= 1;
            }
            if prev.card == ch.card {
                let new_hand = match hand {
                    HandType::HighCard => HandType::OnePair,
                    HandType::OnePair => HandType::Three,
                    HandType::Three => HandType::Four,
                    HandType::Four => HandType::Five,
                    _ => panic!("Only checking repetitions in this run"),
                };
                debug!("Updated hand: {:?}", new_hand);
                hand = new_hand;
            } else {
                debug!("HandType found: {:?}", hand);
                hands.push(hand);
                hand = HandType::HighCard;
            }
        }
        hands.push(hand);
        debug!("Hand: {:?}", hands);
        hands.sort();
        hands.reverse();
        debug!("Sorted Hand: {:?}", hands);
        if joker {
            for _ in 0..jokers {
                let new_hand = match hands[0] {
                    HandType::HighCard => HandType::OnePair,
                    HandType::OnePair => HandType::Three,
                    HandType::Three => HandType::Four,
                    HandType::Four => HandType::Five,
                    _ => panic!("Only checking repetitions in this run"),
                };
                hands[0] = new_hand;
            }
            debug!("Updated hand with {} jokers: {:?}", jokers, hands);
        }
        match hands[0] {
            HandType::Three   => {
                debug!("Next hand type: {:?}", hands[1]);
                if hands[1] == HandType::OnePair {
                    debug!("Update hand {:?} with {:?} -> FullHouse", hands[0], hands[1]);
                    HandType::FullHouse
                } else {
                    debug!("Hand: {:?}", hands[0]);
                    hands[0]
                }
            },
            HandType::OnePair => {
                if hands[1] == HandType::OnePair {
                    debug!("Update hand {:?} with {:?} -> TwoPairs", hands[0], hands[1]);
                    HandType::TwoPairs
                } else {
                    hands[0]
                }
            },
            _ => {
                debug!("Hand: {:?}", hands[0]);
                hands[0]
            },
        }
    }
}

impl Into<u32> for HandType {
    fn into(self) -> u32 {
        match self {
            HandType::Five      => 6,
            HandType::Four      => 5,
            HandType::FullHouse => 4,
            HandType::Three     => 3,
            HandType::TwoPairs  => 2,
            HandType::OnePair   => 1,
            HandType::HighCard  => 0,
        }
    }
}

impl Ord for HandType {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_strength:  u32 = self.clone().into();
        let other_strength: u32 = other.clone().into();
        self_strength.cmp(&other_strength)
    }
}

impl PartialOrd for HandType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
struct Hand {
    hand: String,
    hand_type: HandType,
    strength: u32,
    bid: u32,
}

impl Hand {
    fn new(hand_str: &str, bid_str: &str, joker: bool) -> Self {
        let cards = Hand::parse_hand(hand_str);
        let hand_type = HandType::new(&cards, joker);
        let bid = bid_str.parse().unwrap();
        let strength = cards.iter().fold(0, |acc, x| acc*15 + x.into(joker));
        Hand {
            hand: hand_str.to_string(),
            hand_type,
            strength,
            bid,
        }
    }

    fn parse_hand(hand_str: &str) -> Vec<CardType> {
        hand_str.chars()
                .map(|card| CardType{card})
                .collect::<Vec<_>>()
    }
}


impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.hand_type == other.hand_type && self.strength == other.strength
    }
}
impl Eq for Hand {}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.hand_type.cmp(&other.hand_type);
        match ord
        {
            Ordering::Equal => self.strength.cmp(&other.strength),
            _ => ord,
        }
    }
}

impl PartialOrd for Hand{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


fn main() -> Result<()> {
    env_logger::builder().format_timestamp(None).init();
    let args: Vec<String> = std::env::args().collect();
    let part: u32 = args[1].parse()?;
    let input: String = fs::read_to_string("input")?.parse()?;
    let lines = input.split('\n').filter(|line| !line.is_empty());
    let mut hands: Vec<_> = lines.map(|line| match line.split_whitespace().collect_tuple() {
            Some((hand, bid)) => Hand::new(hand, bid, part==2),
            _ => panic!("Unsupported format"),
        }).collect();
    hands.sort();
    let result = hands.iter().enumerate().map(|(rank, hand)| {
        debug!("Hand rank {}: {:?}", rank+1, hand);
        (rank as u64+1)*(hand.bid as u64)
    }).fold(0, |acc, x| acc+x);
    println!("Part 1:{}", result);
    Ok(())
}
