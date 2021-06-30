use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;
use chrono::{NaiveDateTime};

use clap::{crate_authors, crate_version, ArgEnum, Clap};

use console::style;

/// `Command`
///
///
#[derive(ArgEnum, Debug)]
enum Command {
    Count,
    Split,
    Time,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "count" => Ok(Self::Count),
            "split" => Ok(Self::Split),
            "time" => Ok(Self::Time),
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
    ///e.g. --starttime "2021-06-29 13:06:56"
    #[clap(short, long)]
    starttime: Option<String>,
    /// e.g. --stoptime "2021-06-29 13:18:06"
    #[clap(short, long)]
    stoptime: Option<String>,
    #[clap(short, long)]
    file_name: String,
    /// don't forget the () for cature the e.g. process id
    #[clap(short, long)]
    regex: Option<String>,
    #[clap(possible_values(Command::VARIANTS)/*, default_value("count")*/)]
    command: Command,
}

fn write_line_to_key_file(findings: &mut HashMap<String, File>, key: &str, line: &str) {

    let file = findings.entry(key.to_string()).or_insert(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(
                &key.replace(">", "_")
                    .replace("<", "_")
                    .replace(":", "_")
                    .replace("?", "_")
                    .replace("*", "_")
                    .replace(r"/", "_")
                    .replace(r"\", "_")
                    .replace(r"|", "_")
                    .replace("\"", "_"),
            )
            .expect("cannot open file"),
    );

    file.write_all(line.as_bytes())
        .and(file.write_all(b"\n"))
        .expect("write failed");
}

fn split(reader: BufReader<File>, re: Regex) {
    let mut findings: HashMap<String, File> = HashMap::new();
    let mut last_used_file_name: String = "".to_string();

    for (index, line) in reader.lines().enumerate() {
        //  iconv -s -f "CP1252" -t UTF-8 "thinmon.log" > ttt
        if line.is_err() {
            println!("Error in line: {}", index);
        }
        let line = line.unwrap();
        if let Some(caps) = re.captures(&line) {
            let cap = caps.get(1).map_or("", |m| m.as_str());
            let rawfilename = format!("{}.log", cap);
            last_used_file_name = rawfilename.to_string();
            write_line_to_key_file(&mut findings, &rawfilename, &line);
        } else {
            if last_used_file_name.is_empty() {
                continue;
            }
            write_line_to_key_file(&mut findings, &last_used_file_name, &line);
        }
    }
}

fn timesplit(reader: BufReader<File>, cuthead: &str, cuttail: &str ) {
    //let mut findings: HashMap<String, File> = HashMap::new();
    // let mut last_used_file_name: String = "".to_string();

    let splitregex = "\\d+-\\d+-\\d+ \\d+:\\d+:\\d+";
    let reparam = Regex::new(splitregex).unwrap();

    let limit_head = NaiveDateTime::parse_from_str(cuthead, "%Y-%m-%d %H:%M:%S");
    let limit_tail = NaiveDateTime::parse_from_str(cuttail, "%Y-%m-%d %H:%M:%S");
    let mut last_result: bool = false;

    for (index, line) in reader.lines().enumerate() {
        //  iconv -s -f "CP1252" -t UTF-8 "thinmon.log" > ttt
        if line.is_err() {
            eprintln!("Error in line: {}", index);
        }

        let line = line.unwrap();
        if let Some(caps) = reparam.captures(&line) {
            let timestring = caps.get(0).map_or("", |m| m.as_str());

            if let Ok(linetime) = NaiveDateTime::parse_from_str(&timestring, "%Y-%m-%d %H:%M:%S") {

                let result = match (limit_head, limit_tail) {
                    (Ok(h),Err(_)) => linetime >= h,
                    (Err(_),Ok(t)) => linetime <= t,
                    (Ok(h),Ok(t)) => linetime >= h && linetime <= t,
                    (Err(_),Err(_)) => panic!("both dates wrong"),
                };

                if result {
                    println!("{}", &line);
                    last_result = true;
                }
                if let Ok(limit_tail)=limit_tail {
                    if  linetime >= limit_tail {
                        break;
                    }
                }
            }
        } else {
            if last_result {
                println!("{}", &line);
            }
        }
    }
}

fn count(reader: BufReader<File>, re: Regex, args: Vec<String>) {
    let mut count_findings: HashMap<String, u32> = HashMap::new();
    let mut found_all = 0;

    for (index, line) in reader.lines().enumerate() {
        //  iconv -s -f "CP1252" -t UTF-8 "thinmon.log" > ttt
        if line.is_err() {
            println!("Error in line: {}", index);
        }
        let line = line.unwrap();
        for cap in re.captures_iter(&line) {
            found_all = found_all + 1;
            count_findings
                .entry(cap[1].to_string())
                .and_modify(|c| *c = *c + 1)
                .or_insert(1);
        }
    }

    let mut sorted: Vec<(_, _)> = count_findings.iter().collect();
    sorted.sort_by(|a, b| a.1.cmp(b.1).reverse());

    if count_findings.len() > 0 {
        println!("{:?}", &args[1..]);
        println!("found {} overall results", found_all);
        println!("found {} unique results", count_findings.len());
        for (name, count) in sorted {
            println!(
                "{cnt:>5} {n}",
                cnt = style(format!("{}", count)).yellow(),
                n = name
            );
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let opts = Opts::parse();
    let filename = opts.file_name;
    let splitregex = opts.regex.unwrap_or_default();
    let cuthead = opts.starttime.unwrap_or_default();
    let cuttail = opts.stoptime.unwrap_or_default();
    let command = opts.command;

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let re = Regex::new(&splitregex).unwrap();

    match command {
        Command::Count => count(reader, re, args),
        Command::Split => split(reader, re),
        Command::Time => timesplit(reader, &cuthead, &cuttail),
    }
}
