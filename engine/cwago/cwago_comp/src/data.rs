// -------------------------
//
// Cwago.
//
// cwago/cwago_comp/src/data.rs
// (C) 2023 CwagoCommunity.
//
//! コンポーネントデータを提供します。
// =========================

pub use serde::{
    Serialize, 
    Deserialize
};

use crate::ty::Info;

/// コンポーネントデータトレイトです。
pub trait Data<'de>: Sized + Serialize + Deserialize<'de> + 'static {
    fn info() -> &'static Info;
}