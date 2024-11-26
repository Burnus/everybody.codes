use core::fmt::Display;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError<'a> {
    InputMalformed,
    LineMalformed(&'a str),
    ParseIntError(&'a str),
}

impl Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputMalformed => write!(f, "Input should consist of the spin rates, an empty line and the sequence of cat faces"),
            Self::LineMalformed(e) => write!(f, "Unable to parse malformed line: {e}\nShould be of format:\n"),
            Self::ParseIntError(e) => write!(f, "Unable to parse into a number: {e}"),
        }
    }
}

type Face = [u8; 3];

struct Configuration {
    advance_by: Vec<usize>,
    wheels: Vec<Vec<Face>>,
}

impl<'a> TryFrom<&'a str> for Configuration {
    type Error = ParseError<'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Some((advance, faces)) = value.split_once("\n\n") {
            let advance_by: Vec<usize> = advance
                .split(',')
                .map(|i| i.parse::<usize>().map_err(|_| Self::Error::ParseIntError(i)))
                .collect::<Result<Vec<usize>, ParseError>>()?;
            let wheel_count = advance_by.len();
            let mut wheels = vec![Vec::new(); wheel_count];

            for line in faces.lines() {
                if line.len() > 4 * wheel_count || [1, 2].contains(&(line.len() % 4)) {
                    return Err(Self::Error::LineMalformed(line));
                }
                for wheel in 0..wheel_count.min((line.len() + 1) / 4) {
                    let face = &line[4*wheel..4*wheel+3];
                    if face == "   " {
                        continue;
                    }
                    let face: [u8; 3] = face.as_bytes().try_into().unwrap();

                    wheels[wheel].push(face);
                }
            }
            Ok(Self { advance_by, wheels, })
        } else {
            Err(Self::Error::InputMalformed)
        }
    }
}

impl Configuration {
    /// Get the faces after `pull_count` pulls of the right lever, and `adjust` pulls of the left
    /// lever. Pushes of the left lever are represented by negative `adjust` values.
    ///
    /// The absolute value of `adjust` MUST be less than or equal to pull_count, as the calculation
    /// may underflow otherwise. `panic()`s in debug mode otherwise.
    fn at(&self, pull_count: usize, adjust: isize) -> Vec<[u8; 3]> {
        debug_assert!(adjust.abs_diff(0) <= pull_count);
        self.wheels
            .iter()
            .enumerate()
            .map(|(idx, wheel)| wheel[((pull_count * self.advance_by[idx]) as isize + adjust) as usize % wheel.len()])
            .collect()
    }
    
    fn print_at(&self, pull_count: usize) -> String {
        self.at(pull_count, 0).iter().map(Self::face).collect::<Vec<_>>().join(" ")
    }

    fn face(bytes: &[u8; 3]) -> String {
        std::str::from_utf8(bytes).unwrap().to_string()
    }

    fn score(faces: &[[u8; 3]]) -> usize {
        // Since we never calculate the score in the way described in part 1 (including muzzles),
        // we only need to worry about the "eyes", i. e. symbols 0 and 2 in our faces.
        let mut symbols = HashMap::new();
        faces.iter().for_each(|face| {
            symbols.entry(face[0]).and_modify(|count| *count += 1).or_insert(1);
            symbols.entry(face[2]).and_modify(|count| *count += 1).or_insert(1);
        });

        symbols.iter().filter(|(_s, count)| **count > 2).map(|(_s, count)| *count - 2).sum()
    }

    fn score_after(&self, pull_count: usize) -> usize {
        // All symbols must repeat after a number of pulls equal to the least common multiple of
        // all wheel sizes. If we surpass that number, we can extrapolate any future scores.
        let cycle_len = self.wheels
            .iter()
            .map(|wheel| wheel.len())
            .reduce(lcm)
            .unwrap();

        if cycle_len < pull_count {
            let rest = (1..=pull_count % cycle_len)
                .map(|pull| Self::score(&self.at(pull, 0)))
                .sum::<usize>();
            let per_cycle = rest + (((pull_count % cycle_len)+1)..=cycle_len)
                .map(|pull| Self::score(&self.at(pull, 0)))
                .sum::<usize>();
            (pull_count / cycle_len) * per_cycle + rest
        } else {
            (1..=pull_count)
                .map(|pull| Self::score(&self.at(pull, 0)))
                .sum::<usize>()
        }
    }

    fn min_max(&self, pull_count: usize) -> (usize, usize) {
        // After each pull of the right lever, for each sum of pulls - pushes of the left lever, 
        // this Vec will represent the (min, max) coin values that can be won this way at index 
        // [pull_count - pulls + pushes].
        // These values are added to the possible outcomes of pulling the right lever again (and
        // possibly pushing/pulling the left one once more), to determine the new (min, max)s.
        // Since the left lever can only be operated once per pull, the new values for each cell
        // only depend on the (min, max)s of it and their two neighbouring cells (except for the
        // edge cases of pushing/pulling every time, which only have one neighbour in the previous
        // step), and the possible scores for this pull itself.
        let mut res = vec![(0, 0); 2 * pull_count + 1];

        (1..=pull_count as isize)
            .for_each(|pull| {
                let possible_outcomes = (-pull..=pull)
                    .map(|push_pull| {
                        let this = Self::score(&self.at(pull as usize, push_pull));
                        let pred = res
                            .iter()
                            .skip(0.max(pull_count as isize + push_pull - 1) as usize)
                            .take(3)
                            .filter(|range| **range != (0, 0) || pull == 1)
                            .copied()
                            .collect::<Vec<(usize, usize)>>();
                        let min = *pred.iter().map(|(min, _max)| min).min().unwrap();
                        let max = *pred.iter().map(|(_min, max)| max).max().unwrap();
                        (min + this, max + this)
                    }).collect::<Vec<(usize, usize)>>();
                possible_outcomes.iter().enumerate().for_each(|(idx, outcome)|
                    res[pull_count - pull as usize + idx] = *outcome);
            });

        let min = *res.iter().map(|(min, _max)| min).min().unwrap();
        let max = *res.iter().map(|(_min, max)| max).max().unwrap();
        (min, max)
    }
}

fn gcd(lhs: usize, rhs: usize) -> usize {
    let mut a = lhs;
    let mut b = rhs;
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn lcm(lhs: usize, rhs: usize) -> usize {
    (lhs / gcd(lhs, rhs)).saturating_mul(rhs)
}

pub fn run(input: &str, part: usize) -> Result<String, ParseError> {
    let config = Configuration::try_from(input)?;
    match part {
        1 => Ok(config.print_at(100)),
        2 => Ok(format!("{}", config.score_after(202420242024))),
        3 => {
            let (min, max) = config.min_max(256);
            Ok(format!("{max} {min}"))
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
        let expected = [">.- -.- ^,-", "280014668134", "627 128"];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1].to_string()));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [">_^ ^.> >_^ >,^", "105328965118", "619 80"];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1].to_string()));
        }
    }
}
