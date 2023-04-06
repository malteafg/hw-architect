use super::IdBehaviour;

use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeMap;

/// The number of elements that will be reserved space for each time the allocated memory needs to
/// be extended.
/// This an arbitrary number maybe find better number, or scale it exponentially.
const RESERVE_CHUNKS: usize = 8192;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdMap<K: IdBehaviour, V, S = SafeMap> {
    vec: Vec<Option<V>>,
    key_marker: PhantomData<K>,
    safe_marker: PhantomData<S>,
    len: usize,
}

/// Remove clone requirement once extend can use into_iter
impl<K: IdBehaviour, V, S> IdMap<K, V, S> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            key_marker: std::marker::PhantomData,
            safe_marker: std::marker::PhantomData,
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

    pub fn clear(&mut self) {
        self.len = 0;
        for v in self.vec.iter_mut() {
            *v = None;
        }
    }

    /// Maybe subtract the current empty space from additional
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

    pub fn contains_key(&mut self, k: K) -> bool {
        let k_num = k.to_usize();
        if k_num >= self.capacity() {
            return false;
        }
        self.vec[k_num].is_some()
    }

    /// TODO check if from_usize creates a memory allocation
    pub fn keys(&self) -> impl Iterator<Item = K> + '_ {
        self.vec
            .iter()
            .enumerate()
            .filter_map(|(k_num, v)| v.as_ref().map(|_| K::from_usize(k_num)))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.vec.iter().filter_map(|v| v.as_ref())
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> + '_ {
        self.vec.iter_mut().filter_map(|v| v.as_mut())
    }

    /// TODO check if from_usize creates a memory allocation
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> + '_ {
        self.vec
            .iter()
            .enumerate()
            .filter_map(|(k_num, v)| v.as_ref().map(|v| (K::from_usize(k_num), v)))
    }

    /// TODO check if from_usize creates a memory allocation
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> + '_ {
        self.vec
            .iter_mut()
            .enumerate()
            .filter_map(|(k_num, v)| v.as_mut().map(|v| (K::from_usize(k_num), v)))
    }

    // pub fn into_iter(self) -> impl IntoIterator<Item = (K, V)> + 'static {
    //     self.vec
    //         .into_iter()
    //         .enumerate()
    //         .filter_map(|(k_num, v)| match v {
    //             Some(v) => Some((K::from_usize(k_num), v)),
    //             None => None,
    //         })
    // }
}

impl<K: IdBehaviour, V> IdMap<K, V, UnsafeMap> {
    pub fn get(&self, k: K) -> &V {
        let k_num = k.to_usize();
        let result = &self.vec[k_num];
        result.as_ref().unwrap()
    }

    pub fn get_mut(&mut self, k: K) -> &mut V {
        let k_num = k.to_usize();
        let result = &mut self.vec[k_num];
        result.as_mut().unwrap()
    }

    pub fn insert(&mut self, k: K, v: V) {
        let k_num = k.to_usize();

        #[cfg(debug_assertions)]
        if self.contains_key(k) {
            panic!("Cannot insert an already existing key into an unsafe id map!")
        }

        let capacity = self.capacity();
        if k_num >= capacity {
            let additional = k_num - capacity + 1;
            self.reserve(additional);
        }

        self.len += 1;
        self.vec[k_num] = Some(v);
    }

    /// Removes an Id from the map and returns its value.
    pub fn remove(&mut self, k: K) -> Option<V> {
        let k_num = k.to_usize();

        #[cfg(debug_assertions)]
        if !self.contains_key(k) {
            panic!("Cannot remove a non existing key from an unsafe id map!")
        }

        self.len -= 1;
        std::mem::take(&mut self.vec[k_num])
    }
}

impl<K: IdBehaviour, V: Clone> IdMap<K, V, SafeMap> {
    pub fn get(&self, k: K) -> &Option<V> {
        let k_num = k.to_usize();
        &self.vec[k_num]
    }

    pub fn get_mut(&mut self, k: K) -> &mut Option<V> {
        let k_num = k.to_usize();
        &mut self.vec[k_num]
    }

    pub fn insert(&mut self, k: K, v: V) {
        let k_num = k.to_usize();

        // this could be removed for some instances
        if self.contains_key(k) {
            self.vec[k_num] = Some(v);
            return;
        }

        let capacity = self.capacity();
        if k_num >= capacity {
            let additional = k_num - capacity + 1;
            self.reserve(additional);
        }

        self.len += 1;
        self.vec[k_num] = Some(v);
    }

    /// Removes an Id from the map and returns its value.
    pub fn remove(&mut self, k: K) -> Option<V> {
        let k_num = k.to_usize();

        // this could be removed for some instances
        if !self.contains_key(k) {
            return None;
        }

        self.len -= 1;
        std::mem::take(&mut self.vec[k_num])
    }

    /// TODO figure out how to implement into iter and remove clones.
    /// If an Id is already present, this function will override.
    pub fn extend(&mut self, other: IdMap<K, V, SafeMap>) {
        for (k, v) in other.iter() {
            self.insert(k, v.clone());
        }
    }
}

// impl<'a, K: IdBehaviour, V> Iterator for &'a IdMap<K, V> {
//     type Item = (&'a K, &'a V);
//     fn next(&mut self) -> Option<Self::Item> {

//     }
// }
