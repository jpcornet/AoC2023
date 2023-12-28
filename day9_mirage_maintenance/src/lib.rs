use exrunner::ExRunner;
use std::io::BufRead;

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let (predictprevsum, predictendsum) = input.lines().map(|l| {
        let line = l.expect("Error reading input");
        let nums: Vec<i32> = line.split_whitespace().map(|n| n.parse().expect("Input should be numeric")).collect();
        let mut derive = vec![nums];
        while !is_all_zeros(derive.last().unwrap()) {
            let mut prev = None;
            derive.push( derive.last().unwrap().iter().filter_map(move |&x| {
                if let Some(pr) = prev {
                    prev = Some(x);
                    Some(x - pr)
                } else {
                    prev = Some(x);
                    None
                }
            }).collect());
        }
        if derive.last().unwrap().len() == 0 {
            panic!("Cannot derive to proper sequence, input = {}", line);
        }
        let predictend: i32 = derive.iter().map(|v| v.last().unwrap()).sum();
        let predictprev: i32 = derive.iter().rev().fold(0, |a, v| v[0] - a);
        // er.debugln(&format!("Line = {}. Derive goes {} deep. Prediction is: {} .. {}", line, derive.len(), predictprev, predictend));
        ( predictprev, predictend )
    }).fold((0, 0), |a, e| (a.0 + e.0, a.1 + e.1));
    er.part1(predictendsum, Some("Sum of all predictions"));
    er.part2(predictprevsum, Some("Sum of all backwards extrapolations"));
}

trait Zero {
    const ZERO: Self;
}

impl Zero for i32 { const ZERO: Self = 0; }
impl Zero for i64 { const ZERO: Self = 0; }

fn is_all_zeros<T: PartialEq + Zero>(nums: &Vec<T>) -> bool {
    nums.iter().all(|x| *x == T::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45
".as_bytes()
        )
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 9".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("114".to_string()));
        assert_eq!(er.answ()[1], Some("2".to_string()));
    }
}
