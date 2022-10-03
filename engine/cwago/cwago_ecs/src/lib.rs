// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/lib.rs
// (C) 2022 Taichi Ito.
// ====================

//! ECS基本機能を提供します。

pub mod ent;
pub mod comp;
pub mod sys;
pub mod ecs;

//
// やること
//
// ECS機能作成
// System-EventからのみComponent-Iterが取得できるようにする
//
// ECS.despawnを作る
//