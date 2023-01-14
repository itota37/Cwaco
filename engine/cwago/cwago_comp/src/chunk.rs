// -------------------------
//
// Cwago.
//
// cwago/cwago_comp/src/chunk.rs
// (C) 2023 CwagoCommunity.
//
//! チャンクを提供します。
// =========================

use std::any::TypeId;

use cwago_utility::hash::FxHashMap;

use crate::{
    ent::Id, 
    ty::Info
};

/// エンティティとデータの集まりです。
pub struct Chunk {
    ids: Vec<Id>,
    datas: FxHashMap<TypeId, Datas>,
}
struct Datas {
    info: &'static Info,
    buf: Vec<u8>,
}