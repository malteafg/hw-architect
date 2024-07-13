use super::IdBehaviour;

use fixedbitset::FixedBitSet;
use serde::{Deserialize, Serialize};

use rand::{thread_rng, Rng};

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

    pub fn write_into(&self, other: &mut IdSet<V>) {
        if self.capacity() > other.capacity() {
            other.reserve(self.capacity() - other.capacity());
        }

        let other_slice = other.set.as_mut_slice();
        for (i, b) in self.set.as_slice().iter().enumerate() {
            other_slice[i] = *b;
        }
    }

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

    pub fn contains(&self, v: V) -> bool {
        let v_num = v.to_usize();
        self.set.contains(v_num)
    }

    /// TODO maybe test this?
    /// Cannot be called if empty
    /// This is linear time maybe make faster.
    pub fn get_random(&self) -> V {
        let mut rng = thread_rng();
        let mut i = rng.gen_range(0..self.len());
        for v in self.iter() {
            if i == 0 {
                return v;
            }
            i -= 1;
        }
        panic!("Could not find element in id set");
    }

    pub fn insert(&mut self, v: V) -> bool {
        let v_num = v.to_usize();
        let capacity = self.capacity();

        if v_num >= capacity {
            let additional = v_num - capacity + 1;
            self.reserve(additional);
        }

        self.set.put(v_num)
    }

    pub fn remove(&mut self, v: V) -> bool {
        let v_num = v.to_usize();

        if v_num >= self.capacity() {
            return false;
        }

        let result = self.set.contains(v_num);
        self.set.set(v_num, false);
        result
    }

    pub fn iter(&self) -> impl Iterator<Item = V> + '_ {
        (0..self.capacity()).filter_map(|v_num| {
            if self.set.contains(v_num) {
                Some(V::from_usize(v_num))
            } else {
                None
            }
        })
    }
}
