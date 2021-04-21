use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

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
                .replace("\\", "_")
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
