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

/// A map for mapping Id's to arbitrary data.
///
/// `S` represents the safety of a map. A map can be either a `{SafeMap}` or an `{UnsafeMap}` where
/// `{SafeMap}` is the default. `{UnsafeMap}` is more likely to panic but should be used for
/// efficiency reasons, if the user is absolutely certain that they will not be inserting the same
/// key twice or removing/getting keys that do not exist in the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdMap<K: IdBehaviour, V, S = SafeMap> {
    vec: Vec<Option<V>>,
    key_marker: PhantomData<K>,
    safe_marker: PhantomData<S>,
    len: usize,
}

/// Remove clone requirement once extend can use into_iter
impl<K: IdBehaviour, V, S> IdMap<K, V, S> {
    /// Creates an empty `IdMap`.
    ///
    /// The map is initially created with a capacity of 0, so it will not allocate until it is
    /// first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use utils::id::{SegmentId, IdMap, UnsafeMap};
    /// // a safe map
    /// let mut map: IdMap<SegmentId, i32> = IdMap::new();
    /// // an unsafe map
    /// let mut map: IdMap<SegmentId, i32, UnsafeMap> = IdMap::new();
    /// ```
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            key_marker: std::marker::PhantomData,
            safe_marker: std::marker::PhantomData,
            len: 0,
        }
    }

    /// Creates an empty `IdMap` with at least the specified capacity.
    ///
    /// The map will be able to hold at least `capacity` elements without reallocating. This method
    /// is allowed to allocate for more elements than `capacity`. If `capacity` is 0, the map will
    /// not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use utils::id::{IdMap, SegmentId};
    /// let mut map: IdMap<SegmentId, i32> = IdMap::with_capacity(10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        let mut result = Self::new();
        result.reserve(capacity);
        result
    }

    /// Returns the total number of elements the map can hold without reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use utils::id::{IdMap, SegmentId};
    /// let mut map: IdMap<SegmentId, i32> = IdMap::with_capacity(10);
    /// assert!(map.capacity() > 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// Returns the number of elements contained in the map, also referred to
    /// as its 'length'.
    ///
    /// # Examples
    ///
    /// ```
    /// use utils::id::{IdMap, SegmentId};
    /// let mut map: IdMap<SegmentId, i32> = IdMap::with_capacity(10);
    /// assert_eq!(map.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use utils::id::{IdMap, SegmentId};
    /// let mut map: IdMap<SegmentId, i32> = IdMap::with_capacity(10);
    /// assert!(map.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clears the map, removing all values. Thus, aftewards the length of the map will be 0.
    ///
    /// Note that this method has no effect on the allocated capacity of the map.
    ///
    /// TODO add example, but how to create Id in doc code?
    pub fn clear(&mut self) {
        self.len = 0;
        for v in self.vec.iter_mut() {
            *v = None;
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `IdMap<K, V>`. The collection may reserve more space to
    /// speculatively avoid frequent reallocations. After calling `reserve`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes. This is a result
    /// of the underlying implementation using a `Vec<V>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use utils::id::{IdMap, SegmentId};
    /// let mut map: IdMap<SegmentId, i32> = IdMap::new();
    /// map.reserve(10);
    /// assert!(map.capacity() > 10);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        // round up to nearest multiple of RESERVE_CHUNKS
        let additional = additional + (RESERVE_CHUNKS - 1) & !(RESERVE_CHUNKS - 1);
        self.vec.reserve(additional);

        let new_spaces = self.vec.capacity() - self.len;
        for _ in 0..new_spaces {
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
