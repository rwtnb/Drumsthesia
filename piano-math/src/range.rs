#![allow(dead_code)]

use std::ops::{Range, RangeBounds};

const KEY_CIS: u8 = 1;
const KEY_DIS: u8 = 3;
const KEY_FIS: u8 = 6;
const KEY_GIS: u8 = 8;
const KEY_AIS: u8 = 10;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct KeyId(u8);

pub struct KeyboardRange {
    range: Range<u8>,

    keys: Vec<KeyId>,
}

impl KeyboardRange {
    pub fn new<R>(range: R) -> Self
    where
        R: RangeBounds<usize>,
    {
        let mut keys = Vec::new();

        let start = range.start_bound();
        let end = range.end_bound();

        let start = match start {
            std::ops::Bound::Included(id) => *id,
            std::ops::Bound::Excluded(id) => *id + 1,
            std::ops::Bound::Unbounded => 0,
        } as u8;

        let end = match end {
            std::ops::Bound::Included(id) => *id + 1,
            std::ops::Bound::Excluded(id) => *id,
            std::ops::Bound::Unbounded => 0,
        } as u8;

        let range = start..end;

        for id in range.clone().map(KeyId) {
            keys.push(id);
        }

        Self {
            range,

            keys,
        }
    }

    pub fn standard_88_keys() -> Self {
        Self::new(27..53)
    }
}

impl KeyboardRange {
    pub fn contains(&self, item: u8) -> bool {
        self.range.contains(&item)
    }

    pub fn count(&self) -> usize {
        self.keys.len()
    }

    pub fn iter(&self) -> std::slice::Iter<KeyId> {
        self.keys.iter()
    }
}

impl Default for KeyboardRange {
    fn default() -> Self {
        Self::standard_88_keys()
    }
}

#[cfg(test)]
mod tests {}
