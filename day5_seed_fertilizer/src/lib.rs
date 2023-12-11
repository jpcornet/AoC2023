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
    destend: Vec<(PItem, usize, Option<PItem>)>, // tuple contains: end+1 of range, index of range, minimum of start of all ranges after this one
}

struct ProcItemIter<'a> {
    num: PItem,
    direct: Option<Option<PItem>>,
    index: usize,
    cmap: &'a ConvMap,
}

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
        // now build the reverse maps, by calculating "revmap" and "destend"
        // revmap is the same ranges as convmaps, now sorted on start of target range.
        // destend contains the end of all ranges, sorted. Plus an index into the revmap for that range,
        // and the minimum of all revmaps that come after this range in the destend array, if any.
        // when doing a reverse map, we first find the range that ends just before the target, then
        // iterate in the "destend" map, until we hit a range where the minimum of all ranges is after our target.
        // range length is determined either until the end of our current range, or until the start of the next range,
        // which ever comes sooner.
        let mut revmaps = convmaps.clone();
        revmaps.sort_by_key(|ci| ci.dest_start);
        let mut destend: Vec<_> = revmaps.iter().enumerate().map(|(index, ci)| (ci.dest_start + ci.len, index, None)).collect();
        destend.sort_by_key(|&(end, _, _)| end);
        // fill the minimum of ranges after target in destend, by iterating from the end
        let mut minstart = None;
        for de in destend.iter_mut().rev() {
            de.2 = minstart;
            if minstart.is_none() || minstart.unwrap() > revmaps[de.1].dest_start {
                minstart = Some(revmaps[de.1].dest_start);
            }
        }

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
            let nxtpos = index.unwrap_err();
            if nxtpos < self.convmaps.len() {
                (inelem, Some(self.convmaps[nxtpos].src_start - inelem))
            } else {
                (inelem, None)
            }
        }
    }

    fn revmap(&self, inelem: PItem) -> ProcItemIter {
        // try a straigt conversion. Only if the given number does not fall in any map.
        let index = self.convmaps.binary_search_by(|ci| {
            if ci.src_start > inelem {
                Ordering::Greater
            } else if ci.src_start + ci.len <= inelem {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        let direct;
        if let Err(dirnext) = index {
            if dirnext >= self.convmaps.len() {
                // direct is possible, but there is no known range length
                direct = Some(None);
            } else {
                // direct is possible, dirnext contains the next range, so range length is known
                direct = Some(Some(self.convmaps[dirnext].src_start - inelem));
            }
        } else {
            direct = None;
        }

        let stindex = self.destend.binary_search_by_key(&inelem, |&(end, _, _)| end);
        // we matched one after the end of the range, so we can start right where we matched.
        let de_ind = stindex.unwrap_or_else(|e| e);
        ProcItemIter{ num: inelem, direct, index: de_ind, cmap: self }
    }
}

impl<'a> Iterator for ProcItemIter<'a> {
    type Item = (PItem, Option<PItem>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(dirsize) = self.direct.take() {
            return Some((self.num, dirsize));
        }
        // search ranges until we run out of possible ones: either end of ranges, or all future ranges start after target
        while self.index < self.cmap.destend.len() {
            // translate our index to the correct revmap
            let ci = &self.cmap.revmaps[ self.cmap.destend[self.index].1 ];
            let nxtstart = self.cmap.destend[self.index].2;
            self.index += 1;
            if ci.dest_start <= self.num && ci.dest_start + ci.len > self.num {
                let pi = self.num - ci.dest_start + ci.src_start;
                let mut rlen = ci.dest_start + ci.len - self.num;
                // find the first range after num
                let nxtind = self.cmap.revmaps.binary_search_by_key(&(self.num+1), |ci| ci.dest_start)
                    .unwrap_or_else(|e| e);
                if nxtind < self.cmap.revmaps.len() {
                    let nxtstart = self.cmap.revmaps[nxtind].dest_start;
                    if nxtstart - self.num < rlen {
                        rlen = nxtstart - self.num;
                    }
                }
                return Some((pi, Some(rlen)));
            }
            if nxtstart.is_some() && nxtstart.unwrap() > self.num {
                break;
            }
        }
        // we reached the end of possible targets
        None
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

    // part 2, treat the seeds as a (start, len) pair. Convert to list of seed ranges
    let mut seedranges = Vec::new();
    let mut si = seeds.iter();
    while let Some(s) = si.next() {
        let lasts = *s + *si.next().expect("Expect even amount of numbers on seed line");
        seedranges.push((*s, lasts));
    }
    seedranges.sort_by_key(|&(start, _)| start);

    // Loop over the location rev map until we find a match
    let mut minloc: PItem = 0;
'location:
    loop {
        let mut min_range = None;
        let piter = maporder.last().unwrap().revmap(minloc);
        for (pi, rlen) in piter {
            // er.debugln(&format!("  revmapped to {} range {pi} len {:?}", maporder.last().unwrap().fromwhat, rlen));
            // keep track of minimum range len
            if min_range.is_none() || (rlen.is_some() && min_range.unwrap() > rlen.unwrap()) {
                min_range = rlen;
            }
            if let Some(offset) = find_map_match(pi, maporder.len() - 2, rlen, &maporder, &seedranges) {
                // println!("  * Solution found for location {}", minloc + offset);
                er.part2(minloc + offset, Some("Minimum location based on ranges of seeds"));
                break 'location;
            }
        }
        if let Some(mr) = min_range {
            minloc += mr;
        } else {
            panic!("No solution found for part 2");
        }
    }
}

fn find_map_match(pi: PItem, mapidx: usize, rlen: Option<PItem>, maporder: &Vec<&ConvMap>, seedranges: &Vec<(PItem, PItem)>) -> Option<PItem> {
    let cmap = maporder[mapidx];
    // loop over entries in this map until we run out of rlen
    let mut offset = 0;
    while rlen.is_none() || offset < rlen.unwrap() {
        let mut min_range = None;
        let piter = cmap.revmap(pi + offset);
        for (pi2, mut rlen2) in piter {
            // this range should not exceed the given range
            if rlen.is_some() && (rlen2.is_none() || rlen2.unwrap() + offset > rlen.unwrap()) {
                rlen2 = Some(rlen.unwrap() - offset);
            }
            // keep track of minimum range of current map
            if min_range.is_none() || (rlen2.is_some() && min_range.unwrap() > rlen2.unwrap()) {
                min_range = rlen2;
            }
            if mapidx > 0 {
                // recurse
                if let Some(offset2) = find_map_match(pi2, mapidx - 1, rlen2, maporder, seedranges) {
                    // println!("    * Solution found for {} {}, offset is {}", cmap.fromwhat, pi + offset + offset2, offset + offset2);
                    return Some(offset + offset2);
                }
            } else if let Some(offset2) = find_seed_match(pi2, rlen2, seedranges) {
                return Some(offset + offset2);
            }
        }
        if let Some(mr) = min_range {
            offset += mr;
        } else {
            // infinte range returned no usable matches, or no mappings possible at all
            return None;
        }
    }
    None
}

fn find_seed_match(pi: PItem, rlen: Option<PItem>, seedranges: &Vec<(PItem, PItem)>) -> Option<PItem> {
    let inseed = seedranges.binary_search_by(|&(start, end)| {
        if start > pi {
            Ordering::Greater
        } else if end <= pi {
            Ordering::Less
        } else { // start <= pi && pi < end
            Ordering::Equal
        }
    });

    if let Err(seedpos) = inseed {
        // no direct match in seeds for start of range
        if seedpos >= seedranges.len() {
            // match after last seed range
            return None;
        }
        // see if [pi, pi+rlen> overlaps with a seed range
        if pi <= seedranges[seedpos].0 && (
            rlen.is_none() || // range is infinte so seedrange is included as it starts after pi
            pi + rlen.unwrap() - 1 >= seedranges[seedpos].0 // start of seedrange is in pi - pi+rlen-1
        ) {
            // return offset to start of the seed range
            return Some(seedranges[seedpos].0 - pi);
        } else {
            return None;
        }
    } else {
        // pi is in a seed range
        return Some(0);
    }
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
