use exrunner::ExRunner;
use std::io::BufRead;

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let in_vec: Vec<String> = input.lines()
                    .map(|x| x.unwrap()).collect();
    er.parse_done();
    let in_digits: Vec<Vec<&str>> = in_vec.iter()
        .map(|x| x.matches(char::is_numeric).collect()).collect();
    let in_nums: Vec<i32> = in_digits.iter()
        .map(|x| format!("{}{}", x[0], x[x.len()-1]).parse().unwrap()).collect();
    er.part1(in_nums.into_iter().sum::<i32>(), Some("Sum of scattered numbers"));
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

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 1 - trebuchet".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("142".to_string()));
    }}
