use exrunner::ExRunner;
use std::{io::BufRead, collections::HashSet};

fn parse(input: impl BufRead) -> Vec<(usize, usize)> {
    input.lines().enumerate().flat_map(|(y, l)| {
        let line = l.expect("Error reading input");
        line.chars().enumerate().filter_map(move |(x, c)| {
            if c == '#' {
                Some((x, y))
            } else {
                None
            }
        }).collect::<Vec<_>>().into_iter()
    }).collect()
}

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
    let expand_galaxies: Vec<_> = galaxies.into_iter().map(|(x, y)| {
        (x + sparse_x.iter().filter(|&&sx| sx < x).count(), y + sparse_y.iter().filter(|&&sy| sy < y).count())
    }).collect();
    // er.debugln(&format!("Expanded galaxies at positions: {:?}", expand_galaxies));
    // determine distance pairs
    let mut dist_sum = 0;
    for g1 in 0..expand_galaxies.len() - 1 {
        for g2 in g1+1..expand_galaxies.len() {
            dist_sum += galaxy_dist(&expand_galaxies[g1], &expand_galaxies[g2])
        }
    }
    er.part1(dist_sum, Some("Distance between expanded galaxies"));
}

fn galaxy_dist(g1: &(usize, usize), g2: &(usize, usize)) -> i32 {
    let g1x = g1.0 as i32;
    let g1y = g1.1 as i32;
    let g2x = g2.0 as i32;
    let g2y = g2.1 as i32;
    (g1x - g2x).abs() + (g1y - g2y).abs()
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
        let er = ExRunner::run("day 11".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("374".to_string()));
    }
}
