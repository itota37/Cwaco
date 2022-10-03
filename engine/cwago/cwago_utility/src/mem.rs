// --------------------
//
// Cwago.
//
// cwago/cwago_utility/src/mem.rs
// (C) 2022 Taichi Ito.
// ====================

//! メモリ管理機能を提供します。

use std::mem::size_of;
use std::alloc::{alloc, dealloc, Layout, GlobalAlloc};
use std::ptr::{null_mut, drop_in_place};
use std::sync::{Arc, Mutex, Once};

/// グローバルメモリです。
pub struct Memory {}
static mut GLOBAL_MEMORY: Option<AtomDynMemory> = None;
static GLOBAL_MEMORY_ONCE: Once = Once::new();
impl Memory {
    
    /// メモリを作成します。
    pub const fn new() -> Self {Memory {}}
}
unsafe fn imit_memory() {

    GLOBAL_MEMORY_ONCE.call_once(||{
        GLOBAL_MEMORY = Some(AtomDynMemory::new());
    });
}
unsafe impl GlobalAlloc for Memory {
    
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        
        imit_memory();
        
        GLOBAL_MEMORY
        .as_mut()
        .expect("重大なエラーが発生しました。Memory/alloc/0")
        .alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        
        imit_memory();
        
        GLOBAL_MEMORY
        .as_mut()
        .expect("重大なエラーが発生しました。Memory/dealloc/0")
        .dealloc(ptr, layout);
    }
}

/// スレッドセーフ可変長メモリです。
pub struct AtomDynMemory {

    memory16: AtomFixMemory,  // 16バイトメモリです。
    memory32: AtomFixMemory,  // 32バイトメモリです。
    memory64: AtomFixMemory,  // 64バイトメモリです。
    memory128: AtomFixMemory, // 128バイトメモリです。
    memory256: AtomFixMemory, // 256バイトメモリです。
}
impl AtomDynMemory {

    const MEMORY_16_SIZE: usize = 16;
    const MEMORY_32_SIZE: usize = 32;
    const MEMORY_64_SIZE: usize = 64;
    const MEMORY_128_SIZE: usize = 128;
    const MEMORY_256_SIZE: usize = 256;
    const MEMORY_16_LEN: usize = 32;
    const MEMORY_32_LEN: usize = 32;
    const MEMORY_64_LEN: usize = 32;
    const MEMORY_128_LEN: usize = 16;
    const MEMORY_256_LEN: usize = 16;
    
    /// メモリを作成します。
    pub fn new() -> Self {

        Self { 
            memory16: AtomFixMemory::new(Self::MEMORY_16_SIZE, Self::MEMORY_16_LEN),
            memory32: AtomFixMemory::new(Self::MEMORY_32_SIZE, Self::MEMORY_32_LEN), 
            memory64: AtomFixMemory::new(Self::MEMORY_64_SIZE, Self::MEMORY_64_LEN), 
            memory128: AtomFixMemory::new(Self::MEMORY_128_SIZE, Self::MEMORY_128_LEN), 
            memory256: AtomFixMemory::new(Self::MEMORY_256_SIZE, Self::MEMORY_256_LEN) 
        }
    }

    /// メモリを確保します。
    /// 
    /// # Arguments
    /// 
    /// * `layout` - 確保するメモリのメモリレイアウトです。
    /// 
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {

        if layout.align() <= Self::MEMORY_16_SIZE {
            self.memory16.alloc()
        } else if layout.align() <= Self::MEMORY_32_SIZE {
            self.memory32.alloc()
        } else if layout.align() <= Self::MEMORY_64_SIZE {
            self.memory64.alloc()
        } else if layout.align() <= Self::MEMORY_128_SIZE {
            self.memory128.alloc()
        } else if layout.align() <= Self::MEMORY_256_SIZE {
            self.memory256.alloc()
        } else {
            alloc(layout)
        }
    }

