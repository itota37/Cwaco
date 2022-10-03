// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/component/data.rs
// (C) 2022 Taichi Ito.
// ====================

//! Component-Dataを提供します。

use std::{
    collections::VecDeque, 
    sync::{
        Arc, 
        atomic::AtomicBool
    },
    fmt::Display
};
use cwago_utility::hash::FxHashMap;
use crate::ent::Id;
use super::ty::{
    self,
    Request, 
    Type,
    Datas
};

/// Componentのデータを定義するトレイトです。
pub trait Data: Clone + 'static {}

/// Idと関連データの集合です。
pub(crate) struct Chunk {

    ids: Vec<Id>,                     // Entity-Idのリストです。
    datas: FxHashMap<Type, DataInfo>, // 型別データ情報です。
}
impl Chunk {

    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `req` - 生成するリクエストです。
    /// 
    pub(crate) fn new(req: &Request) -> Self {

        // 空のIdリスト、型ごとにバッファが用意された状態で生成します。
        
        let ids = Vec::new();
        let mut datas = FxHashMap::default();
        for ty in req.get() {

            datas.insert(ty.clone(), DataInfo::new(ty.size()));
        }

        Chunk { ids, datas }
    }
    
    /// リクエストと一致するか判定します。
    /// 
    /// # Arguments
    /// 
    /// * `req` - 判定するリクエストです。
    /// 
    pub(crate) fn match_req(&self, req: &Request) -> bool {
        
        let tys = req.get();
        
        // 長さを判定します。
        if self.datas.len() != tys.len() { return false; }
        
        // 各要素を判定します。
        for ty in tys {
            
            if !self.datas.contains_key(ty) { return false; }
        }
        
        true
    }

    /// Idにデータをアタッチします。
    /// 
    /// # Arguments
    /// 
    /// * `id` - データと関連付けるIdです。
    /// * `args` - 初期化データリストです。
    /// 
    pub(crate) fn attach(&mut self, id: &Id, args: &Datas) -> Result<usize, Error>{
        
        let req = &args.request();

        // 型が一致しているか判定します。
        if !self.match_req(req) { return Err(Error::TypeError(ty::Error::MissingType)); }
        
        // 追加位置を取得し、新たなIdを追加します。
        let idx = self.ids.len();
        self.ids.push(id.clone());
        
        // 各データの追加と初期化をします。
        for ty in req.get() {
            
            let from = args
            .get(ty)
            .expect("重大なエラーが発生しました。Chunk/spawn/0"); // このエラーが発生した場合、match_reqが機能していない場合があります。

            // 初期化するメモリの位置を取得します。
            let to = self.datas
            .get_mut(ty)
            .expect("重大なエラーが発生しました。Chunk/spawn/1") // このエラーが発生した場合、match_reqが機能していない場合があります。
            .push();
            
            // 初期化値を代入します。
            ty.copy_ptr(from, to);
        }
        
        Ok(idx)
    } 
    
    /// Idとデータをデタッチします。
    /// 
    /// # Arguments
    /// 
    /// * `idx` - 位置です。
    /// 
    pub(crate) fn dettach(&mut self, idx: usize) -> Result<(), Error> {
        
        for (ty, info) in self.datas.iter_mut() {
            
            if let (Some(to), Some(from)) = (info.get_mut(idx), info.last_mut()) {
                
                if to == from {
                    
                    ty.drop_ptr(to);
                    
                } else {
                    
                    ty.drop_ptr(to);
                    ty.copy_ptr(from, to);
                    ty.drop_ptr(from);
                }
                
            } else {
                
                return Err(Error::IndexOverflow);
            }
        }
        
        Ok(())
    }
    
    /// 指定位置のデータを取得します。
    /// 
    /// # Arguments
    /// 
    /// * `idx` - 位置です。
    /// 
    pub(crate) fn datas(&self, idx: usize) -> Result<Datas, Error> {
        
        let mut datas = FxHashMap::default();
        for (ty, info) in self.datas.iter() {
            
            if let Some(ptr) = info.get(idx) {
                
                datas.insert(*ty, ptr);
                
            } else {
                
                return Err(Error::IndexOverflow);
            }
        }
        
        Ok(Datas::new(datas))
    }
}

/// 1つの型の生配列情報です。
struct DataInfo {

    buffer: Vec<u8>,                  // 生データです。
    size: usize,                      // 型のサイズです。
    flag: AccessFlag,                 // 状態フラグです。
    waits: VecDeque<Arc<AtomicBool>>, // アクセス待ちワーカーの待機フラグキューです。
}
impl DataInfo {
    
    /// 生成します。
    ///
    /// # Arguments
    ///
    /// * `size` - 型のサイズです。
    ///
    fn new(size: usize) -> Self {

        DataInfo { 
            buffer: Vec::new(), 
            size,
            flag: AccessFlag::Free, 
            waits: VecDeque::new() 
        }
    }
    
    /// 領域を追加します。
    fn push(&mut self) -> *mut u8 {
        
        // 追加位置の先頭位置を取得します。
        let idx = self.buffer.len();
        // 追加します。
        for _ in 0..self.size {
            
            self.buffer.push(0_u8);
        }
        
        // 追加位置のポインタを返します。
        unsafe {
            self.buffer
            .as_mut_ptr()
            .add(idx)
        }
    }
    
    /// 指定位置の不変ポインタを取得します。
    ///
    /// # Arguments
    ///
    /// * `idx` - 指定の位置です。
    ///
    fn get(&self, idx: usize) -> Option<*const u8> {
        
        let idx = self.size * idx;
        if idx < self.buffer.len() {
            
            Some(unsafe {
                self.buffer
                .as_ptr()
                .add(idx)
            })
            
        } else {
        
            None
        }
    }
    
    /// 指定位置の可変ポインタを取得します。
    ///
    /// # Arguments
    ///
    /// * `idx` - 指定の位置です。
    ///
    fn get_mut(&mut self, idx: usize) -> Option<*mut u8> {
        
        let idx = self.size * idx;
        if idx < self.buffer.len() {
            
            Some(unsafe {
                self.buffer
                .as_mut_ptr()
                .add(idx)
            })
            
        } else {
        
            None
        }
    }
    
    /// 末尾の要素の不変ポインタを取得します。
    fn last_mut(&mut self) -> Option<*mut u8> {
    
        if self.buffer.is_empty() 
        && self.buffer.len() < self.size {
            
            None
            
        } else {
            
            let idx = self.buffer.len() - self.size;
            Some(unsafe {
                self.buffer
                .as_mut_ptr()
                .add(idx)
            })
        }
    }
}

/// 現在のアクセス状態です。
enum AccessFlag {

    Free,         // どこからも参照されていません。
    Const(usize), // 複数から不変参照されています。
    Mut,          // 1つの可変参照されています。
}

/// エラーです。
#[derive(Debug)]
pub enum Error {
    
    /// 配列の要素数を超えています。
    IndexOverflow, 
    /// 型情報に起因するエラーです。
    TypeError(ty::Error),
}
impl Display for Error {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match self {

            Error::IndexOverflow => f.write_str("配列の要素数を超えています。"),
            Error::TypeError(err) => f.write_fmt(format_args!("型情報に起因するエラーです。>> {}", err.to_string())),
        }
    }
}
impl std::error::Error for Error {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        
        match self {
            
            Error::TypeError(err) => Some(err),
            _ => None
        }
    }
}
