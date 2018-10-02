#[macro_use]
extern crate criterion;
extern crate easy_reader;

use std::fs::File;
use criterion::Criterion;
use easy_reader::EasyReader;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("build_index", |b| b.iter(|| {
        let file = File::open("resources/fatty_lipsum_lf").unwrap();
        let mut easy_reader = EasyReader::new(file).unwrap();
        easy_reader.build_index().unwrap();
    }));

    let file = File::open("resources/fatty_lipsum_lf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();
    easy_reader.build_index().unwrap();
    c.bench_function(
        "Random lines [1000][index]",
        move |b| b.iter(|| {
            for _i in 0..1000 {
                easy_reader.random_line().unwrap().unwrap();
            }
        }),
    );

    c.bench_function(
        "Random lines [1000][no_index]",
        |b| b.iter(|| {
            let file = File::open("resources/fatty_lipsum_lf").unwrap();
            let mut easy_reader = EasyReader::new(file).unwrap();
            for _i in 0..1000 {
                easy_reader.random_line().unwrap().unwrap();
            }
        }),
    );

    let file = File::open("resources/fatty_lipsum_lf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();
    easy_reader.build_index().unwrap();
    c.bench_function(
        "Read backward [1000][index]",
        move |b| b.iter(|| {
            easy_reader.from_eof();
            while let Ok(Some(_line)) = easy_reader.prev_line() {}
        }),
    );

    c.bench_function(
        "Read backward [1000][no-index]",
        move |b| b.iter(|| {
            let file = File::open("resources/fatty_lipsum_lf").unwrap();
            let mut easy_reader = EasyReader::new(file).unwrap();
            while let Ok(Some(_line)) = easy_reader.prev_line() {}
        }),
    );

    let file = File::open("resources/fatty_lipsum_lf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();
    easy_reader.build_index().unwrap();
    c.bench_function(
        "Read forward [EasyReader][index]",
        move |b| b.iter(|| {
            while let Ok(Some(_line)) = easy_reader.next_line() {}
            easy_reader.from_bof();
        }),
    );

    c.bench_function(
        "Read forward [EasyReader][no_index]",
        move |b| b.iter(|| {
            let file = File::open("resources/fatty_lipsum_lf").unwrap();
            let mut easy_reader = EasyReader::new(file).unwrap();
            while let Ok(Some(_line)) = easy_reader.next_line() {}
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
