// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/sys/comp/chunk.rs
// (C) 2022 Taichi Ito.
// ====================

//! データのチャンクを提供します。

use std::{
    sync::{
        Arc, 
        Mutex
    }, 
    collections::VecDeque, 
    thread::Thread
};
use cwago_utility::hash::FxHashMap;
use super::ty::Type;

/// データのチャンクです。
pub(crate) struct Chunk {

    buffs: Arc<Mutex<FxHashMap<Type, Buffer>>>
}

//~Result<(), Error> attach(&mut self, &Id id, Datas args)
//~Result<Datas, Error> dettach(&mut self, &Id id)
//~Result<HashMap<Type, PointerType>, Error> request<A:Archetype>(&mut self)

/// 1つの型に対応するデータの配列です。
struct Buffer {

    /// データの配列です。
    buff: Vec<u8>,

    /// このデータのアクセス情報です。
    acc: AccessInfo,
}

/// アクセス情報です。
struct AccessInfo {

    /// アクセス状況です。
    state: AccessState,

    /// アクセス待ちスレッド行列です。
    waits: VecDeque<Thread>,
}

/// アクセス状況です。
enum AccessState {
    
    /// ロックは掛かっていません。
    Free,

    /// 単一、または、複数のスレッドから共有ロックが掛かっています。
    Const(usize),

    /// 単一のスレッドから占有ロックが掛かっています。
    Mut,
}