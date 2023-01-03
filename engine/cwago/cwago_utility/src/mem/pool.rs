// -------------------------
//
// Cwago.
//
// cwago/cwago_utility/src/mem/pool.rs
// (C) 2023 CwagoCommunity.
//
//! メモリプールを提供します。
// =========================

use std::{
    alloc::Layout,
    mem::{
        size_of, 
        transmute
    },
    ptr::null_mut
};
use log::error;
use super::os::OSMemory;

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE_MAX: usize = 256;
    const COUNT_MAX: usize = 256;

    #[test]
    fn test_pool() {

        std::env::set_var("RUST_LOG", "error");
        env_logger::init();

        // サイズが1~256までで作成可能かテストします。
        for size in 1..SIZE_MAX {
            // 要素数が1~256までで作成可能かテストします。
            for count in 1..COUNT_MAX {
                test_pool_one(size, count)
            }
        }
    }
    fn test_pool_one(size: usize, count: usize) {
        // 作成します。
        let mut  pool = Pool::new(size, count);

        // 使いまわしが可能かテストします。
        for _lap in 0..3usize {
            
            let mut ptrs = [null_mut::<u8>(); COUNT_MAX];

            // メモリが要素数確保可能かテストします。
            for i in 0..count {
                ptrs[i] = pool.alloc();
                assert_ne!(ptrs[i], null_mut(), "{}回目のメモリ確保で失敗しました。", i);
            }

            // 使用可能な要素が存在しないかテストします。
            assert!(pool.is_empty(), "要素分メモリを確保したプールに非使用中の要素が存在します。");

            // メモリ範囲を識別できるかテストします。
            let ptr_min = *ptrs
                .iter()
                .filter(|&p| !p.is_null())
                .min()
                .expect("最小値が見つかりませんでした。");
            let ptr_max = *ptrs
                .iter()
                .filter(|&p| !p.is_null())
                .max()
                .expect("最大値が見つかりませんでした。");
            let ptr_less = unsafe { ptr_min.offset(-1) };
            let ptr_over = unsafe { ptr_max.offset(1) };
            assert!(pool.is_managed(ptr_min), "最小アドレスが管理範囲からはじかれました。");
            assert!(pool.is_managed(ptr_max), "最大アドレスが管理範囲からはじかれました。");
            assert!(!pool.is_managed(ptr_less), "未満アドレスが管理範囲に含まれました。");
            assert!(!pool.is_managed(ptr_over), "超過アドレスが管理範囲に含まれました。");

            // 確保したメモリに重複が無いかテストします。
            for i in 0..count {
                unsafe { *ptrs[i] = i as u8 };
            }
            for i in 0..count {
                assert_eq!(unsafe{ *ptrs[i] }, i as u8, "{}回目に確保したメモリに設定されていた値は{}でした。", i, unsafe{ *ptrs[i] });
            }

            // メモリを要素数解放可能かテストします。
            for i in 0..count {
                assert!(pool.dealloc(ptrs[i]), "{}回目に確保したメモリを解放できませんでした。", i);
            }

            // 使用中の要素が存在しないかテストします。
            assert!(pool.is_full(), "すべて解放されたプールに使用中の要素が存在します。");
        }
    }
}

/// メモリ領域を複数の要素として管理します。
#[derive(Debug)]
pub(super) struct Pool {
    all_count: usize,   // 管理対象の要素数です。
    free_count: usize,  // 現在確保している要素数です。
    layout: Layout,     // メモリ領域のレイアウトです。
    buffer: *mut u8,    // メモリ領域です。
    top: *mut *mut u8,  // 要素の単方向連結リストの先頭です。
    min_address: usize, // 管理するアドレスの最小値です。
    max_address: usize, // 管理するアドレスの最大値です。
}
impl Pool {

    const PTR_SIZE: usize = size_of::<*mut u8>(); // ポインタのサイズです。

