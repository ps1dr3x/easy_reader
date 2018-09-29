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
//!     let file = File::open("resources/test-file-lf").unwrap();
//!     let mut easy_reader = EasyReader::new(file).unwrap();
//!
//!     println!("First line: {}", easy_reader.next_line().unwrap());
//!     println!("Second line: {}", easy_reader.next_line().unwrap());
//!     println!("First line: {}", easy_reader.prev_line().unwrap());
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
//!     let file = File::open("resources/test-file-lf").unwrap();
//!     let mut easy_reader = EasyReader::new(file).unwrap();
//!
//!     loop {
//!         println!("{}", easy_reader.random_line().unwrap());
//!     }
//! }
//! ```

extern crate rand;

use std::{
    io::{
        prelude::*,
        Error,
        SeekFrom,
        ErrorKind
    },
    fs::File
};
use rand::Rng;

const CHUNK_SIZE: usize = 10;
const CR_BYTE: u8 = '\r' as u8;
const LF_BYTE: u8 = '\n' as u8;

#[derive(Clone, PartialEq)]
enum ReadMode {
    Prev,
    Current,
    Next,
    Random
}

pub struct EasyReader {
    file: File,
    file_size: usize,
    current_start_line_offset: usize,
    current_end_line_offset: usize
}

impl EasyReader {
    pub fn new(file: File) -> Result<EasyReader, Error> {
        let file_size = file.metadata()?.len() as usize;
        if file_size == 0 { return Err(Error::new(ErrorKind::UnexpectedEof, "Empty file")) }

        Ok(EasyReader {
            file: file,
            file_size: file_size,
            current_start_line_offset: 0,
            current_end_line_offset: 0
        })
    }

    pub fn from_bof(&mut self) -> &mut EasyReader {
        self.current_start_line_offset = 0;
        self.current_end_line_offset = 0;
        self
    }

    pub fn from_eof(&mut self) -> &mut EasyReader {
        self.current_start_line_offset = self.file_size;
        self.current_end_line_offset = self.file_size;
        self
    }

    pub fn prev_line(&mut self) -> Result<String, Error> {
        self.read_line(ReadMode::Prev)
    }

    pub fn current_line(&mut self) -> Result<String, Error> {
        self.read_line(ReadMode::Current)
    }

    pub fn next_line(&mut self) -> Result<String, Error> {
        self.read_line(ReadMode::Next)
    }

    pub fn random_line(&mut self) -> Result<String, Error> {
        self.read_line(ReadMode::Random)
    }

