// Copyright 2018 Michele Federici (@ps1dr3x) <michele@federici.tech>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # EasyReader
//!
//! Move forward, backward or randomly through the lines of huge files. Easily and fastly.
//! 
//! #### Why?
//! 
//! Mainly because with Rust (currently) there isn't an easy way to read huge files line by line in reverse and/or randomly, and to freely move forwards and backwards through the lines without consuming an iterator.
//!
//! ### Example: basic usage
//!
//! ```rust
//! extern crate easy_reader;
//!
//! use easy_reader::EasyReader;
//! use std::fs::File;
//!
//! fn main() {
//!     let file: File = File::open("resources/test-file-lf").unwrap();
//!     let mut easy_reader: EasyReader = EasyReader::new(file).unwrap();
//!
//!     println!("First line: {}", easy_reader.next_line().unwrap());
//!     println!("Also first line: {}", easy_reader.prev_line().unwrap());
//!     println!("Second line: {}", easy_reader.next_line().unwrap());
//!     println!("Random line: {}", easy_reader.random_line().unwrap());
//!
//!     // Iteration through the entire file (reverse)
//!     easy_reader.from_eof();
//!     while let Ok(line) = easy_reader.prev_line() {
//!         println!("{}", line);
//!     }
//! 
//!     // You can always start/restart reading from the end of file (EOF)
//!     easy_reader.from_eof();
//!     println!("Last line: {}", easy_reader.prev_line().unwrap());
//!     // Or the begin of file (BOF)
//!     easy_reader.from_bof();
//!     println!("First line: {}", easy_reader.next_line().unwrap());
//! }
//! ```
//! 
//! ### Example: read random lines endlessly
//! 
//! ```no_run
//! extern crate easy_reader;
//!
//! use easy_reader::EasyReader;
//! use std::fs::File;
//!
//! fn main() {
//!     let file: File = File::open("resources/test-file-lf").unwrap();
//!     let mut easy_reader: EasyReader = EasyReader::new(file).unwrap();
//!
//!     loop {
//!         println!("{}", easy_reader.random_line().unwrap());
//!     }
//! }
//! ```

extern crate rand;

use std::io::{
    prelude::*,
    Error,
    SeekFrom,
    ErrorKind
};
use std::fs::File;
use rand::Rng;

const CR_BYTE: u8 = '\r' as u8;
const LF_BYTE: u8 = '\n' as u8;

#[derive(PartialEq)]
enum ReadMode {
    Prev,
    Next,
    Random
}

pub struct EasyReader {
    file: File,
    current_offset: usize
}

impl EasyReader {
    pub fn new(file: File) -> Result<EasyReader, Error> {
        let file_size: u64 = file.metadata().unwrap().len();
        if file_size == 0 { return Err(Error::new(ErrorKind::UnexpectedEof, "Empty file")) }

        Ok(EasyReader {
            file: file,
            current_offset: 0
        })
    }

    pub fn from_bof(&mut self) -> &mut EasyReader {
        self.current_offset = 0;
        self
    }

    pub fn from_eof(&mut self) -> &mut EasyReader {
        self.current_offset = self.file.metadata().unwrap().len() as usize;
        self
    }

    pub fn next_line(&mut self) -> Result<String, Error> {
        self.read_line(ReadMode::Next)
    }

    pub fn prev_line(&mut self) -> Result<String, Error> {
        self.read_line(ReadMode::Prev)
    }

    pub fn random_line(&mut self) -> Result<String, Error> {
        self.read_line(ReadMode::Random)
    }

