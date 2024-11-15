use std::collections::{BTreeSet, HashMap};
use std::mem;
use std::num::ParseIntError;

// This struct serves only to sort the open set in our A* algorithm.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Brightness {
    expected: usize,
    remaining: usize,
    cost: usize,
    last_step: usize,
}

// A* algorithm with memoization
fn required_stamps(brightness: usize, stamps: &[usize], mem: &mut HashMap<usize, usize>) -> usize {
    let mut path = HashMap::new();
    let mut open_set = BTreeSet::from([Brightness {
        expected: brightness.div_ceil(*stamps.iter().find(|b| **b <= brightness).unwrap()), 
        remaining: brightness, 
        cost: 0,
        last_step: 0,
    }]);
    while let Some(current) = open_set.pop_first() {
        if current.remaining == 0 {
            mem.insert(brightness, current.cost);
            let mut prev_remaining = current.last_step;
            let mut prev_cost = 1;
            while prev_remaining < brightness {
                if let Some(cost) = mem.get(&prev_remaining) {
                    prev_cost = *cost + 1;
                } else {
                    mem.insert(prev_remaining, prev_cost);
                    prev_cost += 1;
                }
                prev_remaining = *path.get(&prev_remaining).unwrap();
            }
            return current.cost;
        }
        path.entry(current.remaining).or_insert_with(|| {
            if let Some(cost) = mem.get(&current.remaining) {
                open_set.insert(Brightness { expected: current.cost + *cost, remaining: 0, cost: current.cost + *cost, last_step: current.remaining, });
            } else {
                for stamp in stamps.iter().filter(|&s| *s <= current.remaining) {
                    let remaining = current.remaining - stamp;
                    let expected = current.cost + if remaining > 0 {
                        remaining.div_ceil(*stamps.iter().find(|b| **b <= remaining).unwrap())
                    } else { 1 };
                    open_set.insert(Brightness { expected, remaining, cost: current.cost + 1, last_step: *stamp, });
                }
            }
            current.remaining + current.last_step
        });
    }
    0
}

fn required_stamps_split(brightness: usize, stamps: &[usize], mem: &mut HashMap<usize, usize>) -> usize {
    // Since the brightnesses must not differ by more than 100, we know that they must be
    // `brightness/2+delta` and `brightness` minus that respectively, for some `delta` within [0..=50].
    // We try all such pairs and return the lowest combined costs we find.
    // Thanks to memoization, this becomes cheaper for later calls and is free for the second call
    // at delta=0.
    (0..=50).map(|delta| 
        required_stamps(brightness/2+delta, stamps, mem) + 
        required_stamps(brightness-(brightness/2+delta), stamps, mem)
        ).min().unwrap()
}

pub fn run(input: &str, part: usize) -> Result<usize, ParseIntError> {
    let brightnesses: Vec<_> = input.lines().map(|n| n.parse()).collect::<Result<Vec<usize>, _>>()?;
    match part {
        1 => {
            const STAMPS: [usize; 4] = [10, 5, 3, 1];
            let mut mem = STAMPS.iter().map(|s| (*s, 1)).collect();
            Ok(brightnesses.iter().map(|b| required_stamps(*b, &STAMPS, &mut mem)).sum())
        },
        2 => {
            const STAMPS: [usize; 10] = [30, 25, 24, 20, 16, 15, 10, 5, 3, 1];
            let mut mem = STAMPS.iter().map(|s| (*s, 1)).collect();
            Ok(brightnesses.iter().map(|b| required_stamps(*b, &STAMPS, &mut mem)).sum())
        },
        3 => {
            const STAMPS: [usize; 18] = [101, 100, 75, 74, 50, 49, 38, 37, 30, 25, 24, 20, 16, 15, 10, 5, 3, 1];
            const ROUNDS: usize = 5;
            let mut mem: HashMap<usize, usize> = STAMPS.iter().map(|s| (*s, 1)).collect();

            // Precondition the memory to speed up the lookups a bit. This isn't worth the effort
            // for the earlier parts, but here it saves a few seconds.
            let mut last = STAMPS.to_vec();
            last.reserve(STAMPS.len().pow(ROUNDS as u32 - 1));
            let mut next = Vec::with_capacity(STAMPS.len().pow(ROUNDS as u32));
            for round in 2..=ROUNDS+1 {
                next.clear();
                last.iter().for_each(|b| STAMPS.iter().for_each(|s| {
                    mem.entry(b+s).or_insert_with(|| {
                        next.push(b+s);
                        round
                    });
                }));
                mem::swap(&mut last, &mut next);
            }
            Ok(brightnesses.iter().map(|b| required_stamps_split(*b, &STAMPS, &mut mem)).sum())
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
        let expected = [10, 10, 10449];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [13348, 5108, 150481];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
