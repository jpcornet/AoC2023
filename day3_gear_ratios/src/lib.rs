use exrunner::ExRunner;
use std::io::BufRead;
use regex::Regex;

const SYMBOLS: &str = "@#$%^&*-+=<>?/";

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let field: Vec<Vec<_>> = input.lines().map(|x| x.unwrap().into_bytes()).collect();
    let numbers_re = Regex::new(r"[0-9]+").unwrap();
    // look for numeric strings in field, then search around for a symbol
    let sum: i32 = field.iter().enumerate().flat_map(|(y, l)| {
        let nums: Vec<_> = numbers_re.find_iter(String::from_utf8(l.to_vec()).unwrap().as_str())
            .filter_map(|m| {
                // look for a symbol around the number
                let upper = if y > 0 { y - 1 } else { 0 };
                let lower = if y < field.len() - 1 { y + 1 } else { y };
                let left = if m.start() > 0 { m.start() - 1 } else { 0 };
                let right = if m.end() < l.len() { m.end() } else { m.end() - 1 };
                let hassymb = (upper..lower+1).into_iter()
                    .any(|ty| (left..right+1).into_iter()
                        .any(|tx| { 
                            SYMBOLS.contains(field[ty][tx] as char)
                        }
                    )
                );
                if hassymb {
                    let res: i32 = m.as_str().parse().unwrap();
                    Some(res)
                } else {
                    None
                }
            }).collect();
        nums.into_iter()
    }).sum();
    er.part1(sum, Some("Sum of numbers with symbols"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..
".as_bytes()
        )
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 3".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("4361".to_string()));
        assert_eq!(er.answ()[1], Some("467835".to_string()));
    }
}
