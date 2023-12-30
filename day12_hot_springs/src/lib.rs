use exrunner::ExRunner;
use std::{io::BufRead, ops::ControlFlow, collections::HashMap};

#[derive(Debug)]
struct CacheStats {
    cache_hit: i32,
    cache_miss: i32,
}

#[derive(PartialEq, Eq, Hash)]
struct SpringCacheEntry {
    springs: Vec<u8>,
    runs: Vec<i32>,
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let mut total_pos = 0;
    let mut cs = CacheStats{ cache_hit: 0, cache_miss: 0 };
    // create a cache for spring positions & runs to possibilities.
    let mut springruncache = HashMap::new();
    for l in input.lines() {
        let line = l.expect("Error reading input");
        let (springs, runstr) = line.split_once(' ').expect("Need space in input");
        let runs: Vec<i32> = runstr.split(',').map(|n| n.parse().expect("runs should be numeric")).collect();
        let possibilities = count_solutions(springs, &runs, &mut springruncache, &mut cs);
        // er.debugln(&format!("Line: {}, possible solutions: {}", line, possibilities));
        total_pos += possibilities;
    }
    er.part1(total_pos, Some("Sum of all possible solutions"));
    er.debugln(&format!("Cache stats: {:?}", cs));
}

fn count_solutions(springs: &str, runs: &Vec<i32>, cache: &mut HashMap<SpringCacheEntry, i32>, cachestat: &mut CacheStats) -> i32 {
    // convert to springgroups. Also convert to [byte] as it's easier to address.
    let springgroups: Vec<_> = springs.split('.').filter_map(|s| {
        if s.len() > 0 { Some(s.as_bytes()) }  else { None }
    }).collect();
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
            spring_distributions(springgroups[i], &rundist[i], cache, cachestat)
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

fn spring_distributions(springs: &[u8], runs: &Vec<i32>, cache: &mut HashMap<SpringCacheEntry, i32>, cachestat: &mut CacheStats) -> i32 {
    // easy ones first. No runs.
    if runs.len() == 0 {
        // if any springs are definately broken, there is no way to do it.
        // if there are only unknown springs, there is only 1 way to do it, which is all not broken.
        let possible = if springs.contains(&b'#') { 0 } else { 1 };
        return possible;
    }
    // sum of all runs
    let totalruns = runs.iter().fold(0, |acc, &e| acc + e);
    // minimum length of runs plus blanks in between
    let minlength = totalruns + runs.len() as i32 - 1;
    // cramming too many runs on the springs
    if  minlength > springs.len() as i32 {
        return 0;
    }
    // not enough runs for the springs already set
    let set_springs = springs.iter().filter(|&&c| c == b'#').count();
    if  set_springs as i32 > totalruns {
        return 0;
    }
    // only set springs
    if !springs.contains(&b'?') {
        let possible = if runs.len() == 1 && runs[0] == springs.len() as i32 { 1 } else { 0 };
        return possible;
    }

    // only unknown springs. This can easily be calculated with the minimum length and the number of runs
    if !springs.contains(&b'#') {
        let wiggle = springs.len() as i32 - minlength;
        let bindivs = runs.len() as i32;
        // in how many ways can we distribute these "wiggle" blanks over the empty spaces, including the leading
        // and trailing empty space, so over runs.len() + 1 spaces. Or divided by runs.len() divisions between
        // the spaces. There are (bindivs + wiggle over bindivs) possibilities.
        let possibilities = (0..bindivs).fold(1, |acc, e| {
            acc * (wiggle + bindivs - e) / (e+1)
        });
        // println!("Only unknown springs {} runs {:?}, possibilities: {possibilities}", std::str::from_utf8(springs).unwrap(), runs);
        return possibilities;
    }

    // try the cache
    let key = SpringCacheEntry{ springs: springs.to_vec(), runs: runs.to_vec() };
    if let Some(&possibilities) = cache.get(&key) {
        cachestat.cache_hit += 1;
        return possibilities;
    }

    // last resort. Try sticking run number 1 in, then recurse with the rest of the runs and the string
    let mut possible = 0;
    let firstrun = runs[0] as usize;
    let restruns = runs[1..].to_vec();
    for pos in 0..(springs.len() + 1 - firstrun as usize) {
        // make sure that the spring before the run is ?, so can be good.
        if pos > 0 && springs[pos-1] == b'#' {
            // it's a broken spring, so previous pos started a run, so nothing after this is possible
            break;
        }
        // make sure that the spring after the run is ? or end of run
        if pos + firstrun == springs.len() || springs[pos + firstrun] == b'?' {
            // calculate possibilities for the rest of the runs and the rest of the springs.
            // if we reached the end of the springs, we have 1 possibility if restruns is empty
            if pos + firstrun + 1 < springs.len() {
                let recpossible = spring_distributions(&springs[pos+firstrun+1..], &restruns, cache, cachestat);
                possible += recpossible;
            } else if restruns.len() == 0 {
                possible += 1;
            }
        }
    }

    // insert into cache
    cache.insert(key, possible );
    cachestat.cache_miss += 1;
    return possible;
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
