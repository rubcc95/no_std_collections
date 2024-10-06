use std::hash::{BuildHasher, RandomState};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use no_std_collections::traits::vec::Vec as VecTrait;
use rand::{Rng, RngCore};

#[cfg(feature = "std")]
mod bench{
    use super::*;
    pub fn push(c: &mut Criterion) {
        let mut group = c.benchmark_group("Push");
    
        group.bench_function("Vec", |b| {
            b.iter_batched(
                || {
                    let rng = RandomState::new();
                    let mut res = [0; 1000];
                    for i in 0..1000 {
                        res[i] = rng.hash_one(i);
                    }
                    (res, Vec::with_capacity(1000))
                },
                |(data, mut vec)| {
                    for i in data.into_iter() {
                        vec.push(black_box(i));
                    }
                    //black_box(unsafe { vec.get_unchecked(99) });
                },
                criterion::BatchSize::NumIterations(100),
            );
        });
        group.bench_function("VecTrait", |b| {
            b.iter_batched(
                || {
                    let rng = RandomState::new();
                    let mut res = [0; 1000];
                    for i in 0..1000 {
                        res[i] = rng.hash_one(i);
                    }
                    (res, Vec::with_capacity(1000))
                },
                |(data, mut vec)| {
                    for i in data.into_iter() {
                        VecTrait::push(&mut vec, black_box(i));
                    }
                    //black_box(unsafe { vec.get_unchecked(99) });
                },
                criterion::BatchSize::NumIterations(100),
            );
        });
    
        group.finish();
    }
    
    pub fn remove(c: &mut Criterion) {
        let mut group = c.benchmark_group("Remove");
    
        group.bench_function("Vec", |b| {
            b.iter_batched(
                || {
                    let rng = RandomState::new();
                    let res: Vec<u64> = (0..1000).into_iter().map(|i| rng.hash_one(i)).collect();
                    res
                },
                |mut vec| {
                    while vec.len() > 1 {
                        vec.remove(black_box(1));
                    }
                },
                criterion::BatchSize::NumIterations(100),
            );
        });
        group.bench_function("VecTrait", |b| {
            b.iter_batched(
                || {
                    let rng = RandomState::new();
                    let res: Vec<u64> = (0..1000).into_iter().map(|i| rng.hash_one(i)).collect();
                    res
                },
                |mut vec| {
                    while vec.len() > 1 {
                        VecTrait::remove(&mut vec, black_box(1));
                    }
                },
                criterion::BatchSize::NumIterations(100),
            );
        });
    
        group.finish();
    }
    
    pub fn insert(c: &mut Criterion) {
        let mut group = c.benchmark_group("Insert");
        let mut rng = rand::thread_rng();
    
        group.bench_function("Vec", |b| {
            b.iter_batched(
                || {
                    (
                        (1001..1).into_iter().map(|i| rng.gen_range(0..i)).collect(),
                        (0..1000).into_iter().map(|_| rng.next_u32()).collect(),
                    )
                },
                |(order, values): (Vec<usize>, Vec<u32>)| {
                    let mut vec = Vec::new();
                    let ptr = values.as_ptr();
                    for &i in &order {
                        vec.insert(i, unsafe { *ptr });
                    }
                },
                criterion::BatchSize::NumIterations(100),
            );
        });
        group.bench_function("VecTrait", |b| {
            b.iter_batched(
                || {
                    (
                        (1001..1).into_iter().map(|i| rng.gen_range(0..i)).collect(),
                        (0..1000).into_iter().map(|_| rng.next_u32()).collect(),
                    )
                },
                |(order, values): (Vec<usize>, Vec<u32>)| {
                    let mut vec = Vec::new();
                    let ptr = values.as_ptr();
                    for &i in &order {
                        VecTrait::insert(&mut vec, i, unsafe { *ptr });
                    }
                },
                criterion::BatchSize::NumIterations(100),
            );
        });
    
        group.finish();
    }
    
    pub fn with_capacity(c: &mut Criterion) {
        let mut group = c.benchmark_group("Unchecked Add Diff");
        let mut rng = rand::thread_rng();
    
        group.bench_function("Checked", |b| {
            b.iter_batched(
                || {
                        (0..1000).into_iter().map(|_| rng.next_u32()).collect()
                    
                },
                |values: Vec<u32>| {
                    let mut vec = Vec::with_capacity(1000);                
                    for &i in &values {
                        vec.push(i);
                    }
                },
                criterion::BatchSize::NumIterations(100),
            );
        });
        group.bench_function("Unchecked", |b| {
            b.iter_batched(
                || {
                        (0..1000).into_iter().map(|_| rng.next_u32()).collect()
                    
                },
                |values: Vec<u32>| {
                    let mut vec = Vec::with_capacity(1000);                
                    for &i in &values {
                        unsafe { VecTrait::push_unchecked(&mut vec, i) };
                    }
                },
                criterion::BatchSize::NumIterations(100),
            );
        });
    
        group.finish();
    }
    
}
#[cfg(feature = "std")]
criterion_group!(benches, bench::insert, bench::push, bench::remove, bench::with_capacity);

#[cfg(feature = "std")]
criterion_main!(benches);

#[cfg(not(feature = "std"))]
fn main(){

}
