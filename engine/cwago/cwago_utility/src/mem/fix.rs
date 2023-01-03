// -------------------------
//
// Cwago.
//
// cwago/cwago_utility/src/mem/fix.rs
// (C) 2023 CwagoCommunity.
//
//! 固定長メモリを提供します。
// =========================

use std::{
    alloc::Layout,
    mem::{
        size_of, 
        transmute
    },
    ptr::{
        null_mut, 
        drop_in_place
    }
};
use log::error;
use super::{
    os::OSMemory,
    pool::Pool
};

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE_MAX: usize = 256;
    const COUNT_MAX: usize = 256;
    const LENGTH_MAX: usize = COUNT_MAX * 2;

    #[test]
    fn test_fix_memory() {

        std::env::set_var("RUST_LOG", "error");
        env_logger::init();

        // サイズが1~256までで作成可能かテストします。
        for size in 1..SIZE_MAX {
            // 要素数が1~256までで作成可能かテストします。
            for count in 1..COUNT_MAX {
                print!("test size:{} count:{} \r", size, count);
                test_fix_memory_one(size, count);
            }
        }
        println!();
    }
    fn test_fix_memory_one(size: usize, count: usize) {

        // 作成します。
        let mut mem = FixMemory::new(size, count);

        // 使いまわしが可能かテストします。
        for _lap in 0..3usize {

            let mut ptrs = [null_mut::<u8>(); LENGTH_MAX];

            // メモリが確保可能かテストします。
            for i in 0..LENGTH_MAX {
                ptrs[i] = mem.alloc();
                assert_ne!(ptrs[i], null_mut(), "{}回目のメモリ確保で失敗しました。", i);
            }

            for i in 1..mem.pools_count {
                assert!(unsafe { &**mem.pools.add(i - 1) }.min_address() < unsafe { &**mem.pools.add(i) }.min_address(), "並んでない");
            }

            // 確保したメモリに重複が無いかテストします。
            for i in 0..LENGTH_MAX {
                unsafe { *ptrs[i] = i as u8 };
            }
            for i in 0..LENGTH_MAX {
                assert_eq!(unsafe{ *ptrs[i] }, i as u8, "{}回目に確保したメモリに設定されていた値は{}でした。", i, unsafe{ *ptrs[i] });
            }

            // メモリを要素数解放可能かテストします。
            for i in 0..LENGTH_MAX {
                assert!(mem.dealloc(ptrs[i]), "{}回目に確保したメモリを解放できませんでした。", i);
            }
        }
    }
}

/// 固定長メモリを管理します。
#[derive(Debug)]
pub(super) struct FixMemory {
    elements_size: usize,  // メモリのサイズです。
    elements_count: usize, // 1つのプールが管理する要素数です。
    pools_length: usize,   // プール配列の配列長です。
    pools_count: usize,    // 現在管理しているプールの数です。
    pools_layout: Layout,  // プール配列のメモリレイアウトです。
    pool_layout: Layout,   // プールのメモリレイアウトです。
    pools: *mut *mut Pool, // プール配列です。
    alloc_pool: *mut Pool, // アロケート対象のプールです。
}
impl FixMemory {

    const INIT_POOLS_LENGTH: usize = 8;
    const EXPANSION_MULTIPLY: usize = 2;

    /// 固定長メモリマネージャを作成します。
    /// 
    /// # 引数
    /// 
    /// * elements_size - メモリのサイズです。
    /// * elements_count - 1つのプールが管理する要素数です。
    /// 
    /// # 戻り値
    /// 
    /// 成功した際はインスタンス、失敗した際はNoneが返ります。
    /// 
    pub(super) fn new(elements_size: usize, elements_count: usize) -> FixMemory {
        // サイズ、または、要素数0の場合作成されません。
        if elements_size == 0 || elements_count == 0 {
            error!("メモリ要素サイズ:{}, 要素数:{} でメモリプールの作成に失敗しました。", elements_size, elements_count);
            panic!()
        }

        // プール配列の配列長と初期要素数です。
        let pools_length = Self::INIT_POOLS_LENGTH;
        let pools_count = 1usize;

        // プール配列を作成します。
        let (pools, pools_layout) = Self::alloc_pools(pools_length);

        // プールのメモリレイアウトを作成します。
        let pool_size = size_of::<Pool>();
        let pool_align = pool_size.next_power_of_two();
        let pool_layout = unsafe { Layout::from_size_align_unchecked(pool_size, pool_align) };

        // 初期プールを作成します。
        let alloc_pool = Self::new_pool(pool_layout, elements_size, elements_count);
        if alloc_pool == null_mut() {

            // 進行不可能です。
            // エラーログを残して、異常終了します。
            error!("メモリ確保に失敗しました。");
            panic!();
        }
        unsafe { *pools = alloc_pool };

        FixMemory { 
            elements_size, 
            elements_count, 
            pools_length, 
            pools_count, 
            pools_layout, 
            pool_layout, 
            pools, 
            alloc_pool
        }
    }

