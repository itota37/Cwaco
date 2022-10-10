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
    BufferPattern
};

/// コンポーネントデータを定義するトレイトです。
pub trait Data: Clone + 'static {}

/// データを一時保存する構造体です。
pub(crate) struct Datas {

    /// 型別のデータバッファです。
    datas: Vec<(Type, Vec<u8>)>,
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

// -----
//
// TupleIterX
//
// =====