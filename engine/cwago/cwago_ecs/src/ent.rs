// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/ent.rs
// (C) 2022 Taichi Ito.
// ====================

//! ECSエンティティを提供します。

use std::{future::Future, hash::Hash, fmt::Display};
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
impl Display for Id {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Id({})", self.index()))
    }
}
impl PartialEq for Id {
    
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index()
        && self.version() == other.version()
    }
}
impl Eq for Id {}
impl PartialOrd for Id {
    
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index().partial_cmp(&other.index())
    }
}
impl Ord for Id {
    
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index().cmp(&other.index())
    }
}
impl Hash for Id {
    
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index().hash(state);
    }
}

/// エンティティシステムです。
pub struct Entity<'w> {

    world: &'w mut World,
    info: UpdateInfo
}
pub(crate) struct UpdateInfo {
    inserts: FxHashMap<Type, Value>,
    removes: FxHashSet<Type>,
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
            info: UpdateInfo { 
                inserts: FxHashMap::default(),
                removes: FxHashSet::default()
            }
        }
    }

    /// コンポーネントデータを追加します。
    /// 
    /// # Arguments
    /// 
    /// * `arg` - 初期値です。
    /// 
    pub fn insert<D>(self, arg: D) -> Self
    where D: Data {
        let ty = Type::of::<D>();
        // 削除対象から外します。
        if self.info.removes.contains(&ty) {
            self.info.removes.remove(&ty);
        }
        // 追加対象に設定、または、上書きします。
        self.info.inserts.insert(
            ty, 
            Value::from_data(arg)
        );
        self
    }

    /// コンポーネントデータを削除します。
    pub fn remove<D>(self) -> Self
    where D: Data {
        let ty = Type::of::<D>();
        // 追加対象から外します。
        if self.info.inserts.contains_key(&ty) {
            self.info.inserts.remove(&ty);
        }
        // 削除対象に設定します。
        self.info.removes.insert(ty);
        self
    }

    /// 対象のIdを設定します。
    /// 
    /// # Argument
    /// 
    /// * `id` - 対象のIdです。
    /// 
    pub fn to(self, id: Id) {
        self.world.push_entity_info(id, self.info);
    }
}