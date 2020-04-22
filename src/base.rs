use crate::results::*;
use crate::searcher::Searcher;
use bstr::{io::BufReadExt, ByteSlice};
use std::io::BufRead;

trait Base {
    fn no_line_number(&mut self) -> GenResult;
    fn no_line_number_caseless(&mut self) -> GenResult;
    fn line_number(&mut self) -> GenResult;
    fn line_number_caseless(&mut self) -> GenResult;
}

pub trait BaseSearch {
    fn get_matches(&mut self) -> GenResult;
}

// Closures try to borrow `self` as a whole so assign disjoint fields to
// variables first
impl<'a, R: BufRead> Base for Searcher<'a, R> {
    fn no_line_number(&mut self) -> GenResult {
        let (reader, pattern) = (&mut self.reader, &self.matcher.pattern);

        let mut sir = SearchInnerResult::default();

        reader.for_byte_line_with_terminator(|line| {
            sir.check_and_store_nln(pattern, line, check_contains);
            Ok(true)
        })?;

        sir.upcast()
    }

    fn no_line_number_caseless(&mut self) -> GenResult {
        let (reader, pattern) = (&mut self.reader, &self.matcher.pattern);

        let mut sir = SearchInnerResult::default();

        reader.for_byte_line_with_terminator(|line| {
            if line.is_ascii() {
                sir.check_and_store_separate_nln(
                    pattern,
                    &line.to_ascii_lowercase(),
                    line,
                    check_contains,
                );
            } else {
                let mut buf = Default::default();
                line.to_lowercase_into(&mut buf);
                sir.check_and_store_separate_nln(pattern, &buf, line, check_contains);
            }
            Ok(true)
        })?;

        sir.upcast()
    }

    fn line_number(&mut self) -> GenResult {
        let (reader, pattern) = (&mut self.reader, &self.matcher.pattern);

        let mut line_number = 0;
        let mut sir = SearchInnerResult::default();

        reader.for_byte_line_with_terminator(|line| {
            line_number += 1;
            sir.check_and_store(pattern, line_number, line, check_contains);
            Ok(true)
        })?;

        sir.upcast()
    }

    fn line_number_caseless(&mut self) -> GenResult {
        let (reader, pattern) = (&mut self.reader, &self.matcher.pattern);

        let mut line_number = 0;
        let mut sir = SearchInnerResult::default();

        reader.for_byte_line_with_terminator(|line| {
            line_number += 1;
            if line.is_ascii() {
                sir.check_and_store_separate(
                    pattern,
                    line_number,
                    &line.to_ascii_lowercase(),
                    line,
                    check_contains,
                );
            } else {
                let mut buf = Default::default();
                line.to_lowercase_into(&mut buf);
                sir.check_and_store_separate(pattern, line_number, &buf, line, check_contains);
            }
            Ok(true)
        })?;

        sir.upcast()
    }
}

