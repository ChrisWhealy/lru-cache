use criterion::{BenchmarkId, Criterion, Throughput};
use lru::LruCache;
use lru_cache::LruCache as MyLruCache;
use rand::Rng;
use std::{
    hint::black_box,
    num::{NonZero, NonZeroUsize},
    sync::{Arc, Barrier, Mutex},
    thread,
    time::Duration,
};

const CACHE_SIZES: [NonZero<usize>; 3] = [
    NonZeroUsize::new(1000).unwrap(),
    NonZeroUsize::new(5000).unwrap(),
    NonZeroUsize::new(10000).unwrap(),
];

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
/// Multi-threaded reads of known items from a pre-filled cache
fn get(c: &mut Criterion) {
    let mut group = c.benchmark_group("LRU Performance Comparison (Multi-threaded)");
    let barrier = Arc::new(Barrier::new(THREAD_COUNT));

    for cache_size in CACHE_SIZES {
        group.throughput(Throughput::Elements(cache_size.get() as u64));
        group.bench_with_input(
            BenchmarkId::new("get", format!("lru::LruCache-{cache_size}")),
            &cache_size,
            |b, &size| {
                b.iter_batched(
                    // Create pre-filled cache
                    || {
                        let mut cache = LruCache::new(size);

                        for i in 0..size.get() {
                            cache.put(gen_item_key(i), gen_item_value(i as u32));
                        }

                        // Wrap the cache in an Arc<Mutex<_>> to provide both shared ownership and mutable access
                        Arc::new(Mutex::new(cache))
                    },
                    // Read from pre-filled cache
                    |cache| {
                        let mut handles = vec![];

                        for _ in 0..THREAD_COUNT {
                            let cache_clone = Arc::clone(&cache);
                            let barrier_clone = Arc::clone(&barrier);

                            let handle = thread::spawn(move || {
                                let mut rng = rand::rng();
                                barrier_clone.wait();

                                for _ in 0..OPERATIONS_PER_THREAD {
                                    let mut unlocked_cache = cache_clone.lock().unwrap();
                                    let rnd_idx = rng.random_range(0..size.get());
                                    if let Some(_value) = unlocked_cache.get(&gen_item_key(rnd_idx)) {
                                    };
                                }
                            });

                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.join().unwrap();
                        }
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
                    // Create pre-filled cache
                    || {
                        let cache = MyLruCache::new(size);

                        for i in 0..size.get() {
                            cache.put(gen_item_key(i), gen_item_value(i as u32));
                        }

                        // Wrap the cache in an Arc<Mutex<_>> to provide both shared ownership and mutable access
                        Arc::new(Mutex::new(cache))
                    },
                    // Read from pre-filled cache
                    |cache| {
                        let mut handles = vec![];

                        for _ in 0..THREAD_COUNT {
                            let cache_clone = Arc::clone(&cache);
                            let barrier_clone = Arc::clone(&barrier);

                            let handle = thread::spawn(move || {
                                let mut rng = rand::rng();
                                barrier_clone.wait();

                                for _ in 0..OPERATIONS_PER_THREAD {
                                    let unlocked_cache = cache_clone.lock().unwrap();
                                    let rnd_idx = rng.random_range(0..size.get());
                                    if let Some(_value) = unlocked_cache.get(&gen_item_key(rnd_idx)) {
                                    };
                                }
                            });

                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.join().unwrap();
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
/// Multi-threaded puts of random items into a pre-filled cache
fn put(c: &mut Criterion) {
    let mut group = c.benchmark_group("LRU Performance Comparison (Multi-threaded)");
    let barrier = Arc::new(Barrier::new(THREAD_COUNT));

    for cache_size in CACHE_SIZES {
        group.throughput(Throughput::Elements(cache_size.get() as u64));
        group.bench_with_input(
            BenchmarkId::new("put", format!("lru::LruCache-{cache_size}")),
            &cache_size,
            |b, &size| {
                b.iter_batched(
                    // Create pre-filled cache
                    || {
                        let mut cache = LruCache::new(size);

                        // Pre-populate cache
                        for i in 0..size.get() {
                            cache.put(gen_item_key(i), gen_item_value(i as u32));
                        }

                        // Wrap the cache in an Arc<Mutex<_>> to provide both shared ownership and mutable access
                        Arc::new(Mutex::new(cache))
                    },
                    |cache| {
                        let mut handles = vec![];

                        for _ in 0..THREAD_COUNT {
                            let cache_clone = Arc::clone(&cache);
                            let barrier_clone = Arc::clone(&barrier);

                            let handle = thread::spawn(move || {
                                barrier_clone.wait();
                                let mut unlocked_cache = cache_clone.lock().unwrap();

                                // Perform a mix of operations
                                for idx in 0..OPERATIONS_PER_THREAD {
                                    match idx % 10 {
                                        // 70% reads
                                        0..=6 => {
                                            unlocked_cache.get(&gen_item_key(idx));
                                        }
                                        // 20% writes
                                        7..=8 => {
                                            unlocked_cache.put(gen_item_key(idx), gen_item_value(idx as u32));
                                        }
                                        // 10% get_mru
                                        9 => {
                                            unlocked_cache.pop_mru();
                                        }
                                        _ => unreachable!(),
                                    };
                                }
                            });

                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.join().unwrap();
                        }
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
                    // Wrap the cache in an Arc<Mutex<_>> to provide both shared ownership and mutable access
                    || {
                        let cache = MyLruCache::new(size);

                        // Pre-populate cache
                        for i in 0..size.get() {
                            cache.put(gen_item_key(i), gen_item_value(i as u32));
                        }

                        // Wrap the cache in an Arc<Mutex<_>> to provide both shared ownership and mutable access
                        Arc::new(Mutex::new(cache))
                    },
                    |cache| {
                        let mut handles = vec![];

                        for _ in 0..THREAD_COUNT {
                            let cache_clone = Arc::clone(&cache);
                            let barrier_clone = Arc::clone(&barrier);

                            let handle = thread::spawn(move || {
                                barrier_clone.wait();
                                let unlocked_cache = cache_clone.lock().unwrap();

                                // Perform a mix of operations
                                for idx in 0..OPERATIONS_PER_THREAD {
                                    match idx % 10 {
                                        // 70% reads
                                        0..=6 => {
                                            unlocked_cache.get(&gen_item_key(idx));
                                        }
                                        // 20% writes
                                        7..=8 => {
                                            unlocked_cache.put(gen_item_key(idx), gen_item_value(idx as u32));
                                        }
                                        // 10% get_mru
                                        9 => {
                                            unlocked_cache.pop_mru();
                                        }
                                        _ => unreachable!(),
                                    };
                                }
                            });

                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.join().unwrap();
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
pub fn main() {
    let mut criterion: Criterion<_> = Criterion::default()
        .configure_from_args()
        .measurement_time(Duration::from_secs(10));

    get(&mut criterion);
    put(&mut criterion);

    criterion.final_summary();
}
