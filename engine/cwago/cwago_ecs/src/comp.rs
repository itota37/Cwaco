// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/comp.rs
// (C) 2022 Taichi Ito.
// ====================

//! コンポーネントシステムを提供します。

// 目次
// --------------------
// Data
// World
// Components
// Point
// Chunk
// Value
// SpawnRes
// DespawnRes
// ====================

use std::{
    sync::{
        Arc, 
        Mutex
    }, 
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll
    }, mem::transmute
};
use cwago_utility::hash::{
    FxHashSet, 
    FxHashMap
};

use crate::{
    ent::{
        Id, 
        Entity, UpdateInfo, 
    }, 
    err::Error, ty::{RefType, Type}
};



// --------------------
//
// Data
//
// ====================



// コンポーネントデータを定義するトレイトです。
pub trait Data: Clone + 'static {
}



// --------------------
//
// World
//
// ====================



/// ワールドシステムです。
pub struct World {

    id_cnt: u32,                                   // Id.indexのカウントです。
    points: Vec<Option<Point>>,                    // Id.indexを添え字にしたデータ位置情報の配列です。
    chunks: Arc<Mutex<Vec<Chunk>>>,                // チャンク配列です。
    spawns: Arc<Mutex<Vec<SpawnRes>>>,             // Id生成フューチャーの配列です。
    despawns: Arc<Mutex<Vec<DespawnRes>>>,         // Id削除フューチャーの配列です。
    updates: Arc<Mutex<FxHashMap<Id, UpdateInfo>>> // Idに対するコンポーネント変更情報です。
}
impl World {
    
    /// 生成します。
    pub fn new() -> Self {

        World { 
            id_cnt: 0,
            points: Vec::new(), 
            chunks: Arc::new(Mutex::new(Vec::new())), 
            spawns: Arc::new(Mutex::new(Vec::new())), 
            despawns: Arc::new(Mutex::new(Vec::new())),
            updates: Arc::new(Mutex::new(FxHashMap::default()))
        }
    }

    /// エンティティを生成します。
    pub fn spawn(&mut self) -> impl Future<Output = Result<Id, Error>> {
        let arc = Arc::new(Mutex::new(Poll::Pending));
        let res = SpawnRes::new(arc);
        self.spawns
        .lock()
        .expect("重大なエラーが発生しました。[cwago_ecs/src/comp.rs : World::spawn/0]")
        .push(res.clone());
        res
    }

    /// エンティティを削除します。
    /// 
    /// # Arguments 
    /// 
    /// * `id` - 削除するエンティティIDです。
    /// 
    pub fn despawn(&mut self, id: &Id) -> impl Future<Output = Result<(), Error>> {
        let arc = Arc::new(Mutex::new(Poll::Pending));
        let res = DespawnRes::new(arc);
        self.despawns
        .lock()
        .expect("重大なエラーが発生しました。[cwago_ecs/src/comp.rs : World::despawn/0]")
        .push(res.clone());
        res
    }

    /// エンティティシステムを取得します。
    pub fn entity(&mut self) -> Entity {
        Entity::new(self)
    }

    /// コンポーネントシステムを取得します。
    pub fn components(&mut self) -> Components {
        Components::new(self)
    }

    /// エンティティの情報をスタックします。
    /// 
    /// # Arguments
    /// 
    /// * `id` - 更新するIdです。
    /// * `info` - 更新情報です。
    /// 
    pub(crate) fn push_entity_info(&mut self, id: Id, info: UpdateInfo) {
        self.updates
        .lock()
        .expect("重大なエラーが発生しました。[cwago_ecs/src/comp.rs : World::push_entity_info/0]")
        .insert(id, info);
    }
}



// --------------------
//
// Components
//
// ====================



/// コンポーネントシステムです。
pub struct Components<'w> {
    world: &'w mut World,
}
impl<'w> Components<'w> {
    
    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `world` - idを管理しているワールドシステムです。
    /// 
    fn new(world: &'w mut World) -> Self {
        Components { world }
    }
}



// --------------------
//
// Point
//
// ====================



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



// --------------------
//
// Chunk
//
// ====================



/// コンポーネントデータの集合です。
struct Chunk {
    ids: Vec<Id>,
    datas: FxHashMap<Type, Vec<u8>>,
}
impl Chunk {
    
    /// 指定位置のデータマップを取得します。
    /// 
    /// # Arguments
    /// 
    /// * `point` - 指定位置です。
    /// 
    fn values(&self, point: usize) -> Option<FxHashMap<Type, Value>> {
        // 値が範囲外の場合Noneを返します。
        if self.ids.len() <= point { return None; }
        // データをマップにクローンします。
        let mut map = FxHashMap::default();
        for (ty, vec) in self.datas {
            let ptr = vec.as_ptr();
            let ptr = unsafe { ptr.add(ty.size() * point) };
            let value = Value::from_ptr(&ty, ptr);
            map.insert(ty, value);
        }
        Some(map)
    }

