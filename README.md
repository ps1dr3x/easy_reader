# EasyReader

[![Build Status](https://travis-ci.org/ps1dr3x/easy_reader.svg?branch=master)](https://travis-ci.org/ps1dr3x/easy_reader)
[![Latest Version](https://img.shields.io/crates/v/easy_reader.svg)](https://crates.io/crates/easy_reader)
[![Documentation](https://docs.rs/easy_reader/badge.svg)](https://docs.rs/easy_reader)
[![Rustc Version](https://img.shields.io/badge/rustc-1.52+-green.svg)](https://rust-lang.org/)

The main goal of this library is to allow long navigations through the lines of large files, freely moving forwards and backwards or getting random lines without having to consume an iterator.

Currently with Rust's standard library is possible to read a file line by line only through Lines (https://doc.rust-lang.org/std/io/trait.BufRead.html#method.lines), with which is impossible (or very expensive) to read backwards and to get random lines. Also, being an iterator, every line that has already been read is consumed and to get back to the same line you need to reinstantiate the reader and consume all the lines until the desired one (eg. in the case of the last line, all).

**Notes:**

EasyReader by default does not generate an index, it just searches for line terminators from time to time, this allows it to be used with very large files without "startup" times and excessive RAM consumption.
However, the lack of an index makes the reading slower and does not allow to take random lines with a perfect distribution, for these reasons there's a method to generate it; the start time will be slower, but all the following readings will use it and will therefore be faster (excluding the index build time, reading times are a bit longer but still comparable to those of a sequential forward reading through Lines) and in the random reading case the lines will be taken with a perfect distribution.
By the way, it's not advisable to generate the index for very large files, as an excessive RAM consumption could occur.

### Example: basic usage

```rust
use easy_reader::EasyReader;
use std::{
    fs::File,
    io::{
        self,
        Error
    }
};

fn navigate() -> Result<(), Error> {
    let file = File::open("resources/test-file-lf")?;
    let mut reader = EasyReader::new(file)?;

    // Generate index (optional)
    reader.build_index();

    // Move through the lines
    println!("First line: {}", reader.next_line()?.unwrap());
    println!("Second line: {}", reader.next_line()?.unwrap());
    println!("First line: {}", reader.prev_line()?.unwrap());
    println!("Random line: {}", reader.random_line()?.unwrap());

    // Iteration through the entire file (reverse)
    reader.eof();
    while let Some(line) = reader.prev_line()? {
        println!("{}", line);
    }

    // You can always start/restart reading from the end of file (EOF)
    reader.eof();
    println!("Last line: {}", reader.prev_line()?.unwrap());
    // Or the begin of file (BOF)
    reader.bof();
    println!("First line: {}", reader.next_line()?.unwrap());

    Ok(())
}
```

### Example: read random lines endlessly

```rust
use easy_reader::EasyReader;
use std::{
    fs::File,
    io::{
        self,
        Error
    }
};

fn navigate_forever() -> Result<(), Error> {
    let file = File::open("resources/test-file-lf")?;
    let mut reader = EasyReader::new(file)?;

    // Generate index (optional)
    reader.build_index();

    loop {
        println!("{}", reader.random_line()?.unwrap());
    }
}
```
