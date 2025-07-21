use std::num::{NonZero, NonZeroUsize};
use lru_cache::LruCache;
// use lru::LruCache;

use rand::{Rng, rngs::ThreadRng};

const CACHE_SIZE_1K: NonZero<usize> = NonZeroUsize::new(1000).unwrap();

const DATASET_SIZE_5K: usize = 5000;

// ---------------------------------------------------------------------------------------------------------------------
// Helper functions to generate test data
fn gen_item_key(idx: usize) -> String {
    format!("item-{idx}")
}

fn gen_item_value(idx: usize, rng: &mut ThreadRng) -> String {
    format!("Value for item-{idx}: {}", rng.random::<u32>())
}

// ---------------------------------------------------------------------------------------------------------------------
// Cache hit ratio
#[test]
fn measure_cache_hit_ratio() -> Result<(), String> {
    let hit_percentage = 0.8;
    let cache_success_threshold = 0.75;

    let cache_size = CACHE_SIZE_1K;
    let dataset_size = DATASET_SIZE_5K;
    let cache = LruCache::new(cache_size);

    let mut rng = rand::rng();
    let mut hits = 0;
    let mut misses = 0;

    // Pre-fill cache
    for idx in 0..cache_size.get() {
        cache.put(gen_item_key(idx), gen_item_value(idx, &mut rng));
    }

    // Perform more reads than there are items in the cache
    for _ in 0..dataset_size {
        // Use an 80:20 split between operations that generate a hit or a miss
        let (l_bound, u_bound) = if rng.random::<f64>() < hit_percentage {
            (0, cache_size.get())
        } else {
            (cache_size.get(), dataset_size)
        };

        // Attempt to read item from cache
        cache
            .get(&gen_item_key(rng.random_range(l_bound..u_bound)))
            .map_or_else(|| misses += 1, |_| hits += 1);
    }

    let hit_ratio = hits as f64 / (hits + misses) as f64;

    // Should have a decent hit ratio
    if hit_ratio < cache_success_threshold {
        Err(format!(
            "Poor hit ratio: Expected hit ratio >= {:.2}%, instead got {:.2}%",
            cache_success_threshold,
            hit_ratio,
        ))
    } else {
        Ok(())
    }
}