    /// データを挿入します。
    /// 
    /// # Arguments
    /// 
    /// * `id` - 挿入するIdです。
    /// * `values` - 挿入するデータのマップです。
    /// 
    fn insert(&mut self, id: Id, values: FxHashMap<Type, Value>) -> Result<usize, Error> {
        
    }

    /// 指定位置のデータを削除します。
    /// 
    /// # Arguments
    /// 
    /// * `point` - 指定位置です。
    /// 
    fn remove(&mut self, point: usize) -> Result<(), Error> {
        // 値が範囲外の場合Noneを返します。
        if self.ids.len() <= point { 
            return Err(Error::InnerError("重大なエラーが発生しました。指定の位置にデータはありません。[cwago_ecs/src/comp.rs : Chunk::remove/0".to_string())); 
        }
        // 末尾の位置を所得します。
        let last_point = self.ids.len() - 1;
        // ラストのデータをマップにクローンします。
        let mut map = FxHashMap::default();
        for (ty, vec) in self.datas {
            let ptr = vec.as_ptr();
            let ptr = unsafe { ptr.add(ty.size() * last_point) };
            let value = Value::from_ptr(&ty, ptr);
            map.insert(ty, value);
        }
        // 指定位置のデータをドロップします。
        for (ty, vec) in &mut self.datas {
            let ptr = vec.as_mut_ptr();
            let ptr = unsafe { ptr.add(ty.size() * point) };
            ty.drop_ptr(ptr);
        }
        // リストのデータを指定位置にクローンします。
        for (ty, value) in map {
            let vec = self.datas.get_mut(&ty)
            .expect("重大なエラーが発生しました。[cwago_ecs/src/comp.rs : Chunk::remove/1");
            let ptr = vec.as_mut_ptr();
            let ptr = unsafe { ptr.add(ty.size() * point) };
            ty.clone_ptr(value.ptr(), ptr);
        }
        // ラストのデータをドロップします。
        for (ty, vec) in &mut self.datas {
            let ptr = vec.as_mut_ptr();
            let ptr = unsafe { ptr.add(ty.size() * last_point) };
            ty.drop_ptr(ptr);
        }
        // 末尾を削除します。
        for (ty, vec) in &mut self.datas {
            for i in 0..ty.size() {
                vec.pop();
            }
        }
        // Idを削除します。
        self.ids.remove(point);

        Ok(())
    }
}



// --------------------
//
// Value
//
// ====================



/// Dataの一時コピーです。
#[derive(Debug)]
pub struct Value {
    value: Vec<u8>
}
impl Value {
    
    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `data` - コピーするデータです。
    /// 
    pub(crate) fn from_data<D>(data: D) -> Self
    where D: Data {
        let value = vec![0_u8; Type::of::<D>().size()];
        unsafe {
            let ptr = transmute::<*mut u8, *mut D>(value.as_mut_ptr());
            let tmp = ptr.as_mut()
            .expect("重大なエラーが発生しました。[cwago_ecs/src/comp.rs : Value::from_data]");
            *tmp = data.clone();
        }
        Value { value }
    }

    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `ty` - 型情報です。
    /// * `ptr` - コピーするデータポインタです。
    /// 
    pub(crate) fn from_ptr(ty: &Type, ptr: *const u8) -> Self {
        let value = vec![0_u8; ty.size()];
        ty.clone_ptr(ptr, value.as_mut_ptr());
        Value { value }
    }

    /// ポインタを取得します。
    pub(crate) fn ptr(&self) -> *const u8 {
        self.value.as_ptr()
    }
}



// --------------------
//
// SpawnRes
//
// ====================



/// Id遅延生成のための構造体です。
#[derive(Debug, Clone)]
struct SpawnRes {
    arc: Arc<Mutex<Poll<Result<Id, Error>>>>
}
impl SpawnRes {

    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `arc` - 内容です。
    /// 
    fn new(arc: Arc<Mutex<Poll<Result<Id, Error>>>>) -> Self {
        Self { arc }
    }
}
impl Future for SpawnRes {

    type Output = Result<Id, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        *self.arc
        .lock()
        .expect("重大なエラーが発生しました。[cwago_ecs/src/comp.rs : SpawnRes::poll/0]")
    }
}



// --------------------
//
// DespawnRes
//
// ====================



/// Id遅延削除のための構造体です。
#[derive(Debug, Clone)]
struct DespawnRes {
    arc: Arc<Mutex<Poll<Result<(), Error>>>>
}
impl DespawnRes {

    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `arc` - 内容です。
    /// 
    fn new(arc: Arc<Mutex<Poll<Result<(), Error>>>>) -> Self {
        Self { arc }
    }
}
impl Future for DespawnRes {

    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        *self.arc
        .lock()
        .expect("重大なエラーが発生しました。[cwago_ecs/src/comp.rs : DespawnRes::poll/0]")
    }
}