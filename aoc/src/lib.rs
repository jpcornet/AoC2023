use std::collections::{HashMap, hash_map::Entry};
use std::io::{Read, BufReader, ErrorKind};
use std::fs::File;
use std::path::PathBuf;
use std::{fs, env};
use std::os::unix::fs::MetadataExt;
use std::time::Duration;
use std::process::exit;
use reqwest;
use clap::{Args, Parser};
use comfy_table::Table;
use comfy_table::presets::UTF8_FULL;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use exrunner::{ExRunner, duration_format, ExCtx};

/// command line tool to run Advent of Code puzzles and display output and timings
///
/// This tool will run the Advent of Code puzzles, by default the latest one or the
/// one given on the command line, or the one in the subdirectory where you are.
/// Will give "raw" output for individual puzzles or present the results in a table,
/// together with timing info.
#[derive(Parser, Debug)]
#[command(author, version = None)]
pub struct CliArgs {
    /// Run all puzzles
    #[arg(short, long)]
    pub all: bool,

    /// input file name (default: input.txt)
    #[arg(short, long)]
    pub input: Option<String>,

    #[command(flatten)]
    format: OutputFormat,

    /// which puzzle(s) to run
    pub puzzle: Vec<u32>,
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
struct OutputFormat {
    /// Output raw. Default unless --all is given.
    #[arg(short, long)]
    raw: bool,

    /// Output in table form.
    #[arg(short, long, group="format")]
    table: bool,
}

#[derive(Clone)]
pub struct Day {
    pub dir: &'static str,
    // Need to specify the specific type of BufReader<File> here, because function
    // pointers to generic functions do not exist.
    pub solve: fn(BufReader<File>, &mut ExRunner),
}

// returns the first number in a string
fn first_number<'a>(input: &'a str) -> &'a str {
    let start_off = input.find(|c: char| c.is_ascii_digit());
    if start_off.is_none() {
        return "";
    }
    let start_off = start_off.unwrap();
    let end_off = input[start_off..].find(|c: char| !c.is_ascii_digit()).unwrap_or(input[start_off..].len());
    &input[start_off..start_off+end_off]
}

// convert list of puzzle numbers to Vec of Day structures.
pub fn to_days(puzzle: &Vec<u32>, days: &[Day]) -> Vec<Day> {
    // keep hash of puzzle number and index
    let mut puzzle_pos: HashMap<u32, Option<usize>> = HashMap::new();
    for (index, d) in days.iter().enumerate() {
        let puzzlenum: u32 = first_number(d.dir).parse().expect(&format!("Cannot find puzzle number in {}", d.dir));
        assert!(!puzzle_pos.contains_key(&puzzlenum), "Duplicate puzzle number");
        puzzle_pos.insert(puzzlenum, Some(index));
    }
    let mut result: Vec<Day> = Vec::new();
    for p in puzzle {
        match puzzle_pos.get(p) {
            None => { eprintln!("Puzzle number {p} does not exist"); exit(1); },
            Some(Some(i)) => { result.push(days[*i].clone()); puzzle_pos.insert(*p, None); },
            Some(None) => { eprintln!("Trying to run puzzle {p} twice?"); exit(1); },
        };
    }
    result
}

// Convert current directory to Day ref, or error if not found.
pub fn current_puzzle(days: &'static [Day]) -> std::io::Result<&'static [Day]> {
    let curdir = env::current_dir()?;
    let curdir_str = curdir.to_string_lossy() + "/";
    for (index, d) in days.into_iter().enumerate() {
        if curdir_str.contains(&format!("/{}/", d.dir)) {
            return Ok(&days[index..=index]);
        }
    }
    Err(std::io::Error::new(ErrorKind::NotFound, "Current directory is not a puzzle"))
}

// run a list of puzzles
pub fn run_puzzles(rootdir: PathBuf, args: &CliArgs, days: &[Day], year: u16) {
    let defaultinput = String::from("input.txt");
    let inputfile  = args.input.as_ref().unwrap_or(&defaultinput);
    // determine output format, raw or table
    let f_raw;
    let f_table;
    if args.format.raw || args.format.table {
        f_raw = args.format.raw;
        f_table = args.format.table;
    } else if args.all {
        f_table = true;
        f_raw = false;
    } else {
        f_table = false;
        f_raw = true;
    }
    let mut table = Table::new();
    if f_table {
        table.load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS);
    }
    table.set_header(vec!["", "part1", "part2", "parse", "time1", "time2", "close"]);
    let mut total_time = Duration::from_secs(0);
    for (index, d) in days.iter().enumerate() {
        let mut fname = rootdir.clone();
        fname.push(d.dir);
        fname.push("input");
        fname.push(inputfile);
        let meta = fs::metadata(&fname);
        match meta {
            Err(e) if e.kind() == ErrorKind::NotFound && args.input.is_none() => download_input(&rootdir, &d.dir, &fname, year),
            Err(e) => panic!("Error fetching {}: {e}", fname.to_string_lossy()),
            Ok(m) if !m.is_file() => panic!("{} is not a file, but a {:?}", fname.to_string_lossy(), m),
            _ => (),
        };
        let fh = File::open(&fname);
        if let Err(e) = fh {
            eprintln!("Error: cannot open file {} for exercise {}: {e}", fname.to_string_lossy(), d.dir);
            continue;
        }
        let mut ct = ExCtx::new(d.solve, BufReader::new(fh.unwrap()));
        if f_raw {
            ct.with_stderr();
        }
        let er = ct.do_run(d.dir.to_string());
        total_time += er.totaltime().unwrap_or(Duration::from_secs(0));
        if f_raw {
            if index > 0 {
                println!("---");
            }
            er.print_raw();
        }
        if f_table {
            let mut row = vec![d.dir.to_string()];
            let mut answers: Vec<String> = er.answ().into_iter().map(|x| x.unwrap_or(String::from(""))).collect();
            row.append(&mut answers);
            let mut times: Vec<String> = [er.parsetime(), er.time1(), er.time2(), er.cleanuptime()].iter()
                .map(|x| if let Some(d) = x { duration_format(d) } else { String::from("") }).collect();
            row.append(&mut times);
            table.add_row(row);
        }
    }
    if f_table {
        println!("{table}");
    }
    if days.len() > 1 {
        if f_raw {
            println!("===");
        }
        println!("Total puzzles runtime: {:?}", total_time);
    }
}

