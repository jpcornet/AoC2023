use exrunner::ExRunner;
use std::io::BufRead;

fn parse(input: impl BufRead) -> Vec<Vec<u8>> {
    input.lines().map(|l| {
        let line = l.expect("Error reading input");
        line.as_bytes().to_vec()
    }).collect()
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let dish = parse(input);
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
    }
}
