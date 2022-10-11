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

use cwago_utility::hash::{FxHashMap, FxHashSet};

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

/// コンポーネントアクセス時に指定可能な型を定義するトレイトです。
pub trait Archetype 
where Self::Iter: Iterator {
    
    /// コンポーネントのバッファにアクセスするためのイテレータ型です。
    type Iter;

    /// アクセスするコンポーネントを指定したランタイム時構造体を生成します。
    fn for_request() -> Request;

    /// バッファのマップからバッファにアクセスするイテレータを生成します。
    fn for_iter(buff: &Buffers) -> Self::Iter;
}

/// アクセスするコンポーネントを指定したランタイム時構造体です。
#[derive(Debug)]
pub struct Request {

    /// 型情報のリストです。
    types: Vec<TypePattern>,
}
impl Request {

    /// 作成します。
    pub(crate) fn new() -> Self {
        Request { 
            types: Vec::new() 
        }
    }

    /// 重複なしに追加します。
    pub(crate) fn append(&mut self, rhs: &Request) {
        let mut set = FxHashSet::default();
        for ty in &self.types {
            set.insert(ty.clone());
        }
        for ty in &rhs.types {
            set.insert(ty.clone());
        }
        self.types = set.into_iter().collect();
    }
}

/// アクセスパターンの列挙と型情報です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum TypePattern {

    /// 不変参照です。
    Const(Type),

    /// 可変参照です。
    Mut(Type),
}

/// 型別のバッファのリストです。
pub struct Buffers {

    /// 型別のバッファです。
    buffs: FxHashMap<Type, BufferPattern>,
}
impl Buffers {

    /// バッファを取得します。
    /// 
    /// # Arguments
    /// 
    /// * `ty` - 取得するバッファの型情報です。
    /// 
    pub(crate) fn get(&self, ty: &Type) -> Option<BufferPattern> {

        self.get(ty)
    }
}

// Buffers
// BuffersPattern
// BufferInfo BufferMutInfo

/// パターン別バッファの列挙です。
pub(crate) enum BufferPattern {

    /// 不変バッファです。
    Const(Vec<BufferInfo>),

    /// 可変バッファです。
    Mut(Vec<BufferMutInfo>),
}

/// 不変バッファ情報です。
pub(crate) struct BufferInfo {

    /// バッファです。
    buff: *const u8,

    /// 要素数です。
    len: usize,
}
impl BufferInfo {
    
    /// 作成します。
    /// 
    /// # Arguments
    /// 
    /// * `buff` - バッファです。
    /// * `len` - 要素数です。
    /// 
    pub(crate) fn new(buff: *const u8, len: usize) -> Self {

        BufferInfo { 
            buff, 
            len 
        }
    }

    /// バッファを取得します。
    pub(crate) fn buffer(&self) -> *const u8{

        self.buff
    }

    /// 要素数を取得します。
    pub(crate) fn len(&self) -> usize {

        self.len
    }
}

/// 可変バッファ情報です。
pub(crate) struct BufferMutInfo {

    /// バッファです。
    buff: *mut u8,

    /// 要素数です。
    len: usize,
}
impl BufferMutInfo {
    
    /// 作成します。
    /// 
    /// # Arguments
    /// 
    /// * `buff` - バッファです。
    /// * `len` - 要素数です。
    /// 
    pub(crate) fn new(buff: *mut u8, len: usize) -> Self {

        BufferMutInfo { 
            buff, 
            len 
        }
    }

    /// バッファを取得します。
    pub(crate) fn buffer(&self) -> *mut u8{

        self.buff
    }

    /// 要素数を取得します。
    pub(crate) fn len(&self) -> usize {

        self.len
    }
}