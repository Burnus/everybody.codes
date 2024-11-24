use core::fmt::Display;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyInput,
    GridTooBig,
    NonRectangular,
    NoStart,
    ParseCharError(char),
    TooManyHerbs,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "Input doesn't contain a map"),
            Self::GridTooBig => write!(f, "Input map is too big. Maximum allowed size is 256x256."),
            Self::NonRectangular => write!(f, "Input is not rectangular"),
            Self::NoStart => write!(f, "First line doesn't contain a walkable tile"),
            Self::ParseCharError(e) => write!(f, "Unable to parse into a field: {e}"),
            Self::TooManyHerbs => write!(f, "At most 16 herbs are supported"),
        }
    }
}

type Coordinates = (usize, usize);

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Position {
    estimated_total_costs: u16,
    costs: u16,
    to_collect: u16,
    coordinates: u16,
    collecting: u8,
}

struct Map {
    walkable: Vec<Vec<bool>>,
    herbs: Vec<Vec<u16>>,
    width: usize,
    height: usize,
    start: Coordinates,
}

impl TryFrom<&str> for Map {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut herb_ids: HashMap<char, usize> = HashMap::new();
        let mut herbs: Vec<Vec<u16>> = Vec::new();
        let mut walkable = Vec::new();
        for (y, line) in value.lines().enumerate() {
            let mut walkable_row = Vec::new();
            for (x, c) in line.chars().enumerate() {
                match c {
                    '.' => walkable_row.push(true),
                    '#' | '~' => walkable_row.push(false),
                    l if l.is_ascii_uppercase() => {
                        walkable_row.push(true);
                        let next_id = herbs.len();
                        if let Some(&idx) = herb_ids.get(&l) {
                            herbs[idx].push(((x << 8) + y) as u16);
                        } else {
                            herb_ids.insert(l, next_id);
                            herbs.push(vec![((x << 8) + y) as u16]);
                        }
                    },
                    e => return Err(Self::Error::ParseCharError(e)),
                }
            }
            walkable.push(walkable_row);
        }
        if herbs.len() > 16 {
            return Err(Self::Error::TooManyHerbs);
        }
        let height = walkable.len();
        if height == 0 {
            return Err(Self::Error::EmptyInput);
        }
        let width = walkable[0].len();
        if height > 0xFF && width > 0xFF {
            return Err(Self::Error::GridTooBig);
        }
        if walkable.iter().any(|row| row.len() != width) {
            return Err(Self::Error::NonRectangular);
        }
        let start_x = walkable[0].iter().position(|tile| *tile).ok_or(Self::Error::NoStart)?;
        Ok(Self { walkable, herbs, width, height, start: (start_x, 0), })
    }
}

