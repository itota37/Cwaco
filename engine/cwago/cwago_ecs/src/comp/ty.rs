// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/component/ty.rs
// (C) 2022 Taichi Ito.
// ====================

//! Component-Dataに設定可能な型の条件と関連機能を提供します。

use std::{
    any::{
        TypeId, 
        type_name
    }, 
    mem::{
        size_of, 
        transmute
    }, 
    hash::Hash,
    fmt::Display,
    marker::PhantomData,
    ptr::drop_in_place
};
use cwago_utility::hash::FxHashMap;
use super::data::Data;

/// Component-Dataに設定可能な型の条件を定義するトレイトです。
pub trait Archetype 
where Self::Iter: Iterator + Clone {

    /// イテレータ型です。
    type Iter;

    /// ランタイムに参照可能なリクエストを構築します。
    fn request() -> Request;

    /// リクエストに対するレスポンスからイテレータを構築します。
    fn iter(res: &Response) -> Self::Iter;
    
    /// 値を移動可能な形に再構築します。 
    fn values(&self) -> Datas;
}

/// ランタイムに参照可能なArhetype情報です。
pub struct Request {

    types: Vec<Type>, // 型情報リストです。
}
impl Request {

    /// 作成します。
    /// 
    /// # Arguments
    /// 
    /// * `types` - 型リストです。
    /// 
    pub(crate) fn new(types: Vec<Type>) -> Self {

        Request { types }
    }
    
    /// アクセスします。
    pub(crate) fn get(&self) -> &Vec<Type> {

        &self.types
    }
}

/// 動的型情報です。
#[derive(Debug, Clone, Copy)]
pub struct Type {

    id: TypeId,                                        // Idです。
    size: usize,                                       // サイズです。
    name: &'static str,                                // 型名です。
    copy: fn(*const u8, *mut u8) -> Result<(), Error>, // ポインタからポインタへクローンした値を移動させる関数のポインタです。
    drop: fn(*mut u8),                                 // ポインタが指すメモリで
}
impl Type {
    
    /// 生成します。
    pub fn of<T>() -> Self 
    where T: Clone + 'static {

        Type { 
            id: TypeId::of::<T>(), 
            size: size_of::<T>(), 
            name: type_name::<T>(),
            copy: |from: *const u8, to: *mut u8| {
                
                let from = unsafe { transmute::<*const u8, *const T>(from) };
                let to = unsafe { transmute::<*mut u8, *mut T>(to) };
                
                let from = if let Some(from) = unsafe {from.as_ref()} {
                    from
                } else {
                    return Err(Error::ArgumentError);
                };
                let to = if let Some(to) = unsafe {to.as_mut()} {
                    to
                } else {
                    return Err(Error::ArgumentError);
                };
                
                *to = from.clone();
                Ok(())
            },
            drop: |ptr: *mut u8| unsafe {
            
                let ptr = transmute::<*mut u8, *mut T>(ptr);
                drop_in_place(ptr);
            },
        }
    }

    /// 型Idを取得します。
    pub fn id(&self) -> TypeId {

        self.id
    }

    /// 型サイズを取得します。
    pub fn size(&self) -> usize {

        self.size
    }

    /// 型名を取得します。
    pub fn name(&self) -> &'static str {

        self.name
    }
    
    /// ポインタからポインタへクローンした値を移動します。
    ///
    /// # Arguments
    ///
    /// * `from` - 参照元データのポインタです。
    /// * `to` - 代入先メモリのポインタです。
    ///
    pub(crate) fn copy_ptr(&self, from: *const u8, to: *mut u8) -> Result<(), Error> {
        
        (self.copy)(from, to)
    }
    
    /// 型がDropを実装している場合、ポインタのメモリ上でdropを呼び出します。
    ///
    /// # Arguments
    ///
    /// * `ptr` - 対象のポインタです。
    ///
    pub(crate) fn drop_ptr(&self, ptr: *mut u8) {
        
        (self.drop)(ptr);
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

/// リクエストに対して返された情報です。
pub struct Response {

    ids: Vec<ResponseData>,                    // Entity-Id生配列データのリストです。
    datas: FxHashMap<Type, Vec<ResponseData>>, // 型別のComponent-Data生配列データのリストです。
}
impl Response {

    /// 新規作成します。
    /// 
    /// # Arguments
    /// 
    /// * `types` - 型リストです。
    /// 
    pub(crate) fn new(types: &Vec<Type>) -> Self {

        let ids = Vec::new();
        let mut datas = FxHashMap::default();
        for ty in types {

            datas.insert(ty.clone(), Vec::new());
        }
        Response { ids, datas }
    }
    
    /// 生配列を追加します。
    /// 
    /// # Arguments
    /// 
    /// * `ids` - Entity-Idの生配列データです。
    /// * `datas` - 型別のComponent-Data生配列データです。
    /// 
    pub(crate) fn push(&mut self, ids: &ResponseData, datas: &FxHashMap<Type, ResponseData>) -> Result<(), Error> {

        // 型の種類の数が一致しないので、終了します。
        if self.datas.len() != datas.len() { return Err(Error::MissingType); }

        for data in datas {

            // IdとDataの数が一致しない、
            // または、型の種類が一致しないので、終了します。
            if (&ids).len != data.1.len 
            || !self.datas.contains_key(data.0) { 
                return Err(Error::MissingType); 
            }
        }

        self.ids.push(ids.clone());
        for data in datas {

            self.datas.get_mut(data.0)
            .unwrap()
            .push(data.1.clone());
        }
        
        Ok(())
    }

    /// Idのレスポンスデータのリストを取得します。
    pub(crate) fn ids(&self) -> &Vec<ResponseData> {

        &self.ids
    }

    /// 型からレスポンスデータのリストを取得します。
    pub(crate) fn datas<T>(&self) -> Option<&Vec<ResponseData>> 
    where T: Clone + 'static {

        self.datas.get(&Type::of::<T>())
    }
}

