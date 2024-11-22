use core::fmt::Display;
use std::{collections::{BTreeSet, HashSet}, num::ParseIntError};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyInput,
    ParseDirError(Option<char>),
    ParseIntError(ParseIntError),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "Input did not contain any branches"),
            Self::ParseDirError(e) => write!(f, "Unable to parse {e:?} into a Direction"),
            Self::ParseIntError(e) => write!(f, "Unable to parse into a number: {e}"),
        }
    }
}

impl From<ParseIntError> for ParseError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

#[derive(PartialEq, Eq)]
enum Direction { Up, Down, Right, Left, Forward, Backward }

impl TryFrom<Option<char>> for Direction {
    type Error = ParseError;

    fn try_from(value: Option<char>) -> Result<Self, Self::Error> {
        match value {
            Some('U') => Ok(Self::Up),
            Some('D') => Ok(Self::Down),
            Some('R') => Ok(Self::Right),
            Some('L') => Ok(Self::Left),
            Some('F') => Ok(Self::Forward),
            Some('B') => Ok(Self::Backward),
            e => Err(Self::Error::ParseDirError(e)),
        }
    }
}

struct Schedule {
    steps: Vec<(Direction, usize)>,
}

impl TryFrom<&str> for Schedule {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut steps = Vec::new();
        for step in value.split(',') {
            let direction = Direction::try_from(step.chars().next())?;
            let distance = step[1..].parse::<usize>()?;
            steps.push((direction, distance));
        }
        Ok(Schedule { steps, })
    }
}

impl Schedule {
    fn max_height(&self) -> usize {
        self.steps
            .iter()
            .fold((0, 0), |(curr, max), next| {
                match next {
                    (Direction::Up, dist) => (curr+dist, max.max(curr+dist)),
                    (Direction::Down, dist) => (curr-dist, max),
                    _ => (curr, max),
                }
            }).1
    }

    fn segments(&self) -> HashSet<[isize; 3]> {
        let mut curr = [0, 0, 0];
        let mut segments = HashSet::new();
        self.steps.iter().for_each(|(dir, dist)| {
            let (dim_idx, signum) = match dir {
                Direction::Up => (0, 1),
                Direction::Down => (0, -1),
                Direction::Right => (1, 1),
                Direction::Left => (1,-1),
                Direction::Forward => (2, 1),
                Direction::Backward => (2, -1),
            };
            (0..*dist).for_each(|_| {
                curr[dim_idx] += signum;
                segments.insert(curr);
            });
        });
        segments
    }

    fn leaf(&self) -> [isize; 3] {
        self.steps
            .iter()
            .fold([0, 0, 0], |acc, (dir, dist)| {
                match dir {
                    Direction::Up => [acc[0] + *dist as isize, acc[1], acc[2]],
                    Direction::Down => [acc[0] - *dist as isize, acc[1], acc[2]],
                    Direction::Right => [acc[0], acc[1] + *dist as isize, acc[2]],
                    Direction::Left => [acc[0], acc[1] - *dist as isize, acc[2]],
                    Direction::Forward => [acc[0], acc[1], acc[2] + *dist as isize],
                    Direction::Backward => [acc[0], acc[1], acc[2] - *dist as isize],
                }
            })
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Position {
    estimated_total_costs: usize,
    costs_so_far: usize,
    coordinates: [isize; 3],
}

fn distance(from: &[isize; 3], to: &[isize; 3], segments: &HashSet<[isize; 3]>) -> Option<usize> {
    let estimate = |curr: &[isize; 3]| -> usize {
        curr[0].abs_diff(to[0]) + curr[1].abs_diff(to[1]) + curr[2].abs_diff(to[2])
    };
    let mut visited = HashSet::new();
    let mut open_set = BTreeSet::from([Position{
        estimated_total_costs: estimate(from),
        costs_so_far: 0,
        coordinates: *from,
    }]);
    while let Some(curr) = open_set.pop_first() {
        let coords = curr.coordinates;
        if &coords == to {
            return Some(curr.costs_so_far);
        }
        if !visited.contains(&coords) {
            visited.insert(coords);
            [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)]
                .iter()
                .filter(|(dy, dx, dz)| segments.contains(&[coords[0]+dy, coords[1]+dx, coords[2]+dz]) && !visited.contains(&[coords[0]+dy, coords[1]+dx, coords[2]+dz]))
                .for_each(|(dy, dx, dz)| {
                    let coordinates = [coords[0]+dy, coords[1]+dx, coords[2]+dz];
                    let costs_so_far = curr.costs_so_far+1;
                    let estimated_total_costs = costs_so_far + estimate(&coordinates);
                    open_set.insert(Position{ estimated_total_costs, costs_so_far, coordinates, });
                });
        }
    }
    None
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseError> {
    let branches = input.lines().map(Schedule::try_from).collect::<Result<Vec<_>, _>>()?;
    if branches.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    match part {
        1 => Ok(branches[0].max_height()),
        2 => {
            let segments = branches
                .iter()
                .map(|branch| branch.segments())
                .reduce(|acc, e| acc.union(&e).cloned().collect())
                .unwrap();
            Ok(segments.len())
        },
        3 => {
            let segments = branches
                .iter()
                .map(|branch| branch.segments())
                .reduce(|acc, e| acc.union(&e).cloned().collect())
                .unwrap();
            let mut leaves: Vec<_> = branches.iter().map(|branch| branch.leaf()).collect();
            leaves.sort();
            let mut min = usize::MAX;

            'height: for height in leaves[0][0]..=leaves.last().unwrap()[0] {
                let mut total_d = 0;
                for leaf in &leaves {
                    if let Some(leaf_d) = distance(leaf, &[height, 0, 0], &segments) {
                        total_d += leaf_d;
                    } else {
                        continue 'height;
                    }
                }
                min = min.min(total_d);
            }
            Ok(min)
        }
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
        let expected = [7, 32, 46];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [155, 4956, 1378];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
