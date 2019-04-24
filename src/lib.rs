// Copyright 2018 Michele Federici (@ps1dr3x) <michele@federici.tech>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # EasyReader
//! 
//! The main goal of this library is to allow long navigations through the lines of large files, freely moving forwards and backwards or getting random lines without having to consume an iterator.
//! 
//! Currently with Rust's standard library is possible to read a file line by line only through Lines (https://doc.rust-lang.org/std/io/trait.BufRead.html#method.lines), with which is impossible (or very expensive) to read backwards and to get random lines. Also, being an iterator, every line that has already been read is consumed and to get back to the same line you need to reinstantiate the reader and consume all the lines until the desired one (eg. in the case of the last line, all).
//! 
//! **Notes:**
//! 
//! EasyReader by default does not generate an index, it just searches for line terminators from time to time, this allows it to be used with very large files without "startup" times and excessive RAM consumption.
//! However, the lack of an index makes the reading slower and does not allow to take random lines with a perfect distribution, for these reasons there's a method to generate it; the start time will be slower, but all the following readings will use it and will therefore be faster (excluding the index build time, reading times are a bit longer but still comparable to those of a sequential forward reading through Lines) and in the random reading case the lines will be taken with a perfect distribution.
//! By the way, it's not advisable to generate the index for very large files, as an excessive RAM consumption could occur.
//!
//! ### Example: basic usage
//!
//! ```rust
//! use easy_reader::EasyReader;
//! use std::{
//!     fs::File,
//!     io::{
//!         self,
//!         Error
//!     }
//! };
//!
//! fn easy() -> Result<(), Error> {
//!     let file = File::open("resources/test-file-lf")?;
//!     let mut reader = EasyReader::new(file)?;
//! 
//!     // Generate index (optional)
//!     reader.build_index();
//!
//!     // Move through the lines
//!     println!("First line: {}", reader.next_line()?.unwrap());
//!     println!("Second line: {}", reader.next_line()?.unwrap());
//!     println!("First line: {}", reader.prev_line()?.unwrap());
//!     println!("Random line: {}", reader.random_line()?.unwrap());
//!
//!     // Iteration through the entire file (reverse)
//!     reader.eof();
//!     while let Some(line) = reader.prev_line()? {
//!         println!("{}", line);
//!     }
//! 
//!     // You can always start/restart reading from the end of file (EOF)
//!     reader.eof();
//!     println!("Last line: {}", reader.prev_line()?.unwrap());
//!     // Or the begin of file (BOF)
//!     reader.bof();
//!     println!("First line: {}", reader.next_line()?.unwrap());
//! 
//!     Ok(())
//! }
//! ```
//! 
//! ### Example: read random lines endlessly
//! 
//! ```no_run
//! use easy_reader::EasyReader;
//! use std::{
//!     fs::File,
//!     io::{
//!         self,
//!         Error
//!     }
//! };
//!
//! fn easy() -> Result<(), Error> {
//!     let file = File::open("resources/test-file-lf")?;
//!     let mut reader = EasyReader::new(file)?;
//!
//!     // Generate index (optional)
//!     reader.build_index();
//! 
//!     loop {
//!         println!("{}", reader.random_line()?.unwrap());
//!     }
//! }
//! ```

use std::io::{
    self,
    prelude::*,
    Error,
    SeekFrom,
    ErrorKind
};
use rand::Rng;
use fnv::FnvHashMap;

const CR_BYTE: u8 = b'\r';
const LF_BYTE: u8 = b'\n';

#[derive(Clone, PartialEq)]
enum ReadMode {
    Prev,
    Current,
    Next,
    Random
}

pub struct EasyReader<R> {
    file: R,
    file_size: u64,
    chunk_size: usize,
    current_start_line_offset: u64,
    current_end_line_offset: u64,
    indexed: bool,
    offsets_index: Vec<(usize, usize)>,
    newline_map: FnvHashMap<usize, usize>
}

impl<R: Read + Seek> EasyReader<R> {
    pub fn new(mut file: R) -> Result<Self, Error> {
        let file_size = file.seek(SeekFrom::End(0))?;
        if file_size == 0 { return Err(Error::new(ErrorKind::UnexpectedEof, "Empty file")) }

        Ok(EasyReader {
            file,
            file_size,
            chunk_size: 200,
            current_start_line_offset: 0,
            current_end_line_offset: 0,
            indexed: false,
            offsets_index: Vec::new(),
            newline_map: FnvHashMap::default()
        })
    }

    pub fn chunk_size(&mut self, size: usize) -> &mut Self {
        self.chunk_size = size;
        self
    }

    pub fn bof(&mut self) -> &mut Self {
        self.current_start_line_offset = 0;
        self.current_end_line_offset = 0;
        self
    }

    pub fn eof(&mut self) -> &mut Self {
        self.current_start_line_offset = self.file_size;
        self.current_end_line_offset = self.file_size;
        self
    }

    pub fn build_index(&mut self) -> io::Result<&mut Self> {
        if self.file_size > usize::max_value() as u64 {
            // 32bit ¯\_(ツ)_/¯
            return Err(Error::new(
                ErrorKind::InvalidData,
                "File too large to build an index")
            );
        }

        while let Ok(Some(_line)) = self.next_line() {
            self.offsets_index.push((self.current_start_line_offset as usize, self.current_end_line_offset as usize));
            self.newline_map.insert(self.current_start_line_offset as usize, self.offsets_index.len() - 1);
        }
        self.indexed = true;
        Ok(self)
    }

    pub fn prev_line(&mut self) -> io::Result<Option<String>> {
        self.read_line(ReadMode::Prev)
    }

