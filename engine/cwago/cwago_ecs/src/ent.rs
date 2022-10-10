// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/ent.rs
// (C) 2022 Taichi Ito.
// ====================

//! ECSエンティティを提供します。

/// エンティティIDです。
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
}