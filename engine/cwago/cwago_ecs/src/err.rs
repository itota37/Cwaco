// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/err.rs
// (C) 2022 Taichi Ito.
// ====================

//! ECSエラーを提供します。

use std::{error, fmt::Display};

/// エラー情報です。
#[derive(Debug)]
pub enum Error {

    /// 同時に存在可能なエンティティIDの数を超過しました。
    IdOverflow,

    /// 無効なエンティティIDです。
    InvalidId,

    /// 指定された型が重複しています。
    DuplicateType,

    /// 型が一致していません。
    MissmatchType,

    /// デッドロックしています。
    /// 同スレッドで同データにアクセスしようとした可能性があります。
    DeadLock,
}
impl error::Error for Error {

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
impl Display for Error {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match self {
            Error::IdOverflow => f.write_str("同時に存在可能なエンティティIDの数を超過しました。"),
            Error::InvalidId => f.write_str("無効なエンティティIDです。"),
            Error::DuplicateType => f.write_str("指定された型が重複しています。"),
            Error::MissmatchType => f.write_str("型が一致していません。"),
            Error::DeadLock => f.write_str("デッドロックしています。"),
        }
    }
}