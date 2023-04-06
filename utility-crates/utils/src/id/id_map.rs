use super::IdBehaviour;

use std::marker::PhantomData;

/// The number of elements that will be reserved space for each time the allocated memory needs to
/// be extended.
const RESERVE_CHUNKS: usize = 8192;

/// Maybe always insert
pub struct IdMap<K: IdBehaviour, V> {
    vec: Vec<Option<V>>,
    marker: PhantomData<K>,
}

impl<K: IdBehaviour, V> IdMap<K, V> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            marker: std::marker::PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut result = Self::new();
        result.reserve(capacity);
        result
    }

    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    pub fn insert(&mut self, k: &K, v: V) {
        let k_num = k.to_usize();
        let max_index = self.vec.len() - 1;

        // allocate more space if necessary
        if k_num > max_index {
            let additional = k_num - max_index;
            self.reserve(additional);
        }

        self.vec[k_num] = Some(v);
    }

    pub fn iter_values(&self) -> impl Iterator<Item = &V> + '_ {
        self.vec.iter().filter_map(|v| v.as_ref())
    }

    pub fn iter_values_mut(&mut self) -> impl Iterator<Item = &mut V> + '_ {
        self.vec.iter_mut().filter_map(|v| v.as_mut())
    }

    /// TODO check if from_usize creates a memory allocation
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> + '_ {
        self.vec
            .iter()
            .enumerate()
            .filter_map(|(k, v)| v.as_ref().map(|v| (K::from_usize(k), v)))
    }

    /// TODO check if from_usize creates a memory allocation
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> + '_ {
        self.vec
            .iter_mut()
            .enumerate()
            .filter_map(|(k, v)| v.as_mut().map(|v| (K::from_usize(k), v)))
    }

    /// Removes an Id from the map and returns its value. It panics if the id does not exist within
    /// the map.
    pub fn remove(&mut self, k: &K) -> V {
        let k_num = k.to_usize();
        let result = std::mem::take(&mut self.vec[k_num]);
        result.unwrap()
    }

    pub fn reserve(&mut self, additional: usize) {
        // round up to nearest multiple of RESERVE_CHUNKS
        let additional = additional + (RESERVE_CHUNKS - 1) & !(RESERVE_CHUNKS - 1);
        self.vec.reserve(additional);

        for _ in 0..additional {
            self.vec.push(None)
        }
    }

    pub fn shrink_to_fit(&mut self) {
        // iterate backwards and remove None until Some
        for i in 0..self.vec.len() {
            let i = self.vec.len() - i;
            if let Some(_) = self.vec[i] {
                break;
            }
            self.vec.pop();
        }

        self.vec.shrink_to_fit();
    }
}

// impl<'a, K: IdBehaviour, V> Iterator for &'a IdMap<K, V> {
//     type Item = (&'a K, &'a V);
//     fn next(&mut self) -> Option<Self::Item> {

//     }
// }
