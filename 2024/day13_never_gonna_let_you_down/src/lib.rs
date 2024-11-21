use core::fmt::Display;
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    NoEndError,
    NoStartError,
    ParseCharError(char),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoEndError => write!(f, "Unable to find an end tile"),
            Self::NoStartError => write!(f, "Unable to find a start tile"),
            Self::ParseCharError(e) => write!(f, "Unable to parse tile: {e}"),
        }
    }
}

type Coordinates = (usize, usize);

/// Try to parse the input into:
/// * a `HashMap` from `Coordinates` to their levels,
/// * a `Vec` containing all starting points as `Coordinates`, and
/// * the `Coordinates` of the end point.
///
/// The starting and end points are included in the `HashMap` and set to level 0 (as per the
/// challenge description).
fn try_parse(input: &str) -> Result<(HashMap<Coordinates, usize>, Vec<Coordinates>, Coordinates), ParseError> {
    let mut platforms = HashMap::new();
    let mut start = Vec::new();
    let mut end = None;

    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            match c {
                'S' => {
                    start.push((x, y));
                    platforms.insert((x, y), 0);
                },
                'E' => {
                    end = Some((x, y));
                    platforms.insert((x, y), 0);
                },
                n if n.is_ascii_digit() => _ = platforms.insert((x, y), n as usize - b'0' as usize),
                '#' | ' ' => (),
                e => return Err(ParseError::ParseCharError(e)),
            }
        }
    }
    if !start.is_empty() {
        if let Some(end) = end {
            Ok((platforms, start, end))
        } else {
            Err(ParseError::NoEndError)
        }
    } else {
        Err(ParseError::NoStartError)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Position {
    costs_so_far: usize,
    coordinates: Coordinates,
}

/// Use the Dijkstra algorithm to find the shortest path between the end and any starting point
/// and returns the length of the path. Panics if no such path exists.
fn shortest_path(platforms: &HashMap<Coordinates, usize>, start: Vec<Coordinates>, end: Coordinates) -> usize {
    let level_delta = |lhs: Coordinates, rhs: Coordinates| -> usize {
        let left = *platforms.get(&lhs).unwrap();
        let right = *platforms.get(&rhs).unwrap();
        left.abs_diff(right).min(left.abs_diff(right+10)).min(right.abs_diff(left+10))
    };
    let mut open_set = BTreeSet::from([Position{ costs_so_far: 0, coordinates: end, }]);
    let mut visited = HashSet::new();
    while let Some(pos) = open_set.pop_first() {
        let curr_coords = pos.coordinates;
        if start.contains(&curr_coords) {
            return pos.costs_so_far;
        }
        if !visited.contains(&curr_coords) {
            visited.insert(curr_coords);
            [(0,1), (1,0), (2,1), (1,2)]
                .iter()
                .filter(|(dx, dy)| {
                    curr_coords.0 + dx > 0 && curr_coords.1 + dy > 0 &&
                    !visited.contains(&(curr_coords.0+dx-1, curr_coords.1+dy-1))
                }).for_each(|(dx, dy)| {
                    let coordinates = (curr_coords.0 + dx - 1, curr_coords.1 + dy - 1);
                    if platforms.contains_key(&coordinates) {
                        let costs_so_far = pos.costs_so_far + level_delta(curr_coords, coordinates) + 1;
                        open_set.insert(Position { costs_so_far, coordinates });
                    }
                });
        }
    }
    panic!("No path found");
}

// The part number is irrelevant for this quest, since the Dijkstra function can handle all of them
// in the same way. However, the parameter is kept for consistency with the other quests.
pub fn run(input: &str, _part: usize) -> Result<usize, ParseError> {
    let (platforms, start, end) = try_parse(input)?;
    Ok(shortest_path(&platforms, start, end))
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
        let expected = [28, 28, 14];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [165, 608, 539];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
