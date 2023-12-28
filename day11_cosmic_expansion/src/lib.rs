use exrunner::ExRunner;
use std::{io::BufRead, collections::HashSet};

fn parse(input: impl BufRead) -> Vec<(i64, i64)> {
    input.lines().enumerate().flat_map(|(y, l)| {
        let line = l.expect("Error reading input");
        line.chars().enumerate().filter_map(move |(x, c)| {
            if c == '#' {
                Some((x as i64, y as i64))
            } else {
                None
            }
        }).collect::<Vec<_>>().into_iter()
    }).collect()
}

static mut BIG_EXPANSION: i64 = 1000000;

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let galaxies = parse(input);
    // determine field size
    let xmax = galaxies.iter().map(|(x, _)| *x).max().unwrap() + 1;
    let ymax = galaxies.iter().map(|(_, y)| *y).max().unwrap() + 1;
    er.parse_done();
    // determine row/cols that do not appear in input
    let mut sparse_x = HashSet::new();
    for x in 0..xmax {
        sparse_x.insert(x);
    }
    let mut sparse_y = HashSet::new();
    for y in 0..ymax {
        sparse_y.insert(y);
    }
    for (xg, yg) in &galaxies {
        sparse_x.remove(xg);
        sparse_y.remove(yg);
    }
    // now expand the voids
    let expand_galaxies: Vec<_> = galaxies.iter().map(|&(x, y)| {
        (x + sparse_x.iter().filter(|&&sx| sx < x).count() as i64,
        y + sparse_y.iter().filter(|&&sy| sy < y).count() as i64)
    }).collect();
    // er.debugln(&format!("Expanded galaxies at positions: {:?}", expand_galaxies));
    er.part1(sum_dist_pairs(&expand_galaxies), Some("Distance between expanded galaxies"));
    // no multi-threading, so it's safe...
    let big = unsafe { BIG_EXPANSION - 1 };
    let bigexpand_galaxies: Vec<_> = galaxies.iter().map(|&(x, y)| {
        (x + sparse_x.iter().filter(|&&sx| sx < x).count() as i64 * big,
        y + sparse_y.iter().filter(|&&sy| sy < y).count() as i64 * big)
    }).collect();
    er.part2(sum_dist_pairs(&bigexpand_galaxies), Some("Distance between big expanded galaxies"));
}

fn sum_dist_pairs(glx: &Vec<(i64, i64)>) -> i64 {
    (0..glx.len()-1).flat_map(|g1| {
        (g1..glx.len()).map(move |g2| {
            galaxy_dist(&glx[g1], &glx[g2])
        })
    }).sum()
}

fn galaxy_dist(g1: &(i64, i64), g2: &(i64, i64)) -> i64 {
    (g1.0 - g2.0).abs() + (g1.1 - g2.1).abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....
".as_bytes()
        )
    }

    #[test]
    fn test_part12() {
        unsafe {
            BIG_EXPANSION = 10;
        }
        let er = ExRunner::run("day 11".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("374".to_string()));
        assert_eq!(er.answ()[1], Some("1030".to_string()));
        unsafe {
            BIG_EXPANSION = 100;
        }
        let er = ExRunner::run("day 11".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[1], Some("8410".to_string()));
    }
}
