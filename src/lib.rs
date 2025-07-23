use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    num::NonZeroUsize,
};

// ---------------------------------------------------------------------------------------------------------------------
/// LRU cache
pub struct LruCache<K, V> {
    capacity: NonZeroUsize,
    store: HashMap<K, V>,
    order: VecDeque<K>,
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
            store: HashMap::with_capacity(capacity.get()),
            order: VecDeque::with_capacity(capacity.get()),
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    /// Attempt to fetch an item
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(value) = self.store.get(key).cloned() {
            // Update key's order to MRU
            if let Some(pos) = self.order.iter().position(|k| *k == *key) {
                self.order.remove(pos);
            }
            self.order.push_front(key.clone());
            Some(value)
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    /// Removes the most recently used item
    pub fn pop_mru(&mut self) -> Option<V> {
        if let Some(popped_key) = self.order.pop_front() {
            self.store.remove(&popped_key)
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    /// Removes the least recently used item
    pub fn pop_lru(&mut self) -> Option<V> {
        if let Some(popped_key) = self.order.pop_back() {
            self.store.remove(&popped_key)
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    /// Inserts a new item.
    /// * If the item already exists, it returns the old value else it returns `None`
    /// * If the addition of the new item exceeds the cache's capacity, the oldest item is evicted before the new item is
    /// added
    pub fn put(&mut self, key: K, new_value: V) -> Option<V> {
        if self.store.contains_key(&key) {
            // Remove existing item's old position in order
            if let Some(pos) = self.order.iter().position(|k| *k == key) {
                self.order.remove(pos);
            }
        } else {
            if self.store.len() >= self.capacity.get() {
                if let Some(oldest) = self.order.pop_back() {
                    self.store.remove(&oldest);
                }
            }
        };

        self.order.push_front(key.clone());
        self.store.insert(key, new_value)
    }
}

// ---------------------------------------------------------------------------------------------------------------------
pub mod test_utils;

#[cfg(test)]
mod unit_tests;
