// -------------------------
//
// Cwago.
//
// cwago/cwago_memory/src/os.rs
// (C) 2023 CwagoCommunity.
//
//! OSメモリのシングルトンを提供します。
// =========================

use std::{
    alloc::{
        Layout, 
        System, 
        GlobalAlloc
    }, 
    sync::Once, ptr::null_mut
};

use cwago_utility::log::error;


// OSが提供するメモリのシングルトンです。
pub(super) struct OSMemory;
static mut SYSTEM: Option<System> = None;
static ONCE: Once = Once::new();
impl OSMemory {

    /// メモリを確保します。
    /// 
    /// # 引数
    /// 
    /// * layout - 確保するメモリレイアウトです。
    /// 
    /// # 戻り値
    /// 
    /// 確保したメモリです。
    /// 
    pub(super) fn alloc(layout: Layout) -> *mut u8 {
        // OSメモリを初期化します。
        ONCE.call_once(|| unsafe{
            SYSTEM = Some(System::default())
        });

        // メモリを確保します。
        let ptr = unsafe { SYSTEM.unwrap().alloc(layout) };
        if ptr == null_mut() {
            if layout.size() == 0 {
                error!("確保しようとしたメモリサイズが0でした。");
            } else if layout.align() == 0 {
                error!("確保しようとしたメモリサイズのアラインメントが0でした。");
            } else {
                error!("メモリ確保に失敗しました。");
            }
            panic!();
        }

        ptr
    }

    /// 0初期化したメモリを確保します。
    /// 
    /// # 引数
    /// 
    /// * layout - 確保するメモリレイアウトです。
    /// 
    /// # 戻り値
    /// 
    /// 確保したメモリです。
    /// 
    pub(super) fn alloc_zeroed(layout: Layout) -> *mut u8 {
        let ptr = Self::alloc(layout);

        // 0初期化します。
        for i in 0..layout.align() {
            unsafe { *ptr.add(i) = 0u8 };
        }

        ptr
    }

    /// メモリを解放します。
    /// 
    /// # 引数
    /// 
    /// * pointer - 解放するメモリです。
    /// * layout - 解放するメモリレイアウトです。
    /// 
    pub(super) fn dealloc(pointer: *mut u8, layout: Layout) {

        if let Some(system) = unsafe { &SYSTEM } {
            unsafe { system.dealloc(pointer, layout) };
        } else {
            error!("メモリが確保される前に解放しようとしました。");
        }
    }
}