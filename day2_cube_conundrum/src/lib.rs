use exrunner::ExRunner;
use std::io::BufRead;
use std::collections::HashMap;

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let max_colour = HashMap::from([
        ("red", 12),
        ("green", 13),
        ("blue", 14),
    ]);
    let mut possible = 0;
    let mut totpower = 0;
    for l in input.lines() {
        let line = l.unwrap();
        let (gamenrstr, gameplay) = line.split_once(':').expect("Input line contains no colon");
        let mut gns = gamenrstr.split_whitespace();
        assert_eq!(gns.next(), Some("Game"), "Invalid input line");
        let gamenr: i32 = gns.next().expect("Input line should contain number")
            .parse().expect("number should be numeric");
        let subgames = gameplay.split(';');
        let mut is_possible = true;
        let mut min_cubes: HashMap<&str, i32> = HashMap::new();
        for sg in subgames {
            for cubes in sg.split(',') {
                let numcolour = cubes.trim_start().split_once(|c: char| c.is_whitespace())
                    .expect("gameplay must have number and colour");
                let num: i32 = numcolour.0.parse().expect("Number of cubes should be int");
                let max_of = max_colour.get(numcolour.1).expect("Unknown colour name");
                if num > *max_of {
                    is_possible = false;
                }
                min_cubes.entry(numcolour.1)
                    .and_modify(|e| if *e < num { *e = num })
                    .or_insert(num);
            }
        }
        if is_possible {
            possible += gamenr;
        }
        let power = *min_cubes.get("red").unwrap_or(&0)
            * *min_cubes.get("green").unwrap_or(&0)
            * *min_cubes.get("blue").unwrap_or(&0);
        // er.debugln(&format!("Power for game {} is {}", gamenr, power));
        totpower += power;
    }
    er.part1(possible, None);
    er.part2(totpower, None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
        "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green
".as_bytes()
        )
    }

    #[test]
    fn test_part1() {
        let er = ExRunner::run("day 2 - cube conundrum".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("8".to_string()));
        assert_eq!(er.answ()[1], Some("2286".to_string()));
    }
}
