use core::fmt::Display;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError<'a> {
    LineMalformed(&'a str),
}

impl Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LineMalformed(e) => write!(f, "Unable to parse malformed line: {e}\nShould be of format:\nA:B,C,D"),
        }
    }
}

type Termite=usize;

fn try_cycles_from(input: &str) -> Result<(Vec<Vec<Termite>>, HashMap<&str, Termite>), ParseError> {
    let mut names = HashMap::new();
    let mut res = Vec::new();
    for line in input.lines() {
        if let Some((from, to)) = line.split_once(':') {
            let next_idx = names.len();
            let from_idx = *names.entry(from).or_insert(next_idx);
            while res.len() <= from_idx {
                res.push(Vec::new());
            }
            res[from_idx] = to
                .split(',')
                .map(|name| {
                    let next_idx = names.len();
                    *names.entry(name).or_insert(next_idx)
                }).collect();
        } else {
            return Err(ParseError::LineMalformed(line));
        }
    }
    Ok((res, names))
}

fn reproduce(cycles: &[Vec<Termite>], population: &mut Vec<usize>, days: usize) {
    let mut next_gen = Vec::with_capacity(cycles.len());
    (0..days).for_each(|_| {
        next_gen = vec![0; cycles.len()];
        population.iter().enumerate().for_each(|(category, count)| {
            let children = &cycles[category];
            children.iter().for_each(|child| {
                next_gen[*child] += count;
            });
        });
        std::mem::swap(population, &mut next_gen);
        next_gen.clear();
    });
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseError> {
    let (cycles, names) = try_cycles_from(input)?;
    let mut termites = vec![0; cycles.len()];
    match part {
        1 => {
            termites[*names.get("A").unwrap()] = 1;
            reproduce(&cycles, &mut termites, 4);
            Ok(termites.iter().sum())
        },
        2 => {
            termites[*names.get("Z").unwrap()] = 1;
            reproduce(&cycles, &mut termites, 10);
            Ok(termites.iter().sum())
        },
        3 => {
            let mut low = usize::MAX;
            let mut high = usize::MIN;

            (0..cycles.len()).for_each(|c| {
                termites = vec![0; cycles.len()];
                termites[c] = 1;
                reproduce(&cycles, &mut termites, 20);
                let population = termites.iter().sum();
                low = low.min(population);
                high = high.max(population);
            });

            Ok(high-low)
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
        let expected = [8, 144, 268815];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [41, 213729, 896125189572];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
