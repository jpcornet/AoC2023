use exrunner::ExRunner;
use std::collections::HashSet;
use std::{io::BufRead, collections::HashMap};
use std::io::Lines;

struct TerrainParser<R: BufRead> {
    input: Option<R>,
    inputlines: Option<Lines<R>>,
}

#[derive(Debug)]
struct Terrain {
    rows: HashMap<String, Vec<usize>>,
    cols: HashMap<String, Vec<usize>>,
}

impl<R: BufRead> TerrainParser<R> {
    fn new(input: R) -> TerrainParser<R> {
        TerrainParser{ input: Some(input), inputlines: None }
    }
}

impl<R: BufRead> Iterator for TerrainParser<R> {
    type Item = Terrain;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rows = HashMap::new();
        let mut columns = Vec::new();
        let mut y = 0;
        let mut lines_iter;
        if let Some(li) = self.inputlines.take() {
            lines_iter = li;
        } else {
            let input = self.input.take().unwrap();
            lines_iter = input.lines()
        }
        while let Some(l) = lines_iter.next() {
            let line = l.expect("Error reading input");
            let trimmed = line.trim().to_owned();
            if trimmed.len() == 0 {
                // blank line. If nothing is read yet, try next line, otherwise return
                if columns.len() == 0 {
                    continue;
                } else {
                    break;
                }
            }
            if columns.len() == 0 {
                // first line, prepare columns
                for _ in 0..trimmed.len() {
                    columns.push(String::new());
                }
            } else if trimmed.len() != columns.len() {
                panic!("Non-rectangular input");
            }
            for (x, c) in trimmed.chars().enumerate() {
                columns[x].push(c);
            }
            rows.entry(trimmed).and_modify(|e: &mut Vec<usize>| e.push(y)).or_insert(vec![y]);
            y += 1;
        }
        // put the iterator back
        self.inputlines = Some(lines_iter);
        if columns.len() == 0 {
            // end of input
            None
        } else {
            let mut cols = HashMap::new();
            for (x, c) in columns.into_iter().enumerate() {
                cols.entry(c).and_modify(|e: &mut Vec<usize>| e.push(x)).or_insert(vec![x]);
            }
            Some(Terrain{ rows, cols })
        }
    }
}

pub fn solve<R: BufRead>(input: R, er: &mut ExRunner) {
    let terrains: TerrainParser<R> = TerrainParser::new(input);
    let mut notes = 0;
    let mut notes2 = 0;
    for t in terrains {
        let (numcols, numscols) = find_reflection(&t.cols);
        let (numrows, numsrows) = find_reflection(&t.rows);
        // if both are set, complain.
        if numcols.is_some() && numrows.is_some() {
            let nc = numcols.unwrap();
            let nr = numrows.unwrap();
            er.debugln(&format!("Both horizontal and vertical mirrors, at {nr} and {nc} respectively. Terrain = {:?}", t));
        }
        if let Some(nc) = numcols {
            // er.debugln(&format!("Found vertical mirror after column {nc}"));
            notes += nc + 1;
        }
        if let Some(nr) = numrows {
            // er.debugln(&format!("Found horizontal mirror below row {nr}"));
            notes += 100 * (nr + 1);
        } else if numcols.is_none() {
            er.debugln(&format!("No mirror found! terrain = {:?}", t));
        }

        if numscols.is_some() && numsrows.is_some() {
            let nc = numscols.unwrap();
            let nr = numsrows.unwrap();
            er.debugln(&format!("Both horizontal and vertical smudged mirrors, at {nr} and {nc} respectively. Terrain = {:?}", t));
        }
        if let Some(nc) = numscols {
            notes2 += nc + 1;
        }
        if let Some(nr) = numsrows {
            notes2 += 100 * (nr + 1)
        } else if numscols.is_none() {
            er.debugln(&format!("No smudged mirror found! Terrain = {:?}", t));
        }
    }
    er.part1(notes, Some("Sum of notes on mirrors"));
    er.part2(notes2, Some("Sum of notes on smudged mirrors"));
}

