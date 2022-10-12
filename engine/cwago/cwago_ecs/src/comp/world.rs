// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/sys/comp/world.rs
// (C) 2022 Taichi Ito.
// ====================

//! ワールドシステム機能を提供します。

use std::sync::{
    Arc, 
    Mutex
};
use crate::{ent::Id, err::Error};
use super::{
    chunk::Chunk, 
    data::Datas, 
    ty::Archetype
};

/// ワールドシステムです。
pub struct World {

    id_cnt: u32,
    ids: Vec<Option<IdInfo>>,
    chunks: Arc<Mutex<Vec<Chunk>>>,
    spawns: Arc<Mutex<Vec<Datas>>>, 
    despawn: Arc<Mutex<Vec<Id>>>
}
impl World {
    
    /// 生成します。
    pub fn new() -> Self {

        World { 
            id_cnt: 0,
            ids: Vec::new(), 
            chunks: Arc::new(Mutex::new(Vec::new())), 
            spawns: Arc::new(Mutex::new(Vec::new())), 
            despawn: Arc::new(Mutex::new(Vec::new())) 
        }
    }

    /// エンティティを生成します。
    /// 
    /// # Arguments
    /// 
    /// * `args` - コンポーネントの初期化値です。
    /// 
    pub fn spawn<A>(&mut self, args: A) -> Result<Id, Error> 
    where A: Archetype {
        let req = A::for_request();
    }
}

//+Result<Id, Error> spawn<A:Archetype>(&mut self, A args)
//+Result<(), Error> despawn(&mut self, &Id id)
//+Result<Query, Error> query<A:Archetype>(&mut self)

/// Idと関連するデータの位置の記録です。
struct IdInfo {

    /// チャンクの位置です。
    chunk_idx: usize,

    /// データの位置です。
    data_idx: usize
}
impl IdInfo {
    
    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `chunk_idx` - チャンクの位置です。
    /// * `data_idx` - データの位置です。
    /// 
    fn new(chunk_idx: usize, data_idx: usize) -> Self{

        IdInfo { chunk_idx, data_idx }
    }
}