// download input to puzzle
fn download_input(rootdir: &PathBuf, dirname: &str, target: &PathBuf, year: u16) {
    let session_cookie = match get_session_cookie(rootdir) {
        Err(e) => panic!("No input file, and no session cookie found: {e}"),
        Ok(s) => format!("session={s}"),
    };
    let daynum = first_number(dirname);
    let url = format!("https://adventofcode.com/{year}/day/{daynum}/input");
    let client = reqwest::blocking::Client::new();
    let res = client.get(&url)
        .header(reqwest::header::COOKIE, session_cookie)
        .send();
    let mut resp = match res {
        Err(e) => panic!("Cannot download input from {url}: {e}"),
        Ok(resp) if !resp.status().is_success() => panic!("Error downloading input from {url}: {}", resp.status()),
        Ok(resp) => resp,
    };
    // make sure output directory exists. Create it if not
    let targetdir = target.parent().unwrap().clone();
    let meta = fs::metadata(targetdir);
    if meta.is_err() && meta.err().unwrap().kind() == ErrorKind::NotFound {
        println!("Creating input directory {}", targetdir.to_string_lossy());
        fs::create_dir(targetdir).expect("Cannot create input directory");
    }
    let mut fh = match File::options().write(true).create_new(true).open(target) {
        Err(e) => panic!("Cannot create {}: {e}", target.to_string_lossy()),
        Ok(f) => f,
    };
    println!("Downloading input from {url}");
    resp.copy_to(&mut fh).expect("Error reading from URL writing to example input");
}

fn get_session_cookie(rootdir: &PathBuf) -> std::io::Result<String> {
    match env::var("SESSION_COOKIE") {
        Ok(s) => return Ok(s),
        _ => (),
    };
    let mut cookiefile = rootdir.clone();
    cookiefile.push("session.cookie");
    let mut fh = File::open(cookiefile)?;
    let mut contents = String::new();
    fh.read_to_string(&mut contents)?;
    Ok(contents.trim().to_string())
}

// libc-specific: get access to uid
#[link(name="c")]
extern "C" {
    fn geteuid() -> u32;
}

// Find the root directory of the puzzles by looking up from the current dir, or from the directory of the binary
pub fn find_root_dir(target: &str) -> std::io::Result<PathBuf> {
    // only look at directories that are owned by the current user, so get euid.
    let uid: u32;
    unsafe { uid = geteuid(); }
    // maintain a hash of directories that we looked at
    let mut seen: HashMap<PathBuf, ()> = HashMap::new();
    let root_dir =
        find_in_ancestors(env::current_dir()?, target, uid, &mut seen).or_else(|_|
            // search again, from program
            find_in_ancestors(PathBuf::from(env::args().next().unwrap()).canonicalize()?, target, uid, &mut seen))?;
    return Ok(root_dir);
}

// find a subdirectory somewhere in the current dir or one of the directories above, only checking directories owned by the given uid.
fn find_in_ancestors(startdir: PathBuf, target: &str, uid: u32, seen: &mut HashMap<PathBuf, ()>) -> std::io::Result<PathBuf> {
    // try_find would make this a bit cleaner, but that's only in nightly at the moment.
    for d in startdir.ancestors() {
        // verify d is a directory and is owned by the right user
        let attr = fs::metadata(d)?;
        // if it's not a directory, skip it.
        if !attr.is_dir() {
            continue;
        }
        // if it has the wrong uid, stop immediately
        if attr.uid() != uid {
            return Err(std::io::Error::new(ErrorKind::PermissionDenied, "Cannot find target directory"));
        }
        // if it's in the seen map, we have seen this dir already and we can abort with a "not found"
        let e = seen.entry(d.to_path_buf());
        if let Entry::Occupied(_) = e {
            return Err(std::io::Error::new(ErrorKind::NotFound, "Cannot find target directory"));
        }
        // try to access the target dir and see if it exists
        let mut targetdir = d.to_path_buf();
        targetdir.push(target);
        let t_attr = fs::metadata(&targetdir);
        if t_attr.is_ok() && t_attr.as_ref().unwrap().is_dir() && t_attr.as_ref().unwrap().uid() == uid {
            // target dir exists, so parent is root dir we want.
            return Ok(d.to_path_buf());
        }
        // mark that we've searched this path, and continue up the tree
        e.or_insert(());
    }
    // if we get here, we didn't find it
    Err(std::io::Error::new(ErrorKind::NotFound, "Searched all the way to the top, nothing found"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_number() {
        assert_eq!(first_number("foo42bar"), "42");
        assert_eq!(first_number("123"), "123");
        assert_eq!(first_number("yolo"), "");
    }
}
