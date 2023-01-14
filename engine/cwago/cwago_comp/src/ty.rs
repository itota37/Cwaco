// -------------------------
//
// Cwago.
//
// cwago/cwago_type/src/ty.rs
// (C) 2023 CwagoCommunity.
//
//! 動的型情報を提供します。
// =========================

use std::{
    mem::{
        size_of, 
        forget
    }, 
    ptr::drop_in_place, sync::Once, any::type_name
};

use cwago_utility::log::error;

/// 動的型情報です。
pub struct Info {
    name: &'static str,                    // 型名です。
    size: usize,                           // 型サイズです。
    init: unsafe fn(*mut ()),              // デフォルト初期化します。
    drop: unsafe fn(*mut ()),              // ドロップします。
    mov: unsafe fn(*mut(), *mut()),        // 第1引数の内容を第2引数の位置にムーブします。
    cast_se: unsafe fn(
        *mut()
    ) -> *mut dyn erased_serde::Serialize, // erased_serde::Serializeトレイトポインタに変換します。
    run_de: unsafe fn(
        *mut(), 
        &mut (dyn for<'s> erased_serde::Deserializer<'s>)
    ) -> Result<(), erased_serde::Error>,  // デシリアライズします。
}
impl Info {
    /// 作成します。
    /// 
    /// # 引数
    /// 
    /// * `name` - 他の型と区別可能な一意の名前です。
    /// 
    /// # 戻り値
    /// 
    /// インスタンスです。
    /// 
    fn new<'de, T>(name: &'static str) -> Self
    where T: Sized + Default + serde::Serialize + serde::Deserialize<'de> + 'static 
    {
        Self { 
            name, 
            size: size_of::<T>(), 
            init: |ptr|{
                let ptr = ptr as *mut T;
                let ini = T::default();
                // iniをptrにバイト単位でコピーします。
                unsafe{ ptr.copy_from(&ini, 1) };
                // ptrにムーブしたのでiniがドロップしないようにします。
                forget(ini);                                 
            }, 
            drop: |ptr|{
                let ptr = ptr as *mut T;
                // ptrをドロップします。
                unsafe{ drop_in_place(ptr) }; 
            }, 
            mov: |l, r|{
                let l = l as *mut T;
                let r = r as *mut T;
                // lをドロップします。
                unsafe{ drop_in_place(l) };    
                // rをlにバイト単位でコピーします。
                unsafe{ l.copy_from(r, 1) };
            }, 
            cast_se: |ptr|{
                let ptr = ptr as *mut T;
                let ptr = ptr as *mut dyn erased_serde::Serialize;
                ptr
            }, run_de: {
                |
                    ptr: *mut(), 
                    de: &mut (dyn for<'s> erased_serde::Deserializer<'s>)
                | -> Result<(), erased_serde::Error> 
                {
                    let ptr = ptr as *mut T;
                    let val = erased_serde::deserialize::<T>(de)?;
                    // ptrをドロップします。
                    unsafe{ drop_in_place(ptr) };
                    // valをptrにバイト単位でコピーします。
                    unsafe{ ptr.copy_from(&val, 1) };
                    // valにムーブしたのでiniがドロップしないようにします。
                    forget(val);
                    Ok(())
                }
            } 
        }
    }

    /// 型名を取得します。
    /// 
    /// # 戻り値
    /// 
    /// 他の型と区別可能な一意の名前です。
    /// 
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// 型サイズを取得します。
    /// 
    /// # 戻り値
    /// 
    /// 型サイズです。
    /// 
    pub fn size(&self) -> usize {
        self.size
    }

    /// バッファ上の位置を初期化します。
    /// 
    /// # 引数
    /// 
    /// * `ptr` - バッファ上の初期化位置のポインタです。
    /// 
    unsafe fn initialize(&self, ptr: *mut ()) {
        (self.init)(ptr)
    }

    /// パッファ上の位置を解体します。
    /// 
    /// # 引数
    /// 
    /// * `ptr` - バッファ上の解体位置のポインタです。
    /// 
    unsafe fn drop(&self, ptr: *mut ()) {
        (self.drop)(ptr)
    }

    /// バッファ上のある位置から別の位置へ値をムーブします。
    /// 
    /// # 引数
    /// 
    /// * `from` - ムーブする値の位置のポインタです。
    /// * `to` - ムーブ先の位置のポインタです。
    /// 
    unsafe fn move_ptr(&self, from: *mut (), to: *mut ()) {
        (self.mov)(from, to)
    }

    /// シリアライズトレイトポインタに変換します。
    /// 
    /// # 引数
    /// 
    /// * `ptr` - 変換する位置のポインタです。
    /// 
    /// # 戻り値
    /// 
    /// 変換したシリアライズトレイトポインタです。
    /// 
    unsafe fn cast_serialize(
        &self, 
        ptr: *mut ()
    ) -> *mut dyn erased_serde::Serialize {
        (self.cast_se)(ptr)
    }

    /// デシリアライズします。
    /// 
    /// # 引数
    /// 
    /// * `ptr` - デシリアライズ先の位置のポインタです。
    /// * `deserializer` - デシリアライザです。
    /// 
    /// # 戻り値
    /// 
    /// 失敗した際、エラーを返します。
    /// 
    unsafe fn deserialize(
        &self, 
        ptr: *mut (), 
        deserializer: &mut (dyn for<'de> erased_serde::Deserializer<'de>)
    ) -> Result<(), erased_serde::Error> {
        (self.run_de)(ptr, deserializer)
    }
}

