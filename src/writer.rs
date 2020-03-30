use crate::cli::{CliResult, Config};
use crate::ext::BStringExt;
use crate::matcher::{MatchResult, MatcherResult};
use std::io::Write;

pub struct Writer<W> {
    pub wrt: W,
}

impl<W: Write> Writer<W> {
    pub fn print_matches(mut self, matcher_result: MatcherResult, config: &Config) -> CliResult {
        match matcher_result {
            Ok(match_result) => self
                .print_lines_iter(match_result, config)
                .expect("Error occured while printing matches"),
            Err(err) => eprintln!("Error occured while searching for matches: {}", err),
        };
        Ok(())
    }

    fn print_lines_iter(&mut self, match_result: MatchResult, config: &Config) -> CliResult {
        let no_line_number = config.no_line_number;
        let matches = match_result.matches;
        let line_numbers = match_result.line_numbers;
        if !no_line_number {
            let line_numbers_inner = line_numbers.unwrap();
            for (line_number, single_match) in line_numbers_inner.iter().zip(matches) {
                writeln!(
                    self.wrt,
                    "{}:{}",
                    line_number,
                    BStringExt::to_utf8(&single_match)
                )?;
            }
        } else {
            for line in matches.iter() {
                writeln!(self.wrt, "{}", BStringExt::to_utf8(line))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::matcher::*;
    use std::fs::File;
    use std::io::Cursor;
    use std::io::{Read, Seek, SeekFrom, Write};

    const DICKENS: &str = "\
He started      \r
make a run
& stopped.
He started
made a quick run
and stopped
He started
made a RuN
and then stopped\
";

    #[test]
    fn print_dickens() {
        let expected = "\
2:make a run
5:made a quick run
";
        // Build config and matcher
        let config = Config {
            no_line_number: false,
        };
        let mut matcher = Matcher {
            reader: &mut Cursor::new(DICKENS.as_bytes()),
            pattern: &"run".to_owned(),
            config: &config,
        };
        let matches = matcher.get_matches();

        // Write to temp file
        let mut tmpfile: File = tempfile::tempfile().unwrap();
        let wrt = Writer {
            wrt: Write::by_ref(&mut tmpfile),
        };
        wrt.print_matches(matches, &config).unwrap();

        // Seek to start (!)
        tmpfile.seek(SeekFrom::Start(0)).unwrap();

        // Read back
        let mut got = String::new();
        tmpfile.read_to_string(&mut got).unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    fn print_dickens_no_line_number() {
        let expected = "\
make a run
made a quick run
";
        // Build config and matcher
        let config = Config {
            no_line_number: true,
        };
        let mut matcher = Matcher {
            reader: &mut Cursor::new(DICKENS.as_bytes()),
            pattern: &"run".to_owned(),
            config: &config,
        };
        let matches = matcher.get_matches();

        // Write to temp file
        let mut tmpfile: File = tempfile::tempfile().unwrap();
        let wrt = Writer {
            wrt: Write::by_ref(&mut tmpfile),
        };
        wrt.print_matches(matches, &config).unwrap();

        // Seek to start (!)
        tmpfile.seek(SeekFrom::Start(0)).unwrap();

        // Read back
        let mut got = String::new();
        tmpfile.read_to_string(&mut got).unwrap();

        assert_eq!(expected, got);
    }
}
