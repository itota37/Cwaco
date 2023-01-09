// -------------------------
//
// Cwago.
//
// cwago/cwago_serde/src/ser.rs
// (C) 2022 CwagoCommunity.
//
//! シリアライズを提供します。
// =========================

use std::marker::PhantomData;

use serde::ser::{self, SerializeSeq};

use super::{
    pack::Pack,
    error::Error
};

/// 非型指定シリアライズトレイトです。
pub trait UngenericizedSerialize{
    fn ungenericized_serialize(&self, v: &mut dyn UngenericizedSerializer) -> Result<Ok, Error>;
}
impl ser::Serialize for dyn UngenericizedSerialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.serialize(serializer)
    }
}

/// 非型指定シリアライザトレイトです。
pub trait UngenericizedSerializer {

    fn ungenericized_serialize_bool(self, v: bool) -> Result<Ok, Error>;
    fn ungenericized_serialize_i8(self, v: i8) -> Result<Ok, Error>;
    fn ungenericized_serialize_i16(self, v: i16) -> Result<Ok, Error>;
    fn ungenericized_serialize_i32(self, v: i32) -> Result<Ok, Error>;
    fn ungenericized_serialize_i64(self, v: i64) -> Result<Ok, Error>;
    fn ungenericized_serialize_u8(self, v: u8) -> Result<Ok, Error>;
    fn ungenericized_serialize_u16(self, v: u16) -> Result<Ok, Error>;
    fn ungenericized_serialize_u32(self, v: u32) -> Result<Ok, Error>;
    fn ungenericized_serialize_u64(self, v: u64) -> Result<Ok, Error>;
    fn ungenericized_serialize_f32(self, v: f32) -> Result<Ok, Error>;
    fn ungenericized_serialize_f64(self, v: f64) -> Result<Ok, Error>;
    fn ungenericized_serialize_char(self, v: char) -> Result<Ok, Error>;
    fn ungenericized_serialize_str(self, v: &str) -> Result<Ok, Error>;
    fn ungenericized_serialize_bytes(self, v: &[u8]) -> Result<Ok, Error>;
    fn ungenericized_serialize_none(self) -> Result<Ok, Error>;
    fn ungenericized_serialize_some(
        self, 
        value: &dyn UngenericizedSerialize
    ) -> Result<Ok, Error>;
    fn ungenericized_serialize_unit(self) -> Result<Ok, Error>;
    fn ungenericized_serialize_unit_struct(
        self,
        name: &'static str
    ) -> Result<Ok, Error>;
    fn ungenericized_serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str
    ) -> Result<Ok, Error>;
    fn ungenericized_serialize_newtype_struct(
        self,
        name: &'static str,
        value: &dyn UngenericizedSerialize
    ) -> Result<Ok, Error>;
    fn ungenericized_serialize_newtype_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &dyn UngenericizedSerialize
    ) -> Result<Ok, Error>;
    fn ungenericized_serialize_seq(
        self,
        len: Option<usize>
    ) -> Result<UngenericizedSerializeSeq, Error>;
    fn ungenericized_serialize_tuple(
        self,
        len: usize
    ) -> Result<UngenericizedSerializeTuple, Error>;
    fn ungenericized_serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize
    ) -> Result<UngenericizedSerializeTupleStruct, Error>;
    fn ungenericized_serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize
    ) -> Result<UngenericizedSerializeTupleVariant, Error>;
    fn ungenericized_serialize_map(
        self,
        len: Option<usize>
    ) -> Result<UngenericizedSerializeMap, Error>;
    fn ungenericized_serialize_struct(
        self,
        name: &'static str,
        len: usize
    ) -> Result<UngenericizedSerializeStruct, Error>;
    fn ungenericized_serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize
    ) -> Result<UngenericizedSerializeStructVariantfv , Error>;
}

/// UngenericizedSerializerのOkの型です。
pub(crate) struct Ok {
    data: Pack
}
impl Ok {
    pub(crate) fn new<T>(value: T) -> Self {
        Ok { data: Pack::from(value) }
    }

    pub(crate) fn take<T>(self) -> T {
        self.take()
    }
}

