use exrunner::ExRunner;
use std::{io::BufRead, collections::{HashMap, HashSet}};

fn parse(input: impl BufRead) -> Vec<Vec<u8>> {
    input.lines().map(|l| {
        l.expect("Error reading input").as_bytes().to_vec()
    }).collect()
}

type DIR = u8;
const NORTH: DIR = 1;
const WEST: DIR = 2;
const SOUTH: DIR = 4;
const EAST: DIR = 8;

// implement / mirror
fn mirror1(d: DIR) -> DIR {
    match d {
        NORTH => EAST,
        WEST => SOUTH,
        SOUTH => WEST,
        EAST => NORTH,
        _ => panic!("Unknown direction {d} in mirror /")
    }
}

// implement \ mirror
fn mirror2(d: DIR) -> DIR {
    match d {
        NORTH => WEST,
        WEST => NORTH,
        SOUTH => EAST,
        EAST => SOUTH,
        _ => panic!("Unknown direction {d} in mirror \\")
    }
}

fn dxdy(dir: DIR) -> (i32, i32) {
    if dir == NORTH {
        (0, -1)
    } else if dir == WEST {
        (-1, 0)
    } else if dir == SOUTH {
        (0, 1)
    } else if dir == EAST {
        (1, 0)
    } else {
        panic!("Unknown direction {}", dir);
    }
}

fn shine(floor: &Vec<Vec<u8>>, seen: &mut HashMap<(i32, i32), DIR>, output: &mut HashSet<(i32, DIR)>, x: i32, y: i32, dir: DIR) {
    // off the grid, abort.
    if y < 0 {
        output.insert((x, SOUTH));
        return;
    }
    if y >= floor.len() as i32 {
        output.insert((x, NORTH));
        return;
    }
    if x < 0 {
        output.insert((y, EAST));
        return;
    }
    if x >= floor[y as usize].len() as i32 {
        output.insert((y, WEST));
        return;
    }

    let mut seen_dir = 0;
    seen.entry((x, y)).and_modify(|sdir| {
        seen_dir = *sdir;
        *sdir |= dir;
    }).or_insert(dir);
    if seen_dir & dir != 0 {
        // already seen in this direction
        return;
    }

    let tile = floor[y as usize][x as usize];
    // println!("Energize point {x},{y} direction {dir} tile {}", tile as char);
    if tile == b'/' {
        let newdir = mirror1(dir);
        let (dx, dy) = dxdy(newdir);
        shine(floor, seen, output, x + dx, y + dy, newdir);
        return;
    } else if tile == b'\\' {
        let newdir = mirror2(dir);
        let (dx, dy) = dxdy(newdir);
        shine(floor, seen, output, x + dx, y + dy, newdir);
        return;
    } else if tile == b'-' && (dir == NORTH || dir == SOUTH) {
        // beam split east/west
        shine(floor, seen, output, x + 1, y, EAST);
        shine(floor, seen, output, x - 1, y, WEST);
        return;
    } else if tile == b'|' && (dir == EAST || dir == WEST) {
        // beam split north/south
        shine(floor, seen, output, x, y - 1, NORTH);
        shine(floor, seen, output, x, y + 1, SOUTH);
        return;
    } else {
        // tile == b'.' or beam splitter in wrong direction, just continue.
        let (dx, dy) = dxdy(dir);
        shine(floor, seen, output, x + dx, y + dy, dir);
        return;
    }
}

fn count_energized(floor: &Vec<Vec<u8>>, x: i32, y: i32, dir: DIR) -> (usize, HashSet<(i32, DIR)>) {
    let mut light_seen = HashMap::new();
    let mut output = HashSet::new();
    shine(&floor, &mut light_seen, &mut output, x, y, dir);
    (light_seen.len(), output)
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let floor = parse(input);
    er.parse_done();
    // collect outputs already seen. We cannot improve stuff by shining light into it.
    let mut output_seen = HashSet::new();
    // determine max energized level
    let mut max_energized = None;
    // first do all EAST and WEST light.
    for y in 0..floor.len() {
        for dir in [EAST, WEST] {
            if output_seen.contains(&(y as i32, dir)) {
                // already seen as output, makes no sense shining light in here, that can only get the same
                // or less light in
                continue;
            }
            let x = if dir == EAST { 0 } else { floor[y].len() - 1 };
            let (energized, output) = count_energized(&floor, x as i32, y as i32, dir);
            for o in output {
                output_seen.insert(o);
            }
            if y == 0 && dir == EAST {
                // we solved part 1
                er.part1(energized, Some("Number of energized tiles"));
            }
            if max_energized.is_none() || energized > max_energized.unwrap() {
                max_energized = Some(energized);
                // println!("New max energized level {energized} shining dir {dir} pos {y}");
            }
        }
    }
    for x in 0..floor[0].len() {
        for dir in [SOUTH, NORTH] {
            if output_seen.contains(&(x as i32, dir)) {
                continue;
            }
            let y = if dir == SOUTH { 0 } else { floor.len() - 1};
            let (energized, output) = count_energized(&floor, x as i32, y as i32, dir);
            for o in output {
                output_seen.insert(o);
            }
            if max_energized.is_none() || energized > max_energized.unwrap() {
                max_energized = Some(energized);
                // println!("New max energized level {energized} shining dir {dir} pos {x}");
            }
        }
    }
    er.part2(max_energized.unwrap(), Some("Maximum energized tiles"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
r".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|....
".as_bytes())
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 16".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("46".to_string()));
        assert_eq!(er.answ()[1], Some("51".to_string()));
    }
}
