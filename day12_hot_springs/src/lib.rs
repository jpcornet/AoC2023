use exrunner::ExRunner;
use std::{io::BufRead, collections::HashMap};
// use std::time::Instant;

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
    let mut inbuf = Vec::new();
    for l in input.lines() {
        let line = l.expect("Error reading input");
        let (springs, runstr) = line.split_once(' ').expect("Need space in input");
        let runs: Vec<i32> = runstr.split(',').map(|n| n.parse().expect("runs should be numeric")).collect();
        let possibilities = spring_distributions(springs.as_bytes(), &runs, &mut springruncache, &mut cs);
        inbuf.push((springs.to_string(), runs));
        // er.debugln(&format!("Line: {}, possible solutions: {}", line, possibilities));
        total_pos += possibilities;
    }
    er.part1(total_pos, Some("Sum of all possible solutions"));
    // now for part 2
    let mut total_pos = 0;
    for (springs, runs) in inbuf {
        let longsprings = &format!("{}?{}?{}?{}?{}", springs, springs, springs, springs, springs);
        let mut longruns = runs.to_owned();
        for _ in 0..4 {
            let mut copyrun = runs.to_owned();
            longruns.append(&mut copyrun);
        }
        // let start = Instant::now();
        let possibilities = spring_distributions(longsprings.as_bytes(), &longruns, &mut springruncache, &mut cs);
        // let elapsed = start.elapsed().as_micros();
        // er.debugln(&format!("Runtime {}. Longsprings {} longruns {:?} possibilities: {}", elapsed, longsprings, longruns, possibilities));
        total_pos += possibilities;
    }
    er.part2(total_pos, Some("all possible solutions for long springs"));
    er.debugln(&format!("Cache stats: {:?}", cs));
}

fn spring_distributions(springs: &[u8], runs: &[i32], cache: &mut HashMap<SpringCacheEntry, i64>, cachestat: &mut CacheStats) -> i64 {
    // eprintln!("springs {}, runs {:?}", std::str::from_utf8(springs).unwrap(), runs);
    // easy ones first. No runs.
    if runs.len() == 0 {
        // if any springs are definately broken, there is no way to do it.
        // if there are only unknown springs, there is only 1 way to do it, which is all not broken.
        let possible = if springs.contains(&b'#') { 0 } else { 1 };
        // eprintln!("  no runs, possible={possible}");
        return possible;
    }

    // skip any leading "." springs that are known good.
    let firstbad = springs.iter().enumerate().find_map(|(i, &c)| {
        if c == b'.' { None } else { Some(i) }
    });
    if firstbad.is_none() {
        // no bad springs left. Since we already ruled out runs.len() == 0, this is not possible.
        // eprintln!("  no springs, not possible");
        return 0;
    }
    let activesprings = &springs[firstbad.unwrap()..];

    // sum of all runs
    let totalruns: i32 = runs.iter().sum();
    // minimum length of runs plus blanks in between
    let minlength = totalruns + runs.len() as i32 - 1;

    // cramming too many runs on the springs
    if  minlength > activesprings.len() as i32 {
        return 0;
    }
    // not enough runs for the springs already set
    let set_springs = activesprings.iter().filter(|&&c| c == b'#').count();
    if set_springs as i32 > totalruns {
        // eprintln!("  not enough runs, set_springs={set_springs} > totalruns={totalruns}");
        return 0;
    }
    // too many runs for the springs left
    let unknown_springs = activesprings.iter().filter(|&&c| c == b'?').count();
    if totalruns > set_springs as i32 + unknown_springs as i32 {
        // eprintln!("  too many runs, totalruns={totalruns} > set_springs={set_springs} + unknown={unknown_springs}");
        return 0;
    }

    // only concentrate on the first batch of possibly ungood springs
    let firstgood = activesprings.iter().enumerate().find_map(|(i, &c)| {
        if c == b'.' { Some(i) } else { None }
    });
    let firstbatch;
    let nextsprings;
    if let Some(offset) = firstgood {
        firstbatch = &activesprings[..offset];
        nextsprings = &activesprings[offset..];
    } else {
        firstbatch = activesprings;
        nextsprings = &[];
    }

    // only set springs
    if !firstbatch.contains(&b'?') {
        if runs[0] != firstbatch.len() as i32 {
            // no match, so not possible
            // eprintln!("  only set springs, run {} does not match.", runs[0]);
            return 0;
        } else if firstgood.is_some() {
            // it's a match, and there are springs left. Recurse for the rest of the springs and the runs
            let recpossible = spring_distributions(nextsprings, &runs[1..], cache, cachestat);
            // eprintln!("  recurse returned {recpossible}");
            return recpossible;
        } else if runs.len() == 1 {
            // it's a match but there are no springs left, and no runs either. So 1 possibility
            // eprintln!("  match, at end of springs, possible = 1");
            return 1;
        } else {
            // no springs left, but there are runs left, so not possible.
            // eprintln!("  match at end of springs, runs left = {:?}, not possible.", &runs[1..]);
            return 0;
        }
    }

    // try the cache
    let key = SpringCacheEntry{ springs: activesprings.to_vec(), runs: runs.to_vec() };
    if let Some(&possibilities) = cache.get(&key) {
        cachestat.cache_hit += 1;
        // eprintln!("  cached result: {possibilities}");
        return possibilities;
    }

    // only unknown springs. We can calculate the possibilities using statistics.
    if !firstbatch.contains(&b'#') {
        // try to squeeze in an increasing number of runs, starting from zero... upto all runs.
        let mut total_possible = 0;
        // optimisation: See how many runs even fit in the rest of the springs, then start with
        // testing at least as much as are left.
        let startruns = min_runs_left(runs, nextsprings);
        for numruns in startruns..runs.len()+1 {
            let restruns = if numruns < runs.len() {
                &runs[numruns..]
            } else {
                &[]
            };
            let theseruns: i32 = runs[..numruns].iter().sum();
            // for zero runs, minlength comes out as -1 but that does not matter
            let minlength = theseruns + numruns as i32 - 1;
            let wiggle = firstbatch.len() as i32 - minlength;
            // if wiggle is negative, it means these runs do not fit, and neither will any more runs, so break
            if wiggle < 0 {
                break;
            }
            let rest_possible = spring_distributions(nextsprings, restruns, cache, cachestat);
            // eprintln!("All-unknowns {} trying to place runs {:?}. wiggle={wiggle}. Rest {} runs {:?} possible={rest_possible}",
            //     std::str::from_utf8(firstbatch).unwrap(), &runs[..numruns], std::str::from_utf8(nextsprings).unwrap(), restruns);
            if rest_possible > 0 {
                // in how many ways can we distribute these "wiggle" blanks over the empty spaces, including the leading
                // and trailing empty space, so over numruns + 1 spaces. Or divided by numruns divisions between
                // the spaces. There are (numruns + wiggle over numruns) possibilities.
                let possibilities = (0..numruns as i64).fold(1, |acc, e| {
                    acc * (wiggle as i64 + numruns as i64 - e) / (e+1)
                });
                // eprintln!("  put runs {:?} in {}, possibilities={possibilities}, rest recursed={rest_possible}",
                //     &runs[..numruns], std::str::from_utf8(firstbatch).unwrap());
                total_possible += possibilities * rest_possible;
            }
        }
        // eprintln!(" only unknowns, all possible={total_possible}, stored in cache");
        cache.insert(key, total_possible );
        cachestat.cache_miss += 1;
        return total_possible;
    }

    // last resort. Try sticking run number 1 in, then recurse with the rest of the runs and the springs.
    // we already know the first batch is a combination of ? and # springs, so we must put at least 1 run in.
    let mut possible = 0;
    let firstrun = runs[0] as usize;
    let restruns = &runs[1..];
    if firstrun > firstbatch.len() {
        // not possible
        // eprintln!("  first run {firstrun} does not match {}, not possible", std::str::from_utf8(firstbatch).unwrap());
        return 0;
    }
    for pos in 0..(firstbatch.len() + 1 - firstrun as usize) {
        // make sure that the spring before the run is ?, so can be good.
        if pos > 0 && firstbatch[pos-1] == b'#' {
            // it's a broken spring, so previous pos started a run, so nothing after this is possible
            break;
        }
        // make sure that the spring after the run is ? or end of run
        if pos + firstrun == firstbatch.len() || firstbatch[pos + firstrun] == b'?' {
            // calculate possibilities for the rest of the runs and the rest of the springs.
            // if we reached the end of the springs, we have 1 possibility if restruns is empty
            if pos + firstrun + 1 < activesprings.len() {
                let recpossible = spring_distributions(&activesprings[pos+firstrun+1..], restruns, cache, cachestat);
                possible += recpossible;
            } else if restruns.len() == 0 {
                possible += 1;
            }
        }
    }

    // insert into cache
    cache.insert(key, possible );
    cachestat.cache_miss += 1;
    // eprintln!("  mixed result, possible={possible}, stored in cache");
    return possible;
}

