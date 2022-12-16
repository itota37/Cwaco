// -------------------------
//
// Cwago.
//
// cwago/cwago_utility/src/mem.rs
// (C) 2022 CwagoCommunity.
//
//! メモリシステムを提供します。
// =========================

use std::{
    alloc::{
        alloc,
        dealloc,
        Layout
    },
    mem::{
        size_of, 
        transmute
    },
    ptr::null_mut
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool() {
        // サイズ0でプールの作成に失敗するかテストします。
        assert!(Pool::new(0, 1).is_none(), "サイズ0で失敗しません。");

        // 要素数0でプールの作成に失敗するかテストします。
        assert!(Pool::new(1, 0).is_none(), "要素数0で失敗しません。");

        // サイズが1~256までで作成可能かテストします。
        for size in 1..256usize {
            // 要素数が1~256までで作成可能かテストします。
            for count in 1..256usize {
                // 作成に成功するかテストします。
                let pool = if let Some(pool) = Pool::new(size, count) {
                    pool
                } else {
                    panic!("サイズ:{} 要素数:{} で作成に失敗しました。", size, count)
                };
                
                // 要素サイズがポインタサイズ以上に矯正されているかテストします。
                assert!(pool.layout.size() >= size_of::<*mut u8>(), "サイズ:{} が ポインタサイズ{} に矯正されていません。", pool.layout.size(), size_of::<*mut u8>());
            }
        }
    }
}

/// メモリ領域を複数の要素として管理します。
#[derive(Debug)]
struct Pool {
    layout: Layout,    // メモリ領域のレイアウトです。
    buffer: *mut u8,   // メモリ領域です。
    top: *mut *mut u8, // 要素の単方向連結リストの先頭です。
}
impl Pool {
    /// プールを作成します。
    ///
    /// # 引数
    /// 
    /// * size - 要素のサイズです。(ポインタサイズ以上に矯正されます。)
    /// * count - 要素数です。
    /// 
    /// # 戻り値
    /// 
    /// 成功した際はインスタンス、失敗した際はNoneが返ります。
    fn new(size: usize, count: usize) -> Option<Pool> {
        // サイズ、または、要素数0の場合作成されません。
        if size == 0 || count == 0 {
            return None;
        }

        // 1要素のサイズと整列長です。
        const PTR_SIZE: usize = size_of::<*mut u8>();
        let size = if size < PTR_SIZE { PTR_SIZE } else { size };
        let align = size.next_power_of_two();
        
        // 領域のサイズと整列長です。
        let buf_size = align * count;
        let buf_align = buf_size.next_power_of_two();
        
        // 領域を確保します。
        let layout = unsafe { Layout::from_size_align_unchecked(buf_size, buf_align) };
        let buffer = unsafe { alloc(layout) };
        if buffer == null_mut() {
            return None;        
        }
        
        // 連結リストを作成します。
        // 
        //     buffer [ptr][ptr][ptr]...
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

        Some(Pool{ layout, buffer, top })
    }
}
impl Drop for Pool {
    /// プールを解体します。
    fn drop(&mut self) {
        unsafe { dealloc(self.buffer, self.layout) };
    }
}