    /// メモリを解放します。
    /// 
    /// # Arguments
    /// 
    /// * `ptr`    - 解放するメモリポインタです。
    /// * `layout` - 解放するメモリのメモリレイアウトです。
    /// 
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {

        if layout.align() <= Self::MEMORY_16_SIZE {
            self.memory16.dealloc(ptr);
        } else if layout.align() <= Self::MEMORY_32_SIZE {
            self.memory32.dealloc(ptr);
        } else if layout.align() <= Self::MEMORY_64_SIZE {
            self.memory64.dealloc(ptr);
        } else if layout.align() <= Self::MEMORY_128_SIZE {
            self.memory128.dealloc(ptr);
        } else if layout.align() <= Self::MEMORY_256_SIZE {
            self.memory256.dealloc(ptr);
        } else {
            dealloc(ptr, layout);
        }
    }

}

/// スレッドセーフ固定長メモリです。
pub struct AtomFixMemory {
    size: usize,                   // データサイズです。size >= address_size
    pool_max_length: usize,        // 1つのプールが管理するデータの数です。
    memory: Arc<Mutex<FixMemory>>, // 固定長メモリです。
}
impl AtomFixMemory {
    
    /// メモリを作成します。
    /// 
    /// # Arguments
    /// 
    /// * `size`  - 固定長データ領域のサイスです。
    /// * `count` - 1つのプールが管理するデータの数です。
    /// 
    pub fn new(size: usize, count: usize) -> Self {

        Self {
            size, 
            pool_max_length: count,  
            memory: Arc::new(Mutex::new(FixMemory::new(size, count))) 
        }
    }

    /// メモリを確保します。
    pub unsafe fn alloc(&mut self) -> *mut u8 {

        match self.memory.lock() {

            Ok(mut mem) => mem.alloc(),
            Err(_) => null_mut(),
        }
    }

    /// メモリを解放します。
    /// 
    /// # Arguments
    /// 
    /// * `ptr`  - 解放するポインタです。
    /// 
    pub unsafe fn dealloc(&mut self, ptr: *mut u8) {

        if let Ok(mut mem) = self.memory.lock() {

            mem.dealloc(ptr);
        }
    }

    /// 確保するメモリ領域のサイズを取得します。
    /// 必ず、size > size_of::<*mut u8>() です。
    pub fn size(&self) -> usize {

        self.size
    }

    /// 1つのプールが管理するメモリ領域の数を取得します。
    pub fn pool_max_len(&self) -> usize {

        self.pool_max_length
    }
}
unsafe impl Send for AtomFixMemory {}
unsafe impl Sync for AtomFixMemory {}

/// 可変長メモリです。
pub struct DynMemory {

    memory16: FixMemory,  // 16バイトメモリです。
    memory32: FixMemory,  // 32バイトメモリです。
    memory64: FixMemory,  // 64バイトメモリです。
    memory128: FixMemory, // 128バイトメモリです。
    memory256: FixMemory, // 256バイトメモリです。
}
impl DynMemory {

    const MEMORY_16_SIZE: usize = 16;
    const MEMORY_32_SIZE: usize = 32;
    const MEMORY_64_SIZE: usize = 64;
    const MEMORY_128_SIZE: usize = 128;
    const MEMORY_256_SIZE: usize = 256;
    const MEMORY_16_LEN: usize = 32;
    const MEMORY_32_LEN: usize = 32;
    const MEMORY_64_LEN: usize = 32;
    const MEMORY_128_LEN: usize = 16;
    const MEMORY_256_LEN: usize = 16;
    
    /// メモリを作成します。
    pub fn new() -> Self {

        Self { 
            memory16: FixMemory::new(Self::MEMORY_16_SIZE, Self::MEMORY_16_LEN),
            memory32: FixMemory::new(Self::MEMORY_32_SIZE, Self::MEMORY_32_LEN), 
            memory64: FixMemory::new(Self::MEMORY_64_SIZE, Self::MEMORY_64_LEN), 
            memory128: FixMemory::new(Self::MEMORY_128_SIZE, Self::MEMORY_128_LEN), 
            memory256: FixMemory::new(Self::MEMORY_256_SIZE, Self::MEMORY_256_LEN) 
        }
    }

