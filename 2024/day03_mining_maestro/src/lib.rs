use core::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    GridMalformed(usize, usize, usize),
    InvalidTile(char),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GridMalformed(first, idx, len) => write!(f, "Input Grid is not rectangular. First line has {first} characters, but line {idx} has {len}."),
            Self::InvalidTile(e) => write!(f, "Unable to parse into a tile: {e}"),
        }
    }
}

struct Map {
    map: Vec<Vec<usize>>,
    height: usize,
    width: usize,
}

impl TryFrom<&str> for Map {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lines: Vec<_> = value.lines().collect();
        if lines.is_empty() {
            return Ok(Self { map: vec![vec![]], height: 0, width: 0, });
        }
        let width = lines[0].len();
        let mut map = vec![vec![0; width]; lines.len()];
        for (y, l) in lines.iter().enumerate() {
            if l.len() != width {
                return Err(Self::Error::GridMalformed(width, y, l.len()));
            }
            for (x, c) in l.chars().enumerate() {
                match c {
                    '.' => (),
                    '#' => map[y][x] = 1,
                    e => return Err(Self::Error::InvalidTile(e)),
                }
            }
        }
        Ok(Self { map, height: lines.len(), width })
    }
}

impl Map{
    fn at_or_zero(&self, (x, y): (usize, usize)) -> usize {
        if x >= self.width || y >= self.height { 0 } else { self.map[y][x] }
    }

    fn neighbours(&self, (x, y): (usize, usize), royal: bool) -> Vec<usize> {
        let mut res = if royal {
            vec![
                self.at_or_zero((x.wrapping_sub(1), y)),
                self.at_or_zero((x+1, y)),
                self.at_or_zero((x, y.wrapping_sub(1))),
                self.at_or_zero((x, y+1)),
                self.at_or_zero((x.wrapping_sub(1), y.wrapping_sub(1))),
                self.at_or_zero((x+1, y.wrapping_sub(1))),
                self.at_or_zero((x.wrapping_sub(1), y+1)),
                self.at_or_zero((x+1, y+1)),
            ]
        } else {
            vec![
                self.at_or_zero((x.wrapping_sub(1), y)),
                self.at_or_zero((x+1, y)),
                self.at_or_zero((x, y.wrapping_sub(1))),
                self.at_or_zero((x, y+1)),
            ]
        };
        res.sort();
        res
    }

    fn maximize(&mut self, royal: bool) {
        let (x_min, y_min, x_max, y_max) = if royal {
            (0, 0, self.width, self.height)
        } else {
            (1, 1, self.width-1, self.height-1)
        };
        loop {
            let mut changed = false;
            (x_min..x_max).for_each(|x|
                (y_min..y_max).for_each(|y| {
                    let curr = self.map[y][x];
                    let lowest = *self.neighbours((x, y), royal).first().unwrap_or(&0);
                    if curr > 0 && lowest >= curr {
                        self.map[y][x] = lowest + 1;
                        changed = true;
                    }
                }));
            if !changed {
                return;
            }
        }
    }

    fn total_sum(&self) -> usize {
        self.map.iter().map(|line| line.iter().sum::<usize>()).sum()
    }
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseError> {
    let mut map = Map::try_from(input)?;
    match part {
        1 | 2 => map.maximize(false),
        3 => map.maximize(true),
        _ => panic!("Illegal part number"),
    };
    Ok(map.total_sum())
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
        let expected = [35, 35, 29];
        for part in 1..=3 {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [127, 2674, 10571];
        for part in 1..=3 {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
