#[macro_use]
extern crate criterion;
extern crate easy_reader;

use std::{
    fs::File,
    io::{
        BufReader,
        BufRead
    }
};
use criterion::Criterion;
use easy_reader::EasyReader;

fn read_lf_file_forward_bufreader() {
    let f = File::open("resources/fatty_lipsum_lf").unwrap();
    let reader = BufReader::new(&f);
    for line in reader.lines() {
        line.unwrap();
    }
}

fn read_lf_file_forward() {
    let file = File::open("resources/fatty_lipsum_lf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();
    while let Ok(Some(_line)) = easy_reader.next_line() {}
}

fn read_lf_file_backward() {
    let file = File::open("resources/fatty_lipsum_lf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();
    easy_reader.from_eof();
    while let Ok(Some(_line)) = easy_reader.prev_line() {}
}

fn read_lf_x_random_lines(x: usize) {
    let file = File::open("resources/fatty_lipsum_lf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();
    for _i in 0..x {
        easy_reader.random_line().unwrap().unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("read_lf_file_forward_bufreader", |b| b.iter(|| read_lf_file_forward_bufreader()));
    c.bench_function("read_lf_file_forward", |b| b.iter(|| read_lf_file_forward()));
    c.bench_function("read_lf_file_backward", |b| b.iter(|| read_lf_file_backward()));
    c.bench_function_over_inputs("read_lf_x_random_lines", |b, &&x| {
        b.iter(|| read_lf_x_random_lines(x))
    }, &[10, 1000]);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