    pub fn current_line(&mut self) -> io::Result<Option<String>> {
        self.read_line(ReadMode::Current)
    }

    pub fn next_line(&mut self) -> io::Result<Option<String>> {
        self.read_line(ReadMode::Next)
    }

    pub fn random_line(&mut self) -> io::Result<Option<String>> {
        self.read_line(ReadMode::Random)
    }

    fn read_line(&mut self, mode: ReadMode) -> io::Result<Option<String>> {
        match mode {
            ReadMode::Prev => {
                if self.current_start_line_offset == 0 { return Ok(None) }

                if self.indexed && self.current_start_line_offset < self.file_size {
                    let current_line = *self.newline_map.get(&(self.current_start_line_offset as usize)).unwrap();
                    self.current_start_line_offset = self.offsets_index[current_line - 1].0 as u64;
                    self.current_end_line_offset = self.offsets_index[current_line - 1].1 as u64;
                    return self.read_line(ReadMode::Current);
                } else {
                    self.current_end_line_offset = self.current_start_line_offset;
                }
            },
            ReadMode::Current => {
                if self.current_start_line_offset == self.current_end_line_offset {
                    if self.current_start_line_offset == self.file_size {
                        self.current_start_line_offset = self.find_start_line(ReadMode::Prev)? as u64;
                    }
                    if self.current_end_line_offset == 0 {
                        self.current_end_line_offset = self.find_end_line()? as u64;
                    }
                }
            },
            ReadMode::Next => {
                if self.current_end_line_offset == self.file_size { return Ok(None) }

                if self.indexed && self.current_start_line_offset > 0 {
                    let current_line = *self.newline_map.get(&(self.current_start_line_offset as usize)).unwrap();
                    self.current_start_line_offset = self.offsets_index[current_line + 1].0 as u64;
                    self.current_end_line_offset = self.offsets_index[current_line + 1].1 as u64;
                    return self.read_line(ReadMode::Current);
                } else {
                    self.current_start_line_offset = self.current_end_line_offset;
                }
            },
            ReadMode::Random => {
                if self.indexed {
                    let rnd_idx = rand::thread_rng().gen_range(0, self.offsets_index.len() - 1);
                    self.current_start_line_offset = self.offsets_index[rnd_idx].0 as u64;
                    self.current_end_line_offset = self.offsets_index[rnd_idx].1 as u64;
                    return self.read_line(ReadMode::Current);
                } else {
                    self.current_start_line_offset = rand::thread_rng().gen_range(0, self.file_size);
                }
            }
        }

        if mode != ReadMode::Current {
            self.current_start_line_offset = self.find_start_line(mode.clone())?;
            self.current_end_line_offset = self.find_end_line()?;
        }

        let offset = self.current_start_line_offset;
        let line_length = self.current_end_line_offset - self.current_start_line_offset;
        let buffer = self.read_bytes(offset, line_length as usize)?;

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

        Ok(Some(line))
    }

    fn find_start_line(&mut self, mode: ReadMode) -> io::Result<u64> {
        let mut new_start_line_offset = self.current_start_line_offset;

        let mut n_chunks = 0;
        loop {
            if new_start_line_offset == 0 { break; }

            let mut found = false;
            match mode {
                ReadMode::Prev | ReadMode::Random => {
                    let mut margin = 0;
                    let from = {
                        if new_start_line_offset < (self.chunk_size as u64) {
                            margin = self.chunk_size - (new_start_line_offset as usize);
                            0
                        } else {
                            new_start_line_offset - (self.chunk_size as u64)
                        }
                    };

                    let mut chunk = self.read_chunk(from)?;
                    chunk.reverse();

                    for (i, chunk_el) in chunk.iter().enumerate().take(self.chunk_size) {
                        if i < margin { continue; }
                        if new_start_line_offset == 0 {
                            found = true;
                            break;
                        } else {
                            if n_chunks == 0
                            && self.current_start_line_offset == new_start_line_offset
                            && mode != ReadMode::Random {
                                // Not moved yet
                                new_start_line_offset -= 1;
                                continue;
                            }

                            if *chunk_el == LF_BYTE {
                                found = true;
                            }
                        }

                        if found { break; }
                        new_start_line_offset -= 1;
                    }
                },
                ReadMode::Current => (),
                ReadMode::Next => {
                    let chunk = self.read_chunk(new_start_line_offset)?;

                    for chunk_el in chunk.iter().take(self.chunk_size) {
                        if *chunk_el == LF_BYTE {
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

    fn find_end_line(&mut self) -> io::Result<u64> {
        let mut new_end_line_offset = self.current_start_line_offset;

        loop {
            if new_end_line_offset == self.file_size { break }

            let chunk = self.read_chunk(new_end_line_offset)?;

            let mut found = false;
            for i in 0..self.chunk_size {
                if new_end_line_offset == self.file_size {
                    found = true;
                    break;
                } else if chunk[i] == LF_BYTE {
                    // Handle CRLF files
                    if i > 0 {
                        if chunk[i - 1] == CR_BYTE {
                            new_end_line_offset -= 1;
                        }
                    } else if new_end_line_offset < self.file_size {
                        let next_byte = self.read_bytes(new_end_line_offset - 1, 1)?[0];
                        if next_byte == CR_BYTE {
                            new_end_line_offset -= 1;
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

    fn read_chunk(&mut self, offset: u64) -> io::Result<Vec<u8>> {
        let chunk_size = self.chunk_size;
        self.read_bytes(offset, chunk_size)
    }

    fn read_bytes(&mut self, offset: u64, bytes: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; bytes];
        self.file.seek(SeekFrom::Start(offset as u64))?;
        self.file.read(&mut buffer)?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests;