    fn read_line(&mut self, mode: ReadMode) -> Result<String, Error> {
        match mode {
            ReadMode::Prev => {
                if self.current_start_line_offset == 0 {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "BOF reached"))
                }
                self.current_end_line_offset = self.current_start_line_offset;
            },
            ReadMode::Current => {
                if self.current_start_line_offset == self.current_end_line_offset {
                    if self.current_start_line_offset == self.file_size {
                        self.current_start_line_offset = self.find_start_line(ReadMode::Prev)?;
                    }
                    if self.current_end_line_offset == 0 {
                        self.current_end_line_offset = self.find_end_line()?;
                    }
                }
            },
            ReadMode::Next => {
                if self.current_end_line_offset == self.file_size {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "EOF reached"))
                }
                self.current_start_line_offset = self.current_end_line_offset;
            },
            ReadMode::Random => {
                self.current_start_line_offset = rand::thread_rng().gen_range(0, self.file_size);
            }
        }

        if mode != ReadMode::Current {
            self.current_start_line_offset = self.find_start_line(mode.clone())?;
            self.current_end_line_offset = self.find_end_line()?;
        }

        let offset = self.current_start_line_offset;
        let line_length = self.current_end_line_offset - self.current_start_line_offset;
        let buffer = self.read_bytes(offset, line_length)?;

        let line = String::from_utf8(buffer)
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!(
                        "The line starting at byte: {} and ending at byte: {} is not valid UTF-8. Conversion error: {}",
                        self.current_start_line_offset,
                        self.current_end_line_offset,
                        err
                    )
                )
            })?;

        Ok(line)
    }

    fn find_start_line(&mut self, mode: ReadMode) -> Result<usize, Error> {
        let mut new_start_line_offset = self.current_start_line_offset;

        let mut n_chunks = 0;
        loop {
            if new_start_line_offset == 0 { break; }

            let mut found = false;
            match mode {
                ReadMode::Prev | ReadMode::Random => {
                    let mut margin = 0;
                    let from = {
                        if new_start_line_offset < CHUNK_SIZE {
                            margin = CHUNK_SIZE - new_start_line_offset;
                            0
                        } else {
                            new_start_line_offset - CHUNK_SIZE
                        }
                    };

                    let mut chunk = self.read_chunk(from)?;
                    chunk.reverse();

                    for i in 0..CHUNK_SIZE {
                        if i < margin { continue; }
                        if new_start_line_offset == 0 {
                            found = true;
                            break;
                        } else {
                            if n_chunks == 0 &&
                              self.current_start_line_offset == new_start_line_offset &&
                              mode != ReadMode::Random {
                                // We've not moved yet
                                new_start_line_offset -= 1;
                                continue;
                            }

                            if chunk[i] == LF_BYTE {
                                found = true;
                            }
                        }

                        if found { break; }
                        new_start_line_offset -= 1;
                    }
                },
                ReadMode::Current => (),
                ReadMode::Next => {
                    let mut chunk = self.read_chunk(new_start_line_offset)?;

                    for i in 0..CHUNK_SIZE {
                        if new_start_line_offset >= self.file_size - 1 {
                            return Err(Error::new(ErrorKind::UnexpectedEof, "EOF reached"))
                        }

                        if chunk[i] == LF_BYTE {
                            found = true;
                        }

                        new_start_line_offset += 1;
                        if found { break; }
                    }
                }
            }

            if found { break; }
            n_chunks += 1;
        }

        Ok(new_start_line_offset)
    }

    fn find_end_line(&mut self) -> Result<usize, Error> {
        let mut new_end_line_offset = self.current_start_line_offset;

        loop {
            if new_end_line_offset == self.file_size { break }

            let chunk = self.read_chunk(new_end_line_offset)?;

            let mut found = false;
            for i in 0..CHUNK_SIZE {
                if new_end_line_offset == self.file_size {
                    found = true;
                    break;
                } else if chunk[i] == LF_BYTE {
                    // Handle CRLF files
                    if i > 0 {
                        if chunk[i - 1] == CR_BYTE {
                            new_end_line_offset -= 1;
                        }
                    } else {
                        if new_end_line_offset < self.file_size {
                            let next_byte = self.read_bytes(new_end_line_offset - 1, 1)?[0];
                            if next_byte == CR_BYTE {
                                new_end_line_offset -= 1;
                            }
                        }
                    }
                    found = true;
                    break;
                } else {
                    new_end_line_offset += 1;
                }
            }
            if found { break; }
        }

        Ok(new_end_line_offset)
    }

    fn read_chunk(&mut self, offset: usize) -> Result<[u8; CHUNK_SIZE], Error> {
        let mut buffer: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
        self.file.seek(SeekFrom::Start(offset as u64))?;
        self.file.read(&mut buffer)?;
        Ok(buffer)
    }

    fn read_bytes(&mut self, offset: usize, bytes: usize) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0; bytes];
        self.file.seek(SeekFrom::Start(offset as u64))?;
        self.file.read(&mut buffer[..])?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{ Instant, Duration };

    fn duration_as_millis(d: Duration) -> f64 {
        d.as_secs() as f64 * 1000. + d.subsec_nanos() as f64 / 1e6
    }

    #[test]
    fn test_empty_file() {
        let file = File::open("resources/empty-file").unwrap();
        let easy_reader: Result<EasyReader, Error> = EasyReader::new(file);

        assert!(easy_reader.is_err(), "Empty file, but the constructor hasn't returned an Error");
    }

    #[test]
    fn test_one_line_file() {
        let file = File::open("resources/one-line-file").unwrap();
        let mut easy_reader = EasyReader::new(file).unwrap();

        assert!(easy_reader.next_line().unwrap().eq("A"), "The single line of one-line-file should be: A");
        assert!(easy_reader.next_line().is_err(), "There is no other lines in one-line-file, this should be: Err(Error::new(ErrorKind::UnexpectedEof, \"EOF reached\"))");
        assert!(easy_reader.prev_line().is_err(), "There is no other lines in one-line-file, this should be: Err(Error::new(ErrorKind::UnexpectedEof, \"BOF reached\"))");
        assert!(easy_reader.current_line().unwrap().eq("A"), "The single line of one-line-file should be: A");
        
        easy_reader.from_bof();
        assert!(easy_reader.next_line().unwrap().eq("A"), "The single line of one-line-file from the bof should be: A");

        easy_reader.from_eof();
        assert!(easy_reader.prev_line().unwrap().eq("A"), "The single line of one-line-file from the eof should be: A");

        for _i in 1..10 {
            assert!(easy_reader.random_line().unwrap().eq("A"), "The single line of one-line-file should be: A (test: 10 random lines)");
        }
    }

    #[test]
    fn test_move_through_lines() {
        let file = File::open("resources/test-file-lf").unwrap();
        let mut easy_reader = EasyReader::new(file).unwrap();

        easy_reader.from_eof();
        assert!(easy_reader.prev_line().unwrap().eq("EEEE  EEEEE  EEEE  EEEEE"), "[test-file-lf] The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE");
        assert!(easy_reader.prev_line().unwrap().eq("DDDD  DDDDD DD DDD DDD DD"), "[test-file-lf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD");
        assert!(easy_reader.prev_line().unwrap().eq("CCCC  CCCCC"), "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC");
        assert!(easy_reader.current_line().unwrap().eq("CCCC  CCCCC"), "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC");
        assert!(easy_reader.next_line().unwrap().eq("DDDD  DDDDD DD DDD DDD DD"), "[test-file-lf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD");

        easy_reader.from_bof();
        assert!(easy_reader.next_line().unwrap().eq("AAAA AAAA"), "[test-file-lf] The first line from the BOF should be: AAAA AAAA");
        assert!(easy_reader.next_line().unwrap().eq("B B BB BBB"), "[test-file-lf] The second line from the BOF should be: B B BB BBB");
        assert!(easy_reader.next_line().unwrap().eq("CCCC  CCCCC"), "[test-file-lf] The third line from the BOF should be: CCCC  CCCCC");
        assert!(easy_reader.current_line().unwrap().eq("CCCC  CCCCC"), "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC");
        assert!(easy_reader.prev_line().unwrap().eq("B B BB BBB"), "[test-file-lf] The second line from the BOF should be: B B BB BBB");

        let file = File::open("resources/test-file-crlf").unwrap();
        let mut easy_reader = EasyReader::new(file).unwrap();

        easy_reader.from_eof();
        assert!(easy_reader.prev_line().unwrap().eq("EEEE  EEEEE  EEEE  EEEEE"), "[test-file-crlf] The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE");
        assert!(easy_reader.prev_line().unwrap().eq("DDDD  DDDDD DD DDD DDD DD"), "[test-file-crlf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD");
        assert!(easy_reader.prev_line().unwrap().eq("CCCC  CCCCC"), "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC");
        assert!(easy_reader.current_line().unwrap().eq("CCCC  CCCCC"), "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC");
        assert!(easy_reader.next_line().unwrap().eq("DDDD  DDDDD DD DDD DDD DD"), "[test-file-crlf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD");

        easy_reader.from_bof();
        assert!(easy_reader.next_line().unwrap().eq("AAAA AAAA"), "[test-file-crlf] The first line from the BOF should be: AAAA AAAA");
        assert!(easy_reader.next_line().unwrap().eq("B B BB BBB"), "[test-file-crlf] The second line from the BOF should be: B B BB BBB");
        assert!(easy_reader.next_line().unwrap().eq("CCCC  CCCCC"), "[test-file-crlf] The third line from the BOF should be: CCCC  CCCCC");
        assert!(easy_reader.current_line().unwrap().eq("CCCC  CCCCC"), "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC");
        assert!(easy_reader.prev_line().unwrap().eq("B B BB BBB"), "[test-file-crlf] The second line from the BOF should be: B B BB BBB");
    }

    #[test]
    fn test_random_line() {
        let file = File::open("resources/test-file-lf").unwrap();
        let mut easy_reader = EasyReader::new(file).unwrap();

        for _i in 0..100 {
            let random_line = easy_reader.random_line().unwrap();
            assert!(!random_line.is_empty(), "Empty line, but test-file-lf does not contain empty lines");
        }

        let file = File::open("resources/test-file-crlf").unwrap();
        let mut easy_reader = EasyReader::new(file).unwrap();

        for _i in 0..100 {
            let random_line = easy_reader.random_line().unwrap();
            assert!(!random_line.is_empty(), "Empty line, but test-file-crlf does not contain empty lines");
        }
    }

    #[test]
    fn test_iterations() {
        let file = File::open("resources/test-file-lf").unwrap();
        let mut easy_reader = EasyReader::new(file).unwrap();

        while let Ok(line) = easy_reader.next_line() {
            assert!(!line.is_empty(), "Empty line, but test-file-lf does not contain empty lines");
        }
        assert!(easy_reader.current_end_line_offset == easy_reader.file_size, "After the \"while next-line\" iteration the offset should be at the EOF");
        assert!(easy_reader.current_line().unwrap().eq("EEEE  EEEEE  EEEE  EEEEE"), "The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE");
        assert!(easy_reader.prev_line().unwrap().eq("DDDD  DDDDD DD DDD DDD DD"), "The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD");

        easy_reader.from_eof();
        while let Ok(line) = easy_reader.prev_line() {
            assert!(!line.is_empty(), "Empty line, but test-file-lf does not contain empty lines");
        }
        assert!(easy_reader.current_start_line_offset == 0, "After the \"while prev-line\" iteration the offset should be at the BOF");
        assert!(easy_reader.current_line().unwrap().eq("AAAA AAAA"), "The first line from the BOF should be: AAAA AAAA");
        assert!(easy_reader.next_line().unwrap().eq("B B BB BBB"), "The second line from the BOF should be: B B BB BBB");
    }

    #[test]
    fn read_forward_1000_times() {
        let now = Instant::now();

        for _i in 0..1000 {
            let file = File::open("resources/test-file-lf").unwrap();
            let mut easy_reader = EasyReader::new(file).unwrap();
            while let Ok(_line) = easy_reader.next_line() {}
        }

        let elapsed = duration_as_millis(now.elapsed());
        println!("\nread_forward_10000_times: {}ms", elapsed);
    }

    #[test]
    fn read_backward_1000_times() {
        let now = Instant::now();

        for _i in 0..1000 {
            let file = File::open("resources/test-file-lf").unwrap();
            let mut easy_reader = EasyReader::new(file).unwrap();
            easy_reader.from_eof();
            while let Ok(_line) = easy_reader.prev_line() {}
        }

        let elapsed = duration_as_millis(now.elapsed());
        println!("\nread_backward_10000_times: {}ms", elapsed);
    }

    #[test]
    fn read_10000_random_lines() {
        let now = Instant::now();

        let file = File::open("resources/test-file-lf").unwrap();
        let mut easy_reader = EasyReader::new(file).unwrap();

        for _i in 0..10000 {
            easy_reader.random_line().unwrap();
        }

        let elapsed = duration_as_millis(now.elapsed());
        println!("\nread_10000_random_lines: {}ms", elapsed);
    }
}
