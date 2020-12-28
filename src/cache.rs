use linked_hash_map::LinkedHashMap;
use std::ops::{Div, Sub};
use std::time::{Duration, Instant};

pub trait CacheIFace<K: std::cmp::Eq, O> {
    fn invalidate_all(&mut self);
    fn invalidate_stale(&mut self);
    fn opt(&mut self, key: &K) -> Option<&O>;
    fn opt_no_touch(&self, key: &K) -> Option<&O>;
    fn get(&mut self, key: &K) -> Option<&O>;
    fn contains_key(&self, key: &K) -> bool;
    fn invalidate(&mut self, key: &K);
    fn insert(&mut self, key: K, item: O);
}

struct CacheEntry<O> {
    item: O,
    last_used: Instant,
}

pub struct Cache<K: std::cmp::Eq + std::hash::Hash, O> {
    // on every get remove and add (oldest items come first)
    entries: LinkedHashMap<K, CacheEntry<O>>,
    producer: Box<dyn Fn(&K) -> Option<O>>,
    max_inactive_time: Duration,
    inactive_resolution: Duration,
    max_size: usize,
}

impl<K: std::cmp::Eq + std::hash::Hash, O> Cache<K, O> {
    pub fn new<P>(producer: P, max_inactive_time: Duration, max_size: usize) -> Self
    where
        P: Fn(&K) -> Option<O> + 'static,
    {
        let inactive_resolution = max_inactive_time.div(10);
        Cache {
            entries: LinkedHashMap::new(),
            producer: Box::new(producer),
            max_inactive_time,
            inactive_resolution,
            max_size,
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<K: std::cmp::Eq + std::hash::Hash + Clone, O> CacheIFace<K, O> for Cache<K, O> {
    fn invalidate_all(&mut self) {
        self.entries.clear();
    }

    fn invalidate_stale(&mut self) {
        let now = Instant::now();
        let max_age = now.sub(self.max_inactive_time);

        loop {
            let front_opt: Option<(&K, &CacheEntry<O>)> = self.entries.front();
            if let Some(entry) = front_opt {
                let e = entry.1;
                if e.last_used.lt(&max_age) {
                    let _drop_entry = self.entries.pop_front();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn opt(&mut self, key: &K) -> Option<&O> {
        let entry_opt = self.entries.get_mut(key);
        if let Some(e) = entry_opt {
            let now = Instant::now();
            // check if the entry falls outside the resolution , prevents entries being reinserted on every get
            if e.last_used.lt(&now.sub(self.inactive_resolution)) {
                drop(e);
                let mut removed_entry = self.entries.remove(key).unwrap();
                removed_entry.last_used = now;
                self.entries.insert(key.clone(), removed_entry);
            }
            self.entries.get(key).map(|i| &i.item)
        } else {
            None
        }
    }

    fn opt_no_touch(&self, key: &K) -> Option<&O> {
        self.entries.get(key).map(|e| &e.item)
    }

    fn get(&mut self, key: &K) -> Option<&O> {
        self.invalidate_stale();
        if self.contains_key(key) {
            self.opt(key)
        } else {
            let item_opt = (*self.producer)(key);
            if let Some(item) = item_opt {
                self.insert(key.clone(), item);
            }
            self.opt(key)
        }
    }

    fn contains_key(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }

    fn invalidate(&mut self, key: &K) {
        self.entries.remove(key);
    }

    fn insert(&mut self, key: K, item: O) {
        let entry = CacheEntry {
            item,
            last_used: Instant::now(),
        };
        self.entries.insert(key.clone(), entry);
        while self.entries.len() > self.max_size {
            let _drop_entry = self.entries.pop_front();
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::cache::{Cache, CacheIFace};
    use std::time::Duration;

    #[test]
    fn test_cache() {
        let producer = |key: &i32| Some(format!("entry: {}", key));
        let mut cache = Cache::new(producer, Duration::from_secs(2), 10);

        let _one = cache.get(&1);
        let _two = cache.get(&2);
        let three = cache.get(&3);
        assert_eq!(three.expect("three not found").as_str(), "entry: 3");

        assert_eq!(3, cache.len());

        std::thread::sleep(Duration::from_secs(3));
        cache.invalidate_stale();

        assert_eq!(0, cache.len());

        for x in 0..15 {
            let _ = cache.get(&x);
        }

        assert_eq!(10, cache.len());
    }
}
