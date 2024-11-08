use core::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError<'a> {
    LineMalformed(&'a str),
    NoteMalformed(&'a str),
}

impl Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LineMalformed(e) => write!(f, "Unable to parse malformed line: {e}\nShould be of format:\nWORDS:FOO,BAR,BAZ"),
            Self::NoteMalformed(e) => write!(f, "Unable to parse malformed note: {e}\nShould be of format:\nWORDS:FOO,BAR;BAZ\n\nBLA FASEL FOO"),
        }
    }
}

fn reverse_str(s: &str) -> String {
    s.chars().rev().collect::<String>()
}

fn find_matches(word: &str, runic: &[String]) -> Vec<bool> {
    let mut matches = vec![false; word.len()];
    runic.iter().for_each(|r| {
        let mut start = 0;
        while let Some(match_idx) = word[start..].find(r) {
            (start+match_idx..start+match_idx+r.len()).for_each(|idx| matches[idx] = true);
            start += match_idx + 1;
        }
    });
    matches
}

fn count_symbols(word: &str, runic: &[String]) -> usize {
    find_matches(word, runic).iter().filter(|i| **i).count()
}

fn count_symbols_wrapping(words: &[&str], runic: &[String]) -> usize {
    let width = words[0].len();
    let height = words.len();
    let mut matches = vec![vec![false; width]; height];

    // horizontal
    words.iter().enumerate().for_each(|(y, w)| {
        let word = format!("{w}{w}{w}");
        find_matches(&word, runic).iter().enumerate().skip(width).take(width).for_each(|(x, m)| if *m { matches[y][x-width] = true });
    });

    // vertical
    (0..width).for_each(|x| {
        let word = words.iter().map(|w| w.chars().nth(x).unwrap()).collect::<String>();
        find_matches(&word, runic).iter().enumerate().for_each(|(y, m)| if *m { matches[y][x] = true });
    });

    matches.iter().map(|line| line.iter().filter(|m| **m).count()).sum()
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseError> {
    let lines: Vec<_> = input.lines().collect::<Vec<_>>();
    if lines.len() < 3 {
        return Err(ParseError::NoteMalformed(input));
    }
    if let Some((_, words)) = lines[0].split_once(':') {
        let words = words.split(',').collect::<Vec<_>>();
        let mut words_omni = Vec::new();
        words.iter().for_each(|w| {
            words_omni.push(w.to_string());
            words_omni.push(reverse_str(w));
        });
        let inscription = lines.iter().skip(2).flat_map(|l| l.split_whitespace().collect::<Vec<_>>()).collect::<Vec<_>>();
        match part {
            1 => Ok(inscription.iter().map(|w| words.iter().map(|word| w.matches(*word).count()).sum::<usize>()).sum()),
            2 => Ok(inscription.iter().map(|w| count_symbols(w, &words_omni)).sum()),
            3 => Ok(count_symbols_wrapping(&inscription, &words_omni)),
            _ => panic!("Illegal part number"),
        }
    } else {
        Err(ParseError::LineMalformed(lines[0]))
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
        let expected = [4, 37, 10];
        for part in 1..=3 {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [34, 5078, 11593];
        for part in 1..=3 {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
