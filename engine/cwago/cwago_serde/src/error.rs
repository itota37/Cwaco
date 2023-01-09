// -------------------------
//
// Cwago.
//
// cwago/cwago_serde/src/error.rs
// (C) 2022 CwagoCommunity.
//
//! エラーを提供します。
// =========================

use std::{
    fmt::Display, 
    error
};

use serde;

/// Errorの型です。
#[derive(Debug)]
pub(crate) struct Error {
    msg: String,
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg.as_str())
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
impl serde::ser::Error for Error {
    fn custom<T>(msg:T) -> Self where T:Display {
        Error { msg: msg.to_string() }
    }
}