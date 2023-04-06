use super::IdBehaviour;

use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

/// The number of elements that will be reserved space for each time the allocated memory needs to
/// be extended.
const RESERVE_CHUNKS: usize = 8192;

/// Maybe always insert
#[derive(Serialize, Deserialize)]
pub struct IdMap<K: IdBehaviour, V> {
    vec: Vec<Option<V>>,
    marker: PhantomData<K>,
    len: usize,
}

impl<K: IdBehaviour, V> IdMap<K, V> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            marker: std::marker::PhantomData,
            len: 0,
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

    pub fn len(&self) -> usize {
        self.len
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

    pub fn get(&self, k: &K) -> &V {
        let k_num = k.to_usize();
        &self.vec[k_num].as_ref().unwrap()
    }

    pub fn get_mut(&mut self, k: &K) -> &mut V {
        let k_num = k.to_usize();
        let result = &mut self.vec[k_num];
        let result = result.as_mut().unwrap();
        result
    }

    pub fn insert(&mut self, k: &K, v: V) {
        let k_num = k.to_usize();
        let vec_len = self.vec.len();

        // allocate more space if necessary
        if k_num >= vec_len {
            let additional = k_num - vec_len + 1;
            self.reserve(additional);
        }

        self.len += 1;
        self.vec[k_num] = Some(v);
    }

    /// Removes an Id from the map and returns its value. It panics if the id does not exist within
    /// the map.
    pub fn remove(&mut self, k: &K) -> V {
        let k_num = k.to_usize();

        self.len -= 1;
        std::mem::take(&mut self.vec[k_num]).unwrap()
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
}

// impl<'a, K: IdBehaviour, V> Iterator for &'a IdMap<K, V> {
//     type Item = (&'a K, &'a V);
//     fn next(&mut self) -> Option<Self::Item> {

//     }
// }
