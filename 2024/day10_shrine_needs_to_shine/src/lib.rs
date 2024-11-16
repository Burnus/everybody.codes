use core::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    GridMalformed(String),
    GridOfGridsMalformed,
    ParseCharError(char),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GridMalformed(e) => write!(f, "Grid must be 8*8 characters: {e}"),
            Self::GridOfGridsMalformed => write!(f, "All grid components must be of equal length"),
            Self::ParseCharError(e) => write!(f, "Unable to parse character {e}"),
        }
    }
}

#[derive(Clone)]
struct Grid {
    columns: Vec<Vec<char>>,
    rows: Vec<Vec<char>>,
    runic_word: Vec<Vec<char>>,
    solved: bool,
}

impl TryFrom<&str> for Grid {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lines: Vec<_> = value.lines().collect();
        if !lines.len() == 8 || lines.iter().any(|line| line.len() != 8) {
            return Err(Self::Error::GridMalformed(value.to_string()));
        }
        let rows = lines[2..6].iter().map(|line| line.chars().take(2).chain(line.chars().skip(6)).collect::<Vec<char>>()).collect();
        let columns = (2..6).map(|col| [0, 1, 6, 7].iter().map(|row| lines[*row].chars().nth(col).unwrap()).collect::<Vec<char>>()).collect();

        Ok(Self { columns, rows, runic_word: vec![vec!['.'; 4]; 4], solved: false, })
    }
}

impl Grid {
    fn fill(&mut self) {
        let mut solved = true;
        (0..4).for_each(|row| {
            (0..4).for_each(|col| {
                if let Some(c) = self.columns[col].iter().find(|c| **c != '?' && self.rows[row].contains(*c)) {
                    self.runic_word[row][col] = *c;
                } else {
                    solved = false;
                }
            });
        });
        self.solved = false;
    }
    
    fn runic_word(&mut self) -> String {
        if !self.solved {
            self.fill();
        }
        self.runic_word.iter().flatten().collect()
    }

    fn effective_power(&mut self) -> usize {
        self.runic_word().chars().enumerate().map(|(idx, c)| (idx+1)*(c as usize - b'@' as usize)).sum()
    }
}

fn into_grids_split(input: &str) -> Result<Vec<Vec<String>>, ParseError> {
    let rows: Vec<_> = input.split("\n\n").collect();
    rows
        .iter()
        .map(|row| {
            let lines: Vec<&str> = row.lines().collect();
            let cells: Vec<Vec<&str>> = lines
                .iter()
                .map(|s| s.split(' ')
                .collect::<Vec<&str>>())
                .collect();
            (0..cells[0].len())
                .map(|idx| cells
                    .iter()
                    .map(|line| line.get(idx).copied().ok_or(ParseError::GridOfGridsMalformed))
                    .collect::<Result<Vec<&str>, _>>()
                    .map(|s| s.join("\n"))
                ).collect::<Result<Vec<String>, _>>()
        }).collect::<Result<Vec<_>, ParseError>>()
}

fn into_grids_shared(input: &str) -> Result<Vec<Vec<String>>, ParseError> {
    let lines: Vec<_> = input.lines().collect();
    (0..lines.len()-7)
        .step_by(6)
        .map(|first_row| {
            if lines[first_row].len() < 8 {
                Err(ParseError::GridOfGridsMalformed)
            } else {
                Ok((0..lines[first_row].len()-7)
                    .step_by(6)
                    .map(|first_col| {
                        lines[first_row..first_row+8]
                            .iter()
                            .map(|line| line.chars().skip(first_col).take(8).collect::<String>())
                            .collect::<Vec<String>>()
                            .join("\n")
                    }).collect::<Vec<String>>())
            }
        }).collect::<Result<Vec<_>, _>>()
}