impl Map {
    fn route(&self, start: Coordinates, dest: Coordinates) -> Option<u16> {
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

    fn route_single(&self, herb_idx: usize) -> Option<u16> {
        let mut open_set = VecDeque::from([(self.start, 0)]);
        let mut visited = HashSet::from([self.start]);
        if let Some(targets) = self.herbs.get(herb_idx) {
            let targets: Vec<_> = targets.iter().map(|coords| ((coords >> 8) as usize, (coords & 0xff) as usize)).collect();
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

    fn route_all_bfs(&self) -> Option<u16> {
        let start = ((self.start.0 << 8) + self.start.1) as u16;
        let all_herbs = (1_u16 << self.herbs.len()) - 1;
        let herbs_lut: HashMap<u16, u8> = self.herbs
            .iter()
            .enumerate()
            .flat_map(|(herb, coords)| coords.iter().map(|c| (*c, herb as u8)).collect::<Vec<_>>())
            .collect();
        let mut open_set = VecDeque::from([(start, all_herbs, 0)]);
        let mut visited = HashSet::from([(start, all_herbs)]);
        while let Some((pos, to_collect, dist)) = open_set.pop_front() {
            let (x, y) = (pos >> 8, pos & 0xFF);
            let to_collect = if let Some(herb) = herbs_lut.get(&pos) {
                to_collect & !(1_u16 << herb)
            } else {
                to_collect
            };

            if to_collect == 0 && pos == start {
                return Some(dist);
            }
            [(0, 1), (2, 1), (1, 0), (1, 2)]
                .iter()
                    .filter(|(dx, dy)| {
                        x + dx > 0 &&
                            y + dy > 0 &&
                            x + dx <= self.width as u16 &&
                            y + dy <= self.height as u16 &&
                            self.walkable[(y+dy-1) as usize][(x+dx-1) as usize]
                    }).for_each(|(dx, dy)| {
                        let next_pos = pos + (dx << 8) + dy - 0x101;
                        if !visited.contains(&(next_pos, to_collect)) {
                            visited.insert((next_pos, to_collect));
                            open_set.push_back((next_pos, to_collect, dist+1));
                        }
                    });
        }
        None
    }

    fn route_all_a_star(&self) -> Option<u16> {
        let start = ((self.start.0 << 8) + self.start.1) as u16;
        let interesting: Vec<(u8, u16)> = self.herbs
            .iter()
            .enumerate()
            .flat_map(|(herb, coords)| coords.iter().cloned().map(|c| (herb as u8, c)).collect::<Vec<(u8, u16)>>())
            .chain([(255, start)])
            .collect();
        let network: HashMap<(u16, u16), u16> = interesting
            .iter()
            .enumerate()
            .flat_map(|(l_idx, (l_herb, l_coords))| interesting
                .iter()
                .skip(l_idx + 1)
                .filter(|(r_herb, _r_coords)| l_herb != r_herb)
                .flat_map(|(_r_herb, r_coords)| {
                    let dist = self.route(((l_coords >> 8) as usize, (l_coords & 0xFF) as usize), ((r_coords >> 8) as usize, (r_coords & 0xFF) as usize)).unwrap();
                    [((*l_coords, *r_coords), dist), ((*r_coords, *l_coords), dist)]
                }).collect::<Vec<_>>())
            .collect();
        let estimate: HashMap<(u16, u8), u16> = interesting
            .iter()
            .flat_map(|(herb, coords)| self.herbs
                .iter()
                .enumerate()
                .filter(|(other_herb, _)| *other_herb as u8 != *herb)
                .map(|(other_herb, coords_vec)| ((*coords, other_herb as u8), coords_vec
                    .iter()
                    .map(|other_coords| network.get(&(*coords, *other_coords)).unwrap() + network.get(&(*other_coords, start)).unwrap())
                    .min()
                    .unwrap())
                ).collect::<Vec<_>>()
            ).collect();

        let all_herbs = (1_u16 << self.herbs.len()) - 1;
        let mut open_set = BTreeSet::from([Position{ 
            estimated_total_costs: 0,
            costs: 0, 
            coordinates: start, 
            to_collect: all_herbs,
            collecting: 0,
        }]);
        let mut visited = HashMap::new();
        while let Some(pos) = open_set.pop_first() {
            if pos.to_collect == 0 {
                if pos.coordinates == start {
                    return Some(pos.costs);
                } else {
                    let costs = pos.costs + network.get(&(pos.coordinates, start)).unwrap();
                    open_set.insert(Position { 
                        estimated_total_costs: costs,
                        costs,
                        coordinates: start, 
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
    
                let collected = all_herbs & !(pos.to_collect | (1 << pos.collecting));
                if let Some(remaining) = visited.get(&(pos.coordinates, collected)) {
                    open_set.insert(Position { 
                        estimated_total_costs: pos.costs + remaining,
                        costs: pos.costs + remaining,
                        coordinates: start,
                        to_collect: 0,
                        collecting: 0,
                    });
                } else {
                    (0..self.herbs.len())
                        .filter(|herb| pos.to_collect & (1_u16 << herb) != 0)
                        .for_each(|herb| {
                            let to_collect = pos.to_collect & !(1_u16 << herb);
                            self.herbs[herb]
                                .iter()
                                .for_each(|&coordinates| {
                                    let costs = pos.costs + network.get(&(pos.coordinates, coordinates)).unwrap();
                                    let estimated_total_costs = costs + (0..self.herbs.len())
                                        .filter(|other_herb| to_collect & (1_u16 << other_herb) != 0)
                                        .map(|other_herb| *estimate.get(&(coordinates, other_herb as u8)).unwrap())
                                        .max()
                                        .unwrap_or(0);
                   
                                    open_set.insert(Position {
                                        estimated_total_costs,
                                        costs,
                                        coordinates, 
                                        to_collect,
                                        collecting: herb as u8,
                                    });
                                });
                        });
                }
            }
       
        }
    
        None
    }
}

pub fn run(input: &str, part: usize) -> Result<u16, ParseError> {
    let map = Map::try_from(input)?;
    match part {
        1 => Ok(2 * map.route_single(0).unwrap()),
        2 => Ok(map.route_all_bfs().unwrap()),
        3 => Ok(map.route_all_a_star().unwrap()),
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
        let expected = [26, 38, 38];
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
