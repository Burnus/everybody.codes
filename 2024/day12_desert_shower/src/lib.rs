use core::fmt::Display;
use std::num::ParseIntError;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError<'a> {
    LineMalformed(&'a str),
    ParseCharError(char),
    ParseIntError(ParseIntError),
}

impl Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LineMalformed(e) => write!(f, "Unable to parse line: {e}. It should be formatted like \"23 42\""),
            Self::ParseCharError(e) => write!(f, "Unable to parse item: {e}"),
            Self::ParseIntError(e) => write!(f, "Error while trying to parse an integer: {e:?}"),
        }
    }
}

impl From<ParseIntError> for ParseError<'_> {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase{ Ascend, Glide, Descend, }

#[derive(PartialEq, Eq, Clone, Copy)]
struct Coordinates{
    x: usize, 
    y: usize,
}

impl<'a> TryFrom<&'a str> for Coordinates {
    type Error = ParseError<'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Some((x, y)) = value.split_once(' ') {
            Ok(Self { x: x.parse()?, y: y.parse()? })
        } else {
            Err(Self::Error::LineMalformed(value))
        }
    }
}

struct Catapult {
    coordinates: Coordinates,
    segment_number: usize,
}

impl Catapult {
    fn can_hit(&self, target: Coordinates) -> Option<Phase> {
        if target.x <= self.coordinates.x {
            None
        } else if target.y <= self.coordinates.y {
            if (target.x + target.y - (self.coordinates.x + self.coordinates.y)) % 3 == 0 {
                Some(Phase::Descend)
            } else {
                None
            }
        } else {
            match (target.x - self.coordinates.x) / (target.y - self.coordinates.y) {
                0 => if target.y - self.coordinates.y == target.x - self.coordinates.x {
                        Some(Phase::Ascend)
                    } else {
                        None
                    },
                1 => Some(Phase::Glide),
                _ => if (target.x + target.y - (self.coordinates.x + self.coordinates.y)) % 3 == 0 {
                        Some(Phase::Descend)
                    } else {
                        None
                    },
            }
        }
    }

    fn power_to_hit(&self, target: Coordinates) -> Option<usize> {
        match self.can_hit(target) {
            Some(Phase::Ascend) | Some(Phase::Glide) => Some(target.y - self.coordinates.y),
            Some(Phase::Descend) => Some((target.x + target.y - (self.coordinates.x + self.coordinates.y)) / 3),
            None => None,
        }
    }
}

fn try_parse(input: &str) -> Result<(Vec<Catapult>, Vec<Coordinates>), ParseError> {
    let lines: Vec<_> = input.lines().collect();
    let height = lines.len()-1;
    let mut catapults = Vec::new();
    let mut targets = Vec::new();

    for (line_num, line) in lines.iter().enumerate() {
        let y = height - line_num;
        for (x, c) in line.chars().enumerate() {
            match c {
                '.' | '=' => (),
                'T' => targets.push(Coordinates { x, y }),
                'H' => targets.append(&mut vec![Coordinates { x, y }; 2]),
                c if ['A', 'B', 'C'].contains(&c) => catapults.push(
                    Catapult { coordinates: Coordinates { x, y }, segment_number: c as usize - b'@' as usize }),
                e => return Err(ParseError::ParseCharError(e)),
            }
        }
    }
    Ok((catapults, targets))
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseError> {
    match part {
        1 | 2 => {
            let (catapults, targets) = try_parse(input)?;
            let score = (0..targets.len())
                .map(|shot| {
                    let target = targets[shot];
                    let catapult = catapults.iter().find(|c| c.can_hit(target).is_some()).unwrap();
                    catapult.segment_number * catapult.power_to_hit(target).unwrap()
                }).sum();
            Ok(score)
        },
        3 => {
            let catapults = [
                Catapult { coordinates: Coordinates { x: 0, y: 0 }, segment_number: 1 },
                Catapult { coordinates: Coordinates { x: 0, y: 1 }, segment_number: 2 },
                Catapult { coordinates: Coordinates { x: 0, y: 2 }, segment_number: 3 },
            ];
            let meteors = input.lines().map(Coordinates::try_from).collect::<Result<Vec<_>, _>>()?;
            let score = meteors.iter().map(|meteor| {
                for time in meteor.x.div_ceil(2)..(meteor.x).max(meteor.y) {
                    let target = Coordinates { x: meteor.x - time, y: meteor.y - time };
                    let shooters: Vec<_> = catapults.iter().filter(|c| c.can_hit(target).is_some()).collect();
                    if !shooters.is_empty() {
                        return shooters.iter().map(|c| 
                                c.segment_number * c.power_to_hit(target).unwrap()
                            ).min().unwrap();
                    }
                }
                panic!("A meteor could not be hit by any catapult");
            }).sum();

            Ok(score)
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
        let expected = [13, 22, 13];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [203, 20075, 718678];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
