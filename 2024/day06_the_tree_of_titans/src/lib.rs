use core::fmt::Display;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    ChildDuplicate(String),
    LineMalformed(String),
    ParentUnknown(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChildDuplicate(e) => write!(f, "Trying to add child node {e} for the second time"),
            Self::LineMalformed(e) => write!(f, "Unable to parse malformed line: {e}\nShould be of format:\nA:B,C,D"),
            Self::ParentUnknown(e) => write!(f, "Trying to add children for unknown parent {e}"),
        }
    }
}

struct Node {
    depth: usize,
    is_fruit: bool,
    parent: String,
}

fn try_build_tree(input: &str) -> Result<(HashMap<String, usize>, Vec<Node>), ParseError> {
    const PESTS: [&str; 2] = ["ANT", "BUG"];
    let mut names = HashMap::from([("RR".to_string(), 0)]);
    let mut nodes = vec![Node{ depth: 0, is_fruit: false, parent: String::new()}];
    let mut parent_unknown = String::new();
    for l in input.lines() {
        if let Some((parent, children)) = l.split_once(':') {
            if !PESTS.contains(&parent) {
                if let Some(parent_id) = names.get(parent) {
                    let children: Vec<_> = children.split(',').filter(|c| !PESTS.contains(c)).map(|c| c.to_string()).collect();
                    let parent_depth = nodes[*parent_id].depth;
                    for child in children {
                        if child != "@" && names.contains_key(&child) {
                            return Err(ParseError::ChildDuplicate(child.to_string()));
                        }
                        let child_id = nodes.len();
                        names.insert(child.clone(), child_id);
                        nodes.push(Node { depth: parent_depth+1, is_fruit: child == "@", parent: parent.to_string(), });
                    }
                } else {
                    parent_unknown += l;
                    parent_unknown += "\n";
                }
            }
        } else {
            return Err(ParseError::LineMalformed(l.to_string()));
        }
    }
    while !parent_unknown.is_empty() {
        let mut next_parent_unknown = String::new();
        for l in parent_unknown.lines() {
            if let Some((parent, children)) = l.split_once(':') {
                if !PESTS.contains(&parent) {
                    if let Some(parent_id) = names.get(parent) {
                        let children: Vec<_> = children.split(',').filter(|c| !PESTS.contains(c)).map(|c| c.to_string()).collect();
                        let parent_depth = nodes[*parent_id].depth;
                        for child in children {
                            if child != "@" && names.contains_key(&child) {
                                return Err(ParseError::ChildDuplicate(child.to_string()));
                            }
                            let child_id = nodes.len();
                            names.insert(child.clone(), child_id);
                            nodes.push(Node { depth: parent_depth+1, is_fruit: child == "@", parent: parent.to_string(), });
                        }
                    } else {
                        next_parent_unknown += l;
                        next_parent_unknown += "\n";
                    }
                }
            } else {
                return Err(ParseError::LineMalformed(l.to_string()));
            }
        }
        std::mem::swap(&mut parent_unknown, &mut next_parent_unknown);
    }
    Ok((names, nodes))
}

fn name(original: &str, full_name: bool) -> String {
    if full_name {
        original.to_string()
    } else {
        original[..1].to_string()
    }
}

fn find_unique(names: &HashMap<String, usize>,nodes: &[Node], full_name: bool) -> String {
    let mut lengths = HashMap::new();
    nodes.iter().filter(|n| n.is_fruit).for_each(|n| {
        lengths.entry(n.depth).and_modify(|(count, _first)| *count += 1).or_insert((1, n.parent.clone()));
    });
    eprintln!("{lengths:?}");
    if let Some((_length, (_count, parent))) = lengths.iter().find(|(_, (count, _))| *count == 1) {
        let mut res = format!("{}@", name(parent, full_name));
        let mut parent_id = names.get(parent).unwrap();
        let mut parent = nodes[*parent_id].parent.clone();
        while !parent.is_empty() {
            res = name(&parent, full_name) + &res;
            parent_id = names.get(&parent).unwrap();
            parent = nodes[*parent_id].parent.clone();
        }
        return res;
    }
    String::new()
}

pub fn run(input: &str, part: usize) -> Result<String, ParseError> {
    let (names, nodes) = try_build_tree(input)?;
    match part {
        1 => Ok(find_unique(&names, &nodes, true)),
        2 | 3 => Ok(find_unique(&names, &nodes, false)),
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
        let expected = ["RRB@", "RB@", "RB@"];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1].to_string()));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = ["RRHXFGVZKRSH@", "RKRJFMNFBN@", "RQSBWPVSQTHJ@"];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1].to_string()));
        }
    }
}