/// UngenericizedSerializerのSerializeSeqの型です。
pub struct UngenericizedSerializeSeq<'a> {
    data: Pack,
    serialize_element: fn(&mut Pack, &dyn UngenericizedSerialize) -> Result<(), Error>,
    end: fn(Pack) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn UngenericizedSerializer>
}
impl<'a> UngenericizedSerializeSeq<'a> {
    /// 生成します。
    /// 
    /// # 戻り値
    /// 
    /// インスタンスです。
    /// 
    fn new<S>(seq: S) -> Self
    where
        S: ser::SerializeSeq {
        UngenericizedSerializeSeq { 
            data: Pack::from(seq), 
            serialize_element: |seq, v|{
                unsafe{seq.unpack_mut::<S>()}
                    .serialize_element(v)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            end: |seq|{
                unsafe{seq.unpack::<S>()}
                    .end()
                    .map(Ok::new)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            lifetime: PhantomData 
        }    
    }
}
impl<'a> ser::SerializeSeq for UngenericizedSerializeSeq<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize {
        (self.serialize_element)(&mut self.data, &value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.end)(self.data)
    }
}

/// UngenericizedSerializerのSerializeTupleの型です。
pub struct UngenericizedSerializeTuple<'a> {
    data: Pack,
    serialize_element: fn(&mut Pack, &dyn UngenericizedSerialize) -> Result<(), Error>,
    end: fn(Pack) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn UngenericizedSerializer>
}
impl<'a> UngenericizedSerializeTuple<'a> {
    /// 生成します。
    /// 
    /// # 戻り値
    /// 
    /// インスタンスです。
    /// 
    fn new<S>(tup: S) -> Self
    where
        S: ser::SerializeTuple {
        UngenericizedSerializeTuple { 
            data: Pack::from(tup), 
            serialize_element: |tup, v|{
                unsafe{tup.unpack_mut::<S>()}
                    .serialize_element(v)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            end: |tup|{
                unsafe{tup.unpack::<S>()}
                    .end()
                    .map(Ok::new)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            lifetime: PhantomData 
        }    
    }
}
impl<'a> ser::SerializeTuple for UngenericizedSerializeTuple<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        (self.serialize_element)(&mut self.data, &value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.end)(self.data)
    }
}

/// UngenericizedSerializerのSerializeTupleStructの型です。
pub struct UngenericizedSerializeTupleStruct<'a> {
    data: Pack,
    serialize_field: fn(&mut Pack, &dyn UngenericizedSerialize) -> Result<(), Error>,
    end: fn(Pack) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn UngenericizedSerializer>
}
impl<'a> UngenericizedSerializeTupleStruct<'a> {
    /// 生成します。
    /// 
    /// # 戻り値
    /// 
    /// インスタンスです。
    /// 
    fn new<S>(stru: S) -> Self
    where
        S: ser::SerializeTupleStruct {
        UngenericizedSerializeTupleStruct { 
            data: Pack::from(stru), 
            serialize_field: |stru, v|{
                unsafe{stru.unpack_mut::<S>()}
                    .serialize_field(v)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            end: |stru|{
                unsafe{stru.unpack::<S>()}
                    .end()
                    .map(Ok::new)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            lifetime: PhantomData 
        }    
    }
}
impl<'a> ser::SerializeTupleStruct for UngenericizedSerializeTupleStruct<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        (self.serialize_field)(&mut self.data, &value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.end)(self.data)
    }
}

/// UngenericizedSerializerのSerializeTupleVariantの型です。
pub struct UngenericizedSerializeTupleVariant<'a> {
    data: Pack,
    serialize_field: fn(&mut Pack, &dyn UngenericizedSerialize) -> Result<(), Error>,
    end: fn(Pack) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn UngenericizedSerializer>
}
impl<'a> UngenericizedSerializeTupleVariant<'a> {
    /// 生成します。
    /// 
    /// # 戻り値
    /// 
    /// インスタンスです。
    /// 
    fn new<S>(var: S) -> Self
    where
        S: ser::SerializeTupleVariant {
        UngenericizedSerializeTupleVariant { 
            data: Pack::from(var), 
            serialize_field: |var, v|{
                unsafe{var.unpack_mut::<S>()}
                    .serialize_field(v)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            end: |var|{
                unsafe{var.unpack::<S>()}
                    .end()
                    .map(Ok::new)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            lifetime: PhantomData 
        }    
    }
}
impl<'a> ser::SerializeTupleVariant for UngenericizedSerializeTupleVariant<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        (self.serialize_field)(&mut self.data, &value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.end)(self.data)
    }
}

