// -------------------------
//
// Cwago.
//
// cwago/cwago_memory/src/lib.rs
// (C) 2022 CwagoCommunity.
//
//! cwago_memoryライブラリのメインファイルです。
// =========================

use std::{
    alloc::GlobalAlloc, 
    sync::Once
};

use cwago_utility::log::error;

mod os;
mod pool;
mod fix;
mod dy;

#[cfg(test)]
mod tests {
    use std::{
        ptr::null_mut, 
        alloc::Layout
    };

    use super::*;

    const SIZE_MAX: usize = 512;
    const LENGTH_MAX: usize = 512;

    #[test]
    fn test_allocator() {

        std::env::set_var("RUST_LOG", "error");
        env_logger::init();

        let mem = Allocator::new();

        // サイズが1~256までで作成可能かテストします。
        for size in 1..SIZE_MAX {

            let align = size.next_power_of_two();
            let layout = unsafe { Layout::from_size_align_unchecked(size, align) };

            for _lap in 0..3usize {

                let mut ptrs = [null_mut::<u8>(); LENGTH_MAX];
    
                // メモリが確保可能かテストします。
                for i in 0..LENGTH_MAX {
                    ptrs[i] = unsafe{ mem.alloc(layout) };
                    assert_ne!(ptrs[i], null_mut(), "{}回目のメモリ確保で失敗しました。", i);
                }
    
                // 確保したメモリに重複が無いかテストします。
                for i in 0..LENGTH_MAX {
                    unsafe { *ptrs[i] = (i % 256) as u8 };
                }
                for i in 0..LENGTH_MAX {
                    assert_eq!(unsafe{ *ptrs[i] }, (i % 256) as u8, "{}回目に確保したメモリに設定されていた値は{}でした。", i, unsafe{ *ptrs[i] });
                }
    
                // メモリを要素数解放します。
                for i in 0..LENGTH_MAX {
                    unsafe{ mem.dealloc(ptrs[i], layout) };
                }
            }
        }
    }
}

/// メモリアロケータです。
#[derive(Debug, Clone, Copy)]
pub struct Allocator;
static mut DY_MEMORY: Option<dy::DyMemory> = None;
static ONCE: Once = Once::new();
impl Allocator {
    /// 作成します。
    /// 
    /// # 戻り値
    /// 
    /// Memoryの静的なインスタンスです。
    /// 
    pub const fn new() -> Allocator {
        Allocator {}
    }
}
unsafe impl Send for Allocator {}
unsafe impl Sync for Allocator {}
unsafe impl GlobalAlloc for Allocator {
    /// メモリを確保します。
    /// 
    /// # 引数
    /// 
    /// * `layout` -  確保するメモリのレイアウトです。
    /// 
    /// # 戻り値
    /// 
    /// 確保したメモリのポインタです。
    /// 
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        // 初期化します。
        ONCE.call_once(|| unsafe {
            DY_MEMORY = Some(dy::DyMemory::new());
        });
        
        // メモリを確保します。
        match unsafe { DY_MEMORY.as_mut() } {
            Some(mem) => mem,
            None => {
                error!("メモリシステムの初期化に失敗しました。");
                panic!()
            },
        }.alloc(layout)
    }

    /// メモリを解放します。
    /// 
    /// # 引数
    /// 
    /// * `ptr` - 解放するメモリのポインタです。
    /// * `layout` - 解放するメモリのレイアウトです。
    /// 
    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        // 初期化します。
        ONCE.call_once(|| unsafe {
            DY_MEMORY = Some(dy::DyMemory::new());
        });
        
        // メモリを解放します。
        match unsafe { DY_MEMORY.as_mut() }{
            Some(mem) => mem,
            None => {
                error!("メモリシステムの初期化に失敗しました。");
                panic!()
            },
        }.dealloc(ptr, layout)
    }
}