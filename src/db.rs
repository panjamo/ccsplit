use rusqlite::Error as SqlError;
use rusqlite::{Connection, Result};

use std::fs::File;
use std::io::Read;
use std::iter::FromIterator;

use regex::{Captures, Match, Regex};
use std::path::Path;

#[derive(Debug)]
pub enum DatabaseErr {
    Io(std::io::Error),
    Sql(SqlError),
    Regex(regex::Error),
}

impl From<std::io::Error> for DatabaseErr {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<SqlError> for DatabaseErr {
    fn from(e: SqlError) -> Self {
        Self::Sql(e)
    }
}

impl From<regex::Error> for DatabaseErr {
    fn from(e: regex::Error) -> Self {
        Self::Regex(e)
    }
}

/// `Queryable`
///
///
struct Queryable {
    file: String,
    log_name: String,
}

impl Queryable {
    pub fn new(file: impl AsRef<str>, log_name: impl AsRef<str>) -> Self {
        Self {
            file: file.as_ref().to_owned(),
            log_name: log_name.as_ref().to_owned(),
        }
    }

    pub fn import_from_file(
        &self,
        mut file: impl AsRef<Path>,
        regex_str: impl AsRef<str>,
    ) -> Result<usize, DatabaseErr> {
        let mut buf = String::new();
        File::open(file)
            .and_then(|mut f| f.read_to_string(&mut buf).map(|_| buf))
            .map_err(Into::into)
            .and_then(|ref content| self.import_from_str(content, regex_str))
    }

    pub fn import_from_str(
        &self,
        s: impl AsRef<str>,
        regex_str: impl AsRef<str>,
    ) -> Result<usize, DatabaseErr> {
        let rx = Regex::new(regex_str.as_ref())?;
        let cols = rx
            .capture_names()
            .flat_map(|e| e)
            .map(|n: &str| format!(", {} STRING", n));

        let query = format!(
            "DROP TABLE IF EXISTS {log_name};CREATE TABLE {log_name} (line_no INTEGER PRIMARY KEY{});",
            String::from_iter(cols),
            log_name = &self.log_name,
        );

        Connection::open(&self.file)
            .and_then(|c| c.execute_batch(&query).map(|_| c))
            .map_err(Into::into)
            .and_then(|mut c| self.populate_table_from(&mut c, s.as_ref(), &rx))
    }

    fn populate_table_from(
        &self,
        conn: &mut Connection,
        content: &str,
        regex: &Regex,
    ) -> Result<usize, DatabaseErr> {
        #[inline(always)]
        fn process_line(
            query: &str,
            cols: &[&str],
            c: &Captures,
            conn: &Connection,
        ) -> Result<(), SqlError> {
            let mut stmt = conn.prepare_cached(&query)?;

            cols.iter()
                .map(|n| c.name(n).as_ref().map(Match::as_str))
                .enumerate()
                .for_each(|(i, v)| {
                    stmt.raw_bind_parameter(i + 1, v.unwrap_or_default())
                        .unwrap()
                });

            stmt.raw_execute().map(|_| ())
        }

        let cols: Vec<_> = regex.capture_names().flat_map(|e| e).collect();
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            &self.log_name,
            cols.join(", "),
            std::iter::repeat("?")
                .take(cols.len())
                .collect::<Vec<_>>()
                .join(", ")
        );