    /// メモリを確保します。
    /// 
    /// # Arguments
    /// 
    /// * `layout` - 確保するメモリのメモリレイアウトです。
    /// 
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {

        if layout.align() <= Self::MEMORY_16_SIZE {
            self.memory16.alloc()
        } else if layout.align() <= Self::MEMORY_32_SIZE {
            self.memory32.alloc()
        } else if layout.align() <= Self::MEMORY_64_SIZE {
            self.memory64.alloc()
        } else if layout.align() <= Self::MEMORY_128_SIZE {
            self.memory128.alloc()
        } else if layout.align() <= Self::MEMORY_256_SIZE {
            self.memory256.alloc()
        } else {
            alloc(layout)
        }
    }

    /// メモリを解放します。
    /// 
    /// # Arguments
    /// 
    /// * `ptr`    - 解放するメモリポインタです。
    /// * `layout` - 解放するメモリのメモリレイアウトです。
    /// 
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {

        if layout.align() <= Self::MEMORY_16_SIZE {
            self.memory16.dealloc(ptr);
        } else if layout.align() <= Self::MEMORY_32_SIZE {
            self.memory32.dealloc(ptr);
        } else if layout.align() <= Self::MEMORY_64_SIZE {
            self.memory64.dealloc(ptr);
        } else if layout.align() <= Self::MEMORY_128_SIZE {
            self.memory128.dealloc(ptr);
        } else if layout.align() <= Self::MEMORY_256_SIZE {
            self.memory256.dealloc(ptr);
        } else {
            dealloc(ptr, layout);
        }
    }

}

/// 固定長メモリです。
pub struct FixMemory {

    size: usize,            // データサイズです。size >= address_size
    pool_max_length: usize, // 1つのプールが管理するデータの数です。
    pool_layout: Layout,    // プールポインタのメモリレイアウトです。
    active_pool: *mut Pool, // alloc対象のプールポインタです。
    list: PoolList,         // プールのリストです。
}
impl FixMemory {

    /// メモリを作成します。
    /// 
    /// # Arguments
    /// 
    /// * `size`  - 固定長データ領域のサイスです。
    /// * `count` - 1つのプールが管理するデータの数です。
    /// 
    pub fn new(size: usize, count: usize) -> Self {

        unsafe {
            let pool_layout = Layout::new::<Pool>();
            let active_pool = alloc(pool_layout) as *mut Pool;
            (*active_pool) = Pool::new(size, count);
            let mut list = PoolList::new();
            list.insert(active_pool);
            Self { size, pool_max_length: count, pool_layout, active_pool, list}
        }
    }

    /// メモリを確保します。
    pub unsafe fn alloc(&mut self) -> *mut u8 {

        // 現在のプールが空の場合、プールを確保します。
        if (*self.active_pool).is_empty() {
            self.active_pool = self.add_pool();
        }
        // メモリを確保します。
        (*self.active_pool).alloc()
    }

    /// メモリを解放します。
    /// 
    /// # Arguments
    /// 
    /// * `ptr`  - 解放するポインタです。
    /// 
    pub unsafe fn dealloc(&mut self, ptr: *mut u8) {

        // メモリを解放します。
        let pool = self.list.search(ptr as usize);
        (*pool).dealloc(ptr);
        // メモリを解放したプールが確保用でない、かつ、フルの場合、解放します。
        if pool != self.active_pool && (*pool).is_full() {
            self.remove_pool(pool);
        }
    }

    /// 確保するメモリ領域のサイズを取得します。
    /// 必ず、size > size_of::<*mut u8>() です。
    pub fn size(&self) -> usize {

        self.size
    }

    /// 1つのプールが管理するメモリ領域の数を取得します。
    pub fn pool_max_len(&self) -> usize {

        self.pool_max_length
    }
    