fn into_grids(input: &str) -> Result<Vec<Vec<String>>, ParseError> {
    match input.lines().nth(8) {
        None => Ok(vec![vec![input.to_string()]]),
        Some("") => into_grids_split(input),
        _ => into_grids_shared(input),
    }
}
fn solve_grids(grids: &mut [Vec<Grid>]) -> bool {
    let mut any_solved = false;
    for grid_y in 0..grids.len() {
        for grid_x in 0..grids[grid_y].len() {
            if grids[grid_y][grid_x].solved {
                continue;
            }
            let mut this_grid = grids[grid_y][grid_x].clone();
            this_grid.fill();
            if !this_grid.solved {
                let open: Vec<(usize, usize)> = this_grid.runic_word
                    .iter()
                    .enumerate()
                    .flat_map(|(row, row_vec)| row_vec
                        .iter()
                        .enumerate()
                        .filter(|(_col, c)| **c == '.')
                        .map(|(col, _c)| (row, col))
                        .collect::<Vec<(usize, usize)>>()
                    ).collect();
                let mut to_solve = open.len();
                for (y, x) in open {
                    let row: Vec<char> = this_grid.rows[y].to_vec();
                    let col: Vec<char> = this_grid.columns[x].to_vec();
                    if !this_grid.rows[y].contains(&'?') {
                        let to_find: Vec<usize> = col.iter().enumerate().filter(|(_idx, c)| **c == '?').map(|(idx, _c)| idx).collect();
                        if to_find.len() == 1 {
                            let col_idx = to_find[0];
                            let to_insert: Vec<char> = row.iter().filter(|c| !this_grid.runic_word[y].contains(c)).cloned().collect();
                            if to_insert.len() == 1 {
                                let found = to_insert[0];
                                this_grid.runic_word[y][x] = found;
                                this_grid.columns[x][col_idx] = found;
                                match col_idx {
                                    0 | 1 if grid_y > 0 => grids[grid_y-1][grid_x].columns[x][col_idx+2] = found,
                                    2 | 3 if grid_y < grids.len()-1 => grids[grid_y+1][grid_x].columns[x][col_idx-2] = found,
                                    _ => (),
                                }
                                to_solve -= 1;
                            }
                        }
                    } else if !col.contains(&'?') {
                        let to_find: Vec<usize> = row.iter().enumerate().filter(|(_idx, c)| **c == '?').map(|(idx, _c)| idx).collect();
                        if to_find.len() == 1 {
                            let row_idx = to_find[0];
                            let to_insert: Vec<char> = col.iter().filter(|c| !this_grid.runic_word.iter().any(|rw_row| rw_row[x] == **c)).cloned().collect();
                            if to_insert.len() == 1 {
                                let found = to_insert[0];
                                this_grid.runic_word[y][x] = found;
                                this_grid.rows[y][row_idx] = found;
                                match row_idx {
                                    0 | 1 if grid_x > 0 => grids[grid_y][grid_x-1].rows[y][row_idx+2] = found,
                                    2 | 3 if grid_x < grids[grid_y].len()-1 => grids[grid_y][grid_x+1].rows[y][row_idx-2] = found,
                                    _ => (),
                                }
                                to_solve -= 1;
                            }
                        }
                    }
                }
                if to_solve == 0 {
                    this_grid.solved = true;
                    any_solved = true;
                }
                grids[grid_y][grid_x] = this_grid;
            } else {
                grids[grid_y][grid_x].runic_word = this_grid.runic_word;
            }
        }
    }
    any_solved
}

pub fn run(input: &str, part: usize) -> Result<String, ParseError> {
    let mut grids = into_grids(input)?.iter().map(|row| row.iter().map(|g| Grid::try_from(&g[..])).collect::<Result<Vec<_>, _>>()).collect::<Result<Vec<Vec<_>>, _>>()?;
    match part {
        1 => Ok(grids[0][0].runic_word()),
        2 => Ok(format!("{}", grids.iter_mut().flatten().map(|g| g.effective_power()).sum::<usize>())),
        3 => {
            while solve_grids(&mut grids) {}
            Ok(format!("{}", grids.iter_mut().flatten().filter(|g| g.solved).map(|g| g.effective_power()).sum::<usize>()))
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
        let expected = ["PTBVRCZHFLJWGMNS", "1851", "3889"];
        for part in 1..=expected.len() {
            let sample_input = read_file(&format!("tests/sample{part}"));
            assert_eq!(run(&sample_input, part), Ok(expected[part-1].to_string()));
        }
    }

    #[test]
    fn test_challenge() {
        let expected = ["LWZNDSRFXMPTQVBG", "197973", "213343"];
        for part in 1..=expected.len() {
            let challenge_input = read_file(&format!("tests/challenge{part}"));
            assert_eq!(run(&challenge_input, part), Ok(expected[part-1].to_string()));
        }
    }
}
