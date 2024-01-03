use exrunner::ExRunner;
use std::io::BufRead;
use std::collections::HashMap;

fn parse(input: impl BufRead) -> Vec<Vec<u8>> {
    input.lines().map(|l| {
        let line = l.expect("Error reading input");
        line.as_bytes().to_vec()
    }).collect()
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let mut dish = parse(input);
    assert!(dish.len() > 0, "Dish should not be empty");
    assert!(dish[0].len() > 0, "Dish should not contain empty lines");
    er.parse_done();
    let dishlen = dish.len() as i32;
    let total_load: i32 = (0..dish[0].len()).map(|x| {
        let mut rock_load = dishlen;
        (0..dishlen).map(|y| {
            let row = &dish[y as usize];
            match row[x] {
                b'O' => { rock_load -= 1; rock_load + 1 },
                b'#' => { rock_load = dishlen - y - 1; 0 },
                _ => { 0 },
            }
        }).sum::<i32>()
    }).sum();
    er.part1(total_load, Some("Load of rocks after rolling north"));

    // part 2, now actually do the rolling around
    // keep a hash of positions, so we can detect when we're repeating.
    let mut dishpos = HashMap::new();
    let mut repeats: i64 = 0;
    let limit = 1000000000;
    while repeats < limit {
        spin_cycle(&mut dish);
        repeats += 1;
        dishpos.entry(dish.to_vec()).and_modify(|r| {
            let cycle = repeats - *r;
            let left = limit - repeats;
            repeats += left - (left % cycle);
        }).or_insert(repeats);
    }

    let actual_load: i32 = (0..dish[0].len()).map(|x| {
        (0..dishlen).map(|y| {
            let row = &dish[y as usize];
            if row[x] == b'O' {
                dishlen - y
            } else {
                0
            }
        }).sum::<i32>()
    }).sum();
    er.part2(actual_load, Some("Load after lots of spin cycles"));
}

fn spin_cycle(dish: &mut Vec<Vec<u8>>) {
    do_tilt(dish, 0, -1);
    do_tilt(dish, -1, 0);
    do_tilt(dish, 0, 1);
    do_tilt(dish, 1, 0);
}

fn do_tilt(dish: &mut Vec<Vec<u8>>, dx: i32, dy: i32) {
    // major is the axis over which we are moving, minor is the other axis
    let major;
    let minor;
    let xbase;
    let xmult;
    let ybase;
    let ymult;
    if dx != 0 {
        major = dish[0].len() as i32;
        minor = dish.len() as i32;
        xbase = if dx > 0 { major - 1 } else { 0 };
        xmult = 0;
        ybase = 0;
        ymult = 1;
    } else {
        major = dish.len() as i32;
        minor = dish[0].len() as i32;
        ybase = if dy > 0 { major - 1 } else { 0 };
        ymult = 0;
        xbase = 0;
        xmult = 1;
    }
    for a1 in 0..minor {
        // start positions for x, y
        let mut xpos = (xbase + xmult * a1) as usize;
        let mut ypos = (ybase + ymult * a1) as usize;
        // start of where we pile the blocks
        let mut xpile = xpos;
        let mut ypile = ypos;
        for _ in 0..major {
            match dish[ypos][xpos] {
                b'O' => {
                    // it's a round boulder, roll it to xpile, ypile
                    if xpos != xpile || ypos != ypile {
                        dish[ypile][xpile] = b'O';
                        dish[ypos][xpos] = b'.';
                    }
                    // pile on the next position
                    xpile = (xpile as i32 - dx) as usize;
                    ypile = (ypile as i32 - dy) as usize;
                },
                b'#' => {
                    // it's square boulder, piling will be on the next pos
                    xpile = (xpos as i32 - dx) as usize;
                    ypile = (ypos as i32 - dy) as usize;
                },
                _ => { // nothing
                }
            };
            xpos = (xpos as i32 - dx) as usize;
            ypos = (ypos as i32 - dy) as usize;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....
".as_bytes())
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 14".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("136".to_string()));
        assert_eq!(er.answ()[1], Some("64".to_string()));
    }
}
