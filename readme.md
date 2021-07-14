```
ccsplit 0.1.0
Jochen Schieberlein <jochen.schieberlein@thinprint.com>

USAGE:
    ccsplit [OPTIONS] <command> --file-name <file-name>

ARGS:
    <command>    [possible values: count, split, time, diff]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --file-name <file-name>                  
        --minuend-regex <minuend-regex>          e.g. --minuend_regex  "regex"
    -r, --regex <regex>                          don't forget the () for cature the e.g. process id
        --starttime <starttime>                  e.g. --starttime "2021-06-29 13:06:56"
        --stoptime <stoptime>                    e.g. --stoptime "2021-06-29 13:18:06"
        --subtrahend-regex <subtrahend-regex>    e.g. --subtrahend_regex  "regex"
```