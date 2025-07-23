use lru_cache::LruCache;
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    thread,
};

fn main() {
    let am_cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(2).unwrap())));

    let cache1 = Arc::clone(&am_cache);
    let cache2 = Arc::clone(&am_cache);
    let mut handles = Vec::new();

    handles.push(thread::spawn(move || {
        let mut unlocked_cache = cache1.lock().unwrap();
        unlocked_cache.put("banana", 1);
        unlocked_cache.put("pear", 2);
    }));

    handles.push(thread::spawn(move || {
        let mut unlocked_cache = cache2.lock().unwrap();
        unlocked_cache.put("apple", 3);
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    let mut cache = am_cache.lock().unwrap();
    println!("banana: {:?}", cache.get(&"banana")); // Might have been evicted
    println!("apple:  {:?}", cache.get(&"apple"));  // Might have been evicted
    println!("pear:   {:?}", cache.get(&"pear"));   // Should still be there
}
