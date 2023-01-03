// -------------------------
//
// Cwago.
//
// cwago/cwago_utility/src/mem/dy.rs
// (C) 2023 CwagoCommunity.
//
//! 可変長メモリを提供します。
// =========================

use std::{alloc::Layout, sync::Mutex};

use log::error;

use super::{
    fix::FixMemory, 
    os::OSMemory
};

#[cfg(test)]
mod tests {
    use std::ptr::null_mut;

    use super::*;

    const SIZE_MAX: usize = 512;
    const LENGTH_MAX: usize = 512;

    #[test]
    fn test_dy_memory() {

        std::env::set_var("RUST_LOG", "error");
        env_logger::init();

        let mut mem = DyMemory::new();

        // サイズが1~256までで作成可能かテストします。
        for size in 1..SIZE_MAX {

            let align = size.next_power_of_two();
            let layout = unsafe { Layout::from_size_align_unchecked(size, align) };

            for _lap in 0..3usize {

                let mut ptrs = [null_mut::<u8>(); LENGTH_MAX];
    
                // メモリが確保可能かテストします。
                for i in 0..LENGTH_MAX {
                    ptrs[i] = mem.alloc(layout);
                    assert_ne!(ptrs[i], null_mut(), "{}回目のメモリ確保で失敗しました。", i);
                }
    
                // 確保したメモリに重複が無いかテストします。
                for i in 0..LENGTH_MAX {
                    unsafe { *ptrs[i] = i as u8 };
                }
                for i in 0..LENGTH_MAX {
                    assert_eq!(unsafe{ *ptrs[i] }, i as u8, "{}回目に確保したメモリに設定されていた値は{}でした。", i, unsafe{ *ptrs[i] });
                }
    
                // メモリを要素数解放します。
                for i in 0..LENGTH_MAX {
                    mem.dealloc(ptrs[i], layout);
                }
            }
        }
    }
}

/// 可変長メモリを管理します。
#[derive(Debug)]
pub(super) struct DyMemory {
    memory16: Mutex<FixMemory>,
    memory32: Mutex<FixMemory>,
    memory64: Mutex<FixMemory>,
    memory128: Mutex<FixMemory>,
    memory256: Mutex<FixMemory>,
}
impl DyMemory {

    const COUNT16: usize = 32;  // memory16の1要素の要素数です。
    const COUNT32: usize = 32;  // memory32の1要素の要素数です。
    const COUNT64: usize = 32;  // memory64の1要素の要素数です。
    const COUNT128: usize = 16; // memory128の1要素の要素数です。
    const COUNT256: usize = 16; // memory256の1要素の要素数です。
    /// 作成します。
    /// 
    /// # 戻り値
    /// 
    /// DyMemoryのインスタンスです。
    /// 
    pub(super) fn new() -> DyMemory {
        DyMemory { 
            memory16: Mutex::new(FixMemory::new(16usize, Self::COUNT16)), 
            memory32: Mutex::new(FixMemory::new(32usize, Self::COUNT32)), 
            memory64: Mutex::new(FixMemory::new(64usize, Self::COUNT64)), 
            memory128: Mutex::new(FixMemory::new(128usize, Self::COUNT128)), 
            memory256: Mutex::new(FixMemory::new(256usize, Self::COUNT256)) 
        }
    }

    /// メモリを確保します。
    /// 
    /// # 引数
    /// 
    /// * layout - 確保するメモリのレイアウトです。
    /// 
    /// # 戻り値
    /// 
    /// 確保したメモリのポインタです。
    /// 
    pub(super) fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if layout.align() <= 16 {
            match self.memory16.lock() {
                Ok(mut mem) => mem.alloc(),
                Err(_) => {
                    error!("メモリ確保中に他スレッドが異常終了しました。");
                    panic!()
                },
            }
        } else if layout.align() <= 32 {
            match self.memory32.lock() {
                Ok(mut mem) => mem.alloc(),
                Err(_) => {
                    error!("メモリ確保中に他スレッドが異常終了しました。");
                    panic!()
                },
            }
        } else if layout.align() <= 64 {
            match self.memory64.lock() {
                Ok(mut mem) => mem.alloc(),
                Err(_) => {
                    error!("メモリ確保中に他スレッドが異常終了しました。");
                    panic!()
                },
            }
        } else if layout.align() <= 128 {
            match self.memory128.lock() {
                Ok(mut mem) => mem.alloc(),
                Err(_) => {
                    error!("メモリ確保中に他スレッドが異常終了しました。");
                    panic!()
                },
            }
        } else if layout.align() <= 256 {
            match self.memory256.lock() {
                Ok(mut mem) => mem.alloc(),
                Err(_) => {
                    error!("メモリ確保中に他スレッドが異常終了しました。");
                    panic!()
                },
            }
        } else {
            OSMemory::alloc(layout)
        }
    }

    /// メモリを解放します。
    /// 
    /// # 引数
    /// 
    /// * pointer - 解放するメモリのポインタです。
    /// * layout - 解放するメモリのレイアウトです。
    /// 
    pub(super) fn dealloc(&mut self, pointer: *mut u8, layout: Layout) {
        if layout.align() <= 16 {
            match self.memory16.lock() {
                Ok(mut mem) => if !mem.dealloc(pointer) {
                    OSMemory::dealloc(pointer, layout);
                },
                Err(_) => {
                    error!("メモリ解放中に他スレッドが異常終了しました。");
                    panic!()
                },
            }
            
        } else if layout.align() <= 32 {
            match self.memory32.lock() {
                Ok(mut mem) => if !mem.dealloc(pointer) {
                    OSMemory::dealloc(pointer, layout);
                },
                Err(_) => {
                    error!("メモリ解放中に他スレッドが異常終了しました。");
                    panic!()
                },
            }
        } else if layout.align() <= 64 {
            match self.memory64.lock() {
                Ok(mut mem) => if !mem.dealloc(pointer) {
                    OSMemory::dealloc(pointer, layout);
                },
                Err(_) => {
                    error!("メモリ解放中に他スレッドが異常終了しました。");
                    panic!()
                },
            }
        } else if layout.align() <= 128 {
            match self.memory128.lock() {
                Ok(mut mem) => if !mem.dealloc(pointer) {
                    OSMemory::dealloc(pointer, layout);
                },
                Err(_) => {
                    error!("メモリ解放中に他スレッドが異常終了しました。");
                    panic!()
                },
            }
        } else if layout.align() <= 256 {
            match self.memory256.lock() {
                Ok(mut mem) => if !mem.dealloc(pointer) {
                    OSMemory::dealloc(pointer, layout);
                },
                Err(_) => {
                    error!("メモリ解放中に他スレッドが異常終了しました。");
                    panic!()
                },
            }
        } else {
            OSMemory::dealloc(pointer, layout);
        }
    }
}