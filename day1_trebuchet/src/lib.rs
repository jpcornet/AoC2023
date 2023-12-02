use exrunner::ExRunner;
use std::io::BufRead;
use std::collections::HashMap;

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let numbers = HashMap::from([
        ("one", 1),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
        ("nine", 9),
    ]);

    let in_vec: Vec<String> = input.lines()
                    .map(|x| x.unwrap()).collect();
    er.parse_done();
    let in_digits: Vec<Vec<&str>> = in_vec.iter()
        .map(|x| x.matches(char::is_numeric).collect()).collect();
    let in_nums: Vec<i32> = in_digits.iter()
        .map(|x| {
            if x.len() > 0 {
                format!("{}{}", x[0], x[x.len()-1]).parse().unwrap()
            } else { 0 }
        }
    ).collect();
    er.part1(in_nums.into_iter().sum::<i32>(), Some("Sum of scattered numbers"));
    let in2_digits: Vec<i32> = in_vec.iter()
        .map(|l| {
            let mut lpos = l.find(|c: char| c.is_ascii_digit());
            let mut lval = None;
            if let Some(pos) = lpos {
                let bval = (l.as_bytes())[pos] - b'0';
                lval = Some(bval.into());
            }
            for (name, val) in &numbers {
                if let Some(pos) = l.find(*name) {
                    if lpos.is_none() || pos < lpos.unwrap() {
                        lpos = Some(pos);
                        lval = Some(*val);
                    }
                }
            }
            let mut rpos = l.rfind(|c: char| c.is_ascii_digit());
            let mut rval = None;
            if let Some(pos) = rpos {
                let bval = (l.as_bytes())[pos] - b'0';
                rval = Some(bval.into());
            }
            for (name, val) in &numbers {
                if let Some(pos) = l.rfind(*name) {
                    if rpos.is_none() || pos > rpos.unwrap() {
                        rpos = Some(pos);
                        rval = Some(*val);
                    }
                }
            }
            lval.unwrap_or(0) * 10 + rval.unwrap_or(0)
        }
    ).collect();
    // er.debugln(&format!("in2_digits = {:?}", in2_digits));
    er.part2(in2_digits.into_iter().sum::<i32>(), None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet
".as_bytes()
        )
    }

    fn test_input2() -> BufReader<&'static [u8]> {
        BufReader::new(
            "two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen
".as_bytes()
        )
    }

    #[test]
    fn test_part1() {
        let er = ExRunner::run("day 1 - trebuchet".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("142".to_string()));
    }

    #[test]
    fn test_part2() {
        let er = ExRunner::run("day 1 - trebuchet".to_string(), solve, test_input2());
        er.print_raw();
        assert_eq!(er.answ()[1], Some("281".to_string()));
    }
}