/// 作成します。
/// 
/// # 引数
/// 
/// * `name` - 他の型と区別可能な一意の名前です。
/// 
/// # 戻り値
/// 
/// インスタンスです。
/// 
pub fn info_of<'de, T>(name: &'static str) -> &'static Info
where T: Sized + Default + serde::Serialize + serde::Deserialize<'de> + 'static 
{
    static mut INFO: Option<Info> = None;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| unsafe{
        INFO = Some(Info::new::<'de, T>(name));
    });
    match unsafe{ INFO.as_ref() } {
        Some(info) => info,
        None => {
            error!("'{}'の動的型情報の作成に失敗しました。", type_name::<T>());
            panic!()
        },
    }
}

/// バッファ上の位置を初期化します。
/// 
/// # 引数
/// 
/// * `ptr` - バッファ上の初期化位置のポインタです。
/// 
pub unsafe fn initialize_buf(info: &'static Info, ptr: *mut u8) {
    info.initialize(ptr as *mut ())
}

/// パッファ上の位置を解体します。
/// 
/// # 引数
/// 
/// * `ptr` - バッファ上の解体位置のポインタです。
/// 
pub unsafe fn drop_buf(info: &'static Info, ptr: *mut u8) {
    info.drop(ptr as *mut ())
}

/// バッファ上のある位置から別の位置へ値をムーブします。
/// 
/// # 引数
/// 
/// * `from` - ムーブする値の位置のポインタです。
/// * `to` - ムーブ先の位置のポインタです。
/// 
pub unsafe fn move_buf(info: &'static Info, from: *mut u8, to: *mut u8) {
    info.move_ptr(from as *mut (), to as *mut ())
}

/// シリアライズトレイトポインタに変換します。
/// 
/// # 引数
/// 
/// * `ptr` - 変換する位置のポインタです。
/// 
/// # 戻り値
/// 
/// 変換したシリアライズトレイトポインタです。
/// 
pub unsafe fn cast_serialize_from_buf(
    info: &'static Info, 
    ptr: *mut u8
) -> *mut dyn erased_serde::Serialize {
    info.cast_serialize(ptr as *mut ())
}

/// デシリアライズします。
/// 
/// # 引数
/// 
/// * `ptr` - デシリアライズ先の位置のポインタです。
/// * `deserializer` - デシリアライザです。
/// 
/// # 戻り値
/// 
/// 失敗した際、エラーを返します。
/// 
pub unsafe fn deserialize_buf(
    info: &'static Info, 
    ptr: *mut u8, 
    deserializer: &mut (dyn for<'de> erased_serde::Deserializer<'de>)
) -> Result<(), erased_serde::Error> {
    info.deserialize(ptr as *mut (), deserializer)
}