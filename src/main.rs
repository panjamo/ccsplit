use std::env;
use std::io::{BufRead, BufReader};
use regex::Regex;
use std::fs::File;
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
// use std::collections::HashMap;
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

    for (_index, line) in reader.lines().enumerate() {
        let line = line.unwrap(); // Ignore errors.
        for cap in re.captures_iter(&line) {
            let rawfilename = format!("{}.log", &cap[1]);
            let dfilename = &rawfilename.replace("/", "_").replace(":", "_").replace("?", "_").replace("*", "_").replace("\\", "_").replace("\"", "_");
            println!("{}.{} open file", _index + 1, dfilename);

            let mut file = OpenOptions::new().append(true).create(true).open(dfilename).expect(
                "cannot open file");
            file.write_all(line.as_bytes()).expect("write failed");
            file.write_all("\n".as_bytes()).expect("write failed");
            // println!("{}.{}: {}", index + 1, dfilename, line);
        }
    }
}