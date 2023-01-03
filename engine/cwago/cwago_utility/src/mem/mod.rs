// -------------------------
//
// Cwago.
//
// cwago/cwago_utility/src/mem/mod.rs
// (C) 2023 CwagoCommunity.
//
//! メモリシステムを提供します。
// =========================

use std::{
    alloc::GlobalAlloc, 
    sync::Once
};

mod os;
mod pool;
mod fix;
mod dy;

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
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        ONCE.call_once(|| unsafe {
            DY_MEMORY = Some(dy::DyMemory::new());
        });
        
        unsafe { DY_MEMORY.as_mut() }.unwrap().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        ONCE.call_once(|| unsafe {
            DY_MEMORY = Some(dy::DyMemory::new());
        });
        
        unsafe { DY_MEMORY.as_mut() }.unwrap().dealloc(ptr, layout)
    }
}