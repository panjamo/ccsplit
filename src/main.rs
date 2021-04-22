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

use console::style;

/// `Command`
///
///
#[derive(ArgEnum, Debug)]
enum Command {
    Count,
    Split,
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
    fn default() -> Self {
        Self::Count
    }
}

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    // #[clap(short, long)]
    // split: bool,
    // #[clap(short, long)]
    // count: bool,
    #[clap(short, long)]
    file_name: String,
    #[clap(short, long)]
    regex: String,
    #[clap(possible_values(Command::VARIANTS)/*, default_value("count")*/)]
    command: Command,
}

fn split (reader: BufReader<File>, re: Regex) {
    let mut findings: HashMap<String, File> = HashMap::new();

    for (index, line) in reader.lines().enumerate() {
        //  iconv -s -f "CP1252" -t UTF-8 "thinmon.log" > ttt
        if line.is_err() {
            println!("Error in line: {}", index);
        }
        let line = line.unwrap();
        for cap in re.captures_iter(&line) {
            let rawfilename = format!("{}.log", &cap[1]);
            // println!("{}.{} open file", _index + 1, dfilename);

            let file = findings.entry(rawfilename.to_string()).or_insert(
                OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(
                        &rawfilename
                            .replace("/", "_")
                            .replace(":", "_")
                            .replace("?", "_")
                            .replace("*", "_")
                            .replace(r"\", "_")
                            .replace("\"", "_"),
                    )
                    .expect("cannot open file"),
            );

            file.write_all(line.as_bytes())
                .and(file.write_all(b"\n"))
                .expect("write failed");
            // println!("{}.{}: {}", index + 1, dfilename, line);
        }
    }
}

fn count (reader: BufReader<File>, re: Regex, args: Vec<String>) {
    let mut findings: HashMap<String, u32> = HashMap::new();

    for (index, line) in reader.lines().enumerate() {
        //  iconv -s -f "CP1252" -t UTF-8 "thinmon.log" > ttt
        if line.is_err() {
            println!("Error in line: {}", index);
        }
        let line = line.unwrap();
        for cap in re.captures_iter(&line) {
            findings.entry(cap[1].to_string()).and_modify(|c| *c = *c + 1).or_insert(1);
        }
    }

    let mut sorted: Vec<(_, _)> = findings.iter().collect();
    sorted.sort_by(|a, b| a.1.cmp(b.1).reverse());

    if findings.len() > 0 {
        println!("{:?}", &args[1..]);
        println!("found {} unique results", findings.len());
        // for (name, count) in findings {
        for (name, count) in sorted {
                println!("{cnt:>5} {n}", cnt = style(format!("{}", count)).yellow(), n = name);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let opts = Opts::parse();
    let filename = opts.file_name;
    let splitregex = &opts.regex;
    let command = opts.command;

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let re = Regex::new(splitregex).unwrap();

    match command {
        Command::Count => count (reader, re, args),
        Command::Split => split (reader, re),
    }
    
}
