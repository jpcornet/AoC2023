use exrunner::ExRunner;
use std::io::BufRead;
use phf::phf_map;

struct Pipemaze {
    field: Vec<Vec<u8>>,
    startpos: (usize, usize),
}

fn parse(input: impl BufRead) -> Pipemaze {
    let field: Vec<Vec<_>> = input.lines().map(|l| {
        let line = l.expect("Error reading input");
        line.as_bytes().to_vec()
    }).collect();
    let mut findstart = field.iter().enumerate().filter_map(|(y, l)| {
        l.iter().enumerate().filter_map(|(x, c)| {
            if *c == 'S' as u8 {
                Some((x, y))
            } else {
                None
            }
        }).next()
    });
    let startpos = findstart.next().expect("No S startpos in input");
    if findstart.next().is_some() {
        panic!("Too many startpos S in input");
    }
    Pipemaze{ field, startpos }
}

type DIR = u8;
const NORTH: DIR = 1;
const WEST: DIR = 2;
const SOUTH: DIR = 4;
const EAST: DIR = 8;

// PIPES is used as the lookup table for character -> pipe directions.
static PIPES: phf::Map<char, DIR> = phf_map! {
    '|' => NORTH | SOUTH,
    '-' => EAST | WEST,
    'L' => NORTH | EAST,
    'J' => NORTH | WEST,
    '7' => SOUTH | WEST,
    'F' => SOUTH | EAST,
};

// mirror a direction
fn mirror_dir(dir: DIR) -> DIR {
    if dir & (NORTH|SOUTH) != 0 {
        dir ^ (NORTH|SOUTH)
    } else {
        dir ^ (EAST|WEST)
    }
}

// Walk the pipe for 1 step. Returns the new position and direction, or None if not possible
fn walk_pipe(pm: &Pipemaze, pos: (usize, usize), dir: DIR) -> Option<((usize, usize), DIR)> {
    let (dx, dy) = if dir == NORTH {
        (0, -1)
    } else if dir == WEST {
        (-1, 0)
    } else if dir == SOUTH {
        (0, 1)
    } else if dir == EAST {
        (1, 0)
    } else {
        panic!("Unknown direction {} at position ({},{})", dir, pos.0, pos.1);
    };
    if pos.0 == 0 && dx == -1 {
        return None;
    }
    if pos.1 == 0 && dy == -1 {
        return None;
    }
    if pos.1 as i32 + dy >= pm.field.len() as i32 {
        return None;
    }
    let newy = (pos.1 as i32 + dy) as usize;
    if pos.0 as i32 + dx >= pm.field[newy].len() as i32 {
        return None;
    }
    let newx = (pos.0 as i32 + dx) as usize;
    let pchar = pm.field[newy][newx] as char;
    // println!("Walk in direction {dir} from {},{} to {newx},{newy}. Now at {pchar}", pos.0, pos.1);
    if pchar == 'S' {
        // We've reached the starting position again
        return Some(((newx, newy), 0 as DIR));
    } else if let Some(pdir) = PIPES.get(&pchar) {
        // if we walk in for direction X, the pipe should go in the mirror direction
        let mdir = mirror_dir(dir);
        if (mdir & *pdir) != 0 {
            // we leave from the other pipe end
            let newdir = *pdir & !mdir;
            return Some(((newx, newy), newdir));
        }
    }
    // either invalid pipe char, or invalid direction in incoming pipe
    return None;
}

fn walk_around(pm: &Pipemaze, mut dir: DIR) -> Option<usize> {
    let mut pos = pm.startpos;
    let mut pathlen = 0;
    while let Some((newpos, newdir)) = walk_pipe(pm, pos, dir) {
        pathlen += 1;
        pos = newpos;
        dir = newdir;
        if pos == pm.startpos {
            return Some(pathlen);
        }
    }
    None
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let pm = parse(input);
    er.parse_done();
    for dir in [NORTH, WEST, SOUTH, EAST] {
        if let Some(pathlen) = walk_around(&pm, dir) {
            er.part1(pathlen / 2, Some("Halfway of loop length"));
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input1() -> BufReader<&'static [u8]> {
        BufReader::new(
"7-F7-
.FJ|7
SJLL7
|F--J
LJ.LJ
".as_bytes()
        )
    }

    fn test_input2() -> BufReader<&'static [u8]> {
        BufReader::new(
"-L|F7
7S-7|
L|7||
-L-J|
L|-JF
".as_bytes()
        )
    }

    #[test]
    fn test_part1() {
        let er = ExRunner::run("day 10".to_string(), solve, test_input1());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("8".to_string()));
    }

    #[test]
    fn test2_part1() {
        let er = ExRunner::run("day 10".to_string(), solve, test_input2());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("4".to_string()));
    }
}
