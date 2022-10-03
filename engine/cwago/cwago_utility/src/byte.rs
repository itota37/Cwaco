// --------------------
//
// Cwago.
//
// cwago/cwago_utility/src/byte.rs
// (C) 2022 Taichi Ito.
// ====================

//! バイトオーダー機能を提供します。

use std::mem::size_of;

/// システムがリトルエンディアンの場合に、ビッグエンディアンへ変換します。
pub fn change_big_endian(binary: &Vec<u8>) -> Vec<u8> {

    if let Endian::Little = Endian::of() {
        
        change_endian(binary)
        
    } else {

        binary.clone()
    }
}

/// システムがビッグエンディアンの場合に、リトルエンディアンへ変換します。
pub fn change_little_endian(binary: &Vec<u8>) -> Vec<u8> {

    if let Endian::Big = Endian::of() {
        
        change_endian(binary)

    } else {

        binary.clone()
    }
}

/// ビッグエンディアンとリトルエンディアンを入れ替えます。
pub fn change_endian(binary: &Vec<u8>) -> Vec<u8> {

    let mut vec = Vec::new();
    for b in binary.iter().rev() {

        vec.push(*b);
    }
    vec
}

/// バイナリデータを取得します。
pub fn binary_of<T>(number: T) -> Vec<u8>
where T: ToBinary {

    number.to_binary()
}

/// バイナリデータを取得するトレイトです。
pub trait ToBinary {

    /// バイナリデータを取得します。
    fn to_binary(&self) -> Vec<u8>;
}

macro_rules! impl_ToBinary {
    ( $t: ty ) => {
        
        impl ToBinary for $t {
            fn to_binary(&self) -> Vec<u8> {
                
                let mut vec = Vec::new();
                let ptr = self as *const $t as *const u8;
                for i in 0..size_of::< $t >() {
        
                    vec.push(unsafe{*ptr.add(i)});
                }
                vec
            }
        }
    };
}
impl_ToBinary!(i8);
impl_ToBinary!(u8);
impl_ToBinary!(i16);
impl_ToBinary!(u16);
impl_ToBinary!(i32);
impl_ToBinary!(u32);
impl_ToBinary!(i64);
impl_ToBinary!(u64);
impl_ToBinary!(i128);
impl_ToBinary!(u128);
impl_ToBinary!(f32);
impl_ToBinary!(f64);
impl_ToBinary!(char);
impl_ToBinary!(bool);

/// エンディアンの列挙です。
pub enum Endian {
    /// ビッグエンディアンです。
    Big,
    /// リトルエンディアンです。
    Little,
    /// ミドルエンディアンです。
    Middle,
}
impl Endian {

    /// 現在の環境のバイトオーダーを判定します。
    pub fn of() -> Self {
        let v32 = 0x00112233_u32;
        let p32 = std::ptr::addr_of!(v32);
        let p8 = p32 as *const [u8; 4];
        unsafe {
            let a8 = *p8;
            match a8[0] {
                0x00_u8 => Self::Big,
                0x33_u8 => Self::Little,
                _ => Self::Middle,
            }
        }
    }
}