// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/ent.rs
// (C) 2022 Taichi Ito.
// ====================

//! ECSエンティティを提供します。

use std::future::Future;

use cwago_utility::hash::FxHashSet;
use crate::{
    comp::World,
    ty::{
        Archetype, 
        Type, RefType
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

    id: Id,
    world: &'w mut World,
    types: FxHashSet<RefType>,
}
impl<'w> Entity<'w> {
    
    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `id` - 操作対象のエンティティIDです。
    /// * `world` - idを管理しているワールドシステムです。
    /// 
    pub(crate) fn new(id: Id, world: &'w mut World) -> Self {
        Entity {
            id,
            world,
            types: FxHashSet::default()
        }
    }

    /// 型指定します。
    /// 
    /// # Argument
    /// 
    /// * `arg` - 初期値です。
    /// 
    pub fn request<A>(self, arg: A) -> Self
    where A: Archetype {
        self.types.insert(A::for_type());
        self
    }

    // アタッチします。
    pub fn attach(self) -> Result<Id, Error> {
        self.world.attach(self.id, self.types)
    }
}