use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
use chrono::{offset::TimeZone, DateTime, Datelike, Local};
use chrono::{NaiveDateTime, ParseResult};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;

use clap::{crate_authors, crate_version, ArgEnum, Clap};

use chrono::format::{Parsed, StrftimeItems};
use console::style;

// TODO, homogenisieren der Logfiles ...
// pushd logs
// for /R %%G in ( *.7z ) do pushd %%~dpG & 7z x "%%G" -y & popd
// for /R %%G in (*tpsw32.log) do iconv -s -f "CP1252" -t UTF-8 "%%G" > "%%G.log1" && mv -f "%%G.log1" "%%G"
// for /R %%G in (*thinmon.log) do iconv -s -f "CP1252" -t UTF-8 "%%G" > "%%G.log1" && mv -f "%%G.log1" "%%G"
// popd

// Setlocal EnableDelayedExpansion
// for /R %%G in (*.log) do (
//     for /F %%D in ('file -bi "%%G" ^| awk -F "=" "{print $2}"') do (
//         if [%%D] == [utf-16le] (
//             echo %%G %%D ---- %%G.log1
//             iconv -s -f "%%D" -t UTF-8 "%%G" | tr "\r" "\n" | tr -s "\n" "\n"  | tr -d \0 | sed "1s/^\xEF\xBB\xBF//" ^
//             | sed -r "s/^([0-9]{2}):([0-9]{2}):([0-9]{2})[:.]([0-9]{3}) +([0-9]{2})[.-]([0-9]{2})[.-]([0-9]{4})/\5-\6-\7 \1:\2:\3:\4/" ^
//             | sed -r "s/^([0-9]{2})[.-]([0-9]{2})[.-]([0-9]{4}) +([0-9]{2}):([0-9]{2}):([0-9]{2})[:.]([0-9]{3})/\3-\2-\1 \4:\5:\6:\7/" ^
//             | sed -r "s/^([0-9]{4})[.-]([0-9]{2})[.-]([0-9]{2}) +([0-9]{2}):([0-9]{2}):([0-9]{2})[:.]([0-9]{3})/\1-\2-\3 \4:\5:\6:\7/" ^
//             | sed -r "s/^([0-9]{2})\/([0-9]{2}) +([0-9]{2}):([0-9]{2}):([0-9]{2})[:.]([0-9]{3})/2021-\1-\2 \3:\4:\5:\6/" ^
//             > "%%G.log1"
//         ) ELSE (
//             echo make \n and delete zero charcters
//             tr "\r" "\n" < "%%G" | tr -d \0 | tr -s "\n" "\n" | sed "1s/^\xEF\xBB\xBF//" ^
//             | sed -r "s/^([0-9]{2}):([0-9]{2}):([0-9]{2})[:.]([0-9]{3}) +([0-9]{2})[.-]([0-9]{2})[.-]([0-9]{4})/\5-\6-\7 \1:\2:\3:\4/" ^
//             | sed -r "s/^([0-9]{2})[.-]([0-9]{2})[.-]([0-9]{4}) +([0-9]{2}):([0-9]{2}):([0-9]{2})[:.]([0-9]{3})/\3-\2-\1 \4:\5:\6:\7/" ^
//             | sed -r "s/^([0-9]{4})[.-]([0-9]{2})[.-]([0-9]{2}) +([0-9]{2}):([0-9]{2}):([0-9]{2})[:.]([0-9]{3})/\1-\2-\3 \4:\5:\6:\7/" ^
//             | sed -r "s/^([0-9]{2})\/([0-9]{2}) +([0-9]{2}):([0-9]{2}):([0-9]{2})[:.]([0-9]{3})/2021-\1-\2 \3:\4:\5:\6/" ^
//             > "%%G.log1"
//         )
//         mv -f "%%G.log1" "%%G"
//     )
// )

/// `Command`
///
///
#[derive(ArgEnum, Debug)]
enum Command {
    Count,
    Split,
    Time,
    Diff,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "count" => Ok(Self::Count),
            "split" => Ok(Self::Split),
            "time" => Ok(Self::Time),
            "diff" => Ok(Self::Diff),
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
    ///e.g. --starttime "2021-06-29 13:06:56"
    #[clap(long)]
    starttime: Option<String>,
    /// e.g. --stoptime "2021-06-29 13:18:06"
    #[clap(long)]
    stoptime: Option<String>,
    /// e.g. --subtrahend_regex  "regex"
    #[clap(long)]
    subtrahend_regex: Option<String>,
    /// e.g. --minuend_regex  "regex"
    #[clap(long)]
    minuend_regex: Option<String>,
    #[clap(short, long)]
    file_name: String,
    /// don't forget the () for cature the e.g. process id
    #[clap(short, long)]
    regex: Option<String>,
    #[clap(possible_values(Command::VARIANTS)/*, default_value("count")*/)]
    command: Command,
}

