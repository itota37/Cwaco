// -------------------------
//
// Cwago.
//
// cwago/cwago_serde/src/pack.rs
// (C) 2022 CwagoCommunity.
//
//! 型を隠したパック型を提供します。
// =========================

use std::{
    mem, 
    any::type_name
};

use cwago_utility::log::error;

/// 型を隠して保持します。
pub struct Pack {
    data: *mut (),
    drop: fn(*mut ()),
    id: usize,
}
impl Pack {
    /// 作成します。
    /// 
    /// # 引数
    /// 
    /// * `value` - パックする値です。
    /// 
    /// # 戻り値
    /// 
    /// パックした値です。
    /// 
    pub fn from<T>(value: T) -> Self {
        let data = Box::into_raw(Box::new(value)) as *mut ();
        let drop = |ptr: *mut ()| {
            mem::drop(unsafe{ Box::from_raw(ptr as *mut T) })
        };
        let id = Self::id::<T>();
        Pack { data, drop, id }
    }

    /// 可変参照に変換します。
    /// 
    /// # 戻り値
    /// 
    /// 可変参照です。
    /// 
    pub unsafe fn unpack_mut<T>(&mut self) -> &mut T {
        if self.id != Self::id::<T>() {
            Self::err::<T>();
        }

        unsafe{ &mut *(self.data as *mut T) }
    } 

    /// 元の値に変換します。
    /// 
    /// # 戻り値
    /// 
    /// 元の値です。
    /// 
    pub unsafe fn unpack<T>(&mut self) -> T {
        if self.id != Self::id::<T>() {
            Self::err::<T>();
        }

        let value = unsafe{ Box::from_raw(self.data as *mut T) };
        mem::forget(self);
        *value
    } 

    /// 型IDを作成します。
    /// 
    /// # 戻り値
    /// 
    /// 型IDです。
    /// 
    fn id<T>() -> usize {
        Self::id::<T> as usize
    }

    /// エラー処理です。
    /// 
    /// # 注意
    /// 
    /// パニックします。
    /// 
    fn err<T>() -> ! {
        error!("{}型に変換できませんでした。", type_name::<T>());
        panic!()
    }
}
impl Drop for Pack {
    /// 解体します。
    fn drop(&mut self) {
        (self.drop)(self.data)
    }
}