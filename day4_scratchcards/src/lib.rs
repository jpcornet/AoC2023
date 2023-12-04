use exrunner::ExRunner;
use std::io::BufRead;
use std::collections::HashMap;
use std::collections::VecDeque;

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let wins: Vec<_> = input.lines().map(|l| {
        let line = l.unwrap();
        let (win, have) = line.split_once(':').expect("Line needs Card #:")
            .1.split_once('|').expect("Line needs numbers | numbers");
        let mut winnums = HashMap::new();
        for w in win.trim().split_whitespace() {
            winnums.insert(w, ());
        }
        have.trim().split_whitespace().filter(|h| winnums.contains_key(*h)).collect::<Vec<_>>().len()
    }).collect();
    let part1: i32 = wins.iter().filter_map(|w| if *w > 0 { Some(1 << (*w-1)) } else { None }).sum();
    er.part1(part1, None);
    let mut copies_won: VecDeque<usize> = VecDeque::new();
    let part2: usize = wins.iter().map(|w| {
        let copies = copies_won.pop_front().unwrap_or(0) + 1;
        // er.debugln(&format!("Next card has {} copies", copies));
        // we win copies of each of the next w cards
        for i in 0..*w {
            if let Some(c) = copies_won.get_mut(i) {
                *c += copies;
            } else {
                // no copies for this card yet, so we just win number of copies
                copies_won.push_back(copies);
            }
        }
        copies
    }).sum();
    if !copies_won.is_empty() {
        er.debugln(&format!("Warning, copies_won is not empty, contains: {:?}", copies_won));
    }
    er.part2(part2, None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11
".as_bytes()
        )
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 3".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("13".to_string()));
        assert_eq!(er.answ()[1], Some("30".to_string()));
    }
}