    // プールを追加し、プールポインタを返します。
    unsafe fn add_pool(&mut self) -> *mut Pool {

        let pool = alloc(self.pool_layout) as *mut Pool;
        (*pool) = Pool::new(self.size, self.pool_max_length);
        self.list.insert(pool);
        pool
    }

    // プールを削除します。
    // 
    // # Arguments
    // 
    // * `pool`  - 削除するプールです。
    // 
    unsafe fn remove_pool(&mut self, pool: *mut Pool) {

        self.list.remove(pool);
        drop_in_place(pool);
        dealloc(pool as *mut u8, self.pool_layout);
    }
    
}
impl Drop for FixMemory {

    /// 終了時に呼ばれます。
    fn drop(&mut self) {

        for ptr in &self.list {

            unsafe {
                drop_in_place(ptr);
                dealloc(ptr as *mut u8, self.pool_layout);
            }
        }
    }
}

// 順序付きプールポインタリストです。
struct PoolList {

    length: usize,
    list_layout: Layout,
    list: *mut *mut Pool,
}
impl PoolList {
    
    // リストを作成します。
    fn new() -> PoolList {

        PoolList { 
            length: 0, 
            list_layout: Layout::new::<*mut *mut Pool>(), 
            list: null_mut() 
        }
    }

    // リストに追加します。
    //
    // # Arguments
    //
    // * `ptr` - 追加するプールポインタです。
    //
    unsafe fn insert(&mut self, ptr: *mut Pool) {
        
        // 挿入位置を取得します。
        let idx = self.insert_index_of(ptr);
        // 新しいリストを作成します。
        let (list, layout) = PoolList::new_list(self.length + 1);
        // 新しいリストに要素をコピーします。
        PoolList::copy(self.list, 0, list, 0, idx);
        PoolList::copy(self.list, idx, list, idx + 1, self.length - idx);
        // 挿入します。
        (*list.offset(idx as isize)) = ptr;
        // 古いリストを解放します。
        drop_in_place(self.list);
        dealloc(self.list as *mut u8, self.list_layout);
        // リストを入れ替えます。
        self.list = list;
        self.list_layout = layout;
        self.length += 1;
    }

    // リストから削除します。
    //
    // # Arguments
    //
    // * `ptr` - 削除するプールポインタです。
    //
    unsafe fn remove(&mut self, ptr: *mut Pool) {

        // 削除位置を取得します。
        let adr = (*ptr).buffer as usize;
        let idx = self.index_of(adr, |l,r| ((*l).buffer as usize) == r);
        // 新しいリストを作成します。
        let (list, layout) = PoolList::new_list(self.length - 1);
        // 新しいリストに要素をコピーします。
        PoolList::copy(self.list, 0, list, 0, idx);
        PoolList::copy(self.list, idx + 1, list, idx, self.length - idx);
        // 古いリストを解放します。
        drop_in_place(self.list);
        dealloc(self.list as *mut u8, self.list_layout);
        // リストを入れ替えます。
        self.list = list;
        self.list_layout = layout;
        self.length -= 1;
    }

    // アドレスから管理しているプールポインタを探します。
    //
    // # Arguments
    //
    // * `adr` - プールで管理されているアドレスです。
    //
    unsafe fn search(&self, adr: usize) -> *mut Pool {

        // プールの位置を取得します。
        let equal = |l: *mut Pool, r: usize| {

            // プールの管轄領域内か判定します。
            let begin = (*l).buffer as usize;
            let end = (*l).buffer.offset(((*l).size() * (*l).max_len()) as isize) as usize;
            (begin <= r) && (r < end)
        };
        let idx = self.index_of(adr, equal);
        // プールを取得します。
        *self.list.offset(idx as isize)
    }

