use core::fmt::Display;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyMap,
    InvalidChar(char),
    MultipleStartingPositions,
    NoStartingPosition,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMap => write!(f, "Input doesn't contain a map"),
            Self::InvalidChar(e) => write!(f, "Unable to parse {e} into a map tile"), 
            Self::MultipleStartingPositions => write!(f, "Multiple starting positions found"),
            Self::NoStartingPosition => write!(f, "No starting position found"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile { Warm, Cold, Stagnant, Rock }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Direction { Up, Down, Left, Right, None }

struct Map {
    tiles: Vec<Vec<Tile>>,
    starting: (usize, usize),
    checkpoints: Vec<(usize, usize)>,
}

impl TryFrom<&str> for Map {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut tiles = Vec::new();
        let mut starting = None;
        let mut checkpoints = BTreeMap::new();
        for (y, line) in value.lines().enumerate() {
            let mut row = Vec::new();
            for (x, c) in line.chars().enumerate() {
                match c {
                    '#' => row.push(Tile::Rock),
                    '+' => row.push(Tile::Warm),
                    '-' => row.push(Tile::Cold),
                    '.' => row.push(Tile::Stagnant),
                    'S' => {
                        if starting.is_none() {
                            row.push(Tile::Stagnant);
                            starting = Some((x, y));
                        } else {
                            return Err(Self::Error::MultipleStartingPositions);
                        }
                    },
                    c if ['A', 'B', 'C'].contains(&c) => {
                        row.push(Tile::Stagnant);
                        checkpoints.insert(c, (x, y));
                    },
                    e => return Err(Self::Error::InvalidChar(e)),
                }
            }
            tiles.push(row);
        }
        if tiles.is_empty() {
            return Err(Self::Error::EmptyMap);
        }
        if let Some(starting) = starting {
            let checkpoints = checkpoints.values().cloned().collect();
            Ok(Self { tiles, starting, checkpoints, })
        } else {
            Err(Self::Error::NoStartingPosition)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Glider {
    altitude: usize,
    coordinates: (usize, usize),
    facing: Direction,
}

impl PartialOrd for Glider {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Glider {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (2 * self.altitude + self.coordinates.1).cmp(&(2 * other.altitude + other.coordinates.1))
            // .then_with(|| self.coordinates.1.cmp(&other.coordinates.1))
            .then_with(|| self.altitude.cmp(&other.altitude))
        // self.altitude.cmp(&other.altitude)
            // .then_with(|| self.coordinates.1.cmp(&other.coordinates.1))
            .then_with(|| self.coordinates.cmp(&other.coordinates))
            .then_with(|| self.facing.cmp(&other.facing))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AltState {
    glider: Glider,
    time_remaining: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct RaceState {
    estimated_total: usize,
    time_spent: usize,
    checkpoints_remaining: usize,
    glider: Glider,
}


impl PartialOrd for AltState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AltState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (other.glider.altitude + other.time_remaining).cmp(&(self.glider.altitude + self.time_remaining))
            .then_with(|| other.glider.cmp(&self.glider))
    }
}

impl Glider {
    fn new(starting_position: (usize, usize), altitude: usize) -> Self {
        Self { 
            altitude,
            coordinates: starting_position, 
            facing: Direction::None,
        }
    }

    fn next_tiles(&self, height: usize, width: usize) -> Vec<(usize, usize, Direction)> {
        let mut res = Vec::new();
        let (x, y) = self.coordinates;
        if x > 0 && self.facing != Direction::Right {
            res.push((x-1, y, Direction::Left));
        }
        if y > 0 && self.facing != Direction::Down {
            res.push((x, y-1, Direction::Up));
        }
        if x < width-1 && self.facing != Direction::Left {
            res.push((x+1, y, Direction::Right));
        }
        if y < height-1 && self.facing != Direction::Up {
            res.push((x, y+1, Direction::Down));
        }
        res
    }

    fn fly_max(&self, map: &Map, time: usize) -> usize {
        let start = AltState { glider: *self, time_remaining: time, };
        let mut open_set = BTreeSet::from([start]);
        let mut visited = HashSet::new();
        while let Some(state) = open_set.pop_first() {
            let glider = state.glider;
            if state.time_remaining == 0 {
                return glider.altitude;
            }
            glider.next_tiles(map.tiles.len(), map.tiles[glider.coordinates.1].len()).iter().for_each(|&(x, y, facing)| {
                let tile = map.tiles[y][x];
                let altitude = match tile {
                    Tile::Warm => glider.altitude + 1,
                    Tile::Cold => glider.altitude.saturating_sub(2),
                    _ => glider.altitude - 1,
                };
                let glider = Glider { altitude, coordinates: (x, y), facing };
                let next = AltState { glider, time_remaining: state.time_remaining - 1, };
                if tile != Tile::Rock && altitude > 0 && !visited.contains(&next) {
                    visited.insert(next);
                    open_set.insert(next);
                }
            });
        }
        0
    }

    fn race(&self, map: &Map) -> usize {
        let target_altitude = self.altitude;
        let checkpoints = map.checkpoints.len();
        let estimate = |from: &RaceState| -> usize {
            let mut target = checkpoints - from.checkpoints_remaining;
            let mut dist = 0;
            let mut pos = from.glider.coordinates;
            while target < checkpoints {
                let next = map.checkpoints[target];
                dist += pos.0.abs_diff(next.0) + pos.1.abs_diff(next.1);
                pos = next;
                target += 1;
            }
            dist += pos.0.abs_diff(map.starting.0) + pos.1.abs_diff(map.starting.1);
            dist.max(target_altitude.saturating_sub(from.glider.altitude))
        };
        let start = RaceState { glider: *self, estimated_total: 0, time_spent: 0, checkpoints_remaining: checkpoints };
        let mut open_set = VecDeque::from([start]);
        let mut visited = HashSet::new();
        while let Some(state) = open_set.pop_front() {
            let glider = state.glider;
            if state.checkpoints_remaining == 0 && glider.coordinates == map.starting && glider.altitude >= target_altitude {
                return state.time_spent;
            }
            let checkpoints_remaining = if state.checkpoints_remaining > 0 && glider.coordinates == map.checkpoints[map.checkpoints.len() - state.checkpoints_remaining] {
                state.checkpoints_remaining - 1
            } else {
                state.checkpoints_remaining
            };
            glider.next_tiles(map.tiles.len(), map.tiles[glider.coordinates.1].len()).iter().for_each(|&(x, y, facing)| {
                let tile = map.tiles[y][x];
                let altitude = match tile {
                    Tile::Warm => glider.altitude + 1,
                    Tile::Cold => glider.altitude.saturating_sub(2),
                    _ => glider.altitude - 1,
                };
                let glider = Glider { altitude, coordinates: (x, y), facing };
                let mut next = RaceState { glider, checkpoints_remaining, time_spent: state.time_spent + 1, estimated_total: 0 };
                let remaining = estimate(&next);
                next.estimated_total = state.time_spent + remaining + 1;
                if tile != Tile::Rock && altitude > 0 && !visited.contains(&(glider, checkpoints_remaining)) {
                    visited.insert((glider, checkpoints_remaining));
                    open_set.push_back(next);
                }
            });
        }
        0
    }

    fn glide_max(&self, map: &Map) -> usize {
        let start = *self;
        let mut open_set = BTreeSet::from([start]);
        let mut visited = HashMap::new();
        while let Some(glider) = open_set.pop_last() {
            let (x, y) = glider.coordinates;
            if glider.altitude == 0 {
                return y;
            }
            // glider.next_tiles(map.tiles.len(), map.tiles[glider.coordinates.1].len()).iter().for_each(|&(x, y, facing)| {
            [(1, 2, Direction::Down), (0, 1, Direction::Left), (2, 1, Direction::Right)]
                .iter()
                .filter(|(dx, dy, facing)| {
                    x+dx > 0 &&
                    x+dx <= map.tiles[(y+dy-1) % map.tiles.len()].len() &&
                    (*facing != Direction::Right || glider.facing != Direction::Left) &&
                    (*facing != Direction::Left || glider.facing != Direction::Right) &&
                    map.tiles[(y+dy-1) % map.tiles.len()][x+dx-1] != Tile::Rock
                }).for_each(|&(dx, dy, facing)|
            {
                let tile = map.tiles[(y+dy-1) % map.tiles.len()][x+dx-1];
                if let Some(altitude) = match tile {
                    Tile::Warm => Some(glider.altitude + 1),
                    Tile::Cold => glider.altitude.checked_sub(2),
                    _ => Some(glider.altitude - 1),
                } {
                    let glider = Glider { altitude, coordinates: (x+dx-1, y+dy-1), facing };
                    // if tile != Tile::Rock && altitude > 0 && !visited.contains(&glider) {
                    match visited.get(&(glider.coordinates, glider.facing)) {
                        l if l.is_none() || *l.unwrap() < altitude => {
                            visited.insert((glider.coordinates, glider.facing), altitude);
                            open_set.insert(glider);
                        },
                        _ => (),
                    }
                }
            });
        }
        0
    }
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseError> {
    let map = Map::try_from(input)?;
    match part {
        1 => {
            let glider = Glider::new(map.starting, 1000);
            Ok(glider.fly_max(&map, 100))
        },
        2 => {
            let glider = Glider::new(map.starting, 10000);
            Ok(glider.race(&map))
        },
        3 => {
            let glider = Glider::new(map.starting, 384400);
            Ok(glider.glide_max(&map))
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
        let expected = [1045, 24, 768790];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [1029, 556, 768792];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
