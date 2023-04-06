use super::IdBehaviour;

use fixedbitset::FixedBitSet;
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

const RESERVE_CHUNKS: usize = 8192;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdSet<V: IdBehaviour> {
    set: FixedBitSet,
    marker: PhantomData<V>,
}

impl<V: IdBehaviour> IdSet<V> {
    pub fn new() -> Self {
        Self {
            set: FixedBitSet::new(),
            marker: std::marker::PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            set: FixedBitSet::with_capacity(capacity),
            marker: std::marker::PhantomData,
        }
    }

    pub fn capacity(&self) -> usize {
        self.set.len()
    }

    pub fn len(&self) -> usize {
        self.set.count_ones(..)
    }

    pub fn clear(&mut self) {
        self.set.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        // round up to nearest multiple of RESERVE_CHUNKS
        let additional = additional + (RESERVE_CHUNKS - 1) & !(RESERVE_CHUNKS - 1);
        let bits = self.len() + additional;
        self.set.grow(bits);
    }

    // pub fn shrink_to_fit(&mut self) {}

    pub fn is_empty(&self) -> bool {
        self.set.is_clear()
    }

    pub fn is_disjoint(&self, other: &IdSet<V>) -> bool {
        self.set.is_disjoint(&other.set)
    }

    pub fn is_subset(&self, other: &IdSet<V>) -> bool {
        self.set.is_subset(&other.set)
    }

    pub fn is_superset(&self, other: &IdSet<V>) -> bool {
        self.set.is_superset(&other.set)
    }

    pub fn contains(&self, v: &V) -> bool {
        let v_num = v.to_usize();
        self.set.contains(v_num)
    }

    pub fn insert(&mut self, v: &V) -> bool {
        let v_num = v.to_usize();
        let set_len = self.set.len();

        if v_num >= set_len {
            let additional = v_num - set_len + 1;
            self.reserve(additional);
        }

        self.set.put(v_num)
    }

    pub fn remove(&mut self, v: &V) -> bool {
        let v_num = v.to_usize();
        let result = self.set.contains(v_num);

        self.set.set(v_num, false);
        result
    }
}
