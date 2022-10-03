// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/ent.rs
// (C) 2022 Taichi Ito.
// ====================

//! ECS-Entity機能を提供します。

/// EntityのIdです。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id {

    index: u32,   // Entity配列のインデクスです。
    version: u32, // 再利用前と後を区別するためのバージョンです。
}
impl Id {
    
    /// 新規作成します。
    /// 
    /// # Arguments
    /// 
    /// * `index` - インデクスです。
    /// 
    pub(crate) fn new(index: u32) -> Self {

        Id { index, version: 0_u32 }
    }

    /// 再利用のためにバージョンをインクリメントして返します。
    pub(crate) fn increment(&self) -> Self {

        let index = self.index;
        let version = if self.version == u32::MAX {
            0
        } else {
            self.version + 1
        };
        Id { index, version }
    }

    /// インデクスを取得します。
    pub(crate) fn index(&self) -> usize {

        self.index as usize
    }

}
impl PartialOrd for Id {
    
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {

        self.index.partial_cmp(&other.index)
    }
}
impl Ord for Id {

    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        
        self.index.cmp(&other.index)
    }
}