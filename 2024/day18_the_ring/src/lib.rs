use core::fmt::Display;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyMap,
    InvalidChar(char),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMap => write!(f, "Input was empty"),
            Self::InvalidChar(e) => write!(f, "Unable to parse {e}. Valid characters are \'.\', \'#\', and \'P\'."),
        }
    }
}

type Coordinates = (usize, usize);

#[derive(Clone)]
struct Map {
    walkable: Vec<Vec<bool>>,
    trees: Vec<Coordinates>,
    entries: Vec<Coordinates>,
}

impl TryFrom<&str> for Map {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut walkable = Vec::new();
        let mut trees = Vec::new();
        let mut entries = Vec::new();

        for (y, line) in value.lines().enumerate() {
            let mut walkable_line = Vec::new();
            for (x, c) in line.chars().enumerate() {
                match c {
                    '.' => walkable_line.push(true),
                    '#' => walkable_line.push(false),
                    'P' => {
                        walkable_line.push(true);
                        trees.push((y, x));
                    },
                    e => return Err(Self::Error::InvalidChar(e)),
                }
            }
            walkable.push(walkable_line);
        }
        if walkable.is_empty() {
            return Err(Self::Error::EmptyMap);
        }
        if let Some(x) = walkable[0].iter().position(|w| *w) {
            entries.push((0, x));
        } 
        if let Some(x) = walkable.last().unwrap().iter().position(|w| *w) {
            entries.push((walkable.len()-1, x));
        }
        if let Some(y) = walkable.iter().position(|row| row.first() == Some(&true)) {
            entries.push((y, 0));
        }
        if let Some(y) = walkable.iter().position(|row| row.iter().last() == Some(&true)) {
            entries.push((y, walkable[y].len()-1));
        } 
        Ok(Self { walkable, trees, entries, })
    }
}

impl Map {
    fn water(&self) -> Vec<usize> {
        let mut open_set = self.entries.iter().map(|e| (*e, 0)).collect::<VecDeque<_>>();
        let mut to_collect = self.trees.clone();
        let mut visited: HashSet<Coordinates> = self.entries.iter().cloned().collect();
        let mut watering_times = Vec::new();

        while let Some(((y, x), dist)) = open_set.pop_front() {
            if let Ok(idx) = to_collect.binary_search(&(y, x)) {
                to_collect.remove(idx);
                watering_times.push(dist);
            }
            if to_collect.is_empty() {
                return watering_times;
            }
            [(1,0), (1,2), (0,1), (2,1)]
                .iter()
                .filter(|(dy, dx)|
                    y + dy > 0 &&
                    y + dy <= self.walkable.len() &&
                    x + dx > 0 &&
                    x + dx <= self.walkable[y + dy - 1].len() &&
                    self.walkable[y + dy - 1][x + dx - 1]
                ).for_each(|(dy, dx)| {
                    let new_pos = (y + dy - 1, x + dx - 1);
                    if !visited.contains(&new_pos) {
                        visited.insert(new_pos);
                        open_set.push_back((new_pos, dist+1));
                    }
                });
        }
        // We didn't find a path to all the plants
        Vec::new()
    }

    fn best_watering(&mut self) -> Option<usize> {
        // Find the best spot by watering from the trees and noting how much combined time was
        // spent to reach it. Note that this isn't necessarily the spot we reach first.
        let mut open_set = self.trees.iter().enumerate().map(|(idx, t)| (*t, idx, 0)).collect::<VecDeque<_>>();
        let mut visited: HashMap<Coordinates, (Vec<usize>, usize)> = self.trees.iter().enumerate().map(|(idx, t)| (*t, (Vec::from([idx]), 0))).collect();

        while let Some(((y, x), orig, dist)) = open_set.pop_front() {
            [(1,0), (1,2), (0,1), (2,1)]
                .iter()
                .filter(|(dy, dx)|
                    y + dy > 0 &&
                    y + dy <= self.walkable.len() &&
                    x + dx > 0 &&
                    x + dx <= self.walkable[y + dy - 1].len() &&
                    self.walkable[y + dy - 1][x + dx - 1]
                ).for_each(|(dy, dx)| {
                    let new_pos = (y + dy - 1, x + dx - 1);
                    if !visited.contains_key(&new_pos) || !visited.get(&new_pos).unwrap().0.contains(&orig)  {
                        visited.entry(new_pos).and_modify(|(trees, d)| {
                            trees.push(orig);
                            *d += dist; 
                        }).or_insert((vec![orig], dist));
                        open_set.push_back((new_pos, orig, dist+1));
                    }
                });
        }

        // Now, exccluding spots with a tree in them, find the one with the best score.
        if let Some(entry) = visited
            .iter()
            .filter(|(coords, _)| !self.trees.contains(coords))
            .min_by_key(|(_coords, (_trees, dist))| dist)
            .map(|(coords, _)| coords)
        {
            self.entries.push(*entry);
            Some(self.water().iter().sum())
        } else {
            None
        }
    }
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseError> {
    let mut map = Map::try_from(input)?;
    match part {
        1 | 2 => Ok(*map.water().last().unwrap()),
        3 => Ok(map.best_watering().unwrap()),
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
        let expected = [11, 21, 12];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [103, 1383, 246261];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
