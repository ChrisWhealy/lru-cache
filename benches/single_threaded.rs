mod common;

use common::*;
use lru_cache::test_utils::*;
use criterion::{BenchmarkId, Criterion, Throughput};
use lru::LruCache;
use lru_cache::LruCache as MyLruCache;
use rand::Rng;
use std::time::Duration;

// ---------------------------------------------------------------------------------------------------------------------
/// Exactly fill the cache
fn insertion_without_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("LRU Performance Comparison (Single Threaded)");

    for cache_size in CACHE_SIZES {
        group.throughput(Throughput::Elements(cache_size.get() as u64));
        group.bench_with_input(
            BenchmarkId::new(
                "insertion_without_eviction",
                format!("MyLruCache-{cache_size}"),
            ),
            &cache_size,
            |b, &size| {
                b.iter_batched(
                    || MyLruCache::new(size),
                    |mut cache| {
                        for i in 0..size.get() {
                            cache.put(gen_item_key(i), gen_item_value(i as u32));
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new(
                "insertion_without_eviction",
                format!("lru::LruCache-{cache_size}"),
            ),
            &cache_size,
            |b, &size| {
                b.iter_batched(
                    || LruCache::new(size),
                    |mut cache| {
                        for i in 0..size.get() {
                            cache.put(gen_item_key(i), gen_item_value(i as u32));
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------------------------------------------------
/// Randomly read known items from a pre-populated cache
fn get(c: &mut Criterion) {
    let mut group = c.benchmark_group("LRU Performance Comparison (Single Threaded)");
    let mut rng = rand::rng();

    for cache_size in CACHE_SIZES {
        group.throughput(Throughput::Elements(cache_size.get() as u64));
        group.bench_with_input(
            BenchmarkId::new("get", format!("lru::LruCache-{cache_size}")),
            &cache_size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let mut cache = LruCache::new(size);

                        // Pre-populate cache
                        for i in 0..size.get() {
                            cache.put(gen_item_key(i), gen_item_value(i as u32));
                        }

                        cache
                    },
                    |mut cache| {
                        cache.get(&gen_item_key(rng.random_range(0..size.get())));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("get", format!("lru_cache::MyLruCache-{cache_size}")),
            &cache_size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let mut cache = MyLruCache::new(size);

                        // Pre-populate cache
                        for i in 0..size.get() {
                            cache.put(gen_item_key(i), gen_item_value(i as u32));
                        }

                        cache
                    },
                    |mut cache| {
                        let mut rng = rand::rng();
                        cache.get(&gen_item_key(rng.random_range(0..size.get())));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------------------------------------------------
/// Randomly write items to the cache that have a 50% likelihood of already being present
fn put(c: &mut Criterion) {
    let mut group = c.benchmark_group("LRU Performance Comparison (Single Threaded)");
    let mut rng = rand::rng();

    for cache_size in CACHE_SIZES {
        group.throughput(Throughput::Elements(cache_size.get() as u64));
        group.bench_with_input(
            BenchmarkId::new("put", format!("lru::LruCache-{cache_size}")),
            &cache_size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let mut cache = LruCache::new(size);

                        // Pre-populate cache
                        for i in 0..size.get() {
                            cache.put(gen_item_key(i), gen_item_value(i as u32));
                        }

                        cache
                    },
                    |mut cache| {
                        cache.put(
                            gen_item_key(rng.random_range(0..size.get() * 2)),
                            gen_item_value(rng.random_range(0..size.get() * 2) as u32),
                        );
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("put", format!("lru_cache::MyLruCache-{cache_size}")),
            &cache_size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let mut cache = MyLruCache::new(size);

                        // Pre-populate cache
                        for i in 0..size.get() {
                            cache.put(gen_item_key(i), gen_item_value(i as u32));
                        }

                        cache
                    },
                    |mut cache| {
                        cache.put(
                            gen_item_key(rng.random_range(0..size.get() * 2)),
                            gen_item_value(rng.random_range(0..size.get() * 2) as u32),
                        );
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------------------------------------------------
pub fn main() {
    let mut criterion: Criterion<_> = Criterion::default()
        .configure_from_args()
        .measurement_time(Duration::from_secs(10));

    insertion_without_eviction(&mut criterion);
    get(&mut criterion);
    put(&mut criterion);

    criterion.final_summary();
}
