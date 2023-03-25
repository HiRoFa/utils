use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

/// AutoIdMap is a wrapper around HashMap which automatically creates a unique id for it's entries
/// # Example
/// ```no_run
///
/// use hirofa_utils::auto_id_map::AutoIdMap;
/// let mut map = AutoIdMap::new();
/// let id1 = map.insert("hi");
/// let id2 = map.insert("hi2");
/// assert_ne!(id1, id2);
/// assert_eq!(map.len(), 2);
/// let s1 = map.remove(&id1);
/// assert_eq!(s1, "hi");
/// assert_eq!(map.len(), 1);
/// ```
pub struct AutoIdMap<T> {
    max_size: usize,
    rng: ThreadRng,
    pub map: HashMap<usize, T>,
}

impl<T> AutoIdMap<T> {
    /// create a new instance of the AutoIdMap
    pub fn new() -> AutoIdMap<T> {
        Self::new_with_max_size(usize::MAX)
    }

    pub fn new_with_max_size(max_size: usize) -> AutoIdMap<T> {
        AutoIdMap {
            max_size,
            rng: thread_rng(),
            map: HashMap::new(),
        }
    }

    pub fn foreach_value<F: Fn(&T)>(&self, f: F) {
        for i in self.map.values() {
            f(i);
        }
    }

    pub fn foreach_value_mut<F: Fn(&mut T)>(&mut self, f: F) {
        for i in self.map.values_mut() {
            f(i);
        }
    }

    pub fn foreach<F: Fn(&usize, &T)>(&self, f: F) {
        for i in &self.map {
            f(i.0, i.1);
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn remove_values<F: Fn(&T) -> bool>(&mut self, f: F) -> Vec<T> {
        let mut rems = vec![];
        let mut rem_keys = vec![];
        {
            for i in self.map.iter() {
                if f(i.1) {
                    rem_keys.push(*i.0);
                }
            }
        }
        for k in rem_keys {
            rems.push(self.map.remove(&k).unwrap());
        }
        rems
    }

    pub fn contains_value<F: Fn(&T) -> bool>(&self, f: F) -> bool {
        for v in self.map.values() {
            if f(v) {
                return true;
            }
        }
        false
    }

    /// insert an element and return the new id
    pub fn insert(&mut self, elem: T) -> usize {
        self.try_insert(elem).expect("map is full")
    }

    /// insert an element and return the new id
    pub fn try_insert(&mut self, elem: T) -> Result<usize, &str> {
        if self.map.len() >= self.max_size {
            Err("AutoIdMap is full")
        } else {
            let mut id = self.rng.gen_range(0..self.max_size);

            while self.map.contains_key(&id) {
                if id >= self.max_size - 1 {
                    id = 0;
                } else {
                    id += 1;
                }
            }

            self.map.insert(id, elem);
            Ok(id)
        }
    }

    /// replace an element, this will panic if you pass an id that is not present
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn replace(&mut self, id: &usize, elem: T) {
        // because we really don't want you to abuse this to insert your own id's :)
        if !self.contains_key(id) {
            panic!("no entry to replace for {}", id);
        }
        self.map.insert(*id, elem);
    }

    /// get an element based on it's id
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn get(&self, id: &usize) -> Option<&T> {
        self.map.get(id)
    }

    /// get an element based on it's id
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn get_mut(&mut self, id: &usize) -> Option<&mut T> {
        self.map.get_mut(id)
    }

    /// remove an element based on its id
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn remove(&mut self, id: &usize) -> T {
        self.map.remove(id).expect("no such elem")
    }

    /// remove an element based on its id
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn remove_opt(&mut self, id: &usize) -> Option<T> {
        self.map.remove(id)
    }

    /// get the size of the map
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// see if map is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// check if a map contains a certain id
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn contains_key(&self, id: &usize) -> bool {
        self.map.contains_key(id)
    }
}

impl<T> Default for AutoIdMap<T> {
    fn default() -> Self {
        AutoIdMap::new()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::auto_id_map::AutoIdMap;

    #[test]
    fn test_aim() {
        let mut map = AutoIdMap::new_with_max_size(8);
        for _x in 0..8 {
            map.insert("foo");
        }
        assert_eq!(map.len(), 8);
        map.remove(&5);
        let free_id = map.insert("fail?");

        assert_eq!(free_id, 5);
    }

    #[test]
    fn test_aim_ms() {
        let mut map = AutoIdMap::new_with_max_size(8);
        for _x in 0..8 {
            map.insert("foo");
        }
        assert_eq!(map.len(), 8);
        map.remove(&5);
        let free_id = map.insert("fail?");

        assert_eq!(free_id, 5);

        let res = map.try_insert("foobar");
        // should be full
        assert!(res.is_err());
    }
}
