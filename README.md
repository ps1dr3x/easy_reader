# EasyReader

[![Build Status](https://travis-ci.org/ps1dr3x/easy_reader.svg?branch=master)](https://travis-ci.org/ps1dr3x/easy_reader)
[![Latest Version](https://img.shields.io/crates/v/easy_reader.svg)](https://crates.io/crates/easy_reader)
[![Documentation](https://docs.rs/easy_reader/badge.svg)](https://docs.rs/easy_reader)
[![Rustc Version](https://img.shields.io/badge/rustc-1.25+-green.svg)](https://blog.rust-lang.org/2018/03/29/Rust-1.25.html)

Move forward, backward or randomly through the lines of huge files. Easily and fastly.

#### Why?

Mainly because with Rust (currently) there isn't an easy way to read huge files line by line in reverse and/or randomly, and to freely move forwards and backwards through the lines without consuming an iterator.

### Example: basic usage

```rust
extern crate easy_reader;

use easy_reader::EasyReader;
use std::{
    fs::File,
    io::{
        self,
        Error
    }
};

fn easy() -> Result<(), Error> {
    let file = File::open("resources/test-file-lf")?;
    let mut easy_reader = EasyReader::new(file)?;

    println!("First line: {}", easy_reader.next_line()?.unwrap());
    println!("Second line: {}", easy_reader.next_line()?.unwrap());
    println!("First line: {}", easy_reader.prev_line()?.unwrap());
    println!("Random line: {}", easy_reader.random_line()?.unwrap());

    // Iteration through the entire file (reverse)
    easy_reader.from_eof();
    while let Some(line) = easy_reader.prev_line()? {
        println!("{}", line);
    }

    // You can always start/restart reading from the end of file (EOF)
    easy_reader.from_eof();
    println!("Last line: {}", easy_reader.prev_line()?.unwrap());
    // Or the begin of file (BOF)
    easy_reader.from_bof();
    println!("First line: {}", easy_reader.next_line()?.unwrap());

    Ok(())
}
```

### Example: read random lines endlessly

```rust
extern crate easy_reader;

use easy_reader::EasyReader;
use std::{
    fs::File,
    io::{
        self,
        Error
    }
};

fn easy() -> Result<(), Error> {
    let file = File::open("resources/test-file-lf")?;
    let mut easy_reader = EasyReader::new(file)?;

    loop {
        println!("{}", easy_reader.random_line()?.unwrap());
    }
}
```
