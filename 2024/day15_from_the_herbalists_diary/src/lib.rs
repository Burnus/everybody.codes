use core::fmt::Display;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyInput,
    NonRectangular,
    NoStart,
    ParseCharError(char),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "Input doesn't contain a map"),
            Self::NonRectangular => write!(f, "Input is not rectangular"),
            Self::NoStart => write!(f, "First line doesn't contain a walkable tile"),
            Self::ParseCharError(e) => write!(f, "Unable to parse into a field: {e}"),
        }
    }
}

type Coordinates = (usize, usize);

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Position {
    estimated_total_costs: u32,
    costs: u32,
    to_collect: usize,
    coordinates: Coordinates,
    collecting: u8,
}

struct Map {
    walkable: Vec<Vec<bool>>,
    herbs: HashMap<u8, Vec<Coordinates>>,
    width: usize,
    height: usize,
    start: Coordinates,
}

impl TryFrom<&str> for Map {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut herbs: HashMap<u8, Vec<Coordinates>> = HashMap::new();
        let mut walkable = Vec::new();
        for (y, line) in value.lines().enumerate() {
            let mut walkable_row = Vec::new();
            for (x, c) in line.chars().enumerate() {
                match c {
                    '.' => walkable_row.push(true),
                    '#' | '~' => walkable_row.push(false),
                    l if l.is_ascii_uppercase() => {
                        walkable_row.push(true);
                        herbs.entry(l as u8 - b'A').and_modify(|v| v.push((x, y))).or_insert(Vec::from([(x, y)]));
                    },
                    e => return Err(Self::Error::ParseCharError(e)),
                }
            }
            walkable.push(walkable_row);
        }
        let height = walkable.len();
        if height == 0 {
            return Err(Self::Error::EmptyInput);
        }
        let width = walkable[0].len();
        if walkable.iter().any(|row| row.len() != width) {
            return Err(Self::Error::NonRectangular);
        }
        let start_x = walkable[0].iter().position(|tile| *tile).ok_or(Self::Error::NoStart)?;
        Ok(Self { walkable, herbs, width, height, start: (start_x, 0), })
    }
}

impl Map {
    fn route_single(&self, herb: u8) -> Option<u32> {
        let mut open_set = VecDeque::from([(self.start, 0)]);
        let mut visited = HashSet::from([self.start]);
        if let Some(targets) = self.herbs.get(&herb) {
            while let Some((pos, dist)) = open_set.pop_front() {
                if targets.contains(&pos) {
                    return Some(dist);
                }
                [(0, 1), (2, 1), (1, 0), (1, 2)]
                    .iter()
                    .filter(|(dx, dy)| {
                        pos.0 + dx > 0 &&
                        pos.1 + dy > 0 &&
                        pos.0 + dx <= self.width &&
                        pos.1 + dy <= self.height &&
                        self.walkable[pos.1+dy-1][pos.0+dx-1]
                    }).for_each(|(dx, dy)| {
                        let next_pos = (pos.0+dx-1, pos.1+dy-1);
                        if !visited.contains(&next_pos) {
                            visited.insert(next_pos);
                            open_set.push_back((next_pos, dist+1));
                        }
                    });
            }
        }
        None
    }

    fn route(&self, start: Coordinates, dest: Coordinates) -> Option<u32> {
        let mut open_set = VecDeque::from([(start, 0)]);
        let mut visited = HashSet::from([start]);
        while let Some((pos, dist)) = open_set.pop_front() {
            if pos == dest {
                return Some(dist);
            }
            [(0, 1), (2, 1), (1, 0), (1, 2)]
                .iter()
                    .filter(|(dx, dy)| {
                        pos.0 + dx > 0 &&
                            pos.1 + dy > 0 &&
                            pos.0 + dx <= self.width &&
                            pos.1 + dy <= self.height &&
                            self.walkable[pos.1+dy-1][pos.0+dx-1]
                    }).for_each(|(dx, dy)| {
                        let next_pos = (pos.0+dx-1, pos.1+dy-1);
                        if !visited.contains(&next_pos) {
                            visited.insert(next_pos);
                            open_set.push_back((next_pos, dist+1));
                        }
                    });
        }
        None
    }