// cram as many runs as possible at the end of the springs, and return how many runs from the start do not fit,
// which could be anything from 0 to runs.len(), inclusive.
fn min_runs_left(runs: &[i32], springs: &[u8]) -> usize {
    let mut springpos = springs.len() as i32;
    let mut runidx = runs.len();
'runindex:
    while runidx > 0 {
        let run = runs[runidx - 1];
        // find the first position at the end of springs where this fits
    'springpos:
        while springpos - run >= 0 {
            // make sure springpos is the end of a run
            if (springpos as usize) < springs.len() && springs[springpos as usize] == b'#' {
                // not a run end, so try next springpos
                springpos -= 1;
                continue 'springpos;
            }
            for i in 0..run as usize {
                if springs[(springpos - run) as usize + i] == b'.' {
                    // doesn't fit. Try next springpos
                    springpos -= 1;
                    continue 'springpos;
                }
            }
            // make sure that the item before the run is not a # known broken spring
            if springpos as i32 - run - 1 >= 0 && springs[(springpos - run) as usize - 1] == b'#' {
                // it doesn't fit. Decrease springpos and try again
                springpos -= 1;
                continue 'springpos;
            }
            // it fits, try next run
            runidx -= 1;
            // decrease springpos with run plus one for the space between runs.
            springpos -= run + 1;
            continue 'runindex;
        }
        // this doesn't fit anymore, so return
        return runidx;
    }
    runidx
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
        assert_eq!(er.answ()[1], Some("525152".to_string()));
    }

    fn test_slowinput() -> BufReader<&'static [u8]> {
    //    BufReader::new(".??.?.????. 2,1,2".as_bytes())
    //    BufReader::new(".?????.?.???.????? 2,1,1,1,2,1".as_bytes())
        BufReader::new("??.?.?..?????..??.?? 2,1".as_bytes())
    }

    #[test]
    fn test_slow() {
        let er = ExRunner::run("day 12".to_string(), solve, test_slowinput());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("32".to_string()));
    }
}