    // アドレスから位置を所得します。
    //
    // # Arguments
    //
    // * `adr`   - 比較するアドレスです。
    // * `equal` - 等しいか場合、真を返す関数です。
    //
    unsafe fn index_of(&self, adr: usize, equal: impl Fn(*mut Pool, usize) -> bool) -> usize {

        // 二分探査で探索します。
        let mut l = 0;
        let mut r = self.length - 1;
        while l <= r {

            let center = (l + r) / 2;
            let target = *self.list.offset(center as isize);

            if equal(target, adr) {
                return center;
            } else if ((*target).buffer as usize) < adr {
                l = center + 1;
            } else {
                r = center - 1;
            }
        }
        self.length
    }

    // プールポインタから挿入位置位置を決定します。
    //
    // # Arguments
    //
    // * `ptr` - プールポインタです。
    //
    unsafe fn insert_index_of(&self, ptr: *mut Pool) -> usize {

        let mut idx = 0;
        while idx < self.length {

            if (*ptr).buffer < (**self.list.offset(idx as isize)).buffer {

                break;
            }
            idx += 1;
        }
        idx
    }

    // リストを作成します。
    //
    // # Arguments
    //
    // * `from`       - コピー元です。
    // * `from_begin` - コピー元開始位置です。
    // * `to`         - コピー先です。
    // * `to_begin`   - コピー先開始位置です。
    // * `length`     - コピー要素数です。
    //
    unsafe fn copy(from: *mut *mut Pool, from_begin: usize, to: *mut *mut Pool, to_begin: usize, length: usize) {
        
        let mut f = from_begin as isize;
        let mut t = to_begin as isize;

        for _ in 0..length {
            (*to.offset(t)) = *from.offset(f);
            f += 1;
            t += 1;
        }
    }

    // リストを作成します。
    //
    // # Arguments
    //
    // * `size` - リストサイズです。
    //
    unsafe fn new_list(size: usize) -> (*mut *mut Pool, Layout) {

        let layout = PoolList::new_layout(size);
        let list = alloc(layout) as *mut *mut Pool;
        (list, layout)
    }

    // メモリレイアウトを作成します。
    //
    // # Arguments
    //
    // * `size` - リストサイズです。
    //
    unsafe fn new_layout(size: usize) -> Layout {

        let list_size = size_of::<*mut Pool>() * size;
        let list_align = list_size.next_power_of_two();
        Layout::from_size_align_unchecked(list_size, list_align)
    }
}
impl Drop for PoolList {

    // 終了時に呼ばれます。
    fn drop(&mut self) {

        unsafe {
            // リストを解放します。
            drop_in_place(self.list);
            dealloc(self.list as *mut u8, self.list_layout);
        }
    }
}
// PoolListのイテレータです。
struct PoolListItr {

    pos: *mut *mut Pool, // 現在の位置です。
    end: *mut *mut Pool, // 番兵です。
}
impl Iterator for PoolListItr {

    type Item = *mut Pool;

    // 現在の位置の値を取得し、イテレータを一つ進めます。
    fn next(&mut self) -> Option<Self::Item> {

        if self.pos >= self.end {

            None

        } else {

            unsafe {
                let value = *self.pos;
                self.pos = self.pos.offset(1);
                Some(value)
            }
        }
    }
}
impl IntoIterator for &PoolList {

    type Item = *mut Pool;

    type IntoIter = PoolListItr;

    // イテレータを取得します。
    fn into_iter(self) -> Self::IntoIter {

        unsafe {
            PoolListItr{pos: self.list, end: self.list.offset(self.length as isize)}
        }
    }
}

/// 固定長メモリプールです。
pub struct Pool {

    size: usize,           // データサイズです。size >= address_size
    max_length: usize,     // このプールが管理するデータの数です。
    buffer: *mut u8,       // データを取り出すバッファです。
    buffer_layout: Layout, // バッファのメモリレイアウトです。
    free_length: usize,    // 分配可能なデータの数です。
    free_ptr: *mut u8,     // 次分配するデータのポインタです。
}
impl Pool {

