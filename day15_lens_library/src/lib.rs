use exrunner::ExRunner;
use std::{io::BufRead, collections::HashMap};

pub fn hash(s: &str) -> i32 {
    s.chars().fold(0, |acc, c| ((acc + c as i32) * 17) % 256)
}

#[derive(Debug)]
struct LensPos {
    focallen: i64,
    boxpos: usize,
}

struct LensBox {
    maxidx: usize,
    lenses: HashMap<String, LensPos>,
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let long = input.lines().map(|l| l.expect("Error reading input")).collect::<Vec<_>>().join("");
    let part1: i32 = long.split(',').map(|instr| hash(instr)).sum();
    er.part1(part1, Some("sum of HASH value of each step"));

    // part2, do the lens positions. First initialize the empty boxes
    let mut boxes = Vec::new();
    for _ in 0..256 {
        boxes.push(LensBox{maxidx: 0, lenses: HashMap::new()});
    }
    for instr in long.split(',') {
        if let Some((name, focalstr)) = instr.split_once('=') {
            let focallen: i64 = focalstr.parse().expect("focal length must be numeric");
            let boxnr = hash(name);
            let lensbox = &mut boxes[boxnr as usize];
            lensbox.lenses.entry(name.to_string()).and_modify(|lp| lp.focallen = focallen)
                .or_insert_with(|| {lensbox.maxidx += 1; LensPos{ focallen, boxpos: lensbox.maxidx }});
        } else if instr.ends_with('-') {
            let name = instr.trim_end_matches('-');
            let boxnr = hash(name);
            let lensbox = &mut boxes[boxnr as usize];
            lensbox.lenses.remove(name);
        } else {
            panic!("Unknown instruction {}", instr);
        }
    }

    // now calculate the total focal strength
    let mut focalstrength = 0;
    for boxnr in 0..boxes.len() {
        let lensbox = &mut boxes[boxnr as usize];
        let mut lensorder: Vec<_> = lensbox.lenses.values().collect();
        lensorder.sort_by_key(|lp| lp.boxpos);
        // er.debugln(&format!("Boxnr {} lenses in order: {:?}", boxnr + 1, lensorder));
        focalstrength += lensorder.iter().enumerate()
            .map(|(idx, lp)| (boxnr as i64 + 1) * (idx as i64 + 1) * lp.focallen)
            .sum::<i64>();
    }
    er.part2(focalstrength, Some("Total focal strength after lens init"));
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
        assert_eq!(er.answ()[1], Some("145".to_string()));
    }

    #[test]
    fn test_hash() {
        assert_eq!(hash("HASH"), 52);
        assert_eq!(hash("rn=1"), 30);
    }
}
