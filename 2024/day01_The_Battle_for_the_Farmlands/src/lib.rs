use core::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    ParseCharError(char)
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseCharError(e) => write!(f, "Unable to parse into creature: {e}"),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Creature{ Ant, Beetle, Cockroach, Dragonfly, None }

impl TryFrom<char> for Creature {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' => Ok(Self::Ant),
            'B' => Ok(Self::Beetle),
            'C' => Ok(Self::Cockroach),
            'D' => Ok(Self::Dragonfly),
            'x' => Ok(Self::None),
            e => Err(Self::Error::ParseCharError(e)),
        }
    }
}

impl Creature {
    fn required_potions(&self) -> usize {
        match self {
            Self::Ant | Self::None => 0,
            Self::Beetle => 1,
            Self::Cockroach => 3,
            Self::Dragonfly => 5,
        }
    }
}

fn required_potions_for_group(pair: &[Creature]) -> usize {
    pair.iter().map(|c| c.required_potions()).sum::<usize>() +
        match pair.iter().filter(|c| c != &&Creature::None).count() {
            3 => 6,
            2 => 2,
            _ => 0,
        }
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseError> {
    let items: Vec<_> = input.lines().next().unwrap_or_default().chars().map(Creature::try_from).collect::<Result<Vec<_>, _>>()?;
    match part {
        1 => Ok(items.iter().map(|c| c.required_potions()).sum()),
        2 => Ok(items.chunks(2).map(required_potions_for_group).sum()),
        3 => Ok(items.chunks(3).map(required_potions_for_group).sum()),
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
        let expected = [5, 28, 30];
        for part in 1..=3 {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [1310, 5582, 27825];
        for part in 1..=3 {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
