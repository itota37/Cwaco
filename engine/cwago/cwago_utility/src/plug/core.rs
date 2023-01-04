// -------------------------
//
// Cwago.
//
// cwago/cwago_utility/src/plug/core.rs
// (C) 2023 CwagoCommunity.
//
//! プラグインコアシステムを提供します。
// =========================

use std::path::Path;

use crate::hash::FxHashMap;

/// プラグインマネージャです。
pub(crate) struct Plugin {
    libs: FxHashMap<&'static str, libloading::Library>
}
impl Plugin {
    /// パスを指定してプラグインを読込みます。
    /// 
    /// # 引数
    /// 
    /// * path - プラグインファイルへのパスです。
    /// 
    /// # 戻り値
    /// 
    /// 読み込みの成否論理値です。
    /// 
    pub(crate) fn load(path: &Path) -> bool {
        
    }
}
