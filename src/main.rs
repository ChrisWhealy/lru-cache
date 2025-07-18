use lru_cache::LruCache;
use std::{sync::Arc, thread};

fn main() {
    let cache = Arc::new(LruCache::new(2));

    let cache1 = Arc::clone(&cache);
    let jh1 = thread::spawn(move || {
        // cache1.put("apple", 1);
        cache1.put("banana", 1);
        cache1.put("pear", 2);
    });

    let cache2 = Arc::clone(&cache);
    let jh2 = thread::spawn(move || {
        cache2.put("apple", 3);
    });

    jh1.join().unwrap();
    jh2.join().unwrap();

    println!("banana: {:?}", cache.get(&"banana")); // Should have been evicted
    println!("apple:  {:?}", cache.get(&"pear")); // Should still be there
}
