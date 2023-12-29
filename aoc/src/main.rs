use clap::{Parser, CommandFactory};
use std::process::exit;
use std::io::ErrorKind;
use aoc::*;

const YEAR: u16 = 2023;

// all puzzle days. Note that the puzzle number should be the first number in the directory name.
const DAYS: &'static [Day] = &[
    Day{ dir: "day1_trebuchet", solve: day1_trebuchet::solve },
    Day{ dir: "day2_cube_conundrum", solve: day2_cube_conundrum::solve },
    Day{ dir: "day3_gear_ratios", solve: day3_gear_ratios::solve },
    Day{ dir: "day4_scratchcards", solve: day4_scratchcards::solve },
    Day{ dir: "day5_seed_fertilizer", solve: day5_seed_fertilizer::solve },
    Day{ dir: "day6_wait_for_it", solve: day6_wait_for_it::solve },
    Day{ dir: "day7_camel_cards", solve: day7_camel_cards::solve },
    Day{ dir: "day8_haunted_wasteland", solve: day8_haunted_wasteland::solve },
    Day{ dir: "day9_mirage_maintenance", solve: day9_mirage_maintenance::solve },
    Day{ dir: "day10_pipe_maze", solve: day10_pipe_maze::solve },
    Day{ dir: "day11_cosmic_expansion", solve: day11_cosmic_expansion::solve },
    Day{ dir: "day12_hot_springs", solve: day12_hot_springs::solve },
];

fn main() {
    let args = CliArgs::parse();
    // reject "--all" and explicit puzzle numbers
    if args.all && !args.puzzle.is_empty() {
        let mut cmd = CliArgs::command();
        cmd.error(clap::error::ErrorKind::ArgumentConflict,
            "Cannot use --all and explicit puzzle numbers.")
            .exit();
    }
    let rootdir = find_root_dir(&DAYS[0].dir);
    if let Err(e) = rootdir {
        eprintln!("Cannot find path to exercises: {:?}", e);
        exit(2);
    }
    let rootdir = rootdir.unwrap();
    // which puzzles to run
    if args.all {
        run_puzzles(rootdir, &args, &DAYS, YEAR);
    } else if !args.puzzle.is_empty() {
        run_puzzles(rootdir, &args, &to_days(&args.puzzle, &DAYS), YEAR);
    } else {
        let puzzle = current_puzzle(&DAYS);
        match puzzle {
            Ok(d) => run_puzzles(rootdir, &args, d, YEAR),
            Err(e) if e.kind() == ErrorKind::NotFound => run_puzzles(rootdir, &args, &DAYS[DAYS.len()-1..], YEAR),
            Err(e) => {
                eprintln!("Error searching for puzzle from current dir: {e}");
                exit(1);
            },
        };
    }
}