        conn.execute_batch("PRAGMA synchronous = OFF;PRAGMA journal_mode = OFF;")?;
        conn.execute_batch("BEGIN TRANSACTION;")?;
        regex
            .captures_iter(content)
            .map(|e| process_line(&query, &cols, &e, conn))
            .collect::<Result<Vec<_>, _>>()
            .and_then(|e| conn.execute_batch("END TRANSACTION;").map(|_| e.len()))
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn ac_database_from_test() {
        // let pattern = r"(?<Date>[^ ]+) (?<Time>[^ ]+) (?<Process>[^ ]+) (?<Thread>[^ ]+ *\(*[^ ]*) (?<Level>_[^_]*_/.) (?<File>[^:]*:[^ ]*) (?<Function>\[[^\]]+\].) (?<User>_[^_]*_) (?<Line>.*)";
        // let pattern = r"(?P<date_time>[[:digit:]]+\.[[:digit:]]+\.[[:digit:]]+ [[:digit:]]+:[[:digit:]]+:[[:digit:]]+:[[:digit:]]+) (Id:(?P<thread_id_old>[[:digit:]]+)|(?P<proc_id>[[:digit:]]+) (?P<thread_id>[[:digit:]]+))( \((?P<originator_thread_id>[[:digit:]]+)\)|) (?P<severity>[[:word:]]+)/[[:word:]=:]*( DLL|) (?P<file_line>[[:word:].]+:[[:digit:]]+)( (?P<signature>\[[[:word:]:~]+\][[:word:]]+)|) (?P<module>[[:word:]]+) (?P<message>.*)";
        let pattern = r"(?P<date_time>\d+\.\d+\.\d+ \d+:\d+:\d+:\d+) (Id:(?P<thread_id_old>\d+)|(?P<proc_id>\d+) (?P<thread_id>\d+))( \((?P<originator_thread_id>\d+)\)|) (?P<severity>\w+)/[^: ]*.( DLL|) (?P<file_line>[\w.]+:\d+)( (?P<signature>\[[^\[\]]+\]\w+)|) (?P<module>\w+) (?P<message>.*)";
        // let pattern = r"(?P<date_time>\d+\.\d+\.\d+ \d+:\d+:\d+:\d+) (?P<proc_id>\d+) (?P<thread_id>\d+) (?P<severity>\w+)/[^:]* (?P<file_line>[\w.]+:\d+) (?P<signature>[^ ]+|) (?P<module>\w+) (?P<message>.*)";

        let q = Queryable::new(r"c:\temp\log_file.db", "TPAutoConnSvc");
        let data = q
            .import_from_file(r"C:\Temp\TPAutoConnSvc.log", pattern)
            .unwrap();
        println!("Added rows: {}\n", data);
        assert_eq!(data, 106268);
    }

    #[test]
    pub fn ac2_database_from_test() {
        let pattern = r"(?P<date_time>\d+\.\d+\.\d+ \d+:\d+:\d+:\d+) (Id:(?P<thread_id_old>\d+)|(?P<proc_id>\d+) (?P<thread_id>\d+))( \((?P<originator_thread_id>\d+)\)|) (?P<severity>\w+)/[^: ]*.( DLL|) (?P<file_line>[\w.]+:\d+)( (?P<signature>\[[^\[\]]+\]\w+)|) (?P<module>\w+) (?P<message>.*)";

        let q = Queryable::new(r"c:\temp\log_file.db", "TPAutoConnect");
        let data = q
            .import_from_file(r"C:\Temp\TPAutoConnect.log", pattern)
            .unwrap();
        println!("Added rows: {}\n", data);
        assert_eq!(data, 7108);
    }

    #[test]
    pub fn clnt_database_from_test() {
        // 07/05 12:32:47:137 11900 4168 _INF_ CommunicationSocketAsync.cpp:332 [CommunicationSocketAsync::HandleOnClose] Socket OnClose Event (DataS 0x2f4)
        let pattern = r"((?P<date_time>\d+/\d+ \d+:\d+:\d+:\d+) (?P<proc_id>\d+) (?P<thread_id>\d+) (?P<severity>\w+) (?P<file_line>[\w.]+:\d+) (?P<signature>\[[^\[\]]+\]) ){0,1}(?P<message>.*)";

        let q = Queryable::new(r"c:\temp\log_file.db", "ThnClnt");
        let data = q
            .import_from_file(r"C:\Temp\client_utf8.log", pattern)
            .unwrap();
        println!("Added rows: {}\n", data);
        assert_eq!(data, 7988232);
    }
}
