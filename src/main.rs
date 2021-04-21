use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;

use clap::{crate_authors, crate_version, ArgEnum, Clap};


/// `Command`
///
///
#[derive(ArgEnum, Debug)]
enum Command {
    Count,
    Split
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "count" => Ok(Self::Count),
            "split" => Ok(Self::Split),
            invalid => Err(format!("{} is an invalid command", invalid)),
        }
    }
}

impl Default for Command {
    fn default() -> Self { Self::Count }
}

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    // #[clap(short, long)]
    // split: bool,
    // #[clap(short, long)]
    // count: bool,
    #[clap(short, long)]
    file_name: Option<String>,
    #[clap(short, long)]
    regex: Option<String>,
    #[clap(possible_values(Command::VARIANTS)/*, default_value("count")*/)]
    command: Command,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = &args[1];
    let splitregex = &args[2];

    // Open the file in read-only mode (ignoring errors).
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let re = Regex::new(splitregex).unwrap();
    let mut open_files: HashMap<String, File> = HashMap::new();

    for (index, line) in reader.lines().enumerate() {
        //  iconv -s -f "CP1252" -t UTF-8 "thinmon.log" > ttt
        if line.is_err() {
            println!("Error in line: {}", index);
        }
        let line = line.unwrap();
        for cap in re.captures_iter(&line) {
            let rawfilename = format!("{}.log", &cap[1]);
            let dfilename = &rawfilename
                .replace("/", "_")
                .replace(":", "_")
                .replace("?", "_")
                .replace("*", "_")
                .replace(r"\", "_")
                .replace("\"", "_");
            // println!("{}.{} open file", _index + 1, dfilename);

            let file = open_files.entry(dfilename.to_string()).or_insert(
                OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(dfilename)
                    .expect("cannot open file"),
            );

            file.write_all(line.as_bytes()).expect("write failed");
            file.write_all("\n".as_bytes()).expect("write failed");
            // println!("{}.{}: {}", index + 1, dfilename, line);
        }
    }
}