/// レスポンスの1データの情報です。
#[derive(Debug, Clone, Copy)]
pub(crate) struct ResponseData {

    ptr: ResponsePtr, // 生配列ポインタです。
    len: usize,       // 配列の要素数です。
}
impl ResponseData {
    
    /// ポインタ、サイズ、要素数から不変データを生成します。
    /// 
    /// # Arguments
    /// 
    /// * `ptr` - ポインタです。
    /// * `len` - 要素数です。
    /// 
    pub(crate) fn new(ptr: *const u8, len: usize) -> Self {

        ResponseData { ptr: ResponsePtr::Const(ptr), len }
    }
    
    /// ポインタ、サイズ、要素数から可変データを生成します。
    /// 
    /// # Arguments
    /// 
    /// * `ptr` - ポインタです。
    /// * `size` - サイズです。
    /// * `len` - 要素数です。
    /// 
    pub(crate) fn new_mut(ptr: *mut u8, size: usize, len: usize) -> Self {

        ResponseData { ptr: ResponsePtr::Mut(ptr), len }
    }

    /// 不変ポインタと要素数を取得します。
    pub(crate) fn get(&self) -> Option<(*const u8, usize)> {

        match self.ptr {

            ResponsePtr::Const(ptr) => Some((ptr, self.len)),
            ResponsePtr::Mut(_) => None,
        }
    }

    /// 不変ポインタと要素数を取得します。
    pub(crate) fn get_mut(&self) -> Option<(*mut u8, usize)> {

        match self.ptr {
            
            ResponsePtr::Const(_) => None,
            ResponsePtr::Mut(ptr) => Some((ptr, self.len)),
        }
    }
}

/// レスポンスポインタの種類です。
#[derive(Debug, Clone, Copy)]
enum ResponsePtr {

    Const(*const u8), // 不変ポインタです。
    Mut(*mut u8),     // 可変ポインタです。
}

/// 単体Idのデータを移動させるための構造体です。
pub struct Datas {
    
    datas: FxHashMap<Type, Vec<u8>>, // バイナリデータのリストです。
}
impl Datas {
    
    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `ptrs` - データポインタリストです。
    ///
    pub(crate) fn new(ptrs: FxHashMap<Type, *const u8>) -> Self {
        
        // 型別に要素をクローンします。
        let mut datas = FxHashMap::default();
        for (ty, ptr) in ptrs {
            
            let mut vec = vec![0_u8; ty.size()];
            ty.copy_ptr(ptr, vec.as_mut_ptr());
            datas.insert(ty, vec);
        }
        Datas { datas }
    }
    
