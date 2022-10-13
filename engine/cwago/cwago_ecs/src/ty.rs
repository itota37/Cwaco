// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/sys/comp/ty.rs
// (C) 2022 Taichi Ito.
// ====================

//! コンポーネントに関連する型情報を提供します。

use std::{
    any::{
        TypeId, 
        type_name
    }, 
    mem::size_of, 
    hash::Hash, 
    fmt::Display
};
use cwago_utility::hash::{
    FxHashMap, 
    FxHashSet
};
use crate::err::Error;

/// ランタイム型情報です。
#[derive(Debug, Clone, Copy)]
pub struct Type {

    /// 型IDです。
    id: TypeId,

    /// 型名です。
    name: &'static str,

    /// 型サイズです。
    size: usize,
}
impl Type {
    
    /// ランタイム型情報を取得します。
    pub fn of<T>() -> Self
    where T: Sized + 'static {

        Type { 
            id: TypeId::of::<T>(), 
            name: type_name::<T>(), 
            size: size_of::<T>() 
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

/// 指定型の条件を定義します。
pub trait Archetype {
    
    /// 参照型情報を取得します。
    fn for_type() -> RefType;
}