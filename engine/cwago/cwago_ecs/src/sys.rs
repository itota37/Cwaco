// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/sys.rs
// (C) 2022 Taichi Ito.
// ====================

//! ECS-System機能を提供します。

use crate::comp::ty::Archetype;

/// イベントトレイトです。
pub trait Event<A>
where A: Arguments {
    
    /// イベントに関数を追加します。
    /// 
    /// # Arguments
    /// 
    /// * `func` - 追加する関数です。
    /// 
    fn push(func: A::Signature);

    /// イベントから関数を削除します。
    /// 
    /// # Arguments
    /// 
    /// * `func` - 削除する関数です。
    /// 
    fn pop(func: A::Signature);

    /// イベントを実行します。
    /// 
    /// # Arguments
    /// 
    /// * `args` - 実行する関数の引数に渡す引数です。
    /// 
    fn run(args: A);
}

/// イベント処理でコンポーネントデータを取得するための構造体です。
#[derive(Debug, Clone)]
pub struct Query<A> 
where A: Archetype {
    iter: A::Iter,
}
impl<A> Iterator for Query<A> 
where A: Archetype {
    
    type Item = <A::Iter as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        
        self.iter.next()
    }
}

/// イベント関数の引数の並びを定義するトレイトです。
pub trait Arguments: Clone {

    /// 対応する関数のシグネチャです。
    type Signature;
}

/// イベント関数の引数を定義するトレイトです。
pub trait Argument {}

impl<A> Argument for Query<A> 
where A: Archetype {}