    /// インスタンスを作成します。
    /// 
    /// # Arguments
    /// 
    /// * `size`  - 固定長データ領域のサイスです。
    /// * `count` - このプールが管理するデータの数です。
    /// 
    pub fn new(size: usize, count: usize) -> Self {

        unsafe {
            const PTR_SIZE: usize = size_of::<*mut u8>();
            
            // データサイズを size >= address_size に調整します。
            let data_size = if size < PTR_SIZE {PTR_SIZE} else {size};
            
            // バッファを作成します。
            let buffer_size = data_size * count;
            let buffer_align = buffer_size.next_power_of_two();
            let buffer_layout = Layout::from_size_align_unchecked(buffer_size, buffer_align);
            let buffer = alloc(buffer_layout);
            
            // バッファから固定長データ領域の単方向連結リストを構築します。
            let mut list_top = null_mut() as *mut u8;
            for i in (0..buffer_size).step_by(data_size) {
                // 前のデータ領域の値に次のデータ領域のアドレスを設定します。
                let cullent = buffer.offset(i as isize);
                list_top = Pool::push_list(list_top, cullent);
            }
            
            // インスタンスを作成します。
            Pool { 
                size: data_size,
                max_length: count,
                buffer: buffer, 
                buffer_layout: buffer_layout,
                free_length: count, 
                free_ptr: list_top 
            }
        }
    }

    /// 固定長メモリ領域を確保します。
    pub unsafe fn alloc(&mut self) -> *mut u8 {

        if self.free_length == 0 {

            let result;
            (self.free_ptr, result) = Pool::pop_list(self.free_ptr);
            self.free_length -= 1;
            result

        } else {

            null_mut()
        }
    }

    /// 固定長メモリ領域を解放します。
    /// 
    /// # Arguments
    /// 
    /// * `ptr` - 解放するポインタです。
    /// 
    pub unsafe fn dealloc(&mut self, ptr: *mut u8) {

        if self.is_managed(ptr) {

            self.free_ptr = Pool::push_list(self.free_ptr, ptr);
            self.free_length += 1;
        }
    }

    /// 対象のポインタがこのプールの管轄か判定します。
    /// 
    /// # Arguments
    /// 
    /// * `ptr` - 判定するポインタです。
    /// 
    pub fn is_managed(&self, ptr: *mut u8) -> bool {

        unsafe {
            let min_ptr = self.buffer;
            let max_ptr = min_ptr.offset((self.size * self.max_length) as isize);
            min_ptr <= ptr && ptr <= max_ptr
        }
    }

    /// 固定長メモリ領域のバイトサイズを取得します。
    pub fn size(&self) -> usize {

        self.size
    }

    /// このプールが管理する固定長メモリ領域の数を取得します。
    pub fn max_len(&self) -> usize {

        self.max_length
    }

    /// 現在使用可能な固定長メモリ領域の数を取得します。
    pub fn free_len(&self) -> usize {

        self.free_length
    }

    /// プールが空か判定します。
    pub fn is_empty(&self) -> bool {

        self.free_length == 0
    }

    /// プールがフルか判定します。
    pub fn is_full(&self) -> bool {

        self.free_length == self.max_length
    }

    // データ領域の単方向連結リストに追加し、次の先頭を返します。
    //
    // # Arguments
    //
    // * `list_top` - リストの先頭です。
    // * `element` - 追加ポインタです。
    // 
    unsafe fn push_list(list_top: *mut u8, element: *mut u8) -> *mut u8 {

        let element_usize = element as *mut usize;
        (*element_usize) = list_top as usize;
        element
    }

    // データ領域の単方向連結リストから取り出し、次の先頭と取り出したポインタのタプルを返します。
    //
    // # Arguments
    //
    // * `list_top` - リストの先頭です。
    // 
    unsafe fn pop_list(list_top: *mut u8) -> (*mut u8, *mut u8) {

        let result = list_top;
        let list_top_usize = list_top as *mut usize;
        let next_top = (*list_top_usize) as *mut u8;
        (next_top, result)
     }
}
impl Drop for Pool {

    /// 終了時に呼ばれます。
    fn drop(&mut self) {
        
        unsafe {
            dealloc(self.buffer, self.buffer_layout);
        }
    }
}
