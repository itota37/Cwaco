// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/sys/comp/ty.rs
// (C) 2022 Taichi Ito.
// ====================

//! コンポーネントに関連する型情報を提供します。

// 目次
// --------------------
// Type
// RefType
// Archetype
// ====================

use std::{
    any::{
        TypeId, 
        type_name
    }, 
    mem::{size_of, transmute}, 
    hash::Hash, 
    fmt::Display, ptr::drop_in_place
};
use cwago_utility::hash::{
    FxHashMap, 
    FxHashSet
};
use crate::{err::Error, comp::Data};



// --------------------
//
// Type
//
// ====================



/// ランタイム型情報です。
#[derive(Debug, Clone, Copy)]
pub struct Type {

    /// 型IDです。
    id: TypeId,
    /// 型名です。
    name: &'static str,
    /// 型サイズです。
    size: usize,
    /// ドロップ関数オブジェクトです。
    drop_obj: fn(*mut u8),
    /// クローン関数オブジェクトです。
    clone_obj: fn(*const u8, *mut u8),
}
impl Type {
    
    /// ランタイム型情報を取得します。
    pub fn of<D>() -> Self
    where D: Data {

        Type { 
            id: TypeId::of::<D>(), 
            name: type_name::<D>(), 
            size: size_of::<D>(),
            drop_obj: |ptr: *mut u8| unsafe { 
                drop_in_place(transmute::<*mut u8, *mut D>(ptr)); 
            },
            clone_obj: |from_ptr: *const u8, to_ptr: *mut u8| unsafe { 
                let from_ptr = transmute::<*const u8, *const D>(from_ptr);
                let to_ptr = transmute::<*mut u8, *mut D>(to_ptr);
                let from_ref = from_ptr.as_ref()
                .expect(
                    format_args!("重大なエラーが発生しました。[cwago_ecs/src/ty.rs : Type::of<{}>/0]", 
                    type_name::<D>()).as_str().unwrap()
                );
                let to_ref = to_ptr.as_mut()
                .expect(
                    format_args!("重大なエラーが発生しました。[cwago_ecs/src/ty.rs : Type::of<{}>/1]", 
                    type_name::<D>()).as_str().unwrap()
                );
                *to_ref = from_ref.clone();
            }
        }
    }

    /// 型IDを取得します。
    pub fn id(&self) -> TypeId {

        self.id
    }

    /// 型名を取得します。
    pub fn name(&self) -> &'static str {

        self.name
    }

    /// 型サイズを取得します。
    pub fn size(&self) -> usize {

        self.size
    }

    /// バッファ上のデータをドロップします。
    /// 
    /// # Arguments 
    /// 
    /// * `ptr` - データのポインタです。
    /// 
    pub(crate) fn drop_ptr(&self, ptr: *mut u8) {
        (self.drop_obj)(ptr);
    }

    /// バッファ上のデータを別のバッファへクローンさせます。
    /// 
    /// # Arguments
    /// 
    /// * `from_ptr` - クローン元のポインタです。
    /// * `to_ptr` - クローン先のポインタです。
    /// 
    pub(crate) fn clone_ptr(&self, from_ptr: *const u8, to_ptr: *mut u8) {
        (self.clone_obj)(from_ptr, to_ptr);
    }
}
impl Display for Type {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Type({})", self.name()))
    }
}
impl PartialEq for Type {
    
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Type {}
impl PartialOrd for Type {
    
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}
impl Ord for Type {
    
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
impl Hash for Type {
    
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}



// --------------------
//
// RefType
//
// ====================



/// ランタイム参照型情報です。
#[derive(Debug, Clone, Copy)]
pub enum RefType {

    /// 不変参照型です。
    Const(Type),

    /// 可変参照型です。
    Mut(Type),
}
impl Display for RefType {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefType::Const(ty) => f.write_fmt(format_args!("Const({})", ty.to_string())),
            RefType::Mut(ty) => f.write_fmt(format_args!("Mut({})", ty.to_string())),
        }
    }
}
impl PartialEq for RefType {
    
    fn eq(&self, other: &Self) -> bool {
        match self {
            RefType::Const(ty1) => {
                match other {
                    RefType::Const(ty2) => ty1.eq(ty2),
                    RefType::Mut(ty2) => false,
                }
            },
            RefType::Mut(ty1) => {
                match other {
                    RefType::Const(ty2) => false,
                    RefType::Mut(ty2) => ty1.eq(ty2),
                }
            },
        }
    }
}
impl Eq for RefType {}
impl PartialOrd for RefType {
    
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {

        // Const < Mut
        match self {
            RefType::Const(ty1) => {
                match other {
                    RefType::Const(ty2) => ty1.partial_cmp(ty2),
                    RefType::Mut(ty2) => Some(std::cmp::Ordering::Less),
                }
            },
            RefType::Mut(ty1) => {
                match other {
                    RefType::Const(ty2) => Some(std::cmp::Ordering::Greater),
                    RefType::Mut(ty2) => ty1.partial_cmp(ty2),
                }
            },
        }
    }
}
impl Ord for RefType {
    
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        
        // Const < Mut
        match self {
            RefType::Const(ty1) => {
                match other {
                    RefType::Const(ty2) => ty1.cmp(ty2),
                    RefType::Mut(ty2) => std::cmp::Ordering::Less,
                }
            },
            RefType::Mut(ty1) => {
                match other {
                    RefType::Const(ty2) => std::cmp::Ordering::Greater,
                    RefType::Mut(ty2) => ty1.cmp(ty2),
                }
            },
        }
    }
}
impl Hash for RefType {
    
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            RefType::Const(ty) => ty.hash(state),
            RefType::Mut(ty) => ty.hash(state),
        }
    }
}



// --------------------
//
// Archetype
//
// ====================



/// 指定型の条件を定義します。
pub trait Archetype {
    
    /// 参照型情報を取得します。
    fn for_type() -> RefType;
}