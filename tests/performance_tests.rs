use std::{
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};

use lru_cache::LruCache;
use rand::{Rng, rngs::ThreadRng};

const CACHE_SIZE_1K: usize = 1000;
const CACHE_SIZE_10K: usize = 10000;

const DATASET_SIZE_5K: usize = 5000;
const DATASET_SIZE_10K: usize = 10000;

const THREAD_COUNT: usize = 8;
const OPERATIONS_PER_THREAD: usize = 1000;
const WORKLOAD_TEST_DURATION: Duration = Duration::from_secs(5);

// ---------------------------------------------------------------------------------------------------------------------
// Helper functions to generate test data
fn gen_item_key(idx: usize) -> String {
    format!("item-{idx}")
}

fn gen_item_key_in_thread(thread_id: usize, idx: usize) -> String {
    format!("thread-{thread_id}-item-{idx}")
}

fn gen_item_value(idx: usize, rng: &mut ThreadRng) -> String {
    format!("Value for item-{idx}: {}", rng.random::<u32>())
}

fn generate_test_data(size: usize) -> Vec<(String, String)> {
    let mut rng = rand::rng();
    (0..size)
        .map(|i| (gen_item_key(i), gen_item_value(i, &mut rng)))
        .collect()
}

// ---------------------------------------------------------------------------------------------------------------------
#[test]
fn measure_single_thread_read_write_performance() -> Result<(), String> {
    let cache_size = CACHE_SIZE_1K;
    let cache = LruCache::new(cache_size);
    let test_data = generate_test_data(DATASET_SIZE_10K);

    // Fill cache
    for (key, value) in &test_data[..cache_size] {
        cache.put(key.clone(), value.clone());
    }

    // Don't care how long it takes to fill the cache
    let start = Instant::now();

    // Read every third value - lots of cache misses are expected
    for (i, (key, value)) in test_data.iter().enumerate() {
        if i % 3 == 0 {
            cache.get(key);
        } else {
            cache.put(key.clone(), value.clone());
        }
    }

    println!(
        "Single thread: {} mixed read/write operation in {:?}",
        DATASET_SIZE_10K,
        start.elapsed()
    );

    // The cache should contain at least one item
    cache
        .get_mru()
        .ok_or(String::from("Cache is empty!"))
        .map(|_| ())
}

// ---------------------------------------------------------------------------------------------------------------------
// Multi-threaded contention test
#[test]
fn measure_multi_threaded_contention() {
    let cache = Arc::new(LruCache::new(CACHE_SIZE_10K));
    let barrier = Arc::new(Barrier::new(THREAD_COUNT));
    let mut handles = vec![];

    let start = Instant::now();

    for thread_id in 0..THREAD_COUNT {
        let cache_clone = Arc::clone(&cache);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let thread_data: Vec<(String, String)> = (0..OPERATIONS_PER_THREAD)
                .map(|i| {
                    (
                        gen_item_key_in_thread(thread_id, i),
                        gen_item_value(i, &mut rand::rng()),
                    )
                })
                .collect();

            // Wait til everyone is ready
            barrier_clone.wait();

            let thread_start = Instant::now();

            // Perform a mix of operations
            for (i, (key, value)) in thread_data.iter().enumerate() {
                match i % 4 {
                    // 50% are reads
                    0 | 1 => cache_clone.get(key),
                    // 25% are writes
                    2 => cache_clone.put(key.clone(), value.clone()),
                    // 25% access MRU
                    3 => cache_clone.get_mru(),
                    _ => unreachable!(),
                };
            }

            thread_start.elapsed()
        });

        handles.push(handle);
    }

    let mut thread_times = Vec::new();

    // Off we go...
    for handle in handles {
        thread_times.push(handle.join().unwrap());
    }

    let total_time = start.elapsed();
    let avg_thread_time: Duration =
        thread_times.iter().sum::<Duration>() / thread_times.len() as u32;

    println!(
        "Multi-thread ({} threads): Total time: {:?}, Avg thread time: {:?}",
        THREAD_COUNT, total_time, avg_thread_time
    );
    println!(
        "Throughput: {:.3} ops/sec",
        (THREAD_COUNT * OPERATIONS_PER_THREAD) as f64 / total_time.as_secs_f64()
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// Cache hit ratio
#[test]
fn measure_cache_hit_ratio() -> Result<(), String> {
    let cache_size = CACHE_SIZE_1K;
    let dataset_size = DATASET_SIZE_5K;
    let hit_percentage = 0.8;
    let cache_success_threshold = 0.8;
    let cache = Arc::new(LruCache::new(cache_size));
    let test_data = generate_test_data(dataset_size);
    let mut rng = rand::rng();
    let mut hits = 0;
    let mut misses = 0;

    // Fill cache
    for (key, value) in &test_data[..cache_size] {
        cache.put(key.clone(), value.clone());
    }

    // Don't care how long it took to fill the cache
    let start = Instant::now();

    for _ in 0..dataset_size * 2 {
        // Use an 80:20 split between operations that generate a hit or a miss
        let (l_bound, u_bound) = if rng.random::<f64>() < hit_percentage {
            (0, cache_size)
        } else {
            (cache_size, dataset_size)
        };

        // Attempt to read item from cache
        cache
            .get(&test_data[rng.random_range(l_bound..u_bound)].0)
            .map_or_else(|| misses += 1, |_| hits += 1);
    }

    let duration = start.elapsed();
    let hit_ratio = hits as f64 / (hits + misses) as f64;

    println!(
        "Hit ratio test: {:.2}% hits, {:?} for 10k gets",
        hit_ratio * 100.0,
        duration
    );

    // Should have decent hit ratio
    if hit_ratio < cache_success_threshold {
        Err(format!(
            "Poor hit ratio ({:.2}%) for {} reads on cache of size {}",
            hit_ratio,
            dataset_size * 2,
            cache_size
        ))
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Measure performance under realistic workload patterns
#[test]
fn measure_workload() -> Result<(), String> {
    let cache_size = CACHE_SIZE_1K;
    let cache = Arc::new(LruCache::new(cache_size));
    let num_threads = THREAD_COUNT;

    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    for _thread_id in 0..num_threads {
        let cache_clone = Arc::clone(&cache);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let mut rng = rand::rng();
            let mut operations = 0;

            // Are we all ready?
            barrier_clone.wait();

            let start = Instant::now();

            while start.elapsed() < WORKLOAD_TEST_DURATION {
                // Generated key is 50% likely to be absent from the cache
                let idx = rng.random_range(0..cache_size * 2);
                let key = gen_item_key(idx);
                let value = gen_item_value(idx, &mut rng);

                match rng.random_range(0..10) {
                    // 70% reads
                    0..=6 => cache_clone.get(&key),
                    // 20% writes
                    7..=8 => cache_clone.put(key, value),
                    // 10% get_mru
                    9 => cache_clone.get_mru(),
                    _ => unreachable!(),
                };
                operations += 1;
            }

            (operations, start.elapsed())
        });

        handles.push(handle);
    }

    let mut total_operations = 0;
    for handle in handles {
        let (ops, _) = handle.join().unwrap();
        total_operations += ops;
    }

    let throughput = total_operations as f64 / WORKLOAD_TEST_DURATION.as_secs_f64();
    println!(
        "Realistic workload: {:.0} ops/sec over {} seconds",
        throughput,
        WORKLOAD_TEST_DURATION.as_secs()
    );

    Ok(())
}