fn find_reflection(r: &HashMap<String, Vec<usize>>) -> (Option<usize>, Option<usize>) {
    // convert the hashmap values to a list.
    let mut poslist = Vec::new();
    // while doing that, remember any positions potentially next to a mirror, having 2 adjacent positions, or having adjacent positions with 2 rows in between.
    let mut mirror_positions = HashSet::new();
    for (terrain, positions) in r {
        for &pos in positions {
            if poslist.len() <= pos {
                let mut appendlist: Vec<(Vec<usize>, String)> = std::iter::repeat((vec![], "".to_string())).take(pos+1-poslist.len()).collect();
                poslist.append(&mut appendlist);
            }
            poslist[pos] = (positions.to_vec(), terrain.to_string());
            // check if pos is next to a mirror: if pos + 1 is also in positions
            if positions.contains(&(pos + 1)) {
                mirror_positions.insert(pos);
            }
            // secondary check, if pos is 1 removed from a mirror, insert pos + 1. This is for the smudge check, position along the mirror might not match.
            if positions.contains(&(pos + 3)) {
                mirror_positions.insert(pos + 1);
            }
        }
    }
    // for all possible mirror positions, see if it mirrors all the way to the edge
    // return centermost mirror position if there are multiple.
    let mut mirror_found = None;
    let mut smudge_mirror_found = None;
    let center = poslist.len() as i32 / 2;
    for mir in mirror_positions {
        let mut normal_mirror = true;
        let mut smudge_mirror = false;
        let mut smudge_pos = None;
        for dist in 0..mir+1 {
            // check position mir - dist also contains mir + 1 + dist.
            // or if mir + 1 + dist exceeds the possible positions, we are done
            let otherpos = mir + 1 + dist;
            if otherpos >= poslist.len() {
                break;
            }
            if !poslist[mir-dist].0.contains(&otherpos) {
                // no match, not a normal mirror
                normal_mirror = false;
                // could still be a smudged mirror
                if smudge_pos.is_none() {
                    if let Some(smudged) = smudge_match(&poslist[mir-dist].1, &poslist[otherpos].1) {
                        smudge_pos = Some((smudged, mir - dist));
                        smudge_mirror = true;
                    } else {
                        // not a smudged image
                        break;
                    }
                } else {
                    // cannot have a second smudge
                    smudge_mirror = false;
                    break;
                }
            }
        }
        if normal_mirror {
            if let Some(prevmir) = mirror_found {
                if (prevmir as i32 - center).abs() > (mir as i32 - center).abs() {
                    mirror_found = Some(mir);
                    println!("Duplicate mirrors, at {prevmir} and {mir}, last one is better");
                } else {
                    println!("Duplicate mirrors, at {prevmir} and {mir}, first one is better");
                }
            } else {
                mirror_found = Some(mir);
            }
        }
        if smudge_mirror {
            if let Some(prevmir) = smudge_mirror_found {
                if (prevmir as i32 - center).abs() > (mir as i32 - center).abs() {
                    smudge_mirror_found = Some(mir);
                    println!("Duplicate smudged mirrors, at {prevmir} and {mir}, last one is better");
                } else {
                    println!("Duplicate smudged mirrors, at {prevmir} and {mir}, first one is better");
                }
            } else {
                smudge_mirror_found = Some(mir);
                // println!("Found a smudged mirror at {}, smudge pos is {},{}", mir, smudge_pos.unwrap().0, smudge_pos.unwrap().1);
            }
        }
    }

    // last resort smudge detection. First or last rows might contain a smudged mirror.
    if poslist.len() >= 2 {
        if let Some(_smudge) = smudge_match(&poslist[0].1, &poslist[1].1) {
            if smudge_mirror_found.is_some() {
                println!("Duplicate smudged mirrors, at {} and 0, first one is better", smudge_mirror_found.unwrap());
            } else {
                smudge_mirror_found = Some(0);
                // println!("Found a smudged mirror at 0, smudge pos is {_smudge},0");
            }
        }
        if let Some(_smudge) = smudge_match(&poslist[poslist.len()-2].1, &poslist[poslist.len()-1].1) {
            if smudge_mirror_found.is_some() && smudge_mirror_found.unwrap() != poslist.len() - 2 {
                println!("Duplicate smudged mirrors, at {} and {}, first one is better", smudge_mirror_found.unwrap(), poslist.len() - 2);
            } else {
                smudge_mirror_found = Some(poslist.len() - 2);
                // println!("Found a smudged mirror at {}, smudge pos is {_smudge},{}", poslist.len() - 2, poslist.len() - 2);
            }
        }
    }
    // we reached the end of the possible mirror positions, return what we found.
    (mirror_found, smudge_mirror_found)
}

fn smudge_match(a: &str, b: &str) -> Option<usize> {
    let ba = a.as_bytes();
    let bb = b.as_bytes();
    if ba.len() != bb.len() {
        return None;
    }
    let mut smudge = None;
    for x in 0..ba.len() {
        if ba[x] != bb[x] {
            if smudge.is_some() {
                // more than 1 differing, return not found.
                return None;
            } else {
                smudge = Some(x);
            }
        }
    }
    smudge
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#
".as_bytes()
        )
    }

    fn test_input2() -> BufReader<&'static [u8]> {
        BufReader::new(
"#..#..#..#.##
..........###
..........###
#..#..#..#.##
###....####.#
....##......#
##..##..##..#
###....####..
....##.....##
.#......#.#.#
.##....##..##
..#.##.#....#
####..######.
.##..#.##.###
..##..##...#.
".as_bytes()
        )
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 13".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("405".to_string()));
        assert_eq!(er.answ()[1], Some("400".to_string()));
    }

    #[test]
    fn test_bug() {
        let er = ExRunner::run("day 13".to_string(), solve, test_input2());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("200".to_string()));
        assert_eq!(er.answ()[1], Some("5".to_string()));
    }
}
