use exrunner::ExRunner;
use std::{io::{BufRead, Lines}, cmp::Ordering, collections::HashMap};

#[derive(Debug, Clone)]
struct ProcItem {
    what: String,
    num: i64
}

#[derive(Debug)]
struct ConvMapItem {
    dest_start: i64,
    src_start: i64,
    len: i64
}

#[derive(Debug)]
struct ConvMap {
    fromwhat: String,
    towhat: String,
    convmaps: Vec<ConvMapItem>
}

impl ConvMap {
    fn new(name: &str, lines: &mut Lines<impl BufRead>) -> ConvMap {
        let (fromname, toname) = name.split_once("-to-").expect("Map name must contain -to-");
        let mut items: Vec<_> = lines.take_while(|l| l.is_ok() && l.as_ref().unwrap() != "").map(|l| {
            let nums: Vec<_> = l.expect("Error reading input").split_whitespace()
                .map(|n| n.parse().expect("maps should be numeric"))
                .collect();
            assert_eq!(nums.len(), 3, "Maps should contain 3 numbers per line");
            ConvMapItem{ dest_start: nums[0], src_start: nums[1], len: nums[2]}
        }).collect();
        items.sort_by(|a, b| a.src_start.cmp(&b.src_start));
        // make sure the items do not overlap
        if items.len() > 0 {
            let mut last = items[0].src_start + items[0].len;
            for i in &items[1..] {
                assert!(last <= i.src_start, "Overlap in {name}: range {:?} starts before {last}", *i);
                last = i.src_start + i.len;
            }
        }
        ConvMap{ fromwhat: fromname.to_string(), towhat: toname.to_string(), convmaps: items }
    }

    fn map(&self, inelem: &ProcItem) -> (ProcItem, Option<i64>) {
        if inelem.what != self.fromwhat {
            panic!("Cannot index map {}-to-{} with a {}", self.fromwhat, self.towhat, inelem.what);
        }
        let index = self.convmaps.binary_search_by(|ci| {
            if ci.src_start > inelem.num {
                Ordering::Greater
            } else if ci.src_start + ci.len <= inelem.num {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        if let Ok(i) = index {
            (ProcItem{ what: self.towhat.to_string(), num: inelem.num - self.convmaps[i].src_start + self.convmaps[i].dest_start },
                Some(self.convmaps[i].src_start + self.convmaps[i].len - inelem.num) )
        } else {
            let pi = ProcItem{ what: self.towhat.to_string(), num: inelem.num };
            let nxtpos = index.err().unwrap_or(self.convmaps.len());
            if nxtpos < self.convmaps.len() {
                (pi, Some(self.convmaps[nxtpos].src_start - inelem.num))
            } else {
                (pi, None)
            }
        }
    }
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let mut l = input.lines();
    // first, get the seeds.
    let sline= l.next().expect("Input should not be empty").expect("Error reading input");
    let seeds_str = sline.strip_prefix("seeds:").expect("Seeds line should contains seeds:");
    let seeds: Vec<_> = seeds_str.trim().split_whitespace()
        .map(|n| ProcItem{ what: "seed".to_string(), num: n.parse().expect("Seeds should be nums") }).collect();
    assert_eq!(l.next().expect("Should have at least 1 map").expect("Error reading input"), "", "line after seeds should be blank");

    // import the maps
    let mut maps = HashMap::new();
    while let Some(mname_line) = l.next() {
        let mname_map = mname_line.expect("Error reading input");
        let mname = mname_map.strip_suffix(" map:").expect("Expected a map");
        let map = ConvMap::new(mname, &mut l);
        maps.insert(map.fromwhat.to_string(), map);
    }
    er.parse_done();

    let mut min_loc = None;
    for s in &seeds {
        let mut pi = s.clone();
        while pi.what != "location" {
            // er.debugln(&format!("Have a {:?}", pi));
            let cmap = maps.get(&pi.what).expect(&format!("No map available for {}", pi.what));
            (pi, _) = cmap.map(&pi);
        }
        // er.debugln(&format!("Got a location: {:?}", pi));
        if min_loc.is_none() || min_loc.unwrap() > pi.num {
            min_loc = Some(pi.num);
        }
    }
    er.part1(min_loc.unwrap_or(0), Some("Minimum location based on individual seeds"));

    // part 2, treat the seeds as a (start, len) pair
    min_loc = None;
    let mut si = seeds.iter();
    while let Some(s) = si.next() {
        let lasts = s.num + si.next().expect("Expect even amount of numbers on seed line").num;
        er.debugln(&format!("Starting with seed {:?}, upto = {}", s, lasts));
        let mut stseed = s.clone();
        let mut minrange = None;
        while stseed.num < lasts {
            let mut pi = stseed.clone();
            // er.debugln(&format!("  trying seed {:?}", stseed));
            while pi.what != "location" {
                let cmap = maps.get(&pi.what).expect("map error");
                let rsize;
                (pi, rsize) = cmap.map(&pi);
                if minrange.is_none() {
                    minrange = rsize;
                } else if rsize.is_some() && minrange.unwrap() > rsize.unwrap() {
                    minrange = rsize;
                }
            }
            if min_loc.is_none() || min_loc.unwrap() > pi.num {
                min_loc = Some(pi.num);
                er.debugln(&format!("Minimum location so far: {}", pi.num));
            }
            assert!(minrange.unwrap_or(0) > 0, "minrange cannot be zero");
            stseed.num += minrange.unwrap();
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
