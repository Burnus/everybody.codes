use core::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError<'a> {
    InputMalformed(&'a str),
    InvalidOperation(char),
    MessageTooSmall,
    NonRectangular,
    StartMarkerCount,
    EndMarkerCount,
}

impl Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputMalformed(e) => write!(f, "Unable to parse malformed input: {e}\nShould be the key, followed by an empty line, and the encrypted message."),
            Self::InvalidOperation(e) => write!(f, "Unable to parse {e} into an operation"),
            Self::MessageTooSmall => write!(f, "Message must be at least 3*3 characters"),
            Self::NonRectangular => write!(f, "All lines of the message must be of equal length"),
            Self::StartMarkerCount => write!(f, "There must be exactly one \'>\' in the message"),
            Self::EndMarkerCount => write!(f, "There must be exactly one \'<\' in the message"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Operation{ Left, Right, }

struct Key {
    operations: Vec<Operation>
}

impl<'a> TryFrom<&'a str> for Key {
    type Error = ParseError<'a>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let operations = value.chars().map(|c| match c {
            'L' => Ok(Operation::Left),
            'R' => Ok(Operation::Right),
            e => Err(Self::Error::InvalidOperation(e)),
        }).collect::<Result<Vec<_>, _>>()?;
        Ok(Self { operations })
    }
}

struct Message {
    chars: Vec<Vec<char>>,
    width: usize,
    height: usize,
}

impl<'a> TryFrom<&'a str> for Message {
    type Error = ParseError<'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let chars: Vec<_> = value
            .lines()
            .map(|line| line.chars().collect::<Vec<_>>())
            .collect();

        let height = chars.len();
        if height < 3 {
            return Err(Self::Error::MessageTooSmall);
        }
        let width = chars[0].len();
        if width < 3 {
            return Err(Self::Error::MessageTooSmall);
        }
        if chars.iter().any(|row| row.len() != width) {
            return Err(Self::Error::NonRectangular);
        }
        if chars.iter().map(|row| row.iter().filter(|c| **c == '>').count()).sum::<usize>() != 1 {
            return Err(Self::Error::StartMarkerCount);
        }
        if chars.iter().map(|row| row.iter().filter(|c| **c == '<').count()).sum::<usize>() != 1 {
            return Err(Self::Error::EndMarkerCount);
        }

        Ok(Self { chars, width, height, })
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chars
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<String>()
            .split(&['>', '<'][..])
            .nth(1)
            .unwrap())
    }
}

impl Message {
    fn permutation_cycles(&self, key: &Key) -> Vec<Vec<(usize, usize)>> {
        let mut grid = (0..self.height).map(|y|
                (0..self.width).map(|x| (y, x)).collect::<Vec<_>>()
            ).collect::<Vec<_>>();
        (0..self.height-2).for_each(|y|
            (0..self.width-2).for_each(|x| {
                let step = y * (self.width-2) + x;
                rotate(&mut grid, x+1, y+1, key.operations[step % key.operations.len()]);
        }));
        let mut res: Vec<Vec<(usize, usize)>> = Vec::new();
        (0..self.height).for_each(|old_y|
            (0..self.width).for_each(|old_x| {
                let mut old = (old_y, old_x);
                loop {
                    let new = grid
                        .iter()
                        .enumerate()
                        .find_map(|(y, row)| row
                            .iter()
                            .enumerate()
                            .find(|(_x, old_entry)| **old_entry == old)
                            .map(|(x, _old_entry)| (y, x))
                        ).unwrap();
                    if let Some(idx) = res.iter().position(|seq| seq.contains(&old)) {
                        if !res[idx].contains(&new) {
                            // assert!(res[idx].iter().position(|o| *o == old) == Some(res[idx].len()-1));
                            res[idx].push(new);
                            old = new;
                        } else {
                            break;
                        }
                    } else {
                        res.push(Vec::from([old, new]));
                        old = new;
                    }
                }
            }));
        res
    }

    fn apply_permutation_cycles(&mut self, cycles: &[Vec<(usize, usize)>], count: usize) {
        let old = self.chars.clone();
        (0..self.height).for_each(|y|
            (0..self.width).for_each(|x| {
                let cycle = cycles.iter().find(|c| c.contains(&(y, x))).unwrap();
                let offset = cycle.iter().position(|ge| *ge == (y, x)).unwrap();
                let (new_y, new_x) = cycle[(count + offset) % cycle.len()];
                self.chars[new_y][new_x] = old[y][x];
        }));
    }
}

fn rotate(grid: &mut [Vec<(usize, usize)>], x: usize, y: usize, direction: Operation) {
    let temp = grid[y-1][x-1];
    match direction {
        Operation::Left => {
            grid[y-1][x-1] = grid[y-1][x];
            grid[y-1][x]   = grid[y-1][x+1];
            grid[y-1][x+1] = grid[y][x+1];
            grid[y][x+1] = grid[y+1][x+1];
            grid[y+1][x+1] = grid[y+1][x];
            grid[y+1][x] = grid[y+1][x-1];
            grid[y+1][x-1] = grid[y][x-1];
            grid[y][x-1] = temp;
        },
        Operation::Right => {
            grid[y-1][x-1] = grid[y][x-1];
            grid[y][x-1] = grid[y+1][x-1];
            grid[y+1][x-1] = grid[y+1][x];
            grid[y+1][x] = grid[y+1][x+1];
            grid[y+1][x+1] = grid[y][x+1];
            grid[y][x+1] = grid[y-1][x+1];
            grid[y-1][x+1] = grid[y-1][x];
            grid[y-1][x] = temp;
        },
    }
}

pub fn run(input: &str, part: usize) -> Result<String, ParseError> {
    if let Some((key, message)) = input.split_once("\n\n") {
        let key = Key::try_from(key)?;
        let mut message = Message::try_from(message)?;
        let cycles = message.permutation_cycles(&key);
        match part {
            1 => message.apply_permutation_cycles(&cycles, 1),
            2 => message.apply_permutation_cycles(&cycles, 100),
            3 => message.apply_permutation_cycles(&cycles, 1048576000),
            _ => panic!("Illegal part number"),
        }
        Ok(message.to_string())
    } else {
        Err(ParseError::InputMalformed(input))
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
        let expected = ["WIN", "VICTORY"];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1].to_string()));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = ["4877113951383767", "5529455775582299", "2423423664347316"];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1].to_string()));
        }
    }
}