    /// アクセスします。
    /// 
    /// # Arguments
    /// 
    /// * `ty` - 対象のポインタの型です。
    /// 
    pub(crate) fn get(&self, ty: &Type) -> Option<*const u8> {

        match self.datas.get(ty) {
            
            Some(vec) => Some(vec.as_ptr()),
            None => None
        }
    }
    
    /// 統合します。
    ///
    /// # Arguments
    ///
    /// * `datas` - 統合するデータです。
    ///
    pub(crate) fn append(&mut self, datas: &Datas) {
        
        for (ty, vec) in &datas.datas {
            
            self.datas.insert(ty.clone(), vec.clone());
        }
    }

    /// 分割します。
    ///
    /// # Arguments
    ///
    /// * `req` - 分割するデータのリクエストです。
    ///
    pub(crate) fn split(&mut self, req: &Request) -> Result<Datas, Error> {

        let mut datas = FxHashMap::default();
        for ty in req.get() {
            
            let ptr = match self.datas.get(ty) {
                
                Some(vec) => vec.as_ptr(),
                None => return Err(Error::MissingType),
            };

            let mut vec = vec![0_u8; ty.size()];
            ty.copy_ptr(ptr, vec.as_mut_ptr());
            datas.insert(ty.clone(), vec);

            let ptr = match self.datas.get_mut(ty) {
                
                Some(vec) => vec.as_mut_ptr(),
                None => return Err(Error::MissingType),
            };

            ty.drop_ptr(ptr);
            self.datas.remove(ty);
        }
        Ok(Datas { datas })
    }
    
    /// リクエストを作成します。
    pub(crate) fn request(&self) -> Request {

        let mut types = Vec::new();
        for ty in self.datas.keys() {

            types.push(*ty);
        }

        Request::new(types)
    }
}
impl Drop for Datas {
    
    fn drop(&mut self) {
        
        for (ty, vec) in self.datas.iter_mut() {
            
            ty.drop_ptr(vec.as_mut_ptr());
        }
    }
}

/// 不変イテレータ内で生ポインタを扱う為の構造体です。
#[derive(Debug, Clone)]
struct IterData<T> {

    _dammy: PhantomData<T>,
    ptr: *const u8,
    step: usize,
    idx: usize,
    len: usize,
}
impl<T> IterData<T> {

    /// 生成します。
    /// 
    /// #Arguments
    /// 
    /// * `ptr` - バッファです。
    /// * `step` - 1ステップの長さです。
    /// * `len` - 長さです。
    /// 
    fn new(ptr: *const u8, step: usize, len: usize) -> Self {

        Self { 
            _dammy: PhantomData::default(), 
            ptr, 
            step, 
            idx: 0, 
            len 
        }
    }
}
impl<T> Iterator for IterData<T> {
    
    type Item = *const T;

    fn next(&mut self) -> Option<Self::Item> {
        
        if self.idx < self.len {
            
            let res_cnt = self.idx;
            self.idx += 1;
            
            unsafe {
                let ptr = self.ptr.add(self.step * res_cnt);
                let ptr = transmute::<*const u8, *const T>(ptr);
                Some(ptr)
            }
            
        } else {
        
            None
        }
    }
}

/// 可変イテレータ内で生ポインタを扱う為の構造体です。
#[derive(Debug, Clone)]
struct IterMutData<T> {

    _dammy: PhantomData<T>,
    ptr: *mut u8,
    step: usize,
    idx: usize,
    len: usize,
}
impl<T> IterMutData<T> {

    /// 生成します。
    /// 
    /// #Arguments
    /// 
    /// * `ptr` - バッファです。
    /// * `step` - 1ステップの長さです。
    /// * `len` - 長さです。
    /// 
    fn new(ptr: *mut u8, step: usize, len: usize) -> Self {

        Self { 
            _dammy: PhantomData::default(), 
            ptr, 
            step, 
            idx: 0, 
            len 
        }
    }
}
impl<T> Iterator for IterMutData<T> {
    
