// -------------------------
//
// Cwago.
//
// cwago/cwago_comp/src/ent.rs
// (C) 2023 CwagoCommunity.
//
//! エンティティを提供します。
// =========================

use std::{
    fmt::{
        Display, 
        Debug
    }
};

/// 同時に存在可能なエンティティの最大値です。
pub const EXISTENCES_MAX: usize = u32::MAX as usize;

/// エンティティIDです。
#[derive(Debug, Clone, Copy)]
pub struct Id {
    idx: u32, // エンティティ配列の添え字です。
    ver: u32, // idxを使いまわす際の重複を回避する世代値です。
}
impl Id {
    /// Idを作成します。
    /// 
    /// # 引数
    /// 
    /// * `idx` - Id配列の添え字です。
    /// * `ver` - 世代値です。
    /// 
    /// # 戻り値
    /// 
    /// Idです。
    /// 
    pub(super) fn new(idx: u32, ver: u32) -> Id {
        Id { idx, ver }
    }
    
}
impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let h = (self.idx as usize) << 32;
        let id = h | (self.ver as usize); 
        write!(f, "{}", id)
    }
}