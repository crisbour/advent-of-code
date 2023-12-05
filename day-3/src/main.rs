use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{anyhow, Result};
use std::iter::zip;

#[derive(PartialEq, Clone, Copy, Debug)]
enum Element {
    None,
    Number {val: u32, used: bool},
    Symbol {symbol: char},
}

#[derive(Debug)]
struct Matrix {
    pub elements: Vec<Vec<Rc<RefCell<Element>>>>,
    pub gear_ratio: u32,
}

impl Matrix {
    fn new() -> Self {
        Self { elements: Vec::new(), gear_ratio: 0 }
    }

    fn new_row(&mut self, _row: usize, column_size: usize) {
        self.elements.push(Vec::with_capacity(column_size));
    }

    fn new_element(&mut self, row: usize, _column: usize, element: &mut Rc<RefCell<Element>>) {
        self.elements[row].push(Rc::clone(&element));
    }

    fn valid_indices(&self, row: usize, col: usize, row_offset: isize, col_offset: isize) -> Option<(usize, usize)> {
        let row  = row as isize + row_offset;
        let col  = col as isize + col_offset;
        let rows = self.elements.len() as isize;
        let cols = self.elements[0].len() as isize;
        if row >= 0 && row < rows && col >= 0 && col < cols {
            Some((row as usize, col as usize))
        } else {
            None
        }
    }

    fn find_nearby_numbers(&mut self, row: usize, col: usize, element: Element) -> Vec<u32> {
        let steps = zip( vec![ -1, -1, -1, 0, 1, 1, 1, 0],
                         vec![ -1, 0, 1, 1, 1, 0, -1, -1]);
        let mut numbers = vec![];
        let mut count_gear = 0;
        if let Element::Symbol{symbol} = element {
            for (i, j) in steps {
                if let Some((k, l)) = self.valid_indices(row, col, i, j) {
                    let neighbour = *self.elements[k][l].borrow();
                    if let Element::Number{val, used} = neighbour {
                        if !used {
                            if symbol=='*' {
                                count_gear += 1;
                            }
                            *self.elements[k][l].borrow_mut() = Element::Number{val, used: true};
                            numbers.push(val);
                        }
                    };
                }
            }
        }
        if count_gear == 2 {
            let gear_ratio = numbers.iter().fold(1, |acc, x| acc*x);
            //println!("Ratio of {:?}, is {}", numbers, gear_ratio);
            self.gear_ratio += gear_ratio;
        }
        numbers
    }

    fn find_numbers(&mut self) -> Vec<u32> {
        let mut numbers = vec![];
        let elements: Vec<_> = self.elements
                        .iter()
                        .enumerate()
                        .map(|(i, row)| row.iter()
                                           .enumerate()
                                           .map(move |(j, element)| (i, j, Rc::clone(element)))
                        ).flatten().collect();
        for (i, j, element) in elements {
            self.find_nearby_numbers(i, j, *element.borrow())
                .iter()
                .for_each(|x| numbers.push(*x));
        }
        numbers
    }
}

fn main() -> Result<()> {
    let input: String = fs::read_to_string("input")?.parse()?;
    let lines: Vec<&str> = input.split('\n').collect();
    let mut matrix = Matrix::new();
    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            break;
        }
        matrix.new_row(i, line.len());
        let line_itr = line.chars().enumerate();
        let mut element = Rc::new(RefCell::new(Element::None));
        for (j, c) in line_itr {
            if let Some(digit) = c.to_digit(10) {
                let inner_element = *element.borrow();
                if let Element::Number{val, used} = inner_element {
                    let new_val = val * 10 + digit;
                    *element.borrow_mut() = Element::Number{ val: new_val, used};
                } else {
                    element = Rc::new(RefCell::new(Element::Number{val: digit, used: false }));
                }
            } else {
                element = if c=='.' {
                    Rc::new(RefCell::new(Element::None))
                } else {
                    Rc::new(RefCell::new(Element::Symbol{symbol: c}))
                };
            }
            matrix.new_element(i, j, &mut element);
        }
    }

    let sum: u32 = matrix.find_numbers().iter().sum();
    println!("Part 1: {}", sum);
    println!("Part 2: {}", matrix.gear_ratio);
    Ok(())
}
