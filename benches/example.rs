use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gargoyle::tws::codec::

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}


const STR: &str = include_str!("sample.txt");
const DATA: &[u8] = STR.as_bytes();

fn naive_full(c: &mut Criterion) {
    c.bench_function(|| {
        DATA.iter()
            .enumerate()
            .filter_map(|(i, c)| match *c {
                0 => Some(i as u32),
                _ => None,
            })
            .collect::<Vec<_>>()
            .windows(2)
            .map(|o| std::ops::Range {
                start: o[0],
                end: o[1] - 1,
            })
            .collect::<Vec<_>>()
    })
}

fn simd_full(c: &mut Criterion) {
    c.bench_function(|| unsafe { super::calc_splits(DATA) })
}

fn naive_simple(c: &mut Criterion) {
    c.bench_function(|| {
        DATA.iter()
            .enumerate()
            .filter_map(|(i, c)| match *c {
                0 => Some(i as u32),
                _ => None,
            })
            .collect::<Vec<_>>()
    })
}

fn simd_simple(c: &mut Criterion) {
    c.bench_function(|| unsafe {
        let mut vec = Vec::new();
        super::calc_splits_dst(DATA, &mut vec);
    })
}

fn memchr_simple(c: &mut Criterion) {
    let _ = memchr::memchr_iter(0, "helloworld".as_bytes()).collect::<Vec<_>>();
    c.bench_function(|| memchr::memchr_iter(0, DATA).collect::<Vec<_>>())
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
