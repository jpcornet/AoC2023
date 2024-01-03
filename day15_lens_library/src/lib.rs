use exrunner::ExRunner;
use std::io::BufRead;

pub fn hash(s: &str) -> i32 {
    s.chars().fold(0, |acc, c| ((acc + c as i32) * 17) % 256)
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let long = input.lines().map(|l| l.expect("Error reading input")).collect::<Vec<_>>().join("");
    let part1: i32 = long.split(',').map(|instr| hash(instr)).sum();
    er.part1(part1, Some("sum of HASH value of each step"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new("rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7".as_bytes())
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 15".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("1320".to_string()));
    }

    #[test]
    fn test_hash() {
        assert_eq!(hash("HASH"), 52);
        assert_eq!(hash("rn=1"), 30);
    }
}