    /// プールを作成します。
    ///
    /// # 引数
    /// 
    /// * size - 要素のサイズです。(ポインタサイズ以上に矯正されます。)
    /// * count - 要素数です。
    /// 
    /// # 戻り値
    /// 
    /// インスタンスが返ります。
    pub(super) fn new(size: usize, count: usize) -> Pool {
        // サイズ、または、要素数0の場合作成されません。
        if size == 0 || count == 0 {
            error!("メモリ要素サイズ:{}, 要素数:{} でメモリプールの作成に失敗しました。", size, count);
            panic!()
        }

        // 1要素のサイズと整列長です。
        let size = if size < Self::PTR_SIZE { Self::PTR_SIZE } else { size };
        let align = size.next_power_of_two();
        
        // 領域のサイズと整列長です。
        let buf_size = align * count;
        let buf_align = buf_size.next_power_of_two();
        
        // 領域を確保します。
        let layout = unsafe { Layout::from_size_align_unchecked(buf_size, buf_align) };
        let buffer = OSMemory::alloc(layout);
        if buffer == null_mut() {
            error!("メモリの確保に失敗しました。");
            panic!()        
        }
        
        // 連結リストを作成します。
        // 
        //    buffer [ptr][ptr][ptr]...
        //            | ^  | ^  | ^
        // null_mut <-' '--' '--' '-- top
        //
        let mut top = null_mut();
        for i in 0..count {
            let lp = unsafe { buffer.add(i * align) }; 
            let lpp = unsafe { transmute::<*mut u8, *mut *mut u8>(lp) };
            let rp = unsafe { transmute::<*mut *mut u8, *mut u8>(top) };
            unsafe { *lpp = rp };
            top = lpp; 
        }

        // 範囲のアドレスを計算します。
        let min_address = unsafe { buffer.add(0) } as usize;
        let max_address = unsafe { buffer.add(align * (count - 1)) } as usize;

        Pool{ 
            all_count: count, 
            free_count: count, 
            layout, 
            buffer, 
            top, 
            min_address, 
            max_address 
        }
    }

    /// 要素を確保します。
    /// 
    /// # 戻り値
    /// 
    /// 確保したメモリへのポインタ、または、ヌルポインタです。
    /// 
    pub(super) fn alloc(&mut self) -> *mut u8 {
        // 要素リストが空の場合、ヌルポインタを返します。
        if self.top == null_mut() {
            return null_mut();
        }
        // リストから要素を1つ取り出し返します。
        self.free_count -= 1;
        let ptr = self.top;
        unsafe { self.top = transmute::<*mut u8, *mut *mut u8>(*ptr) };
        unsafe { transmute::<*mut *mut u8, *mut u8>(ptr) }
    }

    /// 要素を解放します。
    /// 
    /// # 引数
    /// 
    /// * pointer - 解放するポインタです。
    /// 
    /// # 戻り値
    /// 
    /// このプールで解放された場合、真を返します。
    /// 
    pub(super) fn dealloc(&mut self, pointer: *mut u8) -> bool {
        // ポインタがプールの管理外の場合、偽を返します。
        if !self.is_managed(pointer) {
            return false;
        }
        // リストに要素を挿入して、真を返します。
        self.free_count += 1;
        let ptr = unsafe { transmute::<*mut u8, *mut *mut u8>(pointer) };
        unsafe { *ptr = transmute::<*mut *mut u8, *mut u8>(self.top) };
        self.top = ptr;
        true
    }

    /// 管理範囲に収まるか判定します。
    /// 
    /// # 引数
    /// 
    /// * pointer - 判定するポインタです。
    /// 
    /// # 戻り値
    /// 
    /// 範囲内の場合真を返します。
    /// 
    pub(super) fn is_managed(&self, pointer: *mut u8) -> bool {
        let adr = pointer as usize;
        self.min_address <= adr && adr <= self.max_address
    }

    /// このプールのすべての要素が非使用中か判定します。
    /// 
    /// # 戻り値
    /// 
    /// すべての要素が未使用の際、真を返します。
    /// 
    pub(super) fn is_full(&self) -> bool {
        self.all_count == self.free_count
    }

    /// このプールに使用中の要素が無いか判定します。
    /// 
    /// # 戻り値
    /// 
    /// 使用中の要素が無い際、真を返します。
    /// 
    pub(super) fn is_empty(&self) -> bool {
        0usize == self.free_count
    }

    /// 管理する最小アドレスを返します。
    /// 
    /// # 戻り値
    /// 
    /// 管理する最小アドレスです。
    /// 
    pub(super) fn min_address(&self) -> usize {
        self.min_address
    }
    
}
impl Drop for Pool {
    /// プールを解体します。
    fn drop(&mut self) {
        OSMemory::dealloc(self.buffer, self.layout);
    }
}