    fn read_line(&mut self, mode: ReadMode) -> Result<String, Error> {
        match mode {
            ReadMode::Prev => {
                if self.current_offset == 0 {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "BOF reached"))
                }
            },
            ReadMode::Next => {
                if self.current_offset == self.file.metadata()?.len() as usize {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "EOF reached"))
                }
            },
            ReadMode::Random => {
                let file_size = self.file.metadata().unwrap().len() as usize;
                self.current_offset = rand::thread_rng().gen_range(0, file_size);
            }
        }

        let start_line_offset = self.find_start_line()?;
        self.current_offset = start_line_offset;
        let end_line_offset = self.find_end_line()?;

        if mode != ReadMode::Prev {
            self.current_offset = end_line_offset;
        }

        let mut buffer = vec![0; end_line_offset - start_line_offset];
        self.file.seek(SeekFrom::Start(start_line_offset as u64))?;
        self.file.read(&mut buffer)?;

        let line = String::from_utf8(buffer)
            .map_err(|err| {
                println!("Error {}", err);
                Error::new(std::io::ErrorKind::Other, "TODO!")
            })?;

        Ok(line)
    }

    fn find_start_line(&mut self) -> Result<usize, Error> {
        let mut start_line_offset: usize = self.current_offset;

        loop {
            if start_line_offset == 0 { break }

            let byte: u8 = self.read_byte(start_line_offset)?;

            if (byte == CR_BYTE || byte == LF_BYTE) && (start_line_offset == self.current_offset) {
                // Reading forward
                start_line_offset += 1;
            } else if byte != LF_BYTE && byte != CR_BYTE && start_line_offset == (self.current_offset + 1) {
                // Forward reading break condition
                break;
            } else if byte == LF_BYTE && start_line_offset == (self.current_offset - 1) ||
                byte != LF_BYTE {
                // Reading backward
                start_line_offset -= 1;
            } else {
                // Backward reading break condition
                start_line_offset += 1;
                break;
            }
        }

        Ok(start_line_offset)
    }

    fn find_end_line(&mut self) -> Result<usize, Error> {
        let file_size: usize = self.file.metadata()?.len() as usize;
        let mut end_line_offset: usize = self.current_offset;

        loop {
            if end_line_offset == file_size { break }

            let byte: u8 = self.read_byte(end_line_offset)?;

            if (byte == CR_BYTE && (end_line_offset == self.current_offset)) ||
                (byte == LF_BYTE && (end_line_offset == (self.current_offset + 1))) ||
                (byte != LF_BYTE && byte != CR_BYTE) {
                end_line_offset += 1;
            } else {
                break;
            }
        }

        Ok(end_line_offset)
    }

    fn read_byte(&mut self, offset: usize) -> Result<u8, Error> {
        let mut buffer: [u8; 1] = [0];
        self.file.seek(SeekFrom::Start(offset as u64))?;
        self.file.read(&mut buffer)?;
        Ok(buffer[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let file: File = File::open("resources/empty-file").unwrap();
        let easy_reader: Result<EasyReader, Error> = EasyReader::new(file);

        assert!(easy_reader.is_err(), "Empty file, but the constructor hasn't returned an Error");
    }

    #[test]
    fn test_one_line_file() {
        let file: File = File::open("resources/one-line-file").unwrap();
        let mut easy_reader: EasyReader = EasyReader::new(file).unwrap();

        assert!(easy_reader.next_line().unwrap().eq("A"), "The sole line of one.unwrap()-line-file should be: A");
        assert!(easy_reader.next_line().is_err(), "There is no other lines in one-line-file, this should be: Err(Error::new(ErrorKind::UnexpectedEof, \"EOF reached\"))");
        assert!(easy_reader.prev_line().unwrap().eq("A"), "The sole line of one.unwrap()-line-file should be: A");
        assert!(easy_reader.prev_line().is_err(), "There is no other lines in one-line-file, this should be: Err(Error::new(ErrorKind::UnexpectedEof, \"BOF reached\"))");
    }

    #[test]
    fn test_move_through_lines() {
        let file: File = File::open("resources/test-file-lf").unwrap();
        let mut easy_reader: EasyReader = EasyReader::new(file).unwrap();

        easy_reader.from_eof();
        assert!(easy_reader.prev_line().unwrap().eq("EEEE EEEE"), "[test-file-lf] The first line from the EOF should be: EEEE EEEE");
        assert!(easy_reader.prev_line().unwrap().eq("DDDD DDDD"), "[test-file-lf] The second line from the EOF should be: DDDD DDDD");
        assert!(easy_reader.prev_line().unwrap().eq("CCCC CCCC"), "[test-file-lf] The third line from the EOF should be: CCCC CCCC");

        easy_reader.from_bof();
        assert!(easy_reader.next_line().unwrap().eq("AAAA AAAA"), "[test-file-lf] The first line from the BOF should be: AAAA AAAA");
        assert!(easy_reader.next_line().unwrap().eq("BBBB BBBB"), "[test-file-lf] The second line from the BOF should be: BBBB BBBB");
        assert!(easy_reader.next_line().unwrap().eq("CCCC CCCC"), "[test-file-lf] The third line from the BOF should be: CCCC CCCC");

        let file: File = File::open("resources/test-file-crlf").unwrap();
        let mut easy_reader: EasyReader = EasyReader::new(file).unwrap();

        easy_reader.from_eof();
        assert!(easy_reader.prev_line().unwrap().eq("EEEE EEEE"), "[test-file-crlf] The first line from the EOF should be: EEEE EEEE");
        assert!(easy_reader.prev_line().unwrap().eq("DDDD DDDD"), "[test-file-crlf] The second line from the EOF should be: DDDD DDDD");
        assert!(easy_reader.prev_line().unwrap().eq("CCCC CCCC"), "[test-file-crlf] The third line from the EOF should be: CCCC CCCC");

        easy_reader.from_bof();
        assert!(easy_reader.next_line().unwrap().eq("AAAA AAAA"), "[test-file-crlf] The first line from the BOF should be: AAAA AAAA");
        assert!(easy_reader.next_line().unwrap().eq("BBBB BBBB"), "[test-file-crlf] The second line from the BOF should be: BBBB BBBB");
        assert!(easy_reader.next_line().unwrap().eq("CCCC CCCC"), "[test-file-crlf] The third line from the BOF should be: CCCC CCCC");
    }

    #[test]
    fn test_random_line() {
        let file: File = File::open("resources/test-file-lf").unwrap();
        let mut easy_reader: EasyReader = EasyReader::new(file).unwrap();

        let random_line: String = easy_reader.random_line().unwrap();
        assert!(!random_line.is_empty(), "Empty line, but test-file-lf does not contain empty lines");

        let file: File = File::open("resources/test-file-crlf").unwrap();
        let mut easy_reader: EasyReader = EasyReader::new(file).unwrap();

        let random_line: String = easy_reader.random_line().unwrap();
        assert!(!random_line.is_empty(), "Empty line, but test-file-crlf does not contain empty lines");
    }

    #[test]
    fn test_iterations() {
        let file: File = File::open("resources/test-file-lf").unwrap();
        let mut easy_reader: EasyReader = EasyReader::new(file).unwrap();

        while let Ok(line) = easy_reader.next_line() {
            assert!(!line.is_empty(), "Empty line, but test-file-lf does not contain empty lines");
        }
        assert!(easy_reader.current_offset == easy_reader.file.metadata().unwrap().len() as usize, "After the \"while next-line\" iteration the offset should be at the EOF");
        assert!(easy_reader.prev_line().unwrap().eq("EEEE EEEE"), "The first line from the EOF should be: EEEE EEEE");

        easy_reader.from_eof();
        while let Ok(line) = easy_reader.prev_line() {
            assert!(!line.is_empty(), "Empty line, but test-file-lf does not contain empty lines");
        }
        assert!(easy_reader.current_offset == 0, "After the \"while prev-line\" iteration the offset should be at the BOF");
        assert!(easy_reader.next_line().unwrap().eq("AAAA AAAA"), "The first line from the BOF should be: AAAA AAAA");
    }
}
