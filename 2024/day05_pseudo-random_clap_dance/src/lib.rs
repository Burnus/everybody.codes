use core::fmt::{Display, Write};
use std::collections::{BTreeMap, HashMap, VecDeque};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    GridMalformed(usize, usize, usize),
    ParseIntError(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GridMalformed(first, idx, len) => write!(f, "Grid is not rectangular: First line has {first} items, but line {idx} has {len}."),
            Self::ParseIntError(e) => write!(f, "Unable to parse into a number: {e}"),
        }
    }
}

struct Dancers {
    dancers: Vec<VecDeque<usize>>,
    columns: usize,
    round: usize,
}

impl TryFrom<&str> for Dancers {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let columns = value.lines().next().unwrap_or_default().split_whitespace().count();
        let mut dancers = vec![VecDeque::new(); columns];
        for (y, l) in value.lines().enumerate() {
            let numbers: Vec<_> = l.split_whitespace().map(|c| c.parse::<usize>().map_err(|_| ParseError::ParseIntError(c.to_string()))).collect::<Result<Vec<_>, _>>()?;
            if numbers.len() != columns {
                return Err(Self::Error::GridMalformed(columns, y, numbers.len()));
            }
            numbers.iter().enumerate().for_each(|(x, n)| dancers[x].push_back(*n));
        }
        Ok(Self { dancers, columns, round: 0 })
    }
}

impl Dancers {
    fn dance(&mut self) -> usize {
        let clapper = self.dancers[self.round % self.columns].pop_front().unwrap();
        self.round += 1;
        let column = &mut self.dancers[self.round % self.columns];
        let direction = (clapper-1) / column.len();
        let residual = clapper % column.len();
        if direction & 1 == 0 {
            // left side
            if residual == 0 {
                column.insert(column.len()-1, clapper);
            } else {
                column.insert(residual-1, clapper);
            }
        } else {
            //right side
            if residual == 0 {
                column.insert(1, clapper)
            } else {
                column.insert(column.len()-residual+1, clapper)
            }
        }
        self.dancers.iter().fold(String::new(), |mut output, c| {
            let _ = write!(output, "{}", c.front().unwrap());
            output
        }).parse().unwrap()
    }
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseError> {
    let mut dancers = Dancers::try_from(input)?;
    match part {
        1 => {
            for _ in 0..9 { dancers.dance(); }
            Ok(dancers.dance())
        },
        2 => {
            // There are probably loops in the results to be exploited here, but I don't see how
            // to spot them algorithmically and saving the entire state after each dance, just
            // to look for repetitions, seems excessive.
            let mut numbers = HashMap::new();
            loop {
                let this = dancers.dance();
                let repetitions: usize = *numbers.get(&this).unwrap_or(&0);
                if repetitions == 2023 {
                    return Ok(this * dancers.round);
                } else {
                    numbers.insert(this, repetitions+1);
                }
            }
        },
        3 => {
            // Admittedly, there is no proof that no more higher numbers will appear later. 
            // But I simply hope that at the point at which a number repeats itself for the 
            // 10th time, no more new ones will appear.
            let mut numbers = BTreeMap::new();
            loop {
                let this = dancers.dance();
                let repetitions: usize = *numbers.get(&this).unwrap_or(&0);
                if repetitions == 9 {
                    return Ok(numbers.pop_last().unwrap().0);
                } else {
                    numbers.insert(this, repetitions+1);
                }
            }

        },
        _ => panic!("Illegal part number"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;

    fn read_file(name: &str) -> String {
        read_to_string(name).expect(&format!("Unable to read file: {name}")[..]).trim().to_string()
    }

    #[test]
    fn test_sample() {
        let expected = [2323, 50877075, 6584];
        for part in 1..=3 {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [2232, 14029502980017, 8265100210021008];
        for part in 1..=3 {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
