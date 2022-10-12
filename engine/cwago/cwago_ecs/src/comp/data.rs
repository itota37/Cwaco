// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/sys/comp/data.rs
// (C) 2022 Taichi Ito.
// ====================

//! コンポーネントデータを定義するトレイトと関連機能を提供します。

use std::{
    marker::PhantomData, 
    mem::transmute
};
use crate::err::Error;
use super::ty::{
    Type, 
    Buffers, 
    BufferPattern, 
    Archetype,
    Request,
    TypePattern
};

/// コンポーネントデータを定義するトレイトです。
pub trait Data: Clone + 'static {}

/// データを一時保存する構造体です。
pub(crate) struct Datas {

    /// 型別のデータバッファです。
    datas: Vec<(Type, Vec<u8>)>,
}

/// 不変参照によるArchitypeの実装です。
impl<'i, D> Archetype for &'i D 
where D: Data {
    
    type Iter = DataIter<'i, D>;

    fn for_request() -> Request {
        Request::from_pattern(TypePattern::Const(Type::of::<D>()))
    }

    fn for_iter(buff: &Buffers) -> Result<Self::Iter, Error> {
        Self::Iter::new(buff)
    }
}
/// 不変コンポーネントデータイテレータです。
pub struct DataIter<'i, D> 
where D: Data {

    _dummy: PhantomData<&'i D>,
    buffs: Vec<(*const u8, usize)>,
    step: usize,
    buff_idx: usize,
    data_idx: usize,
}
impl<'i, D> DataIter<'i, D> 
where D: Data {
    
    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `buffs` - 型別バッファリストです。
    /// 
    fn new(buffs: &Buffers) -> Result<Self, Error> {

        let ty = Type::of::<D>();
        if let Some(pattern) = buffs.get(&ty) {
            if let BufferPattern::Const(infos) = pattern {
                let mut buffs = Vec::new();
                for info in infos {
                    buffs.push((info.buffer(), info.len()));
                }

                return Ok(DataIter {
                    _dummy: PhantomData::default(),
                    buffs,
                    step: ty.size(),
                    buff_idx: 0,
                    data_idx: 0,
                });
            }
        }
        Err(Error::MissmatchType)
    }
}
impl<'i, D> Iterator for DataIter<'i, D> 
where D: Data {
    
    type Item = &'i D;

    fn next(&mut self) -> Option<Self::Item> {

        // ポインタを取得します。
        let (ptr, len) = self.buffs.get(self.buff_idx)?;
        let ptr = unsafe {
            let ptr = ptr.add(self.step * self.data_idx);
            transmute::<*const u8, *const D>(ptr)
        };

        // 一つ進めます。
        self.data_idx += 1;
        if self.data_idx == *len {
            self.data_idx = 0;
            self.buff_idx += 1;
        }

        unsafe { ptr.as_ref::<'i>() }
    }
}

/// 可変参照によるArchitypeの実装です。
impl<'i, D> Archetype for &'i mut D 
where D: Data {
    
    type Iter = DataIterMut<'i, D>;

    fn for_request() -> Request {
        Request::from_pattern(TypePattern::Mut(Type::of::<D>()))
    }

    fn for_iter(buff: &Buffers) -> Result<Self::Iter, Error> {
        Self::Iter::new(buff)
    }
}
/// 可変コンポーネントデータイテレータです。
pub struct DataIterMut<'i, D> 
where D: Data {

    _dummy: PhantomData<&'i mut D>,
    buffs: Vec<(*mut u8, usize)>,
    step: usize,
    buff_idx: usize,
    data_idx: usize,
}
impl<'i, D> DataIterMut<'i, D> 
where D: Data {
    
    /// 生成します。
    /// 
    /// # Arguments
    /// 
    /// * `buffs` - 型別バッファリストです。
    /// 
    fn new(buffs: &Buffers) -> Result<Self, Error> {

        let ty = Type::of::<D>();
        if let Some(pattern) = buffs.get(&ty) {
            if let BufferPattern::Mut(infos) = pattern {
                let mut buffs = Vec::new();
                for info in infos {
                    buffs.push((info.buffer(), info.len()));
                }

                return Ok(DataIterMut {
                    _dummy: PhantomData::default(),
                    buffs,
                    step: ty.size(),
                    buff_idx: 0,
                    data_idx: 0,
                });
            }
        }
        Err(Error::MissmatchType)
    }
}
impl<'i, D> Iterator for DataIterMut<'i, D> 
where D: Data {
    
    type Item = &'i mut D;

    fn next(&mut self) -> Option<Self::Item> {

        // ポインタを取得します。
        let (ptr, len) = self.buffs.get(self.buff_idx)?;
        let ptr = unsafe {
            let ptr = ptr.add(self.step * self.data_idx);
            transmute::<*mut u8, *mut D>(ptr)
        };

        // 一つ進めます。
        self.data_idx += 1;
        if self.data_idx == *len {
            self.data_idx = 0;
            self.buff_idx += 1;
        }

        unsafe { ptr.as_mut::<'i>() }
    }
}

// タプルイテレータとArchetypeのタプル実装です。
macro_rules! impl_Archetype_for_Tuple {
    ($i:tt|$($t:tt),+|$($n:tt),+) => {
        pub struct $i<$($t),+> 
        where $($t: Archetype),+ {
            iters: ($($t::Iter,)+)
        }
        impl<$($t),+> Iterator for $i<$($t),+> 
        where $($t: Archetype),+ {
            
            type Item = ($(<$t::Iter as Iterator>::Item,)+);

            fn next(&mut self) -> Option<Self::Item> {
                Some(($(self.iters.$n.next()?,)+))
            }
        }
        impl<$($t),+> Archetype for ($($t,)+) 
        where $($t: Archetype),+ {
            
            type Iter = $i<$($t),+>;

            fn for_request() -> Request {
                let mut req = Request::new();
                $(req.append(&$t::for_request());)+
                req
            }

            fn for_iter(buff: &Buffers) -> Result<Self::Iter, Error> {
                Ok($i {
                    iters: ($($t::for_iter(buff)?,)+)
                })
            }
        }
    };
}
impl_Archetype_for_Tuple!(TupleIter1|T0|0);
impl_Archetype_for_Tuple!(TupleIter2|T0, T1|0, 1);
impl_Archetype_for_Tuple!(TupleIter3|T0, T1, T2|0, 1, 2);
impl_Archetype_for_Tuple!(TupleIter4|T0, T1, T2, T3|0, 1, 2, 3);
impl_Archetype_for_Tuple!(TupleIter5|T0, T1, T2, T3, T4|0, 1, 2, 3, 4);
impl_Archetype_for_Tuple!(TupleIter6|T0, T1, T2, T3, T4, T5|0, 1, 2, 3, 4, 5);
impl_Archetype_for_Tuple!(TupleIter7|T0, T1, T2, T3, T4, T5, T6|0, 1, 2, 3, 4, 5, 6);
impl_Archetype_for_Tuple!(TupleIter8|T0, T1, T2, T3, T4, T5, T6, T7|0, 1, 2, 3, 4, 5, 6, 7);
impl_Archetype_for_Tuple!(TupleIter9|T0, T1, T2, T3, T4, T5, T6, T7, T8|0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_Archetype_for_Tuple!(TupleIter10|T0, T1, T2, T3, T4, T5, T6, T7, T8, T9|0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
impl_Archetype_for_Tuple!(TupleIter11|T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10|0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_Archetype_for_Tuple!(TupleIter12|T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11|0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
impl_Archetype_for_Tuple!(TupleIter13|T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12|0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_Archetype_for_Tuple!(TupleIter14|T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13|0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
impl_Archetype_for_Tuple!(TupleIter15|T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14|0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
impl_Archetype_for_Tuple!(TupleIter16|T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15|0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);