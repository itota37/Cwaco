// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/comp.rs
// (C) 2022 Taichi Ito.
// ====================

//! コンポーネントシステムを提供します。

use std::{
    sync::{
        Arc, 
        Mutex
    }, 
    future::Future
};
use cwago_utility::hash::{
    FxHashSet, 
    FxHashMap
};

use crate::{
    ent::{
        Id, 
        Entity, 
    }, 
    err::Error, ty::{RefType, Type}
};

/// ワールドシステムです。
pub struct World {

    id_cnt: u32,
    points: Vec<Option<Point>>,
    chunks: Arc<Mutex<Vec<Chunk>>>,
    spawns: Arc<Mutex<Vec<SpawnResult>>>, 
    despawn: Arc<Mutex<Vec<Id>>>
}
impl World {
    
    /// 生成します。
    pub fn new() -> Self {

        World { 
            id_cnt: 0,
            points: Vec::new(), 
            chunks: Arc::new(Mutex::new(Vec::new())), 
            spawns: Arc::new(Mutex::new(Vec::new())), 
            despawn: Arc::new(Mutex::new(Vec::new())) 
        }
    }

    /// エンティティを生成します。
    /*pub fn spawn(&mut self) -> Future<Result<Id, Error>> {
    }*/

    /// エンティティを削除します。
    /// 
    /// # Arguments 
    /// 
    /// * `id` - 削除するエンティティIDです。
    /// 
    /*pub fn despawn(&mut self, id: &Id) -> Future<Result<(), Error>> {

    }*/

    /// エンティティシステムを取得します。
    /// 
    /// # Arguments
    /// 
    /// * `id` - 操作対象のエンティティIDです。
    /// 
    pub fn entity(&mut self, id: Id) -> Entity {
        Entity::new(id, self)
    }

    /// コンポーネントシステムを取得します。
    /*pub fn component(&mut self) -> Component {

    }*/

    pub(crate) fn attach(&mut self, id: Id, types: FxHashSet<RefType>) -> Result<Id, Error> {
        
        match self.point(id) {
            Some(poi) => {
                // 移動
            },
            None => {
                // 新規作成
            },
        };


    }

    fn point(&self, id: Id) -> Option<Point> {
        
        if let Some(poi) = self.points.get(id.index()) {
            return *poi;
        }

        None
    }
}

/// コンポーネントデータの集合です。
struct Chunk {
    ids: Vec<Id>,
    datas: FxHashMap<Type, *mut u8>,
}

struct SpawnResult {

}
impl Future for SpawnResult {
    type Output = Result<Id, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        todo!()
    }
}

/// Idと関連するデータの位置の記録です。
#[derive(Clone, Copy)]
struct Point {

    /// チャンクの位置です。
    chunk_idx: usize,

    /// データの位置です。
    data_idx: usize
}
impl Point {
    
    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `chunk_idx` - チャンクの位置です。
    /// * `data_idx` - データの位置です。
    /// 
    fn new(chunk_idx: usize, data_idx: usize) -> Self{

        Point { chunk_idx, data_idx }
    }
}