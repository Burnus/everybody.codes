use core::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    NoStars,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoStars => write!(f, "No Stars found in input. Stars should be represented by \'*\'."),
        }
    }
}

#[derive(Clone, Debug)]
struct Star {
    x: usize,
    y: usize,
}

impl Star {
    fn distance_to(&self, other: &Self) -> usize {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

#[derive(Clone, Debug)]
struct Constellation {
    stars: usize,
    distance: usize,
}

impl TryFrom<&str> for Constellation {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Initialize the stars without connections
        let stars: Vec<Star> = value
            .lines()
            .enumerate()
            .flat_map(|(y, line)| line
                .chars()
                .enumerate()
                .filter(|(_x, c)| *c == '*')
                .map(|(x, _c)| Star { x, y, })
                .collect::<Vec<_>>()
            ).collect();

        if stars.is_empty() {
            return Err(Self::Error::NoStars);
        }

        // Find connections using Prim's algorithm
        let mut distances = vec![vec![0; stars.len()]; stars.len()];
        stars.iter().enumerate().for_each(|(l_idx, l_star)| stars.iter().enumerate().skip(l_idx+1).for_each(|(r_idx, r_star)| {
            let distance = l_star.distance_to(r_star);
            distances[l_idx][r_idx] = distance;
            distances[r_idx][l_idx] = distance;
        }));
        let mut conncections = vec![Vec::new(); stars.len()];
        let (first_idx, dist) = distances[0].iter().enumerate().skip(1).min_by_key(|(_idx, dist)| *dist).unwrap();
        conncections[0].push(first_idx);
        conncections[first_idx].push(0);
        let mut distance = *dist;

        loop {
            let missing: Vec<_> = conncections.iter().enumerate().filter(|(_idx, conns)| conns.is_empty()).map(|(idx, _conns)| idx).collect();
            if missing.is_empty() {
                break;
            }
            let (next_l, (next_r, dist)) = missing
                .iter()
                .map(|l_idx| (*l_idx, distances[*l_idx]
                        .iter()
                        .enumerate()
                        .filter(|(r_idx, _dist)| missing.binary_search(r_idx).is_err())
                        .min_by_key(|(_r_idx, dist)| **dist)
                        .unwrap_or((0, &usize::MAX)))
                ).min_by_key(|(_l_idx, (_r_idx, dist))| **dist)
                .unwrap();
            conncections[next_l].push(next_r);
            conncections[next_r].push(next_l);
            distance += *dist;
        }

        Ok(Self { stars: stars.len(), distance, })
    }
}

impl Constellation {
    fn size(&self) -> usize {
        self.stars + self.distance
    }
}

struct BrilliantConstellations {
    constellations: Vec<Constellation>
}

impl TryFrom<&str> for BrilliantConstellations {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Initialize the stars without connections
        let stars: Vec<_> = value
            .lines()
            .enumerate()
            .flat_map(|(y, line)| line
                .chars()
                .enumerate()
                .filter(|(_x, c)| *c == '*')
                .map(|(x, _c)| Star { x, y, })
                .collect::<Vec<_>>()
            ).collect();

        if stars.is_empty() {
            return Err(Self::Error::NoStars);
        }

        // Find connections using Prim's algorithm
        let mut distances = vec![vec![0; stars.len()]; stars.len()];
        stars.iter().enumerate().for_each(|(l_idx, l_star)| stars.iter().enumerate().skip(l_idx+1).for_each(|(r_idx, r_star)| {
            let distance = l_star.distance_to(r_star);
            distances[l_idx][r_idx] = distance;
            distances[r_idx][l_idx] = distance;
        }));
        let mut constellations: Vec<(Vec<usize>, usize)> = Vec::new();
        for first_idx in 0..stars.len() {
            if constellations.iter().any(|(cons, _d)| cons.contains(&first_idx)) {
                continue;
            }
            if let Some((next_idx, dist)) = distances[first_idx]
                .iter()
                .enumerate()
                .filter(|(idx, dist)| **dist < 6 && 
                    *idx != first_idx && 
                    !constellations.iter().any(|(cons, _d)| cons.contains(idx)))
                .min_by_key(|(_idx, dist)| **dist) 
            {
                let mut this_cons = Vec::from([first_idx, next_idx]);
                let mut this_len = *dist;
                while let Some((other, dist)) = this_cons
                    .iter()
                    .flat_map(|seen| distances[*seen]
                        .iter()
                        .enumerate()
                        .filter(|(new, dist)| **dist < 6 &&
                            !this_cons.contains(new) &&
                            !constellations.iter().any(|(cons, _d)| cons.contains(new))
                        ))
                    .min_by_key(|(_idx, dist)| **dist)
                {
                    this_cons.push(other);
                    this_len += dist;
                }
                constellations.push((this_cons, this_len));

            } else {
                constellations.push((Vec::from([first_idx]), 0));
            }
        }
        Ok(Self { constellations: constellations.iter().map(|(stars, dist)| Constellation{ stars: stars.len(), distance: *dist }).collect(), })
    }
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseError> {
    // let items: Vec<_> = input.lines().map(::try_from).collect::<Result<Vec<_>, _>>()?;
    match part {
        1 | 2 => {
            let constellation = Constellation::try_from(input)?;
            Ok(constellation.size())
        },
        3 => {
            let brilliant_constellations = BrilliantConstellations::try_from(input)?;
            let mut sizes: Vec<_> = brilliant_constellations.constellations.iter().map(Constellation::size).collect();
            sizes.sort_by_key(|s| usize::MAX - s);
            Ok(sizes.iter().take(3).product())
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
        let expected = [16, 16, 15624];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [135, 1244, 3818228112];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
