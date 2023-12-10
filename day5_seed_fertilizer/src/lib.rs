use exrunner::ExRunner;
use std::{io::{BufRead, Lines}, cmp::Ordering, collections::HashMap};

type PItem = i64;

#[derive(Debug, Clone)]
struct ConvMapItem {
    dest_start: PItem,
    src_start: PItem,
    len: PItem
}

#[derive(Debug, Clone)]
struct ConvMap {
    fromwhat: String,
    towhat: String,
    convmaps: Vec<ConvMapItem>,
    revmaps: Vec<ConvMapItem>,
    destend: Vec<(PItem, usize)>,
}

// struct ProcItemIter<'a> {
//     num: PItem,
//     direct: Option<Option<i64>>,
//     index: usize,
//     cmap: &'a ConvMap,
// }

impl ConvMap {
    fn new(name: &str, lines: &mut Lines<impl BufRead>) -> ConvMap {
        let (fromname, toname) = name.split_once("-to-").expect("Map name must contain -to-");
        let mut convmaps: Vec<_> = lines.take_while(|l| l.is_ok() && l.as_ref().unwrap() != "").map(|l| {
            let nums: Vec<_> = l.expect("Error reading input").split_whitespace()
                .map(|n| n.parse().expect("maps should be numeric"))
                .collect();
            assert_eq!(nums.len(), 3, "Maps should contain 3 numbers per line");
            ConvMapItem{ dest_start: nums[0], src_start: nums[1], len: nums[2]}
        }).collect();
        convmaps.sort_by_key(|ci| ci.src_start);
        // make sure the convmaps do not overlap
        if convmaps.len() > 0 {
            let mut last = convmaps[0].src_start + convmaps[0].len;
            for i in &convmaps[1..] {
                assert!(last <= i.src_start, "Overlap in {name}: range {:?} starts before {last}", *i);
                last = i.src_start + i.len;
            }
        }
        // now build the reverse maps
        let mut revmaps = convmaps.clone();
        revmaps.sort_by_key(|ci| ci.dest_start);
        // now fill "destend"
        let mut destend: Vec<_> = revmaps.iter().enumerate().map(|(index, ci)| (ci.dest_start + ci.len, index)).collect();
        destend.sort_by_key(|&(end, _)| end);

        ConvMap{ fromwhat: fromname.to_string(), towhat: toname.to_string(), convmaps, revmaps, destend }
    }

    fn map(&self, inelem: PItem) -> (PItem, Option<PItem>) {
        let index = self.convmaps.binary_search_by(|ci| {
            if ci.src_start > inelem {
                Ordering::Greater
            } else if ci.src_start + ci.len <= inelem {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        if let Ok(i) = index {
            (inelem - self.convmaps[i].src_start + self.convmaps[i].dest_start,
                Some(self.convmaps[i].src_start + self.convmaps[i].len - inelem) )
        } else {
            let nxtpos = index.err().unwrap_or(self.convmaps.len());
            if nxtpos < self.convmaps.len() {
                (inelem, Some(self.convmaps[nxtpos].src_start - inelem))
            } else {
                (inelem, None)
            }
        }
    }
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let mut l = input.lines();
    // first, get the seeds.
    let sline= l.next().expect("Input should not be empty").expect("Error reading input");
    let seeds_str = sline.strip_prefix("seeds:").expect("Seeds line should contains seeds:");
    let seeds: Vec<PItem> = seeds_str.trim().split_whitespace()
        .map(|n| n.parse().expect("Seeds should be nums")).collect();
    assert_eq!(l.next().expect("Should have at least 1 map").expect("Error reading input"), "", "line after seeds should be blank");

    // import the maps
    let mut maps = HashMap::new();
    while let Some(mname_line) = l.next() {
        let mname_map = mname_line.expect("Error reading input");
        let mname = mname_map.strip_suffix(" map:").expect("Expected a map");
        let map = ConvMap::new(mname, &mut l);
        maps.insert(map.fromwhat.to_string(), map);
    }
    // Now order the maps from "seed-to-X" to "X-to-location", so we can call them in order
    let mut maporder = Vec::new();
    let mut have = "seed";
    while have != "location" {
        let cmap = maps.get(have).expect(&format!("No map available for {}", have));
        maporder.push(cmap);
        have = &cmap.towhat;
    }
    er.parse_done();

    let mut min_loc = None;
    for s in &seeds {
        let mut pi = *s;
        for &cmap in &maporder {
            (pi, _) = cmap.map(pi);
        }
        // er.debugln(&format!("Got a location: {:?}", pi));
        if min_loc.is_none() || min_loc.unwrap() > pi {
            min_loc = Some(pi);
        }
    }
    er.part1(min_loc.unwrap_or(0), Some("Minimum location based on individual seeds"));

    // part 2, treat the seeds as a (start, len) pair
    min_loc = None;
    let mut si = seeds.iter();
    while let Some(s) = si.next() {
        let lasts = *s + *si.next().expect("Expect even amount of numbers on seed line");
        er.debugln(&format!("Starting with seed {}, upto = {}", *s, lasts));
        let mut stseed = *s;
        let mut minrange = None;
        while stseed < lasts {
            let mut pi = stseed;
            // er.debugln(&format!("  trying seed {:?}", stseed));
            for &cmap in &maporder {
                let rsize;
                (pi, rsize) = cmap.map(pi);
                if minrange.is_none() {
                    minrange = rsize;
                } else if rsize.is_some() && minrange.unwrap() > rsize.unwrap() {
                    minrange = rsize;
                }
            }
            if min_loc.is_none() || min_loc.unwrap() > pi {
                min_loc = Some(pi);
                er.debugln(&format!("Minimum location so far: {}", pi));
            }
            assert!(minrange.unwrap_or(0) > 0, "minrange cannot be zero");
            stseed += minrange.unwrap();
        }
    }
    er.part2(min_loc.unwrap_or(0), Some("Minimum location based on seed ranges"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4
".as_bytes())
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 5".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("35".to_string()));
        assert_eq!(er.answ()[1], Some("46".to_string()));
    }
}