    /// メモリを確保します。
    /// 
    /// # 戻り値
    /// 
    /// 確保したメモリへのポインタ、または、ヌルポインタです。
    /// 
    pub(super) fn alloc(&mut self) -> *mut u8 {
        // メモリ確保用プールが空なので、プールを確保します。
        if unsafe { &*self.alloc_pool }.is_empty() {
            self.add_pool();
        }

        // メモリ確保用プールからメモリを確保します。
        unsafe { &mut *self.alloc_pool }.alloc()
    }

    /// メモリを解放します。
    /// 
    /// # 引数
    /// 
    /// * pointer - 解放するメモリへのポインタです。
    /// 
    /// # 戻り値
    /// 
    /// 解放に成功したかの論理値です。
    /// 
    pub(super) fn dealloc(&mut self, pointer: *mut u8) -> bool {
        // 対応するプールの位置を探索します。
        let index = if let Some(index) = self.search_pool(pointer) {
            index
        } else {
            return false;
        };

        // 確保したメモリでメモリを解放します。
        unsafe { &mut **self.pools.add(index) }.dealloc(pointer);

        // 対象プールのすべての要素が未使用、かつ、メモリ確保対象でない場合削除します。
        if unsafe { &mut **self.pools.add(index) }.is_full() && unsafe { *self.pools.add(index) } != self.alloc_pool {
            self.remove_pool(index);
        }
        true
    }
    

    /// プールを追加して、メモリ確保用プールに設定します。
    fn add_pool(&mut self) {
        if self.pools_count == self.pools_length {
            self.expand_pools();
        }

        self.alloc_pool = Self::new_pool(self.pool_layout, self.elements_size, self.elements_count);

        // 挿入位置を取得します。
        let index = Self::insert_index(self.alloc_pool, self.pools, self.pools_count);

        // 挿入位置から後ろを移動します。
        for i in (index..self.pools_count).rev() {
            unsafe { (*self.pools.add(i + 1)) = *self.pools.add(i) };
        }

        // 挿入します。
        unsafe { (*self.pools.add(index)) = self.alloc_pool };
        self.pools_count += 1;
    } 

    /// プールを削除します。
    /// 
    /// # 引数
    /// 
    /// * index - 削除するプールの位置です。
    /// 
    fn remove_pool(&mut self, index: usize) {
        Self::drop_pool(unsafe { *self.pools.add(index) }, self.pool_layout);

        // 削除位置から後ろを詰めます。
        for i in index..self.pools_count {
            unsafe { *self.pools.add(i) = *self.pools.add(i + 1) }; 
        }
        
        self.pools_count -= 1;
    }

    /// 挿入位置を探査します。
    /// 
    /// # 引数
    /// 
    /// * pool - 挿入する対象です。
    /// * pools - 挿入対象の配列です。
    /// * count - 要素数です。
    /// 
    /// # 戻り値
    /// 
    /// 挿入位置を返します。
    /// 
    fn insert_index(pool: *mut Pool, pools: *mut *mut Pool, count: usize) -> usize {
        for i in 0..count {
            if unsafe { &*pool }.min_address() < unsafe { &**pools.add(i) }.min_address() {
                return i;
            }
        }
        count
    }