fn write_line_to_key_file(findings: &mut HashMap<String, File>, key: &str, line: &str) {
    let file = findings.entry(key.to_string()).or_insert_with(|| {
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
            .expect("cannot open file")
    });

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

fn timesplit(reader: BufReader<File>, cuthead: &str, cuttail: &str) {
    //let mut findings: HashMap<String, File> = HashMap::new();
    // let mut last_used_file_name: String = "".to_string();

    let time_pattern_client_src = "\\d+-\\d+-\\d+ \\d+:\\d+:\\d+";
    let time_client_regex = Regex::new(time_pattern_client_src).unwrap();

    let limit_head = NaiveDateTime::parse_from_str(cuthead, "%Y-%m-%d %H:%M:%S");
    let limit_tail = NaiveDateTime::parse_from_str(cuttail, "%Y-%m-%d %H:%M:%S");
    let mut last_result: bool = false;

    for (index, line) in reader.lines().enumerate() {
        //  iconv -s -f "CP1252" -t UTF-8 "thinmon.log" > ttt
        if line.is_err() {
            eprintln!("Error in line: {}", index);
        }

        let line = line.unwrap();
        if let Some(caps) = time_client_regex.captures(&line) {
            let timestring = caps.get(0).map_or("", |m| m.as_str());

            if let Ok(linetime) = NaiveDateTime::parse_from_str(&timestring, "%Y-%m-%d %H:%M:%S") {
                let result = match (limit_head, limit_tail) {
                    (Ok(h), Err(_)) => linetime >= h,
                    (Err(_), Ok(t)) => linetime <= t,
                    (Ok(h), Ok(t)) => linetime >= h && linetime <= t,
                    (Err(_), Err(_)) => panic!("both dates wrong"),
                };

                if result {
                    println!("{}", &line);
                    last_result = true;
                }
                if let Ok(limit_tail) = limit_tail {
                    if linetime >= limit_tail {
                        break;
                    }
                }
            }
        } else if last_result {
            println!("{}", &line);
        }
    }
}

/// `LogTimeRegex`
///
///
#[derive(Debug)]
struct LogTimeRegex {
    pub regex: Regex,
    pub definition: String,
    pub destination: String,
}

impl Default for LogTimeRegex {
    fn default() -> Self {
        Self {
            regex: Regex::new(r"^(\d+)/(\d+) (\d+):(\d+):(\d+):(\d+)").unwrap(),
            definition: String::default(),
            destination: String::from(r"2021-$1-$2 $3:$4:$5:$6"),
        }
    }
}

fn detect_time_regex(line: &str) -> Option<LogTimeRegex> {
    lazy_static! {
        static ref TIME_CLIENT_REGEX: Regex =
            Regex::new(r"^(\d+)/(\d+) (\d+):(\d+):(\d+):(\d+)").unwrap();
        static ref TIME_THINMON_REGEX: Regex =
            Regex::new(r"^(\d{2}):(\d{2}):(\d{2})[:.](\d{3}) +(\d{2})[.-](\d{2})[.-](\d{4})")
                .unwrap();
        static ref TIME_RENDERSLAVEAGENT_REGEX: Regex =
            Regex::new(r"^(\d{4})[.-](\d{2})[.-](\d{2}) +(\d{2}):(\d{2}):(\d{2})[:.](\d{3})")
                .unwrap();
        static ref TIME_TPPSRV_REGEX: Regex =
            Regex::new(r"^(\d{2})[.-](\d{2})[.-](\d{4}) +(\d{2}):(\d{2}):(\d{2})[:.](\d{3})")
                .unwrap();
    }

    if TIME_CLIENT_REGEX.is_match(line) {
        Some(LogTimeRegex {
            regex: TIME_CLIENT_REGEX.clone(),
            definition: String::from("%m/%d %H:%M:%S:%3f"),
            destination: String::from(r"2021-$1-$2 $3:$4:$5:$6"),
        })
    } else if TIME_THINMON_REGEX.is_match(line) {
        Some(LogTimeRegex {
            regex: TIME_THINMON_REGEX.clone(),
            definition: String::from("%H:%M:%S.%3f %d.%m.%Y"),
            destination: String::from(r"$7-$6-$5 $1:$2:$3:$4"),
        })
    } else if TIME_RENDERSLAVEAGENT_REGEX.is_match(line) {
        Some(LogTimeRegex {
            regex: TIME_RENDERSLAVEAGENT_REGEX.clone(),
            definition: String::from("%Y.%m.%d %H:%M:%S.%3f"),
            destination: String::from(r"$1-$2-$3 $4:$5:$6:$7"),
        })
    } else if TIME_TPPSRV_REGEX.is_match(line) {
        Some(LogTimeRegex {
            regex: TIME_TPPSRV_REGEX.clone(),
            definition: String::from("%d.%m.%Y %H:%M:%S.%3f"),
            destination: String::from(r"$3-$2-$1 $4:$5:$6:$7"),
        })
    } else {
        None
    }
}

fn smart_parse_naive_date_time_from_str(s: &str, strftime: &str) -> ParseResult<NaiveDateTime> {
    let mut parsed = Parsed::new();
    chrono::format::parse(&mut parsed, s, StrftimeItems::new(strftime));
    match parsed.year {
        // we have all values needed...
        Some(_) => parsed,

        // we are missing the year value. just set it to todays year
        None => {
            parsed.set_year(Local::now().year() as i64);
            parsed
        }
    }
    .to_naive_datetime_with_offset(0)
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    pub fn date_tests() {
        let date = "07/15 20:38:15:123456789";
        let detected = detect_time_regex(date).expect("date not recognized");

        println!(
            "{:?}",
            smart_parse_naive_date_time_from_str(date, &detected.definition)
                .map_err(|e| e.to_string())
        );
        assert_eq!(
            NaiveDateTime::new(
                NaiveDate::from_ymd(Local::now().year(), 7, 15),
                NaiveTime::from_hms_milli(20, 38, 15, 123)
            ),
            smart_parse_naive_date_time_from_str(date, &detected.definition).unwrap()
        );
        assert_ne!(
            NaiveDateTime::new(
                NaiveDate::from_ymd(Local::now().year(), 7, 15),
                NaiveTime::from_hms_milli(20, 38, 15, 321)
            ),
            smart_parse_naive_date_time_from_str(date, &detected.definition).unwrap()
        );
    }
}

fn timediff(
    reader: BufReader<File>,
    subtrahendregex: &str,
    minuendregex: &str,
    log_file_name: &str,
) {
    let mut time_convention = None::<LogTimeRegex>;
    let default_time_conv = LogTimeRegex::default();

    let subtrahend_regex = Regex::new(subtrahendregex).unwrap();
    let minuend_regex = Regex::new(minuendregex).unwrap();

    let mut date_time: Option<DateTime<Local>> = None;

    for (index, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => panic!("Error in line: {} ({})", index, e),
        };

        time_convention = time_convention.or_else(|| detect_time_regex(&line));
        if time_convention.is_none() {
            // println!("? [ {}:{} ] {}", log_file_name, index + 1, &line);
        }

        let time_conv = time_convention.as_ref().unwrap_or(&default_time_conv);
        if let Some(caps) = time_conv.regex.captures(&line) {
            let raw_time_string = caps.get(0).map_or("", |m| m.as_str()).to_owned();
            let complete_timestring = time_conv
                .regex
                .replace(&raw_time_string, time_conv.destination.as_str())
                /*+ "000000"*/;

            if let Ok(linetime) =
                NaiveDateTime::parse_from_str(&complete_timestring, "%Y-%m-%d %H:%M:%S:%3f")
            {
                if minuend_regex.is_match(&line) {
                    println!("[ {}:{} ] {}", log_file_name, index + 1, &line);
                    date_time = Some(Local.from_local_datetime(&linetime).unwrap());
                } else if date_time.is_some() && subtrahend_regex.is_match(&line) {
                    let end_time: DateTime<Local> = Local.from_local_datetime(&linetime).unwrap();
                    let resulttime = end_time.signed_duration_since(date_time.unwrap());
                    println!(
                        "{} [ {}:{} ] {}",
                        resulttime,
                        log_file_name,
                        index + 1,
                        line
                    );
                    // date_time = None;
                }
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
            found_all += 1;
            count_findings
                .entry(cap[1].to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
    }

    let mut sorted: Vec<(_, _)> = count_findings.iter().collect();
    sorted.sort_by(|a, b| a.1.cmp(b.1).reverse());

    if !count_findings.is_empty() {
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
    let time_pattern_client_src = opts.regex.unwrap_or_default();
    let cuthead = opts.starttime.unwrap_or_default();
    let cuttail = opts.stoptime.unwrap_or_default();
    let minuend_regex = opts.minuend_regex.unwrap_or_default();
    let subtrahend_regex = opts.subtrahend_regex.unwrap_or_default();
    let command = opts.command;

    let file = File::open(&filename).unwrap();
    let reader = BufReader::new(file);
    let re = Regex::new(&time_pattern_client_src).unwrap();

    match command {
        Command::Count => count(reader, re, args),
        Command::Split => split(reader, re),
        Command::Time => timesplit(reader, &cuthead, &cuttail),
        Command::Diff => timediff(reader, &subtrahend_regex, &minuend_regex, &filename),
    }
}
// extern crate regex;
// use regex::Regex;

// fn main1() {
//     let rg = Regex::new(r"(\d+)").unwrap();

//     // Regex::replace replaces first match
//     // from it's first argument with the second argument
//     // => Some string with numbers (not really)
//     rg.replace("Some string with numbers 123", "(not really)");

//     // Capture groups can be accesed via $number
//     // => Some string with numbers (which are 123)
//     rg.replace("Some string with numbers 123", "(which are $1)");

//     let rg = Regex::new(r"(?P<num>\d+)").unwrap();

//     // Named capture groups can be accesed via $name
//     // => Some string with numbers (which are 123)
//     rg.replace("Some string with numbers 123", "(which are $num)");

//     // Regex::replace_all replaces all the matches, not only the first
//     // => Some string with numbers (not really) (not really)
//     rg.replace_all("Some string with numbers 123 321", "(not really)");
// }
