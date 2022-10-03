// --------------------
//
// Cwago.
//
// cwago/cwago_utility/src/hash.rs
// (C) 2022 Taichi Ito.
// ====================

//! 高速なハッシュ関数、及び、ハッシュ関数を使用した集合、連想配列を提供します。

use std::{
    mem::size_of, 
    ops::BitXor, 
    hash::{
        Hasher,
        BuildHasherDefault,
    },
    collections
};

/// FxHasherを使用した集合型です。
pub type FxHashSet<T> = collections::HashSet<T, BuildFxHasher>;

/// FxHasherを使用した連想配列型です。
pub type FxHashMap<K, V> = collections::HashMap<K, V, BuildFxHasher>;

/// FxHasherを作成するビルダーです。
pub type BuildFxHasher = BuildHasherDefault<FxHasher>;

/// 高速なハッシュ計算機能です。
#[derive(Clone, Copy)]
pub struct FxHasher {
    hash: usize,
}

#[cfg(target_pointer_width = "32")]
const K: usize = 0x9e3779b9;
#[cfg(target_pointer_width = "64")]
const K: usize = 0x517cc1b727220a95;

impl FxHasher {
    #[inline]
    fn add_to_hash(&mut self, i: usize) {
        self.hash = self.hash.rotate_left(5).bitxor(i).wrapping_mul(K);
    }
}
impl Default for FxHasher {
    #[inline]
    fn default() -> FxHasher {
        FxHasher { hash: 0 }
    }
}
impl Hasher for FxHasher {
    fn finish(&self) -> u64 {
        self.hash as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        #[cfg(target_pointer_width = "32")]
        let read_usize = |bytes: &[u8]| u32::from_ne_bytes(bytes[..4].try_into().unwrap());
        #[cfg(target_pointer_width = "64")]
        let read_usize = |bytes: &[u8]| u64::from_ne_bytes(bytes[..8].try_into().unwrap());

        let mut hash = FxHasher { hash: self.hash };
        assert!(size_of::<usize>() <= 8);
        let mut bytes = bytes;
        while bytes.len() >= size_of::<usize>() {
            hash.add_to_hash(read_usize(bytes) as usize);
            bytes = &bytes[size_of::<usize>()..];
        }
        if (size_of::<usize>() > 4) && (bytes.len() >= 4) {
            hash.add_to_hash(u32::from_ne_bytes(bytes[..4].try_into().unwrap()) as usize);
            bytes = &bytes[4..];
        }
        if (size_of::<usize>() > 2) && bytes.len() >= 2 {
            hash.add_to_hash(u16::from_ne_bytes(bytes[..2].try_into().unwrap()) as usize);
            bytes = &bytes[2..];
        }
        if (size_of::<usize>() > 1) && bytes.len() >= 1 {
            hash.add_to_hash(bytes[0] as usize);
        }
        self.hash = hash.hash;
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add_to_hash(i as usize);
    }

    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as usize);
        self.add_to_hash((i >> 32) as usize);
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add_to_hash(i);
    }
}