    /// メモリプールを確保します。
    /// 
    /// # 引数
    /// 
    /// * layout - プールのメモリレイアウトです。
    /// * size - プールが管理する要素のサイズです。
    /// * count - プールが管理する要素数です。
    /// 
    /// # 戻り値
    /// 
    /// 確保したメモリプールです。
    /// 
    /// # 異常終了
    /// 
    /// ヒープメモリの確保に失敗した場合異常終了します。
    /// メモリプールの作成に失敗した場合異常終了します。
    /// 
    fn new_pool(layout: Layout, size: usize, count: usize) -> *mut Pool {
        let pool: *mut Pool = unsafe { transmute(OSMemory::alloc_zeroed(layout)) };
        if pool == null_mut() {

            error!("メモリ要求サイズ:{}, 整列長:{} でメモリの確保に失敗しました。", layout.size(), layout.align());
            panic!();
        }

        unsafe { *pool = Pool::new(size, count) };
        
        pool
    }

    /// メモリプールを解放します。
    /// 
    /// # 引数
    /// 
    /// * pool - 解放するメモリプールのポインタです。
    /// * layout - ポインタのメモリレイアウトです。
    /// 
    fn drop_pool(pool: *mut Pool, layout: Layout) {
        unsafe { drop_in_place(pool) };
        OSMemory::dealloc(unsafe { transmute(pool) }, layout);
    }

    /// プール配列を拡張します。 
    fn expand_pools(&mut self) {
        // 拡張した新配列を作成します。
        let length = self.pools_length * Self::EXPANSION_MULTIPLY;
        let (pools, layout) = Self::alloc_pools(length);

        // 内容を移動させます。
        for i in 0..self.pools_count {
            unsafe { *pools.add(i) = *self.pools.add(i) };
        }

        // 旧配列を解放します。
        Self::dealloc_pools(self.pools, self.pools_layout);

        // 新しいデータで上書きします。
        self.pools = pools;
        self.pools_length = length;
        self.pools_layout = layout;
    }

    /// プール配列を作成します。
    /// 
    /// # 引数
    /// 
    /// * length - 配列長です。
    /// 
    /// # 戻り値
    /// 
    /// 成功した際は配列ポインタとメモリレイアウトが返ります。
    fn alloc_pools(length: usize) -> (*mut *mut Pool, Layout) {
        // 配列のサイズと整列長です。
        let size = size_of::<*mut Pool>() * length;
        let align = size.next_power_of_two();
        
        // 配列を確保します。
        let layout = unsafe { Layout::from_size_align_unchecked(size, align) };
        let buffer = unsafe { transmute(OSMemory::alloc(layout)) };
        if buffer == null_mut() {
            error!("メモリ要求サイズ:{}, 整列長:{} でメモリの確保に失敗しました。", layout.size(), layout.align());
            panic!();
        }

        (buffer, layout)
    }

    /// プール配列を解体します。
    /// 
    /// # 引数
    /// 
    /// * pools - プール配列ポインタです。
    /// * layout - プール配列のメモリレイアウトです。
    /// 
    fn dealloc_pools(pools: *mut *mut Pool, layout: Layout) {
        OSMemory::dealloc(unsafe { transmute(pools) }, layout);
    }

    /// ポインタから管理プールの位置を取得します。
    /// 
    /// # 引数
    /// 
    /// * pointer - 対象のポインタです。
    /// 
    /// # 戻り値
    /// 
    /// メモリプールの添え字、または、Noneです。
    /// 
    fn search_pool(&self, pointer: *mut u8) -> Option<usize> {
        let mut min = 0usize;
        let mut max = self.pools_count - 1usize;
        let mut pivot = (max as f32 * 0.5f32).floor() as usize;
        loop {
            if unsafe { &**self.pools.add(pivot) }.is_managed(pointer) {
                break Some (pivot);
            }
            if min >= max {
                break None;
            } else if (pointer as usize) < unsafe { &**self.pools.add(pivot) }.min_address() {
                max = pivot - 1usize;
            } else {
                min = pivot + 1usize;
            }
            pivot = ((max - min) as f32 * 0.5f32).floor() as usize + min;
        }
    }
}
impl Drop for FixMemory {
    /// 固定長メモリを解体します。
    fn drop(&mut self) { 
        for i in 0..self.pools_count {
            Self::drop_pool(unsafe { *self.pools.add(i) }, self.pool_layout);
        }
        Self::dealloc_pools(self.pools, self.pool_layout);
    }
}