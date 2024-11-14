// use core::fmt::Display;
use std::num::ParseIntError;

// #[derive(Debug, PartialEq, Eq)]
// pub enum ParseError<'a> {
//     LineMalformed(&'a str),
//     ParseIntError(&'a str),
// }
//
// impl Display for ParseError<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::LineMalformed(e) => write!(f, "Unable to parse malformed line: {e}\nShould be of format:\n"),
//             Self::ParseIntError(e) => write!(f, "Unable to parse into a number: {e}"),
//         }
//     }
// }
//
// impl TryFrom<&str> for  {
//     type Error = ParseError;
//
//     fn try_from(value: char) -> Result<Self, Self::Error> {
//     }
// }

fn pyramid_height(blocks: usize) -> usize {
    ((blocks.saturating_sub(1)) as f64).sqrt() as usize + 1
}

fn pyramid_blocks(height: usize) -> usize {
    height * height
}

fn construct_hollow_shrine(priests: usize, acolytes: usize, available_blocks: usize) -> usize {
    let mut thicknesses = vec![1];
    loop {
        let thickness = (thicknesses.last().unwrap() * priests ) % acolytes + acolytes;
        thicknesses.push(thickness);
        let mut columns = Vec::with_capacity(thicknesses.len());
        let width = thicknesses.len() * 2 - 1;
        let mut height = 0;
        thicknesses.iter().rev().for_each(|thickness| {
            height += thickness;
            columns.push(height);
        });
        let mut empty = Vec::with_capacity(thicknesses.len());
        empty.push(0);
        columns.iter().enumerate().skip(1).for_each(|(idx, column)| {
            empty.push((columns[idx-1] - 1).min( (priests * width * column) % acolytes ));
        });
        let total = columns.iter().sum::<usize>() * 2 - columns.last().unwrap() - empty.iter().sum::<usize>() * 2 + empty.last().unwrap();
        if total >= available_blocks {
            return total - available_blocks;
        }
    }
}

fn construct_shrine(priests: usize, acolytes: usize, available_blocks: usize) -> (usize, usize) {
    let mut remaining = available_blocks - 1;
    let mut last_thickness = 1;
    let mut layer = 2;
    loop {
        let thickness = (last_thickness * priests ) % acolytes;
        last_thickness = thickness;
        let required = (layer * 2 - 1) * thickness;
        if remaining <= required {
            return (required - remaining, layer * 2 - 1);
        }
        remaining -= required;
        layer += 1;
    }
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseIntError> {
    let items: usize = input.parse()?;
    match part {
        1 => {
            let height = pyramid_height(items);
            let missing = pyramid_blocks(height) - items;
            let width = 2 * height - 1;
            Ok(missing * width)
        },
        2 => {
            let (acolytes, available_blocks) = if items < 20 { (5, 50) } else { (1111, 20240000) };
            let (missing, width) = construct_shrine(items, acolytes, available_blocks);
            Ok(missing * width)
        },
        3 => {
            let (acolytes, available_blocks) = if items < 20 { (5, 160) } else { (10, 202400000) };
            Ok(construct_hollow_shrine(items, acolytes, available_blocks))
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
    fn test_hollow() {
        let expected = [18, 66, 114, 161, 238, 352, 490, 568, 689, 1884, 7600, 30654, 123130, 491004, 1964800, 7863294, 31461370, 125820924];
        for blocks in expected {
            assert_eq!(construct_hollow_shrine(2, 5, blocks), 1);
        }
    }

    #[test]
    fn test_sample() {
        let expected = [21, 27, 2];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [9758090, 142569157, 41082];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
