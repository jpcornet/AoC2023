use exrunner::ExRunner;
use std::io::BufRead;
use std::collections::HashMap;

struct Node {
    left: String,
    right: String,
}

struct Maps {
    instructions: Vec<u8>,
    maps: HashMap<String, Node>,
}

fn parse(input: impl BufRead) -> Maps {
    let mut lines = input.lines();
    let instructions = lines.next().expect("Input cannot be empty").expect("Error reading input").as_bytes().to_vec();
    let mut maps = HashMap::new();
    while let Some(line) = lines.next() {
        let linestr = line.expect("Error reading input");
        if linestr.len() > 0 {
            let (wname, nodes) = linestr.split_once('=').expect("Input needs =");
            let name = wname.trim().to_string();
            let brackets: &[_] = &['(', ')'];
            let (wleft, wright) = nodes.trim().trim_matches(brackets).split_once(',').expect("Input needs ,");
            let left = wleft.trim().to_string();
            let right = wright.trim().to_string();
            let node = Node{ left, right };
            maps.insert(name, node);
        }
    }
    Maps{ instructions, maps }
} 

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let navigate = parse(input);
    er.parse_done();
    let mut count = 0;
    let mut node = "AAA";
    let mut instr_index = 0;
    while node != "ZZZ" {
        let lr = navigate.maps.get(node).expect("Undefined node");
        if navigate.instructions[instr_index] == 'L' as u8 {
            node = &lr.left;
        } else if navigate.instructions[instr_index] == 'R' as u8 {
            node = &lr.right;
        } else {
            panic!("Unknown Left/Right instruction");
        }
        count += 1;
        instr_index += 1;
        if instr_index >= navigate.instructions.len() {
            instr_index = 0;
        }
    }
    er.part1(count, Some("Number of steps to ZZZ"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input1() -> BufReader<&'static [u8]> {
        BufReader::new(
"RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)
".as_bytes()
        )
    }

    fn test_input2() -> BufReader<&'static [u8]> {
        BufReader::new(
"LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)
".as_bytes()
        )
    }

    #[test]
    fn test_part1() {
        let er = ExRunner::run("day 8".to_string(), solve, test_input1());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("2".to_string()));
    }

    #[test]
    fn test2_part1() {
        let er = ExRunner::run("day 8".to_string(), solve, test_input2());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("6".to_string()));
    }
}