impl<'a, R: BufRead> BaseSearch for Searcher<'a, R> {
    fn get_matches(&mut self) -> GenResult {
        let (ignore_case, no_line_number) = (
            self.matcher.config.ignore_case,
            self.matcher.config.no_line_number,
        );

        match (no_line_number, ignore_case) {
            (true, true) => self.no_line_number_caseless(),
            (true, false) => self.no_line_number(),
            (false, true) => self.line_number_caseless(),
            (false, false) => self.line_number(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::matcher::MatcherBuilder;
    use std::io::Cursor;

    const LINE: &str = "He started\nmade a run\n& stopped";
    const LINE_BIN: &str = "He started\nmad\x00e a run\n& stopped";
    const LINE_BIN2: &str = "He started\r\nmade a r\x00un\n& stopped";
    const LINE_BIN3: &str = "He started\r\nmade a r\x00un\r\n& stopped";
    const LINE_MAX_NON_ASCII: &str = "He started again\na\x00nd again\n& AΓain";

    #[test]
    fn find_no_match() {
        let mut line = Cursor::new(LINE.as_bytes());
        let pattern = "Made".to_owned();

        let matcher = MatcherBuilder::new()
            .no_line_number(true)
            .max_count(None)
            .build(pattern);

        let searcher = Searcher {
            reader: &mut line,
            matcher: &matcher,
        };

        let gen_result = searcher.search_matches();
        let gen_inner_result = gen_result.as_ref().unwrap();

        if let GenInnerResult::Search(search_result) = gen_inner_result {
            let matches = &search_result.matches;
            let line_numbers = &search_result.line_numbers;

            assert!(matches.is_empty());
            assert_eq!(line_numbers, &LineNumbers::None);
        };
    }

    #[test]
    fn find_a_match() {
        let mut line = Cursor::new(LINE.as_bytes());
        let pattern = "made".to_owned();

        let matcher = MatcherBuilder::new()
            .no_line_number(false)
            .max_count(None)
            .build(pattern);

        let searcher = Searcher {
            reader: &mut line,
            matcher: &matcher,
        };

        let gen_result = searcher.search_matches();
        let gen_inner_result = gen_result.as_ref().unwrap();

        if let GenInnerResult::Search(search_result) = gen_inner_result {
            let matches = &search_result.matches;
            let line_numbers = &search_result.line_numbers;

            assert!(matches.len() == 1);
            assert_eq!(matches[0], &b"made a run"[..]);
            let line_number_inner: Vec<u64> = vec![2];
            assert_eq!(line_numbers, &LineNumbers::Some(line_number_inner));
        };
    }

    #[test]
    fn search_binary_text() {
        let mut line = Cursor::new(LINE_BIN.as_bytes());
        let pattern = "made".to_owned();

        let matcher = MatcherBuilder::new()
            .no_line_number(false)
            .max_count(None)
            .build(pattern);

        let searcher = Searcher {
            reader: &mut line,
            matcher: &matcher,
        };

        let gen_result = searcher.search_matches();
        let gen_inner_result = gen_result.as_ref().unwrap();

        if let GenInnerResult::Search(search_result) = gen_inner_result {
            let matches = &search_result.matches;

            assert_eq!(matches.len(), 0);
        };
    }

    #[test]
    fn search_binary_text2() {
        let mut line = Cursor::new(LINE_BIN2.as_bytes());
        let pattern = "made".to_owned();

        let matcher = MatcherBuilder::new()
            .no_line_number(false)
            .max_count(None)
            .build(pattern);

        let searcher = Searcher {
            reader: &mut line,
            matcher: &matcher,
        };

        let gen_result = searcher.search_matches();
        let gen_inner_result = gen_result.as_ref().unwrap();

        if let GenInnerResult::Search(search_result) = gen_inner_result {
            let matches = &search_result.matches;

            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0], &b"made a r\x00un"[..]);
        };
    }

    #[test]
    fn search_binary_text3() {
        let mut line = Cursor::new(LINE_BIN3.as_bytes());
        let pattern = "r\x00un".to_owned();

        let matcher = MatcherBuilder::new()
            .no_line_number(false)
            .max_count(None)
            .build(pattern);

        let searcher = Searcher {
            reader: &mut line,
            matcher: &matcher,
        };

        let gen_result = searcher.search_matches();
        let gen_inner_result = gen_result.as_ref().unwrap();

        if let GenInnerResult::Search(search_result) = gen_inner_result {
            let matches = &search_result.matches;

            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0], &b"made a r\x00un"[..]);
        };
    }

    #[test]
    fn line_number_caseless() {
        let mut line = Cursor::new(LINE_MAX_NON_ASCII.as_bytes());
        let pattern = "again".to_owned();

        let matcher = MatcherBuilder::new()
            .ignore_case(true)
            .max_count(None)
            .no_line_number(false)
            .build(pattern);

        let searcher = Searcher {
            reader: &mut line,
            matcher: &matcher,
        };

        let gen_result = searcher.search_matches();
        let gen_inner_result = gen_result.as_ref().unwrap();

        if let GenInnerResult::Search(search_result) = gen_inner_result {
            let matches = &search_result.matches;
            let line_numbers = &search_result.line_numbers;

            assert!(matches.len() == 2);
            let line_number_inner: Vec<u64> = vec![1, 2];
            assert_eq!(line_numbers, &LineNumbers::Some(line_number_inner));
        };
    }

    #[test]
    fn no_line_number_caseless() {
        let mut line = Cursor::new(LINE_MAX_NON_ASCII.as_bytes());
        let pattern = "aγain".to_owned();

        let matcher = MatcherBuilder::new()
            .ignore_case(true)
            .max_count(None)
            .no_line_number(true)
            .build(pattern);

        let searcher = Searcher {
            reader: &mut line,
            matcher: &matcher,
        };

        let gen_result = searcher.search_matches();
        let gen_inner_result = gen_result.as_ref().unwrap();

        if let GenInnerResult::Search(search_result) = gen_inner_result {
            let matches = &search_result.matches;
            let line_numbers = &search_result.line_numbers;

            assert!(matches.len() == 1);
            assert_eq!(line_numbers, &LineNumbers::None);
        };
    }
}
