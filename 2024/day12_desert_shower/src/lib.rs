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
    /// If this catapult is able to hit the `target` coordinates, this returns `Some(p)`, where `p`
    /// is the `Phase`, in which it will be hit. If it cannot be hit, `None` is returned.
    fn can_hit(&self, target: Coordinates) -> Option<Phase> {
        if target.x <= self.coordinates.x {
            // we only ever shoot to the right
            None
        } else if target.y <= self.coordinates.y {
            // Special case to avoid underflows and division by zero below.
            // If the target is on equal or lower height than the catapult, we can only ever hit it
            // in descend phase (or not at all), so we don't need to check the other cases
            if (target.x + target.y - (self.coordinates.x + self.coordinates.y)) % 3 == 0 {
                Some(Phase::Descend)
            } else {
                None
            }
        } else {
            match (target.x - self.coordinates.x).div_ceil(target.y - self.coordinates.y) {
                // The match formula determines in which phase (if any) we could hit the target:
                // * (>0..1): The y difference is greater than the x difference. We can't possibly
                //            hit. (Exact 0 is already being handled by the special casing above).
                // * 1 exactly: The x and y differences are equal. We definately hit in ascend phase.
                // * (>1..=2): This marks the glide phase. We definately hit there.
                // * (>2..): Descend phase. We hit if the sums of x and y differ by a multiple of 3.
                1 => if target.y - self.coordinates.y == target.x - self.coordinates.x {
                        Some(Phase::Ascend)
                    } else {
                        None
                    },
                2 => Some(Phase::Glide),
                _ => if (target.x + target.y - (self.coordinates.x + self.coordinates.y)) % 3 == 0 {
                        Some(Phase::Descend)
                    } else {
                        None
                    },
            }
        }
    }

    /// Returns the (minimal) shooting power required to hit the `target`, or `None` if it cannot be
    /// hit. If the target is being hit in the ascend phase, any power greater than or equal to the
    /// returned value will hit the target. In the other two phases, only this exact value will
    /// hit.
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
                'H' => targets.append(&mut vec![Coordinates { x, y }; 2]), // same as 2 targets in the same spot
                c if ['A', 'B', 'C'].contains(&c) => 
                    catapults.push( Catapult { 
                        coordinates: Coordinates { x, y }, 
                        segment_number: c as usize - b'@' as usize,
                    }),
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
            // Despite the challenge suggesting it, the order in which we attack the targets
            // doesn't actually matter. Since all targets are being hit in the descend phase, and
            // during that phase, any point can be reached by exactly one of our catapults (see
            // sketch below) with exactly one value of shooting power, the ranking of each target 
            // can only ever have one value. Hence, we only need to make sure to visit every target
            // exactly once.
            //
            // The following sketch shows, which points are reachable from which catapult in
            // descend phase with shooting powers 1..=3 (lowercase letters indicate the catapults, 
            // uppercase the reachable points; dots are not reachable in descend phase):
            //
            // ........C
            // ......C.BC
            // .c..C.BCABC
            // .b..BCABCABC
            // .a..ABCABCABC
            // =============
            let score = targets
                .iter()
                .map(|&target| {
                    catapults
                        .iter()
                        .find_map(|c| c.power_to_hit(target).map(|p| p * c.segment_number))
                        .expect("target unreachable")
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
            let score = meteors
                .iter()
                .map(|meteor| {
                    (meteor.x.div_ceil(2)..=(meteor.x).max(meteor.y))
                        .find_map(|time| {
                            let target = Coordinates { x: meteor.x - time, y: meteor.y - time };
                            catapults
                                .iter()
                                .filter_map(|c| c.power_to_hit(target).map(|p| p * c.segment_number))
                                .min()
                        }).expect("target unreachable")
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
