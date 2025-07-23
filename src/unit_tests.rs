use super::*;
use test_utils::*;
use std::{num::NonZero, sync::{Arc, Barrier, Mutex}, thread};

const CAPACITY: NonZero<usize> = NonZeroUsize::new(10).unwrap();

fn default_empty_cache<K, V>() -> LruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    LruCache::new(CAPACITY)
}

fn default_prefilled_cache() -> LruCache<String, String> {
    let mut c = default_empty_cache();

    for idx in 0..CAPACITY.get() {
        let _ = c.put(gen_item_key(idx), gen_item_value(idx as u32));
    }

    c
}

// -----------------------------------------------------------------------------------------------------------------
#[test]
fn should_put_an_item() -> Result<(), String> {
    let mut c = default_empty_cache();
    let k = gen_item_key(1);
    let v = gen_item_value(1);

    c.put(k.clone(), &v);
    c.get(&k).ok_or(format!("{k} Not Found"))?;

    Ok(())
}

// -----------------------------------------------------------------------------------------------------------------
#[test]
fn should_get_an_existing_item() -> Result<(), String> {
    let mut c = default_prefilled_cache();
    let k = gen_item_key(6);

    c.get(&k).ok_or(format!("Expected item '{k}' not found"))?;

    Ok(())
}

// -----------------------------------------------------------------------------------------------------------------
#[test]
fn last_inserted_item_should_be_mru() -> Result<(), String> {
    let mut c = default_prefilled_cache();
    let k = gen_item_key(CAPACITY.get() - 1);
    let v = gen_item_value(CAPACITY.get() as u32 - 1);

    match c.get(&k) {
        Some(mru) if mru == v => Ok(()),
        Some(mru) => Err(format!("MRU item should be '{v}'. Got '{mru}' instead")),
        None => Err(format!("MRU item should be '{k}'. Got 'None' instead")),
    }
}
// -----------------------------------------------------------------------------------------------------------------
#[test]
fn should_pop_expected_mru_after_reorder() -> Result<(), String> {
    let mut c = default_prefilled_cache();
    let k = gen_item_key(6);
    let v = gen_item_value(6);
    let err_msg = format!("MRU item should be '{v}'");

    c.get(&k).ok_or(format!("{k} not found"))?;

    match c.pop_mru() {
        Some(mru) if mru == v => Ok(()),
        Some(mru) => Err(format!("{err_msg}. Got '{mru}' instead")),
        None => Err(format!("{err_msg}. Got 'None' instead")),
    }
}

// -----------------------------------------------------------------------------------------------------------------
#[test]
fn should_fail_to_get_nonexistent_item() -> Result<(), String> {
    let mut c = default_prefilled_cache();
    let k = gen_item_key(10);

    if c.get(&k).is_some() {
        Err(format!("Found item '{k}' that should not exist in cache"))
    } else {
        Ok(())
    }
}

// -----------------------------------------------------------------------------------------------------------------
#[test]
fn should_fail_to_get_evicted_item() -> Result<(), String> {
    let mut c = default_prefilled_cache();
    let old_k = gen_item_key(0);
    let new_k = gen_item_key(10);
    let v = gen_item_value(10);

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
fn should_pop_mru_after_item_eviction() -> Result<(), String> {
    let mut c = default_prefilled_cache();
    let k = gen_item_key(10);
    let v = gen_item_value(10);

    // Adding a new item evicts the oldest item and makes it the MRU
    c.put(k.clone(), v.clone());

    match c.pop_mru() {
        Some(mru_val) if mru_val == v => Ok(()),
        Some(mru_val) => Err(format!("MRU item should be '{k}'. Got '{mru_val}' instead")),
        None => Err(format!("MRU item should be '{k}'. Got 'None' instead")),
    }
}

// -----------------------------------------------------------------------------------------------------------------
#[test]
fn thread2_should_add_new_item() -> Result<(), String> {
    let barrier = Arc::new(Barrier::new(2));
    let cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(2).unwrap())));
    let k1 = String::from("apple");
    let k2 = String::from("pear");
    let k2_clone = k2.clone();

    let clone1 = Arc::clone(&cache);
    let clone2 = Arc::clone(&cache);
    let b1 = Arc::clone(&barrier);
    let b2 = Arc::clone(&barrier);
    let mut handles = Vec::new();

    handles.push( thread::spawn(move || {
        b1.wait();
        let mut cache = clone1.lock().unwrap();
        cache.put(k1, &1);
    }));

    handles.push(thread::spawn(move || {
        b2.wait();
        let mut cache = clone2.lock().unwrap();
        cache.put(k2, &3);
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    let mut unlocked_cache = cache.lock().unwrap();
    if unlocked_cache.get(&k2_clone).is_some() {
        Ok(())
    } else {
        Err(String::from("Expected item 'pear' not found"))
    }
}