    type Item = *mut T;

    fn next(&mut self) -> Option<Self::Item> {
        
        if self.idx < self.len {
            
            let res_cnt = self.idx;
            self.idx += 1;
            
            unsafe {
                let ptr = self.ptr.add(self.step * res_cnt);
                let ptr = transmute::<*mut u8, *mut T>(ptr);
                Some(ptr)
            }
            
        } else {
        
            None
        }
    }
}

/// 不変イテレータです。
#[derive(Debug, Clone)]
pub struct Iter<'i, T> {
    _dammy: PhantomData<&'i T>,
    iters: Vec<IterData<T>>,
}
impl<'i, T> Iter<'i, T> {

    /// レスポンスから作成します。
    /// 
    /// # Arguments
    /// 
    /// * `datas` - レスポンスデータのリストです。
    /// 
    pub(crate) fn new(datas: &Vec<ResponseData>) -> Self {

        let mut iters = Vec::new();
        for data in datas {

            if let Some((ptr, len)) = data.get() {

                iters.push(IterData::new(ptr, size_of::<T>(), len));
            }
        }

        Self { 
            _dammy: PhantomData::default(), 
            iters 
        }
    }
}
impl<'i, T> Iterator for Iter<'i, T> {
    
    type Item = &'i T;

    fn next(&mut self) -> Option<Self::Item> {
        
        let iter = if let Some(iter) = self.iters.last_mut() {

            iter

        } else {

            return None;
        };

        if let Some(value) = iter.next() {

            return unsafe {value.as_ref::<'i>()};
        }

        self.iters.pop();

        let iter = if let Some(iter) = self.iters.last_mut() {

            iter

        } else {

            return None;
        };

        if let Some(value) = iter.next() {

            unsafe {value.as_ref::<'i>()}

        } else {

            None
        }
    }
}

/// 可変イテレータです。
#[derive(Debug, Clone)]
pub struct IterMut<'i, T> {
    _dammy: PhantomData<&'i mut T>,
    iters: Vec<IterMutData<T>>,
}
impl<'i, T> IterMut<'i, T> {

    /// レスポンスから作成します。
    /// 
    /// # Arguments
    /// 
    /// * `datas` - レスポンスデータのリストです。
    /// 
    pub(crate) fn new(datas: &Vec<ResponseData>) -> Self {

        let mut iters = Vec::new();
        for data in datas {

            if let Some((ptr, len)) = data.get_mut() {

                iters.push(IterMutData::new(ptr, size_of::<T>(), len));
            }
        }

        Self { 
            _dammy: PhantomData::default(), 
            iters 
        }
    }
}
impl<'i, T> Iterator for IterMut<'i, T> {
    
    type Item = &'i mut T;

    fn next(&mut self) -> Option<Self::Item> {
        
        let iter = if let Some(iter) = self.iters.last_mut() {

            iter

        } else {

            return None;
        };

        if let Some(value) = iter.next() {

            return unsafe {value.as_mut::<'i>()};
        }

        self.iters.pop();

        let iter = if let Some(iter) = self.iters.last_mut() {

            iter

        } else {

            return None;
        };

        if let Some(value) = iter.next() {

            unsafe {value.as_mut::<'i>()}
            
        } else {

            None
        }
    }
}

/// エラーです。
#[derive(Debug)]
pub enum Error {
    
    /// 型が一致しませんでした。
    MissingType, 
    /// 不正な引数です。
    ArgumentError,
}
impl Display for Error {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match self {

            Error::MissingType => f.write_str("型が一致しませんでした。"),
            Error::ArgumentError => f.write_str("不正な引数です。"),
        }
    }
}
impl std::error::Error for Error {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

// 不変参照データの実装です。
impl<'i, D> Archetype for &'i D 
where D: Data {
    
    type Iter = Iter<'i, D>;

    fn request() -> Request {
        Request::new(vec![Type::of::<D>()])
    }

    fn iter(res: &Response) -> Self::Iter {
        
        Self::Iter::new(res)
    }

    fn values(&self) -> Datas {
        todo!()
    }
}