/// UngenericizedSerializerのSerializeMapの型です。
pub struct UngenericizedSerializeMap<'a> {
    data: Pack,
    serialize_key: fn(&mut Pack, &dyn UngenericizedSerialize) -> Result<(), Error>,
    serialize_value: fn(&mut Pack, &dyn UngenericizedSerialize) -> Result<(), Error>,
    serialize_entry: fn(&mut Pack, &dyn UngenericizedSerialize, &dyn UngenericizedSerialize) -> Result<(), Error>,
    end: fn(Pack) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn UngenericizedSerializer>,
}
impl<'a> UngenericizedSerializeMap<'a> {
    /// 生成します。
    /// 
    /// # 戻り値
    /// 
    /// インスタンスです。
    /// 
    fn new<T>(data: T) -> Self
    where
        T: ser::SerializeMap {
        UngenericizedSerializeMap {
            data: Pack::from(data),
            serialize_key: |data, v| -> Result<(), Error> {
                unsafe{data.unpack_mut::<T>()}.serialize_key(v).map_err(<Error as ser::Error>::custom)
            },
            serialize_value: |data, v| -> Result<(), Error> {
                unsafe{data.unpack_mut::<T>()}.serialize_value(v).map_err(<Error as ser::Error>::custom) 
            },
            serialize_entry: |data,k ,v| -> Result<(), Error> {
                unsafe{data.unpack_mut::<T>()}.serialize_entry(k, v).map_err(<Error as ser::Error>::custom)
            },
            end: |data| -> Result<Ok, Error> {
                unsafe{data.unpack::<T>()}.end().map(Ok::new).map_err(<Error as ser::Error>::custom)
            },
            lifetime: PhantomData,
        }
    }
}
impl<'a> ser::SerializeMap for UngenericizedSerializeMap<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        (self.serialize_key)(&mut self.data, &key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        (self.serialize_value)(&mut self.data, &value)
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Error>
    where
        K: ?Sized + serde::Serialize,
        V: ?Sized + serde::Serialize,
    {
        (self.serialize_entry)(&mut self.data, &key, &value)
    }

    fn end(self) -> Result<Ok, Error> {
        (self.end)(self.data)
    }
}

/// UngenericizedSerializerのSerializeStructの型です。
pub struct UngenericizedSerializeStruct<'a> {
    data: Pack,
    serialize_field: fn(&mut Pack, &'static str, &dyn UngenericizedSerialize) -> Result<(), Error>,
    end: fn(Pack) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn UngenericizedSerializer>
}
impl<'a> UngenericizedSerializeStruct<'a> {
    /// 生成します。
    /// 
    /// # 戻り値
    /// 
    /// インスタンスです。
    /// 
    fn new<S>(var: S) -> Self
    where
        S: ser::SerializeStruct {
        UngenericizedSerializeStruct { 
            data: Pack::from(var), 
            serialize_field: |var, k, v|{
                unsafe{var.unpack_mut::<S>()}
                    .serialize_field(k, v)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            end: |var|{
                unsafe{var.unpack::<S>()}
                    .end()
                    .map(Ok::from)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            lifetime: PhantomData 
        }    
    }
}
impl<'a> ser::SerializeStruct for UngenericizedSerializeStruct<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        (self.serialize_field)(&mut self.data, &key, &value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.end)(self.data)
    }
}

/// UngenericizedSerializerのSerializeStructVariantの型です。
pub struct UngenericizedSerializeStructVariant<'a> {
    data: Pack,
    serialize_field: fn(&mut Pack, &'static str, &dyn UngenericizedSerialize) -> Result<(), Error>,
    end: fn(Pack) -> Result<Ok, Error>,
    lifetime: PhantomData<&'a dyn UngenericizedSerializer>
}
impl<'a> UngenericizedSerializeStructVariant<'a> {
    /// 生成します。
    /// 
    /// # 戻り値
    /// 
    /// インスタンスです。
    /// 
    fn new<S>(var: S) -> Self
    where
        S: ser::SerializeStructVariant {
        UngenericizedSerializeStructVariant { 
            data: Pack::from(var), 
            serialize_field: |var, k, v|{
                unsafe{var.unpack_mut::<S>()}
                    .serialize_field(k, v)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            end: |var|{
                unsafe{var.unpack::<S>()}
                    .end()
                    .map(Ok::from)
                    .map_err(<Error as ser::Error>::custom)
            }, 
            lifetime: PhantomData 
        }    
    }
}
impl<'a> ser::SerializeStructVariant for UngenericizedSerializeStructVariant<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        (self.serialize_field)(&mut self.data, &key, &value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.end)(self.data)
    }
}