    fn route_all(&self) -> Option<u32> {
        let interesting: Vec<(u8, Coordinates)> = self.herbs
            .iter()
            .flat_map(|(herb, coords)| coords.iter().cloned().map(|c| (*herb, c)).collect::<Vec<(u8, Coordinates)>>())
            .chain([(255, self.start)])
            .collect();
        let network: HashMap<(Coordinates, Coordinates), u32> = interesting
            .iter()
            .enumerate()
            .flat_map(|(l_idx, (l_herb, l_coords))| interesting
                .iter()
                .skip(l_idx + 1)
                .filter(|(r_herb, _r_coords)| l_herb != r_herb)
                .flat_map(|(_r_herb, r_coords)| {
                    let dist = self.route(*l_coords, *r_coords).unwrap();
                    [((*l_coords, *r_coords), dist), ((*r_coords, *l_coords), dist)]
                }).collect::<Vec<_>>())
            .collect();
        let all_herbs: Vec<_> = self.herbs.keys().cloned().collect();
        let all_herbs_int = (1_usize << all_herbs.len()) - 1;
        let mut open_set = BTreeSet::from([Position{ 
            estimated_total_costs: 0,
            costs: 0, 
            coordinates: self.start, 
            to_collect: all_herbs_int,
            collecting: 0,
        }]);
        let mut visited = HashMap::new();
        while let Some(pos) = open_set.pop_first() {
            if pos.to_collect == 0 {
                if pos.coordinates == self.start {
                    return Some(pos.costs);
                } else {
                    let costs = pos.costs + network.get(&(pos.coordinates, self.start)).unwrap();
                    open_set.insert(Position { 
                        estimated_total_costs: costs,
                        costs,
                        coordinates: self.start, 
                        to_collect: 0,
                        collecting: 0,
                    });
                }
            }
            if !visited.keys().any(|(coords, to_coll)| 
                    *coords == pos.coordinates &&
                    (to_coll ^ pos.to_collect) & to_coll == 0
                ) {
                visited.insert((pos.coordinates, pos.to_collect), pos.costs);

                let collected = all_herbs_int - pos.to_collect - (1 << pos.collecting);
                if let Some(remaining) = visited.get(&(pos.coordinates, collected)) {
                    open_set.insert(Position { 
                        estimated_total_costs: pos.costs + remaining,
                        costs: pos.costs + remaining,
                        coordinates: self.start,
                        to_collect: 0,
                        collecting: 0,
                    });
                } else {
                    all_herbs
                        .iter()
                        .enumerate()
                        .filter(|(idx, _herb)| pos.to_collect & (1_usize << idx) != 0)
                        .for_each(|(idx, herb)| {
                            let to_collect = pos.to_collect - (1_usize << idx);
                            self.herbs.get(herb).unwrap()
                                .iter()
                                .for_each(|&coordinates| {
                                    let costs = pos.costs + network.get(&(pos.coordinates, coordinates)).unwrap();
                                    let estimated_total_costs = costs + all_herbs
                                        .iter()
                                        .enumerate()
                                        .filter(|(idx, _herb)| to_collect & (1_usize << idx) != 0)
                                        .map(|(_idx, herb)| self.herbs.get(herb).unwrap()
                                            .iter()
                                            .map(|&c| (c, network.get(&(coordinates, c)).unwrap()))
                                            .min_by_key(|(_coords, dist)| *dist)
                                            .unwrap()
                                        ).map(|(c, d)| d + *network.get(&(c, self.start)).unwrap_or(&0))
                                        .max()
                                        .unwrap_or(0);

                                    open_set.insert(Position {
                                        estimated_total_costs,
                                        costs,
                                        coordinates, 
                                        to_collect,
                                        collecting: idx as u8,
                                    });
                                });
                        });
                }
            }
        
        }

        None
    }
}

pub fn run(input: &str, part: usize) -> Result<u32, ParseError> {
    let map = Map::try_from(input)?;
    match part {
        1 => Ok(2 * map.route_single(b'H' - b'A').unwrap()),
        2 | 3 => Ok(map.route_all().unwrap()),
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
        let expected = [26, 38];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [200, 526, 1504];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
