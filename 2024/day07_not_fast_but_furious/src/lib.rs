use core::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    InputMalformed(String),
    LineMalformed(String),
    ParseActionError(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputMalformed(e) => write!(f, "Unable to parse malformed input: {e}\n\nShould be only the device lines for part 1, or device lines, an empty line and the racetrack otherwise."),
            Self::LineMalformed(e) => write!(f, "Unable to parse malformed line: {e}\nShould be of format:\nA:+,-,="),
            Self::ParseActionError(e) => write!(f, "Unable to parse {e} into an action"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Action{ Increase, Decrease, Remain }

impl TryFrom<&str> for Action {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Self::Increase),
            "-" => Ok(Self::Decrease),
            "=" | "S" => Ok(Self::Remain),
            e => Err(Self::Error::ParseActionError(e.to_string())),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Plan {
    essence: usize,
    power: usize,
    name: String,
    actions: Vec<Action>,
}

impl TryFrom<&str> for Plan {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some((name, actions)) = value.split_once(':') {
            let actions: Vec<_> = actions.split(',').map(Action::try_from).collect::<Result<Vec<_>, _>>()?;
            Ok(Self { essence: 0, power: 10, name: name.to_string(), actions })
        } else {
            Err(Self::Error::LineMalformed(value.to_string()))
        }
    }
}

impl Plan {
    fn execute(&mut self, action: Action) {
        match action {
            Action::Increase => self.power += 1,
            Action::Decrease => self.power = self.power.saturating_sub(1),
            Action::Remain => (),
        }
        self.essence += self.power;
    }
}

fn parse_track(input: &str) -> String {
    let mut res = String::new();
    let chars: Vec<Vec<char>> = input.lines().map(|line| line.chars().collect()).collect();
    let mut last_pos = (0, 0);
    let (mut x, mut y) = (1, 0);

    while (x, y) != (0, 0) {
        res.push(chars[y][x]);
        let neighbours: Vec<_> = [(1, 2), (2, 1), (1, 0), (0, 1)]
            .iter()
            .filter(|(dx, dy)| 
                x+dx > 0 && 
                y+dy > 0 && 
                y+dy < chars.len()+1 && 
                x+dx < chars[y+dy-1].len()+1 &&
                last_pos != (x+dx-1, y+dy-1))
            .map(|(dx, dy)| (x+dx-1, y+dy-1))
            .collect();
        last_pos = (x, y);
        (x, y) = *neighbours
            .iter()
            .find(|(x, y)| chars[*y][*x] != ' ')
            .unwrap();
    }
    res.push('S');

    res
}

fn race(track: &[Action], plans: &mut [Plan], rounds: usize) {
    let track_len = track.len();
    plans.iter_mut().for_each(|plan| {
        (0..rounds).for_each(|round|
            (0..track_len).for_each(|segment| {
                let action = match track[segment] {
                    Action::Increase => Action::Increase,
                    Action::Decrease => Action::Decrease,
                    _ => plan.actions[(round * track_len + segment) % plan.actions.len()],
                };
                plan.execute(action);
            }));
        });
}

fn construct_actions(inc_count: usize, dec_count: usize, rem_count: usize) -> Vec<Vec<Action>> {
    if inc_count == 0 && dec_count == 0 && rem_count == 0 {
        return vec![vec![]];
    }
    let mut res = Vec::new();
    if inc_count > 0 {
        let a = construct_actions(inc_count-1, dec_count, rem_count);
        a.iter().for_each(|actions| {
            let mut new = actions.clone();
            new.push(Action::Increase);
            res.push(new);
        });
    }
    if dec_count > 0 {
        let a = construct_actions(inc_count, dec_count-1, rem_count);
        a.iter().for_each(|actions| {
            let mut new = actions.clone();
            new.push(Action::Decrease);
            res.push(new);
        });
    }
    if rem_count > 0 {
        let a = construct_actions(inc_count, dec_count, rem_count-1);
        a.iter().for_each(|actions| {
            let mut new = actions.clone();
            new.push(Action::Remain);
            res.push(new);
        });
    }
    res
}

fn construct_plans(inc_count: usize, dec_count: usize, rem_count: usize) -> Vec<Plan> {
    let actions = construct_actions(inc_count, dec_count, rem_count);
    actions.iter().map(|a| Plan { essence: 0, power: 10, name: String::new(), actions: a.clone() }).collect()
}

pub fn run(input: &str, part: usize) -> Result<String, ParseError> {
    let components: Vec<_> = input.split("\n\n").collect();
    let track: Vec<Action> = match (components.len(), part) {
        (1, 1) => Vec::from([Action::Remain]),
        (2, 2) | (2, 3) => parse_track(components[1]).chars().map(|c| Action::try_from(&c.to_string()[..])).collect::<Result<Vec<_>, _>>()?,
        _ => return Err(ParseError::InputMalformed(input.to_string())),
    };
    let mut plans: Vec<_> = components[0].lines().map(Plan::try_from).collect::<Result<Vec<_>, _>>()?;
    match part {
        1 => {
            // There is no track for part 1, but this is equivalent to a track consisting only of
            // one Remain action, hence we hand over such a track to our generalized function.
            race(&track, &mut plans, 10);
            plans.sort_by(|a, b| b.cmp(a));
            Ok(plans.iter().map(|plan| plan.name.clone()).collect())
        },
        2 => {
            race(&track, &mut plans, 10);
            plans.sort_by(|a, b| b.cmp(a));
            Ok(plans.iter().map(|plan| plan.name.clone()).collect())
        },
        3 => {
            // Everything must repeat after 11 laps, since this is the length of our action plan.
            // So we know the ordering must be the same after every 11th lap. Since 2024 devides
            // 11, the ordering after lap 2024 must be the same as after lap 11. Therefore we only
            // need to run the simulation for 11 laps.
            race(&track, &mut plans, 11);
            let opponent_essence = plans[0].essence;
            let mut my_plans = construct_plans(5, 3, 3);
            race(&track, &mut my_plans, 11);
            Ok(format!("{}", my_plans.iter().filter(|plan| plan.essence > opponent_essence).count()))
        },
        _ => unreachable!(),
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
    fn test_parse_track() {
        let tracks = [
"S+===
-   +
=+=-+",
"S-=++=-==++=++=-=+=-=+=+=--=-=++=-==++=-+=-=+=-=+=+=++=-+==++=++=-=-=--
-                                                                     -
=                                                                     =
+                                                                     +
=                                                                     +
+                                                                     =
=                                                                     =
-                                                                     -
--==++++==+=+++-=+=-=+=-+-=+-=+-=+=-=+=--=+++=++=+++==++==--=+=++==+++-",
        ];
        let expected = [
"+===++-=+=-S",
"-=++=-==++=++=-=+=-=+=+=--=-=++=-==++=-+=-=+=-=+=+=++=-+==++=++=-=-=---=++==--+++==++=+=--==++==+++=++=+++=--=+=-=+=-+=-+=-+-=+=-=+=-+++=+==++++==---=+=+=-S",
        ];
        for (idx, track) in tracks.iter().enumerate() {
            assert_eq!(parse_track(track), expected[idx].to_string());
        }
    }

    #[test]
    fn test_sample() {
        let expected = ["BDCA", "DCBA"];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1].to_string()));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = ["GKDIHBEJC", "EIKDGJFAC", "4060"];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1].to_string()));
        }
    }
}
