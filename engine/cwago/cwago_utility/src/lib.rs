// -------------------------
//
// Cwago.
//
// cwago/cwago_utility/src/lib.rs
// (C) 2022 CwagoCommunity.
//
//! cwago_utilityライブラリのメインファイルです。
// =========================

pub mod mem;
pub mod plu;

#[global_allocator]
static ALLOCATOR: mem::Allocator = mem::Allocator::new();