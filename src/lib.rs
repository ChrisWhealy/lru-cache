use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    num::NonZeroUsize,
    sync::Mutex,
};

// ---------------------------------------------------------------------------------------------------------------------
struct InnerCache<K, V> {
    store: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K, V> InnerCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub fn new(capacity: NonZeroUsize) -> Self {
        InnerCache {
            store: HashMap::with_capacity(capacity.get()),
            order: VecDeque::with_capacity(capacity.get()),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
/// Basic thread-safe LRU cache
pub struct LruCache<K, V> {
    capacity: NonZeroUsize,
    inner: Mutex<InnerCache<K, V>>,
}

// ---------------------------------------------------------------------------------------------------------------------
impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub fn new(capacity: NonZeroUsize) -> Self {
        LruCache {
            capacity,
            inner: Mutex::new(InnerCache::new(capacity)),
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    /// Attempt to fetch an item
    pub fn get(&self, key: &K) -> Option<V> {
        let mut inner = self.inner.lock().unwrap();

        if let Some(value) = inner.store.get(key).cloned() {
            // Update key's order to MRU
            if let Some(pos) = inner.order.iter().position(|k| *k == *key) {
                inner.order.remove(pos);
            }
            inner.order.push_front(key.clone());
            Some(value)
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    /// Removes the most recently used item
    pub fn pop_mru(&self) -> Option<V> {
        let mut inner = self.inner.lock().unwrap();

        if let Some(popped_key) = inner.order.pop_front() {
            inner.store.remove(&popped_key)
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    /// Inserts a new item.
    /// * If the item already exists, it returns the old value else it returns `None`
    /// * If the addition of the new item exceeds the cache's capacity, the oldest item is evicted before the new item is
    /// added
    pub fn put(&self, key: K, value: V) -> Option<V> {
        let mut inner = self.inner.lock().unwrap();

        // Item already exists?
        let old_value = if inner.store.contains_key(&key) {
            // Yes: update value
            let old_value = inner.store.insert(key.clone(), value);

            // Remove item's existing position in order
            if let Some(pos) = inner.order.iter().position(|k| *k == key) {
                inner.order.remove(pos);
            }
            old_value
        } else {
            // No: Add item
            if inner.store.len() >= self.capacity.get() {
                if let Some(last) = inner.order.pop_back() {
                    // Evict oldest item
                    inner.store.remove(&last);
                }
            }

            inner.store.insert(key.clone(), value);
            None
        };

        inner.order.push_front(key);
        old_value
    }
}

// ---------------------------------------------------------------------------------------------------------------------
pub mod test_utils;

#[cfg(test)]
mod unit_tests;
