use std::env;
use std::io::{BufRead, BufReader};
use regex::Regex;
use std::fs::File;
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

    // Read the file line by line using the lines() iterator from std::io::BufRead.
    for (_index, line) in reader.lines().enumerate() {
        let line = line.unwrap(); // Ignore errors.
        if re.is_match(&line)
        {
            for cap in re.captures_iter(&line) {
                let rawfilename = format!("{}.log", &cap[1]);
                let dfilename = &rawfilename.replace("/", "_").replace(":", "_").replace("?", "_").replace("*", "_").replace("\\", "_").replace("\"", "_");
                println!("{}.{} open file", _index + 1, dfilename);

                let fileexsits = std::path::Path::new(dfilename).exists();
                if fileexsits == false {
                    let  _file_out = std::fs::File::create(dfilename).unwrap();
                }
                let mut file = OpenOptions::new().append(true).open(dfilename).expect(
                    "cannot open file");
                file.write_all(line.as_bytes()).expect("write failed");
                file.write_all("\n".as_bytes()).expect("write failed");
                // println!("{}.{}: {}", index + 1, dfilename, line);
            }
        }
    }
}