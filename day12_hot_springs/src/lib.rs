use exrunner::ExRunner;
use std::{io::BufRead, ops::ControlFlow};

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let mut total_pos = 0;
    for l in input.lines() {
        let line = l.expect("Error reading input");
        let (springs, runstr) = line.split_once(' ').expect("Need space in input");
        let runs: Vec<i32> = runstr.split(',').map(|n| n.parse().expect("runs should be numeric")).collect();
        let possibilities = count_solutions(springs, &runs);
        er.debugln(&format!("Line: {}, possible solutions: {}", line, possibilities));
        total_pos += possibilities;
    }
    er.part1(total_pos, Some("Sum of all possible solutions"));
}

fn count_solutions(springs: &str, runs: &Vec<i32>) -> i32 {
    let springgroups: Vec<_> = springs.split('.').filter(|s| s.len() > 0).collect();
    // corner case: no spring groups at all
    if springgroups.len() == 0 {
        return if runs.len() == 0 { 1 } else { 0 };
    }
    // now distribute the elements of runs over all of the spring groups, and add up all possibilities
    let mut rundist: Vec<Vec<_>> = vec![runs.iter().map(|&x| x).collect()];
    // append empty groups, until rundist has same number of elements as springgroups
    while rundist.len() < springgroups.len() {
        rundist.push(Vec::new());
    }
    let mut total_solutions = 0;
    loop {
        // calculate possibilities for 1 distribution
        // start by calculating the elements with the least amount of numbers assigned,
        // as those are most likely to return quickly.
        let mut sgindex: Vec<_> = (0..springgroups.len()).collect();
        sgindex.sort_by_key(|&i| springgroups[i].len());
        let solutions = sgindex.iter().map(|&i| {
            spring_distributions(springgroups[i], &rundist[i])
        }).try_fold(1, |acc, e| {
            if e > 0 {
                ControlFlow::Continue(acc * e)
            } else {
                ControlFlow::Break(0)
            }
        });
        if let ControlFlow::Continue(x) = solutions {
            total_solutions += x;
            // println!("Solutions for {:?} on {:?} is: {x}", rundist, springgroups);
        }

        // update the distribution to the next possible.
        // move the rightmost value that can move, 1 to the right.
        // then move all remaining elements to that same element
        if let Some(rightmost) = (0..rundist.len()-1).rev().find(|&i| rundist[i].len() > 0) {
            let elem = rundist[rightmost].pop().unwrap();
            let mut newelem = vec![elem];
            for x in rightmost+1..rundist.len() {
                newelem.append(&mut rundist[x]);
            }
            rundist[rightmost+1] = newelem;
        } else {
            // no element found to move, so we are done.
            return total_solutions;
        }
    }
}

fn spring_distributions(springs: &str, runs: &Vec<i32>) -> i32 {
    // easy ones first. No runs.
    if runs.len() == 0 {
        // if any springs are definately broken, there is no way to do it.
        // if there are only unknown springs, there is only 1 way to do it, which is all not broken.
        return if springs.contains('#') { 0 } else { 1 };
    }
    // cramming too many runs on the springs
    let totalruns = runs.iter().fold(0, |acc, &e| acc + e);
    if totalruns + runs.len() as i32 - 1 > springs.len() as i32 {
        return 0;
    }
    // not enough runs for the springs already set
    let set_springs = springs.chars().filter(|&c| c == '#').count();
    if  set_springs as i32 > totalruns {
        return 0;
    }
    // only set springs
    if !springs.contains('?') {
        return if runs.len() == 1 && runs[0] == springs.len() as i32 { 1 } else { 0 };
    }
    // now just brute-force the number of springs and see if it matches.
    let setunknown = totalruns as usize - set_springs;
    let unknownsprings = springs.chars().filter(|&c| c == '?').count();
    let mut possible = 0;
'springposition:
    for mask in 0_i64..1_i64<<unknownsprings {
        if mask.count_ones() == setunknown as u32 {
            // determine run lengths of current layout.
            let mut bit = 1;
            let mut run_idx = 0;
            let mut run_len = 0;
            let mut trypos = String::new();
            for c in springs.chars() {
                let is_broken = if c == '#' {
                    trypos.push(c);
                    true
                } else if c == '?' {
                    let broken = mask & bit != 0;
                    bit <<= 1;
                    trypos.push(if broken { '#' } else { '.' });
                    broken
                } else {
                    panic!("Bad character in springs");
                };
                if is_broken {
                    run_len += 1;
                } else if run_len > 0 && run_idx < runs.len() && runs[run_idx] == run_len {
                    run_idx += 1;
                    run_len = 0;
                } else if run_len > 0 {
                    // mismatch
                    // println!("Mismatch! Input {springs} runs {:?}. Tried {trypos} mismatch at run index {run_idx}", runs);
                    continue 'springposition;
                }
            }
            // any runs at the end?
            if run_len != 0 {
                if run_idx < runs.len() && runs[run_idx] == run_len {
                    run_idx += 1;
                } else {
                    // println!("Mismatch! Input {springs} runs {:?}. Tried {trypos} mismatch at run index {run_idx}", runs);
                    continue 'springposition;
                }
            }
            if run_idx == runs.len() {
                // we found a match!
                possible += 1;
                // println!("Found a match! Input {springs} runs {:?} got: {trypos}", runs);
            }
        }
    }
    // println!("Springs {springs} runs {:?} possibilities: {possible}", runs);
    possible
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1
".as_bytes()
        )
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 12".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("21".to_string()));
    }
}
