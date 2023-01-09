// -------------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/comp/data.rs
// (C) 2022 CwagoCommunity.
//
//! コンポーネントデータを提供します。
// =========================

use serde::{
    Serialize, 
    Deserialize
};



/// コンポーネントデータトレイトです。
pub trait Data<'de>: Clone + Serialize + Deserialize<'de> + 'static {
    
}