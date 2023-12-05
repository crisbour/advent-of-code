use std::fs;
use anyhow::{anyhow, Result};

fn main() -> Result<()> {
    let input: String = fs::read_to_string("input")?.parse()?;
    let lines: Vec<&str> = input.split('\n').collect();
    let mut sum = 0;
    for line in lines {
        if line.is_empty() {
            break;
        }
        let first_idx = line.find(|c: char| c.is_ascii_digit())
                        .ok_or(anyhow!("Couldn't find digit"))?;
        let last_idx = line.rfind(|c: char| c.is_ascii_digit())
                       .ok_or(anyhow!("Couldn't find digit"))?;
        let first = line.chars().nth(first_idx).unwrap().to_digit(10).unwrap();
        let last = line.chars().nth(last_idx).unwrap().to_digit(10).unwrap();
        let number = first * 10 + last;
        sum += number;
    }
    println!("{}", sum);
    Ok(())
}
