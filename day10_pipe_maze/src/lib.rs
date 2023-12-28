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
    let mut findstart = field.iter().enumerate().flat_map(|(y, l)| {
        l.iter().enumerate().filter_map(move |(x, c)| {
            if *c == b'S' {
                Some((x, y))
            } else {
                None
            }
        })
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
static PIPES: phf::Map<u8, DIR> = phf_map! {
    b'|' => NORTH | SOUTH,
    b'-' => EAST | WEST,
    b'L' => NORTH | EAST,
    b'J' => NORTH | WEST,
    b'7' => SOUTH | WEST,
    b'F' => SOUTH | EAST,
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
    let pchar = pm.field[newy][newx];
    // println!("Walk in direction {dir} from {},{} to {newx},{newy}. Now at {pchar}", pos.0, pos.1);
    if pchar == b'S' {
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

// Walk around in the pipemaze, starting at start pos in direction DIR
// if not possible, returns None. If it is possible, returns the path length
// until we reach the start pos again.
// second return value is a copy of the field with only the path itself on it
fn walk_around(pm: &Pipemaze, startdir: DIR) -> (Option<usize>, Vec<Vec<u8>>) {
    let mut pos = pm.startpos;
    let mut pathlen = 0;
    let mut dir = startdir;
    let mut pathonly: Vec<Vec<u8>> = Vec::new();
    // fill the pathonly with dots, same size as input pipemaze
    for l in &pm.field {
        pathonly.push( std::iter::repeat(b'.').take(l.len()).collect() );
    }
    while let Some((newpos, newdir)) = walk_pipe(pm, pos, dir) {
        pathlen += 1;
        pos = newpos;
        if pos == pm.startpos {
            // determine the starting point shape size. We started with startdir, and we end with dir into the startpos
            let startdirs = startdir | mirror_dir(dir);
            let startshape = PIPES.entries().find_map(|(&shape, &dirs)| if dirs == startdirs { Some(shape) } else { None }).expect("Unknown start directions");
            pathonly[pos.1][pos.0] = startshape;
            return (Some(pathlen), pathonly);
        } else {
            // copy this element of the path
            pathonly[pos.1][pos.0] = pm.field[pos.1][pos.0];
            dir = newdir;
        }
    }
    (None, pathonly)
}

fn count_enclosed(field: &Vec<Vec<u8>>) -> usize {
    let mut in_path = false;
    field.iter().map(move |l| {
        let enclosed = l.iter().filter(move |&&c| {
            // pretend to scan just south of the "-" marker. Anytime we cross the path, flip the "in-path" indicator.
            // This means it flips not only on | but also on F and 7. It does not flip on L and J.
            if c == b'|' || c == b'7' || c == b'F' {
                in_path = !in_path;
                false
            } else {
                c == b'.' && in_path
            }
        }).count();
        if in_path {
            panic!("Still in path at end of field");
        }
        enclosed
    }).sum()
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let pm = parse(input);
    er.parse_done();
    // er.debugln(&format!("Input pipemaze:\n{}", field_to_string(&pm.field)));
    for dir in [NORTH, WEST, SOUTH, EAST] {
        if let (Some(pathlen), pathonly) = walk_around(&pm, dir) {
            er.part1(pathlen / 2, Some("Halfway of loop length"));
            // er.debugln(&format!("Path only:\n{}", field_to_string(&pathonly)));
            let enclosed_tiles = count_enclosed(&pathonly);
            er.part2(enclosed_tiles, Some("Number of enclosed tiles"));
            break;
        }
    }
}

// fn field_to_string(field: &Vec<Vec<u8>>) -> String {
//     let lines: Vec<_> = field.iter().map(|l| {
//         str::from_utf8(l).unwrap()
//     }).collect();
//     lines.join("\n")
// }

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

    fn test_input3() -> BufReader<&'static [u8]> {
        BufReader::new(
"...........
.S-------7.
.|F-----7|.
.||.....||.
.||.....||.
.|L-7.F-J|.
.|..|.|..|.
.L--J.L--J.
...........
".as_bytes()
        )
    }

    fn test_input4() -> BufReader<&'static [u8]> {
        BufReader::new(
".F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...
".as_bytes()
        )
    }

    fn test_input5() -> BufReader<&'static [u8]> {
        BufReader::new(
"FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L
".as_bytes()
        )
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 10".to_string(), solve, test_input1());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("8".to_string()));
        assert_eq!(er.answ()[1], Some("1".to_string()));
    }

    #[test]
    fn test2_part12() {
        let er = ExRunner::run("day 10".to_string(), solve, test_input2());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("4".to_string()));
        assert_eq!(er.answ()[1], Some("1".to_string()));
    }

    #[test]
    fn test_part2() {
        let er = ExRunner::run("day 10".to_string(), solve, test_input3());
        er.print_raw();
        assert_eq!(er.answ()[1], Some("4".to_string()));
    }

    #[test]
    fn test2_part2() {
        let er = ExRunner::run("day 10".to_string(), solve, test_input4());
        er.print_raw();
        assert_eq!(er.answ()[1], Some("8".to_string()));
    }

    #[test]
    fn test3_part2() {
        let er = ExRunner::run("day 10".to_string(), solve, test_input5());
        er.print_raw();
        assert_eq!(er.answ()[1], Some("10".to_string()));
    }
}
