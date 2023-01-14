// -------------------------
//
// Cwago.
//
// cwago/cwago_comp/src/iter.rs
// (C) 2023 CwagoCommunity.
//
//! イテレータを提供します。
// =========================

use std::{marker::PhantomData, any::TypeId};

use crate::data::Data;

pub struct Iter<T> {
    _ph: PhantomData<T>,
}
pub struct IterMut<T> {
    _ph: PhantomData<T>,
}

pub struct Req {
    req: ReqElem,
}
enum ReqElem {
    Type(ReqType),
    And(Vec<ReqElem>),
    Or(Vec<ReqElem>),
    With(ReqType),
}
enum ReqType {
    Ref(TypeId),
    Mut(TypeId),
}

pub trait Archetype {
    fn req() -> Req;
}

trait ArchetypeElement {
    fn req() -> ReqElem;
}

pub struct Or<T> {
    _ph: PhantomData<T>,
}
pub struct With<T> {
    _ph: PhantomData<T>,
}

impl<'a, 'de, D> ArchetypeElement for &'a D
where D: Data<'de>
{
    fn req() -> ReqElem {
        ReqElem::Type(ReqType::Ref(TypeId::of::<D>()))
    }
}

impl<'a, 'de, D> ArchetypeElement for &'a mut D
where D: Data<'de>
{
    fn req() -> ReqElem {
        ReqElem::Type(ReqType::Mut(TypeId::of::<D>()))
    }
}

impl<'a, 'de, D> ArchetypeElement for With<&'a D>
where D: Data<'de>
{
    fn req() -> ReqElem {
        ReqElem::With(ReqType::Ref(TypeId::of::<D>()))
    }
}

impl<'a, 'de, D> ArchetypeElement for With<&'a mut D>
where D: Data<'de>
{
    fn req() -> ReqElem {
        ReqElem::With(ReqType::Mut(TypeId::of::<D>()))
    }
}

// サンプル
impl<T0, T1> ArchetypeElement for (T0, T1,)
where T0: ArchetypeElement, T1: ArchetypeElement 
{
    fn req() -> ReqElem {
        ReqElem::And(vec![
            T0::req(),
            T1::req(),
        ])
    }
}

// サンプル
impl<T0, T1> ArchetypeElement for Or<(T0, T1,)>
where T0: ArchetypeElement, T1: ArchetypeElement 
{
    fn req() -> ReqElem {
        ReqElem::Or(vec![
            T0::req(),
            T1::req(),
        ])
    }
}