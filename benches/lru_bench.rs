use criterion::{criterion_group, criterion_main, Criterion};
use lru_cache::LruCache;
use rand::Rng;
use std::{
    hint::black_box,
    num::{NonZero, NonZeroUsize},
    sync::{Arc, Barrier},
    thread,
};

const CACHE_SIZE_100: NonZero<usize> = NonZeroUsize::new(100).unwrap();
const CACHE_SIZE_500: NonZero<usize> = NonZeroUsize::new(500).unwrap();
const CACHE_SIZE_1K: NonZero<usize> = NonZeroUsize::new(1000).unwrap();
const CACHE_SIZE_5K: NonZero<usize> = NonZeroUsize::new(5000).unwrap();
const CACHE_SIZE_10K: NonZero<usize> = NonZeroUsize::new(10000).unwrap();
const DATASET_SIZE_1K: usize = 1000;
const DATASET_SIZE_5K: usize = 5000;
const THREAD_COUNT: usize = 8;
const OPERATIONS_PER_THREAD: usize = 1000;

// ---------------------------------------------------------------------------------------------------------------------
// Helper functions to generate test data
fn gen_item_key(idx: usize) -> String {
    black_box(format!("item-{idx}"))
}

fn gen_item_value(val: u32) -> String {
    black_box(format!("value-{val}"))
}

// ---------------------------------------------------------------------------------------------------------------------
/// Populate cache up to capacity without worrying about random number generation
fn single_threaded_insertion_without_eviction(c: &mut Criterion) {
    let cache_size = CACHE_SIZE_5K;
    let cache = LruCache::new(cache_size);

    c.bench_function("single_threaded_insertion_without_eviction", |b| {
        b.iter(|| {
            for i in 0..cache_size.get() {
                cache.put(gen_item_key(i), gen_item_value(i as u32));
            }
        })
    });
}

// ---------------------------------------------------------------------------------------------------------------------
/// A single thread performs random reads against a pre-populated cache
fn single_threaded_get(c: &mut Criterion) {
    let cache_size = CACHE_SIZE_1K;
    let mut rng = rand::rng();
    let cache = LruCache::new(cache_size);

    // Pre-populate cache
    for i in 0..cache_size.get() {
        cache.put(gen_item_key(i), gen_item_value(rng.random::<u32>()));
    }

    c.bench_function("single_threaded_get", |b| {
        let mut idx = 0;

        b.iter(|| {
            cache.get(&gen_item_key(idx));
            idx = rng.random_range(0..cache_size.get());
        })
    });
}

// ---------------------------------------------------------------------------------------------------------------------
/// `THR£AD_COUNT` threads perform random reads against a pre-filled cache
fn multi_threaded_get(c: &mut Criterion) {
    let mut rng = rand::rng();
    let cache = Arc::new(LruCache::new(CACHE_SIZE_5K));
    let barrier = Arc::new(Barrier::new(THREAD_COUNT));

    // Pre-fill cache
    for idx in 0..DATASET_SIZE_5K {
        cache.put(gen_item_key(idx), gen_item_value(rng.random::<u32>()));
    }

    c.bench_function("multi_threaded_get", |b| {
        b.iter(|| {
            let mut handles = vec![];

            for _ in 0..THREAD_COUNT {
                let cache_clone = Arc::clone(&cache);
                let barrier_clone = Arc::clone(&barrier);

                let handle = thread::spawn(move || {
                    let mut rng = rand::rng();
                    barrier_clone.wait();

                    for _ in 0..OPERATIONS_PER_THREAD {
                        cache_clone.get(&gen_item_key(rng.random_range(0..CACHE_SIZE_1K.get())));
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

// ---------------------------------------------------------------------------------------------------------------------
/// Write performance to different sized caches
fn single_threaded_put_multiple_cache_sizes(c: &mut Criterion) {
    let cache_sizes = [
        CACHE_SIZE_100,
        CACHE_SIZE_500,
        CACHE_SIZE_1K,
        CACHE_SIZE_5K,
        CACHE_SIZE_10K,
    ];

    for cs in cache_sizes {
        let cache = LruCache::new(cs);

        c.bench_function(&format!("single_threaded_put_cache_size_{cs}"), |b| {
            let mut idx = 0;
            b.iter(|| {
                cache.put(
                    black_box(gen_item_key(idx)),
                    black_box(gen_item_value(idx as u32)),
                );
                idx += 1;
            })
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------
/// A single-thread overfills the cache forcing LRU eviction
fn single_threaded_eviction_performance(c: &mut Criterion) {
    let cache = Arc::new(LruCache::new(CACHE_SIZE_1K));
    let mut rng = rand::rng();

    c.bench_function("single_threaded_eviction_performance", |b| {
        b.iter(|| {
            for idx in 0..DATASET_SIZE_5K {
                cache.put(gen_item_key(idx), gen_item_value(rng.random::<u32>()));
            }
        })
    });
}

// ---------------------------------------------------------------------------------------------------------------------
/// `THREAD_COUNT` threads each fill the cache with `CACHE_SIZE` items, but in doing so, force LRU eviction
fn multi_threaded_eviction_performance(c: &mut Criterion) {
    let cache = Arc::new(LruCache::new(CACHE_SIZE_1K));
    let barrier = Arc::new(Barrier::new(THREAD_COUNT));

    c.bench_function("multi_threaded_eviction_performance", |b| {
        b.iter(|| {
            let mut handles = vec![];

            for _ in 0..THREAD_COUNT {
                let cache_clone = Arc::clone(&cache);
                let barrier_clone = Arc::clone(&barrier);

                let handle = thread::spawn(move || {
                    let mut rng = rand::rng();
                    barrier_clone.wait();

                    for idx in 0..DATASET_SIZE_1K {
                        cache_clone.put(gen_item_key(idx), gen_item_value(rng.random::<u32>()));
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

// ---------------------------------------------------------------------------------------------------------------------
/// `THREAD_COUNT` threads perform operations designed to create contention
fn multi_threaded_contention(c: &mut Criterion) {
    let mut rng = rand::rng();
    let cache = Arc::new(LruCache::new(CACHE_SIZE_5K));
    let barrier = Arc::new(Barrier::new(THREAD_COUNT));

    // Pre-fill cache
    for idx in 0..DATASET_SIZE_5K {
        cache.put(gen_item_key(idx), gen_item_value(rng.random::<u32>()));
    }

    c.bench_function("multi_threaded_contention", |b| {
        b.iter(|| {
            let mut handles = vec![];

            for _ in 0..THREAD_COUNT {
                let cache_clone = Arc::clone(&cache);
                let barrier_clone = Arc::clone(&barrier);

                let handle = thread::spawn(move || {
                    barrier_clone.wait();

                    // Perform a mix of operations
                    for idx in 0..OPERATIONS_PER_THREAD {
                        match idx % 10 {
                            // 70% reads
                            0..=6 => cache_clone.get(&gen_item_key(idx)),
                            // 20% writes
                            7..=8 => cache_clone.put(gen_item_key(idx), gen_item_value(idx as u32)),
                            // 10% get_mru
                            9 => cache_clone.get_mru(),
                            _ => unreachable!(),
                        };
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

// ---------------------------------------------------------------------------------------------------------------------
criterion_group!(
    benches,
    single_threaded_insertion_without_eviction,
    multi_threaded_eviction_performance,
    single_threaded_get,
    multi_threaded_get,
    single_threaded_put_multiple_cache_sizes,
    single_threaded_eviction_performance,
    multi_threaded_contention
);

criterion_main!(benches);
