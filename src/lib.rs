use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
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
    pub fn new(capacity: usize) -> Self {
        InnerCache {
            store: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
/// Basic thread-safe LRU cache
pub struct LruCache<K, V> {
    capacity: usize,
    inner: Mutex<InnerCache<K, V>>,
}

// ---------------------------------------------------------------------------------------------------------------------
impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        LruCache {
            capacity,
            inner: Mutex::new(InnerCache::new(capacity)),
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    /// Attempt to fetch an item from get cache.
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
    /// Fetches the most recently used item from the cache.
    pub fn get_mru(&self) -> Option<V> {
        let inner = self.inner.lock().unwrap();

        if inner.order.is_empty() {
            None
        } else {
            inner.store.get(&inner.order[0]).cloned()
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    /// Inserts a new item into the cache.
    /// If the item already exists, it returns the old value else it returns `None`
    /// If the addition of the new item exceeds the cache's capacity, the oldest item is evicted
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
            if inner.store.len() >= self.capacity {
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
// TESTS
// ---------------------------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    const CAPACITY: usize = 10;

    fn test_item_key_gen(idx: usize) -> String {
        format!("item-{idx}")
    }

    fn test_item_value_gen(idx: usize) -> String {
        format!("Value for item-{idx}")
    }

    fn default_empty_cache<K, V>() -> LruCache<K, V>
    where
        K: Clone + Eq + Hash,
        V: Clone,
    {
        LruCache::new(CAPACITY)
    }

    fn default_prefilled_cache() -> LruCache<String, String> {
        let c = default_empty_cache();

        for idx in 0..CAPACITY {
            let _ = c.put(test_item_key_gen(idx), test_item_value_gen(idx));
        }

        c
    }

    // -----------------------------------------------------------------------------------------------------------------
    #[test]
    fn should_put_an_item() -> Result<(), String> {
        let c = default_empty_cache();
        let k = test_item_key_gen(1);
        let v = test_item_value_gen(1);

        c.put(k.clone(), &v);
        c.get(&k).ok_or(format!("{k} Not Found"))?;

        Ok(())
    }

    // -----------------------------------------------------------------------------------------------------------------
    #[test]
    fn should_get_an_existing_item() -> Result<(), String> {
        let c = default_prefilled_cache();
        let k = test_item_key_gen(6);

        c.get(&k).ok_or(format!("Expected item '{k}' not found"))?;

        Ok(())
    }

    // -----------------------------------------------------------------------------------------------------------------
    #[test]
    fn last_inserted_item_should_be_mru() -> Result<(), String> {
        let c = default_prefilled_cache();
        let k = test_item_key_gen(CAPACITY - 1);
        let v = test_item_value_gen(CAPACITY - 1);

        match c.get(&k) {
            Some(mru) if mru == v => Ok(()),
            Some(mru) => Err(format!("MRU item should be '{v}'. Got '{mru}' instead")),
            None => Err(format!("MRU item should be '{k}'. Got 'None' instead")),
        }
    }
    // -----------------------------------------------------------------------------------------------------------------
    #[test]
    fn should_get_expected_mru_after_reorder() -> Result<(), String> {
        let c = default_prefilled_cache();
        let k = test_item_key_gen(6);
        let v = test_item_value_gen(6);
        let err_msg = format!("MRU item should be '{v}'");

        c.get(&k).ok_or(format!("{k} not found"))?;

        match c.get_mru() {
            Some(mru) if mru == v => Ok(()),
            Some(mru) => Err(format!("{err_msg}. Got '{mru}' instead")),
            None => Err(format!("{err_msg}. Got 'None' instead")),
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    #[test]
    fn should_fail_to_get_nonexistent_item() -> Result<(), String> {
        let c = default_prefilled_cache();
        let k = test_item_key_gen(10);

        if c.get(&k).is_some() {
            Err(format!("Found item '{k}' that should not exist in cache"))
        } else {
            Ok(())
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    #[test]
    fn should_fail_to_get_evicted_item() -> Result<(), String> {
        let c = default_prefilled_cache();
        let old_k = test_item_key_gen(0);
        let new_k = test_item_key_gen(10);
        let v = test_item_value_gen(10);

        // Adding a new item to a full cache evicts the oldest item
        c.put(new_k, v);

        if c.get(&old_k).is_some() {
            Err(format!(
                "Found item '{old_k}' when it should have been evicted"
            ))
        } else {
            Ok(())
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    #[test]
    fn should_get_mru_after_item_eviction() -> Result<(), String> {
        let c = default_prefilled_cache();
        let k = test_item_key_gen(10);
        let v = test_item_value_gen(10);

        // Adding a new item evicts the oldest item and makes it the MRU
        c.put(k.clone(), v.clone());

        match c.get_mru() {
            Some(mru_val) if mru_val == v => Ok(()),
            Some(mru_val) => Err(format!("MRU item should be '{k}'. Got '{mru_val}' instead")),
            None => Err(format!("MRU item should be '{k}'. Got 'None' instead")),
        }
    }

    // -----------------------------------------------------------------------------------------------------------------
    #[test]
    fn thread2_should_add_new_item() -> Result<(), String> {
        let cache = Arc::new(LruCache::new(2));
        let k1 = String::from("apple");
        let k2 = String::from("pear");
        let k2_clone = k2.clone();

        let clone1 = Arc::clone(&cache);
        let clone2 = Arc::clone(&cache);

        let jh1 = thread::spawn(move || {
            clone1.put(k1, &1);
        });

        let jh2 = thread::spawn(move || {
            clone2.put(k2, &3);
        });

        jh1.join().unwrap();
        jh2.join().unwrap();

        if cache.get(&k2_clone).is_some() {
            Ok(())
        } else {
            Err(String::from("Expected item 'pear' not found"))
        }
    }
}
