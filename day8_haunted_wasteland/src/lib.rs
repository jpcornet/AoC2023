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
    // for the test input of part 2, there is no "AAA", so skip part1
    if navigate.maps.contains_key("AAA") {
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
    // for part2, find each ??A node, and determine pathlengths to possible ??Z nodes, and a repeat count
    let startnodes = navigate.maps.keys().filter(|&n| n.ends_with("A"));
    let allpaths: Vec<_> = startnodes.map(|s| {
        let sol = find_pathlen(&navigate, s);
        // er.debugln(&format!("Start at {}, paths: {:?}", s, sol));
        sol
    }).collect();
    // we have 2 options: in the "easy" case, each node has 1 path and repeat is same as path.
    // in that case, we can just use the LCM algorithm
    let is_simple = allpaths.iter().all(|p| {
        p.len() == 1 && p[0].repeat.is_some() && p[0].repeat.unwrap() == p[0].initial
    });
    if is_simple {
        // er.debugln("We can use the simple LCM algorithm");
        let mut p_gcd = allpaths[0][0].initial;
        for pathind in 1..allpaths.len() {
            p_gcd = gcd(p_gcd, allpaths[pathind][0].initial);
        }
        // er.debugln(&format!("GCD of paths is {p_gcd}"));
        let mult = allpaths.iter().fold(p_gcd, |a, p| {
            a * p[0].initial / p_gcd
        });
        er.part2(mult, Some("Simple LCM number of steps"));
    } else {
        // loop until we find a number of repeats that makes each node end at a finish node.
        // or where each path has an initial + n * repeat that is the same number.
        let mut steps: i64 = 1;
        loop {
            let mut min_increment = None;
            let mut found = true;
            for nodepaths in &allpaths {
                let mut foundnode = false;
                for p in nodepaths {
                    if steps == p.initial || p.repeat.is_some() && (steps - p.initial) % p.repeat.unwrap() == 0 {
                        foundnode = true;
                    } else {
                        let incr = if steps < p.initial {
                            Some(p.initial - steps)
                        } else if let Some(rp) = p.repeat {
                            Some(rp - (steps - p.initial) % rp)
                        } else {
                            None
                        };
                        if min_increment.is_none() {
                            min_increment = incr;
                        } else if incr.is_some() && min_increment.unwrap() > incr.unwrap() {
                            min_increment = incr;
                        }
                    }
                }
                if !foundnode {
                    found = false;
                }
            }
            if found {
                er.part2(steps, Some("Number of steps for all paths to finish"));
                return;
            } else if let Some(i) = min_increment {
                steps += i;
                er.debugln(&format!("Incrementing {i}, next try steps={steps}"));
            } else {
                steps += 1;
                er.debugln(&format!("Odd, min_increment not set, just stepping 1 to {steps}"));
            }
        }
    }
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b > 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[derive(Debug)]
struct Pathlen {
    initial: i64,
    repeat: Option<i64>,
}

#[derive(PartialEq, Eq, Hash)]
struct Repeatpos<'a> {
    node: &'a str,
    instr_index: usize,
}

fn find_pathlen(nav: &Maps, start: &str) -> Vec<Pathlen> {
    let mut result = Vec::new();
    let mut repeatpos = HashMap::new();
    let mut instr_index = 0;
    let mut count: i64 = 0;
    let mut node = start;
    // break out of the loop as soon as we find a repeat position
    // XXX note: only checks for repeats at endpoints, so could loop forever on faulty input
    loop {
        let lr = nav.maps.get(node).expect("Undefined node");
        if nav.instructions[instr_index] == 'L' as u8 {
            node = &lr.left;
        } else if nav.instructions[instr_index] == 'R' as u8 {
            node = &lr.right;
        } else {
            panic!("Unknown Left/Right instruction");
        }
        count += 1;
        instr_index += 1;
        if instr_index >= nav.instructions.len() {
            instr_index = 0;
        }
        if node.ends_with("Z") {
            let rp = Repeatpos{node, instr_index};
            if let Some(rpentry) = repeatpos.get(&rp) {
                // back at the same node and instr_index, so we repeat.
                // the rpentry points at the result array from where we repeat
                let pl: &Pathlen = &result[*rpentry];
                let repoffset = count - pl.initial;
                for i in *rpentry..result.len() {
                    result[i].repeat = Some(repoffset);
                }
                return result;
            } else {
                // we found an end node but we aren't repeating yet.
                repeatpos.insert(rp, result.len());
                result.push(Pathlen{ initial: count, repeat: None });
            }
        }
    }
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

    fn test_input3() -> BufReader<&'static [u8]> {
        BufReader::new(
"LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)
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

    #[test]
    fn test_part2() {
        let er = ExRunner::run("day 8".to_string(), solve, test_input3());
        er.print_raw();
        assert_eq!(er.answ()[1], Some("6".to_string()));
    }
}
