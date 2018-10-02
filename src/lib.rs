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
//!     let mut easy_reader = EasyReader::new(file)?;
//!
//!     println!("First line: {}", easy_reader.next_line()?.unwrap());
//!     println!("Second line: {}", easy_reader.next_line()?.unwrap());
//!     println!("First line: {}", easy_reader.prev_line()?.unwrap());
//!     println!("Random line: {}", easy_reader.random_line()?.unwrap());
//!
//!     // Iteration through the entire file (reverse)
//!     easy_reader.from_eof();
//!     while let Some(line) = easy_reader.prev_line()? {
//!         println!("{}", line);
//!     }
//! 
//!     // You can always start/restart reading from the end of file (EOF)
//!     easy_reader.from_eof();
//!     println!("Last line: {}", easy_reader.prev_line()?.unwrap());
//!     // Or the begin of file (BOF)
//!     easy_reader.from_bof();
//!     println!("First line: {}", easy_reader.next_line()?.unwrap());
//! 
//!     Ok(())
//! }
//! ```
//! 
//! ### Example: read random lines endlessly
//! 
//! ```no_run
//! extern crate easy_reader;
//!
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
//!     let mut easy_reader = EasyReader::new(file)?;
//!
//!     loop {
//!         println!("{}", easy_reader.random_line()?.unwrap());
//!     }
//! }
//! ```

extern crate rand;
extern crate fnv;

use std::{
    io::{
        self,
        prelude::*,
        Error,
        SeekFrom,
        ErrorKind
    },
    fs::File
};
use rand::Rng;
use fnv::FnvHashMap;

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
    chunk_size: usize,
    current_start_line_offset: usize,
    current_end_line_offset: usize,
    indexed: bool,
    offsets_index: Vec<(usize, usize)>,
    newline_map: FnvHashMap<usize, usize>
}

impl EasyReader {
    pub fn new(file: File) -> Result<EasyReader, Error> {
        let file_size = file.metadata()?.len() as usize;
        if file_size == 0 { return Err(Error::new(ErrorKind::UnexpectedEof, "Empty file")) }

        Ok(EasyReader {
            file: file,
            file_size: file_size,
            chunk_size: 200,
            current_start_line_offset: 0,
            current_end_line_offset: 0,
            indexed: false,
            offsets_index: Vec::new(),
            newline_map: FnvHashMap::default()
        })
    }

    pub fn chunk_size(&mut self, size: usize) -> &mut EasyReader {
        self.chunk_size = size;
        self
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

    pub fn build_index(&mut self) -> io::Result<&mut EasyReader> {
        while let Ok(Some(_line)) = self.next_line() {
            self.offsets_index.push((self.current_start_line_offset, self.current_end_line_offset));
            self.newline_map.insert(self.current_start_line_offset, self.offsets_index.len() - 1);
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
                    let current_line = *self.newline_map.get(&self.current_start_line_offset).unwrap();
                    self.current_start_line_offset = self.offsets_index[current_line - 1].0;
                    self.current_end_line_offset = self.offsets_index[current_line - 1].1;
                    return self.read_line(ReadMode::Current);
                } else {
                    self.current_end_line_offset = self.current_start_line_offset;
                }
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
                if self.current_end_line_offset == self.file_size { return Ok(None) }

                if self.indexed && self.current_start_line_offset > 0 {
                    let current_line = *self.newline_map.get(&self.current_start_line_offset).unwrap();
                    self.current_start_line_offset = self.offsets_index[current_line + 1].0;
                    self.current_end_line_offset = self.offsets_index[current_line + 1].1;
                    return self.read_line(ReadMode::Current);
                } else {
                    self.current_start_line_offset = self.current_end_line_offset;
                }
            },
            ReadMode::Random => {
                if self.indexed {
                    let rnd_idx = rand::thread_rng().gen_range(0, self.offsets_index.len() - 1);
                    self.current_start_line_offset = self.offsets_index[rnd_idx].0;
                    self.current_end_line_offset = self.offsets_index[rnd_idx].1;
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

        Ok(Some(line))
    }

    fn find_start_line(&mut self, mode: ReadMode) -> io::Result<usize> {
        let mut new_start_line_offset = self.current_start_line_offset;

        let mut n_chunks = 0;
        loop {
            if new_start_line_offset == 0 { break; }

            let mut found = false;
            match mode {
                ReadMode::Prev | ReadMode::Random => {
                    let mut margin = 0;
                    let from = {
                        if new_start_line_offset < self.chunk_size {
                            margin = self.chunk_size - new_start_line_offset;
                            0
                        } else {
                            new_start_line_offset - self.chunk_size
                        }
                    };

                    let mut chunk = self.read_chunk(from)?;
                    chunk.reverse();

                    for i in 0..self.chunk_size {
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

                    for i in 0..self.chunk_size {
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

    fn find_end_line(&mut self) -> io::Result<usize> {
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

    fn read_chunk(&mut self, offset: usize) -> io::Result<Vec<u8>> {
        let chunk_size = self.chunk_size;
        self.read_bytes(offset, chunk_size)
    }

    fn read_bytes(&mut self, offset: usize, bytes: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; bytes];
        self.file.seek(SeekFrom::Start(offset as u64))?;
        self.file.read(&mut buffer[..])?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests;
