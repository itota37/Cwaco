// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/ent.rs
// (C) 2022 Taichi Ito.
// ====================

//! ECSエンティティを提供します。

use std::future::Future;
use cwago_utility::hash::{FxHashSet, FxHashMap};
use crate::{
    comp::{
        World,
        Value, Data
    },
    ty::{
        Archetype, 
        Type, 
        RefType,
    }, 
    err::Error
};

/// エンティティIDです。
#[derive(Debug, Clone, Copy)]
pub struct Id {

    /// インデクスです。
    idx: u32,

    /// バージョンです。
    ver: u32,
}
impl Id {

    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `idx` - インデクスです。
    /// * `ver` - バージョンです。
    ///
    pub(crate) fn new(idx: u32, ver: u32) -> Self {

        Id { idx, ver }
    }

    /// インデクスを取得します。
    pub(crate) fn index(&self) -> usize {
        self.idx as usize
    }

    /// バージョンを取得します。
    pub(crate) fn version(&self) -> usize {
        self.ver as usize
    }
}

/// エンティティシステムです。
pub struct Entity<'w> {

    world: &'w mut World,
    values: FxHashMap<Type, Value>,
    types: FxHashSet<RefType>,
}
impl<'w> Entity<'w> {
    
    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `world` - idを管理しているワールドシステムです。
    /// 
    pub(crate) fn new(world: &'w mut World) -> Self {
        Entity {
            world,
            values: FxHashMap::default(),
            types: FxHashSet::default()
        }
    }

    /// 型と初期値を設定します。
    /// 
    /// # Argument
    /// 
    /// * `arg` - 初期値です。
    /// 
    pub fn insert<D>(self, arg: D) -> Self
    where D: Data {
        self.values.insert(Type::of::<D>(), Value::from_data(arg));
        self
    }

    /// 対象のIdを設定します。
    /// 
    /// # Argument
    /// 
    /// * `id` - 対象のIdです。
    /// 
    pub fn to(id: Id) {

    }
}