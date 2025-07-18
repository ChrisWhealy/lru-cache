use criterion::{criterion_group, criterion_main, Criterion};
use lru_cache::LruCache;
use rand::Rng;
use std::{
    hint::black_box,
    sync::{Arc, Barrier},
    thread,
};

const CACHE_SIZE_100: usize = 100;
const CACHE_SIZE_500: usize = 500;
const CACHE_SIZE_1K: usize = 1000;
const CACHE_SIZE_5K: usize = 5000;
const CACHE_SIZE_10K: usize = 10000;
const DATASET_SIZE_5K: usize = 5000;
const THREAD_COUNT: usize = 8;
const OPERATIONS_PER_THREAD: usize = 1000;

// ---------------------------------------------------------------------------------------------------------------------
// Helper functions to generate test data
fn gen_item_key(idx: usize) -> String {
    format!("item-{idx}")
}

fn gen_item_key_in_thread(thread_id: usize, idx: usize) -> String {
    format!("thread-{thread_id}-item-{idx}")
}

fn gen_item_value(val: u32) -> String {
    format!("value-{val}")
}

fn generate_test_data(size: usize) -> Vec<(String, String)> {
    let mut rng = rand::rng();
    (0..size)
        .map(|i| (gen_item_key(i), gen_item_value(rng.random::<u32>())))
        .collect()
}

fn generate_threaded_test_data(size: usize, thread_id: usize) -> Vec<(String, String)> {
    let mut rng = rand::rng();
    (0..size)
        .map(|i| {
            (
                gen_item_key_in_thread(thread_id, i),
                gen_item_value(rng.random::<u32>()),
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------------------------------
fn single_threaded_get(c: &mut Criterion) {
    let cache_size = CACHE_SIZE_1K;
    let mut rng = rand::rng();
    let cache = LruCache::new(cache_size);

    // Pre-populate cache
    for i in 0..cache_size {
        cache.put(gen_item_key(i), gen_item_value(rng.random::<u32>()));
    }

    c.bench_function("single_threaded_get", |b| {
        let mut idx = 0;

        b.iter(|| {
            let key = gen_item_key(black_box(idx));
            cache.get(&key);
            idx = rng.random_range(0..cache_size);
        })
    });
}

// ---------------------------------------------------------------------------------------------------------------------
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
                    gen_item_key(black_box(idx)),
                    gen_item_value(black_box(idx as u32)),
                );
                idx += 1;
            })
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------
fn single_threaded_eviction_performance(c: &mut Criterion) {
    let cache = LruCache::new(CACHE_SIZE_1K);
    let test_data = generate_test_data(DATASET_SIZE_5K);

    c.bench_function("single_threaded_eviction_performance", |b| {
        b.iter(|| {
            // Overfill cache in order to trigger evictions
            for (key, value) in &test_data {
                cache.put(key.clone(), value.clone());
            }
        })
    });
}

// ---------------------------------------------------------------------------------------------------------------------
fn multi_threaded_eviction_performance(c: &mut Criterion) {
    c.bench_function("multi_threaded_eviction_performance", |b| {
        let cache = Arc::new(LruCache::new(CACHE_SIZE_1K));
        let barrier = Arc::new(Barrier::new(THREAD_COUNT));

        b.iter(|| {
            let mut handles = vec![];

            for thread_id in 0..THREAD_COUNT {
                let test_data = Arc::new(generate_threaded_test_data(thread_id, DATASET_SIZE_5K));
                let cache_clone = Arc::clone(&cache);
                let test_data_clone = Arc::clone(&test_data);
                let barrier_clone = Arc::clone(&barrier);

                let handle = thread::spawn(move || {
                    barrier_clone.wait();

                    for i in 0..OPERATIONS_PER_THREAD {
                        // Overfill cache in order to trigger evictions
                        for (key, value) in &*test_data_clone {
                            cache_clone.put(key.clone(), value.clone());
                        }
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
fn multi_threaded_mixed_operations(c: &mut Criterion) {
    c.bench_function("multi_threaded_mixed_operations", |b| {
        let cache = Arc::new(LruCache::new(CACHE_SIZE_1K));
        let barrier = Arc::new(Barrier::new(THREAD_COUNT));

        b.iter(|| {
            let mut handles = vec![];

            for thread_id in 0..THREAD_COUNT {
                let cache_clone = Arc::clone(&cache);
                let barrier_clone = Arc::clone(&barrier);

                let handle = thread::spawn(move || {
                    barrier_clone.wait();

                    for i in 0..OPERATIONS_PER_THREAD {
                        let key = gen_item_key_in_thread(thread_id, i);
                        let value = gen_item_value(i as u32);

                        // Alternate between reading and writing
                        if i % 2 == 0 {
                            cache_clone.put(key, value);
                        } else {
                            cache_clone.get(&key);
                        }
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
    single_threaded_get,
    single_threaded_put_multiple_cache_sizes,
    single_threaded_eviction_performance,
    multi_threaded_eviction_performance,
    multi_threaded_mixed_operations
);

criterion_main!(benches);
