fn strikes_to_align(components: &[isize]) -> isize {
    components.iter().sum::<isize>() - components.len() as isize * *components.iter().min().unwrap_or(&0)
}

fn strikes_to_align_omni(components: &[isize]) -> isize {
    let sum: isize = components.iter().sum();
    let len = components.len() as isize;
    let mut estimate = (sum + len/2) / len;
    
    let mut y0: isize = components.iter().map(|c| c.abs_diff(estimate) as isize).sum();
    let mut y1: isize = components.iter().map(|c| c.abs_diff(estimate+1) as isize).sum();
    let y_m1: isize = components.iter().map(|c| c.abs_diff(estimate-1) as isize).sum();
    let step;

    if y1 < y0 {
        step = 1;
    } else if y_m1 < y0 {
        step = -1;
        y1 = y_m1;
    } else {
        return y0;
    }

    loop {
        y0 = y1;
        estimate += step;
        y1 = components.iter().map(|c| c.abs_diff(estimate+step) as isize).sum();
        if y1 > y0 {
            return y0;
        }
    }
}

pub fn run(input: &str, part: usize) -> Result<isize, std::num::ParseIntError> {
    let nails: Vec<_> = input.lines().map(|l| l.parse::<isize>()).collect::<Result<Vec<_>, _>>()?;
    match part {
        1 | 2 => Ok(strikes_to_align(&nails)),
        3 => Ok(strikes_to_align_omni(&nails)),
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
        let expected = [10, 10, 8];
        for part in 1..=3 {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1]));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = [64, 815526, 120375970];
        for part in 1..=3 {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1]));
        }
    }
}
