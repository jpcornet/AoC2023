use exrunner::ExRunner;
use std::io::BufRead;

#[derive(Debug, Copy, Clone)]
struct RaceCond {
    time: i64,
    dist: i64
}

fn parse(input: impl BufRead) -> Vec<RaceCond> {
    let tags = vec!["Time", "Distance"];
    let x: Vec<Vec<i64>> = input.lines().enumerate().map(|(i, l)| {
        let inline = l.expect("Error reading input");
        let (intag, nums) = inline.split_once(':').expect("Input should contain :");
        assert!(i < tags.len(), "Unexpected line number {i}");
        assert_eq!(intag, tags[i], "Expected {} got {}", tags[i], intag);
        nums.split_whitespace().map(|n| n.parse().expect("expect numbers")).collect()
    }).collect();
    assert_eq!(2, x.len(), "Expected 2 input lines");
    x[0].iter().zip(x[1].iter()).map(|(&time, &dist)| RaceCond{time, dist}).collect()
}

fn wins(r: RaceCond) -> Option<i64> {
    // if we push the button for x milliseconds, we travel x * (time - x) millimeters.
    // solve for x * (time - x) - dist >= 0. Or: - x ** 2 + time * x - dist >= 0.
    // The zero points of that equation can be calculated with the abc formula:
    // (-b +- sqrt(b**2 - 4ac)) / 2a.
    let sq = r.time * r.time - 4 * r.dist;
    if sq.is_negative() {
        return None;
    } else {
        let sqr = (sq as f64).sqrt();
        let s1 = ((r.time as f64) - sqr) / 2.0;
        let s2 = ((r.time as f64) + sqr) / 2.0;
        let w1 = s1.ceil() as i64;
        let w2 = s2.floor() as i64;
        println!("Race {:?} win from {s1} == {w1} to {s2} == {w2}", r);
        if (sqr as i64).pow(2) == sq {
            // Square matches exactly. That means we do not win at the zero points,
            // but break even. So we need to exclude the endpoints, instead of include them.
            return Some(w2 - w1 - 1);
        } else {
            return Some(w2 - w1 + 1);
        }
    }
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let races = parse(input);
    er.parse_done();
    let mut mult: i64 = 1;
    for r in &races {
        let w = wins(*r).expect("Expected to win the race");
        mult *= w;
    }
    er.part1(mult, None);
    let (timestr, diststr) = races.iter().
        fold(("".to_string(), "".to_string()),
        |a, r| (format!("{}{}", a.0, r.time), format!("{}{}", a.1, r.dist)));
    let bigr = RaceCond{ time: timestr.parse().expect("num"), dist: diststr.parse().expect("num") };
    er.part2(wins(bigr).expect("Expected to win"), None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"Time:      7  15   30
Distance:  9  40  200
".as_bytes()
        )
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 6".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("288".to_string()));
        assert_eq!(er.answ()[1], Some("71503".to_string()));
    }
}
