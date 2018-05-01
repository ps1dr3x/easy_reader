# EasyReader &emsp; [![Build Status]][travis]

[Build Status]: https://travis-ci.org/ps1dr3x/easy_reader.svg?branch=master
[travis]: https://travis-ci.org/ps1dr3x/easy_reader

Move forward, backward or randomly through the lines of huge files. Easily and fastly.

#### Why?

Mainly because with Rust (currently) there isn't an easy way to read huge files line by line in reverse and/or randomly, and to freely move forwards and backwards through the lines without consuming an iterator.

### Example: basic usage

```rust
extern crate easy_reader;

use easy_reader::EasyReader;
use std::fs::File;

fn main() {
    let file: File = File::open("resources/test-file-lf").unwrap();
    let mut easy_reader: EasyReader = EasyReader::new(file).unwrap();

    println!("First line: {}", easy_reader.next_line().unwrap());
    println!("Also first line: {}", easy_reader.prev_line().unwrap());
    println!("Random line: {}", easy_reader.random_line().unwrap());

    // Iteration through the entire file (reverse)
    easy_reader.from_eof();
    while let Some(line) = easy_reader.prev_line() {
        println!("{}", line);
    }

    // You can always start/restart reading from the end of file (EOF)
    easy_reader.from_eof();
    println!("Last line: {}", easy_reader.prev_line().unwrap());
    // Or the begin of file (BOF)
    easy_reader.from_bof();
    println!("First line: {}", easy_reader.next_line().unwrap());
}
```

### Example: read random lines endlessly

```rust
extern crate easy_reader;

use easy_reader::EasyReader;
use std::fs::File;

fn main() {
    let file: File = File::open("resources/test-file-lf").unwrap();
    let mut easy_reader: EasyReader = EasyReader::new(file).unwrap();

    loop {
        println!("{}", easy_reader.random_line().unwrap());
    }
}
```
